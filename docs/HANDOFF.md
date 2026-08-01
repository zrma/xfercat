# Handoff

## Start Here

`xfercat`은 connection picker와 preview-first `TransferPlan`을 핵심으로 하는 파일 전송
도구다. Rust/Ratatui PoC가 actual local/remote browser와 핵심 interaction을 검증한다.

## Current State

- Git-backed colocated `jj` repository가 준비됐다.
- product, architecture, publication과 agent harness contract가 문서화됐다.
- Connections, local/remote Browser, Waybill add/remove, destination rename, stable reorder와
  execution Review가 동작한다.
- Connections에서 manual profile을 process lifetime 동안 추가·편집·삭제할 수 있다.
  edit은 stable identity와 이미 staged된 endpoint를 보존하고, delete는 staged reference가
  있으면 non-cascading으로 차단하며 active synthetic connection만 안전하게 해제한다.
- 일반 실행은 user OpenSSH config와 global includes의 concrete aliases를 자동으로 가져오며,
  `I` refresh와 read-only provenance를 제공한다. discovery는 subprocess와 network를 사용하지 않는다.
- staged file kind와 expected size를 보존하고 exact endpoint를 검증하는 typed
  `TransportRequest` 및 raw diagnostic text가 없는 typed result boundary가 준비됐다.
- interactive Review의 명시적 action이 actual upload/download executor를 실행해 item별
  succeeded, failed와 skipped result를 보존한다. terminal item은 암묵적으로 재실행하지 않는다.
  deterministic snapshot은 별도 fixture executor로 all-terminal rendering을 검증한다.
- actual remote session은 system OpenSSH, strict known-host verification, batch authentication와
  `openssh-sftp-client`를 사용한다. imported alias는 effective user config를 system `ssh`에
  위임하고 manual profile은 agent 또는 key-reference path를 사용한다.
- runtime LOCAL pane은 current working directory의 canonical actual entries를 읽고 Enter와
  Backspace로 이동한다. symlink, non-Unicode와 unreadable entry는 staging에서 제외한다.
- runtime REMOTE pane은 canonical SFTP home과 child/parent directory를 실제로 읽는다. remote
  symlink, special file, non-Unicode와 unsafe name도 staging에서 제외하며 raw SSH diagnostic은
  stable failure status로 변환한다.
- staged request는 destination missing/kind/size expectation을 freeze한다. actual transfer는
  source와 destination을 재검증하고 sibling temp, verified close/size와 atomic hard-link 또는
  rename으로 finalization한다. `ASK`/`RENAME` 충돌은 write 없이 실패하고 `SKIP`은 건너뛰며
  unchanged existing regular file만 explicit `OVERWRITE`할 수 있다. remote upload는 destination
  상태에 맞는 `hardlink` 또는 `posix-rename` extension이 없으면 write 전에 거부한다.
- `--snapshot`과 Ratatui `TestBackend`가 110×32 및 80×24 representative state를 검증한다.
- canonical validation은 `scripts/check.sh`가 소유한다.
- public remote가 구성됐다. license와 최종 GUI/TUI 제품 선택은
  결정되지 않았다.

## Next Work

다음 vertical slice를 선택하기 전에 `docs/roadmap.md`의 P1 잔여 acceptance를 검토한다.

권장 순서는 다음과 같다.

1. progress/cancellation contract와 user-visible typed failure detail을 설계한다.
2. retry/resume 또는 persistent plan 중 다음 reliability slice를 선택한다.

## Boundaries

- 구현되지 않은 기능을 status나 README에서 available로 표시하지 않는다.
- private host, credential 또는 실제 user path를 fixture나 tracked 문서에 사용하지 않는다.
- directory/symlink, progress/cancellation과 retry를 implemented로 표현하지 않는다.
- 추가 push, visibility 변경, license와 package publication은 별도 승인 경계다.

## Verify

```sh
scripts/check.sh
jj status
jj diff
```
