# Spec: P1 Waybill Editing

Status: completed

## Goal

Waybill item의 exact source identity를 보존하면서 destination filename과 실행 순서를
명시적으로 수정하고 Review에서 최종 계획을 확인할 수 있게 한다.

## Scope

- selected item의 destination filename rename mode
- stable item ID를 보존하는 move up/down
- rename/reorder 결과를 Waybill과 Review에 동일하게 render
- compact terminal interaction과 snapshot regression

## Constraints

- 기존 사용자 변경과 repository contract를 보존한다.
- rename은 destination leaf만 변경하고 parent path, endpoint와 source를 바꾸지 않는다.
- empty name, `.`·`..`와 path separator가 포함된 name은 거부한다.
- reorder는 item ID와 payload를 바꾸지 않는다.
- filesystem mutation과 network transport는 실행하지 않는다.

## Acceptance Checklist

| ID | Status | Verify | Work item |
| --- | --- | --- | --- |
| C1 | done | focused app tests | valid rename은 destination leaf만 변경한다 |
| C2 | done | focused app tests | invalid rename은 plan과 screen state를 보존한다 |
| C3 | done | focused app tests | reorder는 selected stable item을 이동하고 payload를 보존한다 |
| C4 | done | snapshots and PTY smoke | Waybill, Rename, Review와 compact shortcut을 확인한다 |
| C5 | done | `scripts/check.sh` | canonical repository gate가 통과한다 |

## Required Evidence

- seven app tests including valid/invalid/cancelled rename and stable-ID reorder
- renamed destination과 reordered ID가 보이는 Review snapshot
- 80×24 shortcut visibility test
- actual PTY staging/rename/reorder/review/quit smoke with terminal restore

## Publication Impact

- source, synthetic fixture, test와 repository-owned status만 tracked artifact에 남긴다.
- remote write는 포함하지 않는다.

## Out Of Scope

- source rename 또는 endpoint/profile 변경
- directory rename
- item-level transfer result state
- filesystem/network transport
- remote 생성, push, license와 release

## Completion Rule

모든 acceptance가 evidence와 함께 done이고 전체 gate가 통과한다.
