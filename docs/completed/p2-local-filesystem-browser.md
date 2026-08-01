# Spec: P2 Local Filesystem Browser

Status: completed

## Goal

runtime local pane을 current working directory의 실제 filesystem entry로 채우고 navigation과
staging이 exact canonical local path를 보존하게 한다.

## Scope

- canonical local directory listing and stable sorting
- Enter child directory and Backspace parent navigation
- file kind and size metadata
- symlink, non-Unicode and unreadable entry exclusion summary
- current local/remote directory 기반 upload/download destination staging

## Constraints

- 기존 사용자 변경과 repository contract를 보존한다.
- directory listing은 filesystem을 변경하지 않는다.
- symlink를 follow하거나 lossy path를 staging하지 않는다.
- runtime local path와 inventory를 tracked fixture 또는 문서에 기록하지 않는다.

## Acceptance Checklist

| ID | Status | Verify | Work item |
| --- | --- | --- | --- |
| C1 | done | focused localfs tests | actual entry kind, size와 deterministic order를 읽는다 |
| C2 | done | focused localfs tests | symlink와 invalid entry를 안전하게 제외한다 |
| C3 | done | focused app tests | browser directory 교체와 selection을 보정한다 |
| C4 | done | focused app tests | staged source/destination이 current directories를 freeze한다 |
| C5 | done | snapshots and PTY smoke | dynamic titles와 local navigation을 사용자 화면에서 확인한다 |
| C6 | done | `scripts/check.sh` | canonical repository gate가 통과한다 |

## Required Evidence

- three isolated temporary-directory localfs tests
- two application staging/navigation regressions and deterministic snapshots
- actual PTY local navigation/back smoke with redacted output
- passing canonical `scripts/check.sh`

## Publication Impact

- source, synthetic fixture, tests와 repository-owned status만 tracked artifact에 남긴다.
- actual local path와 inventory는 tracked artifact에 기록하지 않는다.
- remote write는 포함하지 않는다.

## Out Of Scope

- remote SFTP browser and connection
- directory transfer and symlink transfer
- file mutation, upload and download
- remote 생성, push, license와 release

## Completion Rule

모든 acceptance가 evidence와 함께 done이고 전체 gate가 통과한다.
