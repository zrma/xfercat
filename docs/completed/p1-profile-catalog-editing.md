# Spec: P1 Profile Catalog Editing

Status: completed

## Goal

Connections에서 connect action과 분리된 form으로 synthetic profile을 추가·편집하고,
저장한 profile을 현재 process 안에서 즉시 선택할 수 있게 한다.

## Scope

- `A`로 빈 profile form을 열고 valid profile을 추가한다.
- `E`로 selected profile을 prefilled form에서 수정한다.
- label, user, host와 SSH Agent/key-reference authentication을 편집한다.
- validation 실패와 cancel은 catalog를 변경하지 않는다.
- compact terminal과 deterministic snapshot에서 form과 shortcut을 검증한다.

## Constraints

- 기존 사용자 변경과 repository contract를 보존한다.
- profile save와 connect는 별도 action으로 유지한다.
- edit은 stable profile ID를 보존한다.
- credential, private key 내용과 실제 host inventory를 저장하지 않는다.
- catalog는 process lifetime에만 존재하며 disk persistence를 추가하지 않는다.

## Acceptance Checklist

| ID | Status | Verify | Work item |
| --- | --- | --- | --- |
| C1 | done | focused app tests | valid create가 unique stable ID로 profile을 추가한다 |
| C2 | done | focused app tests | edit이 ID를 보존하고 selected profile fields만 갱신한다 |
| C3 | done | focused app tests | invalid/duplicate/cancelled form은 catalog를 변경하지 않는다 |
| C4 | done | snapshots and PTY smoke | form, authentication toggle와 shortcuts가 보인다 |
| C5 | done | `scripts/check.sh` | canonical repository gate가 통과한다 |

## Required Evidence

- six profile application-state tests within 12 passing app tests
- passing create/edit deterministic snapshots and 80×24 shortcut regression
- passing PTY add/save/connect/edit/cancel/quit smoke with terminal restoration
- passing canonical `scripts/check.sh`

## Publication Impact

- source, synthetic fixture, test와 repository-owned status만 tracked artifact에 남긴다.
- remote write는 포함하지 않는다.

## Out Of Scope

- profile delete
- catalog 또는 plan persistence
- OpenSSH config import
- credential manager, private-key storage와 host inventory
- real connection establishment, filesystem mutation과 network transport
- remote 생성, push, license와 release

## Completion Rule

모든 acceptance가 evidence와 함께 done이고 전체 gate가 통과한다.
