# Resolved And Deferred Questions

`RENAME`은 자동으로 unreviewed path를 만들지 않고 `N`으로 exact destination을 바꾼 뒤
실행하는 policy로 유지했다. SFTP v3의 explicit overwrite는 kind/size를 finalization 직전
재검증하지만 atomic content compare-and-swap을 제공하지 않는 제한이 남는다.
