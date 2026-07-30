# Decision 0001: PoC Runtime And Interface

Status: accepted for the product-foundation PoC

## Decision

첫 executable vertical slice는 Rust 2024 library와 binary, Ratatui 0.30.2,
Crossterm 0.29.0으로 구성한 TUI shell로 구현한다.

이 결정은 최종 제품 UI를 TUI로 확정하지 않는다. `ConnectionProfile`,
`TransferPlanItem`과 application state를 UI-independent Rust module에 두고,
Ratatui는 connection picker, Browser, Waybill과 review flow를 검증하는 첫 adapter다.

## Compared Options

### Rust Core With Ratatui TUI

- 기존 terminal workflow와 같은 환경에서 keyboard focus와 state visibility를 직접 검증한다.
- Ratatui `TestBackend`가 전체 terminal UI를 memory buffer에 render하므로 deterministic
  interaction snapshot을 test할 수 있다.
- Ratatui의 `run` entrypoint가 raw mode, alternate screen과 panic 시 restore를 관리한다.
- 별도 app bundle이나 webview 없이 단일 binary로 PoC를 실행할 수 있다.

References:

- [Ratatui crate documentation](https://docs.rs/ratatui/0.30.2/ratatui/)
- [Ratatui TestBackend](https://docs.rs/ratatui/0.30.2/ratatui/backend/struct.TestBackend.html)
- [Crossterm events](https://docs.rs/crossterm/0.29.0/crossterm/event/)

### Desktop GUI Shell

- connection rows, drag/drop과 pointer discoverability를 더 직접적으로 검증할 수 있다.
- packaging, window lifecycle, webview 또는 native toolkit 선택이 domain validation보다 먼저
  PoC 범위를 넓힌다.
- 같은 domain core 위에 후속 adapter로 비교할 수 있다.

## Consequences

- PoC는 synthetic profile과 endpoint만 사용하며 network transport를 연결하지 않는다.
- `--snapshot` entrypoint로 Connections, Workspace와 Review state를 terminal 없이 render한다.
- interactive shell과 snapshot test는 같은 render function과 application state를 사용한다.
- async runtime, SFTP library, persistence와 distribution은 후속 decision이다.

## Revisit Trigger

TUI PoC에서 connection selection이나 Waybill review가 명확해도 pointer discoverability,
drag/drop 또는 platform file integration이 핵심 acceptance를 좌우한다면 동일 Rust core 위에
GUI adapter prototype을 만들어 비교한다.
