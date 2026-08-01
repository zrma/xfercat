use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConnectionProfile {
    pub id: String,
    pub label: String,
    pub source: ConnectionProfileSource,
    pub protocol: Protocol,
    pub user: String,
    pub host: String,
    pub authentication: Authentication,
}

impl ConnectionProfile {
    pub fn open_ssh(alias: impl Into<String>) -> Self {
        let alias = alias.into();
        Self {
            id: format!("openssh:{alias}"),
            label: alias.clone(),
            source: ConnectionProfileSource::OpenSshConfig,
            protocol: Protocol::Sftp,
            user: String::new(),
            host: alias,
            authentication: Authentication::OpenSshConfig,
        }
    }

    pub fn endpoint_summary(&self) -> String {
        match self.source {
            ConnectionProfileSource::OpenSshConfig => format!("Host {}", self.host),
            ConnectionProfileSource::Synthetic | ConnectionProfileSource::Manual => {
                format!("{}@{}", self.user, self.host)
            }
        }
    }

    pub const fn is_open_ssh(&self) -> bool {
        matches!(self.source, ConnectionProfileSource::OpenSshConfig)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectionProfileSource {
    Synthetic,
    Manual,
    OpenSshConfig,
}

impl fmt::Display for ConnectionProfileSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Synthetic => "Synthetic fixture",
            Self::Manual => "Manual process profile",
            Self::OpenSshConfig => "OpenSSH config",
        };
        formatter.write_str(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Protocol {
    Sftp,
}

impl fmt::Display for Protocol {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sftp => formatter.write_str("SFTP"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Authentication {
    SshAgent,
    KeyReference(String),
    OpenSshConfig,
}

impl Authentication {
    pub fn short_label(&self) -> String {
        match self {
            Self::SshAgent => "Agent".into(),
            Self::KeyReference(reference) => format!("Key:{reference}"),
            Self::OpenSshConfig => "OpenSSH".into(),
        }
    }
}

impl fmt::Display for Authentication {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SshAgent => formatter.write_str("SSH Agent"),
            Self::KeyReference(reference) => write!(formatter, "Key ref: {reference}"),
            Self::OpenSshConfig => formatter.write_str("OpenSSH policy"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntryKind {
    Directory,
    File,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrowserEntry {
    pub name: String,
    pub path: String,
    pub kind: EntryKind,
    pub size: Option<u64>,
}

impl BrowserEntry {
    pub fn display_name(&self) -> String {
        match self.kind {
            EntryKind::Directory => format!("{}/", self.name),
            EntryKind::File => self.name.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Endpoint {
    pub profile_id: Option<String>,
    pub label: String,
    pub path: String,
}

impl Endpoint {
    pub fn local(path: impl Into<String>) -> Self {
        Self {
            profile_id: None,
            label: "local".into(),
            path: path.into(),
        }
    }

    pub fn remote(
        profile_id: impl Into<String>,
        label: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        Self {
            profile_id: Some(profile_id.into()),
            label: label.into(),
            path: path.into(),
        }
    }

    pub fn display(&self) -> String {
        format!("{}:{}", self.label, self.path)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransferDirection {
    Upload,
    Download,
}

impl TransferDirection {
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Upload => "↑",
            Self::Download => "↓",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConflictPolicy {
    Ask,
    Overwrite,
    Skip,
    Rename,
}

impl ConflictPolicy {
    pub const fn next(self) -> Self {
        match self {
            Self::Ask => Self::Overwrite,
            Self::Overwrite => Self::Skip,
            Self::Skip => Self::Rename,
            Self::Rename => Self::Ask,
        }
    }
}

impl fmt::Display for ConflictPolicy {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Ask => "ASK",
            Self::Overwrite => "OVERWRITE",
            Self::Skip => "SKIP",
            Self::Rename => "RENAME",
        };
        formatter.write_str(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransferState {
    Staged,
}

impl fmt::Display for TransferState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Staged => formatter.write_str("STAGED"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferPlanItem {
    pub id: u64,
    pub source: Endpoint,
    pub destination: Endpoint,
    pub direction: TransferDirection,
    pub conflict_policy: ConflictPolicy,
    pub state: TransferState,
}
