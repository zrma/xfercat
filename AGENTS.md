# xfercat Agent Guide

이 파일은 짧은 bootstrap map이다. 공통 실행 계약과 xfercat 상세 운영 규칙은
repository-owned 문서가 소유한다.

## First Read

- 공통 하네스 인터페이스와 product overlay: `docs/agent-harness.md`.
- 현재 상태와 다음 순서: `docs/HANDOFF.md`, `docs/status.md`, `docs/roadmap.md`.
- 제품과 아키텍처 경계: `docs/PRODUCT.md`, `docs/ARCHITECTURE.md`.
- 현재 작업: `docs/REPO_MANIFEST.yaml`의 `active_work`.

<!-- agent-harness-baseline:start -->
## Agent Harness Baseline (GPT-5.6)

Baseline ID: `openai-gpt-5.6-2026-07-11`.

- Source of truth: use the `openai-docs` skill and the official [latest model guide](https://developers.openai.com/api/docs/guides/latest-model) plus [GPT-5.6 prompting guidance](https://developers.openai.com/api/docs/guides/prompt-guidance-gpt-5p6) before changing OpenAI model, API, prompt, or agent guidance.
- Model target: when a task asks for the current OpenAI baseline, use `gpt-5.6`. This is harness guidance, not proof that the application calls OpenAI; change runtime model strings only at an existing OpenAI integration point.
- Prompt budget: state the outcome, success criteria, evidence and permission boundaries with the smallest task-relevant instruction and tool set. Remove redundant generic rules and add examples only for an observed failure.
- Request modes: for answer, explain, review, diagnose, or plan requests, inspect and report without implementation. For change, build, or fix requests, make the requested in-scope local changes and run relevant non-destructive validation.
- Permissions: reading, searching, editing in-scope files, and running non-destructive checks are pre-authorized for change tasks. Require confirmation for external writes not explicitly requested, destructive or irreversible actions, purchases or cost, secrets, or material scope expansion.
- Persistence: continue until the requested outcome is complete; do not stop after only analysis, a partial patch, or an intermediate tool success. Stop and escalate only at a real permission, product-decision, or external-state boundary.
- Verification: treat tool and patch success as provisional. Re-read the diff and verify the user-visible or runtime outcome with the narrowest meaningful checks, then broaden only when risk warrants it.
- Publication boundary: before a public push, tag or release, visibility change, or published-history rewrite, run the repository boundary check and any authorized machine-local private-inventory check. Keep private inventory outside published repositories and CI configuration.
- Tracked-artifact privacy: treat tool output, memory-derived environment context, local absolute paths, machine, host or cluster identifiers, internal endpoints or addresses, and full diagnostic logs as local-only by default. Retain repository-owned decisions and redacted verification outcomes with placeholders such as `<repo-root>`, `<private-host>`, `<internal-ip>`, and `<cluster-context>`.
- Output: lead with the conclusion. Include required evidence, material caveats, and the next action; trim introductions, repetition, generic reassurance, and optional background before required content.
- Structure: use a lightweight task-specific plan or output shape. Do not impose a global template or long process narration when the repository already supplies the necessary workflow.
- Evaluation: retain harness instructions only when repository checks or representative tasks show they improve final-answer completeness, evidence quality, reliability, latency, or cost. Evaluate the final result, not just tool-call count.
- Project overlay: the remaining sections of this file and linked project docs define domain-specific architecture, tests, safety boundaries, escalation rules, and publish gates. They may specialize this baseline but must not silently weaken its permission or evidence requirements.
<!-- agent-harness-baseline:end -->

## Project Overlay

- 구현 여부는 code, test와 user-visible smoke가 기준이며 planned 기능을 implemented로 표시하지 않는다.
- connection profile은 접속 descriptor와 credential reference만 소유한다. secret, private key와 실제 host inventory는 저장소나 일반 application state에 복제하지 않는다.
- transfer는 source, destination, 방향, 충돌 정책과 상태가 명시된 `TransferPlanItem`을 거쳐야 한다. overwrite, delete와 external write는 실행 전 preview와 명시적 승인 경계를 유지한다.
- transport와 interface는 domain model에서 분리한다. TUI 또는 GUI 선택이 transfer semantics를 바꾸지 않게 한다.
- tracked surface는 remote visibility와 무관하게 public-ready로 유지한다. prompt, transcript, memory, raw tool output와 machine-local inventory를 tracked artifact로 옮기지 않는다.
- 기본 전체 검증은 `scripts/check.sh`; publication 경계는 `scripts/check-publication-boundary.py`로 확인한다.
- 로컬 VCS는 `jj`를 사용한다. remote 생성, push, license, package publish와 release는 명시적 결정이 있을 때만 수행한다.
