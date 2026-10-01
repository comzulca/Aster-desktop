//! Aster Desktop host. WebView input is intent; this host alone owns OS
//! credentials, fixed HTTPS egress, file grants and terminal request state.

#[cfg(all(test, windows))]
mod creation_process_fixture;
mod credential;
mod credential_fixture;
#[cfg(windows)]
mod journal_privacy;
mod native_review;
#[cfg(windows)]
mod new_file;
mod policy;
mod provider;
mod workspace;

use credential::{CredentialError, CredentialStore};
use futures_util::FutureExt;
use policy::{
    FinishDisposition, Model, SessionState, TransportOutcome, DEFAULT_MODEL, POLICY_VERSION,
};
use serde::{Deserialize, Serialize};
use std::{
    panic::AssertUnwindSafe,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
};
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SelectFileIntent {
    conversation_id: String,
    request_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FileReceipt {
    relative_path: String,
    request_id: String,
    sha256: String,
    byte_count: usize,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProjectSendIntent {
    conversation_id: String,
    request_id: String,
    model_id: String,
    message: String,
    relative_path: String,
    expected_sha256: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProposalIntent {
    conversation_id: String,
    request_id: String,
    proposal_json: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProposalReceipt {
    status: &'static str,
    operation: String,
    relative_path: String,
    new_sha256: String,
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
        #[cfg(not(windows))]
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
            ModelDisplay {
                id: DEFAULT_MODEL,
                label: "Nemotron 3.5 Lightning 30B A3B",
            },
            ModelDisplay {
                id: "meta/muse-glimmer-30b",
                label: "Muse Glimmer 30B",
            },
        ],
        active_request,
        project_selected: project_label.is_some(),
        project_label,
        capabilities: Capabilities {
            project_read: cfg!(windows),
            proposals: cfg!(windows),
            apply_new_file: cfg!(windows),
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
            .buttons(MessageDialogButtons::OkCancelCustom(
                "Eliminar".into(),
                "Cancelar".into(),
            ))
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
                Some(path) => host
                    .workspace
                    .set_native_root(path.into_path().map_err(|_| "workspace_denied")?)
                    .map(|mut selection| {
                        selection.read_available = true;
                        selection.apply_available = true;
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
async fn select_project_file(
    window: WebviewWindow,
    app: tauri::AppHandle,
    input: SelectFileIntent,
) -> Result<Option<FileReceipt>, String> {
    trusted(&window)?;
    if !policy::valid_id(&input.conversation_id) || !policy::valid_id(&input.request_id) {
        return Err("invalid_input".into());
    }
    #[cfg(windows)]
    {
        tauri::async_runtime::spawn_blocking(move || {
            let host = app.state::<Host>();
            let _guard = host.enter_native_dialog()?;
            let Some(selected) = window.dialog().file().blocking_pick_file() else {
                return Ok(None);
            };
            let grant = host
                .workspace
                .grant_native_file_read(
                    &input.conversation_id,
                    &input.request_id,
                    selected.into_path().map_err(|_| "WORKSPACE_DENIED")?,
                )
                .map_err(|e| e.code())?;
            let read = host
                .workspace
                .read_project_file(
                    &input.conversation_id,
                    &input.request_id,
                    &grant.relative_path,
                )
                .map_err(|e| e.code())?;
            // Project plaintext remains in the host snapshot and native review.
            Ok(Some(FileReceipt {
                relative_path: read.relative_path,
                request_id: input.request_id,
                sha256: read.sha256,
                byte_count: read.content.len(),
            }))
        })
        .await
        .map_err(|_| "native_dialog_unavailable".to_owned())?
    }
    #[cfg(not(windows))]
    {
        let _ = app;
        Err("WORKSPACE_UNAVAILABLE".into())
    }
}

#[tauri::command]
async fn review_project_proposal(
    window: WebviewWindow,
    app: tauri::AppHandle,
    input: ProposalIntent,
) -> Result<ProposalReceipt, String> {
    trusted(&window)?;
    if !policy::valid_id(&input.conversation_id) || !policy::valid_id(&input.request_id) {
        return Err("invalid_input".into());
    }
    #[cfg(windows)]
    {
        tauri::async_runtime::spawn_blocking(move || {
            let host = app.state::<Host>();
            let _guard = host.enter_native_dialog()?;
            let result = (|| -> Result<ProposalReceipt, String> {
                let edit =
                    workspace::parse_proposed_edit(&input.proposal_json).map_err(|e| e.code())?;
                let review = match edit.operation {
                    workspace::ProposalOperation::CreateNew => {
                        let journal = host
                            .workspace
                            .journal_directory(
                                app.path()
                                    .local_data_dir()
                                    .map_err(|_| "WORKSPACE_UNAVAILABLE")?,
                            )
                            .map_err(|e| e.code())?;
                        host.workspace.prepare_new_file_proposal(
                            &input.conversation_id,
                            &input.request_id,
                            &input.proposal_json,
                            journal,
                        )
                    }
                    workspace::ProposalOperation::ReplaceExisting => {
                        let selected = window
                            .dialog()
                            .file()
                            .blocking_pick_file()
                            .ok_or("proposal_denied")?;
                        let grant = host
                            .workspace
                            .grant_native_file_read(
                                &input.conversation_id,
                                &input.request_id,
                                selected.into_path().map_err(|_| "WORKSPACE_DENIED")?,
                            )
                            .map_err(|e| e.code())?;
                        host.workspace
                            .read_project_file(
                                &input.conversation_id,
                                &input.request_id,
                                &grant.relative_path,
                            )
                            .map_err(|e| e.code())?;
                        host.workspace.review_model_replacement(
                            &input.conversation_id,
                            &input.request_id,
                            &input.proposal_json,
                        )
                    }
                }
                .map_err(|e| e.code())?;
                let document = native_review::ReviewDocument::proposal(&review.diff)
                    .map_err(|_| "native_review_unavailable")?;
                let owner = window.hwnd().map_err(|_| "native_review_unavailable")?.0 as usize;
                let approval = native_review::confirm_owned(&document, owner)
                    .map_err(|_| "native_review_unavailable")?;
                let status = match approval {
                    None => {
                        let _ = host
                            .workspace
                            .revoke_pending(&input.conversation_id, &input.request_id);
                        "denied"
                    }
                    Some(approval) if edit.operation == workspace::ProposalOperation::CreateNew => {
                        match host
                            .workspace
                            .apply_new_file_candidate(&review.proposal_id, approval)
                            .map_err(|e| e.code())?
                        {
                            workspace::CreationOutcome::Verified => "verified",
                            workspace::CreationOutcome::AttentionRequired => "attention_required",
                        }
                    }
                    Some(approval) => {
                        approval
                            .consume(document.binding_sha256())
                            .map_err(|_| "native_review_unavailable")?;
                        let _ = host
                            .workspace
                            .revoke_pending(&input.conversation_id, &input.request_id);
                        "proposal_only"
                    }
                };
                Ok(ProposalReceipt {
                    status,
                    operation: review.operation,
                    relative_path: review.relative_path,
                    new_sha256: review.new_sha256,
                })
            })();
            let _ = host
                .workspace
                .revoke_pending(&input.conversation_id, &input.request_id);
            result
        })
        .await
        .map_err(|_| "native_dialog_unavailable".to_owned())?
    }
    #[cfg(not(windows))]
    {
        let _ = (app, input);
        Err("WORKSPACE_UNAVAILABLE".into())
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
        cancelled: host
            .sessions
            .lock()
            .map_err(|_| "host_unavailable")?
            .cancel(&request_id),
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
    let SendIntent {
        conversation_id,
        request_id,
        model_id,
        message,
    } = input;
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

    start_stream(
        app,
        conversation_id,
        request_id.clone(),
        model,
        admission,
        key,
        None,
    );
    Ok(Accepted {
        request_id,
        accepted: true,
    })
}

#[tauri::command]
async fn send_project_message(
    window: WebviewWindow,
    app: tauri::AppHandle,
    input: ProjectSendIntent,
) -> Result<Accepted, String> {
    trusted(&window)?;
    let model = Model::parse(&input.model_id).ok_or("unregistered_model")?;
    if !policy::valid_id(&input.conversation_id)
        || !policy::valid_id(&input.request_id)
        || !policy::valid_prompt(&input.message)
    {
        return Err("invalid_input".into());
    }
    #[cfg(windows)]
    {
        let prepared_app = app.clone();
        let prepared = tauri::async_runtime::spawn_blocking(move || {
            let host = prepared_app.state::<Host>();
            let preliminary = (|| {
                let guard = host.enter_native_dialog()?;
                let prompt = host
                    .workspace
                    .compose_project_prompt(
                        &input.conversation_id,
                        &input.request_id,
                        &input.relative_path,
                        &input.expected_sha256,
                        &input.message,
                    )
                    .map_err(|e| e.code().to_owned())?;
                let admission = host
                    .sessions
                    .lock()
                    .map_err(|_| "host_unavailable".to_owned())?
                    .reserve_ephemeral(&input.conversation_id, &input.request_id, model, prompt)
                    .map_err(|e| e.code().to_owned())?;
                Ok::<_, String>((guard, admission))
            })();
            let (_guard, admission) = match preliminary {
                Ok(prepared) => prepared,
                Err(error) => {
                    let _ = host
                        .workspace
                        .revoke_pending(&input.conversation_id, &input.request_id);
                    return Err(error);
                }
            };
            let reviewed = (|| {
                let body =
                    provider::exact_body(model, &admission.messages).map_err(|e| e.code())?;
                let document = host
                    .workspace
                    .prepare_disclosure_candidate(
                        &input.conversation_id,
                        &input.request_id,
                        &input.relative_path,
                        &input.expected_sha256,
                        body,
                    )
                    .map_err(|e| e.code())?;
                let owner = window.hwnd().map_err(|_| "native_review_unavailable")?.0 as usize;
                let approval = native_review::confirm_owned(&document, owner)
                    .map_err(|_| "native_review_unavailable")?
                    .ok_or("disclosure_denied")?;
                let body = host
                    .workspace
                    .finalize_disclosure_candidate(
                        &input.conversation_id,
                        &input.request_id,
                        approval,
                    )
                    .map_err(|e| e.code())?;
                if *admission.cancellation.borrow() {
                    return Err("cancelled");
                }
                let key = host.credentials.read().map_err(|e| e.code())?;
                Ok((body, key))
            })();
            match reviewed {
                Ok((body, key)) => Ok((
                    input.conversation_id,
                    input.request_id,
                    admission,
                    body,
                    key,
                )),
                Err(error) => {
                    if let Ok(mut sessions) = host.sessions.lock() {
                        let disposition = sessions.finish(
                            &input.conversation_id,
                            &input.request_id,
                            TransportOutcome::Failed,
                        );
                        if let Some(disposition) = disposition {
                            let status = if disposition == FinishDisposition::Cancelled {
                                "cancelled"
                            } else {
                                "failed"
                            };
                            let _ = prepared_app.emit_to(
                                MAIN_WINDOW,
                                STREAM_EVENT,
                                StreamEvent {
                                    request_id: &input.request_id,
                                    conversation_id: &input.conversation_id,
                                    kind: "terminal",
                                    text: None,
                                    status: Some(status),
                                    error_code: Some(error),
                                },
                            );
                        }
                    }
                    let _ = host
                        .workspace
                        .revoke_pending(&input.conversation_id, &input.request_id);
                    Err(error.to_owned())
                }
            }
        })
        .await
        .map_err(|_| "native_dialog_unavailable".to_owned())??;
        let (conversation_id, request_id, admission, body, key) = prepared;
        start_stream(
            app,
            conversation_id,
            request_id.clone(),
            model,
            admission,
            key,
            Some(body),
        );
        Ok(Accepted {
            request_id,
            accepted: true,
        })
    }
    #[cfg(not(windows))]
    {
        let _ = (app, input, model);
        Err("WORKSPACE_UNAVAILABLE".into())
    }
}

fn start_stream(
    worker_app: tauri::AppHandle,
    worker_conversation: String,
    worker_id: String,
    model: Model,
    admission: policy::Admission,
    key: zeroize::Zeroizing<String>,
    approved_body: Option<Vec<u8>>,
) {
    tauri::async_runtime::spawn(async move {
        let project_request = approved_body.is_some();
        let cancellation = admission.cancellation;
        let event_cancel = cancellation.clone();
        let on_content = |text: String| {
            if *event_cancel.borrow() {
                return Err(provider::ProviderFailure::Cancelled);
            }
            let session = worker_app.state::<Host>();
            let mut state = session
                .sessions
                .lock()
                .map_err(|_| provider::ProviderFailure::InvalidContract)?;
            if !state.mark_streaming(&worker_conversation, &worker_id) {
                return Err(provider::ProviderFailure::Cancelled);
            }
            // Keep the same host lock while enqueueing the event. A later
            // cancellation acknowledgement cannot precede a new host emit.
            let result = worker_app
                .emit_to(
                    MAIN_WINDOW,
                    STREAM_EVENT,
                    StreamEvent {
                        request_id: &worker_id,
                        conversation_id: &worker_conversation,
                        kind: "content",
                        text: Some(&text),
                        status: None,
                        error_code: None,
                    },
                )
                .map_err(|_| provider::ProviderFailure::InvalidContract);
            drop(state);
            result
        };
        let transport = async {
            match approved_body {
                Some(body) => {
                    provider::stream_chat_exact(
                        model,
                        &admission.messages,
                        body,
                        key,
                        cancellation,
                        on_content,
                    )
                    .await
                }
                None => {
                    provider::stream_chat(model, &admission.messages, key, cancellation, on_content)
                        .await
                }
            }
        };
        let response = AssertUnwindSafe(transport)
            .catch_unwind()
            .await
            .unwrap_or(Err(provider::ProviderFailure::InvalidContract));
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
            if project_request && disposition != FinishDisposition::Completed {
                let _ = state
                    .workspace
                    .revoke_pending(&worker_conversation, &worker_id);
            }
            let (status, error_code) = match disposition {
                FinishDisposition::Completed => ("completed", None),
                FinishDisposition::Cancelled => ("cancelled", None),
                FinishDisposition::Failed => ("failed", provider_error),
            };
            let _ = worker_app.emit_to(
                MAIN_WINDOW,
                STREAM_EVENT,
                StreamEvent {
                    request_id: &worker_id,
                    conversation_id: &worker_conversation,
                    kind: "terminal",
                    text: None,
                    status: Some(status),
                    error_code,
                },
            );
        }
    });
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum LaunchMode {
    Normal,
    ReviewFixture { large: bool },
    CredentialFixture,
    Invalid,
}

fn launch_mode(arguments: &[String]) -> LaunchMode {
    if arguments.is_empty() {
        return LaunchMode::Normal;
    }
    match credential_fixture::invocation(arguments) {
        credential_fixture::Invocation::Requested => return LaunchMode::CredentialFixture,
        credential_fixture::Invocation::Invalid => return LaunchMode::Invalid,
        credential_fixture::Invocation::NotRequested => (),
    }
    if arguments.len() != 1 {
        return LaunchMode::Invalid;
    }
    match arguments[0].as_str() {
        "--aster-review-fixture" => LaunchMode::ReviewFixture { large: false },
        "--aster-review-fixture-large" => LaunchMode::ReviewFixture { large: true },
        _ => LaunchMode::Invalid,
    }
}

fn launch_mode_os(arguments: impl IntoIterator<Item = std::ffi::OsString>) -> LaunchMode {
    let mut converted = Vec::new();
    for argument in arguments {
        // Only zero or one exact argument is admitted. Do not panic on invalid
        // Unicode or print rejected OS argument bytes through a panic hook.
        if !converted.is_empty() {
            return LaunchMode::Invalid;
        }
        match argument.into_string() {
            Ok(value) => converted.push(value),
            Err(_) => return LaunchMode::Invalid,
        }
    }
    launch_mode(&converted)
}

pub fn run() {
    // No supported arbitrary user CLI arguments exist. Every nonempty
    // invocation must select one exact synthetic diagnostic or exit before
    // Host/Tauri/provider initialization (including malformed fixture flags).
    match launch_mode_os(std::env::args_os().skip(1)) {
        LaunchMode::Normal => (),
        LaunchMode::Invalid => {
            eprintln!("aster_startup=invalid_arguments");
            std::process::exit(3);
        }
        LaunchMode::CredentialFixture => {
            let (classification, exit_code) = std::panic::catch_unwind(credential_fixture::run)
                .unwrap_or(("synthetic_gui_failed_closed", 3));
            eprintln!("aster_credential_fixture={classification}");
            // Every diagnostic outcome exits before normal host/provider
            // initialization; it cannot query the production credential.
            std::process::exit(exit_code);
        }
        LaunchMode::ReviewFixture { large } => {
            let result = native_review::run_fixture(large);
            let (classification, exit_code) = match result {
                Ok(true) => ("synthetic_review_confirmed_no_publication", 0),
                Ok(false) => ("synthetic_review_denied_no_publication", 2),
                Err(_) => ("synthetic_review_failed_closed_no_publication", 3),
            };
            eprintln!("aster_review_fixture={classification}");
            std::process::exit(exit_code);
        }
    }
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Host::default())
        .setup(|app| {
            let cleanup_app = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                    let _ = cleanup_app.state::<Host>().workspace.expire_pending();
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_status,
            configure_nvidia_credential,
            remove_nvidia_credential,
            select_project_root,
            select_project_file,
            send_project_message,
            review_project_proposal,
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
    fn os_launch_arguments_are_bounded_and_invalid_unicode_fails_closed() {
        use std::ffi::OsString;
        assert_eq!(launch_mode_os([]), LaunchMode::Normal);
        assert_eq!(
            launch_mode_os([OsString::from(credential_fixture::FLAG)]),
            LaunchMode::CredentialFixture
        );
        assert_eq!(
            launch_mode_os([
                OsString::from(credential_fixture::FLAG),
                OsString::from("extra")
            ]),
            LaunchMode::Invalid
        );
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStringExt;
            assert_eq!(
                launch_mode_os([OsString::from_wide(&[0xd800])]),
                LaunchMode::Invalid
            );
        }
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStringExt;
            assert_eq!(
                launch_mode_os([OsString::from_vec(vec![0xff])]),
                LaunchMode::Invalid
            );
        }
    }

    #[test]
    fn closed_launch_selector_never_starts_normal_host_for_nonempty_arguments() {
        let args = |values: &[&str]| {
            values
                .iter()
                .map(|value| (*value).to_owned())
                .collect::<Vec<_>>()
        };
        assert_eq!(launch_mode(&[]), LaunchMode::Normal);
        assert_eq!(
            launch_mode(&args(&[credential_fixture::FLAG])),
            LaunchMode::CredentialFixture
        );
        assert_eq!(
            launch_mode(&args(&["--aster-review-fixture"])),
            LaunchMode::ReviewFixture { large: false }
        );
        assert_eq!(
            launch_mode(&args(&["--aster-review-fixture-large"])),
            LaunchMode::ReviewFixture { large: true }
        );
        let mut normal_host_starts = 0;
        let mut diagnostic_starts = 0;
        for values in [
            vec!["--aster-credential"],
            vec!["--ASTER-CREDENTIAL-GUI-FIXTURE"],
            vec!["--aster-credential-gui-fixtur"],
            vec!["--aster-credential-gui-fixture=target"],
            vec![credential_fixture::FLAG, "extra"],
            vec!["--aster-review-fixture", "extra"],
            vec!["--aster-review-fixture-large=path"],
            vec!["--target=anything"],
            vec!["--password=anything"],
            vec!["--url=anything"],
            vec!["anything"],
        ] {
            match launch_mode(&args(&values)) {
                LaunchMode::Normal => normal_host_starts += 1,
                LaunchMode::CredentialFixture | LaunchMode::ReviewFixture { .. } => {
                    diagnostic_starts += 1
                }
                LaunchMode::Invalid => (),
            }
        }
        assert_eq!((normal_host_starts, diagnostic_starts), (0, 0));
    }

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
            candidate
                .as_object_mut()
                .unwrap()
                .extend(extra.as_object().unwrap().clone());
            assert!(serde_json::from_value::<SendIntent>(candidate).is_err());
        }
    }

    #[test]
    fn workspace_intents_never_admit_content_handles_or_approval_fields() {
        let file = serde_json::json!({"conversationId":"c1", "requestId":"r1"});
        let send = serde_json::json!({"conversationId":"c1", "requestId":"r1", "modelId":DEFAULT_MODEL,
            "message":"Review this file", "relativePath":"src/example.txt", "expectedSha256":"a".repeat(64)});
        let proposal =
            serde_json::json!({"conversationId":"c1", "requestId":"r1", "proposalJson":"{}"});
        assert!(serde_json::from_value::<SelectFileIntent>(file.clone()).is_ok());
        assert!(serde_json::from_value::<ProjectSendIntent>(send.clone()).is_ok());
        assert!(serde_json::from_value::<ProposalIntent>(proposal.clone()).is_ok());
        for field in [
            "content",
            "absolutePath",
            "ownerHwnd",
            "approved",
            "apiKey",
            "exactBody",
            "url",
            "journalDirectory",
        ] {
            let mut values = [file.clone(), send.clone(), proposal.clone()];
            for value in &mut values {
                value
                    .as_object_mut()
                    .unwrap()
                    .insert(field.into(), serde_json::json!("forged"));
            }
            assert!(serde_json::from_value::<SelectFileIntent>(values[0].clone()).is_err());
            assert!(serde_json::from_value::<ProjectSendIntent>(values[1].clone()).is_err());
            assert!(serde_json::from_value::<ProposalIntent>(values[2].clone()).is_err());
        }
    }

    #[test]
    fn closed_command_manifest_is_explicit_and_not_wildcard() {
        let manifest = include_str!("../build.rs");
        let capability = include_str!("../capabilities/main.json");
        for command in [
            "get_app_status",
            "configure_nvidia_credential",
            "remove_nvidia_credential",
            "select_project_root",
            "select_project_file",
            "send_project_message",
            "review_project_proposal",
            "send_message",
            "cancel_message",
        ] {
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
