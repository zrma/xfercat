## Repository Overlay

- 구현 여부는 code, test와 user-visible smoke가 기준이며 planned 기능을 implemented로 표시하지 않는다.
- connection profile은 접속 descriptor와 credential reference만 소유한다. secret, private key와 실제 host inventory는 저장소나 일반 application state에 복제하지 않는다.
- transfer는 source, destination, 방향, 충돌 정책과 상태가 명시된 `TransferPlanItem`을 거쳐야 한다. overwrite, delete와 external write는 실행 전 preview와 명시적 승인 경계를 유지한다.
- transport와 interface는 domain model에서 분리한다. TUI 또는 GUI 선택이 transfer semantics를 바꾸지 않게 한다.
- tracked surface는 remote visibility와 무관하게 public-ready로 유지한다. prompt, transcript, memory, raw tool output와 machine-local inventory를 tracked artifact로 옮기지 않는다.
- 기본 전체 검증은 `scripts/check.sh`; publication 경계는 `scripts/check-publication-boundary.py`로 확인한다.
- 로컬 VCS는 `jj`를 사용한다. remote 생성, push, license, package publish와 release는 명시적 결정이 있을 때만 수행한다.
