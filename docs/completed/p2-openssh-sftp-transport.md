# Spec: OpenSSH SFTP Transport Decision

Status: completed

## Goal

imported OpenSSH alias와 manual credential reference를 실제 connection에 연결할 transport,
host verification과 remote-write safety contract를 선택한다.

## Scope

- system OpenSSH compatibility boundary
- strict known-host verification and batch authentication
- SFTP v3 client selection
- temporary sibling and rename write strategy
- cancellation limitation and platform support boundary

## Constraints

- 기존 사용자 변경과 repository contract를 보존한다.
- private host, user path, key와 inventory를 tracked artifact에 기록하지 않는다.
- unknown host key를 자동 수락하지 않는다.
- interactive password prompt와 credential material storage를 추가하지 않는다.

## Acceptance Checklist

| ID | Status | Verify | Work item |
| --- | --- | --- | --- |
| C1 | done | official crate docs | OpenSSH config/agent/key compatibility를 확인한다 |
| C2 | done | official crate docs | strict/add/accept host-key semantics를 확인한다 |
| C3 | done | official crate docs | SFTP mutation cancellation 한계를 기록한다 |
| C4 | done | repository review | selected/rejected alternative와 platform boundary를 ADR로 고정한다 |
| C5 | done | `scripts/check.sh` | canonical repository gate가 통과한다 |

## Required Evidence

- current crates.io metadata and official rustdoc
- OpenSSH client version and local isolated-fixture capability check
- passing canonical `scripts/check.sh`

## Publication Impact

- public upstream documentation과 repository-owned decision만 tracked artifact에 남긴다.
- private inventory, actual alias와 raw command output을 기록하지 않는다.
- remote write는 포함하지 않는다.

## Out Of Scope

- connection and file-transfer implementation
- user remote host connection or mutation
- remote 생성, push, license와 release

## Completion Rule

모든 acceptance가 evidence와 함께 done이고 전체 gate가 통과한다.
