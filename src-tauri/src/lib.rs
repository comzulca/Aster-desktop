//! Protocol 4 host: a fixed Contributor transport, temporary originals and
//! bounded finance. WebView messages are intent, never authority or secrets.
mod admission;
mod attachment_fixture;
mod attachment_processing;
mod attachment_worker;
mod attachments;
mod budget;
mod contributor;
mod contributor_policy;
#[cfg(all(test, windows))]
mod creation_process_fixture;
mod credential;
mod credential_fixture;
mod credential_validation;
mod customization;
mod customization_generation;
#[cfg(test)]
mod full_window_admission; // Content-free capacity core only; no active compaction.
mod github_api;
pub mod github_connection;
#[allow(dead_code)]
mod github_credential;
mod github_host;
#[cfg(test)]
mod github_mutation; // Private synthetic fixtures only; no runtime mutation capability.
mod history;
#[cfg(test)]
mod host_attachments_tests;
#[cfg(windows)]
mod journal_privacy;
mod native_operation_lease;
mod native_review;
mod native_visual;
#[cfg(windows)]
mod new_file; // Shared file identity types and frozen recovery, no IPC routes.
mod personalization;
mod policy;
mod private_store;
mod processed_history;
mod provider;
mod render_trace;
mod startup;
#[cfg(test)]
mod transcript_layout; // Synthetic fragment/quota seams only; no active store migration.
mod webview_render_observer;
mod window_controls;
mod window_guard;
#[allow(dead_code)]
mod workspace; // Shared privacy error types/frozen recovery, no v0.2 capability.

use attachment_processing::ProcessedContent;
use attachments::{AdmittedAttachment, AttachmentKind, AttachmentMetadata, AttachmentRegistry};
use budget::{Budget, BudgetStatus, Configuration};
use contributor::{Contributor, ContributorStatus};
use credential::CredentialStore;
use credential_validation::{CredentialValidation, ValidationState};
use futures_util::FutureExt;
use history::{Conversation, ConversationSummary, EntryState, History};
use native_operation_lease::{NativeOperationGate, NativeOperationLease};
use native_review::ReviewDocument;
use policy::{
    FinishDisposition, Model, SessionState, TransportOutcome, DEFAULT_MODEL, POLICY_VERSION,
};
use private_store::StoreError;
use processed_history::ProcessedHistory;
use provider::{ProviderFailure, ProviderOutcome, ProviderTerminal, ReasoningEffort};
use serde::{Deserialize, Serialize};
use std::{
    panic::AssertUnwindSafe,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{Emitter, Manager, State, WebviewWindow};
use tauri_plugin_dialog::DialogExt;

const MAIN_WINDOW: &str = "main";
const STREAM_EVENT: &str = "aster://stream";
#[cfg(test)]
const COMMANDS: &[&str] = &[
    "get_app_status",
    "configure_meta_credential",
    "remove_meta_credential",
    "verify_meta_credential",
    "get_personalization",
    "save_personalization",
    "clear_personalization",
    "get_customization",
    "get_customization_package",
    "save_customization",
    "set_customization_enabled",
    "import_customization",
    "generate_customization",
    "get_github_status",
    "begin_github_connection",
    "poll_github_connection",
    "cancel_github_connection",
    "disconnect_github",
    "list_github_repositories",
    "select_github_repository",
    "read_github_file",
    "record_render_event",
    "get_render_trace",
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
];

struct Persistence {
    history: History,
    processed: Result<ProcessedHistory, StoreError>,
    budget: Budget,
    contributor: Contributor,
}
impl Persistence {
    fn open(base: &Path) -> Result<Self, StoreError> {
        Ok(Self {
            history: History::open(base)?,
            processed: ProcessedHistory::open(base),
            budget: Budget::open(base)?,
            contributor: Contributor::open(base)?,
        })
    }
}
struct Host {
    credentials: CredentialStore,
    credential_validation: Mutex<Result<CredentialValidation, StoreError>>,
    personalization: Mutex<Result<personalization::Personalization, StoreError>>,
    customization: Mutex<Result<customization::Customization, StoreError>>,
    github: Mutex<github_host::GithubHost>,
    sessions: Mutex<SessionState>,
    attachments: Mutex<AttachmentRegistry>,
    persistent: Mutex<Result<Persistence, StoreError>>,
    native_dialog_open: NativeOperationGate,
    expiry_shutdown: AtomicBool,
    window_close: window_guard::CloseState,
}
impl Host {
    fn enter_native_dialog(&self) -> Result<NativeOperationLease, String> {
        self.native_dialog_open
            .acquire(|| self.window_close.is_closing())
            .map_err(str::to_owned)
    }
    fn idle(&self) -> Result<(), String> {
        if self.window_close.is_closing() {
            return Err("window_closing".into());
        }
        if self
            .sessions
            .lock()
            .map_err(|_| "state_unavailable")?
            .active_count()
            != 0
        {
            Err("busy".into())
        } else {
            Ok(())
        }
    }
}
fn now() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|_| "clock_unavailable".into())
}
fn trusted(window: &WebviewWindow) -> Result<(), String> {
    if window.label() == MAIN_WINDOW {
        Ok(())
    } else {
        Err("caller_denied".into())
    }
}
fn code(error: StoreError) -> String {
    error.code().into()
}
fn persistent(value: &mut Result<Persistence, StoreError>) -> Result<&mut Persistence, String> {
    value.as_mut().map_err(|e| code(*e))
}

#[tauri::command]
fn get_personalization(
    window: WebviewWindow,
    host: State<'_, Host>,
) -> Result<personalization::PersonalizationSnapshot, String> {
    trusted(&window)?;
    host.personalization
        .lock()
        .map_err(|_| "state_unavailable")?
        .as_ref()
        .map_err(|e| code(*e))?
        .snapshot()
        .map_err(code)
}

#[tauri::command]
fn save_personalization(
    window: WebviewWindow,
    host: State<'_, Host>,
    input: personalization::SavePersonalizationInput,
) -> Result<personalization::PersonalizationSnapshot, String> {
    trusted(&window)?;
    let _guard = host.enter_native_dialog()?;
    host.idle()?;
    host.personalization
        .lock()
        .map_err(|_| "state_unavailable")?
        .as_mut()
        .map_err(|e| code(*e))?
        .save(input)
        .map_err(code)
}

#[tauri::command]
fn clear_personalization(
    window: WebviewWindow,
    host: State<'_, Host>,
    input: personalization::ExpectedRevisionInput,
) -> Result<personalization::PersonalizationSnapshot, String> {
    trusted(&window)?;
    let _guard = host.enter_native_dialog()?;
    host.idle()?;
    host.personalization
        .lock()
        .map_err(|_| "state_unavailable")?
        .as_mut()
        .map_err(|e| code(*e))?
        .clear(input)
        .map_err(code)
}

#[tauri::command]
fn get_customization(
    window: WebviewWindow,
    host: State<'_, Host>,
) -> Result<customization::CatalogueSnapshot, String> {
    trusted(&window)?;
    host.customization
        .lock()
        .map_err(|_| "state_unavailable")?
        .as_ref()
        .map_err(|e| code(*e))?
        .snapshot()
        .map_err(code)
}

#[tauri::command]
fn get_customization_package(
    window: WebviewWindow,
    host: State<'_, Host>,
    input: customization::PackageIdInput,
) -> Result<customization::PackageView, String> {
    trusted(&window)?;
    host.customization
        .lock()
        .map_err(|_| "state_unavailable")?
        .as_ref()
        .map_err(|e| code(*e))?
        .package(&input.id)
        .map_err(code)
}

#[tauri::command]
fn save_customization(
    window: WebviewWindow,
    host: State<'_, Host>,
    input: customization::SavePackageInput,
) -> Result<customization::CatalogueSnapshot, String> {
    trusted(&window)?;
    let _guard = host.enter_native_dialog()?;
    host.idle()?;
    host.customization
        .lock()
        .map_err(|_| "state_unavailable")?
        .as_mut()
        .map_err(|e| code(*e))?
        .save(input)
        .map_err(code)
}

#[tauri::command]
fn set_customization_enabled(
    window: WebviewWindow,
    host: State<'_, Host>,
    input: customization::SetPackageEnabledInput,
) -> Result<customization::CatalogueSnapshot, String> {
    trusted(&window)?;
    // Cancellation must remain available while a device/read operation owns
    // the native-operation guard. Denying a stale preference revision may not
    // restore the cancelled session or grant any runtime authority.
    if input.id == customization::BUILTIN_GITHUB_ID && !input.enabled {
        host.github
            .lock()
            .map_err(|_| "github_state_unavailable")?
            .set_enabled(false)?;
        return host
            .customization
            .lock()
            .map_err(|_| "state_unavailable")?
            .as_mut()
            .map_err(|e| code(*e))?
            .set_enabled(input)
            .map_err(code);
    }
    let _guard = host.enter_native_dialog()?;
    host.idle()?;
    let enable_github = input.id == customization::BUILTIN_GITHUB_ID && input.enabled;
    let snapshot = host
        .customization
        .lock()
        .map_err(|_| "state_unavailable")?
        .as_mut()
        .map_err(|e| code(*e))?
        .set_enabled(input)
        .map_err(code)?;
    if enable_github {
        host.github
            .lock()
            .map_err(|_| "github_state_unavailable")?
            .set_enabled(true)?;
    }
    Ok(snapshot)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ImportCustomizationIntent {
    expected_revision: u64,
}

#[tauri::command]
async fn import_customization(
    window: WebviewWindow,
    app: tauri::AppHandle,
    host: State<'_, Host>,
    input: ImportCustomizationIntent,
) -> Result<customization::CatalogueSnapshot, String> {
    trusted(&window)?;
    let guard = host.enter_native_dialog()?;
    host.idle()?;
    owner(&window)?;
    let worker_lease = guard.worker_lease();
    let bytes = tauri::async_runtime::spawn_blocking(move || {
        let _worker_lease = worker_lease;
        let selected = app
            .dialog()
            .file()
            .set_parent(&window)
            .set_title("Aster: import an inert skill or plugin package")
            .add_filter("Aster inert JSON package", &["json"])
            .blocking_pick_file()
            .ok_or_else(|| "cancelled".to_owned())?;
        let path = selected.as_path().ok_or("package_denied")?;
        if path
            .extension()
            .and_then(|v| v.to_str())
            .is_none_or(|v| !v.eq_ignore_ascii_case("json"))
        {
            return Err("package_denied".to_owned());
        }
        let file = attachments::read_selected_native_bounded(path, customization::MAX_IMPORT_BYTES)
            .map_err(|_| "package_denied".to_owned())?;
        if file.bytes.len() > customization::MAX_IMPORT_BYTES {
            return Err("package_capacity".to_owned());
        }
        Ok::<_, String>(file.bytes)
    })
    .await
    .map_err(|_| "package_denied")??;
    host.idle()?;
    host.customization
        .lock()
        .map_err(|_| "state_unavailable")?
        .as_mut()
        .map_err(|e| code(*e))?
        .import_json(input.expected_revision, &bytes)
        .map_err(code)
}
#[cfg(windows)]
fn owner(window: &WebviewWindow) -> Result<usize, String> {
    window
        .hwnd()
        .map(|h| h.0 as usize)
        .map_err(|_| "native_owner_unavailable".into())
}
#[cfg(not(windows))]
fn owner(_: &WebviewWindow) -> Result<usize, String> {
    Err("unsupported_platform".into())
}
async fn review(
    window: &WebviewWindow,
    document: ReviewDocument,
    guard: &NativeOperationLease,
) -> Result<bool, String> {
    let hwnd = owner(window)?;
    let worker_lease = guard.worker_lease();
    tauri::async_runtime::spawn_blocking(move || {
        let _worker_lease = worker_lease;
        match native_review::confirm_owned(&document, hwnd).map_err(|error| match error {
            native_review::ReviewError::Cancelled => "cancelled".to_owned(),
            _ => "native_review_unavailable".to_owned(),
        })? {
            Some(approval) => {
                approval
                    .consume(document.binding_sha256())
                    .map_err(|error| match error {
                        native_review::ReviewError::Cancelled => "cancelled".to_owned(),
                        _ => "native_review_expired".to_owned(),
                    })?;
                Ok(true)
            }
            None => Ok(false),
        }
    })
    .await
    .map_err(|_| "native_review_unavailable".to_owned())?
}
fn operation(value: serde_json::Value) -> Result<ReviewDocument, String> {
    ReviewDocument::proposal(&value.to_string()).map_err(|_| "native_review_unavailable".into())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Capabilities {
    attachment_import: bool,
    project_read: bool,
    proposals: bool,
    apply_new_file: bool,
    apply_existing_file: bool,
}
#[derive(Serialize)]
struct ModelDisplay {
    id: &'static str,
    label: &'static str,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppStatus {
    protocol_version: u32,
    policy_version: u32,
    configured: bool,
    credential_available: bool,
    credential_validation: ValidationState,
    default_model: &'static str,
    models: [ModelDisplay; 1],
    active_request: bool,
    project_selected: bool,
    project_label: Option<String>,
    capabilities: Capabilities,
    availability: &'static str,
    contributor: ContributorStatus,
    history_available: bool,
    budget: BudgetStatus,
}
#[derive(Serialize)]
struct CredentialStatus {
    configured: bool,
}
#[derive(Serialize)]
struct ValidationReceipt {
    state: ValidationState,
}
#[derive(Serialize)]
struct TermsReceipt {
    accepted: bool,
}
#[derive(Serialize)]
struct BudgetReceipt {
    configured: bool,
}
#[derive(Serialize)]
struct ReconcileReceipt {
    reconciled: bool,
}
#[derive(Serialize)]
struct DeleteReceipt {
    deleted: bool,
}
#[derive(Serialize)]
struct Cancelled {
    cancelled: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Accepted {
    request_id: String,
    accepted: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ConversationIntent {
    conversation_id: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReconcileIntent {
    request_id: String,
    confirmed_charge_nano_gbp: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SendIntent {
    conversation_id: String,
    request_id: String,
    model_id: String,
    message: String,
    data_class: String,
    reasoning_effort: String,
    #[serde(default)]
    attachment_ids: Vec<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AttachmentIntent {
    conversation_id: String,
    attachment_id: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProcessedIntent {
    conversation_id: String,
    request_id: String,
    attachment_id: String,
}
#[derive(Serialize)]
struct RemovedAttachment {
    removed: bool,
}
#[derive(Serialize)]
struct ViewedAttachment {
    viewed: bool,
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
    error_code: Option<&'a str>,
}

#[tauri::command]
fn get_app_status(window: WebviewWindow, host: State<'_, Host>) -> Result<AppStatus, String> {
    trusted(&window)?;
    let time = now()?;
    let mut state = host.persistent.lock().map_err(|_| "state_unavailable")?;
    let p = persistent(&mut state)?;
    let budget = p.budget.status(time);
    let contributor = p.contributor.status(time);
    let history_available = p.history.list().is_ok();
    let active = host
        .sessions
        .lock()
        .map_err(|_| "state_unavailable")?
        .active_count()
        != 0;
    let key = host.credentials.configured();
    let credential_available = key.is_ok();
    let configured = key.unwrap_or(false);
    let credential_validation = credential_state(&host, configured);
    let writable = p.history.can_append_request().unwrap_or(false) && p.budget.can_append_request();
    let availability = if !cfg!(windows) {
        "unsupported"
    } else if !history_available || !credential_available || !writable {
        "attention_required"
    } else if active {
        "busy"
    } else if !contributor.accepted {
        "needs_terms"
    } else if !configured || credential_validation != ValidationState::Valid {
        "needs_credential"
    } else if budget.requires_reconciliation {
        "attention_required"
    } else if !budget.configured {
        "needs_budget"
    } else if !p.budget.available(time) {
        "attention_required"
    } else {
        "ready"
    };
    Ok(AppStatus {
        protocol_version: 5,
        policy_version: POLICY_VERSION,
        configured,
        credential_available,
        credential_validation,
        default_model: DEFAULT_MODEL,
        models: [ModelDisplay {
            id: DEFAULT_MODEL,
            label: "Muse Spark 1.3 Contributor",
        }],
        active_request: active,
        project_selected: false,
        project_label: None,
        capabilities: Capabilities {
            attachment_import: cfg!(windows)
                && p.processed
                    .as_ref()
                    .ok()
                    .is_some_and(|media| media.import_available().unwrap_or(false)),
            project_read: false,
            proposals: false,
            apply_new_file: false,
            apply_existing_file: false,
        },
        availability,
        contributor,
        history_available,
        budget,
    })
}

fn invalidate_identity(host: &Host) -> Result<(), String> {
    host.credential_validation
        .lock()
        .map_err(|_| "state_unavailable")?
        .as_mut()
        .map_err(|e| code(*e))?
        .revoke()
        .map_err(code)?;
    host.attachments
        .lock()
        .map_err(|_| "state_unavailable")?
        .invalidate();
    let mut state = host.persistent.lock().map_err(|_| "state_unavailable")?;
    let p = persistent(&mut state)?;
    // Revoke first, so a failed history write cannot leave permission valid.
    p.contributor.set(false, now()?).map_err(code)?;
    p.history.invalidate_continuation(now()?).map_err(code)
}
#[tauri::command]
async fn configure_meta_credential(
    window: WebviewWindow,
    app: tauri::AppHandle,
    host: State<'_, Host>,
) -> Result<CredentialStatus, String> {
    trusted(&window)?;
    let guard = host.enter_native_dialog()?;
    host.idle()?;
    // No key is read or sent to the WebView; saved history and finances survive.
    let hwnd = owner(&window)?;
    let worker_lease = guard.worker_lease();
    let configured = tauri::async_runtime::spawn_blocking(move || {
        let _worker_lease = worker_lease;
        CredentialStore
            .enter_with_native_dialog_after(hwnd, || {
                invalidate_identity(&app.state::<Host>())
                    .map_err(|_| credential::CredentialError::Rejected)
            })
            .map_err(|e| e.code().to_owned())
    })
    .await
    .map_err(|_| "credential_unavailable")??;
    if configured {
        // Native review explicitly covers the fixed non-inference GET.
        let _ = verify_credential_inner(&window, &host, &guard).await?;
    }
    Ok(CredentialStatus { configured })
}

fn credential_state(host: &Host, configured: bool) -> ValidationState {
    if !configured {
        return ValidationState::Missing;
    }
    let Ok(key) = host.credentials.read() else {
        return ValidationState::Unavailable;
    };
    host.credential_validation
        .lock()
        .ok()
        .and_then(|proof| proof.as_ref().ok().map(|p| p.state(Some(key.as_str()))))
        .unwrap_or(ValidationState::Unavailable)
}

fn require_valid_credential(host: &Host) -> Result<(), String> {
    let key = host
        .credentials
        .read()
        .map_err(|_| "credential_validation_required")?;
    host.credential_validation
        .lock()
        .map_err(|_| "state_unavailable")?
        .as_ref()
        .map_err(|e| code(*e))?
        .require(&key)
        .map_err(str::to_owned)
}

async fn verify_credential_inner(
    window: &WebviewWindow,
    host: &Host,
    guard: &NativeOperationLease,
) -> Result<ValidationReceipt, String> {
    if !host.credentials.configured().map_err(|e| e.code())? {
        return Ok(ValidationReceipt {
            state: ValidationState::Missing,
        });
    }
    if !review(window, operation(serde_json::json!({
        "operation":"Verify configured Meta API credential",
        "destination":provider::MODEL_VALIDATION_ENDPOINT,
        "effect":"One fixed model metadata GET. The native key is sent only to Meta. No conversation, attachment, Personalization or inference is sent. This proves authentication/model access only, not quota or billing. No automatic retry."
    }))?, guard).await? {
        return Ok(ValidationReceipt { state: credential_state(host, true) });
    }
    let key = host.credentials.read().map_err(|e| e.code())?;
    if !host
        .credential_validation
        .lock()
        .map_err(|_| "state_unavailable")?
        .as_ref()
        .map_err(|e| code(*e))?
        .can_verify()
        .map_err(code)?
    {
        return Err("credential_proof_capacity".into());
    }
    host.credential_validation
        .lock()
        .map_err(|_| "state_unavailable")?
        .as_mut()
        .map_err(|e| code(*e))?
        .revoke()
        .map_err(code)?;
    let result = provider::verify_registered_model(key.clone()).await;
    // A concurrent external credential edit cannot validate different bytes.
    let current = host.credentials.read().map_err(|e| e.code())?;
    if current.as_str() != key.as_str() {
        return Err("credential_validation_stale".into());
    }
    let state = match result {
        Ok(()) => {
            host.credential_validation
                .lock()
                .map_err(|_| "state_unavailable")?
                .as_mut()
                .map_err(|e| code(*e))?
                .record_result(&key, true)
                .map_err(code)?;
            ValidationState::Valid
        }
        Err(ProviderFailure::Authentication) => {
            host.credential_validation
                .lock()
                .map_err(|_| "state_unavailable")?
                .as_mut()
                .map_err(|e| code(*e))?
                .record_result(&key, false)
                .map_err(code)?;
            ValidationState::Invalid
        }
        Err(_) => ValidationState::Unavailable,
    };
    Ok(ValidationReceipt { state })
}

#[tauri::command]
async fn verify_meta_credential(
    window: WebviewWindow,
    host: State<'_, Host>,
) -> Result<ValidationReceipt, String> {
    trusted(&window)?;
    let guard = host.enter_native_dialog()?;
    host.idle()?;
    verify_credential_inner(&window, &host, &guard).await
}
#[tauri::command]
async fn remove_meta_credential(
    window: WebviewWindow,
    host: State<'_, Host>,
) -> Result<CredentialStatus, String> {
    trusted(&window)?;
    let guard = host.enter_native_dialog()?;
    host.idle()?;
    if !review(&window,operation(serde_json::json!({"operation":"Remove Meta API credential","target":"AsterDesktop/MetaModelAPI/v1","preserved":"Earlier provider credentials, protected conversation display history and financial reservations remain."}))?, &guard).await? {return Ok(CredentialStatus{configured:host.credentials.configured().map_err(|e|e.code())?});}
    // A full/unavailable proof or history store must never prevent the owner
    // deleting the native secret. Revocation still latches closed first.
    let revocation = invalidate_identity(&host);
    host.credentials.delete().map_err(|e| e.code())?;
    revocation?;
    Ok(CredentialStatus { configured: false })
}
#[tauri::command]
async fn accept_contributor_terms(
    window: WebviewWindow,
    host: State<'_, Host>,
) -> Result<TermsReceipt, String> {
    trusted(&window)?;
    let guard = host.enter_native_dialog()?;
    host.idle()?;
    if !contributor_policy::current(now()?) {
        return Err("contributor_policy_expired".into());
    }
    let document = operation(
        serde_json::json!({"operation":"Contributor owner attestation","termsRevision":contributor::TERMS_REVISION,
        "model":DEFAULT_MODEL,"destination":provider::ENDPOINT,"disclosure":"Meta Contributor may use prompts and answers for training. Personal, sensitive and confidential content, including private source code, must not be sent. Every complete request and prior answer needs fresh native review.",
        "ownerAttestation":"I have checked that my account, territory and intended use are eligible for this direct Meta API. I use a direct API key, not a coding-harness subscription key. Aster has not verified account entitlement.",
        "spending":"Cancelling, refusing or failing after dispatch does not establish zero cost. Missing usage requires statement-confirmed reconciliation. The £89 local limit does not cap other clients or guarantee Meta's invoice.",
        "terms":"https://dev.meta.ai/legal/terms-of-service","geography":"https://dev.meta.ai/legal/geographic-use-policy",
        "geographyRevision":contributor_policy::GEOGRAPHY_REVISION,"policyReviewedOn":contributor_policy::REVIEWED_ON,
        "expiry":contributor_policy::EXPIRES_ISO,"expiryMeaning":"Aster policy review deadline; not Meta account, key or subscription expiry."}),
    )?;
    if !review(&window, document, &guard).await? {
        return Ok(TermsReceipt { accepted: false });
    }
    if !contributor_policy::current(now()?) {
        return Err("contributor_policy_expired".into());
    }
    let mut state = host.persistent.lock().map_err(|_| "state_unavailable")?;
    persistent(&mut state)?
        .contributor
        .set(true, now()?)
        .map_err(code)?;
    Ok(TermsReceipt { accepted: true })
}
#[tauri::command]
async fn configure_budget(
    window: WebviewWindow,
    host: State<'_, Host>,
    input: Configuration,
) -> Result<BudgetReceipt, String> {
    trusted(&window)?;
    let guard = host.enter_native_dialog()?;
    host.idle()?;
    let time = now()?;
    if !contributor_policy::current(time) {
        return Err("contributor_policy_expired".into());
    }
    budget::validate_configuration_proposal(&input, time).map_err(str::to_owned)?;
    let (status, period_review) = {
        let mut state = host.persistent.lock().map_err(|_| "state_unavailable")?;
        let p = persistent(&mut state)?;
        (
            p.budget.status(time),
            p.budget.review_context(None).map_err(code)?,
        )
    };
    let document = operation(
        serde_json::json!({"operation":"Confirm bounded financial configuration and London calendar period","configuration":input,"currentLedger":status,"previousPeriodAndClock":period_review,
        "period":budget::period(time).map_err(code)?,"policyReviewedOn":contributor_policy::REVIEWED_ON,"policyExpiresAt":contributor_policy::EXPIRES_ISO,"statement":"I have checked the actual date and London calendar month. This explicitly activates the displayed period; changing the account alias does not reset charges. Earlier uncertain reservations remain. The declared GBP/USD rate, positive margin, tax and payment costs conservatively bound my invoice until the stated expiry. No exchange rate or tax exemption is verified by Aster.","limitNanoGbp":budget::LIMIT.to_string(),"noReset":true}),
    )?;
    if !review(&window, document, &guard).await? {
        return Ok(BudgetReceipt { configured: false });
    }
    let mut state = host.persistent.lock().map_err(|_| "state_unavailable")?;
    let p = persistent(&mut state)?;
    let commit_time = now()?;
    budget::validate_configuration_proposal(&input, commit_time).map_err(str::to_owned)?;
    // Bind the native review to its actual London period before any identity
    // mutation. The durable Configure transition checks it again at commit.
    p.budget
        .validate_configuration_review(&input, time, commit_time)
        .map_err(code)?;
    let old_account = p.budget.status(commit_time).account_id;
    if old_account
        .as_deref()
        .is_some_and(|id| id != input.account_id)
    {
        p.contributor.set(false, now()?).map_err(code)?;
        p.history.invalidate_continuation(now()?).map_err(code)?;
    }
    p.budget.configure(input, time, now()?).map_err(code)?;
    Ok(BudgetReceipt { configured: true })
}
#[tauri::command]
async fn reconcile_budget(
    window: WebviewWindow,
    host: State<'_, Host>,
    input: ReconcileIntent,
) -> Result<ReconcileReceipt, String> {
    trusted(&window)?;
    let guard = host.enter_native_dialog()?;
    host.idle()?;
    if !admission::opaque_id(&input.request_id)
        || input.confirmed_charge_nano_gbp.is_empty()
        || input.confirmed_charge_nano_gbp.len() > 12
        || !input
            .confirmed_charge_nano_gbp
            .bytes()
            .all(|b| b.is_ascii_digit())
    {
        return Err("invalid_reconciliation".into());
    }
    let charge = input
        .confirmed_charge_nano_gbp
        .parse::<u64>()
        .map_err(|_| "invalid_reconciliation")?;
    let (pending, original_quote) = {
        let mut state = host.persistent.lock().map_err(|_| "state_unavailable")?;
        let p = persistent(&mut state)?;
        let pending = p
            .budget
            .status(now()?)
            .pending
            .into_iter()
            .find(|p| p.request_id == input.request_id)
            .ok_or("request_not_pending")?;
        (
            pending,
            p.budget
                .review_context(Some(&input.request_id))
                .map_err(code)?,
        )
    };
    let document = operation(
        serde_json::json!({"operation":"Statement-confirmed reconciliation, not API-measured usage","pending":pending,"originalAccountAndQuote":original_quote,"confirmedChargeNanoGbp":charge.to_string(),"ownerAttestation":"I have checked the original account's Meta dashboard or invoice and this is the exact final GBP charge attributable to this request, including currency conversion, tax and payment costs. A zero amount requires explicit evidence that no charge occurred; an undispatched reservation is not API-measured billed usage. This does not reset other requests or the financial ledger. A charge exceeding the reserved bound blocks new dispatch pending engineering review."}),
    )?;
    if !review(&window, document, &guard).await? {
        return Ok(ReconcileReceipt { reconciled: false });
    }
    let mut state = host.persistent.lock().map_err(|_| "state_unavailable")?;
    persistent(&mut state)?
        .budget
        .reconcile(&input.request_id, charge, now()?)
        .map_err(code)?;
    Ok(ReconcileReceipt { reconciled: true })
}
#[tauri::command]
fn list_conversations(
    window: WebviewWindow,
    host: State<'_, Host>,
) -> Result<Vec<ConversationSummary>, String> {
    trusted(&window)?;
    let mut state = host.persistent.lock().map_err(|_| "state_unavailable")?;
    persistent(&mut state)?.history.list().map_err(code)
}
#[tauri::command]
fn create_conversation(
    window: WebviewWindow,
    host: State<'_, Host>,
) -> Result<Conversation, String> {
    create_conversation_for_origin(window, host, SendOrigin::Conversation)
}

// Not serializable and never an IPC field: only the fixed generation command
// may bypass the main chat's temporary-original lifecycle.
#[derive(Clone, Copy, PartialEq, Eq)]
enum SendOrigin {
    Conversation,
    InertPackageGeneration,
}

fn create_conversation_for_origin(
    window: WebviewWindow,
    host: State<'_, Host>,
    origin: SendOrigin,
) -> Result<Conversation, String> {
    trusted(&window)?;
    let _guard = host.enter_native_dialog()?;
    host.idle()?;
    require_valid_credential(&host)?;
    if origin == SendOrigin::Conversation {
        host.attachments
            .lock()
            .map_err(|_| "state_unavailable")?
            .invalidate();
    }
    let mut state = host.persistent.lock().map_err(|_| "state_unavailable")?;
    host.idle()?;
    persistent(&mut state)?.history.create(now()?).map_err(code)
}
#[tauri::command]
fn load_conversation(
    window: WebviewWindow,
    host: State<'_, Host>,
    input: ConversationIntent,
) -> Result<Conversation, String> {
    trusted(&window)?;
    let _guard = host.enter_native_dialog()?;
    host.idle()?;
    host.attachments
        .lock()
        .map_err(|_| "state_unavailable")?
        .invalidate();
    if !admission::opaque_id(&input.conversation_id) {
        return Err("invalid_input".into());
    }
    let mut state = host.persistent.lock().map_err(|_| "state_unavailable")?;
    host.idle()?;
    let p = persistent(&mut state)?;
    checked_conversation(p, &input.conversation_id)
}
#[tauri::command]
async fn delete_conversation(
    window: WebviewWindow,
    host: State<'_, Host>,
    input: ConversationIntent,
) -> Result<DeleteReceipt, String> {
    trusted(&window)?;
    let guard = host.enter_native_dialog()?;
    host.idle()?;
    if !admission::opaque_id(&input.conversation_id) {
        return Err("invalid_input".into());
    }
    let conversation = {
        let mut state = host.persistent.lock().map_err(|_| "state_unavailable")?;
        persistent(&mut state)?
            .history
            .load(&input.conversation_id)
            .map_err(code)?
    };
    if !review(&window,operation(serde_json::json!({"operation":"Remove conversation from visible history","conversationId":conversation.id,"entryCount":conversation.entries.len(),"effect":"Logical removal only. Encrypted recovery records remain and storage quota is not reclaimed. This is not permanent or secure erasure. Financial records and reservations are preserved."}))?, &guard).await?{return Ok(DeleteReceipt{deleted:false});}
    host.attachments
        .lock()
        .map_err(|_| "state_unavailable")?
        .invalidate();
    let mut state = host.persistent.lock().map_err(|_| "state_unavailable")?;
    persistent(&mut state)?
        .history
        .remove_after_native_confirmation(&input.conversation_id, now()?)
        .map_err(code)?;
    Ok(DeleteReceipt { deleted: true })
}

/// References are usable only when both independently protected domains bind
/// the same complete processed allocation. No read or disclosure grant returns.
fn checked_conversation(p: &Persistence, id: &str) -> Result<Conversation, String> {
    checked_history(&p.history, &p.processed, id)
}
fn checked_history(
    history: &History,
    processed: &Result<ProcessedHistory, StoreError>,
    id: &str,
) -> Result<Conversation, String> {
    let conversation = history.load(id).map_err(code)?;
    for entry in &conversation.entries {
        if let Some(files) = &entry.processed_attachments {
            let media = processed.as_ref().map_err(|e| code(*e))?;
            for file in files {
                let (binding, class) = history
                    .processed_reference(id, &entry.request_id, &file.id)
                    .map_err(code)?;
                media
                    .load_bound(id, &entry.request_id, &binding, &class)
                    .map_err(code)?;
            }
        }
    }
    Ok(conversation)
}

fn attachment_prompt(message: &str, files: &[AdmittedAttachment]) -> Result<String, String> {
    if message.len() > policy::MAX_MESSAGE_BYTES {
        return Err("attachment_capacity".into());
    }
    let mut prompt = message.to_owned();
    for (index, file) in files.iter().enumerate() {
        match &file.processed {
            ProcessedContent::Text(text) => {
                prompt.push_str(&format!(
                    "\n\n[Inert processed text attachment {}]\n",
                    index + 1
                ));
                prompt.push_str(text);
                prompt.push_str("\n[End processed attachment]");
            }
            ProcessedContent::Pages(_) => {
                prompt.push_str(&format!("\n\n[Attachment {}: {}. Its native PDF or normalised image accompanies this request only. Saved pages remain local; reattach the original for another complete native reading.]", index + 1, if file.metadata.kind == AttachmentKind::Pdf { "complete native PDF" } else { "complete normalised image" }));
            }
        }
        if prompt.len() > policy::MAX_MESSAGE_BYTES {
            return Err("attachment_capacity".into());
        }
    }
    if !admission::admissible_text(&prompt) {
        return Err("data_admission_denied".into());
    }
    Ok(prompt)
}

fn visual_pages(
    content: &ProcessedContent,
    label: &str,
) -> Result<Vec<native_visual::VisualPage>, String> {
    match content {
        ProcessedContent::Text(_) => Ok(Vec::new()),
        ProcessedContent::Pages(pages) => pages
            .iter()
            .enumerate()
            .map(|(index, page)| {
                native_visual::VisualPage::from_rgba(
                    format!("{label} · {}", index + 1),
                    page.width,
                    page.height,
                    &page.rgba,
                )
                .map_err(str::to_owned)
            })
            .collect(),
    }
}

#[tauri::command]
async fn select_attachments(
    window: WebviewWindow,
    app: tauri::AppHandle,
    host: State<'_, Host>,
    input: ConversationIntent,
) -> Result<Vec<AttachmentMetadata>, String> {
    trusted(&window)?;
    let guard = host.enter_native_dialog()?;
    host.idle()?;
    require_valid_credential(&host)?;
    if !cfg!(windows) || !admission::opaque_id(&input.conversation_id) {
        return Err("attachment_denied".into());
    }
    {
        let mut state = host.persistent.lock().map_err(|_| "state_unavailable")?;
        let p = persistent(&mut state)?;
        if !p
            .processed
            .as_ref()
            .map_err(|e| code(*e))?
            .import_available()
            .map_err(code)?
        {
            return Err("attachment_capacity".into());
        }
        checked_conversation(p, &input.conversation_id)?;
    }
    // Snapshot only host-owned metadata. The expiry monitor may invalidate it
    // while the picker runs; append revalidates this exact selection atomically.
    let previous = host
        .attachments
        .lock()
        .map_err(|_| "state_unavailable")?
        .selection_metadata(&input.conversation_id)
        .map_err(|e| e.code().to_owned())?;
    let expected_ids: Vec<_> = previous.iter().map(|a| a.id.clone()).collect();
    owner(&window)?;
    let worker_lease = guard.worker_lease();
    let batch = tauri::async_runtime::spawn_blocking(move || {
        let _worker_lease = worker_lease;
        let paths = app
            .dialog()
            .file()
            .set_parent(&window)
            .set_title("Aster: select eligible public or synthetic attachments")
            .add_filter(
                "Aster direct formats",
                &[
                    "pdf", "txt", "md", "json", "xml", "yaml", "csv", "tsv", "png", "jpg", "jpeg",
                    "gif",
                ],
            )
            .blocking_pick_files();
        let Some(paths) = paths else {
            return Ok(Vec::new());
        };
        if paths.is_empty() || paths.len() + previous.len() > attachments::MAX_FILES {
            return Err("attachment_capacity".to_owned());
        }
        let mut originals = Vec::new();
        let mut bytes: usize = previous.iter().map(|a| a.source_bytes).sum();
        for path in paths {
            let selected =
                attachments::read_selected_native(path.as_path().ok_or("attachment_denied")?)
                    .map_err(|e| e.code().to_owned())?;
            bytes = bytes
                .checked_add(selected.bytes.len())
                .ok_or("attachment_capacity")?;
            if bytes > attachments::MAX_BATCH_SOURCE {
                return Err("attachment_capacity".into());
            }
            originals.push(selected);
        }
        let mut batch = Vec::new();
        for selected in originals {
            let output = attachment_processing::process_isolated(
                selected.kind,
                &selected.bytes,
                &AtomicBool::new(false),
            )
            .map_err(|_| "attachment_denied".to_owned())?;
            batch.push(
                AdmittedAttachment::from_processed(selected, output)
                    .map_err(|e| e.code().to_owned())?,
            );
        }
        Ok::<_, String>(batch)
    })
    .await
    .map_err(|_| "attachment_unavailable".to_owned())??;
    host.idle()?;
    host.attachments
        .lock()
        .map_err(|_| "state_unavailable")?
        .append_batch(&input.conversation_id, &expected_ids, batch)
        .map_err(|e| e.code().to_owned())
}

#[tauri::command]
fn remove_attachment(
    window: WebviewWindow,
    host: State<'_, Host>,
    input: AttachmentIntent,
) -> Result<RemovedAttachment, String> {
    trusted(&window)?;
    let _guard = host.enter_native_dialog()?;
    host.idle()?;
    if !admission::opaque_id(&input.conversation_id) || !attachments::valid_id(&input.attachment_id)
    {
        return Err("attachment_denied".into());
    }
    host.attachments
        .lock()
        .map_err(|_| "state_unavailable")?
        .remove(&input.conversation_id, &input.attachment_id)
        .map_err(|e| e.code().to_owned())?;
    Ok(RemovedAttachment { removed: true })
}

#[tauri::command]
async fn review_processed_attachment(
    window: WebviewWindow,
    host: State<'_, Host>,
    input: ProcessedIntent,
) -> Result<ViewedAttachment, String> {
    trusted(&window)?;
    let guard = host.enter_native_dialog()?;
    host.idle()?;
    if !admission::opaque_id(&input.conversation_id)
        || !admission::opaque_id(&input.request_id)
        || !attachments::valid_id(&input.attachment_id)
    {
        return Err("processed_attachment_denied".into());
    }
    let stored = {
        let mut state = host.persistent.lock().map_err(|_| "state_unavailable")?;
        let p = persistent(&mut state)?;
        let (binding, class) = p
            .history
            .processed_reference(
                &input.conversation_id,
                &input.request_id,
                &input.attachment_id,
            )
            .map_err(code)?;
        p.processed
            .as_ref()
            .map_err(|e| code(*e))?
            .load_bound(&input.conversation_id, &input.request_id, &binding, &class)
            .map_err(code)?
    };
    let hwnd = owner(&window)?;
    let worker_lease = guard.worker_lease();
    let viewed = tauri::async_runtime::spawn_blocking(move || {
        let _worker_lease = worker_lease;
        let content = stored.materialise(&AtomicBool::new(false)).map_err(code)?;
        let pages = visual_pages(&content, &stored.metadata.label)?;
        if pages.is_empty() {
            return Err("processed_attachment_denied".into());
        }
        native_visual::inspect_owned(
            &pages,
            hwnd,
            std::time::Instant::now() + std::time::Duration::from_secs(300),
            false,
        )
        .map_err(str::to_owned)
    })
    .await
    .map_err(|_| "processed_attachment_denied".to_owned())??;
    Ok(ViewedAttachment { viewed })
}

#[tauri::command]
async fn send_message(
    window: WebviewWindow,
    app: tauri::AppHandle,
    host: State<'_, Host>,
    input: SendIntent,
) -> Result<Accepted, String> {
    send_message_for_origin(window, app, host, input, SendOrigin::Conversation).await
}

fn select_send_attachments(
    origin: SendOrigin,
    ids: &[String],
    consume_chat: impl FnOnce() -> Result<Vec<AdmittedAttachment>, String>,
) -> Result<Vec<AdmittedAttachment>, String> {
    match origin {
        SendOrigin::Conversation => consume_chat(),
        SendOrigin::InertPackageGeneration if ids.is_empty() => Ok(Vec::new()),
        SendOrigin::InertPackageGeneration => Err("generation_input_denied".into()),
    }
}

async fn send_message_for_origin(
    window: WebviewWindow,
    app: tauri::AppHandle,
    host: State<'_, Host>,
    input: SendIntent,
    origin: SendOrigin,
) -> Result<Accepted, String> {
    trusted(&window)?;
    if !admission::opaque_id(&input.conversation_id) || !admission::opaque_id(&input.request_id) {
        return Err("invalid_input".into());
    }
    let model = Model::parse(&input.model_id).ok_or("model_denied")?;
    let class = admission::validate_new_message(&input.data_class, &input.message)
        .map_err(str::to_owned)?;
    let effort = ReasoningEffort::parse(&input.reasoning_effort).ok_or("reasoning_denied")?;
    let guard = host.enter_native_dialog()?;
    host.idle()?;
    require_valid_credential(&host)?;
    let files = select_send_attachments(origin, &input.attachment_ids, || {
        host.attachments
            .lock()
            .map_err(|_| "state_unavailable")?
            .consume(
                &input.conversation_id,
                &input.request_id,
                &input.attachment_ids,
            )
            .map_err(|e| e.code().to_owned())
    })?;
    if files.iter().any(|file| !file.still_valid()) {
        return Err("attachment_expired".into());
    }
    let complete_prompt = attachment_prompt(&input.message, &files)?;
    admission::validate_new_message(&input.data_class, &complete_prompt).map_err(str::to_owned)?;
    let admitted = {
        let mut state = host.persistent.lock().map_err(|_| "state_unavailable")?;
        let p = persistent(&mut state)?;
        checked_conversation(p, &input.conversation_id)?;
        if !files.is_empty() {
            p.processed.as_ref().map_err(|e| code(*e))?;
        }
        if !p.contributor.status(now()?).accepted {
            return Err("contributor_terms_required".into());
        }
        if !p.budget.available(now()?) {
            return Err("budget_attention_required".into());
        }
        if !p.history.can_append_request().map_err(code)? {
            return Err("private_store_capacity".into());
        }
        let pairs = p
            .history
            .eligible_pairs(&input.conversation_id)
            .map_err(code)?;
        if pairs
            .iter()
            .any(|p| admission::DataClass::parse(&p.data_class).is_none())
        {
            return Err("data_admission_denied".into());
        }
        let turns = pairs
            .into_iter()
            .map(|p| (p.prompt, p.answer))
            .collect::<Vec<_>>();
        let mut sessions = host.sessions.lock().map_err(|_| "state_unavailable")?;
        sessions
            .restore_completed(&input.conversation_id, &turns)
            .map_err(|e| e.code())?;
        sessions
            .reserve(
                &input.conversation_id,
                &input.request_id,
                model,
                complete_prompt.clone(),
            )
            .map_err(|e| e.code())?
    };
    let mut recorded = false;
    let mut reserved = false;
    let prepared=async {
        admission::validate_context(&admitted.messages).map_err(str::to_owned)?;
        let mut media = Vec::new(); let mut previews = Vec::new();
        for file in &files {
            if !file.still_valid() { return Err("attachment_expired".into()); }
            match &file.processed {
                ProcessedContent::Text(_) => (),
                ProcessedContent::Pages(pages) if file.metadata.kind == AttachmentKind::Pdf => {
                    media.push(provider::InlineMedia::Pdf(&file.original));
                    previews.extend(visual_pages(&file.processed,&file.metadata.label)?);
                    if pages.len() != file.metadata.page_count { return Err("attachment_conflict".into()); }
                }
                ProcessedContent::Pages(pages) => {
                    let page = pages.first().filter(|_|pages.len()==1).ok_or("attachment_conflict")?;
                    media.push(provider::InlineMedia::Png(&page.png));
                    previews.extend(visual_pages(&file.processed,&file.metadata.label)?);
                }
            }
        }
        let body=provider::exact_body_with_media(model,&admitted.messages,effort,&media).map_err(|e|e.code().to_owned())?;
        let sources:Vec<_> = files.iter().map(|file|serde_json::json!({"metadata":file.metadata,"sourceSha256":file.source_sha256,"egress":if file.metadata.kind==AttachmentKind::Pdf{"Exact native PDF inline"}else if file.metadata.kind==AttachmentKind::Image{"Complete normalised PNG inline"}else{"Complete processed text in newest prompt"}})).collect();
        let metadata=serde_json::json!({"protocolVersion":5,"policyVersion":POLICY_VERSION,"requestId":input.request_id,"conversationId":input.conversation_id,"dataClass":class.as_str(),"reasoningEffort":input.reasoning_effort,"model":DEFAULT_MODEL,"maximumCompletionTokens":8192,"attachments":sources,"completeOriginalAttestation":"The selected classification applies to the complete original including PDF metadata, embedded or hidden objects, not just visible pages. I have established rights and eligibility. Meta may use inputs and outputs for training. Page preview cannot prove that hidden content is absent.","nativeAuthority":"One exact body only. Original allocations expire after five minutes; no tools, file or browser authority. Processed visual history is local only; reattach for another complete native reading."}).to_string();
        let document=ReviewDocument::disclosure_with_context(&body,&metadata).map_err(|_|"native_review_unavailable".to_owned())?;
        let document=if previews.is_empty(){document}else{document.with_visuals(previews).map_err(|_|"native_review_unavailable".to_owned())?};
        let document=if let Some(deadline)=files.iter().map(AdmittedAttachment::deadline).min(){document.with_source_deadline(deadline)}else{document};
        let document=document.with_request_cancellation(admitted.cancellation.clone());
        if !review(&window,document,&guard).await?{return Err("disclosure_denied".into());}
        if files.iter().any(|file|!file.still_valid()){return Err("attachment_expired".into());}
        if *admitted.cancellation.borrow(){return Err("cancelled".into());}
        let mut state=host.persistent.lock().map_err(|_|"state_unavailable")?;let p=persistent(&mut state)?;
        if !p.contributor.status(now()?).accepted{return Err("contributor_terms_required".into());}
        p.budget.reserve(&input.request_id,now()?).map_err(code)?;reserved=true;
        let bindings=if files.is_empty(){Vec::new()}else{p.processed.as_mut().map_err(|e|code(*e))?.append_batch(&input.conversation_id,&input.request_id,class.as_str(),&files).map_err(code)?};
        p.history.begin_request_with_attachments(&input.conversation_id,&input.request_id,&complete_prompt,class.as_str(),now()?,bindings).map_err(code)?;recorded=true;
        let remaining=p.history.remaining_text(&input.conversation_id).map_err(code)?;
        // Reading only the native key occurs after review; a proven unsent
        // failure can release. A durable dispatch intent precedes networking.
        let key=host.credentials.read().map_err(|e|e.code().to_owned())?;
        host.credential_validation.lock().map_err(|_|"state_unavailable")?.as_ref().map_err(|e|code(*e))?.require(&key).map_err(str::to_owned)?;
        if *admitted.cancellation.borrow(){return Err("cancelled".into());}
        let dispatch_time=now()?;
        if files.iter().any(|file|!file.still_valid()){return Err("attachment_expired".into());}
        if !p.contributor.status(dispatch_time).accepted{return Err("contributor_terms_required".into());}
        p.budget.mark_dispatch(&input.request_id,dispatch_time).map_err(code)?;
        Ok::<_,String>((body,key,remaining))
    }.await;
    let (body, key, remaining) = match prepared {
        Ok(value) => value,
        Err(mut error) => {
            // Slot cleanup never depends on a working disk or clock.
            let disposition = if let Ok(mut sessions) = host.sessions.lock() {
                sessions.finish(
                    &input.conversation_id,
                    &input.request_id,
                    if error == "cancelled" {
                        TransportOutcome::Cancelled
                    } else {
                        TransportOutcome::Failed
                    },
                )
            } else {
                None
            };
            let terminal = preparation_terminal(&error, disposition);
            if terminal == EntryState::Cancelled {
                error = "cancelled".into();
            }
            if let (Ok(mut state), Ok(time)) = (host.persistent.lock(), now()) {
                if let Ok(p) = persistent(&mut state) {
                    if reserved {
                        let _ = p.budget.release_unsent(&input.request_id, time);
                    }
                    if recorded {
                        let _ = p.history.finish_request(
                            &input.conversation_id,
                            &input.request_id,
                            "",
                            terminal,
                            false,
                            time,
                        );
                    }
                }
            }
            // Closing remains excluded until both slot and existing durable
            // error cleanup finish. Zero active slots alone is not quiescence.
            drop(guard);
            return Err(error);
        }
    };
    host.sessions
        .lock()
        .map_err(|_| "state_unavailable")?
        .mark_streaming(&input.conversation_id, &input.request_id);
    // The active slot now independently excludes close for the entire stream
    // and final persistent history/financial completion.
    drop(guard);
    let receipt = Accepted {
        request_id: input.request_id.clone(),
        accepted: true,
    };
    tauri::async_runtime::spawn(async move {
        let mut visible = String::new();
        let on_content = |text: String| {
            if visible.len().saturating_add(text.len()) > remaining {
                return Err(ProviderFailure::ResponseLimit);
            }
            visible.push_str(&text);
            app.emit_to(
                MAIN_WINDOW,
                STREAM_EVENT,
                StreamEvent {
                    request_id: &input.request_id,
                    conversation_id: &input.conversation_id,
                    kind: "content",
                    text: Some(&text),
                    status: None,
                    error_code: None,
                },
            )
            .map_err(|_| ProviderFailure::InvalidContract)
        };
        let outcome = AssertUnwindSafe(provider::stream_chat_exact(
            model,
            body,
            key,
            admitted.cancellation,
            on_content,
        ))
        .catch_unwind()
        .await
        .unwrap_or(ProviderOutcome {
            answer: String::new(),
            terminal: ProviderTerminal::Failed,
            failure: Some(ProviderFailure::InvalidContract),
            usage: None,
            dispatched: true,
        });
        // Keep bounded text actually delivered even on a transport unwind.
        let answer = visible;
        let host = app.state::<Host>();
        if outcome.failure == Some(ProviderFailure::Authentication) {
            if let Ok(mut proof) = host.credential_validation.lock() {
                if let Ok(proof) = proof.as_mut() {
                    let _ = proof.revoke();
                }
            }
        }
        let mut error = outcome.failure.map(|e| e.code().to_owned());
        let mut terminal = match outcome.terminal {
            ProviderTerminal::Completed => EntryState::Completed,
            ProviderTerminal::Truncated => EntryState::Truncated,
            ProviderTerminal::Refused => EntryState::Refused,
            ProviderTerminal::Cancelled => EntryState::Cancelled,
            ProviderTerminal::Failed => EntryState::Failed,
        };
        if let Ok(mut state) = host.persistent.lock() {
            if let Ok(p) = persistent(&mut state) {
                if !outcome.dispatched {
                    if p.budget
                        .release_proven_unsent(&input.request_id, now().unwrap_or(0))
                        .is_err()
                    {
                        error = Some("budget_attention_required".into());
                    }
                } else if let Some(usage) = outcome.usage {
                    if p.budget
                        .settle(&input.request_id, usage, now().unwrap_or(0))
                        .is_err()
                    {
                        error = Some("budget_attention_required".into());
                    }
                }
                // An emitted cancel wins before eligible history is committed.
                let transport = if terminal == EntryState::Completed {
                    TransportOutcome::Completed(answer.clone())
                } else if terminal == EntryState::Cancelled {
                    TransportOutcome::Cancelled
                } else {
                    TransportOutcome::Failed
                };
                let disposition = host.sessions.lock().ok().and_then(|mut s| {
                    s.finish(&input.conversation_id, &input.request_id, transport)
                });
                if disposition == Some(FinishDisposition::Cancelled) {
                    terminal = EntryState::Cancelled;
                } else if disposition == Some(FinishDisposition::Failed)
                    && terminal == EntryState::Completed
                {
                    terminal = EntryState::Failed;
                }
                if disposition.is_none() {
                    terminal = EntryState::Failed;
                    error = Some("state_unavailable".into());
                }
                let eligible =
                    terminal == EntryState::Completed && admission::admissible_text(&answer);
                if p.history
                    .finish_request(
                        &input.conversation_id,
                        &input.request_id,
                        &answer,
                        terminal,
                        eligible,
                        now().unwrap_or(0),
                    )
                    .is_err()
                {
                    terminal = EntryState::Failed;
                    error = Some("history_attention_required".into());
                }
            } else {
                terminal = EntryState::Failed;
                error = Some("private_store_unavailable".into());
            }
        } else {
            terminal = EntryState::Failed;
            error = Some("state_unavailable".into());
        }
        // Idempotent fallback only removes a stranded runtime slot; it does
        // not refund, fabricate usage or retry uncertain persistence.
        if let Ok(mut s) = host.sessions.lock() {
            s.finish(
                &input.conversation_id,
                &input.request_id,
                TransportOutcome::Failed,
            );
        }
        let status = match terminal {
            EntryState::Completed => "completed",
            EntryState::Truncated => "truncated",
            EntryState::Refused => "refused",
            EntryState::Cancelled => "cancelled",
            _ => "failed",
        };
        let _ = app.emit_to(
            MAIN_WINDOW,
            STREAM_EVENT,
            StreamEvent {
                request_id: &input.request_id,
                conversation_id: &input.conversation_id,
                kind: "terminal",
                text: None,
                status: Some(status),
                error_code: error.as_deref(),
            },
        );
    });
    Ok(receipt)
}
#[tauri::command]
fn cancel_message(
    window: WebviewWindow,
    host: State<'_, Host>,
    request_id: String,
) -> Result<Cancelled, String> {
    trusted(&window)?;
    if !admission::opaque_id(&request_id) {
        return Err("invalid_input".into());
    }
    Ok(Cancelled {
        cancelled: host
            .sessions
            .lock()
            .map_err(|_| "state_unavailable")?
            .cancel(&request_id),
    })
}

fn preparation_terminal(error: &str, disposition: Option<FinishDisposition>) -> EntryState {
    if error == "cancelled" || disposition == Some(FinishDisposition::Cancelled) {
        EntryState::Cancelled
    } else {
        EntryState::Failed
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum LaunchMode {
    AttachmentWorker,
    AttachmentFixture,
    AttachmentResourceWorker,
    Normal,
    ReviewFixture { large: bool },
    CredentialFixture,
    Invalid,
}
fn launch_mode(arguments: &[String]) -> LaunchMode {
    if arguments.len() == 1 && arguments[0] == attachment_worker::RESOURCE_WORKER_ARGUMENT {
        return LaunchMode::AttachmentResourceWorker;
    }
    if arguments.len() == 1 && arguments[0] == "--aster-attachment-fixture" {
        return LaunchMode::AttachmentFixture;
    }
    if arguments.len() == 1 && arguments[0] == attachment_worker::WORKER_ARGUMENT {
        return LaunchMode::AttachmentWorker;
    }
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
        if !converted.is_empty() {
            return LaunchMode::Invalid;
        }
        match argument.into_string() {
            Ok(v) => converted.push(v),
            Err(_) => return LaunchMode::Invalid,
        }
    }
    launch_mode(&converted)
}
pub fn run() {
    match launch_mode_os(std::env::args_os().skip(1)) {
        LaunchMode::Normal => (),
        LaunchMode::AttachmentResourceWorker => {
            let outcome = std::panic::catch_unwind(attachment_worker::run_resource_worker_stdio);
            std::process::exit(if matches!(outcome, Ok(Ok(()))) { 0 } else { 3 })
        }
        LaunchMode::AttachmentFixture => {
            let report = attachment_fixture::run();
            let exit = if report.all_passed { 0 } else { 3 };
            match serde_json::to_string(&report) {
                Ok(json) => println!("{json}"),
                Err(_) => std::process::exit(3),
            }
            std::process::exit(exit)
        }
        LaunchMode::AttachmentWorker => {
            let result = std::panic::catch_unwind(attachment_worker::run_worker_stdio);
            let exit = match result {
                Ok(Ok(())) => 0,
                Ok(Err(error)) => attachment_worker::worker_failure_exit(error),
                Err(_) => 3,
            };
            std::process::exit(exit as i32);
        }
        LaunchMode::Invalid => {
            eprintln!("aster_startup=invalid_arguments");
            std::process::exit(3)
        }
        LaunchMode::CredentialFixture => {
            let (classification, exit) = std::panic::catch_unwind(credential_fixture::run)
                .unwrap_or(("synthetic_gui_failed_closed", 3));
            eprintln!("aster_credential_fixture={classification}");
            std::process::exit(exit)
        }
        LaunchMode::ReviewFixture { large } => {
            let (code, exit) = match native_review::run_fixture(large) {
                Ok(true) => ("synthetic_review_confirmed_no_publication", 0),
                Ok(false) => ("synthetic_review_denied_no_publication", 2),
                Err(_) => ("synthetic_review_failed_closed_no_publication", 3),
            };
            eprintln!("aster_review_fixture={code}");
            std::process::exit(exit)
        }
    }
    tauri::Builder::default()
        .manage(render_trace::RenderTrace::new())
        .on_page_load(|webview, payload| {
            if webview.label() == MAIN_WINDOW {
                webview
                    .state::<render_trace::RenderTrace>()
                    .native(match payload.event() {
                        tauri::webview::PageLoadEvent::Started => {
                            render_trace::NativePhase::PageStarted
                        }
                        tauri::webview::PageLoadEvent::Finished => {
                            render_trace::NativePhase::PageFinished
                        }
                    });
            }
        })
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let gate =
                startup::StartupGate::new(app.state::<render_trace::RenderTrace>().inner().clone());
            let window = app
                .get_webview_window(MAIN_WINDOW)
                .ok_or("startup_window_unavailable")?;
            app.manage(gate.clone());
            app.state::<render_trace::RenderTrace>()
                .native(render_trace::NativePhase::ParentCreated);
            webview_render_observer::attach(&window);
            gate.start_monitor(window);
            // Existing local-data base only; PrivateStore creates its own fixed
            // protected child. Never reset/repair an unavailable existing store.
            let persistence = app
                .path()
                .local_data_dir()
                .map_err(|_| StoreError::Unavailable)
                .and_then(|base| Persistence::open(&base));
            let catalogue = app
                .path()
                .local_data_dir()
                .map_err(|_| StoreError::Unavailable)
                .and_then(|base| customization::Customization::open(&base));
            let mut github = github_host::GithubHost::default();
            if catalogue
                .as_ref()
                .ok()
                .and_then(|catalogue| catalogue.package(customization::BUILTIN_GITHUB_ID).ok())
                .is_some_and(|package| package.enabled)
            {
                // A persisted enabled preference restores no token, account,
                // repository, session, approval or automatic network work.
                github.set_enabled(true).map_err(std::io::Error::other)?;
            }
            app.manage(Host {
                credentials: CredentialStore,
                credential_validation: Mutex::new(
                    app.path()
                        .local_data_dir()
                        .map_err(|_| StoreError::Unavailable)
                        .and_then(|base| CredentialValidation::open(&base)),
                ),
                personalization: Mutex::new(
                    app.path()
                        .local_data_dir()
                        .map_err(|_| StoreError::Unavailable)
                        .and_then(|base| personalization::Personalization::open(&base)),
                ),
                customization: Mutex::new(catalogue),
                github: Mutex::new(github),
                sessions: Mutex::new(SessionState::default()),
                attachments: Mutex::new(AttachmentRegistry::default()),
                persistent: Mutex::new(persistence),
                native_dialog_open: NativeOperationGate::default(),
                expiry_shutdown: AtomicBool::new(false),
                window_close: window_guard::CloseState::default(),
            });
            // Independent expiry cleanup. Grants deny exactly after their
            // monotonic deadline; allocations are dropped on the next scheduled
            // one-second cleanup tick even if the renderer becomes idle.
            let expiry_app = app.handle().clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
                let Some(host) = expiry_app.try_state::<Host>() else {
                    break;
                };
                if host.expiry_shutdown.load(Ordering::Acquire) {
                    break;
                }
                if let Ok(mut registry) = host.attachments.lock() {
                    registry.prune_expired();
                };
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_status,
            configure_meta_credential,
            remove_meta_credential,
            verify_meta_credential,
            get_personalization,
            save_personalization,
            clear_personalization,
            get_customization,
            get_customization_package,
            save_customization,
            set_customization_enabled,
            import_customization,
            customization_generation::generate_customization,
            github_host::get_github_status,
            github_host::begin_github_connection,
            github_host::poll_github_connection,
            github_host::cancel_github_connection,
            github_host::disconnect_github,
            github_host::list_github_repositories,
            github_host::select_github_repository,
            github_host::read_github_file,
            render_trace::record_render_event,
            render_trace::get_render_trace,
            accept_contributor_terms,
            configure_budget,
            reconcile_budget,
            list_conversations,
            create_conversation,
            load_conversation,
            delete_conversation,
            send_message,
            cancel_message,
            window_controls::window_action,
            startup::startup_shell_ready,
            select_attachments,
            remove_attachment,
            review_processed_attachment
        ])
        .on_window_event(|window, event| {
            if window.label() == MAIN_WINDOW {
                let phase = match event {
                    tauri::WindowEvent::Resized(_) => Some(render_trace::NativePhase::Resized),
                    tauri::WindowEvent::ScaleFactorChanged { .. } => {
                        Some(render_trace::NativePhase::ScaleChanged)
                    }
                    tauri::WindowEvent::Focused(true) => Some(render_trace::NativePhase::Focused),
                    tauri::WindowEvent::Focused(false) => {
                        Some(render_trace::NativePhase::Unfocused)
                    }
                    tauri::WindowEvent::Destroyed => Some(render_trace::NativePhase::Destroyed),
                    _ => None,
                };
                if let Some(phase) = phase {
                    window.state::<render_trace::RenderTrace>().native(phase);
                    let destroying = matches!(event, tauri::WindowEvent::Destroyed);
                    let _ = window.run_on_main_thread(move || {
                        if destroying {
                            webview_render_observer::detach_on_ui_thread();
                        } else {
                            webview_render_observer::sample_on_ui_thread();
                        }
                    });
                }
            }
            window_controls::on_window_event(window, event);
        })
        .run(tauri::generate_context!())
        .expect("Aster Desktop host failed to start");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn acknowledged_cancel_wins_preparation_error() {
        let mut sessions = SessionState::default();
        let conversation = "a".repeat(32);
        let request = "b".repeat(32);
        sessions
            .reserve(
                &conversation,
                &request,
                Model::MuseSpark,
                "Synthetic example".into(),
            )
            .unwrap();
        assert!(sessions.cancel(&request));
        let disposition = sessions.finish(&conversation, &request, TransportOutcome::Failed);
        assert_eq!(
            preparation_terminal("credential_error", disposition),
            EntryState::Cancelled
        );
        assert_eq!(
            preparation_terminal("credential_error", Some(FinishDisposition::Failed)),
            EntryState::Failed
        );
        assert_eq!(
            preparation_terminal("cancelled", None),
            EntryState::Cancelled
        );
    }
    #[test]
    fn protocol_four_intent_is_closed() {
        let base = serde_json::json!({"conversationId":"a".repeat(32),"requestId":"b".repeat(32),"modelId":DEFAULT_MODEL,"message":"Synthetic example","dataClass":"synthetic","reasoningEffort":"medium"});
        assert!(serde_json::from_value::<SendIntent>(base.clone()).is_ok());
        for name in [
            "url",
            "apiKey",
            "approved",
            "projectDisclosure",
            "exactBody",
            "ownerHwnd",
            "tools",
        ] {
            let mut v = base.clone();
            v[name] = serde_json::json!("forged");
            assert!(serde_json::from_value::<SendIntent>(v).is_err());
        }
    }
    #[test]
    fn manifest_exposes_only_closed_meta_profile() {
        let manifest = include_str!("../build.rs");
        let cap = include_str!("../capabilities/main.json");
        for c in COMMANDS {
            assert!(manifest.contains(&format!("\"{c}\"")));
            assert!(cap.contains(&format!("allow-{}", c.replace('_', "-"))));
        }
        for retired in [
            "configure_nvidia_credential",
            "select_project_root",
            "send_project_message",
            "review_project_proposal",
        ] {
            assert!(!manifest.contains(retired));
            assert!(!cap.contains(&retired.replace('_', "-")));
        }
        for permission in [
            "shell:",
            "fs:",
            "http:",
            "core:event:allow-emit",
            "core:window:",
            "core:app:allow-exit",
            "\"windows\": [\"*\"]",
        ] {
            assert!(!cap.contains(permission));
        }
    }

    #[test]
    fn native_guard_retained_through_idle_preparation_cleanup_excludes_close() {
        let host = Host {
            credentials: CredentialStore,
            credential_validation: Mutex::new(Err(StoreError::Unavailable)),
            personalization: Mutex::new(Err(StoreError::Unavailable)),
            customization: Mutex::new(Err(StoreError::Unavailable)),
            github: Mutex::new(github_host::GithubHost::default()),
            sessions: Mutex::new(SessionState::default()),
            attachments: Mutex::new(AttachmentRegistry::default()),
            persistent: Mutex::new(Err(StoreError::Unavailable)),
            native_dialog_open: NativeOperationGate::default(),
            expiry_shutdown: AtomicBool::new(false),
            window_close: window_guard::CloseState::default(),
        };
        let guard = host.enter_native_dialog().unwrap();
        let conversation = "a".repeat(32);
        let request = "b".repeat(32);
        host.sessions
            .lock()
            .unwrap()
            .reserve(
                &conversation,
                &request,
                Model::MuseSpark,
                "Synthetic example".into(),
            )
            .unwrap();
        host.sessions
            .lock()
            .unwrap()
            .finish(&conversation, &request, TransportOutcome::Failed);
        assert_eq!(host.sessions.lock().unwrap().active_count(), 0);
        // A close command uses this same admission gate, including the interval
        // after slot removal but before durable preparation-error cleanup ends.
        assert!(matches!(host.enter_native_dialog(), Err(error) if error == "native_dialog_busy"));
        drop(guard);
        assert!(host.enter_native_dialog().is_ok());
        host.window_close.arm().unwrap();
        assert_eq!(host.idle(), Err("window_closing".into()));
        assert!(matches!(host.enter_native_dialog(), Err(error) if error == "window_closing"));
    }
    #[test]
    fn closed_launch_selector_denies_unknown_and_extra_arguments() {
        assert_eq!(launch_mode(&[]), LaunchMode::Normal);
        assert_eq!(
            launch_mode(&[credential_fixture::FLAG.into()]),
            LaunchMode::CredentialFixture
        );
        assert_eq!(
            launch_mode(&["--url=https://example.com".into()]),
            LaunchMode::Invalid
        );
        assert_eq!(
            launch_mode(&[credential_fixture::FLAG.into(), "extra".into()]),
            LaunchMode::Invalid
        );
    }
}
