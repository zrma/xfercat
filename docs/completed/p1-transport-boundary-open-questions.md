# Resolved And Deferred Questions

이번 slice에는 owner decision이 필요한 미결 항목이 없다.

실제 adapter가 선택되기 전까지 result failure는 stable category와 retryability만 소유한다.
library-specific error와 raw diagnostic string은 boundary 밖 adapter-local evidence로 남긴다.
