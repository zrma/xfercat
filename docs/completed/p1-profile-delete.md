# Spec: P1 Profile Delete

Status: completed

## Goal

Connections에서 process-lifetime profile을 명시적으로 삭제하되 active connection과
Waybill의 stable endpoint reference를 안전하게 처리한다.

## Scope

- Connections의 selected manual 또는 synthetic profile 삭제
- imported OpenSSH profile의 source-owned delete 안내
- active synthetic connection 해제와 selection 보정
- staged plan reference가 있을 때 non-cascading delete 차단
- compact terminal shortcut과 interaction regression

## Constraints

- 기존 사용자 변경과 repository contract를 보존한다.
- profile 삭제가 staged `TransferPlanItem`을 자동 삭제하거나 endpoint를 재작성하지 않는다.
- imported OpenSSH profile은 source config가 소유하며 앱 catalog에서 직접 삭제하지 않는다.
- filesystem mutation, network transport와 catalog persistence를 추가하지 않는다.

## Acceptance Checklist

| ID | Status | Verify | Work item |
| --- | --- | --- | --- |
| C1 | done | focused app tests | unreferenced process profile을 삭제하고 selection을 보정한다 |
| C2 | done | focused app tests | active profile 삭제는 connection identity를 해제한다 |
| C3 | done | focused app tests | staged reference와 imported profile 삭제는 catalog를 변경하지 않는다 |
| C4 | done | snapshots and PTY smoke | Connections delete shortcut과 status를 확인한다 |
| C5 | done | `scripts/check.sh` | canonical repository gate가 통과한다 |

## Required Evidence

- four focused delete tests
- 110x32 및 80x24 Connections shortcut snapshot regression
- actual PTY delete/re-add/connect/quit smoke with terminal restore
- passing canonical `scripts/check.sh`

## Publication Impact

- source, synthetic fixture, test와 repository-owned status만 tracked artifact에 남긴다.
- remote write는 포함하지 않는다.

## Out Of Scope

- delete confirmation modal과 undo
- catalog 또는 plan persistence
- imported OpenSSH source-file mutation
- real connection, filesystem mutation과 network transport
- remote 생성, push, license와 release

## Completion Rule

모든 acceptance가 evidence와 함께 done이고 전체 gate가 통과한다.
