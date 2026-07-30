# Spec: P0 Product Foundation

Status: planned

## Goal

첫 executable vertical slice에 사용할 runtime과 interface를 결정하고, 실제 private host 없이
connection picker, browser와 Waybill semantics를 검증할 수 있는 test boundary를 만든다.

## Scope

- TUI와 GUI 후보를 같은 representative workflow로 비교한다.
- runtime, UI toolkit과 project layout을 결정한다.
- domain types와 synthetic fixture boundary를 정의한다.
- 첫 vertical slice의 executable acceptance test를 추가한다.

## Constraints

- UI toolkit이 domain이나 transport semantics를 소유하지 않는다.
- credential content와 실제 private endpoint는 fixture에 사용하지 않는다.
- network transport 구현은 이 work item의 필수 범위가 아니다.
- dependency 추가는 maintenance, security와 distribution cost를 함께 평가한다.

## Acceptance Checklist

| ID | Status | Verify | Work item |
| --- | --- | --- | --- |
| C1 | todo | decision record review | runtime과 interface 선택 근거가 비교 가능한 evidence를 포함한다 |
| C2 | todo | focused domain tests | profile selection이 profile state를 암묵적으로 수정하지 않는다 |
| C3 | todo | focused domain tests | plan item이 browser focus와 독립된 exact endpoints를 유지한다 |
| C4 | todo | rendered or snapshot smoke | Connections, Browser와 Waybill의 representative state를 확인한다 |
| C5 | todo | `scripts/check.sh` | repository 전체 gate가 통과한다 |

## Required Evidence

- 선택한 runtime과 toolkit의 최소 prototype 또는 공식 capability evidence
- synthetic fixtures를 사용하는 test 결과
- representative UI state의 rendered, snapshot 또는 terminal smoke
- dependency와 packaging 영향

## Publication Impact

repository-owned decision, source와 synthetic fixture만 tracked artifact에 남긴다. remote write는
포함하지 않는다.

## Out Of Scope

- 실제 private server 연결
- credential 저장
- remote 생성, push, license와 release
- full network transfer implementation

## Completion Rule

모든 acceptance가 executable evidence와 함께 done이고 `scripts/check.sh`가 통과한다.
