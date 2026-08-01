# xfercat

Plan first. Transfer once.

`xfercat`은 저장된 연결을 실수 없이 선택하고, 로컬과 원격 사이의 파일 전송을
실행 전에 하나의 명시적인 계획으로 검토하는 파일 전송 도구다.

현재 상태는 actual local/remote browser와 regular-file upload/download가 연결된 TUI PoC다.
system OpenSSH의 strict host verification과 SFTP를 사용하며, Review의 명시적 `Enter`가
actual regular-file upload/download를 item별로 실행한다.

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
cargo run -- --ssh-config <path>
cargo run -- --snapshot connections
cargo run -- --snapshot openssh
cargo run -- --snapshot openssh-empty
cargo run -- --snapshot profile-add
cargo run -- --snapshot profile-edit
cargo run -- --snapshot workspace
cargo run -- --snapshot rename
cargo run -- --snapshot review
cargo run -- --snapshot live-review
```

일반 실행은 user OpenSSH config와 global `Include`에서 concrete `Host` alias를 자동으로
가져온다. wildcard, negated pattern과 conditional include는 picker entry로 만들지 않는다.
imported profile은 read-only alias reference이며 host, user, key와 effective config를 복제하거나
해석하지 않는다. profile을 선택하면 system OpenSSH가 strict known-host와 batch mode로 실제
SFTP session을 열며, runtime LOCAL과 REMOTE pane은 실제 directory를 read-only로 탐색한다.
`--ssh-config`는 별도 OpenSSH config를 discovery와 connection 양쪽에 동일하게 적용한다.
Waybill은 staging 시 destination의 missing/file/directory와 size expectation을 freeze한다.
Review 실행은 source와 destination을 다시 검증하고 sibling temporary file에 쓴 뒤 close,
size 확인과 atomic finalization을 거쳐 final name을 노출한다. remote upload의 missing
destination은 SFTP `hardlink` extension, explicit overwrite는 `posix-rename` extension이
있을 때만 write를 시작한다. atomic publish capability가 없으면 해당 item은 변경 없이 실패한다.

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
- `Enter`: profile 저장 또는 selected profile에 strict SFTP connection
- `Esc`: form 취소

Workspace controls:

- `Tab`: LOCAL, REMOTE와 WAYBILL focus 이동
- `Enter`: focused directory 열기
- `Backspace`: parent directory로 이동
- `Space` / `S`: selected browser item을 Waybill에 추가
- `N`: destination filename rename
- `Shift+K` / `Shift+J`: selected Waybill item reorder
- `P`: conflict policy 변경
- `R`: exact transfer plan Review
- `Enter` in Review: staged regular-file upload/download 실제 실행

현재 transfer는 regular file만 지원한다. directory/symlink transfer, progress UI, 실행 중
cancellation, resume/retry와 persistent plan은 아직 구현하지 않았다. `ASK`와 `RENAME`은
existing destination을 자동 변경하지 않는다. `P`로 explicit `OVERWRITE`/`SKIP`을 고르거나
`N`으로 검토 가능한 exact destination name을 먼저 바꿔야 한다.

## Repository Workflow

```sh
scripts/check.sh
scripts/start-work.sh --work-id <work-id>
scripts/finalize-change.sh --verify-only
```

로컬 변경은 `jj`를 사용한다. 추가 push, license 선택과 release는 별도 결정이다.
