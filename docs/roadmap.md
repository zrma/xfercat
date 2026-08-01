# Roadmap

## P0 Repository Foundation

- [x] initialize colocated Git-backed `jj`
- [x] define product and architecture boundaries
- [x] install canonical AI-first agent harness
- [x] add local verification and publication boundary gates
- [x] choose Rust/Ratatui for the PoC validation shell
- [x] define executable domain fixtures and vertical-slice acceptance

## P1 Local Transfer Plan

- [x] read-only connection profile picker with synthetic profiles
- [x] process-lifetime synthetic connection profile add/edit
- [x] profile delete with active-connection and staged-plan reference handling
- [x] synthetic local/remote dual-pane browser fixture
- [x] Waybill destination rename and stable-ID reorder
- [x] Waybill add, conflict-policy edit and remove
- [x] dry-run preview with source, destination, direction and conflict policy
- [x] item-level success, failure, skip and cancellation states

## P2 Remote Transport

- [x] OpenSSH-compatible connection profile import boundary
- [x] host verification and credential-reference contract
- [x] SFTP session and remote browser
- [x] upload and download execution with item-level results
- [x] stale destination and partial-failure handling
- [ ] progress and user cancellation

## P3 Reliability And Distribution

- [ ] crash-safe plan persistence decision
- [ ] resume and retry contract
- [ ] performance fixtures for large directories and batches
- [ ] accessibility and representative interaction smoke
- [ ] packaging, signing and release policy after explicit distribution decision

Roadmap checkboxes describe acceptance targets. Only `docs/status.md` and executable evidence describe
current implementation.
