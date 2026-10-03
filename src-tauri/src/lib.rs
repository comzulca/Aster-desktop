//! Protocol 3 host: a fixed Contributor transport, durable private history and
//! bounded finance. WebView messages are intent, never authority or secrets.
mod admission;
mod budget;
mod contributor;
#[cfg(all(test, windows))]
mod creation_process_fixture;
mod credential;
mod credential_fixture;
mod history;
#[cfg(windows)]
mod journal_privacy;
mod native_review;
#[cfg(windows)]
mod new_file; // Shared file identity types and frozen recovery, no IPC routes.
mod policy;
mod private_store;
mod provider;
mod window_controls;
mod window_guard;
#[allow(dead_code)]
mod workspace; // Shared privacy error types/frozen recovery, no v0.2 capability.

use budget::{Budget, BudgetStatus, Configuration};
use contributor::{Contributor, ContributorStatus};
use credential::CredentialStore;
use futures_util::FutureExt;
use history::{Conversation, ConversationSummary, EntryState, History};
use native_review::ReviewDocument;
use policy::{
    FinishDisposition, Model, SessionState, TransportOutcome, DEFAULT_MODEL, POLICY_VERSION,
};
use private_store::StoreError;
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

const MAIN_WINDOW: &str = "main";
const STREAM_EVENT: &str = "aster://stream";
#[cfg(test)]
const COMMANDS: &[&str] = &[
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
];

struct Persistence {
    history: History,
    budget: Budget,
    contributor: Contributor,
}
impl Persistence {
    fn open(base: &Path) -> Result<Self, StoreError> {
        Ok(Self {
            history: History::open(base)?,
            budget: Budget::open(base)?,
            contributor: Contributor::open(base)?,
        })
    }
}
struct Host {
    credentials: CredentialStore,
    sessions: Mutex<SessionState>,
    persistent: Mutex<Result<Persistence, StoreError>>,
    native_dialog_open: AtomicBool,
    window_close: window_guard::CloseState,
}
struct NativeDialogGuard<'a>(&'a AtomicBool);
impl Drop for NativeDialogGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}
impl Host {
    fn enter_native_dialog(&self) -> Result<NativeDialogGuard<'_>, String> {
        if self.window_close.is_closing() {
            return Err("window_closing".into());
        }
        let guard = self
            .native_dialog_open
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| NativeDialogGuard(&self.native_dialog_open))
            .map_err(|_| "native_dialog_busy".to_owned())?;
        if self.window_close.is_closing() {
            return Err("window_closing".into());
        }
        Ok(guard)
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
async fn review(window: &WebviewWindow, document: ReviewDocument) -> Result<bool, String> {
    let hwnd = owner(window)?;
    tauri::async_runtime::spawn_blocking(move || {
        match native_review::confirm_owned(&document, hwnd)
            .map_err(|_| "native_review_unavailable".to_owned())?
        {
            Some(approval) => {
                approval
                    .consume(document.binding_sha256())
                    .map_err(|_| "native_review_expired".to_owned())?;
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
    let writable = p.history.can_append_request().unwrap_or(false) && p.budget.can_append_request();
    let availability = if !cfg!(windows) {
        "unsupported"
    } else if !history_available || !credential_available || !writable {
        "attention_required"
    } else if active {
        "busy"
    } else if !contributor.accepted {
        "needs_terms"
    } else if !configured {
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
        protocol_version: 3,
        policy_version: POLICY_VERSION,
        configured,
        credential_available,
        default_model: DEFAULT_MODEL,
        models: [ModelDisplay {
            id: DEFAULT_MODEL,
            label: "Muse Spark 1.3 Contributor",
        }],
        active_request: active,
        project_selected: false,
        project_label: None,
        capabilities: Capabilities {
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
    let _guard = host.enter_native_dialog()?;
    host.idle()?;
    // No key is read or sent to the WebView; saved history and finances survive.
    let hwnd = owner(&window)?;
    let configured = tauri::async_runtime::spawn_blocking(move || {
        CredentialStore
            .enter_with_native_dialog_after(hwnd, || {
                invalidate_identity(&app.state::<Host>())
                    .map_err(|_| credential::CredentialError::Rejected)
            })
            .map_err(|e| e.code().to_owned())
    })
    .await
    .map_err(|_| "credential_unavailable")??;
    Ok(CredentialStatus { configured })
}
#[tauri::command]
async fn remove_meta_credential(
    window: WebviewWindow,
    host: State<'_, Host>,
) -> Result<CredentialStatus, String> {
    trusted(&window)?;
    let _guard = host.enter_native_dialog()?;
    host.idle()?;
    if !review(&window,operation(serde_json::json!({"operation":"Remove Meta API credential","target":"AsterDesktop/MetaModelAPI/v1","preserved":"Earlier provider credentials, protected conversation display history and financial reservations remain."}))?).await? {return Ok(CredentialStatus{configured:host.credentials.configured().map_err(|e|e.code())?});}
    invalidate_identity(&host)?;
    host.credentials.delete().map_err(|e| e.code())?;
    Ok(CredentialStatus { configured: false })
}
#[tauri::command]
async fn accept_contributor_terms(
    window: WebviewWindow,
    host: State<'_, Host>,
) -> Result<TermsReceipt, String> {
    trusted(&window)?;
    let _guard = host.enter_native_dialog()?;
    host.idle()?;
    let document = operation(
        serde_json::json!({"operation":"Contributor owner attestation","termsRevision":contributor::TERMS_REVISION,
        "model":DEFAULT_MODEL,"destination":provider::ENDPOINT,"disclosure":"Meta Contributor may use prompts and answers for training. Personal, sensitive and confidential content, including private source code, must not be sent. Every complete request and prior answer needs fresh native review.",
        "ownerAttestation":"I have checked that my account, territory and intended use are eligible for this direct Meta API. I use a direct API key, not a coding-harness subscription key. Aster has not verified account entitlement.",
        "spending":"Cancelling, refusing or failing after dispatch does not establish zero cost. Missing usage requires statement-confirmed reconciliation. The £89 local limit does not cap other clients or guarantee Meta's invoice.",
        "terms":"https://dev.meta.ai/legal/terms-of-service","geography":"https://dev.meta.ai/legal/geographic-use-policy","expiry":"2026-11-02T00:00:00Z"}),
    )?;
    if !review(&window, document).await? {
        return Ok(TermsReceipt { accepted: false });
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
    let _guard = host.enter_native_dialog()?;
    host.idle()?;
    let time = now()?;
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
        "period":budget::period(time).map_err(code)?,"statement":"I have checked the actual date and London calendar month. This explicitly activates the displayed period; changing the account alias does not reset charges. Earlier uncertain reservations remain. The declared GBP/USD rate, positive margin, tax and payment costs conservatively bound my invoice until the stated expiry. No exchange rate or tax exemption is verified by Aster.","limitNanoGbp":budget::LIMIT.to_string(),"noReset":true}),
    )?;
    if !review(&window, document).await? {
        return Ok(BudgetReceipt { configured: false });
    }
    let mut state = host.persistent.lock().map_err(|_| "state_unavailable")?;
    let p = persistent(&mut state)?;
    let commit_time = now()?;
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
    let _guard = host.enter_native_dialog()?;
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
    if !review(&window, document).await? {
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
    trusted(&window)?;
    host.idle()?;
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
    host.idle()?;
    if !admission::opaque_id(&input.conversation_id) {
        return Err("invalid_input".into());
    }
    let mut state = host.persistent.lock().map_err(|_| "state_unavailable")?;
    host.idle()?;
    persistent(&mut state)?
        .history
        .load(&input.conversation_id)
        .map_err(code)
}
#[tauri::command]
async fn delete_conversation(
    window: WebviewWindow,
    host: State<'_, Host>,
    input: ConversationIntent,
) -> Result<DeleteReceipt, String> {
    trusted(&window)?;
    let _guard = host.enter_native_dialog()?;
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
    if !review(&window,operation(serde_json::json!({"operation":"Remove conversation from visible history","conversationId":conversation.id,"entryCount":conversation.entries.len(),"effect":"Logical removal only. Encrypted recovery records remain and storage quota is not reclaimed. This is not permanent or secure erasure. Financial records and reservations are preserved."}))?).await?{return Ok(DeleteReceipt{deleted:false});}
    let mut state = host.persistent.lock().map_err(|_| "state_unavailable")?;
    persistent(&mut state)?
        .history
        .remove_after_native_confirmation(&input.conversation_id, now()?)
        .map_err(code)?;
    Ok(DeleteReceipt { deleted: true })
}

#[tauri::command]
async fn send_message(
    window: WebviewWindow,
    app: tauri::AppHandle,
    host: State<'_, Host>,
    input: SendIntent,
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
    let admitted = {
        let mut state = host.persistent.lock().map_err(|_| "state_unavailable")?;
        let p = persistent(&mut state)?;
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
                input.message.clone(),
            )
            .map_err(|e| e.code())?
    };
    let mut recorded = false;
    let mut reserved = false;
    let prepared=async {
        admission::validate_context(&admitted.messages).map_err(str::to_owned)?;
        let body=provider::exact_body_with_effort(model,&admitted.messages,effort).map_err(|e|e.code().to_owned())?;
        let metadata=serde_json::json!({"protocolVersion":3,"policyVersion":POLICY_VERSION,"requestId":input.request_id,"conversationId":input.conversation_id,"dataClass":class.as_str(),"reasoningEffort":input.reasoning_effort,"model":DEFAULT_MODEL,"maximumCompletionTokens":8192,"nativeAuthority":"One exact body only; no tools, file or browser authority."}).to_string();
        let document=ReviewDocument::disclosure_with_context(&body,&metadata).map_err(|_|"native_review_unavailable".to_owned())?;
        if !review(&window,document).await?{return Err("disclosure_denied".into());}
        if *admitted.cancellation.borrow(){return Err("cancelled".into());}
        let mut state=host.persistent.lock().map_err(|_|"state_unavailable")?;let p=persistent(&mut state)?;
        if !p.contributor.status(now()?).accepted{return Err("contributor_terms_required".into());}
        p.budget.reserve(&input.request_id,now()?).map_err(code)?;reserved=true;
        p.history.begin_request(&input.conversation_id,&input.request_id,&input.message,class.as_str(),now()?).map_err(code)?;recorded=true;
        let remaining=p.history.remaining_text(&input.conversation_id).map_err(code)?;
        // Reading only the native key occurs after review; a proven unsent
        // failure can release. A durable dispatch intent precedes networking.
        let key=host.credentials.read().map_err(|e|e.code().to_owned())?;
        if *admitted.cancellation.borrow(){return Err("cancelled".into());}
        let dispatch_time=now()?;
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
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Existing local-data base only; PrivateStore creates its own fixed
            // protected child. Never reset/repair an unavailable existing store.
            let persistence = app
                .path()
                .local_data_dir()
                .map_err(|_| StoreError::Unavailable)
                .and_then(|base| Persistence::open(&base));
            app.manage(Host {
                credentials: CredentialStore,
                sessions: Mutex::new(SessionState::default()),
                persistent: Mutex::new(persistence),
                native_dialog_open: AtomicBool::new(false),
                window_close: window_guard::CloseState::default(),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_status,
            configure_meta_credential,
            remove_meta_credential,
            accept_contributor_terms,
            configure_budget,
            reconcile_budget,
            list_conversations,
            create_conversation,
            load_conversation,
            delete_conversation,
            send_message,
            cancel_message,
            window_controls::window_action
        ])
        .on_window_event(window_controls::on_window_event)
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
    fn protocol_three_intent_is_closed() {
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
            sessions: Mutex::new(SessionState::default()),
            persistent: Mutex::new(Err(StoreError::Unavailable)),
            native_dialog_open: AtomicBool::new(false),
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
