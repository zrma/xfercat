#![cfg(unix)]

use std::{
    fs,
    net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::LazyLock,
    thread,
    time::{Duration, Instant},
};

use tempfile::{TempDir, tempdir};
use xfercat::{
    domain::{
        ConflictPolicy, ConnectionProfile, DestinationExpectation, Endpoint, EntryKind,
        TransferDirection, TransferPlanItem, TransferState,
    },
    executor,
    sftp::{ConnectionOptions, SftpSession},
    transfer_io,
    transport::{TransportFailureKind, TransportOutcome, TransportRequest, TransportSkipReason},
};

static FIXTURE_PAYLOAD: LazyLock<Vec<u8>> = LazyLock::new(|| {
    (0..(64 * 1024 * 3 + 17))
        .map(|index| ((index * 31) % 251) as u8)
        .collect()
});

#[test]
#[ignore = "requires local sshd and ssh-keygen executables"]
fn browses_an_isolated_strict_sftp_server() {
    let fixture = SshdFixture::start();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build fixture runtime");

    runtime.block_on(async {
        let session = SftpSession::connect_with_options(
            &ConnectionProfile::open_ssh("xfercat-fixture"),
            &fixture.options(),
        )
        .await
        .expect("connect to isolated SFTP fixture");

        let home = session.home_directory().await.expect("read fixture home");
        assert_eq!(home.entries.len(), 3);
        assert_eq!(home.entries[0].name, "packages");
        assert_eq!(home.entries[1].name, "Alpha.txt");
        assert_eq!(home.entries[1].size, Some(5));
        assert_eq!(home.entries[2].name, "beta.txt");
        assert_eq!(home.entries[2].size, Some(4));
        assert_eq!(home.skipped_entries, 1);

        let child = session
            .read_directory(&home.entries[0].path)
            .await
            .expect("read fixture child");
        assert!(child.entries.is_empty());
        assert!(child.path.ends_with("/packages"));

        session.close().await.expect("close fixture session");
    });
}

#[test]
#[ignore = "requires local sshd and ssh-keygen executables"]
fn transfers_upload_download_conflicts_and_stale_states_safely() {
    let fixture = SshdFixture::start();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build fixture runtime");

    runtime.block_on(async {
        let session = SftpSession::connect_with_options(
            &ConnectionProfile::open_ssh("xfercat-fixture"),
            &fixture.options(),
        )
        .await
        .expect("connect to isolated transfer fixture");
        let home = session
            .home_directory()
            .await
            .expect("read transfer fixture home");
        let local_source = fixture.local.join("payload.bin");
        let remote_upload = format!("{}/uploaded.bin", home.path);
        let upload = transfer_io::execute_request(
            &session,
            &request(
                1,
                TransferDirection::Upload,
                path_string(&local_source),
                remote_upload.clone(),
                FIXTURE_PAYLOAD.len() as u64,
                DestinationExpectation::Missing,
                ConflictPolicy::Ask,
            ),
        )
        .await;
        assert_eq!(
            upload,
            TransportOutcome::Succeeded {
                bytes_transferred: FIXTURE_PAYLOAD.len() as u64
            }
        );
        assert_eq!(
            fs::read(fixture.remote.join("uploaded.bin")).expect("read uploaded fixture"),
            FIXTURE_PAYLOAD.as_slice()
        );

        let local_download = fixture.local.join("downloaded.bin");
        let download = transfer_io::execute_request(
            &session,
            &request(
                2,
                TransferDirection::Download,
                remote_upload,
                path_string(&local_download),
                FIXTURE_PAYLOAD.len() as u64,
                DestinationExpectation::Missing,
                ConflictPolicy::Ask,
            ),
        )
        .await;
        assert_eq!(
            download,
            TransportOutcome::Succeeded {
                bytes_transferred: FIXTURE_PAYLOAD.len() as u64
            }
        );
        assert_eq!(
            fs::read(&local_download).expect("read downloaded fixture"),
            FIXTURE_PAYLOAD.as_slice()
        );

        let existing_remote = format!("{}/beta.txt", home.path);
        let existing = DestinationExpectation::Existing {
            kind: EntryKind::File,
            size: Some(4),
        };
        let ask = transfer_io::execute_request(
            &session,
            &request(
                3,
                TransferDirection::Upload,
                path_string(&local_source),
                existing_remote.clone(),
                FIXTURE_PAYLOAD.len() as u64,
                existing,
                ConflictPolicy::Ask,
            ),
        )
        .await;
        assert!(matches!(
            ask,
            TransportOutcome::Failed {
                kind: TransportFailureKind::DestinationConflict,
                ..
            }
        ));
        assert_eq!(fs::read(fixture.remote.join("beta.txt")).unwrap(), b"beta");

        let skip = transfer_io::execute_request(
            &session,
            &request(
                4,
                TransferDirection::Upload,
                path_string(&local_source),
                existing_remote.clone(),
                FIXTURE_PAYLOAD.len() as u64,
                existing,
                ConflictPolicy::Skip,
            ),
        )
        .await;
        assert_eq!(
            skip,
            TransportOutcome::Skipped {
                reason: TransportSkipReason::ConflictPolicy
            }
        );

        let overwrite = transfer_io::execute_request(
            &session,
            &request(
                5,
                TransferDirection::Upload,
                path_string(&local_source),
                existing_remote,
                FIXTURE_PAYLOAD.len() as u64,
                existing,
                ConflictPolicy::Overwrite,
            ),
        )
        .await;
        assert!(matches!(overwrite, TransportOutcome::Succeeded { .. }));
        assert_eq!(
            fs::read(fixture.remote.join("beta.txt")).unwrap(),
            FIXTURE_PAYLOAD.as_slice()
        );

        let stale_destination = transfer_io::execute_request(
            &session,
            &request(
                6,
                TransferDirection::Upload,
                path_string(&local_source),
                format!("{}/Alpha.txt", home.path),
                FIXTURE_PAYLOAD.len() as u64,
                DestinationExpectation::Missing,
                ConflictPolicy::Overwrite,
            ),
        )
        .await;
        assert!(matches!(
            stale_destination,
            TransportOutcome::Failed {
                kind: TransportFailureKind::StaleDestination,
                ..
            }
        ));
        let stale_source = transfer_io::execute_request(
            &session,
            &request(
                7,
                TransferDirection::Upload,
                path_string(&local_source),
                format!("{}/stale.bin", home.path),
                1,
                DestinationExpectation::Missing,
                ConflictPolicy::Ask,
            ),
        )
        .await;
        assert!(matches!(
            stale_source,
            TransportOutcome::Failed {
                kind: TransportFailureKind::StaleSource,
                ..
            }
        ));
        assert!(!fixture.remote.join("stale.bin").exists());
        assert_no_temporary_files(&fixture.remote);
        assert_no_temporary_files(&fixture.local);

        let mut plan = vec![
            plan_item(
                8,
                path_string(&local_source),
                format!("{}/plan-success.bin", home.path),
                FIXTURE_PAYLOAD.len() as u64,
            ),
            plan_item(
                9,
                path_string(&local_source),
                format!("{}/plan-stale.bin", home.path),
                1,
            ),
        ];
        let summary = executor::execute_live(&mut plan, "openssh:xfercat-fixture", &session).await;
        assert_eq!(summary.succeeded, 1);
        assert_eq!(summary.failed, 1);
        assert_eq!(plan[0].state, TransferState::Succeeded);
        assert_eq!(plan[1].state, TransferState::Failed);
        assert_eq!(
            fs::read(fixture.remote.join("plan-success.bin")).unwrap(),
            FIXTURE_PAYLOAD.as_slice()
        );
        assert!(!fixture.remote.join("plan-stale.bin").exists());
        assert_no_temporary_files(&fixture.remote);

        session
            .close()
            .await
            .expect("close transfer fixture session");
    });
}

struct SshdFixture {
    _root: TempDir,
    config: PathBuf,
    known_hosts: PathBuf,
    control_directory: PathBuf,
    local: PathBuf,
    remote: PathBuf,
    child: Child,
}

impl SshdFixture {
    fn start() -> Self {
        let root = tempdir().expect("create fixture root");
        let remote = root.path().join("remote");
        let local = root.path().join("local");
        let control_directory = PathBuf::from("/tmp");
        fs::create_dir(&remote).expect("create fixture remote root");
        fs::create_dir(&local).expect("create fixture local root");
        fs::write(local.join("payload.bin"), FIXTURE_PAYLOAD.as_slice())
            .expect("write local payload");
        fs::create_dir(remote.join("packages")).expect("create fixture directory");
        fs::write(remote.join("Alpha.txt"), b"alpha").expect("write fixture alpha");
        fs::write(remote.join("beta.txt"), b"beta").expect("write fixture beta");
        std::os::unix::fs::symlink(remote.join("beta.txt"), remote.join("beta-link"))
            .expect("create fixture symlink");

        let host_key = root.path().join("host_ed25519");
        let client_key = root.path().join("client_ed25519");
        generate_key(&host_key);
        generate_key(&client_key);
        let authorized_keys = root.path().join("authorized_keys");
        fs::copy(client_key.with_extension("pub"), &authorized_keys).expect("write authorized key");

        let port = available_port();
        let loopback = Ipv4Addr::LOCALHOST;
        let user = current_user();
        let ssh_directory = root.path().join(".ssh");
        fs::create_dir(&ssh_directory).expect("create fixture SSH directory");
        let config = ssh_directory.join("config");
        let known_hosts = root.path().join("known_hosts");
        let host_public =
            fs::read_to_string(host_key.with_extension("pub")).expect("read host public key");
        let mut fields = host_public.split_whitespace();
        let algorithm = fields.next().expect("host key algorithm");
        let encoded = fields.next().expect("host key payload");
        fs::write(
            &known_hosts,
            format!("[{loopback}]:{port} {algorithm} {encoded}\n"),
        )
        .expect("write fixture known hosts");
        fs::write(
            &config,
            format!(
                "Host xfercat-fixture\n  HostName {loopback}\n  Port {port}\n  User {user}\n  IdentityFile {}\n  IdentitiesOnly yes\n  UserKnownHostsFile {}\n  StrictHostKeyChecking yes\n",
                client_key.display(),
                known_hosts.display()
            ),
        )
        .expect("write fixture client config");

        let server_config = root.path().join("sshd_config");
        fs::write(
            &server_config,
            format!(
                "Port {port}\nListenAddress {loopback}\nHostKey {}\nAuthorizedKeysFile {}\nStrictModes no\nPasswordAuthentication no\nKbdInteractiveAuthentication no\nUsePAM no\nPidFile {}\nLogLevel QUIET\nSubsystem sftp internal-sftp\nForceCommand internal-sftp -d {}\n",
                host_key.display(),
                authorized_keys.display(),
                root.path().join("sshd.pid").display(),
                remote.display()
            ),
        )
        .expect("write fixture server config");

        let sshd = sshd_path();
        assert!(
            Command::new(&sshd)
                .args(["-t", "-f"])
                .arg(&server_config)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .expect("validate fixture sshd config")
                .success(),
            "fixture sshd config must be valid"
        );
        let child = Command::new(&sshd)
            .args(["-D", "-e", "-f"])
            .arg(&server_config)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("start fixture sshd");
        wait_for_server(port);

        Self {
            _root: root,
            config,
            known_hosts,
            control_directory,
            local,
            remote,
            child,
        }
    }

    fn options(&self) -> ConnectionOptions {
        ConnectionOptions {
            config_file: Some(self.config.clone()),
            known_hosts_file: Some(self.known_hosts.clone()),
            control_directory: Some(self.control_directory.clone()),
            connect_timeout: Duration::from_secs(3),
        }
    }
}

fn request(
    item_id: u64,
    direction: TransferDirection,
    source_path: String,
    destination_path: String,
    expected_size: u64,
    destination_expectation: DestinationExpectation,
    conflict_policy: ConflictPolicy,
) -> TransportRequest {
    let (source, destination) = match direction {
        TransferDirection::Upload => (
            Endpoint::local(source_path),
            Endpoint::remote("openssh:xfercat-fixture", "fixture", destination_path),
        ),
        TransferDirection::Download => (
            Endpoint::remote("openssh:xfercat-fixture", "fixture", source_path),
            Endpoint::local(destination_path),
        ),
    };
    TransportRequest {
        item_id,
        source,
        destination,
        direction,
        entry_kind: EntryKind::File,
        expected_size: Some(expected_size),
        destination_expectation,
        conflict_policy,
    }
}

fn plan_item(
    item_id: u64,
    source_path: String,
    destination_path: String,
    expected_size: u64,
) -> TransferPlanItem {
    TransferPlanItem {
        id: item_id,
        source: Endpoint::local(source_path),
        destination: Endpoint::remote("openssh:xfercat-fixture", "fixture", destination_path),
        direction: TransferDirection::Upload,
        entry_kind: EntryKind::File,
        expected_size: Some(expected_size),
        destination_expectation: DestinationExpectation::Missing,
        conflict_policy: ConflictPolicy::Ask,
        state: TransferState::Staged,
    }
}

fn path_string(path: &Path) -> String {
    path.to_str().expect("fixture path is UTF-8").to_owned()
}

fn assert_no_temporary_files(directory: &Path) {
    assert!(
        fs::read_dir(directory)
            .expect("read fixture directory")
            .filter_map(Result::ok)
            .all(|entry| !entry.file_name().to_string_lossy().contains(".xfercat-")),
        "temporary transfer files must be cleaned"
    );
}

impl Drop for SshdFixture {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn generate_key(path: &Path) {
    assert!(
        Command::new("ssh-keygen")
            .args(["-q", "-t", "ed25519", "-N", "", "-f"])
            .arg(path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("run ssh-keygen")
            .success(),
        "fixture key generation must succeed"
    );
}

fn sshd_path() -> PathBuf {
    ["/usr/sbin/sshd", "/usr/local/sbin/sshd", "/usr/bin/sshd"]
        .into_iter()
        .map(PathBuf::from)
        .find(|path| path.is_file())
        .expect("local sshd executable")
}

fn current_user() -> String {
    let output = Command::new("id")
        .arg("-un")
        .output()
        .expect("read fixture user");
    assert!(output.status.success(), "fixture user lookup must succeed");
    String::from_utf8(output.stdout)
        .expect("fixture user is UTF-8")
        .trim()
        .to_owned()
}

fn available_port() -> u16 {
    TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0))
        .expect("reserve fixture port")
        .local_addr()
        .expect("read fixture port")
        .port()
}

fn wait_for_server(port: u16) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        if TcpStream::connect(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port)).is_ok() {
            return;
        }
        thread::sleep(Duration::from_millis(20));
    }
    panic!("fixture sshd did not start");
}
