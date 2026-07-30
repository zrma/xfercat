# Handoff

## Start Here

`xfercat`은 connection picker와 preview-first `TransferPlan`을 핵심으로 하는 파일 전송
도구다. repository와 AI-first harness는 초기화됐지만 product code는 아직 없다.

## Current State

- Git-backed colocated `jj` repository가 준비됐다.
- product, architecture, publication과 agent harness contract가 문서화됐다.
- repository-only validation은 `scripts/check.sh`가 소유한다.
- remote, license, runtime, UI toolkit과 transport library는 결정되지 않았다.

## Next Work

`docs/todo-p0-product-foundation/spec.md`를 실행한다.

1. TUI와 GUI 후보를 동일한 workflow acceptance로 비교한다.
2. runtime과 첫 UI toolkit을 결정하고 decision evidence를 남긴다.
3. synthetic local endpoints만 사용하는 첫 vertical slice의 executable test boundary를 만든다.

## Boundaries

- 구현되지 않은 기능을 status나 README에서 available로 표시하지 않는다.
- private host, credential 또는 실제 user path를 fixture나 tracked 문서에 사용하지 않는다.
- remote 생성, push, license와 package publication은 현재 범위가 아니다.

## Verify

```sh
scripts/check.sh
jj status
jj diff
```
