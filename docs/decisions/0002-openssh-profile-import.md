# Decision 0002: Side-Effect-Free OpenSSH Profile Discovery

Status: accepted for the profile-import boundary

## Context

Manual entry makes the connection picker repeat configuration already owned by the OpenSSH client.
The picker needs useful aliases at startup without connecting, copying credential material or
silently executing commands from user configuration.

OpenSSH configuration may contain wildcard `Host` patterns, recursive `Include` directives and
conditional `Match` blocks. `ssh -G` prints effective configuration after evaluating `Host` and
`Match`; a `Match exec` criterion can execute a command through the user shell. Running `ssh -G`
for every discovered alias is therefore not an acceptable automatic startup operation.

## Decision

- Discover concrete, non-negated `Host` aliases from the user config and global `Include` files.
- Support quoted include paths, `~`, `%d`, environment variables and glob expansion.
- Skip wildcard aliases and conditional includes; report only aggregate skipped-item counts.
- Store an imported profile as a read-only OpenSSH alias reference with stable provenance.
- Do not run `ssh`, evaluate `Match`, resolve effective endpoint/authentication fields or read key
  contents during discovery.
- Keep process-local manual profiles as a fallback for destinations absent from the config.

## Consequences

- Startup and `I` refresh are filesystem reads only and never initiate network traffic.
- The picker labels imported authentication as `OpenSSH policy`; it does not claim Agent or a
  specific identity until an explicit connection boundary can prove that effective configuration.
- An imported alias remains selectable for the synthetic workspace, while `E` directs the user to
  edit the source config and refresh.
- Conditional configuration may affect a later connection even though it does not create picker
  entries during discovery.

## Deferred

- effective config evaluation immediately before an explicit connection
- `Match`, host verification, agent/key execution and transport-library interoperability
- system-wide config and known-hosts discovery

## References

- [OpenSSH client configuration](https://man.openbsd.org/ssh_config)
- [OpenSSH `ssh -G`](https://man.openbsd.org/ssh)
