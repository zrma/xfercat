# Resolved And Deferred Questions

## Runtime And Interface

- TUI는 connection picker와 Waybill state를 검증하는 첫 PoC surface로 충분하다.
- GUI discoverability의 추가 가치는 PoC 결과 뒤 별도 adapter로 비교하며 최종 선택은 보류한다.
- Rust library가 domain/application state를, binary와 Ratatui module이 interface를 소유한다.

## Transport

- 첫 network slice를 SFTP only로 제한할지는 transport decision으로 보류한다.
- OpenSSH config, agent와 host-verification interoperability도 transport decision이 소유한다.

## Persistence

- process 종료 뒤 draft plan persistence는 P3 reliability decision으로 보류한다.
- persistence를 추가하기 전 endpoint/path privacy와 migration contract를 먼저 정의한다.
