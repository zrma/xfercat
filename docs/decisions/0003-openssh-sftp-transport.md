# Decision 0003: OpenSSH SFTP Transport

Status: accepted

## Context

`xfercat` profile import는 user OpenSSH config의 concrete alias를 identity로 사용한다. 실제
connection은 `HostName`, `User`, `Port`, `IdentityFile`, agent, `ProxyJump`, `Match`와 include
semantics를 다시 구현하지 않으면서 unknown host key를 자동 수락하지 않아야 한다.

## Decision

- Unix target에서 system OpenSSH client를 감싸는 `openssh` 0.11 line을 사용한다.
- SFTP v3 operation은 `openssh-sftp-client` 0.15 line을 사용한다.
- imported profile은 alias를 destination으로 전달해 user `ssh_config`의 effective policy를
  OpenSSH가 평가하게 한다.
- manual profile은 explicit host/user와 optional key-reference path를 builder option으로 전달한다.
- 모든 connection은 `KnownHosts::Strict`와 crate가 제공하는 `BatchMode=yes`를 사용한다.
  unknown 또는 changed host key와 interactive password authentication은 connection 전에 실패한다.
- local/remote write는 final destination을 바로 truncate하지 않는다. sibling temporary file에
  쓴 뒤 close와 size verification을 통과하고 conflict policy가 허용할 때 rename한다.
- request cancellation만으로 remote mutation 중단을 보장하지 않는다. remote client 문서가
  이미 전송된 mutation request는 future cancellation 뒤에도 수행될 수 있다고 명시하기 때문이다.

## Rationale

`openssh`는 system `ssh`를 사용하므로 기존 OpenSSH config와 agent behavior를 보존한다.
`openssh-sftp-client`는 이 session에서 SFTP subsystem을 열고 typed filesystem API를 제공한다.
두 crate의 MSRV는 repository Rust 1.97보다 낮고 license는 각각 MIT/Apache-2.0 및 MIT다.

## Rejected Alternatives

- effective config를 앱이 직접 해석: `Match exec`, token expansion, jump host와 vendor behavior를
  안전하게 재현하는 별도 SSH policy engine이 필요하다.
- unknown host key 자동 등록: 첫 connection의 identity를 사용자가 검증하지 않은 채 durable
  trust state를 변경한다.
- final destination 직접 create/truncate: partial failure와 cancellation이 기존 파일이나
  final-name partial file을 남길 수 있다.
- initial pure-Rust SSH stack: 현재 OpenSSH alias compatibility contract보다 protocol/library
  portability를 우선하게 되므로 후속 cross-platform decision으로 남긴다.

## Consequences

- current live transport target은 system OpenSSH가 있는 Unix다.
- password-only host는 지원하지 않으며 agent 또는 non-interactive key가 필요하다.
- unknown host는 사용자가 별도 trusted OpenSSH flow에서 fingerprint를 확인해야 한다.
- app은 raw OpenSSH/SFTP diagnostics를 UI status나 tracked evidence에 복제하지 않고 stable
  failure category로 변환한다.
- cancellation, cleanup과 rename support는 격리된 SFTP fixture에서 별도 검증한다.

## Sources

- <https://docs.rs/openssh/0.11.6/openssh/struct.SessionBuilder.html>
- <https://docs.rs/openssh/0.11.6/openssh/enum.KnownHosts.html>
- <https://docs.rs/openssh-sftp-client/0.15.7/openssh_sftp_client/struct.Sftp.html>
- <https://docs.rs/openssh-sftp-client/0.15.7/openssh_sftp_client/>
- <https://man.openbsd.org/ssh_config>
