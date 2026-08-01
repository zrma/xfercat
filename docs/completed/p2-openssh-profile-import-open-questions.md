# Resolved And Deferred Questions

## Resolved

- automatic discovery는 OpenSSH subprocess를 실행하지 않고 syntax-level alias만 읽는다.
- imported profile은 read-only alias reference이며 effective endpoint/authentication은
  `OpenSSH policy`로 표시한다.
- manual add/edit은 config에 없는 destination을 위한 fallback으로 유지한다.

## Deferred

- actual connection 직전 effective config를 OpenSSH process에 위임할지 Rust transport
  adapter가 해석할지는 transport interoperability decision이 소유한다.
- conditional `Include`, `Match`, host verification과 agent/key selection은 그 decision 전에는
  importer가 해석하지 않는다.
