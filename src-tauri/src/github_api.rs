//! Closed, native-only GitHub device/read adapter. Not an IPC or model tool.
//! Callers must independently own session/cancellation, native account consent,
//! restored-state revalidation and installed repository selection. No constructor
//! or successful GET supplies mutation, Contributor disclosure or admin rights.
//! Primary GitHub contracts checked 10 October 2026: generating a user access
//! token, refreshing user access tokens, REST users and App installations.
//! Device-origin refresh explicitly requires no distributed Client Secret.

use crate::github_connection::{
    parse_poll_error, AppClientId, DeviceResponse, DeviceSession, PollOutcome, RepositoryIdentity,
    DEVICE_CODE_ENDPOINT, DEVICE_GRANT_TYPE, DEVICE_TOKEN_ENDPOINT, RESOURCE_ORIGIN,
};
use base64::Engine;
use reqwest::{header, Client, Method, Response};
use serde::{Deserialize, Deserializer, Serialize};
use std::time::Duration;
use zeroize::Zeroizing;

pub(crate) const REGISTERED_APP_ID: u64 = 5_256_613;
const API_VERSION: &str = "2026-03-10";
const AUTH_BODY_LIMIT: usize = 4_096;
const RESOURCE_BODY_LIMIT: usize = 1_048_576;
const MAX_PAGE: u32 = 10;
const PAGE_SIZE: usize = 100;
const ACCESS_SECONDS: u64 = 28_800;
const REFRESH_SECONDS: u64 = 15_897_600;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ApiError {
    InvalidContract,
    Inactive,
    Expired,
    ReconnectRequired,
    Authentication,
    Permission,
    RateLimited,
    Unavailable,
    Oversized,
    IdentityChanged,
}

impl ApiError {
    pub(crate) fn code(self) -> &'static str {
        match self {
            Self::InvalidContract => "github_contract_rejected",
            Self::Inactive => "github_inactive",
            Self::Expired | Self::ReconnectRequired => "github_reconnect_required",
            Self::Authentication => "github_authentication_failed",
            Self::Permission => "github_permission_denied",
            Self::RateLimited => "github_rate_limited",
            Self::Unavailable => "github_unavailable",
            Self::Oversized => "github_response_too_large",
            Self::IdentityChanged => "github_identity_changed",
        }
    }
}

fn private_string<'de, D: Deserializer<'de>>(d: D) -> Result<Zeroizing<String>, D::Error> {
    String::deserialize(d).map(Zeroizing::new)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TokenWire {
    #[serde(deserialize_with = "private_string")]
    access_token: Zeroizing<String>,
    #[serde(deserialize_with = "private_string")]
    refresh_token: Zeroizing<String>,
    expires_in: u64,
    refresh_token_expires_in: u64,
    scope: String,
    token_type: String,
}

/// No Debug, Clone, Serialize or public token accessors. Host-only custody data.
pub(crate) struct TokenSet {
    pub(crate) access: Zeroizing<String>,
    pub(crate) refresh: Zeroizing<String>,
    pub(crate) issued_at: u64,
    pub(crate) access_expires_at: u64,
    pub(crate) refresh_expires_at: u64,
}

fn token_shape(value: &str, prefix: &str) -> bool {
    value.strip_prefix(prefix).is_some_and(|suffix| {
        (16..=252).contains(&suffix.len()) && suffix.bytes().all(|b| b.is_ascii_alphanumeric())
    })
}

impl TokenSet {
    pub(crate) fn parse_json(bytes: &[u8], now: u64) -> Result<Self, ApiError> {
        if bytes.len() > AUTH_BODY_LIMIT || now == 0 {
            return Err(ApiError::InvalidContract);
        }
        let wire: TokenWire =
            serde_json::from_slice(bytes).map_err(|_| ApiError::InvalidContract)?;
        if !token_shape(&wire.access_token, "ghu_")
            || !token_shape(&wire.refresh_token, "ghr_")
            || wire.token_type != "bearer"
            || !wire.scope.is_empty()
            || wire.expires_in != ACCESS_SECONDS
            || wire.refresh_token_expires_in != REFRESH_SECONDS
        {
            return Err(ApiError::InvalidContract);
        }
        let access_expires_at = now
            .checked_add(ACCESS_SECONDS)
            .ok_or(ApiError::InvalidContract)?;
        let refresh_expires_at = now
            .checked_add(REFRESH_SECONDS)
            .ok_or(ApiError::InvalidContract)?;
        Ok(Self {
            access: wire.access_token,
            refresh: wire.refresh_token,
            issued_at: now,
            access_expires_at,
            refresh_expires_at,
        })
    }

    pub(crate) fn validate_restored(&self) -> Result<(), ApiError> {
        if !token_shape(&self.access, "ghu_")
            || !token_shape(&self.refresh, "ghr_")
            || self.issued_at == 0
            || self.issued_at.checked_add(ACCESS_SECONDS) != Some(self.access_expires_at)
            || self.issued_at.checked_add(REFRESH_SECONDS) != Some(self.refresh_expires_at)
        {
            return Err(ApiError::InvalidContract);
        }
        Ok(())
    }

    fn access_at(&self, now: u64) -> Result<&str, ApiError> {
        self.validate_restored()?;
        if now < self.issued_at {
            return Err(ApiError::ReconnectRequired);
        }
        if now >= self.access_expires_at {
            return Err(ApiError::Expired);
        }
        Ok(&self.access)
    }

    #[cfg(test)]
    fn refresh_at(&self, now: u64) -> Result<&str, ApiError> {
        self.validate_restored()?;
        if now < self.issued_at || now >= self.refresh_expires_at {
            return Err(ApiError::ReconnectRequired);
        }
        Ok(&self.refresh)
    }
}

pub(crate) enum TokenPoll {
    Pending(PollOutcome),
    Granted(TokenSet),
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AccountIdentity {
    pub(crate) id: u64,
    pub(crate) login: String,
}

#[derive(Deserialize)]
struct AccountWire {
    id: u64,
    login: String,
}

fn valid_login(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 39
        && !value.starts_with('-')
        && !value.ends_with('-')
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
}

fn account_from(wire: AccountWire) -> Result<AccountIdentity, ApiError> {
    if wire.id == 0 || !valid_login(&wire.login) {
        return Err(ApiError::InvalidContract);
    }
    Ok(AccountIdentity {
        id: wire.id,
        login: wire.login,
    })
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InstallationRecord {
    pub(crate) id: u64,
    pub(crate) account: AccountIdentity,
    pub(crate) repository_selection: String,
    pub(crate) contents_write: bool,
    pub(crate) issues_write: bool,
    pub(crate) pull_requests_write: bool,
}

#[derive(Deserialize)]
struct InstallationWire {
    id: u64,
    app_id: u64,
    account: AccountWire,
    repository_selection: String,
    suspended_at: Option<serde_json::Value>,
    permissions: PermissionsWire,
}

#[derive(Deserialize)]
struct PermissionsWire {
    metadata: Option<String>,
    contents: Option<String>,
    issues: Option<String>,
    pull_requests: Option<String>,
}

#[derive(Deserialize)]
struct InstallationsWire {
    total_count: u32,
    installations: Vec<InstallationWire>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InstallationPage {
    pub(crate) total_count: u32,
    pub(crate) installations: Vec<InstallationRecord>,
}

fn permission(value: Option<String>) -> Result<bool, ApiError> {
    match value.as_deref() {
        None | Some("read") => Ok(false),
        Some("write") => Ok(true),
        _ => Err(ApiError::InvalidContract),
    }
}

fn installation_from(wire: InstallationWire) -> Result<InstallationRecord, ApiError> {
    if wire.id == 0
        || wire.app_id != REGISTERED_APP_ID
        || wire.suspended_at.is_some()
        || !matches!(wire.repository_selection.as_str(), "all" | "selected")
        || wire.permissions.metadata.as_deref() != Some("read")
    {
        return Err(ApiError::InvalidContract);
    }
    Ok(InstallationRecord {
        id: wire.id,
        account: account_from(wire.account)?,
        repository_selection: wire.repository_selection,
        contents_write: permission(wire.permissions.contents)?,
        issues_write: permission(wire.permissions.issues)?,
        pull_requests_write: permission(wire.permissions.pull_requests)?,
    })
}

#[derive(Deserialize)]
struct RepositoryWire {
    id: u64,
    name: String,
    full_name: String,
    owner: AccountWire,
    private: bool,
    archived: bool,
    disabled: bool,
    permissions: RepositoryPermissionsWire,
}
#[derive(Deserialize)]
struct RepositoryPermissionsWire {
    pull: bool,
    push: bool,
}
#[derive(Deserialize)]
struct RepositoriesWire {
    total_count: u32,
    repositories: Vec<RepositoryWire>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RepositoryRecord {
    pub(crate) identity: RepositoryIdentity,
    pub(crate) private: bool,
    pub(crate) can_read: bool,
    pub(crate) can_write: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RepositoryPage {
    pub(crate) total_count: u32,
    pub(crate) repositories: Vec<RepositoryRecord>,
}

/// Bounded remote text, still private/untrusted data, not Meta disclosure rights.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FileRecord {
    pub(crate) path: String,
    pub(crate) branch: String,
    pub(crate) blob_sha: String,
    pub(crate) content: String,
}

#[derive(Deserialize)]
struct FileWire {
    #[serde(rename = "type")]
    object_type: String,
    path: String,
    sha: String,
    size: u64,
    encoding: String,
    content: String,
}

fn file_path(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 240
        && !value.starts_with('/')
        && value.split('/').all(|part| {
            !part.is_empty()
                && !matches!(part, "." | "..")
                && part.len() <= 80
                && part
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
                && !matches!(
                    part.to_ascii_lowercase().as_str(),
                    ".git" | ".env" | "credentials" | "id_rsa" | "id_ed25519"
                )
                && !part.to_ascii_lowercase().ends_with(".pem")
        })
}

fn branch_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 100
        && !value.starts_with('/')
        && !value.ends_with('/')
        && value.split('/').all(|part| {
            !part.is_empty()
                && !part.starts_with('.')
                && !part.ends_with('.')
                && !part.ends_with(".lock")
                && part
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
                && !part.contains("..")
        })
}

pub(crate) fn validate_file_selection(path: &str, branch: &str) -> Result<(), ApiError> {
    if !file_path(path) || !branch_name(branch) {
        return Err(ApiError::InvalidContract);
    }
    Ok(())
}

fn file_from(wire: FileWire, path: &str, branch: &str) -> Result<FileRecord, ApiError> {
    if !file_path(path)
        || !branch_name(branch)
        || wire.object_type != "file"
        || wire.path != path
        || wire.size > 65_536
        || wire.encoding != "base64"
        || wire.content.len() > 90_000
        || wire.sha.len() != 40
        || !wire
            .sha
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err(ApiError::InvalidContract);
    }
    let encoded: String = wire
        .content
        .chars()
        .filter(|c| !matches!(c, '\n' | '\r'))
        .collect();
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|_| ApiError::InvalidContract)?;
    if bytes.len() != wire.size as usize {
        return Err(ApiError::InvalidContract);
    }
    let content = String::from_utf8(bytes).map_err(|_| ApiError::InvalidContract)?;
    if content.contains('\0') {
        return Err(ApiError::InvalidContract);
    }
    Ok(FileRecord {
        path: wire.path,
        branch: branch.to_owned(),
        blob_sha: wire.sha,
        content,
    })
}

fn repository_from(
    wire: RepositoryWire,
    installation: &InstallationRecord,
) -> Result<RepositoryRecord, ApiError> {
    let owner = account_from(wire.owner)?;
    if owner.id != installation.account.id
        || owner.login != installation.account.login
        || wire.archived
        || wire.disabled
        || !wire.permissions.pull
        || wire.full_name != format!("{}/{}", owner.login, wire.name)
    {
        return Err(ApiError::InvalidContract);
    }
    let identity = RepositoryIdentity::new(installation.id, wire.id, &owner.login, &wire.name)
        .map_err(|_| ApiError::InvalidContract)?;
    Ok(RepositoryRecord {
        identity,
        private: wire.private,
        can_read: wire.permissions.pull,
        can_write: wire.permissions.push && installation.contents_write,
    })
}

/// Constructing a client is local-only; no background request or retry is made.
pub(crate) struct GithubApi {
    client: Client,
}

pub(crate) struct FileSelection<'a> {
    pub(crate) path: &'a str,
    pub(crate) branch: &'a str,
}

impl GithubApi {
    pub(crate) fn new() -> Result<Self, ApiError> {
        let client = Client::builder()
            .https_only(true)
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(20))
            .user_agent("AsterDesktop/0.2.4")
            .no_proxy()
            .build()
            .map_err(|_| ApiError::Unavailable)?;
        Ok(Self { client })
    }

    pub(crate) async fn request_device(&self) -> Result<DeviceResponse, ApiError> {
        let client_id = AppClientId::registered().map_err(|_| ApiError::InvalidContract)?;
        let body = format!("client_id={}&scope=offline_access", client_id.as_str());
        let bytes = self
            .auth_post(DEVICE_CODE_ENDPOINT, Zeroizing::new(body))
            .await?;
        DeviceResponse::parse_json(&bytes).map_err(|_| ApiError::InvalidContract)
    }

    /// The caller must have admitted this single poll via begin_poll and must
    /// recheck the generation after await before recording/publishing results.
    pub(crate) async fn poll_device(
        &self,
        session: &DeviceSession,
        now: u64,
    ) -> Result<TokenPoll, ApiError> {
        let code = session
            .device_code_for_inflight()
            .ok_or(ApiError::Inactive)?;
        let body = Zeroizing::new(format!(
            "client_id={}&device_code={}&grant_type={}",
            AppClientId::registered()
                .map_err(|_| ApiError::InvalidContract)?
                .as_str(),
            code,
            DEVICE_GRANT_TYPE.replace(':', "%3A")
        ));
        let bytes = self.auth_post(DEVICE_TOKEN_ENDPOINT, body).await?;
        match TokenSet::parse_json(&bytes, now) {
            Ok(tokens) => Ok(TokenPoll::Granted(tokens)),
            Err(_) => parse_poll_error(&bytes)
                .map(TokenPoll::Pending)
                .map_err(|_| ApiError::InvalidContract),
        }
    }

    pub(crate) async fn authenticated_account(
        &self,
        tokens: &TokenSet,
        now: u64,
    ) -> Result<AccountIdentity, ApiError> {
        let bytes = self.resource_get(tokens, now, "/user").await?;
        account_from(serde_json::from_slice(&bytes).map_err(|_| ApiError::InvalidContract)?)
    }

    pub(crate) async fn installations(
        &self,
        tokens: &TokenSet,
        now: u64,
        page: u32,
    ) -> Result<InstallationPage, ApiError> {
        page_check(page)?;
        let bytes = self
            .resource_get(
                tokens,
                now,
                &format!("/user/installations?per_page=100&page={page}"),
            )
            .await?;
        let wire: InstallationsWire =
            serde_json::from_slice(&bytes).map_err(|_| ApiError::InvalidContract)?;
        collection_check(wire.total_count, wire.installations.len())?;
        let installations = wire
            .installations
            .into_iter()
            .map(installation_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(InstallationPage {
            total_count: wire.total_count,
            installations,
        })
    }

    /// installation must come from this token's fresh installations response,
    /// never an object supplied by React or restored as runtime authority.
    pub(crate) async fn repositories(
        &self,
        tokens: &TokenSet,
        now: u64,
        installation: &InstallationRecord,
        page: u32,
    ) -> Result<RepositoryPage, ApiError> {
        page_check(page)?;
        if installation.id == 0 {
            return Err(ApiError::InvalidContract);
        }
        let bytes = self
            .resource_get(
                tokens,
                now,
                &format!(
                    "/user/installations/{}/repositories?per_page=100&page={page}",
                    installation.id
                ),
            )
            .await?;
        let wire: RepositoriesWire =
            serde_json::from_slice(&bytes).map_err(|_| ApiError::InvalidContract)?;
        collection_check(wire.total_count, wire.repositories.len())?;
        let repositories = wire
            .repositories
            .into_iter()
            .map(|r| repository_from(r, installation))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(RepositoryPage {
            total_count: wire.total_count,
            repositories,
        })
    }

    pub(crate) async fn repository(
        &self,
        tokens: &TokenSet,
        now: u64,
        identity: &RepositoryIdentity,
        installation: &InstallationRecord,
    ) -> Result<RepositoryRecord, ApiError> {
        if identity.installation_id() != installation.id {
            return Err(ApiError::IdentityChanged);
        }
        let bytes = self
            .resource_get(
                tokens,
                now,
                &format!("/repos/{}/{}", identity.owner(), identity.name()),
            )
            .await?;
        let result = repository_from(
            serde_json::from_slice(&bytes).map_err(|_| ApiError::InvalidContract)?,
            installation,
        )?;
        if result.identity != *identity {
            return Err(ApiError::IdentityChanged);
        }
        Ok(result)
    }

    /// Re-read repository identity before this one bounded explicit remote read.
    /// This method never follows download_url/git_url or a response-supplied URI.
    pub(crate) async fn read_file(
        &self,
        tokens: &TokenSet,
        now: u64,
        identity: &RepositoryIdentity,
        installation: &InstallationRecord,
        selection: FileSelection<'_>,
        cancellation: &tokio::sync::watch::Receiver<bool>,
    ) -> Result<FileRecord, ApiError> {
        let FileSelection { path, branch } = selection;
        validate_file_selection(path, branch)?;
        if *cancellation.borrow() || cancellation.has_changed().is_err() {
            return Err(ApiError::Inactive);
        }
        self.repository(tokens, now, identity, installation).await?;
        if *cancellation.borrow() || cancellation.has_changed().is_err() {
            return Err(ApiError::Inactive);
        }
        let mut url =
            reqwest::Url::parse(RESOURCE_ORIGIN).map_err(|_| ApiError::InvalidContract)?;
        {
            let mut parts = url
                .path_segments_mut()
                .map_err(|_| ApiError::InvalidContract)?;
            parts
                .push("repos")
                .push(identity.owner())
                .push(identity.name())
                .push("contents");
            for part in path.split('/') {
                parts.push(part);
            }
        }
        url.query_pairs_mut().append_pair("ref", branch);
        let path_query = format!(
            "{}?{}",
            url.path(),
            url.query().ok_or(ApiError::InvalidContract)?
        );
        let bytes = self.resource_get(tokens, now, &path_query).await?;
        file_from(
            serde_json::from_slice(&bytes).map_err(|_| ApiError::InvalidContract)?,
            path,
            branch,
        )
    }

    async fn auth_post(
        &self,
        url: &'static str,
        body: Zeroizing<String>,
    ) -> Result<Zeroizing<Vec<u8>>, ApiError> {
        if !matches!(url, DEVICE_CODE_ENDPOINT | DEVICE_TOKEN_ENDPOINT) {
            return Err(ApiError::InvalidContract);
        }
        let response = self
            .client
            .request(Method::POST, url)
            .header(header::ACCEPT, "application/json")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(body.as_bytes().to_vec())
            .send()
            .await
            .map_err(|_| ApiError::Unavailable)?;
        bounded_body(response, AUTH_BODY_LIMIT).await
    }

    async fn resource_get(
        &self,
        tokens: &TokenSet,
        now: u64,
        path: &str,
    ) -> Result<Zeroizing<Vec<u8>>, ApiError> {
        // Private method: only fixed constructors above can create this path.
        if !path.starts_with('/') || path.contains('#') || path.contains('\\') {
            return Err(ApiError::InvalidContract);
        }
        let mut auth = header::HeaderValue::from_str(&format!("Bearer {}", tokens.access_at(now)?))
            .map_err(|_| ApiError::InvalidContract)?;
        auth.set_sensitive(true);
        let response = self
            .client
            .get(format!("{RESOURCE_ORIGIN}{path}"))
            .header(header::ACCEPT, "application/vnd.github+json")
            .header("X-GitHub-Api-Version", API_VERSION)
            .header(header::AUTHORIZATION, auth)
            .send()
            .await
            .map_err(|_| ApiError::Unavailable)?;
        bounded_body(response, RESOURCE_BODY_LIMIT).await
    }
}

fn page_check(page: u32) -> Result<(), ApiError> {
    if !(1..=MAX_PAGE).contains(&page) {
        return Err(ApiError::InvalidContract);
    }
    Ok(())
}
fn collection_check(total: u32, length: usize) -> Result<(), ApiError> {
    if total > MAX_PAGE * PAGE_SIZE as u32 || length > PAGE_SIZE || length as u32 > total {
        return Err(ApiError::Oversized);
    }
    Ok(())
}

async fn bounded_body(
    mut response: Response,
    limit: usize,
) -> Result<Zeroizing<Vec<u8>>, ApiError> {
    let status = response.status().as_u16();
    if status != 200 {
        return Err(match status {
            401 => ApiError::Authentication,
            403 => ApiError::Permission,
            429 => ApiError::RateLimited,
            300..=399 => ApiError::InvalidContract,
            _ => ApiError::Unavailable,
        });
    }
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if !matches!(
        content_type.split(';').next(),
        Some("application/json" | "application/vnd.github+json")
    ) {
        return Err(ApiError::InvalidContract);
    }
    if response.content_length().is_some_and(|n| n > limit as u64) {
        return Err(ApiError::Oversized);
    }
    let mut bytes = Zeroizing::new(Vec::new());
    while let Some(chunk) = response.chunk().await.map_err(|_| ApiError::Unavailable)? {
        if bytes
            .len()
            .checked_add(chunk.len())
            .is_none_or(|n| n > limit)
        {
            return Err(ApiError::Oversized);
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn token_json() -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
        "access_token": "ghu_SYNTHETICNOTASECRET1234", "refresh_token": "ghr_SYNTHETICNOTASECRET5678",
        "expires_in":ACCESS_SECONDS,"refresh_token_expires_in":REFRESH_SECONDS,"scope":"","token_type":"bearer"
    })).unwrap()
    }

    #[test]
    fn expiring_device_contract_and_clock_are_closed() {
        let tokens = TokenSet::parse_json(&token_json(), 100).unwrap();
        assert!(tokens.access_at(100).is_ok());
        assert_eq!(
            tokens.access_at(99).err(),
            Some(ApiError::ReconnectRequired)
        );
        assert_eq!(
            tokens.access_at(100 + ACCESS_SECONDS).err(),
            Some(ApiError::Expired)
        );
        assert!(tokens.refresh_at(100 + ACCESS_SECONDS).is_ok());
        assert_eq!(
            tokens.refresh_at(100 + REFRESH_SECONDS).err(),
            Some(ApiError::ReconnectRequired)
        );
        assert!(TokenSet::parse_json(&token_json(), u64::MAX).is_err());
    }
    #[test]
    fn secrets_unknown_fields_nonexpiring_or_scope_tokens_are_rejected() {
        let fixture: serde_json::Value = serde_json::from_slice(&token_json()).unwrap();
        for (key, bad) in [
            (
                "access_token",
                serde_json::json!("ghp_SYNTHETICNOTASECRET1234"),
            ),
            ("token_type", serde_json::json!("Bearer")),
            ("scope", serde_json::json!("repo")),
            ("expires_in", serde_json::json!(0)),
            ("unexpected", serde_json::json!(true)),
        ] {
            let mut altered = fixture.clone();
            altered[key] = bad;
            assert!(TokenSet::parse_json(&serde_json::to_vec(&altered).unwrap(), 100).is_err());
        }
        assert!(TokenSet::parse_json(&vec![b'x'; AUTH_BODY_LIMIT + 1], 100).is_err());
    }
    #[test]
    fn fixed_client_and_api_contract_do_not_construct_live_authority() {
        assert_eq!(REGISTERED_APP_ID, 5_256_613);
        assert_eq!(API_VERSION, "2026-03-10");
        assert!(page_check(0).is_err());
        assert!(page_check(11).is_err());
        assert!(collection_check(1001, 1).is_err());
        assert!(collection_check(100, 101).is_err());
        assert!(!valid_login("../owner"));
        assert!(!valid_login("owner#fragment"));
    }

    #[test]
    fn remote_text_paths_refs_sha_and_lengths_fail_closed_without_network() {
        for path in [
            "../file",
            "/file",
            "a//b",
            ".git/config",
            ".env",
            "a/credentials",
            "id_rsa",
            "a/key.pem",
            "a%2fb",
            "a\\b",
            "a:stream",
        ] {
            assert!(!file_path(path), "{path}");
        }
        for branch in [
            "",
            "../main",
            "main..old",
            "main#x",
            "a.lock",
            "a//b",
            "main@{{1}}",
            "-bad?",
        ] {
            assert!(!branch_name(branch), "{branch}");
        }
        let make = || FileWire {
            object_type: "file".into(),
            path: "docs/test.md".into(),
            sha: "a".repeat(40),
            size: 9,
            encoding: "base64".into(),
            content: "c3ludGhldGlj\n".into(),
        };
        assert_eq!(
            file_from(make(), "docs/test.md", "test-branch")
                .unwrap()
                .content,
            "synthetic"
        );
        let mut bad = make();
        bad.path = "other.md".into();
        assert!(file_from(bad, "docs/test.md", "main").is_err());
        let mut bad = make();
        bad.size = 10;
        assert!(file_from(bad, "docs/test.md", "main").is_err());
        let mut bad = make();
        bad.sha = "A".repeat(40);
        assert!(file_from(bad, "docs/test.md", "main").is_err());
    }
}
