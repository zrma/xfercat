# Status

Updated: 2026-08-01

## Verdict

The synthetic product-foundation PoC is executable; real file transfer is not implemented.

## Implemented

- colocated Git-backed `jj` repository
- compact `AGENTS.md` bootstrap map and canonical agent harness
- product, architecture, handoff, roadmap and publication contracts
- repository contract and tracked-artifact privacy checks
- local work start and finalize scripts
- Rust 2024 domain and application state
- Ratatui/Crossterm Connections, Browser, Waybill and Review shell
- process-lifetime synthetic connection profile add/edit/delete actions
- stable profile identity, duplicate-label rejection and SSH Agent/key-reference selection
- non-cascading staged-reference delete guard and active synthetic connection cleanup
- side-effect-free OpenSSH concrete-alias discovery from user config and global includes
- startup import, `I` refresh, read-only provenance and empty/manual fallback states
- destination-leaf rename with invalid-name rejection
- stable-ID Waybill reorder reflected in Review
- immutable typed transport request conversion with endpoint/path validation
- typed transport success, skip, failure and cancellation result contract without raw diagnostics
- guarded staged-running-terminal item transitions and representative synthetic executor
- explicit Review execution action with partial result preservation and terminal result rendering
- profile-selection, exact-endpoint and Waybill interaction tests
- deterministic 110×32 and compact 80×24 terminal snapshots

## Not Implemented

- persistent connection catalog
- effective OpenSSH config, conditional `Match` and authentication resolution
- real connection establishment
- filesystem-backed local or remote browser
- transfer execution
- network transport
- packaging, release or updater

## Active Work

- none

## External State

- remote: configured
- remote visibility: public
- license: undecided
- package or release publication: not performed

## Risks

- the TUI is a PoC adapter, not a final GUI/TUI product commitment.
- manual catalog changes remain process-local and reset when the process exits.
- wildcard/conditional OpenSSH rules affect future connection semantics but do not become picker rows.
- transport library choice can affect OpenSSH compatibility, host verification and credential handling.
- plan persistence needs a privacy and crash-recovery contract before durable storage is added.
