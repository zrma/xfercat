## Project Overlay

- 접속 화면의 기본 행위는 profile 선택이며, edit는 명시적인 별도 mode다.
- `TransferPlan`은 browser selection과 분리된 first-class state다.
- 각 `TransferPlanItem`은 전체 endpoint, 방향, conflict policy와 실행 상태를 표시할 수 있어야 한다.
- 실행 성공, 실패, skip과 보류 상태를 항목별로 유지하며 전체 queue를 암묵적으로 비우지 않는다.
- PoC runtime과 무관하게 domain contract와 fixture는 interface-agnostic하게 유지한다.

## Related Documents

- `docs/HANDOFF.md`
- `docs/PRODUCT.md`
- `docs/ARCHITECTURE.md`
- `docs/PUBLICATION.md`
- `docs/status.md`
- `docs/roadmap.md`
- `docs/completed-milestones.md`
