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
