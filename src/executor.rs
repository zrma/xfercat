use crate::{
    domain::{TransferPlanItem, TransferState},
    transport::{
        TransportFailureKind, TransportOutcome, TransportRequest, TransportResult,
        TransportSkipReason,
    },
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ExecutionSummary {
    pub succeeded: usize,
    pub failed: usize,
    pub skipped: usize,
    pub cancelled: usize,
}

impl ExecutionSummary {
    pub const fn total(self) -> usize {
        self.succeeded + self.failed + self.skipped + self.cancelled
    }

    fn record(&mut self, outcome: TransportOutcome) {
        match outcome {
            TransportOutcome::Succeeded { .. } => self.succeeded += 1,
            TransportOutcome::Skipped { .. } => self.skipped += 1,
            TransportOutcome::Failed { .. } => self.failed += 1,
            TransportOutcome::Cancelled => self.cancelled += 1,
        }
    }
}

pub fn execute_representative(plan: &mut [TransferPlanItem]) -> ExecutionSummary {
    let mut summary = ExecutionSummary::default();
    for (execution_index, item) in plan
        .iter_mut()
        .filter(|item| item.state == TransferState::Staged)
        .enumerate()
    {
        item.transition_to(TransferState::Running)
            .expect("the staged filter guarantees a valid running transition");
        let outcome = match TransportRequest::try_from(&*item) {
            Ok(request) => representative_outcome(execution_index, request.expected_size),
            Err(_) => TransportOutcome::Failed {
                kind: TransportFailureKind::Unsupported,
                retryable: false,
            },
        };
        let result = TransportResult {
            item_id: item.id,
            outcome,
        };
        apply_result(item, &result);
        summary.record(result.outcome);
    }

    summary
}

fn representative_outcome(index: usize, expected_size: Option<u64>) -> TransportOutcome {
    match index % 4 {
        0 => TransportOutcome::Succeeded {
            bytes_transferred: expected_size.unwrap_or(0),
        },
        1 => TransportOutcome::Failed {
            kind: TransportFailureKind::ConnectionLost,
            retryable: true,
        },
        2 => TransportOutcome::Skipped {
            reason: TransportSkipReason::DestinationAlreadyCurrent,
        },
        _ => TransportOutcome::Cancelled,
    }
}

fn apply_result(item: &mut TransferPlanItem, result: &TransportResult) {
    assert_eq!(
        item.id, result.item_id,
        "transport result must target the running item identity"
    );
    let terminal_state = match result.outcome {
        TransportOutcome::Succeeded { .. } => TransferState::Succeeded,
        TransportOutcome::Skipped { .. } => TransferState::Skipped,
        TransportOutcome::Failed { .. } => TransferState::Failed,
        TransportOutcome::Cancelled => TransferState::Cancelled,
    };
    item.transition_to(terminal_state)
        .expect("executor applies a terminal result only to a running item");
}

#[cfg(test)]
mod tests {
    use super::{ExecutionSummary, execute_representative};
    use crate::domain::{
        ConflictPolicy, Endpoint, EntryKind, TransferDirection, TransferPlanItem, TransferState,
    };

    #[test]
    fn representative_execution_preserves_all_item_level_results() {
        let mut plan = (1..=4).map(plan_item).collect::<Vec<_>>();

        let summary = execute_representative(&mut plan);

        assert_eq!(
            summary,
            ExecutionSummary {
                succeeded: 1,
                failed: 1,
                skipped: 1,
                cancelled: 1,
            }
        );
        assert_eq!(
            plan.iter().map(|item| item.state).collect::<Vec<_>>(),
            vec![
                TransferState::Succeeded,
                TransferState::Failed,
                TransferState::Skipped,
                TransferState::Cancelled,
            ]
        );
    }

    #[test]
    fn terminal_items_are_not_implicitly_rerun() {
        let mut plan = (1..=4).map(plan_item).collect::<Vec<_>>();
        execute_representative(&mut plan);
        let terminal = plan.clone();

        let summary = execute_representative(&mut plan);

        assert_eq!(summary.total(), 0);
        assert_eq!(plan, terminal);
    }

    #[test]
    fn invalid_request_fails_that_item_and_execution_continues() {
        let mut plan = (1..=4).map(plan_item).collect::<Vec<_>>();
        plan[0].source.path = "relative.bin".into();

        let summary = execute_representative(&mut plan);

        assert_eq!(summary.failed, 2);
        assert_eq!(
            plan.iter().map(|item| item.state).collect::<Vec<_>>(),
            vec![
                TransferState::Failed,
                TransferState::Failed,
                TransferState::Skipped,
                TransferState::Cancelled,
            ]
        );
    }

    fn plan_item(id: u64) -> TransferPlanItem {
        TransferPlanItem {
            id,
            source: Endpoint::local(format!("/outgoing/file-{id}.bin")),
            destination: Endpoint::remote(
                "profile-a",
                "remote-a",
                format!("/incoming/file-{id}.bin"),
            ),
            direction: TransferDirection::Upload,
            entry_kind: EntryKind::File,
            expected_size: Some(id * 1024),
            conflict_policy: ConflictPolicy::Ask,
            state: TransferState::Staged,
        }
    }
}
