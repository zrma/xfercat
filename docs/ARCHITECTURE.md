# Architecture

## Status

Domain boundary is proposed. Runtime, UI toolkit and transport library are not selected.

## Domain Boundary

```text
Connection Catalog -> Session
                         |
Local Endpoint -> Browser Model <- Remote Endpoint
                         |
                    Transfer Plan
                         |
                   Execution Engine
                         |
                  Transport Adapter
```

UI와 transport 구현은 아래 domain contract를 공유한다.

### ConnectionProfile

- stable profile identity
- display label
- protocol and endpoint descriptor
- username when applicable
- authentication reference
- host-verification policy reference

password, private key 내용, agent socket과 실제 private inventory를 profile payload나 tracked
fixture에 복제하지 않는다.

### TransferPlanItem

- stable item identity
- source endpoint and absolute logical path
- destination endpoint and absolute logical path
- upload or download direction
- item kind and expected size when known
- conflict policy: ask, overwrite, skip or rename
- execution state: staged, running, succeeded, failed, skipped or cancelled

plan은 browser focus와 분리되고 item edit, remove, reorder, subset execution과 retry를
지원할 수 있어야 한다.

### Transport Adapter

transport는 structured request와 typed result를 받는다. shell command 문자열을 조합하지
않고 credential material과 raw diagnostic output를 domain이나 UI log로 누출하지 않는다.

## Safety Invariants

- 연결 전에 exact profile, endpoint와 host-verification 상태를 보여준다.
- 실행 전에 exact source, destination, direction과 conflict policy를 freeze한다.
- 실행 직전 destination state가 preview와 달라졌다면 stale plan으로 처리한다.
- overwrite, remote delete와 rename conflict resolution은 명시적 policy 없이 실행하지 않는다.
- symlink, path traversal, case sensitivity와 platform path 차이를 transport 경계에서
  정규화하거나 오류로 노출한다.
- cancellation과 partial failure는 item별 결과를 보존한다.
- logs는 credential, private path와 endpoint를 redaction할 수 있는 structured event를 사용한다.

## Initial Vertical Slice

runtime 선택 뒤 첫 slice는 실제 private host 없이 synthetic local endpoint 두 개를 사용한다.
connection picker의 read-only profile 선택, 두 pane 탐색, Waybill item add/edit/remove와
dry-run execution preview를 end-to-end로 검증한다. network transport는 그 다음 slice에서
같은 domain contract에 연결한다.

## Deferred Decisions

- language and application runtime
- TUI or GUI toolkit
- async runtime and cancellation primitive
- SFTP library and OpenSSH interoperability boundary
- plan persistence format and migration policy
