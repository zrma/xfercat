use xfercat::ui;

#[test]
fn connections_snapshot_keeps_profile_attributes_visible() {
    let snapshot = ui::snapshot("connections").expect("connections snapshot");

    assert!(snapshot.contains("xfercat · Connections"));
    assert!(snapshot.contains("dev-box"));
    assert!(snapshot.contains("SFTP"));
    assert!(snapshot.contains("SSH Agent"));
    assert!(snapshot.contains("Enter Select"));
    assert!(snapshot.contains("I Refresh"));
    assert!(snapshot.contains("A Manual"));
    assert!(snapshot.contains("E Edit"));
    assert!(snapshot.contains("D Delete"));
}

#[test]
fn openssh_catalog_shows_provenance_read_only_policy_and_empty_fallback() {
    let imported = ui::snapshot("openssh").expect("OpenSSH catalog snapshot");
    let empty = ui::snapshot("openssh-empty").expect("empty OpenSSH catalog snapshot");

    assert!(imported.contains("build-box"));
    assert!(imported.contains("release-box"));
    assert!(imported.contains("Host build-box"));
    assert!(imported.contains("OpenSSH config"));
    assert!(imported.contains("OpenSSH policy"));
    assert!(imported.contains("read-only"));
    assert!(empty.contains("No connection profiles"));
    assert!(empty.contains("Press I to refresh or A to add manually"));
    assert!(empty.contains("No OpenSSH user config found"));
}

#[test]
fn profile_forms_show_editable_fields_and_process_lifetime_boundary() {
    let create = ui::snapshot("profile-add").expect("profile add snapshot");
    let edit = ui::snapshot("profile-edit").expect("profile edit snapshot");

    assert!(create.contains("xfercat · Add profile"));
    assert!(create.contains("[PROFILE FORM]"));
    assert!(create.contains("SSH Agent"));
    assert!(create.contains("Enter Save   Esc Cancel"));
    assert!(create.contains("saved only for this process"));
    assert!(edit.contains("xfercat · Edit profile"));
    assert!(edit.contains("dev-box"));
    assert!(edit.contains("deploy"));
    assert!(edit.contains("dev.example"));
}

#[test]
fn workspace_snapshot_shows_focusable_browsers_and_exact_waybill_endpoints() {
    let snapshot = ui::snapshot("workspace").expect("workspace snapshot");

    assert!(snapshot.contains("[LOCAL] /workspace/outgoing"));
    assert!(snapshot.contains("[REMOTE] dev-box:/srv/xfercat"));
    assert!(snapshot.contains("[WAYBILL] 2 item(s)"));
    assert!(snapshot.contains("local:/workspace/outgoing/app.tar.gz"));
    assert!(snapshot.contains("dev-box:/srv/xfercat/app.tar.gz"));
    assert!(snapshot.contains("[ASK] [DEST:MISSING] [STAGED]"));
}

#[test]
fn review_snapshot_is_explicitly_non_executing() {
    let snapshot = ui::snapshot("review").expect("review snapshot");

    assert!(snapshot.contains("DRY-RUN TRANSFER REVIEW"));
    assert!(snapshot.contains("no transport adapter or filesystem mutation"));
    assert!(snapshot.contains("Enter Run synthetic execution"));
    assert!(snapshot.contains("local:/workspace/outgoing/service-copy.log"));
    let download = snapshot.find("#2 ↓").expect("download item");
    let upload = snapshot.find("#1 ↑").expect("upload item");
    assert!(download < upload, "reordered item must render first");
}

#[test]
fn live_review_snapshot_discloses_actual_write_boundary_and_destination_state() {
    let snapshot = ui::snapshot("live-review").expect("live review snapshot");

    assert!(snapshot.contains("TRANSFER EXECUTION REVIEW"));
    assert!(snapshot.contains("actual local/SFTP file writes"));
    assert!(snapshot.contains("Enter Execute staged files"));
    assert!(snapshot.contains("DEST:MISSING"));
}

#[test]
fn synthetic_results_snapshot_preserves_all_terminal_item_states() {
    let snapshot = ui::snapshot("results").expect("synthetic results snapshot");

    assert!(snapshot.contains("Synthetic only: no transport adapter or filesystem mutation"));
    assert!(snapshot.contains("[SUCCEEDED]"));
    assert!(snapshot.contains("[FAILED]"));
    assert!(snapshot.contains("[SKIPPED]"));
    assert!(snapshot.contains("[CANCELLED]"));
    assert!(snapshot.contains("Synthetic results preserved   Esc Back"));
}

#[test]
fn rename_snapshot_shows_current_destination_and_edit_buffer() {
    let snapshot = ui::snapshot("rename").expect("rename snapshot");

    assert!(snapshot.contains("DESTINATION FILENAME"));
    assert!(snapshot.contains("local:/workspace/outgoing/service.log"));
    assert!(snapshot.contains("service-copy.log"));
    assert!(snapshot.contains("Enter Apply   Esc Cancel"));
}

#[test]
fn compact_terminal_keeps_connection_auth_and_all_workspace_keys_visible() {
    let connections = ui::snapshot_at("connections", 80, 24).expect("compact connections");
    let openssh = ui::snapshot_at("openssh", 80, 24).expect("compact OpenSSH catalog");
    let profile = ui::snapshot_at("profile-edit", 80, 24).expect("compact profile editor");
    let workspace = ui::snapshot_at("workspace", 80, 24).expect("compact workspace");
    let results = ui::snapshot_at("results", 80, 24).expect("compact synthetic results");

    assert!(connections.contains("Key:archive-key"));
    assert!(connections.contains("operator@archive.example"));
    assert!(connections.contains("I Refresh   A Manual   E Edit   D Delete"));
    assert!(openssh.contains("OpenSSH config"));
    assert!(openssh.contains("OpenSSH policy"));
    assert!(profile.contains("Tab/Shift+Tab Field"));
    assert!(profile.contains("Enter Save   Esc Cancel"));
    assert!(workspace.contains("Enter Open   Backspace Parent"));
    assert!(workspace.contains("Space/S Add   D Remove"));
    assert!(workspace.contains("N Rename   Shift+K/J Reorder"));
    assert!(workspace.contains("Esc Connections   Q Quit"));
    assert!(results.contains("[SUCCEEDED]"));
    assert!(results.contains("[CANCELLED]"));
    assert!(results.contains("Synthetic results preserved"));
}
