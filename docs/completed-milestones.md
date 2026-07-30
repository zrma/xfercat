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
