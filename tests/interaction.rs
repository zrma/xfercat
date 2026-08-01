use xfercat::ui;

#[test]
fn connections_snapshot_keeps_profile_attributes_visible() {
    let snapshot = ui::snapshot("connections").expect("connections snapshot");

    assert!(snapshot.contains("xfercat · Connections"));
    assert!(snapshot.contains("dev-box"));
    assert!(snapshot.contains("SFTP"));
    assert!(snapshot.contains("SSH Agent"));
    assert!(snapshot.contains("Enter Connect"));
    assert!(snapshot.contains("A Add"));
    assert!(snapshot.contains("E Edit"));
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
    assert!(snapshot.contains("dev-box:/srv/xfercat/releases/app.tar.gz"));
    assert!(snapshot.contains("[ASK] [STAGED]"));
}

#[test]
fn review_snapshot_is_explicitly_non_executing() {
    let snapshot = ui::snapshot("review").expect("review snapshot");

    assert!(snapshot.contains("DRY-RUN TRANSFER REVIEW"));
    assert!(snapshot.contains("no transport adapter is connected"));
    assert!(snapshot.contains("Enter Accept review"));
    assert!(snapshot.contains("local:/workspace/incoming/service-copy.log"));
    let download = snapshot.find("#2 ↓").expect("download item");
    let upload = snapshot.find("#1 ↑").expect("upload item");
    assert!(download < upload, "reordered item must render first");
}

#[test]
fn rename_snapshot_shows_current_destination_and_edit_buffer() {
    let snapshot = ui::snapshot("rename").expect("rename snapshot");

    assert!(snapshot.contains("DESTINATION FILENAME"));
    assert!(snapshot.contains("local:/workspace/incoming/service.log"));
    assert!(snapshot.contains("service-copy.log"));
    assert!(snapshot.contains("Enter Apply   Esc Cancel"));
}

#[test]
fn compact_terminal_keeps_connection_auth_and_all_workspace_keys_visible() {
    let connections = ui::snapshot_at("connections", 80, 24).expect("compact connections");
    let profile = ui::snapshot_at("profile-edit", 80, 24).expect("compact profile editor");
    let workspace = ui::snapshot_at("workspace", 80, 24).expect("compact workspace");

    assert!(connections.contains("Key:archive-key"));
    assert!(connections.contains("operator@archive.example"));
    assert!(connections.contains("A Add   E Edit"));
    assert!(profile.contains("Tab/Shift+Tab Field"));
    assert!(profile.contains("Enter Save   Esc Cancel"));
    assert!(workspace.contains("Space Add   D Remove"));
    assert!(workspace.contains("N Rename   Shift+K/J Reorder"));
    assert!(workspace.contains("Esc Connections   Q Quit"));
}
