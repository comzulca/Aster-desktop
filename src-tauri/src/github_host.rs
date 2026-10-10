//! Closed GitHub commands, native review and epoch-bound device/read ownership.
//! No model tool, writable route, generic endpoint, credential DTO or automatic
//! request is registered here. Constructor/status are purely local.

use crate::github_api::{
    AccountIdentity, GithubApi, InstallationRecord, RepositoryRecord, TokenPoll,
};
use crate::github_connection::{
    ConnectionState, DeviceSession, DeviceTerminal, PollDecision, PollOutcome, RepositoryIdentity,
    VERIFICATION_URI,
};
use crate::github_credential::{GithubCredentialStore, StoredConnection};
use crate::{operation, review, trusted, Host};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::time::{Duration, Instant};
use tauri::{State, WebviewWindow};
use tokio::sync::watch;

const OPERATION_SECONDS: u64 = 120;
const MAX_REPOSITORIES: usize = 1_000;

pub(crate) struct GithubHost {
    connection: ConnectionState,
    started: Instant,
    enabled: bool,
    session: Option<DeviceSession>,
    display_code: Option<String>,
    in_flight: bool,
    stored: Option<StoredConnection>,
    account: Option<AccountIdentity>,
    selected: Option<(InstallationRecord, RepositoryRecord)>,
    repositories: Vec<(InstallationRecord, RepositoryRecord)>,
    cancellation: watch::Sender<bool>,
    reconnect_required: bool,
}

impl Default for GithubHost {
    fn default() -> Self {
        let (cancellation, _) = watch::channel(false);
        Self {
            connection: ConnectionState::default(),
            started: Instant::now(),
            enabled: false,
            session: None,
            display_code: None,
            in_flight: false,
            stored: None,
            account: None,
            selected: None,
            repositories: Vec::new(),
            cancellation,
            reconnect_required: false,
        }
    }
}

impl GithubHost {
    pub(crate) fn set_enabled(&mut self, enabled: bool) -> Result<(), String> {
        self.invalidate_pending();
        self.connection
            .set_enabled(enabled)
            .map_err(|_| "github_state_unavailable")?;
        self.enabled = enabled;
        self.reconnect_required = false;
        Ok(())
    }

    pub(crate) fn cancel(&mut self) -> Result<(), String> {
        self.invalidate_pending();
        self.connection
            .set_enabled(self.enabled)
            .map_err(|_| "github_state_unavailable")?;
        Ok(())
    }

    fn invalidate_pending(&mut self) {
        let _ = self.cancellation.send(true);
        let (cancellation, _) = watch::channel(false);
        self.cancellation = cancellation;
        if let Some(session) = self.session.as_mut() {
            session.cancel();
        }
        self.session = None;
        self.display_code = None;
        self.in_flight = false;
        self.stored = None;
        self.account = None;
        self.selected = None;
        self.repositories.clear();
    }

    fn check_epoch(&self, epoch: u64) -> Result<(), String> {
        if !self.enabled {
            return Err("github_disabled".into());
        }
        if self.connection.generation() != epoch {
            return Err("github_stale_generation".into());
        }
        Ok(())
    }

    fn status(&self) -> GithubStatus {
        let state = if !self.enabled {
            "disabled"
        } else if self.reconnect_required {
            "reconnect_required"
        } else if self.session.is_some() || self.display_code.is_some() {
            "authorising"
        } else if self.account.is_some() && (self.stored.is_some() || self.in_flight) {
            "connected"
        } else {
            "disconnected"
        };
        GithubStatus {
            enabled: self.enabled,
            state,
            account: (state == "connected")
                .then(|| self.account.clone())
                .flatten(),
            repository: (state == "connected")
                .then(|| self.selected.as_ref().map(|(_, r)| r.identity.clone()))
                .flatten(),
            user_code: self.display_code.clone(),
            verification_uri: VERIFICATION_URI,
            next_poll_seconds: self
                .session
                .as_ref()
                .map_or(0, |s| s.next_poll_seconds(self.started.elapsed().as_secs())),
            in_flight: self.in_flight,
            mutations_available: false,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GithubStatus {
    enabled: bool,
    state: &'static str,
    account: Option<AccountIdentity>,
    repository: Option<RepositoryIdentity>,
    user_code: Option<String>,
    verification_uri: &'static str,
    next_poll_seconds: u64,
    in_flight: bool,
    mutations_available: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SelectRepositoryInput {
    installation_id: u64,
    repository_id: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ReadFileInput {
    path: String,
    branch: String,
}

fn lock(host: &Host) -> Result<std::sync::MutexGuard<'_, GithubHost>, String> {
    host.github
        .lock()
        .map_err(|_| "github_state_unavailable".into())
}
fn api_error(error: crate::github_api::ApiError) -> String {
    error.code().into()
}

async fn cancellable<T, F>(cancellation: &watch::Receiver<bool>, future: F) -> Result<T, String>
where
    F: Future<Output = Result<T, String>>,
{
    if *cancellation.borrow() || cancellation.has_changed().is_err() {
        return Err("github_cancelled".into());
    }
    let mut receiver = cancellation.clone();
    tokio::select! {
        biased;
        _ = receiver.changed() => Err("github_cancelled".into()),
        result = future => result,
    }
}

/// Any early error/unwind with a taken native session/token becomes an unbound
/// reconnect state. Never leave an in-flight UI state or reuse uncertain work.
/// It acquires no mutex while waiting; successful paths clear in_flight first.
struct InflightLease<'a> {
    host: &'a Host,
    epoch: u64,
}
impl Drop for InflightLease<'_> {
    fn drop(&mut self) {
        if let Ok(mut state) = self.host.github.lock() {
            if state.connection.generation() == self.epoch && state.in_flight {
                let _ = state.cancel();
                state.reconnect_required = true;
            }
        }
    }
}

#[tauri::command]
pub(crate) fn get_github_status(
    window: WebviewWindow,
    host: State<'_, Host>,
) -> Result<GithubStatus, String> {
    trusted(&window)?;
    Ok(lock(&host)?.status())
}

#[tauri::command]
pub(crate) async fn begin_github_connection(
    window: WebviewWindow,
    host: State<'_, Host>,
) -> Result<GithubStatus, String> {
    trusted(&window)?;
    let guard = host.enter_native_dialog()?;
    host.idle()?;
    let (epoch, cancellation) = {
        let mut state = lock(&host)?;
        if !state.enabled {
            return Err("github_disabled".into());
        }
        state.invalidate_pending();
        let epoch = state
            .connection
            .begin_account_connection()
            .map_err(|_| "github_inactive")?;
        (epoch, state.cancellation.subscribe())
    };
    let document = operation(serde_json::json!({
        "operation":"Connect the dedicated Aster GitHub App",
        "app":"Aster Desktop Engineering", "appId":crate::github_api::REGISTERED_APP_ID,
        "clientId":crate::github_connection::REGISTERED_CLIENT_ID,
        "verificationUri":VERIFICATION_URI,
        "destinations":["https://github.com/login/device/code","https://github.com/login/oauth/access_token","https://api.github.com/user"],
        "effect":"Begin one device authorisation. Enter only the code initiated here at the fixed GitHub verification page. GitHub credentials stay in the native host and Windows Credential Manager. No data or Personalization is sent to Meta. Account selection requires another native review. Connecting permits no repository mutation or Contributor disclosure."
    }))?.with_request_cancellation(cancellation.clone());
    if !review(&window, document, &guard).await? {
        let mut state = lock(&host)?;
        state.check_epoch(epoch)?;
        state.cancel()?;
        return Ok(state.status());
    }
    lock(&host)?.check_epoch(epoch)?;
    let api = GithubApi::new().map_err(api_error)?;
    let request_started = lock(&host)?.started.elapsed().as_secs();
    let response = match cancellable(&cancellation, async {
        api.request_device().await.map_err(api_error)
    })
    .await
    {
        Ok(response) => response,
        Err(error) => {
            let mut state = lock(&host)?;
            if state.connection.generation() == epoch {
                state.cancel()?;
            }
            return Err(error);
        }
    };
    let mut state = lock(&host)?;
    state.check_epoch(epoch)?;
    let session = DeviceSession::new_received(
        response,
        epoch,
        request_started,
        state.started.elapsed().as_secs(),
    )
    .map_err(|_| "github_contract_rejected")?;
    state.display_code = session.user_code().map(str::to_owned);
    state.session = Some(session);
    Ok(state.status())
}

#[tauri::command]
pub(crate) async fn poll_github_connection(
    window: WebviewWindow,
    host: State<'_, Host>,
) -> Result<GithubStatus, String> {
    trusted(&window)?;
    let guard = host.enter_native_dialog()?;
    host.idle()?;
    let (epoch, mut session, cancellation) = {
        let mut state = lock(&host)?;
        let epoch = state.connection.generation();
        state.check_epoch(epoch)?;
        if state.in_flight {
            return Err("github_busy".into());
        }
        let tick = state.started.elapsed().as_secs();
        let mut session = state.session.take().ok_or("github_no_device_session")?;
        match session.begin_poll(tick, epoch) {
            PollDecision::RequestReady => {
                state.in_flight = true;
            }
            PollDecision::WaitUntil(_) | PollDecision::InFlight => {
                state.session = Some(session);
                return Ok(state.status());
            }
            PollDecision::Terminal(_) => {
                state.display_code = None;
                return Ok(state.status());
            }
        }
        (epoch, session, state.cancellation.subscribe())
    };
    let _lease = InflightLease { host: &host, epoch };
    let api = GithubApi::new().map_err(api_error)?;
    let outcome = cancellable(&cancellation, async {
        api.poll_device(&session, crate::now()?)
            .await
            .map_err(api_error)
    })
    .await;
    // No mutex guard crosses await. A disable/switch/cancel makes this result stale.
    {
        let state = lock(&host)?;
        state.check_epoch(epoch)?;
    }
    match outcome {
        Ok(TokenPoll::Pending(outcome)) => {
            let mut state = lock(&host)?;
            state.check_epoch(epoch)?;
            let tick = state.started.elapsed().as_secs();
            state.in_flight = false;
            match session.observe_result(outcome, tick, epoch) {
                PollDecision::WaitUntil(_) => state.session = Some(session),
                _ => {
                    state.display_code = None;
                }
            }
            Ok(state.status())
        }
        Err(error) => {
            let mut state = lock(&host)?;
            state.check_epoch(epoch)?;
            session.observe_result(
                PollOutcome::FailedClosed,
                state.started.elapsed().as_secs(),
                epoch,
            );
            state.in_flight = false;
            state.display_code = None;
            Err(error)
        }
        Ok(TokenPoll::Granted(tokens)) => {
            {
                let mut state = lock(&host)?;
                state.check_epoch(epoch)?;
                if session.observe_result(
                    PollOutcome::TokenReceived,
                    state.started.elapsed().as_secs(),
                    epoch,
                ) != PollDecision::Terminal(DeviceTerminal::AwaitingAccountVerification)
                {
                    state.in_flight = false;
                    state.display_code = None;
                    return Err("github_device_expired".into());
                }
            }
            let account = cancellable(&cancellation, async {
                api.authenticated_account(&tokens, crate::now()?)
                    .await
                    .map_err(api_error)
            })
            .await?;
            lock(&host)?.check_epoch(epoch)?;
            let document=operation(serde_json::json!({
                "operation":"Confirm this authenticated GitHub account for Aster",
                "app":"Aster Desktop Engineering","account":account,
                "credentialTarget":crate::github_credential::TARGET,
                "effect":"Save only this expiring device-flow token pair for the current Windows user. GitHub rights do not permit private/personal/confidential data to be sent to Meta. No repository or write grant is created. Local disconnect forgets Aster tokens but does not revoke GitHub authorisation remotely."
            }))?.with_request_cancellation(cancellation.clone());
            if !review(&window, document, &guard).await? {
                let mut state = lock(&host)?;
                state.check_epoch(epoch)?;
                state.cancel()?;
                return Ok(state.status());
            }
            let reverified = cancellable(&cancellation, async {
                api.authenticated_account(&tokens, crate::now()?)
                    .await
                    .map_err(api_error)
            })
            .await?;
            if reverified != account {
                return Err("github_identity_changed".into());
            }
            // Final comparison + fixed synchronous custody publication share the
            // short mutex section, preventing concurrent disable/disconnect.
            let mut state = lock(&host)?;
            state.check_epoch(epoch)?;
            let stored = StoredConnection {
                account: account.clone(),
                tokens,
                rotation_pending: false,
            };
            GithubCredentialStore
                .save(&stored)
                .map_err(|_| "github_credential_unavailable")?;
            state
                .connection
                .record_verified_account(epoch, account.id, &account.login)
                .map_err(|_| "github_identity_changed")?;
            state.stored = Some(stored);
            state.account = Some(account);
            state.in_flight = false;
            state.display_code = None;
            Ok(state.status())
        }
    }
}

#[tauri::command]
pub(crate) fn cancel_github_connection(
    window: WebviewWindow,
    host: State<'_, Host>,
) -> Result<GithubStatus, String> {
    trusted(&window)?;
    let mut state = lock(&host)?;
    state.cancel()?;
    Ok(state.status())
}

#[tauri::command]
pub(crate) async fn disconnect_github(
    window: WebviewWindow,
    host: State<'_, Host>,
) -> Result<GithubStatus, String> {
    trusted(&window)?;
    let guard = host.enter_native_dialog()?;
    host.idle()?;
    let (epoch, account, cancellation) = {
        let state = lock(&host)?;
        (
            state.connection.generation(),
            state.account.clone(),
            state.cancellation.subscribe(),
        )
    };
    let document=operation(serde_json::json!({"operation":"Forget Aster's local GitHub connection","account":account,
        "target":crate::github_credential::TARGET,"effect":"Invalidate pending sessions and grants before deleting Aster's fixed current-user GitHub credential. This does not remotely revoke GitHub authorisation; manage it manually at https://github.com/settings/apps. Meta credentials, history and Personalization remain untouched."}))?.with_request_cancellation(cancellation);
    if !review(&window, document, &guard).await? {
        return Ok(lock(&host)?.status());
    }
    let mut state = lock(&host)?;
    if state.connection.generation() != epoch {
        return Err("github_stale_generation".into());
    }
    state.cancel()?;
    // Keep persisted enabled preference; explicit disconnect means unbound, not
    // an unrelated catalogue preference mutation or remote revocation.
    GithubCredentialStore
        .delete()
        .map_err(|_| "github_credential_unavailable")?;
    state.reconnect_required = false;
    Ok(state.status())
}

async fn enumerate(
    api: &GithubApi,
    stored: &StoredConnection,
    cancellation: &watch::Receiver<bool>,
) -> Result<Vec<(InstallationRecord, RepositoryRecord)>, String> {
    let account = cancellable(cancellation, async {
        api.authenticated_account(&stored.tokens, crate::now()?)
            .await
            .map_err(api_error)
    })
    .await?;
    if account != stored.account {
        return Err("github_identity_changed".into());
    }
    let mut result = Vec::new();
    let mut request_count = 0;
    for page in 1..=10 {
        request_count += 1;
        let installations = cancellable(cancellation, async {
            api.installations(&stored.tokens, crate::now()?, page)
                .await
                .map_err(api_error)
        })
        .await?;
        for installation in installations.installations {
            for repo_page in 1..=10 {
                request_count += 1;
                if request_count > 20 {
                    return Err("github_enumeration_limit".into());
                }
                let repos = cancellable(cancellation, async {
                    api.repositories(&stored.tokens, crate::now()?, &installation, repo_page)
                        .await
                        .map_err(api_error)
                })
                .await?;
                let length = repos.repositories.len();
                for repo in repos.repositories {
                    if result.iter().any(
                        |(_, existing): &(InstallationRecord, RepositoryRecord)| {
                            existing.identity == repo.identity
                        },
                    ) {
                        return Err("github_contract_rejected".into());
                    }
                    result.push((installation.clone(), repo));
                    if result.len() > MAX_REPOSITORIES {
                        return Err("github_enumeration_limit".into());
                    }
                }
                if repo_page * 100 >= repos.total_count || length < 100 {
                    break;
                }
            }
        }
        if page * 100 >= installations.total_count {
            break;
        }
    }
    Ok(result)
}

fn take_stored(host: &Host) -> Result<(u64, StoredConnection, watch::Receiver<bool>), String> {
    let mut state = lock(host)?;
    let epoch = state.connection.generation();
    state.check_epoch(epoch)?;
    if state.in_flight {
        return Err("github_busy".into());
    }
    let stored = state.stored.take().ok_or("github_reconnect_required")?;
    state.in_flight = true;
    Ok((epoch, stored, state.cancellation.subscribe()))
}
fn return_stored(host: &Host, epoch: u64, stored: StoredConnection) -> Result<(), String> {
    let mut state = lock(host)?;
    state.check_epoch(epoch)?;
    state.stored = Some(stored);
    state.in_flight = false;
    Ok(())
}

fn admit_result<T>(host: &Host, epoch: u64, result: Result<T, String>) -> Result<T, String> {
    if result.as_ref().err().is_some_and(|e| {
        matches!(
            e.as_str(),
            "github_authentication_failed"
                | "github_reconnect_required"
                | "github_identity_changed"
        )
    }) {
        let mut state = lock(host)?;
        state.check_epoch(epoch)?;
        state.cancel()?;
        state.reconnect_required = true;
    }
    result
}

#[tauri::command]
pub(crate) async fn list_github_repositories(
    window: WebviewWindow,
    host: State<'_, Host>,
) -> Result<Vec<RepositoryRecord>, String> {
    trusted(&window)?;
    let _guard = host.enter_native_dialog()?;
    host.idle()?;
    let (epoch, stored, cancellation) = take_stored(&host)?;
    let _lease = InflightLease { host: &host, epoch };
    let api = GithubApi::new().map_err(api_error)?;
    let result = tokio::time::timeout(
        Duration::from_secs(OPERATION_SECONDS),
        enumerate(&api, &stored, &cancellation),
    )
    .await
    .map_err(|_| "github_operation_expired".to_owned())
    .and_then(|r| r);
    return_stored(&host, epoch, stored)?;
    let repositories = admit_result(&host, epoch, result)?;
    let mut state = lock(&host)?;
    state.check_epoch(epoch)?;
    let records = repositories.iter().map(|(_, r)| r.clone()).collect();
    state.repositories = repositories;
    Ok(records)
}

#[tauri::command]
pub(crate) async fn select_github_repository(
    window: WebviewWindow,
    host: State<'_, Host>,
    input: SelectRepositoryInput,
) -> Result<GithubStatus, String> {
    trusted(&window)?;
    let guard = host.enter_native_dialog()?;
    host.idle()?;
    if input.installation_id == 0 || input.repository_id == 0 {
        return Err("github_contract_rejected".into());
    }
    let (epoch, stored, cancellation) = take_stored(&host)?;
    let _lease = InflightLease { host: &host, epoch };
    let api = GithubApi::new().map_err(api_error)?;
    let result = tokio::time::timeout(
        Duration::from_secs(OPERATION_SECONDS),
        enumerate(&api, &stored, &cancellation),
    )
    .await
    .map_err(|_| "github_operation_expired".to_owned())
    .and_then(|r| r);
    return_stored(&host, epoch, stored)?;
    let (installation, repository) = admit_result(&host, epoch, result)?
        .into_iter()
        .find(|(i, r)| {
            i.id == input.installation_id && r.identity.repository_id() == input.repository_id
        })
        .ok_or("github_repository_not_installed")?;
    let document=operation(serde_json::json!({"operation":"Select this installed GitHub repository",
        "account":lock(&host)?.account,"repository":repository,"installationId":installation.id,
        "effect":"Select only this repository for bounded explicit reads and future separately reviewed changes. No mutation or private-content disclosure to Meta is authorised. Changing repository invalidates old proposals."}))?.with_request_cancellation(cancellation);
    if !review(&window, document, &guard).await? {
        return Ok(lock(&host)?.status());
    }
    // Re-read the token's installed scope after exact review, not generic public GET alone.
    let (new_epoch, stored, cancellation) = take_stored(&host)?;
    if new_epoch != epoch {
        return_stored(&host, new_epoch, stored)?;
        return Err("github_stale_generation".into());
    }
    let refreshed = tokio::time::timeout(
        Duration::from_secs(OPERATION_SECONDS),
        enumerate(&api, &stored, &cancellation),
    )
    .await
    .map_err(|_| "github_operation_expired".to_owned())
    .and_then(|r| r);
    return_stored(&host, epoch, stored)?;
    let (confirmed_installation, confirmed) = admit_result(&host, epoch, refreshed)?
        .into_iter()
        .find(|(i, r)| i.id == installation.id && r.identity == repository.identity)
        .ok_or("github_repository_not_installed")?;
    let mut state = lock(&host)?;
    state.check_epoch(epoch)?;
    state
        .connection
        .select_verified_repository(epoch, confirmed.identity.clone())
        .map_err(|_| "github_identity_changed")?;
    state.selected = Some((confirmed_installation, confirmed));
    state.repositories.clear();
    Ok(state.status())
}

#[tauri::command]
pub(crate) async fn read_github_file(
    window: WebviewWindow,
    host: State<'_, Host>,
    input: ReadFileInput,
) -> Result<crate::github_api::FileRecord, String> {
    trusted(&window)?;
    crate::github_api::validate_file_selection(&input.path, &input.branch).map_err(api_error)?;
    let _guard = host.enter_native_dialog()?;
    host.idle()?;
    let selected = lock(&host)?
        .selected
        .clone()
        .ok_or("github_repository_not_selected")?;
    let (epoch, stored, cancellation) = take_stored(&host)?;
    let _lease = InflightLease { host: &host, epoch };
    let api = GithubApi::new().map_err(api_error)?;
    let result = tokio::time::timeout(
        Duration::from_secs(OPERATION_SECONDS),
        cancellable(&cancellation, async {
            let current = enumerate(&api, &stored, &cancellation).await?;
            let (installation, repository) = current
                .into_iter()
                .find(|(i, r)| i.id == selected.0.id && r.identity == selected.1.identity)
                .ok_or("github_repository_not_installed")?;
            api.read_file(
                &stored.tokens,
                crate::now()?,
                &repository.identity,
                &installation,
                crate::github_api::FileSelection {
                    path: &input.path,
                    branch: &input.branch,
                },
                &cancellation,
            )
            .await
            .map_err(api_error)
        }),
    )
    .await
    .map_err(|_| "github_operation_expired".to_owned())
    .and_then(|r| r);
    return_stored(&host, epoch, stored)?;
    admit_result(&host, epoch, result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn temporary_native_token_ownership_keeps_status_consistent_without_restoring_authority() {
        let mut state = GithubHost::default();
        state.set_enabled(true).unwrap();
        state.in_flight = true;
        assert_eq!(state.status().state, "disconnected");
        assert!(state.status().account.is_none());
        assert!(state.status().repository.is_none());

        let account = AccountIdentity {
            id: 1,
            login: "synthetic".into(),
        };
        let identity = RepositoryIdentity::new(2, 3, "synthetic", "fixture").unwrap();
        state.account = Some(account.clone());
        state.selected = Some((
            InstallationRecord {
                id: 2,
                account,
                repository_selection: "selected".into(),
                contents_write: false,
                issues_write: false,
                pull_requests_write: false,
            },
            RepositoryRecord {
                identity: identity.clone(),
                private: true,
                can_read: true,
                can_write: false,
            },
        ));
        assert!(state.stored.is_none());
        let status = state.status();
        assert_eq!(status.state, "connected");
        assert!(status.in_flight);
        assert_eq!(status.repository, Some(identity));
        assert_eq!(status.account.as_ref().unwrap().id, 1);
        assert!(!status.mutations_available);
        let serialized = serde_json::to_value(status).unwrap();
        assert!(serialized["repository"].is_object());
        assert!(serialized["userCode"].is_null());

        // An inconsistent idle state cannot expose retained account/selection.
        state.in_flight = false;
        assert_eq!(state.status().state, "disconnected");
        assert!(state.status().account.is_none());
        assert!(state.status().repository.is_none());
        state.cancel().unwrap();
        assert!(state.account.is_none());
        assert!(state.selected.is_none());
        assert!(!state.status().in_flight);
    }
    #[test]
    fn volatile_enabled_preference_cancels_old_epoch_without_restoring_account_or_network() {
        let mut state = GithubHost::default();
        assert_eq!(state.status().state, "disabled");
        state.set_enabled(true).unwrap();
        let epoch = state.connection.generation();
        assert_eq!(state.status().state, "disconnected");
        assert!(state.status().account.is_none());
        state.cancel().unwrap();
        assert!(state.check_epoch(epoch).is_err());
        state.set_enabled(false).unwrap();
        assert!(!state.status().mutations_available);
        assert_eq!(state.status().state, "disabled");
        assert!(state.status().user_code.is_none());
    }
    #[test]
    fn renderer_cannot_supply_approval_secret_url_or_generation() {
        for fixture in [
            br#"{"installationId":1,"repositoryId":2,"confirmed":true}"#.as_slice(),
            br#"{"installationId":1,"repositoryId":2,"token":"not admitted"}"#,
            br#"{"installationId":1,"repositoryId":2,"url":"https://evil.invalid"}"#,
        ] {
            assert!(serde_json::from_slice::<SelectRepositoryInput>(fixture).is_err());
        }
        assert!(serde_json::from_slice::<ReadFileInput>(
            br#"{"path":"test.md","branch":"main","generation":9}"#
        )
        .is_err());
    }

    #[tokio::test]
    async fn cancelled_or_closed_epoch_never_polls_the_network_future() {
        let (sender, receiver) = watch::channel(false);
        sender.send(true).unwrap();
        let polls = std::cell::Cell::new(0);
        let outcome = cancellable(&receiver, async {
            polls.set(polls.get() + 1);
            Ok(())
        })
        .await;
        assert_eq!(outcome, Err("github_cancelled".to_owned()));
        assert_eq!(polls.get(), 0);
        let (sender, receiver) = watch::channel(false);
        drop(sender);
        let outcome = cancellable(&receiver, async {
            polls.set(polls.get() + 1);
            Ok(())
        })
        .await;
        assert_eq!(outcome, Err("github_cancelled".to_owned()));
        assert_eq!(polls.get(), 0);
    }

    #[tokio::test]
    async fn cancellation_interrupts_a_pending_future_without_retry_or_result() {
        let (sender, receiver) = watch::channel(false);
        let polls = std::cell::Cell::new(0);
        let outcome: Result<(), String> = cancellable(&receiver, async {
            polls.set(polls.get() + 1);
            sender.send(true).unwrap();
            std::future::pending::<Result<(), String>>().await
        })
        .await;
        assert_eq!(outcome, Err("github_cancelled".to_owned()));
        assert_eq!(polls.get(), 1);
    }
}
