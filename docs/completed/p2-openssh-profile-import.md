# Spec: P2 OpenSSH Profile Import

Status: completed

## Goal

기본 Connections 경험을 manual form에서 OpenSSH config alias picker로 전환한다. 앱 시작과
명시적 refresh에서 user config의 concrete `Host` alias를 side-effect 없이 가져오며,
manual add/edit은 fallback으로 유지한다.

## Scope

- user `~/.ssh/config`의 concrete `Host` alias 자동 discovery
- global `Include`의 quoted path, `~`, `%d`, environment variable와 glob expansion
- duplicate alias 제거, deterministic ordering과 include cycle 방지
- imported profile의 OpenSSH provenance와 read-only behavior 표시
- `I` refresh, `A` manual fallback과 config가 없거나 비어 있는 empty state
- deterministic fixture, snapshot과 redacted machine-local smoke

## Constraints

- 기존 사용자 변경과 repository contract를 보존한다.
- startup discovery는 `ssh -G`, `Match exec` 또는 다른 subprocess를 실행하지 않는다.
- wildcard, negated `Host` pattern과 conditional `Include`는 picker entry로 만들지 않는다.
- imported profile은 alias reference만 소유하고 effective config나 credential을 복제하지 않는다.
- tracked fixture와 evidence에 실제 alias, host, user, key path와 config path를 기록하지 않는다.
- config, private key, agent와 network를 변경하지 않는다.

## Acceptance Checklist

| ID | Status | Verify | Work item |
| --- | --- | --- | --- |
| C1 | done | focused discovery tests | concrete alias만 parse, dedupe와 sort한다 |
| C2 | done | focused discovery tests | global include/glob/cycle과 conditional skip을 처리한다 |
| C3 | done | focused app tests | runtime import/refresh가 manual profile과 staged plan을 보존한다 |
| C4 | done | snapshots and PTY smoke | imported/read-only, refresh/manual fallback과 empty state가 보인다 |
| C5 | done | redacted local smoke | 실제 user config에서 count만 확인하고 private value를 출력하지 않는다 |
| C6 | done | `scripts/check.sh` | canonical repository gate가 통과한다 |

## Required Evidence

- five discovery tests and three import-specific app tests within 20 passing unit tests
- passing OpenSSH/empty-catalog snapshots and 80×24 regression within seven interaction tests
- passing redacted local-config PTY import/edit-block/refresh/manual-fallback smoke with terminal restore
- machine-local root/alias/directive counts only; no private values printed
- passing canonical `scripts/check.sh`

## Publication Impact

- source, synthetic config fixture, tests와 repository-owned decision/status만 tracked한다.
- actual config contents, aliases and resolved inventory remain local-only.
- remote write는 포함하지 않는다.

## Out Of Scope

- `ssh -G` effective configuration resolution and `Match` evaluation
- system-wide config and known-hosts discovery
- imported profile editing or source config mutation
- profile persistence, delete and search/filter
- real connection, host verification, transport and credential execution
- remote 생성, push, license와 release

## Completion Rule

모든 acceptance가 evidence와 함께 done이고 전체 gate가 통과한다.
