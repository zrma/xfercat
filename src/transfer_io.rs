use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use bytes::BytesMut;
use openssh_sftp_client::{Error as RemoteError, error::SftpErrorKind, metadata::MetaData};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    domain::{ConflictPolicy, DestinationExpectation, EntryKind, TransferDirection},
    sftp::SftpSession,
    transport::{TransportFailureKind, TransportOutcome, TransportRequest, TransportSkipReason},
};

const BUFFER_SIZE: usize = 64 * 1024;
static TEMP_NONCE: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ObservedKind {
    File,
    Directory,
    Other,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ObservedDestination {
    Missing,
    Existing {
        kind: ObservedKind,
        size: Option<u64>,
    },
}

pub async fn execute_request(
    session: &SftpSession,
    request: &TransportRequest,
) -> TransportOutcome {
    if request.entry_kind != EntryKind::File {
        return failed(TransportFailureKind::Unsupported, false);
    }
    match request.direction {
        TransferDirection::Upload => upload(session, request).await,
        TransferDirection::Download => download(session, request).await,
    }
}

async fn upload(session: &SftpSession, request: &TransportRequest) -> TransportOutcome {
    let source = Path::new(&request.source.path);
    let source_size = match local_source_size(source, request.expected_size).await {
        Ok(size) => size,
        Err(outcome) => return outcome,
    };
    let destination = &request.destination.path;
    let observed = match observe_remote(session, destination).await {
        Ok(observed) => observed,
        Err(outcome) => return outcome,
    };
    if let Some(outcome) = destination_decision(request, observed) {
        return outcome;
    }
    if !remote_atomic_finalization_supported(
        request.destination_expectation,
        session.client().support_hardlink(),
        session.client().support_posix_rename(),
    ) {
        return failed(TransportFailureKind::Unsupported, false);
    }
    let Some(temporary) = remote_temporary_path(destination, request.item_id) else {
        return failed(TransportFailureKind::Unsupported, false);
    };

    let mut local = match tokio::fs::File::open(source).await {
        Ok(file) => file,
        Err(error) => return local_error(error, true),
    };
    let mut options = session.client().options();
    let mut remote = match options.write(true).create_new(true).open(&temporary).await {
        Ok(file) => file,
        Err(error) => return remote_error(&error, false),
    };

    let mut buffer = vec![0_u8; BUFFER_SIZE];
    let mut transferred = 0_u64;
    loop {
        let read = match local.read(&mut buffer).await {
            Ok(0) => break,
            Ok(read) => read,
            Err(error) => {
                let _ = remote.close().await;
                remove_remote_quiet(session, &temporary).await;
                return local_error(error, true);
            }
        };
        if let Err(error) = remote.write_all(&buffer[..read]).await {
            let _ = remote.close().await;
            remove_remote_quiet(session, &temporary).await;
            return remote_error(&error, true);
        }
        transferred += read as u64;
    }
    if session.client().support_fsync()
        && let Err(error) = remote.sync_all().await
    {
        let _ = remote.close().await;
        remove_remote_quiet(session, &temporary).await;
        return remote_error(&error, true);
    }
    if let Err(error) = remote.close().await {
        remove_remote_quiet(session, &temporary).await;
        return remote_error(&error, true);
    }
    if transferred != source_size {
        remove_remote_quiet(session, &temporary).await;
        return failed(TransportFailureKind::StaleSource, false);
    }
    match remote_file_size(session, &temporary).await {
        Ok(size) if size == transferred => {}
        Ok(_) => {
            remove_remote_quiet(session, &temporary).await;
            return failed(TransportFailureKind::RemoteFilesystem, true);
        }
        Err(outcome) => {
            remove_remote_quiet(session, &temporary).await;
            return outcome;
        }
    }
    if local_source_size(source, Some(source_size)).await.is_err() {
        remove_remote_quiet(session, &temporary).await;
        return failed(TransportFailureKind::StaleSource, false);
    }
    let current = match observe_remote(session, destination).await {
        Ok(observed) => observed,
        Err(outcome) => {
            remove_remote_quiet(session, &temporary).await;
            return outcome;
        }
    };
    if !expectation_matches(request.destination_expectation, current) {
        remove_remote_quiet(session, &temporary).await;
        return failed(TransportFailureKind::StaleDestination, false);
    }

    if let Err(outcome) = finalize_remote(
        session,
        &temporary,
        destination,
        request.destination_expectation,
    )
    .await
    {
        remove_remote_quiet(session, &temporary).await;
        return outcome;
    }
    TransportOutcome::Succeeded {
        bytes_transferred: transferred,
    }
}

async fn download(session: &SftpSession, request: &TransportRequest) -> TransportOutcome {
    let source = &request.source.path;
    let source_size = match remote_source_size(session, source, request.expected_size).await {
        Ok(size) => size,
        Err(outcome) => return outcome,
    };
    let destination = Path::new(&request.destination.path);
    let observed = match observe_local(destination).await {
        Ok(observed) => observed,
        Err(outcome) => return outcome,
    };
    if let Some(outcome) = destination_decision(request, observed) {
        return outcome;
    }
    let Some(temporary) = local_temporary_path(destination, request.item_id) else {
        return failed(TransportFailureKind::Unsupported, false);
    };

    let mut remote = match session.client().open(source).await {
        Ok(file) => file,
        Err(error) => return remote_source_error(&error),
    };
    let mut local = match tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .await
    {
        Ok(file) => file,
        Err(error) => {
            let _ = remote.close().await;
            return local_error(error, false);
        }
    };

    let mut buffer = BytesMut::with_capacity(BUFFER_SIZE);
    let mut transferred = 0_u64;
    loop {
        match remote.read(BUFFER_SIZE as u32, buffer).await {
            Ok(Some(chunk)) => {
                if let Err(error) = local.write_all(&chunk).await {
                    let _ = remote.close().await;
                    drop(local);
                    remove_local_quiet(&temporary).await;
                    return local_error(error, true);
                }
                transferred += chunk.len() as u64;
                buffer = chunk;
                buffer.clear();
            }
            Ok(None) => break,
            Err(error) => {
                let _ = remote.close().await;
                drop(local);
                remove_local_quiet(&temporary).await;
                return remote_error(&error, true);
            }
        }
    }
    if let Err(error) = remote.close().await {
        drop(local);
        remove_local_quiet(&temporary).await;
        return remote_error(&error, true);
    }
    if let Err(error) = local.flush().await {
        drop(local);
        remove_local_quiet(&temporary).await;
        return local_error(error, true);
    }
    if let Err(error) = local.sync_all().await {
        drop(local);
        remove_local_quiet(&temporary).await;
        return local_error(error, true);
    }
    drop(local);
    if transferred != source_size {
        remove_local_quiet(&temporary).await;
        return failed(TransportFailureKind::StaleSource, false);
    }
    match tokio::fs::metadata(&temporary).await {
        Ok(metadata) if metadata.len() == transferred => {}
        Ok(_) => {
            remove_local_quiet(&temporary).await;
            return failed(TransportFailureKind::LocalFilesystem, true);
        }
        Err(error) => {
            remove_local_quiet(&temporary).await;
            return local_error(error, true);
        }
    }
    if remote_source_size(session, source, Some(source_size))
        .await
        .is_err()
    {
        remove_local_quiet(&temporary).await;
        return failed(TransportFailureKind::StaleSource, false);
    }
    let current = match observe_local(destination).await {
        Ok(observed) => observed,
        Err(outcome) => {
            remove_local_quiet(&temporary).await;
            return outcome;
        }
    };
    if !expectation_matches(request.destination_expectation, current) {
        remove_local_quiet(&temporary).await;
        return failed(TransportFailureKind::StaleDestination, false);
    }

    if let Err(outcome) =
        finalize_local(&temporary, destination, request.destination_expectation).await
    {
        remove_local_quiet(&temporary).await;
        return outcome;
    }
    TransportOutcome::Succeeded {
        bytes_transferred: transferred,
    }
}

fn destination_decision(
    request: &TransportRequest,
    observed: ObservedDestination,
) -> Option<TransportOutcome> {
    if !expectation_matches(request.destination_expectation, observed) {
        return Some(failed(TransportFailureKind::StaleDestination, false));
    }
    let ObservedDestination::Existing { kind, .. } = observed else {
        return None;
    };
    match request.conflict_policy {
        ConflictPolicy::Skip => Some(TransportOutcome::Skipped {
            reason: TransportSkipReason::ConflictPolicy,
        }),
        ConflictPolicy::Overwrite if kind == ObservedKind::File => None,
        ConflictPolicy::Ask | ConflictPolicy::Rename | ConflictPolicy::Overwrite => {
            Some(failed(TransportFailureKind::DestinationConflict, false))
        }
    }
}

fn expectation_matches(expected: DestinationExpectation, observed: ObservedDestination) -> bool {
    match (expected, observed) {
        (DestinationExpectation::Missing, ObservedDestination::Missing) => true,
        (
            DestinationExpectation::Existing {
                kind: expected_kind,
                size: expected_size,
            },
            ObservedDestination::Existing {
                kind: observed_kind,
                size: observed_size,
            },
        ) => {
            let kind_matches = matches!(
                (expected_kind, observed_kind),
                (EntryKind::File, ObservedKind::File)
                    | (EntryKind::Directory, ObservedKind::Directory)
            );
            kind_matches && expected_size.is_none_or(|size| observed_size == Some(size))
        }
        _ => false,
    }
}

const fn remote_atomic_finalization_supported(
    expectation: DestinationExpectation,
    hardlink: bool,
    posix_rename: bool,
) -> bool {
    match expectation {
        DestinationExpectation::Missing => hardlink,
        DestinationExpectation::Existing { .. } => posix_rename,
    }
}

async fn local_source_size(
    path: &Path,
    expected_size: Option<u64>,
) -> Result<u64, TransportOutcome> {
    let metadata = tokio::fs::symlink_metadata(path)
        .await
        .map_err(|error| local_error(error, true))?;
    if !metadata.file_type().is_file() {
        return Err(failed(TransportFailureKind::Unsupported, false));
    }
    if expected_size.is_some_and(|expected| expected != metadata.len()) {
        return Err(failed(TransportFailureKind::StaleSource, false));
    }
    Ok(metadata.len())
}

async fn remote_source_size(
    session: &SftpSession,
    path: &str,
    expected_size: Option<u64>,
) -> Result<u64, TransportOutcome> {
    let mut fs = session.client().fs();
    let metadata = fs
        .symlink_metadata(path)
        .await
        .map_err(|error| remote_source_error(&error))?;
    let Some(file_type) = metadata.file_type() else {
        return Err(failed(TransportFailureKind::RemoteFilesystem, true));
    };
    if !file_type.is_file() {
        return Err(failed(TransportFailureKind::Unsupported, false));
    }
    let Some(size) = metadata.len() else {
        return Err(failed(TransportFailureKind::RemoteFilesystem, true));
    };
    if expected_size.is_some_and(|expected| expected != size) {
        return Err(failed(TransportFailureKind::StaleSource, false));
    }
    Ok(size)
}

async fn observe_local(path: &Path) -> Result<ObservedDestination, TransportOutcome> {
    match tokio::fs::symlink_metadata(path).await {
        Ok(metadata) => Ok(ObservedDestination::Existing {
            kind: if metadata.file_type().is_file() {
                ObservedKind::File
            } else if metadata.file_type().is_dir() {
                ObservedKind::Directory
            } else {
                ObservedKind::Other
            },
            size: metadata.file_type().is_file().then_some(metadata.len()),
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(ObservedDestination::Missing)
        }
        Err(error) => Err(local_error(error, true)),
    }
}

async fn observe_remote(
    session: &SftpSession,
    path: &str,
) -> Result<ObservedDestination, TransportOutcome> {
    let mut fs = session.client().fs();
    match fs.symlink_metadata(path).await {
        Ok(metadata) => Ok(observed_remote_metadata(metadata)),
        Err(RemoteError::SftpError(SftpErrorKind::NoSuchFile, _)) => {
            Ok(ObservedDestination::Missing)
        }
        Err(error) => Err(remote_error(&error, true)),
    }
}

fn observed_remote_metadata(metadata: MetaData) -> ObservedDestination {
    let kind = metadata.file_type().map_or(ObservedKind::Other, |kind| {
        if kind.is_file() {
            ObservedKind::File
        } else if kind.is_dir() {
            ObservedKind::Directory
        } else {
            ObservedKind::Other
        }
    });
    ObservedDestination::Existing {
        kind,
        size: (kind == ObservedKind::File)
            .then(|| metadata.len())
            .flatten(),
    }
}

async fn remote_file_size(session: &SftpSession, path: &str) -> Result<u64, TransportOutcome> {
    let mut fs = session.client().fs();
    let metadata = fs
        .symlink_metadata(path)
        .await
        .map_err(|error| remote_error(&error, true))?;
    if !metadata.file_type().is_some_and(|kind| kind.is_file()) {
        return Err(failed(TransportFailureKind::RemoteFilesystem, true));
    }
    metadata
        .len()
        .ok_or_else(|| failed(TransportFailureKind::RemoteFilesystem, true))
}

async fn finalize_remote(
    session: &SftpSession,
    temporary: &str,
    destination: &str,
    expectation: DestinationExpectation,
) -> Result<(), TransportOutcome> {
    let mut fs = session.client().fs();
    match expectation {
        DestinationExpectation::Missing => {
            fs.hard_link(temporary, destination)
                .await
                .map_err(|error| remote_finalization_error(&error))?;
            let _ = fs.remove_file(temporary).await;
            Ok(())
        }
        DestinationExpectation::Existing { .. } => fs
            .rename(temporary, destination)
            .await
            .map_err(|error| remote_error(&error, true)),
    }
}

async fn finalize_local(
    temporary: &Path,
    destination: &Path,
    expectation: DestinationExpectation,
) -> Result<(), TransportOutcome> {
    match expectation {
        DestinationExpectation::Missing => {
            tokio::fs::hard_link(temporary, destination)
                .await
                .map_err(local_finalization_error)?;
            let _ = tokio::fs::remove_file(temporary).await;
            Ok(())
        }
        DestinationExpectation::Existing { .. } => tokio::fs::rename(temporary, destination)
            .await
            .map_err(|error| local_error(error, true)),
    }
}

fn remote_temporary_path(destination: &str, item_id: u64) -> Option<String> {
    let (parent, name) = destination.rsplit_once('/')?;
    if name.is_empty() {
        return None;
    }
    let nonce = TEMP_NONCE.fetch_add(1, Ordering::Relaxed);
    let temporary = format!(".{name}.xfercat-{item_id}-{nonce}.part");
    Some(if parent.is_empty() {
        format!("/{temporary}")
    } else {
        format!("{parent}/{temporary}")
    })
}

fn local_temporary_path(destination: &Path, item_id: u64) -> Option<PathBuf> {
    let parent = destination.parent()?;
    let name = destination.file_name()?.to_str()?;
    let nonce = TEMP_NONCE.fetch_add(1, Ordering::Relaxed);
    Some(parent.join(format!(".{name}.xfercat-{item_id}-{nonce}.part")))
}

async fn remove_remote_quiet(session: &SftpSession, path: &str) {
    let _ = session.client().fs().remove_file(path).await;
}

async fn remove_local_quiet(path: &Path) {
    let _ = tokio::fs::remove_file(path).await;
}

fn local_error(error: std::io::Error, retryable: bool) -> TransportOutcome {
    let kind = match error.kind() {
        std::io::ErrorKind::NotFound => TransportFailureKind::SourceNotFound,
        std::io::ErrorKind::PermissionDenied => TransportFailureKind::PermissionDenied,
        std::io::ErrorKind::AlreadyExists => TransportFailureKind::DestinationConflict,
        _ => TransportFailureKind::LocalFilesystem,
    };
    failed(kind, retryable)
}

fn local_finalization_error(error: std::io::Error) -> TransportOutcome {
    if error.kind() == std::io::ErrorKind::AlreadyExists {
        failed(TransportFailureKind::StaleDestination, false)
    } else {
        local_error(error, true)
    }
}

fn remote_source_error(error: &RemoteError) -> TransportOutcome {
    match error {
        RemoteError::SftpError(SftpErrorKind::NoSuchFile, _) => {
            failed(TransportFailureKind::SourceNotFound, false)
        }
        _ => remote_error(error, true),
    }
}

fn remote_finalization_error(error: &RemoteError) -> TransportOutcome {
    match error {
        RemoteError::SftpError(SftpErrorKind::Failure, _) => {
            failed(TransportFailureKind::StaleDestination, false)
        }
        _ => remote_error(error, true),
    }
}

fn remote_error(error: &RemoteError, retryable: bool) -> TransportOutcome {
    let kind = match error {
        RemoteError::SftpError(SftpErrorKind::PermDenied, _) => {
            TransportFailureKind::PermissionDenied
        }
        RemoteError::BackgroundTaskFailure(_)
        | RemoteError::SftpServerFailure(_)
        | RemoteError::RemoteChildSpawnError(_) => TransportFailureKind::ConnectionLost,
        _ => TransportFailureKind::RemoteFilesystem,
    };
    failed(kind, retryable)
}

const fn failed(kind: TransportFailureKind, retryable: bool) -> TransportOutcome {
    TransportOutcome::Failed { kind, retryable }
}

#[cfg(test)]
mod tests {
    use super::{
        ObservedDestination, ObservedKind, destination_decision, expectation_matches,
        local_temporary_path, remote_atomic_finalization_supported, remote_temporary_path,
    };
    use crate::{
        domain::{ConflictPolicy, DestinationExpectation, Endpoint, EntryKind, TransferDirection},
        transport::{
            TransportFailureKind, TransportOutcome, TransportRequest, TransportSkipReason,
        },
    };

    #[test]
    fn destination_expectation_detects_stale_kind_and_size() {
        let expected = DestinationExpectation::Existing {
            kind: EntryKind::File,
            size: Some(7),
        };
        assert!(expectation_matches(
            expected,
            ObservedDestination::Existing {
                kind: ObservedKind::File,
                size: Some(7),
            }
        ));
        assert!(!expectation_matches(
            expected,
            ObservedDestination::Existing {
                kind: ObservedKind::File,
                size: Some(8),
            }
        ));
        assert!(!expectation_matches(
            DestinationExpectation::Missing,
            ObservedDestination::Existing {
                kind: ObservedKind::Other,
                size: None,
            }
        ));
    }

    #[test]
    fn conflict_policy_never_writes_for_ask_rename_or_skip() {
        let observed = ObservedDestination::Existing {
            kind: ObservedKind::File,
            size: Some(7),
        };
        let mut request = request(ConflictPolicy::Ask);
        assert!(matches!(
            destination_decision(&request, observed),
            Some(TransportOutcome::Failed {
                kind: TransportFailureKind::DestinationConflict,
                ..
            })
        ));
        request.conflict_policy = ConflictPolicy::Rename;
        assert!(matches!(
            destination_decision(&request, observed),
            Some(TransportOutcome::Failed {
                kind: TransportFailureKind::DestinationConflict,
                ..
            })
        ));
        request.conflict_policy = ConflictPolicy::Skip;
        assert_eq!(
            destination_decision(&request, observed),
            Some(TransportOutcome::Skipped {
                reason: TransportSkipReason::ConflictPolicy
            })
        );
        request.conflict_policy = ConflictPolicy::Overwrite;
        assert_eq!(destination_decision(&request, observed), None);
    }

    #[test]
    fn remote_transfer_requires_the_atomic_extension_for_its_destination_state() {
        assert!(remote_atomic_finalization_supported(
            DestinationExpectation::Missing,
            true,
            false,
        ));
        assert!(!remote_atomic_finalization_supported(
            DestinationExpectation::Missing,
            false,
            true,
        ));

        let existing = DestinationExpectation::Existing {
            kind: EntryKind::File,
            size: Some(7),
        };
        assert!(remote_atomic_finalization_supported(existing, false, true));
        assert!(!remote_atomic_finalization_supported(existing, true, false));
    }

    #[test]
    fn temporary_names_are_hidden_siblings_without_changing_the_destination() {
        let remote = remote_temporary_path("/incoming/report.bin", 42).expect("remote temp");
        let local = local_temporary_path(std::path::Path::new("/incoming/report.bin"), 42)
            .expect("local temp");
        assert!(remote.starts_with("/incoming/.report.bin.xfercat-42-"));
        assert!(remote.ends_with(".part"));
        assert_eq!(local.parent(), Some(std::path::Path::new("/incoming")));
        assert!(
            local
                .file_name()
                .expect("local temp name")
                .to_string_lossy()
                .starts_with(".report.bin.xfercat-42-")
        );
    }

    fn request(conflict_policy: ConflictPolicy) -> TransportRequest {
        TransportRequest {
            item_id: 42,
            source: Endpoint::local("/outgoing/report.bin"),
            destination: Endpoint::remote("fixture", "fixture", "/incoming/report.bin"),
            direction: TransferDirection::Upload,
            entry_kind: EntryKind::File,
            expected_size: Some(7),
            destination_expectation: DestinationExpectation::Existing {
                kind: EntryKind::File,
                size: Some(7),
            },
            conflict_policy,
        }
    }
}
