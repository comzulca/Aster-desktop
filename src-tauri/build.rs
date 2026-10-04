// All application commands must be listed in this manifest and explicitly
// granted to the trusted `main` webview in capabilities/main.json.
fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "get_app_status",
            "configure_meta_credential",
            "remove_meta_credential",
            "accept_contributor_terms",
            "configure_budget",
            "reconcile_budget",
            "list_conversations",
            "create_conversation",
            "load_conversation",
            "delete_conversation",
            "send_message",
            "cancel_message",
            "window_action",
            "startup_shell_ready",
            "select_attachments",
            "remove_attachment",
            "review_processed_attachment",
        ]),
    ))
    .expect("Tauri capability generation must succeed");
}
