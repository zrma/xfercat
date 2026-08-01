# Completed Milestones

## 2026-07-30 — Repository Foundation

- initialized a colocated Git-backed `jj` repository
- defined product, architecture, publication and active-work contracts
- installed the canonical AI-first harness and repository-only checks
- left runtime, remote, license and publication as explicit future decisions

Evidence: `scripts/check.sh`.

## 2026-07-30 — Product Foundation PoC

- selected Rust 2024 with a Ratatui/Crossterm TUI as a PoC validation shell
- implemented synthetic Connections, local/remote Browser, Waybill and dry-run Review
- preserved exact transfer endpoints independently from browser focus and selection
- added deterministic full and compact terminal snapshots plus an actual PTY interaction smoke
- kept real filesystem mutation, network transport and final GUI/TUI choice out of scope

Evidence: `cargo test`, the three `--snapshot` states, PTY interaction smoke and
`scripts/check.sh`.

## 2026-07-30 — P1 Waybill Editing

- added an isolated destination-filename rename mode
- rejected empty, dot, parent, separator and control-character names without mutating the plan
- added stable-ID move up/down while preserving item payload
- rendered renamed and reordered items identically in Waybill and dry-run Review
- added rename, reorder, compact shortcut and actual PTY interaction evidence

Evidence: focused app tests, the `rename` and `review` snapshots, PTY interaction smoke and
`scripts/check.sh`.

## 2026-08-01 — P1 Profile Catalog Editing

- added create and edit forms that remain isolated from Connect
- preserved stable profile identity and previously staged endpoints during edit
- rejected empty, duplicate and invalid fields without changing the catalog
- supported SSH Agent and non-secret key references without storing credential material
- kept profile changes process-local and left delete, persistence and OpenSSH import deferred

Evidence: six profile app tests, `profile-add` and `profile-edit` snapshots, 80×24 regression,
PTY add/save/connect/edit/cancel/quit smoke with terminal restoration and `scripts/check.sh`.

## 2026-08-01 — P2 OpenSSH Profile Import

- replaced synthetic runtime startup profiles with concrete aliases discovered from user OpenSSH config
- followed global includes with quoted path, home/environment expansion, globbing and cycle protection
- skipped wildcard, negated and conditional entries without evaluating `Match` or running subprocesses
- added startup import, `I` refresh, read-only provenance, empty state and manual fallback UX
- preserved process-local manual profiles and staged plan items across refresh

Evidence: five discovery tests, three import-specific app tests, OpenSSH/empty snapshots, 80×24
regression, redacted local-config PTY smoke with terminal restoration and `scripts/check.sh`.

## 2026-08-01 — P1 Profile Delete

- added Connections delete action for process-lifetime manual and synthetic profiles
- kept imported OpenSSH aliases source-owned with edit-config-and-refresh guidance
- blocked non-cascading deletion while any staged Waybill item references the profile identity
- cleared an active synthetic connection when its unreferenced profile is deleted
- preserved deterministic selection for middle, last and empty catalog states

Evidence: four focused delete tests, 110×32 and 80×24 shortcut snapshots, delete/re-add/connect
PTY smoke and `scripts/check.sh`.

## 2026-08-01 — Typed Transport Boundary

- preserved staged entry kind and expected-size metadata independently from browser state
- froze exact item identity, endpoints, direction and conflict policy into `TransportRequest`
- rejected relative, traversal, control-character and direction/endpoint-role mismatches
- defined succeeded, skipped, failed and cancelled results without raw diagnostic payloads
- kept filesystem, subprocess, network, credential and transport-library choices out of scope

Evidence: four focused transport tests, staged metadata app regression and `scripts/check.sh`.

## 2026-08-01 — P1 Synthetic Executor

- guarded `TransferState` transitions from staged through running to one terminal outcome
- added a deterministic representative executor over the typed transport boundary
- preserved succeeded, failed, skipped and cancelled results item by item after partial failure
- prevented terminal items from being implicitly rerun
- exposed an explicit Review action and result rendering while keeping filesystem/network untouched

Evidence: one state-machine test, three executor tests, Review app regression, staged/results
snapshots at 110×32 and 80×24, actual PTY smoke and `scripts/check.sh`.

## 2026-08-01 — OpenSSH SFTP Transport Decision

- selected the system OpenSSH client to preserve effective user config and agent behavior
- selected `openssh-sftp-client` for typed SFTP v3 operations on Unix
- required strict known-host verification and non-interactive batch authentication
- required temporary sibling writes, verification and rename before final destination exposure
- recorded that cancelling an SFTP future does not guarantee cancellation of an already-sent mutation

Evidence: current crates.io metadata, official rustdoc, local OpenSSH/sshd/SFTP capability check,
decision 0003 and `scripts/check.sh`.

## 2026-08-01 — P2 Local Filesystem Browser

- replaced runtime synthetic local entries with canonical current-directory filesystem entries
- added Enter child-directory and Backspace parent navigation
- preserved regular-file kind, size and exact path in staged transfer items
- excluded symlink, non-Unicode, control-character, special and unreadable entries without guessing
- staged upload/download destinations from the current remote/local pane directories

Evidence: three local filesystem tests, two application navigation/staging regressions, full and
compact snapshots, actual read-only PTY navigation smoke and `scripts/check.sh`.

## 2026-08-01 — P2 SFTP Remote Browser

- connected imported aliases and manual agent/key-reference profiles through system OpenSSH
- enforced strict known-host verification, batch authentication and a bounded connection timeout
- replaced runtime synthetic remote entries with canonical SFTP directory entries
- excluded symlink, non-Unicode, control-character, special and unreadable remote entries
- added child/parent remote navigation, graceful close and sanitized typed failure status
- added an optional explicit config path shared by discovery and system OpenSSH connection

Evidence: five adapter unit tests, two application regressions, an ephemeral localhost sshd/SFTP
fixture, actual PTY connect/navigation/close smoke and `scripts/check.sh`.
