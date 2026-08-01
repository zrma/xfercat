# Spec: P1 Synthetic Executor

Status: completed

## Goal

Review에서 명시적인 synthetic execution action을 실행하고 각 item의 running, succeeded,
failed, skipped와 cancelled transition 및 partial result 보존을 검증한다.

## Scope

- guarded `TransferState` transition contract
- deterministic representative synthetic executor
- typed `TransportResult` to item-state application
- Review execution action, terminal result rendering과 summary
- full and compact terminal regression plus PTY interaction

## Constraints

- 기존 사용자 변경과 repository contract를 보존한다.
- filesystem, subprocess, network와 credential material에 접근하지 않는다.
- synthetic outcome을 실제 transfer success로 표현하지 않는다.
- 한 item의 failure, skip 또는 cancellation이 다른 item result를 지우지 않는다.

## Acceptance Checklist

| ID | Status | Verify | Work item |
| --- | --- | --- | --- |
| C1 | done | focused domain tests | staged-running-terminal transition만 허용한다 |
| C2 | done | focused executor tests | success, failure, skip과 cancellation 결과를 item별로 보존한다 |
| C3 | done | focused executor tests | terminal item을 암묵적으로 재실행하지 않는다 |
| C4 | done | snapshots and PTY smoke | explicit synthetic action과 terminal result를 사용자 화면에서 확인한다 |
| C5 | done | `scripts/check.sh` | canonical repository gate가 통과한다 |

## Required Evidence

- one state-machine and three representative executor unit tests
- staged Review and four-outcome Results snapshots at 110x32 and 80x24
- actual PTY review/execute/back/quit smoke with terminal restore
- passing canonical `scripts/check.sh`

## Publication Impact

- source, synthetic fixtures, tests와 repository-owned status만 tracked artifact에 남긴다.
- remote write는 포함하지 않는다.

## Out Of Scope

- real connection, filesystem mutation과 network transport
- progress bytes, async runtime, user-triggered mid-flight cancellation과 retry
- effective OpenSSH config와 host verification
- remote 생성, push, license와 release

## Completion Rule

모든 acceptance가 evidence와 함께 done이고 전체 gate가 통과한다.
