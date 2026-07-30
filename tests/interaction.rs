use xfercat::ui;

#[test]
fn connections_snapshot_keeps_profile_attributes_visible() {
    let snapshot = ui::snapshot("connections").expect("connections snapshot");

    assert!(snapshot.contains("xfercat · Connections"));
    assert!(snapshot.contains("dev-box"));
    assert!(snapshot.contains("SFTP"));
    assert!(snapshot.contains("SSH Agent"));
    assert!(snapshot.contains("Enter Connect"));
    assert!(snapshot.contains("E Details"));
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
}

#[test]
fn compact_terminal_keeps_connection_auth_and_all_workspace_keys_visible() {
    let connections = ui::snapshot_at("connections", 80, 24).expect("compact connections");
    let workspace = ui::snapshot_at("workspace", 80, 24).expect("compact workspace");

    assert!(connections.contains("Key:archive-key"));
    assert!(connections.contains("operator@archive.example"));
    assert!(workspace.contains("Space Add   D Remove"));
    assert!(workspace.contains("Esc Connections   Q Quit"));
}
