# xfercat

Plan first. Transfer once.

`xfercat`은 저장된 연결을 실수 없이 선택하고, 로컬과 원격 사이의 파일 전송을
실행 전에 하나의 명시적인 계획으로 검토하는 파일 전송 도구다.

현재 상태는 실행 가능한 synthetic TUI PoC다. 실제 filesystem mutation과 network file
transfer는 아직 구현되지 않았다.

## Product Direction

- 연결은 완성된 `ConnectionProfile`을 고르는 행위다.
- 파일 선택은 즉시 전송이 아니라 독립된 `TransferPlan`에 작업을 추가한다.
- 각 작업은 source, destination, 방향, 충돌 정책과 실행 상태를 명시한다.
- 실제 전송은 전체 계획을 검토한 뒤 명시적으로 실행한다.

제품 경계는 [Product](docs/PRODUCT.md), 현재 상태와 다음 작업은
[Handoff](docs/HANDOFF.md)에서 확인한다.

## Run The PoC

```sh
cargo run
cargo run -- --snapshot connections
cargo run -- --snapshot profile-add
cargo run -- --snapshot profile-edit
cargo run -- --snapshot workspace
cargo run -- --snapshot rename
cargo run -- --snapshot review
```

PoC의 profile, path와 전송 결과는 모두 synthetic이다. Review는 dry-run이며 파일을
전송하거나 덮어쓰지 않는다. 추가·편집한 profile은 현재 process에서만 유지되며 앱을
재시작하면 초기 fixture로 돌아간다.

Connection controls:

- `A`: synthetic profile 추가
- `E`: selected profile 편집
- `Tab` / `Shift+Tab`: profile form field 이동
- `Left` / `Right`: SSH Agent와 key reference 전환
- `Enter`: profile 저장 또는 selected profile 연결
- `Esc`: form 취소

Workspace controls:

- `Tab`: LOCAL, REMOTE와 WAYBILL focus 이동
- `Space`: selected browser item을 Waybill에 추가
- `N`: destination filename rename
- `Shift+K` / `Shift+J`: selected Waybill item reorder
- `P`: conflict policy 변경
- `R`: exact transfer plan Review

## Repository Workflow

```sh
scripts/check.sh
scripts/start-work.sh --work-id <work-id>
scripts/finalize-change.sh --verify-only
```

로컬 변경은 `jj`를 사용한다. 추가 push, license 선택과 release는 별도 결정이다.
