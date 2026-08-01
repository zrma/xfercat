# xfercat

Plan first. Transfer once.

`xfercat`은 저장된 연결을 실수 없이 선택하고, 로컬과 원격 사이의 파일 전송을
실행 전에 하나의 명시적인 계획으로 검토하는 파일 전송 도구다.

현재 상태는 실행 가능한 synthetic TUI PoC다. item-level success/failure/skip/cancellation
state는 synthetic executor로 확인할 수 있지만 실제 filesystem mutation과 network file
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
cargo run -- --snapshot openssh
cargo run -- --snapshot openssh-empty
cargo run -- --snapshot profile-add
cargo run -- --snapshot profile-edit
cargo run -- --snapshot workspace
cargo run -- --snapshot rename
cargo run -- --snapshot review
```

일반 실행은 user OpenSSH config와 global `Include`에서 concrete `Host` alias를 자동으로
가져온다. wildcard, negated pattern과 conditional include는 picker entry로 만들지 않는다.
imported profile은 read-only alias reference이며 host, user, key와 effective config를 복제하거나
해석하지 않는다. runtime LOCAL pane은 실제 current directory를 read-only로 탐색한다. REMOTE
pane과 전송 결과는 아직 synthetic이고 Review의 실행 action도 typed state transition만
검증하며 실제 I/O를 수행하지 않는다.

수동으로 추가·편집·삭제한 profile은 현재 process에서만 유지되며 앱을 재시작하면
OpenSSH import와 빈 manual catalog에서 다시 시작한다. staged Waybill item이 참조하는
profile은 item을 먼저 제거해야 삭제할 수 있다.

Connection controls:

- `I`: OpenSSH config alias 새로고침
- `A`: process-local manual profile 추가
- `E`: manual profile 편집; imported profile은 source config에서 관리
- `D`: manual profile 삭제; imported profile과 staged-reference profile은 안전하게 차단
- `Tab` / `Shift+Tab`: profile form field 이동
- `Left` / `Right`: SSH Agent와 key reference 전환
- `Enter`: profile 저장 또는 selected profile을 synthetic workspace에 선택
- `Esc`: form 취소

Workspace controls:

- `Tab`: LOCAL, REMOTE와 WAYBILL focus 이동
- `Enter`: focused directory 열기
- `Backspace`: parent directory로 이동
- `Space`: selected browser item을 Waybill에 추가
- `N`: destination filename rename
- `Shift+K` / `Shift+J`: selected Waybill item reorder
- `P`: conflict policy 변경
- `R`: exact transfer plan Review
- `Enter` in Review: synthetic item results 실행; 실제 파일은 변경하지 않음

## Repository Workflow

```sh
scripts/check.sh
scripts/start-work.sh --work-id <work-id>
scripts/finalize-change.sh --verify-only
```

로컬 변경은 `jj`를 사용한다. 추가 push, license 선택과 release는 별도 결정이다.
