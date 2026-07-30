# Product

## Status

Product contract is defined; no functional client is implemented yet.

## Problem

기존 terminal file-transfer 도구는 연결 profile을 고르는 과정과 protocol, authentication,
field focus를 한 화면에 섞어 접속 자체를 오류가 잦은 form 조작으로 만든다. 파일을 여러
위치에서 선택할 수 있어도 그 상태가 source, destination, 방향과 충돌 정책을 검토하는
독립된 transfer plan으로 승격되지 않는 경우가 많다.

## Product Promise

접속은 한 번의 profile 선택으로 끝나고, 전송은 계획을 먼저 구성한 뒤 검토하고 실행한다.

Tagline: **Plan first. Transfer once.**

## Primary Workflow

1. Connections에서 완성된 `ConnectionProfile`을 선택해 접속한다.
2. Browser의 local/remote pane에서 파일과 디렉터리를 탐색한다.
3. 작업을 Waybill에 추가한다.
4. Waybill에서 source, destination, 방향, 이름과 conflict policy를 검토하거나 수정한다.
5. 실행 preview를 확인하고 일부 또는 전체 작업을 명시적으로 실행한다.
6. 성공, 실패, skip과 보류 결과를 항목별로 확인하고 필요한 작업만 재시도한다.

## Core Concepts

- **Connections**: profile 목록, 검색, 연결, 명시적 create/edit/delete.
- **Browser**: local과 remote endpoint를 같은 semantics로 탐색하는 pane.
- **Waybill**: persistent `TransferPlan`을 검토하고 실행하는 first-class surface.
- **TransferPlanItem**: source, destination, direction, conflict policy와 execution state의
  명시적인 결합.

## MVP Invariants

- profile row를 이동하거나 선택하는 동안 protocol과 authentication 설정은 바뀌지 않는다.
- connection edit는 connect action과 분리된 mode다.
- plan item은 browser의 현재 focus나 현재 directory가 바뀌어도 의미가 바뀌지 않는다.
- overwrite, rename, skip 같은 충돌 정책을 실행 전에 확인할 수 있다.
- partial failure가 다른 항목의 결과를 지우지 않는다.
- 실행과 destructive external write는 명시적 action이다.

## Non-goals

- 범용 desktop file manager 전체를 복제하지 않는다.
- credential manager나 private-key vault를 구현하지 않는다.
- bidirectional sync, filesystem mount와 background mirroring은 초기 범위가 아니다.
- protocol 수를 늘리는 것을 connection과 transfer-plan UX보다 우선하지 않는다.

## Open Product Decisions

- 첫 interface를 TUI로 할지 native 또는 webview GUI로 할지.
- 첫 transport 범위를 SFTP only로 할지 SCP-compatible operation까지 포함할지.
- plan persistence를 process lifetime으로 제한할지 안전한 local draft로 저장할지.

결정 조건과 acceptance는 `docs/todo-p0-product-foundation/spec.md`가 소유한다.
