# Status

Updated: 2026-08-01

## Verdict

The PoC executes actual preview-first regular-file upload and download over SFTP.

## Implemented

- colocated Git-backed `jj` repository
- compact `AGENTS.md` bootstrap map and canonical agent harness
- product, architecture, handoff, roadmap and publication contracts
- repository contract and tracked-artifact privacy checks
- local work start and finalize scripts
- Rust 2024 domain and application state
- Ratatui/Crossterm Connections, Browser, Waybill and Review shell
- process-lifetime manual connection profile add/edit/delete actions
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
- accepted system OpenSSH/SFTP transport, strict host verification and temporary-write contract
- actual canonical local filesystem listing, safe entry filtering and directory navigation
- strict system OpenSSH session with batch authentication and sanitized typed failures
- actual canonical SFTP remote listing, safe entry filtering and child/parent navigation
- optional explicit OpenSSH config shared by discovery and connection
- current-pane exact local/remote path staging
- destination missing/kind/size expectation frozen into each reviewed request
- actual regular-file upload and download with source/destination revalidation
- sibling temporary writes, close/fsync/size verification and atomic finalization
- remote atomic-finalization extension preflight before upload mutation
- safe `ASK`, `OVERWRITE`, `SKIP` and explicit renamed-destination behavior
- live execution with item-level partial success/failure preservation and browser refresh
- profile-selection, exact-endpoint and Waybill interaction tests
- deterministic 110×32 and compact 80×24 terminal snapshots

## Not Implemented

- persistent connection catalog
- effective OpenSSH config, conditional `Match` and authentication resolution
- directory and symlink transfer
- progress UI, user cancellation, resume and retry
- persistent transfer plan
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
- cancelling an already-sent SFTP mutation cannot guarantee that the remote operation stops.
- destination stale checks compare kind and size; SFTP v3 cannot provide an atomic content
  compare-and-swap for explicit overwrite.
- remote upload requires `hardlink` for no-replace publish and `posix-rename` for explicit
  overwrite; servers without the required extension fail that item before writing.
- plan persistence needs a privacy and crash-recovery contract before durable storage is added.
