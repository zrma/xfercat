#![cfg(unix)]

use std::{
    fs,
    io::Write,
    net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use tempfile::{TempDir, tempdir};
use xfercat::{
    domain::ConnectionProfile,
    sftp::{ConnectionOptions, SftpSession},
};

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

#[cfg(target_os = "macos")]
#[test]
#[ignore = "requires local sshd, ssh-keygen, and script executables"]
fn tui_connects_navigates_remote_and_closes_the_fixture_session() {
    let fixture = SshdFixture::start();
    let binary = env!("CARGO_BIN_EXE_xfercat");
    let mut child = Command::new("/usr/bin/script")
        .args([
            "-q",
            "/dev/null",
            "/bin/sh",
            "-c",
            "stty columns 110 rows 32; exec \"$1\" --ssh-config \"$2\"",
            "xfercat-fixture-shell",
            binary,
            fixture.config.to_str().expect("fixture config is UTF-8"),
        ])
        .env("HOME", fixture.root.path())
        .env("XDG_STATE_HOME", "/tmp")
        .current_dir(fixture.root.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("start fixture TUI in a PTY");
    let mut input = child.stdin.take().expect("fixture TUI stdin");
    thread::sleep(Duration::from_millis(150));
    input.write_all(b"\r").expect("select fixture profile");
    input.flush().expect("flush fixture profile selection");
    thread::sleep(Duration::from_millis(350));
    input.write_all(b"\t").expect("focus fixture remote pane");
    input.flush().expect("flush fixture focus");
    thread::sleep(Duration::from_millis(100));
    input.write_all(b"\r").expect("open fixture remote child");
    input.flush().expect("flush fixture child navigation");
    thread::sleep(Duration::from_millis(150));
    input.write_all(b"\x7f").expect("return to fixture parent");
    input.flush().expect("flush fixture parent navigation");
    thread::sleep(Duration::from_millis(150));
    input.write_all(b"q").expect("quit fixture TUI");
    drop(input);

    let output = child.wait_with_output().expect("wait for fixture TUI");
    assert!(output.status.success(), "fixture TUI must exit cleanly");
    let screen = String::from_utf8_lossy(&output.stdout);
    assert!(
        screen.contains("[REMOTE] xfercat-fixture:"),
        "fixture TUI must render the connected remote pane"
    );
    assert!(
        screen.contains("packages/"),
        "fixture TUI must render the remote directory"
    );
}

struct SshdFixture {
    root: TempDir,
    config: PathBuf,
    known_hosts: PathBuf,
    control_directory: PathBuf,
    child: Child,
}

impl SshdFixture {
    fn start() -> Self {
        let root = tempdir().expect("create fixture root");
        let remote = root.path().join("remote");
        let control_directory = PathBuf::from("/tmp");
        fs::create_dir(&remote).expect("create fixture remote root");
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
            root,
            config,
            known_hosts,
            control_directory,
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
