# Publication Policy

## Classification

`xfercat`의 tracked surface는 `public-ready` content 기준을 사용한다. 이것은 remote의 live
visibility 선언이 아니다. 현재 remote는 구성되지 않았고 license도 선택되지 않았다.

## Publishable Content

- product, architecture와 protocol decision
- synthetic endpoint, user, path와 transfer identifier를 사용한 fixture
- 재현 가능한 test command와 redacted pass/fail 판정
- source code, public dependency metadata와 user documentation
- 구현 상태와 repository-owned milestone evidence

## Local-only Content

- credential, token, private key, SSH agent나 socket 정보
- 실제 host alias, username, checkout path, network address와 private repository inventory
- prompt, conversation transcript, memory와 raw tool output
- machine-specific diagnostic bundle와 승인되지 않은 vulnerability detail

필요한 예시는 `<repo-root>`, `<home>`, `<private-host>`, `<internal-ip>` 같은 placeholder 또는
명시적인 synthetic value를 사용한다. private inventory denylist를 저장소나 CI에 넣지 않는다.

## Gate

local change와 remote publication 전에 `scripts/check.sh` 및
`scripts/check-publication-boundary.py`를 실행한다.

remote 생성, push, visibility 변경, tag나 release 전에는 추가로 다음이 필요하다.

1. exact destination, owner와 live visibility 확인.
2. 권한 있는 machine-local private-inventory gate 실행.
3. 공개될 tree, reachable history와 change description 검사.
4. license, package metadata와 README 선언의 정합성 확인.

remote 생성, push, license 선택과 release는 각각 별도 external decision 또는 write다.
