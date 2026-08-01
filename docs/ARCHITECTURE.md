# Architecture

## Status

The PoC uses a Rust 2024 core with a Ratatui/Crossterm validation shell. System OpenSSH with a
typed SFTP client implements the first Unix connection and read-only remote browser. Runtime and
transport rationale are recorded in `docs/decisions/0001-poc-runtime.md` and
`docs/decisions/0003-openssh-sftp-transport.md`.

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
- synthetic, manual 또는 imported OpenSSH provenance
- protocol and endpoint descriptor
- username when applicable
- authentication reference
- host-verification policy reference

password, private key 내용, agent socket과 실제 private inventory를 profile payload나 tracked
fixture에 복제하지 않는다.

Imported OpenSSH profile은 concrete alias reference만 소유한다. startup discovery는 user config와
global `Include`를 syntax-level로 읽되 `ssh -G`, `Match`, effective endpoint와 authentication을
평가하지 않는다. wildcard/negated patterns와 conditional includes는 picker entry에서 제외한다.
이 경계는 `docs/decisions/0002-openssh-profile-import.md`가 소유한다.

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
`TransportRequest` conversion은 source/destination logical path와 upload/download endpoint
role을 검증하고 item ID, entry kind, expected size와 conflict policy를 immutable snapshot으로
보존한다. `TransportResult`는 succeeded, skipped, failed 또는 cancelled outcome만 노출하며
failure는 stable category와 retryability만 domain에 전달한다.

### Local Filesystem Adapter

runtime local browser는 current working directory를 canonicalize하고 directory와 regular file만
노출한다. symlink, non-Unicode, control-character와 unreadable entry는 lossy conversion이나
follow 없이 제외한다. navigation은 read-only이고 staged item은 browser가 읽은 exact canonical
path, file kind와 size를 freeze한다.

### SFTP Adapter

runtime connection은 imported alias 또는 validated manual host를 system OpenSSH에 전달한다.
OpenSSH가 user config, agent, identity와 jump policy를 평가하며 adapter는 strict known-host
verification, batch authentication와 bounded timeout을 강제한다. SFTP browser는 remote path를
canonicalize하고 directory와 regular file만 노출한다. symlink, special file, non-Unicode,
control-character와 unreadable entry는 제외하고 raw SSH/SFTP diagnostic은 domain이나 UI에
전달하지 않는다. session은 Workspace를 나가거나 앱을 종료할 때 graceful close한다.

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

첫 product-foundation slice는 실제 private host 없이 synthetic local/remote endpoint를 사용했다. connection
picker의 OpenSSH alias import/refresh, profile 선택과 process-lifetime manual add/edit/delete,
두 pane 탐색, Waybill item add/edit/remove와 dry-run execution preview를 end-to-end로 검증한다.
profile edit은 stable identity와 staged endpoint를 바꾸지 않고, delete는 staged reference를
non-cascading으로 차단한다. representative synthetic executor는 staged item을 running을 거쳐
terminal state로 전이하고 partial result를 보존하지만 filesystem과 network를 호출하지 않는다.
`src/domain.rs`가 transfer types,
`src/app.rs`가 interaction state, `src/ui.rs`가 Ratatui rendering과 deterministic snapshot을
소유한다. 현재 runtime은 `src/localfs.rs`와 `src/sftp.rs`의 actual browser adapter를 사용하고,
file mutation은 후속 execution slice에서 같은 domain contract에 연결한다.

## Deferred Decisions

- final TUI or GUI product interface
- effective OpenSSH config, conditional `Match` and host-verification resolution
- plan persistence format and migration policy
