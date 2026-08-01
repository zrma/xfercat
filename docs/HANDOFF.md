# Handoff

## Start Here

`xfercat`은 connection picker와 preview-first `TransferPlan`을 핵심으로 하는 파일 전송
도구다. Rust/Ratatui synthetic PoC가 핵심 interaction을 실행 가능하게 검증한다.

## Current State

- Git-backed colocated `jj` repository가 준비됐다.
- product, architecture, publication과 agent harness contract가 문서화됐다.
- Connections, local/remote Browser, Waybill add/remove, destination rename, stable reorder와
  dry-run Review가 동작한다.
- Connections에서 synthetic profile을 process lifetime 동안 추가·편집·삭제할 수 있다.
  edit은 stable identity와 이미 staged된 endpoint를 보존하고, delete는 staged reference가
  있으면 non-cascading으로 차단하며 active synthetic connection만 안전하게 해제한다.
- 일반 실행은 user OpenSSH config와 global includes의 concrete aliases를 자동으로 가져오며,
  `I` refresh와 read-only provenance를 제공한다. discovery는 subprocess와 network를 사용하지 않는다.
- staged file kind와 expected size를 보존하고 exact endpoint를 검증하는 typed
  `TransportRequest` 및 raw diagnostic text가 없는 typed result boundary가 준비됐다.
- Review의 명시적 action이 representative synthetic executor를 실행해 item별 succeeded,
  failed, skipped와 cancelled result를 보존한다. terminal item은 암묵적으로 재실행하지 않는다.
- actual transport는 system OpenSSH, strict known-host verification, batch authentication와
  `openssh-sftp-client`를 사용하기로 결정했다. 구현은 아직 연결되지 않았다.
- `--snapshot`과 Ratatui `TestBackend`가 110×32 및 80×24 representative state를 검증한다.
- canonical validation은 `scripts/check.sh`가 소유한다.
- public remote가 구성됐다. license, transport library와 최종 GUI/TUI 제품 선택은
  결정되지 않았다.

## Next Work

다음 vertical slice를 선택하기 전에 `docs/roadmap.md`의 P1 잔여 acceptance를 검토한다.

권장 순서는 다음과 같다.

1. actual local filesystem browser와 path-safe staging을 연결한다.
2. strict OpenSSH/SFTP session과 remote browser를 연결한다.
3. temporary sibling, verified close와 rename을 사용하는 upload/download execution을 연결한다.

## Boundaries

- 구현되지 않은 기능을 status나 README에서 available로 표시하지 않는다.
- private host, credential 또는 실제 user path를 fixture나 tracked 문서에 사용하지 않는다.
- PoC Review를 실제 transfer success로 표현하지 않는다.
- 추가 push, visibility 변경, license와 package publication은 별도 승인 경계다.

## Verify

```sh
scripts/check.sh
jj status
jj diff
```
