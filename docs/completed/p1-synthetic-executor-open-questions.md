# Resolved And Deferred Questions

이번 slice에는 owner decision이 필요한 미결 항목이 없다.

이번 executor는 representative outcome fixture로 state semantics만 검증한다. 실제 cancellation
primitive, retry eligibility와 progress contract는 async transport 선택 이후 별도 slice에서 정한다.
실제 adapter result를 적용할 때 mismatched item identity를 panic 대신 typed orchestration error로
처리하는 contract도 transport integration slice가 소유한다.
