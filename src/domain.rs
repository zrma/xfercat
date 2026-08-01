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
pub enum DestinationExpectation {
    Missing,
    Existing { kind: EntryKind, size: Option<u64> },
}

impl DestinationExpectation {
    pub const fn short_label(self) -> &'static str {
        match self {
            Self::Missing => "DEST:MISSING",
            Self::Existing {
                kind: EntryKind::Directory,
                ..
            } => "DEST:DIRECTORY",
            Self::Existing {
                kind: EntryKind::File,
                ..
            } => "DEST:FILE",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransferState {
    Staged,
    Running,
    Succeeded,
    Failed,
    Skipped,
    Cancelled,
}

impl fmt::Display for TransferState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Staged => formatter.write_str("STAGED"),
            Self::Running => formatter.write_str("RUNNING"),
            Self::Succeeded => formatter.write_str("SUCCEEDED"),
            Self::Failed => formatter.write_str("FAILED"),
            Self::Skipped => formatter.write_str("SKIPPED"),
            Self::Cancelled => formatter.write_str("CANCELLED"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferPlanItem {
    pub id: u64,
    pub source: Endpoint,
    pub destination: Endpoint,
    pub direction: TransferDirection,
    pub entry_kind: EntryKind,
    pub expected_size: Option<u64>,
    pub destination_expectation: DestinationExpectation,
    pub conflict_policy: ConflictPolicy,
    pub state: TransferState,
}

impl TransferPlanItem {
    pub fn transition_to(
        &mut self,
        next: TransferState,
    ) -> Result<(), TransferStateTransitionError> {
        let allowed = matches!(
            (self.state, next),
            (TransferState::Staged, TransferState::Running)
                | (
                    TransferState::Running,
                    TransferState::Succeeded
                        | TransferState::Failed
                        | TransferState::Skipped
                        | TransferState::Cancelled
                )
        );
        if !allowed {
            return Err(TransferStateTransitionError {
                from: self.state,
                to: next,
            });
        }
        self.state = next;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransferStateTransitionError {
    pub from: TransferState,
    pub to: TransferState,
}

#[cfg(test)]
mod tests {
    use super::{
        ConflictPolicy, DestinationExpectation, Endpoint, EntryKind, TransferDirection,
        TransferPlanItem, TransferState, TransferStateTransitionError,
    };

    #[test]
    fn transfer_state_requires_running_before_a_terminal_result() {
        let mut item = plan_item();

        assert_eq!(
            item.transition_to(TransferState::Succeeded),
            Err(TransferStateTransitionError {
                from: TransferState::Staged,
                to: TransferState::Succeeded,
            })
        );
        item.transition_to(TransferState::Running)
            .expect("staged item can start");
        item.transition_to(TransferState::Succeeded)
            .expect("running item can succeed");
        assert_eq!(item.state, TransferState::Succeeded);
        assert_eq!(
            item.transition_to(TransferState::Running),
            Err(TransferStateTransitionError {
                from: TransferState::Succeeded,
                to: TransferState::Running,
            })
        );
    }

    fn plan_item() -> TransferPlanItem {
        TransferPlanItem {
            id: 1,
            source: Endpoint::local("/outgoing/report.bin"),
            destination: Endpoint::remote("profile-a", "remote-a", "/incoming/report.bin"),
            direction: TransferDirection::Upload,
            entry_kind: EntryKind::File,
            expected_size: Some(4096),
            destination_expectation: DestinationExpectation::Missing,
            conflict_policy: ConflictPolicy::Ask,
            state: TransferState::Staged,
        }
    }
}
