//! Aster Desktop host. WebView input is intent; this host alone owns OS
//! credentials, fixed HTTPS egress, file grants and terminal request state.

mod credential;
mod policy;
mod provider;
mod workspace;

use credential::{CredentialError, CredentialStore};
use futures_util::FutureExt;
use policy::{FinishDisposition, Model, SessionState, TransportOutcome, DEFAULT_MODEL, POLICY_VERSION};
use serde::{Deserialize, Serialize};
use std::{panic::AssertUnwindSafe, sync::{atomic::{AtomicBool, Ordering}, Mutex}};
use tauri::{Emitter, Manager, State, WebviewWindow};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
use workspace::{RootSelection, Workspace};

const MAIN_WINDOW: &str = "main";
const STREAM_EVENT: &str = "aster://stream";

#[derive(Default)]
struct Host {
    credentials: CredentialStore,
    sessions: Mutex<SessionState>,
    workspace: Workspace,
    native_dialog_open: AtomicBool,
}

struct NativeDialogGuard<'a>(&'a AtomicBool);

impl Drop for NativeDialogGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

impl Host {
    fn enter_native_dialog(&self) -> Result<NativeDialogGuard<'_>, String> {
        self.native_dialog_open
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| NativeDialogGuard(&self.native_dialog_open))
            .map_err(|_| "native_dialog_busy".to_owned())
    }
}

fn trusted(window: &WebviewWindow) -> Result<(), String> {
    if window.label() != MAIN_WINDOW {
        return Err("caller_denied".to_owned());
    }
    // Capability ACL checks the window and bundled-local origin *before*
    // calling this handler. The label check is a second host-side boundary.
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelDisplay {
    id: &'static str,
    label: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Capabilities {
    project_read: bool,
    proposals: bool,
    apply_new_file: bool,
    apply_existing_file: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppStatus {
    policy_version: u32,
    configured: bool,
    credential_available: bool,
    default_model: &'static str,
    models: [ModelDisplay; 2],
    active_request: bool,
    project_selected: bool,
    project_label: Option<String>,
    capabilities: Capabilities,
}

#[derive(Serialize)]
struct CredentialStatus {
    configured: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Accepted {
    request_id: String,
    accepted: bool,
}

#[derive(Serialize)]
struct Cancelled {
    cancelled: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SendIntent {
    conversation_id: String,
    request_id: String,
    model_id: String,
    message: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct StreamEvent<'a> {
    request_id: &'a str,
    conversation_id: &'a str,
    #[serde(rename = "type")]
    kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_code: Option<&'static str>,
}

#[tauri::command]
fn get_app_status(window: WebviewWindow, host: State<'_, Host>) -> Result<AppStatus, String> {
    trusted(&window)?;
    let configured = match host.credentials.configured() {
        Ok(value) => value,
        Err(CredentialError::Unavailable) => false,
        Err(error) => return Err(error.code().to_owned()),
    };
    let active_request = host
        .sessions
        .lock()
        .map_err(|_| "host_unavailable")?
        .active_count()
        > 0;
    let project_label = host.workspace.selected_status();
    Ok(AppStatus {
        policy_version: POLICY_VERSION,
        configured,
        credential_available: cfg!(windows),
        default_model: DEFAULT_MODEL,
        models: [
            ModelDisplay { id: DEFAULT_MODEL, label: "Nemotron 3.5 Lightning 30B A3B" },
            ModelDisplay { id: "meta/muse-glimmer-30b", label: "Muse Glimmer 30B" },
        ],
        active_request,
        project_selected: project_label.is_some(),
        project_label,
        capabilities: Capabilities {
            project_read: false,
            proposals: false,
            apply_new_file: false,
            apply_existing_file: false,
        },
    })
}

#[tauri::command]
async fn configure_nvidia_credential(
    window: WebviewWindow,
    app: tauri::AppHandle,
) -> Result<CredentialStatus, String> {
    trusted(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let host = app.state::<Host>();
        let _guard = host.enter_native_dialog()?;
        match host.credentials.enter_with_native_dialog() {
            Ok(_) | Err(CredentialError::Cancelled) => Ok(CredentialStatus {
                configured: host.credentials.configured().map_err(|e| e.code())?,
            }),
            Err(error) => Err(error.code().to_owned()),
        }
    })
    .await
    .map_err(|_| "native_dialog_unavailable".to_owned())?
}

#[tauri::command]
async fn remove_nvidia_credential(
    window: WebviewWindow,
    app: tauri::AppHandle,
) -> Result<CredentialStatus, String> {
    trusted(&window)?;
    tauri::async_runtime::spawn_blocking(move || {
        let host = app.state::<Host>();
        let _guard = host.enter_native_dialog()?;
        // Renderer intent is not enough to delete the key; require native consent.
        let agreed = window
            .dialog()
            .message("¿Eliminar la clave de NVIDIA guardada en Windows Credential Manager?")
            .title("Aster Desktop: eliminar credencial")
            .buttons(MessageDialogButtons::OkCancelCustom("Eliminar".into(), "Cancelar".into()))
            .blocking_show();
        if agreed {
            host.credentials.delete().map_err(|e| e.code())?;
        }
        Ok(CredentialStatus {
            configured: host.credentials.configured().map_err(|e| e.code())?,
        })
    })
    .await
    .map_err(|_| "native_dialog_unavailable".to_owned())?
}

#[tauri::command]
async fn select_project_root(
    window: WebviewWindow,
    app: tauri::AppHandle,
) -> Result<RootSelection, String> {
    trusted(&window)?;
    #[cfg(windows)]
    {
        tauri::async_runtime::spawn_blocking(move || {
            let host = app.state::<Host>();
            let _guard = host.enter_native_dialog()?;
            let selected = window.dialog().file().blocking_pick_folder();
            match selected {
                Some(path) => host.workspace
                    .set_native_root(path.into_path().map_err(|_| "workspace_denied")?)
                    .map(|mut selection| {
                        // Internal Windows file-service proofs are still pending.
                        // Root selection supplies only a label, never read access.
                        selection.read_available = false;
                        selection.apply_available = false;
                        selection
                    })
                    .map_err(|e| e.code().to_owned()),
                None => Ok(RootSelection {
                    selected: false,
                    label: String::new(),
                    read_available: false,
                    apply_available: false,
                }),
            }
        })
        .await
        .map_err(|_| "native_dialog_unavailable".to_owned())?
    }
    #[cfg(not(windows))]
    {
        let _ = app;
        Err("WORKSPACE_UNAVAILABLE".to_owned())
    }
}

#[tauri::command]
fn cancel_message(
    window: WebviewWindow,
    host: State<'_, Host>,
    request_id: String,
) -> Result<Cancelled, String> {
    trusted(&window)?;
    Ok(Cancelled {
        cancelled: host.sessions.lock().map_err(|_| "host_unavailable")?.cancel(&request_id),
    })
}

#[tauri::command]
async fn send_message(
    window: WebviewWindow,
    app: tauri::AppHandle,
    host: State<'_, Host>,
    input: SendIntent,
) -> Result<Accepted, String> {
    trusted(&window)?;
    let SendIntent { conversation_id, request_id, model_id, message } = input;
    let model = Model::parse(&model_id).ok_or("unregistered_model")?;
    if !policy::valid_id(&conversation_id)
        || !policy::valid_id(&request_id)
        || !policy::valid_prompt(&message)
    {
        return Err("invalid_input".to_owned());
    }
    // Project excerpts are not a field of the strict SendIntent DTO. Unknown
    // keys, including projectDisclosure, are rejected by serde before this
    // function runs; no project bytes are available to provider requests.
    let prompt = message;
    // No secret is carried by ordinary IPC. Only the host receives its
    // Credential Manager value; before reserving a paid request check status.
    let key = host.credentials.read().map_err(|e| e.code().to_owned())?;
    let admission = host
        .sessions
        .lock()
        .map_err(|_| "host_unavailable")?
        .reserve(&conversation_id, &request_id, model, prompt)
        .map_err(|e| e.code().to_owned())?;

    let result_id = request_id.clone();
    let worker_id = request_id.clone();
    let worker_conversation = conversation_id.clone();
    let worker_app = app.clone();
    tauri::async_runtime::spawn(async move {
        let cancellation = admission.cancellation;
        let event_cancel = cancellation.clone();
        let response = AssertUnwindSafe(provider::stream_chat(model, &admission.messages, key, cancellation, |text| {
            if *event_cancel.borrow() {
                return Err(provider::ProviderFailure::Cancelled);
            }
            let session = worker_app.state::<Host>();
            let mut state = session.sessions.lock().map_err(|_| provider::ProviderFailure::InvalidContract)?;
            if !state.mark_streaming(&worker_conversation, &worker_id) {
                return Err(provider::ProviderFailure::Cancelled);
            }
            // Keep the same host lock while enqueueing the event. A later
            // cancellation acknowledgement cannot precede a new host emit.
            let result = worker_app
                .emit_to(MAIN_WINDOW, STREAM_EVENT, StreamEvent {
                    request_id: &worker_id,
                    conversation_id: &worker_conversation,
                    kind: "content",
                    text: Some(&text),
                    status: None,
                    error_code: None,
                })
                .map_err(|_| provider::ProviderFailure::InvalidContract);
            drop(state);
            result
        })).catch_unwind().await.unwrap_or(Err(provider::ProviderFailure::InvalidContract));
        let (transport, provider_error) = match response {
            Ok(answer) => (TransportOutcome::Completed(answer), None),
            Err(provider::ProviderFailure::Cancelled) => (TransportOutcome::Cancelled, None),
            Err(error) => (TransportOutcome::Failed, Some(error.code())),
        };
        // Cancellation and final disposition are now decided under one lock.
        // If cancellation acquired the lock first it must finish cancelled;
        // if finish acquired it first, cancellation returns false.
        let state = worker_app.state::<Host>();
        let disposition = state
            .sessions
            .lock()
            .map(|mut s| s.finish(&worker_conversation, &worker_id, transport))
            .unwrap_or(None);
        if let Some(disposition) = disposition {
            let (status, error_code) = match disposition {
                FinishDisposition::Completed => ("completed", None),
                FinishDisposition::Cancelled => ("cancelled", None),
                FinishDisposition::Failed => ("failed", provider_error),
            };
            let _ = worker_app.emit_to(MAIN_WINDOW, STREAM_EVENT, StreamEvent {
                request_id: &worker_id,
                conversation_id: &worker_conversation,
                kind: "terminal",
                text: None,
                status: Some(status),
                error_code,
            });
        }
    });
    Ok(Accepted { request_id: result_id, accepted: true })
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Host::default())
        .invoke_handler(tauri::generate_handler![
            get_app_status,
            configure_nvidia_credential,
            remove_nvidia_credential,
            select_project_root,
            send_message,
            cancel_message,
        ])
        .run(tauri::generate_context!())
        .expect("Aster Desktop host failed to start");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn send_intent_rejects_hidden_disclosure_and_url_overrides() {
        let base = serde_json::json!({
            "conversationId": "c1", "requestId": "r1",
            "modelId": DEFAULT_MODEL, "message": "Hola"
        });
        assert!(serde_json::from_value::<SendIntent>(base.clone()).is_ok());
        for extra in [
            serde_json::json!({"projectDisclosure":{"relativePath":"src/lib.rs","content":"secret"}}),
            serde_json::json!({"projectDisclosure":null}),
            serde_json::json!({"url":"https://example.com/v1/chat/completions"}),
            serde_json::json!({"approved":true}),
        ] {
            let mut candidate = base.clone();
            candidate.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
            assert!(serde_json::from_value::<SendIntent>(candidate).is_err());
        }
    }

    #[test]
    fn closed_command_manifest_is_explicit_and_not_wildcard() {
        let manifest = include_str!("../build.rs");
        let capability = include_str!("../capabilities/main.json");
        for command in ["get_app_status", "configure_nvidia_credential", "remove_nvidia_credential", "select_project_root", "send_message", "cancel_message"] {
            assert!(manifest.contains(&format!("\"{command}\"")));
            assert!(capability.contains(&format!("allow-{}", command.replace('_', "-"))));
        }
        assert!(capability.contains("\"windows\": [\"main\"]"));
        assert!(!capability.contains("\"windows\": [\"*\"]"));
        assert!(!capability.contains("fs:default"));
        assert!(!capability.contains("allow-read-project-file"));
        assert!(!manifest.contains("\"read_project_file\""));
        assert!(!capability.contains("allow-review-proposal"));
        assert!(!capability.contains("shell:"));
        assert!(!capability.contains("core:event:allow-emit"));
    }
}
