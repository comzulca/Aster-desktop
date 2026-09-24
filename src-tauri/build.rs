// All application commands must be listed in this manifest and explicitly
// granted to the trusted `main` webview in capabilities/main.json.
fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new().app_manifest(tauri_build::AppManifest::new().commands(&[
            "get_app_status",
            "configure_nvidia_credential",
            "remove_nvidia_credential",
            "select_project_root",
            "send_message",
            "cancel_message",
        ])),
    )
    .expect("Tauri capability generation must succeed");
}
