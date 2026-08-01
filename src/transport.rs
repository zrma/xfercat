use crate::domain::{ConflictPolicy, Endpoint, EntryKind, TransferDirection, TransferPlanItem};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransportRequest {
    pub item_id: u64,
    pub source: Endpoint,
    pub destination: Endpoint,
    pub direction: TransferDirection,
    pub entry_kind: EntryKind,
    pub expected_size: Option<u64>,
    pub conflict_policy: ConflictPolicy,
}

impl TryFrom<&TransferPlanItem> for TransportRequest {
    type Error = TransportRequestError;

    fn try_from(item: &TransferPlanItem) -> Result<Self, Self::Error> {
        if !is_valid_logical_path(&item.source.path) {
            return Err(TransportRequestError::InvalidSourcePath);
        }
        if !is_valid_logical_path(&item.destination.path) {
            return Err(TransportRequestError::InvalidDestinationPath);
        }

        let source_is_remote = item.source.profile_id.is_some();
        let destination_is_remote = item.destination.profile_id.is_some();
        let endpoint_roles_match = match item.direction {
            TransferDirection::Upload => !source_is_remote && destination_is_remote,
            TransferDirection::Download => source_is_remote && !destination_is_remote,
        };
        if !endpoint_roles_match {
            return Err(TransportRequestError::EndpointDirectionMismatch);
        }

        Ok(Self {
            item_id: item.id,
            source: item.source.clone(),
            destination: item.destination.clone(),
            direction: item.direction,
            entry_kind: item.entry_kind,
            expected_size: item.expected_size,
            conflict_policy: item.conflict_policy,
        })
    }
}

fn is_valid_logical_path(path: &str) -> bool {
    path.starts_with('/')
        && !path.chars().any(char::is_control)
        && !path
            .split('/')
            .any(|component| matches!(component, "." | ".."))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportRequestError {
    InvalidSourcePath,
    InvalidDestinationPath,
    EndpointDirectionMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransportResult {
    pub item_id: u64,
    pub outcome: TransportOutcome,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportOutcome {
    Succeeded {
        bytes_transferred: u64,
    },
    Skipped {
        reason: TransportSkipReason,
    },
    Failed {
        kind: TransportFailureKind,
        retryable: bool,
    },
    Cancelled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportSkipReason {
    ConflictPolicy,
    DestinationAlreadyCurrent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportFailureKind {
    Authentication,
    HostVerification,
    SourceNotFound,
    PermissionDenied,
    DestinationConflict,
    StaleDestination,
    ConnectionLost,
    Unsupported,
}

#[cfg(test)]
mod tests {
    use super::{
        TransportFailureKind, TransportOutcome, TransportRequest, TransportRequestError,
        TransportResult, TransportSkipReason,
    };
    use crate::domain::{
        ConflictPolicy, Endpoint, EntryKind, TransferDirection, TransferPlanItem, TransferState,
    };

    #[test]
    fn freezes_exact_upload_and_download_requests() {
        let upload = plan_item(TransferDirection::Upload);
        let download = plan_item(TransferDirection::Download);

        let upload_request = TransportRequest::try_from(&upload).expect("valid upload");
        let download_request = TransportRequest::try_from(&download).expect("valid download");

        assert_eq!(upload_request.item_id, upload.id);
        assert_eq!(upload_request.source, upload.source);
        assert_eq!(upload_request.destination, upload.destination);
        assert_eq!(upload_request.entry_kind, EntryKind::File);
        assert_eq!(upload_request.expected_size, Some(4096));
        assert_eq!(upload_request.conflict_policy, ConflictPolicy::Rename);
        assert_eq!(download_request.direction, TransferDirection::Download);
        assert!(download_request.source.profile_id.is_some());
        assert!(download_request.destination.profile_id.is_none());
    }

    #[test]
    fn rejects_invalid_endpoint_roles_without_mutating_the_plan() {
        let mut item = plan_item(TransferDirection::Upload);
        item.source = Endpoint::remote("profile-a", "remote-a", "/incoming/report.bin");
        let original = item.clone();

        let result = TransportRequest::try_from(&item);

        assert_eq!(
            result,
            Err(TransportRequestError::EndpointDirectionMismatch)
        );
        assert_eq!(item, original);
    }

    #[test]
    fn rejects_relative_traversal_and_control_character_paths() {
        let mut item = plan_item(TransferDirection::Upload);
        item.source.path = "outgoing/report.bin".into();
        assert_eq!(
            TransportRequest::try_from(&item),
            Err(TransportRequestError::InvalidSourcePath)
        );

        item.source.path = "/outgoing/../report.bin".into();
        assert_eq!(
            TransportRequest::try_from(&item),
            Err(TransportRequestError::InvalidSourcePath)
        );

        item.source.path = "/outgoing/report.bin".into();
        item.destination.path = "/incoming/report\n.bin".into();
        assert_eq!(
            TransportRequest::try_from(&item),
            Err(TransportRequestError::InvalidDestinationPath)
        );
    }

    #[test]
    fn typed_results_preserve_item_identity_without_raw_diagnostics() {
        let outcomes = [
            TransportOutcome::Succeeded {
                bytes_transferred: 4096,
            },
            TransportOutcome::Skipped {
                reason: TransportSkipReason::ConflictPolicy,
            },
            TransportOutcome::Failed {
                kind: TransportFailureKind::ConnectionLost,
                retryable: true,
            },
            TransportOutcome::Cancelled,
        ];

        for outcome in outcomes {
            let result = TransportResult {
                item_id: 42,
                outcome,
            };
            assert_eq!(result.item_id, 42);
        }
    }

    fn plan_item(direction: TransferDirection) -> TransferPlanItem {
        let (source, destination) = match direction {
            TransferDirection::Upload => (
                Endpoint::local("/outgoing/report.bin"),
                Endpoint::remote("profile-a", "remote-a", "/incoming/report.bin"),
            ),
            TransferDirection::Download => (
                Endpoint::remote("profile-a", "remote-a", "/outgoing/report.bin"),
                Endpoint::local("/incoming/report.bin"),
            ),
        };
        TransferPlanItem {
            id: 42,
            source,
            destination,
            direction,
            entry_kind: EntryKind::File,
            expected_size: Some(4096),
            conflict_policy: ConflictPolicy::Rename,
            state: TransferState::Staged,
        }
    }
}
