# Handoff

## Start Here

`xfercat`은 connection picker와 preview-first `TransferPlan`을 핵심으로 하는 파일 전송
도구다. Rust/Ratatui synthetic PoC가 핵심 interaction을 실행 가능하게 검증한다.

## Current State

- Git-backed colocated `jj` repository가 준비됐다.
- product, architecture, publication과 agent harness contract가 문서화됐다.
- Connections, local/remote Browser, Waybill add/remove, destination rename, stable reorder와
  dry-run Review가 동작한다.
- `--snapshot`과 Ratatui `TestBackend`가 110×32 및 80×24 representative state를 검증한다.
- canonical validation은 `scripts/check.sh`가 소유한다.
- remote, license, transport library와 최종 GUI/TUI 제품 선택은 결정되지 않았다.

## Next Work

다음 vertical slice를 선택하기 전에 `docs/roadmap.md`의 P1 잔여 acceptance를 검토한다.

권장 순서는 다음과 같다.

1. filesystem mutation 없이 typed transport request/result boundary를 추가한다.
2. item-level success, failure, skip과 cancellation state transition을 synthetic executor로 검증한다.
3. SFTP library와 OpenSSH interoperability를 별도 decision으로 비교한다.

## Boundaries

- 구현되지 않은 기능을 status나 README에서 available로 표시하지 않는다.
- private host, credential 또는 실제 user path를 fixture나 tracked 문서에 사용하지 않는다.
- PoC Review를 실제 transfer success로 표현하지 않는다.
- remote 생성, push, license와 package publication은 현재 범위가 아니다.

## Verify

```sh
scripts/check.sh
jj status
jj diff
```
