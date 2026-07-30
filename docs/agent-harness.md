# Agent Harness

## Interface

- Structure ID: `agent-harness-v1`.
- Baseline ID: `openai-gpt-5.6-2026-07-11`.
- Convergence stage: `canonical`.
- Target stage: `canonical`.
- Canonical check: `scripts/check-agent-harness-interface.sh`.
- Publication class: `public`.
- Publication boundary check: `scripts/check-publication-boundary.py`.

`AGENTS.md`가 공통 GPT-5.6 계약을 소유하고, 이 문서는 xfercat product와 current
milestone으로 가는 canonical 진입점이다.

Publication class는 live remote visibility가 아니라 tracked artifact의 content 기준이다.
remote가 없거나 private여도 repository gate는 public-ready 기준으로 실행한다. remote 생성,
visibility 변경, push와 license 선택은 각각 별도 결정이다.

Tracked artifact contract: raw tool output와 정확한 로컬 환경 evidence는 local-only로
취급한다. 공개 가능한 기록에는 repository-owned 결정, 필요한 명령 이름과 redacted 검증
판정만 남기고 path, host, address와 credential은 placeholder로 바꾼다.

## Project Objective

저장된 연결을 실수 없이 선택하고, 로컬과 원격 사이의 파일 작업을 독립된 transfer plan으로
구성, 검토, 수정한 뒤 명시적으로 일괄 실행하는 파일 전송 경험을 제공한다.

## Source Of Truth

- 사용자 문제와 MVP 경계: `docs/PRODUCT.md`.
- domain, transport와 security boundary: `docs/ARCHITECTURE.md`.
- 현재 구현과 리스크: `docs/status.md`; 우선순위: `docs/roadmap.md`.
- 무컨텍스트 시작점: `docs/HANDOFF.md`; 현재 작업: manifest의 `active_work`.
- publication 계약: `docs/PUBLICATION.md`.
- 검증 선언: `docs/REPO_MANIFEST.yaml`과 `scripts/check.sh`.

## Autonomy And Permissions

- 목표와 acceptance가 명확한 local, reversible 작업은 추가 승인 없이 구현, 검증,
  문서화와 local `jj` change 정리까지 진행한다.
- external write, secret, 비용, 파괴적 작업, product 방향 변경, published history rewrite와
  승인되지 않은 push는 에스컬레이션한다.
- private host나 credential 없이 fixture와 transport boundary로 진행 가능한 작업은 synthetic
  data로 직접 검증한다.
- public artifact에는 prompt, transcript, memory, raw tool output를 남기지 않고
  repository-owned decision, test와 redacted evidence만 남긴다.

## Execution Loop

1. `jj status`, handoff, status, roadmap와 활성 todo를 확인한다.
2. 현재 acceptance와 connection, browser, transfer-plan, transport 중 변경 경계를 고정한다.
3. 비사소한 새 작업은 `scripts/start-work.sh --work-id <work-id>`로 spec을 만든다.
4. failing test 또는 executable check를 먼저 만들고 가장 작은 vertical slice를 구현한다.
5. focused check에서 시작해 `scripts/check.sh`까지 검증 범위를 넓힌다.
6. durable 상태만 status, roadmap, completed milestone 또는 todo에 반영한다.
7. 하나의 목적을 가진 local `jj` change로 닫고 external write 전에는 승인을 받는다.

## Verification And Evidence

- 전체 local gate: `scripts/check.sh`.
- harness interface: `scripts/check-agent-harness-interface.sh`.
- repository contract: `scripts/check-repository-contract.py`.
- publication boundary: `scripts/check-publication-boundary.py`.
- domain 변경은 transfer direction, source와 destination, conflict policy, cancellation과 partial
  failure 상태를 test한다.
- transport 변경은 synthetic local/remote fixture, timeout, cancellation, host verification과
  credential redaction을 test한다.
- UI 변경은 connection picker, local/remote browser, Waybill과 execution review의 대표 상태를
  render하고 keyboard 또는 pointer smoke를 수행한다.
- 최종 evidence에는 acceptance별 명령, user-visible 결과, 남은 risk와 local/remote 상태를
  구분해 포함한다.

## Escalation

credential이나 private context, runtime 또는 trust model의 product 선택, 비용, destructive
external write, remote 생성, license, published history rewrite와 승인되지 않은 push가
필요할 때만 사용자에게 최소 판단을 요청한다.

## VCS And Publish

- local VCS는 `jj`를 사용하고 change description은 `<type>: <summary>`와 configured
  attribution trailer 규칙을 따른다.
- change는 independently explainable하고 검증 가능한 milestone 단위로 유지한다.
- push, tag, release, remote 생성과 license는 별도 external decision 또는 write 경계다.
- remote visibility를 추측하지 않는다. publish 전에는 live destination과 visibility를
  확인하고 repository gate 및 권한 있는 machine-local gate를 실행한다.

## Harness Evaluation And Improvement

- instruction을 추가하기 전에 실제 실패가 product, code, test 또는 prompt 중 어디에 있는지
  분류한다.
- 같은 대표 task와 acceptance로 변경 전후의 completeness, evidence, reliability와 비용을
  비교한다.
- 다른 문서와 중복되는 일반 규칙은 제거하고 xfercat 고유 경계만 overlay에 유지한다.

## Convergence

- 이 저장소는 `agent-harness-v1` canonical stage다.
- baseline 또는 section contract 변경은 공식 OpenAI 문서와 repository check를 함께 갱신한다.
- 단계 전환은 현재 저장소의 Structure ID, 섹션 순서, canonical check 결과로 검증하며 다른 저장소의 이름, 개수, 로컬 경로나 공개 여부를 전제하지 않는다.

## Project Overlay

- 접속 화면의 기본 행위는 profile 선택이며, edit는 명시적인 별도 mode다.
- `TransferPlan`은 browser selection과 분리된 first-class state다.
- 각 `TransferPlanItem`은 전체 endpoint, 방향, conflict policy와 실행 상태를 표시할 수 있어야 한다.
- 실행 성공, 실패, skip과 보류 상태를 항목별로 유지하며 전체 queue를 암묵적으로 비우지 않는다.
- runtime과 UI toolkit 선택 전에도 domain contract와 fixture는 interface-agnostic하게 유지한다.

## Related Documents

- `docs/HANDOFF.md`
- `docs/PRODUCT.md`
- `docs/ARCHITECTURE.md`
- `docs/PUBLICATION.md`
- `docs/status.md`
- `docs/roadmap.md`
- `docs/completed-milestones.md`
