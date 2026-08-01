# Spec: P2 SFTP Remote Browser

Status: completed

## Goal

strict system OpenSSH session을 실제 SFTP client에 연결하고 remote home과 child/parent
directory를 read-only로 탐색한다.

## Scope

- imported OpenSSH alias 및 manual agent/key-reference profile connection
- strict known-host verification, batch authentication와 bounded connect timeout
- canonical remote directory listing and stable sorting
- remote directory Enter/Backspace navigation
- sanitized typed failure status and graceful session close

## Constraints

- 사용자 target에는 test 또는 validation 중 자동 접속하지 않는다.
- password prompt와 unknown host key 자동 수락을 허용하지 않는다.
- credential, private key 내용, raw SSH diagnostic와 endpoint inventory를 저장하거나 UI에
  복제하지 않는다.
- symlink, special file, non-Unicode와 unsafe name은 browser에서 제외한다.

## Acceptance Checklist

| ID | Status | Verify | Work item |
| --- | --- | --- | --- |
| C1 | done | focused unit tests | profile을 safe system OpenSSH invocation policy로 resolve한다 |
| C2 | done | isolated loopback SFTP fixture | strict trusted host에 연결하고 canonical home을 읽는다 |
| C3 | done | focused adapter tests | remote entry kind, size, exclusion과 stable order를 보존한다 |
| C4 | done | app and interaction tests | live workspace와 remote navigation state를 반영한다 |
| C5 | done | isolated TUI smoke | 실제 remote child/parent 탐색과 graceful close를 확인한다 |
| C6 | done | `scripts/check.sh` | canonical repository gate가 통과한다 |

## Required Evidence

- five profile-resolution, path-safety and sanitized-error regressions
- ephemeral localhost sshd/SFTP listing test with generated fixture keys
- two application navigation and empty-selection regressions
- actual PTY connect/navigation/close smoke
- passing canonical `scripts/check.sh`

## Publication Impact

- loopback fixture는 temporary directory와 generated test keys만 사용한다.
- actual user alias, host, username, path, key와 raw diagnostic은 tracked artifact에 남기지 않는다.

## Out Of Scope

- file upload/download mutation
- directory and symlink transfer
- persistent profiles, password storage and interactive authentication
- remote 생성, push, license와 release

## Completion Rule

모든 acceptance가 evidence와 함께 done이고 전체 gate가 통과했다.
