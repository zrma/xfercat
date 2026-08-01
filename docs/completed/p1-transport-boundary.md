# Spec: Typed Transport Boundary

Status: completed

## Goal

staged `TransferPlanItem`을 exact endpoint와 immutable metadata를 가진 typed transport
request로 freeze하고, raw diagnostic text가 없는 typed result contract를 정의한다.

## Scope

- staged file kind와 expected size metadata 보존
- direction, endpoint role과 logical path validation
- immutable `TransportRequest` conversion
- succeeded, skipped, failed와 cancelled typed outcomes
- structured skip/failure reason without credential or raw diagnostic payload

## Constraints

- 기존 사용자 변경과 repository contract를 보존한다.
- filesystem, subprocess, network와 credential material에 접근하지 않는다.
- request conversion은 plan이나 browser state를 변경하지 않는다.
- transport library, async runtime과 cancellation primitive를 선택하지 않는다.

## Acceptance Checklist

| ID | Status | Verify | Work item |
| --- | --- | --- | --- |
| C1 | done | focused transport tests | valid upload/download plan을 exact request로 freeze한다 |
| C2 | done | focused transport tests | invalid endpoint direction과 logical path를 typed error로 거부한다 |
| C3 | done | focused app tests | browser metadata가 staged plan에 보존된다 |
| C4 | done | focused transport tests | result outcomes가 raw diagnostic text 없이 item identity를 보존한다 |
| C5 | done | `scripts/check.sh` | canonical repository gate가 통과한다 |

## Required Evidence

- four focused transport tests
- staged browser metadata app regression
- passing canonical `scripts/check.sh`

## Publication Impact

- source, synthetic fixtures, tests와 repository-owned contract만 tracked artifact에 남긴다.
- remote write는 포함하지 않는다.

## Out Of Scope

- item state transitions와 executor orchestration
- real connection, filesystem mutation과 network transport
- effective OpenSSH config와 host verification
- remote 생성, push, license와 release

## Completion Rule

모든 acceptance가 evidence와 함께 done이고 전체 gate가 통과한다.
