# xfercat

Plan first. Transfer once.

`xfercat`은 저장된 연결을 실수 없이 선택하고, 로컬과 원격 사이의 파일 전송을
실행 전에 하나의 명시적인 계획으로 검토하는 파일 전송 도구다.

현재 상태는 AI-first repository foundation이다. 실행 가능한 파일 전송 클라이언트는
아직 구현되지 않았다.

## Product Direction

- 연결은 완성된 `ConnectionProfile`을 고르는 행위다.
- 파일 선택은 즉시 전송이 아니라 독립된 `TransferPlan`에 작업을 추가한다.
- 각 작업은 source, destination, 방향, 충돌 정책과 실행 상태를 명시한다.
- 실제 전송은 전체 계획을 검토한 뒤 명시적으로 실행한다.

제품 경계는 [Product](docs/PRODUCT.md), 현재 상태와 다음 작업은
[Handoff](docs/HANDOFF.md)에서 확인한다.

## Repository Workflow

```sh
scripts/check.sh
scripts/start-work.sh --work-id <work-id>
scripts/finalize-change.sh --verify-only
```

로컬 변경은 `jj`를 사용한다. remote 생성, push, license 선택과 release는 별도 결정이다.
