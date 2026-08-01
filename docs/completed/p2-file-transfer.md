# Spec: P2 Actual File Transfer

Status: completed

## Goal

reviewed Waybill의 regular file upload와 download를 실제 SFTP session에서 안전하게 실행한다.

## Scope

- staged source and destination expectation revalidation
- `ASK`, `OVERWRITE`, `SKIP` and explicit renamed-destination behavior
- sibling temporary file, verified close/size and atomic finalization
- item-level actual result preservation after partial failure
- post-execution local and remote browser refresh

## Constraints

- directory, symlink와 special-file transfer는 거부한다.
- final destination을 직접 create 또는 truncate하지 않는다.
- missing destination은 atomic no-replace finalization을 사용한다.
- overwrite는 explicit policy와 unchanged preview state가 모두 필요하다.
- remote upload는 destination state에 필요한 `hardlink` 또는 `posix-rename` extension을
  temporary write 전에 확인한다.
- test와 validation은 generated key를 쓰는 isolated localhost SFTP fixture에만 write한다.

## Acceptance Checklist

| ID | Status | Verify | Work item |
| --- | --- | --- | --- |
| C1 | done | focused domain/app tests | destination expectation을 staging과 rename에서 freeze한다 |
| C2 | done | isolated SFTP fixture | upload/download bytes가 양방향 동일하다 |
| C3 | done | isolated SFTP fixture | missing, overwrite, skip, ask와 stale state를 안전하게 처리한다 |
| C4 | done | isolated failure tests | partial file은 final name에 노출되지 않고 temp를 정리한다 |
| C5 | done | live Review snapshot | actual execution boundary와 destination expectation을 렌더링한다 |
| C6 | done | `scripts/check.sh` | canonical repository gate가 통과한다 |

## Required Evidence

- three destination expectation, policy and temporary-name regressions
- 192 KiB-plus bidirectional byte-identity loopback fixture
- conflict, stale-source/destination, partial-plan continuation and cleanup fixture cases
- live-mode Review snapshot
- passing canonical `scripts/check.sh`

## Publication Impact

- tracked fixture에는 private endpoint, credential, key material 또는 actual path를 넣지 않는다.
- generated keys, endpoint와 transferred bytes는 temporary test directory에만 존재한다.

## Out Of Scope

- recursive directory, symlink and resume support
- interactive password and host-key enrollment
- persistent plan, progress UI and user cancellation
- remote 생성, push, license와 release

## Completion Rule

모든 acceptance가 evidence와 함께 done이고 전체 gate가 통과했다.
