use std::{
    cmp::Ordering,
    io,
    path::{Path, PathBuf},
    time::Duration,
};

use futures_util::StreamExt;
use openssh::{ControlPersist, KnownHosts, SessionBuilder};
use openssh_sftp_client::{Sftp, SftpOptions};

use crate::domain::{
    Authentication, BrowserEntry, ConnectionProfile, ConnectionProfileSource, EntryKind,
};

const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemoteDirectory {
    pub path: String,
    pub entries: Vec<BrowserEntry>,
    pub skipped_entries: usize,
}

impl RemoteDirectory {
    pub fn status(&self) -> String {
        if self.skipped_entries == 0 {
            format!("Remote directory loaded: {} item(s).", self.entries.len())
        } else {
            format!(
                "Remote directory loaded: {} item(s); {} unsafe or unreadable entry(s) skipped.",
                self.entries.len(),
                self.skipped_entries
            )
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConnectionOptions {
    pub config_file: Option<PathBuf>,
    pub known_hosts_file: Option<PathBuf>,
    pub control_directory: Option<PathBuf>,
    pub connect_timeout: Duration,
}

impl Default for ConnectionOptions {
    fn default() -> Self {
        Self {
            config_file: None,
            known_hosts_file: None,
            control_directory: None,
            connect_timeout: DEFAULT_CONNECT_TIMEOUT,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SftpFailureKind {
    InvalidProfile,
    Authentication,
    HostVerification,
    Connection,
    RemoteFilesystem,
    InvalidRemotePath,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SftpFailure {
    pub kind: SftpFailureKind,
    pub retryable: bool,
}

impl SftpFailure {
    pub const fn status(self) -> &'static str {
        match self.kind {
            SftpFailureKind::InvalidProfile => "Connection profile is not usable for SFTP.",
            SftpFailureKind::Authentication => {
                "SSH authentication failed; check the selected agent or key reference."
            }
            SftpFailureKind::HostVerification => {
                "SSH host verification failed; update known_hosts outside xfercat."
            }
            SftpFailureKind::Connection => "SSH connection could not be established.",
            SftpFailureKind::RemoteFilesystem => "Remote directory could not be read.",
            SftpFailureKind::InvalidRemotePath => "Remote path could not be represented safely.",
        }
    }
}

#[derive(Debug)]
pub struct SftpSession {
    client: Sftp,
}

impl SftpSession {
    pub async fn connect(profile: &ConnectionProfile) -> Result<Self, SftpFailure> {
        Self::connect_with_options(profile, &ConnectionOptions::default()).await
    }

    pub async fn connect_with_options(
        profile: &ConnectionProfile,
        options: &ConnectionOptions,
    ) -> Result<Self, SftpFailure> {
        let (builder, destination) = connection_builder(profile, options)?;
        let session = builder
            .connect(destination)
            .await
            .map_err(classify_openssh_error)?;
        let client = Sftp::from_session(session, SftpOptions::default())
            .await
            .map_err(|_| failure(SftpFailureKind::Connection, true))?;
        Ok(Self { client })
    }

    pub async fn home_directory(&self) -> Result<RemoteDirectory, SftpFailure> {
        self.read_directory(".").await
    }

    pub async fn read_directory(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<RemoteDirectory, SftpFailure> {
        let mut fs = self.client.fs();
        let canonical = fs
            .canonicalize(path)
            .await
            .map_err(|_| failure(SftpFailureKind::RemoteFilesystem, true))?;
        let directory = canonical
            .to_str()
            .filter(|path| is_safe_remote_path(path))
            .ok_or_else(|| failure(SftpFailureKind::InvalidRemotePath, false))?
            .to_owned();
        let stream = fs
            .open_dir(&canonical)
            .await
            .map_err(|_| failure(SftpFailureKind::RemoteFilesystem, true))?
            .read_dir();
        futures_util::pin_mut!(stream);
        let mut entries = Vec::new();
        let mut skipped_entries = 0;
        while let Some(candidate) = stream.next().await {
            let candidate = match candidate {
                Ok(candidate) => candidate,
                Err(_) => {
                    skipped_entries += 1;
                    continue;
                }
            };
            let Some(name) = candidate.filename().to_str() else {
                skipped_entries += 1;
                continue;
            };
            if !is_safe_entry_name(name) {
                if !matches!(name, "." | "..") {
                    skipped_entries += 1;
                }
                continue;
            }
            let metadata = candidate.metadata();
            let Some(file_type) = metadata.file_type() else {
                skipped_entries += 1;
                continue;
            };
            let kind = if file_type.is_dir() {
                EntryKind::Directory
            } else if file_type.is_file() {
                EntryKind::File
            } else {
                skipped_entries += 1;
                continue;
            };
            let size = if kind == EntryKind::File {
                match metadata.len() {
                    Some(size) => Some(size),
                    None => {
                        skipped_entries += 1;
                        continue;
                    }
                }
            } else {
                None
            };
            entries.push(BrowserEntry {
                name: name.to_owned(),
                path: join_remote_path(&directory, name),
                kind,
                size,
            });
        }
        entries.sort_by(compare_entries);

        Ok(RemoteDirectory {
            path: directory,
            entries,
            skipped_entries,
        })
    }

    pub async fn close(self) -> Result<(), SftpFailure> {
        self.client
            .close()
            .await
            .map_err(|_| failure(SftpFailureKind::Connection, true))
    }
}

fn connection_builder(
    profile: &ConnectionProfile,
    options: &ConnectionOptions,
) -> Result<(SessionBuilder, String), SftpFailure> {
    if !is_safe_destination(&profile.host) {
        return Err(failure(SftpFailureKind::InvalidProfile, false));
    }

    let mut builder = SessionBuilder::default();
    builder
        .known_hosts_check(KnownHosts::Strict)
        .connect_timeout(options.connect_timeout)
        .control_persist(ControlPersist::ClosedAfterInitialConnection);
    if let Some(path) = &options.config_file {
        builder.config_file(path);
    }
    if let Some(path) = &options.known_hosts_file {
        builder.user_known_hosts_file(path);
    }
    if let Some(path) = &options.control_directory {
        builder.control_directory(path);
    }

    match (&profile.source, &profile.authentication) {
        (ConnectionProfileSource::OpenSshConfig, Authentication::OpenSshConfig) => {}
        (ConnectionProfileSource::Manual, Authentication::SshAgent) => {
            if !is_safe_user(&profile.user) {
                return Err(failure(SftpFailureKind::InvalidProfile, false));
            }
            builder.user(profile.user.clone());
        }
        (ConnectionProfileSource::Manual, Authentication::KeyReference(reference)) => {
            if !is_safe_user(&profile.user) || !is_safe_key_reference(reference) {
                return Err(failure(SftpFailureKind::InvalidProfile, false));
            }
            builder.user(profile.user.clone()).keyfile(reference);
        }
        _ => return Err(failure(SftpFailureKind::InvalidProfile, false)),
    }

    Ok((builder, profile.host.clone()))
}

fn classify_openssh_error(error: openssh::Error) -> SftpFailure {
    if let openssh::Error::Connect(source) = &error {
        if source.kind() == io::ErrorKind::PermissionDenied {
            return failure(SftpFailureKind::Authentication, false);
        }
        let diagnostic = source.to_string().to_ascii_lowercase();
        if diagnostic.contains("host key verification failed")
            || diagnostic.contains("remote host identification has changed")
        {
            return failure(SftpFailureKind::HostVerification, false);
        }
    }
    failure(SftpFailureKind::Connection, true)
}

const fn failure(kind: SftpFailureKind, retryable: bool) -> SftpFailure {
    SftpFailure { kind, retryable }
}

fn is_safe_destination(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('-')
        && !value.chars().any(char::is_whitespace)
        && !value.chars().any(char::is_control)
}

fn is_safe_user(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('-')
        && !value.chars().any(char::is_whitespace)
        && !value.chars().any(char::is_control)
}

fn is_safe_key_reference(value: &str) -> bool {
    !value.is_empty() && !value.chars().any(char::is_control)
}

fn is_safe_remote_path(path: &str) -> bool {
    path.starts_with('/') && !path.chars().any(char::is_control)
}

fn is_safe_entry_name(name: &str) -> bool {
    !name.is_empty()
        && !matches!(name, "." | "..")
        && !name.contains('/')
        && !name.chars().any(char::is_control)
}

fn join_remote_path(directory: &str, name: &str) -> String {
    if directory == "/" {
        format!("/{name}")
    } else {
        format!("{}/{name}", directory.trim_end_matches('/'))
    }
}

fn compare_entries(left: &BrowserEntry, right: &BrowserEntry) -> Ordering {
    match (left.kind, right.kind) {
        (EntryKind::Directory, EntryKind::File) => Ordering::Less,
        (EntryKind::File, EntryKind::Directory) => Ordering::Greater,
        _ => left
            .name
            .to_ascii_lowercase()
            .cmp(&right.name.to_ascii_lowercase())
            .then_with(|| left.name.cmp(&right.name)),
    }
}

#[cfg(test)]
mod tests {
    use std::{io, path::PathBuf, time::Duration};

    use super::{
        ConnectionOptions, SftpFailureKind, classify_openssh_error, connection_builder,
        is_safe_entry_name, is_safe_remote_path,
    };
    use crate::domain::{Authentication, ConnectionProfile, ConnectionProfileSource, Protocol};

    #[test]
    fn resolves_imported_alias_without_copying_effective_config() {
        let profile = ConnectionProfile::open_ssh("fixture-host");
        let options = ConnectionOptions {
            config_file: Some(PathBuf::from("fixture-config")),
            known_hosts_file: Some(PathBuf::from("fixture-known-hosts")),
            control_directory: Some(PathBuf::from("fixture-control")),
            connect_timeout: Duration::from_secs(3),
        };

        let (builder, destination) =
            connection_builder(&profile, &options).expect("valid imported profile");

        assert_eq!(destination, "fixture-host");
        assert_eq!(builder.get_user(), None);
    }

    #[test]
    fn resolves_manual_agent_and_key_profiles_without_secret_material() {
        let mut profile = manual_profile(Authentication::SshAgent);
        let (builder, destination) = connection_builder(&profile, &ConnectionOptions::default())
            .expect("valid agent profile");
        assert_eq!(destination, "fixture.invalid");
        assert_eq!(builder.get_user(), Some("operator"));

        profile.authentication = Authentication::KeyReference("keys/fixture_ed25519".into());
        let (builder, _) =
            connection_builder(&profile, &ConnectionOptions::default()).expect("valid key profile");
        assert_eq!(builder.get_user(), Some("operator"));
    }

    #[test]
    fn rejects_synthetic_mismatched_and_option_like_profiles() {
        let mut profile = manual_profile(Authentication::SshAgent);
        profile.source = ConnectionProfileSource::Synthetic;
        assert_eq!(
            connection_builder(&profile, &ConnectionOptions::default())
                .expect_err("synthetic profile must fail")
                .kind,
            SftpFailureKind::InvalidProfile
        );

        profile.source = ConnectionProfileSource::Manual;
        profile.authentication = Authentication::OpenSshConfig;
        assert_eq!(
            connection_builder(&profile, &ConnectionOptions::default())
                .expect_err("mismatched auth must fail")
                .kind,
            SftpFailureKind::InvalidProfile
        );

        profile.authentication = Authentication::SshAgent;
        profile.host = "-oProxyCommand=unsafe".into();
        assert_eq!(
            connection_builder(&profile, &ConnectionOptions::default())
                .expect_err("option-like destination must fail")
                .kind,
            SftpFailureKind::InvalidProfile
        );
    }

    #[test]
    fn accepts_only_absolute_safe_remote_paths_and_leaf_names() {
        assert!(is_safe_remote_path("/srv/incoming"));
        assert!(!is_safe_remote_path("srv/incoming"));
        assert!(!is_safe_remote_path("/srv/in\ncoming"));
        assert!(is_safe_entry_name("payload.bin"));
        assert!(!is_safe_entry_name(".."));
        assert!(!is_safe_entry_name("nested/name"));
    }

    #[test]
    fn classifies_connection_failures_without_exposing_raw_diagnostics() {
        let authentication = classify_openssh_error(openssh::Error::Connect(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "fixture credential rejected",
        )));
        let host = classify_openssh_error(openssh::Error::Connect(io::Error::new(
            io::ErrorKind::ConnectionAborted,
            "Host key verification failed for fixture.invalid",
        )));

        assert_eq!(authentication.kind, SftpFailureKind::Authentication);
        assert_eq!(host.kind, SftpFailureKind::HostVerification);
        assert!(!authentication.status().contains("fixture"));
        assert!(!host.status().contains("fixture.invalid"));
    }

    fn manual_profile(authentication: Authentication) -> ConnectionProfile {
        ConnectionProfile {
            id: "fixture".into(),
            label: "fixture".into(),
            source: ConnectionProfileSource::Manual,
            protocol: Protocol::Sftp,
            user: "operator".into(),
            host: "fixture.invalid".into(),
            authentication,
        }
    }
}
