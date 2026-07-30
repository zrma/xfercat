# Spec: P0 Product Foundation

Status: completed

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
| C1 | done | `docs/decisions/0001-poc-runtime.md` | runtime과 interface 선택 근거가 비교 가능한 evidence를 포함한다 |
| C2 | done | `cargo test` | profile selection이 profile state를 암묵적으로 수정하지 않는다 |
| C3 | done | `cargo test` | plan item이 browser focus와 독립된 exact endpoints를 유지한다 |
| C4 | done | snapshots and PTY smoke | Connections, Browser와 Waybill의 representative state를 확인한다 |
| C5 | done | `scripts/check.sh` | repository 전체 gate가 통과한다 |

## Required Evidence

- Rust/Ratatui/Crossterm 결정과 공식 capability reference: `docs/decisions/0001-poc-runtime.md`
- synthetic domain and interaction tests: `src/app.rs`, `tests/interaction.rs`
- representative render: `--snapshot connections|workspace|review`
- compact terminal regression: 80×24 `TestBackend` snapshot
- live interaction: Connections부터 Review와 terminal restore까지 PTY smoke
- dependency and packaging impact: Cargo binary only; app packaging은 미선택

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
