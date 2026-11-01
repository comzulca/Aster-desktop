//! Offline v0.2.4 GitHub contracts: SEC-032–034/036, TM-035–037, T-090–092/096.
//!
//! This module performs no I/O and is not an enabled connector. There are no
//! credentials, HTTP clients, browser launchers, IPC handlers or dispatch grants.
//! A registered dedicated App, native consent/custody, authenticated API evidence
//! and separately qualified host integration remain required. Syntax is not proof
//! of App registration, account identity, installation rights or human consent.
//!
//! Primary contracts checked 2026-10-09:
//! <https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/generating-a-user-access-token-for-a-github-app>
//! <https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/refreshing-user-access-tokens>
//! <https://docs.github.com/en/rest/pulls/pulls#merge-a-pull-request>
//! Refresh/rotation and all live routes are deliberately outside this slice.

use serde::{Deserialize, Deserializer, Serialize};
use zeroize::{Zeroize, Zeroizing};

pub const CONTRACT_VERSION: u32 = 1;
/// Owner-supplied public identifier, 10 October 2026; not a secret or auth proof.
/// Constructing it does not enable I/O, credentials, IPC or a connection.
pub const REGISTERED_CLIENT_ID: &str = "Iv23liKA8uYEYklVfzFF";
pub const AUTH_ORIGIN: &str = "https://github.com";
pub const RESOURCE_ORIGIN: &str = "https://api.github.com";
pub const DEVICE_CODE_ENDPOINT: &str = "https://github.com/login/device/code";
pub const DEVICE_TOKEN_ENDPOINT: &str = "https://github.com/login/oauth/access_token";
pub const VERIFICATION_URI: &str = "https://github.com/login/device";
pub const REMOTE_AUTHORISATIONS_URI: &str = "https://github.com/settings/apps";
pub const DEVICE_GRANT_TYPE: &str = "urn:ietf:params:oauth:grant-type:device_code";
pub const MIN_POLL_SECONDS: u64 = 5;
pub const MAX_DEVICE_SECONDS: u64 = 900;
pub const MAX_PROPOSAL_SECONDS: u64 = 120;
const MAX_DEVICE_RESPONSE_BYTES: usize = 4_096;
const MAX_PROPOSAL_BYTES: usize = 16_384;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractError {
    InvalidClientId,
    Oversized,
    Malformed,
    InvalidDeviceResponse,
    InvalidInterval,
    InvalidIdentity,
    InvalidResource,
    InvalidContext,
    Inactive,
    StaleGeneration,
    GenerationExhausted,
    NoPendingAccount,
    Expired,
    AlreadyUsed,
    ReviewMismatch,
    StaleResource,
    EvidenceUnavailable,
}

/// Public identifier syntax only; no Client Secret or endpoint override exists.
#[derive(Debug, PartialEq, Eq)]
pub struct AppClientId(String);

impl AppClientId {
    /// Fixed dedicated-App configuration only. Future qualified host requests
    /// must use this constructor, not an identifier supplied by the renderer.
    pub fn registered() -> Result<Self, ContractError> {
        Self::parse(REGISTERED_CLIENT_ID)
    }

    pub fn parse(value: &str) -> Result<Self, ContractError> {
        if !(8..=128).contains(&value.len())
            || !value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        {
            return Err(ContractError::InvalidClientId);
        }
        Ok(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeviceWire {
    #[serde(deserialize_with = "deserialize_private_code")]
    device_code: Zeroizing<String>,
    #[serde(deserialize_with = "deserialize_private_code")]
    user_code: Zeroizing<String>,
    verification_uri: String,
    expires_in: u64,
    interval: u64,
}

fn deserialize_private_code<'de, D>(deserializer: D) -> Result<Zeroizing<String>, D::Error>
where
    D: Deserializer<'de>,
{
    // Wrap each field immediately so later parse failure drops a protected field.
    String::deserialize(deserializer).map(Zeroizing::new)
}

/// No Debug, Clone, Serialize or device-code getter: the code is host-private.
pub struct DeviceResponse {
    device_code: Zeroizing<String>,
    user_code: Zeroizing<String>,
    expires_in: u64,
    interval: u64,
}

impl DeviceResponse {
    /// The caller owns and must protect/clear its input buffer as well.
    pub fn parse_json(bytes: &[u8]) -> Result<Self, ContractError> {
        if bytes.len() > MAX_DEVICE_RESPONSE_BYTES {
            return Err(ContractError::Oversized);
        }
        let wire: DeviceWire =
            serde_json::from_slice(bytes).map_err(|_| ContractError::Malformed)?;
        let user_bytes = wire.user_code.as_bytes();
        if wire.device_code.len() != 40
            || !wire.device_code.bytes().all(|b| b.is_ascii_alphanumeric())
            || user_bytes.len() != 9
            || user_bytes[4] != b'-'
            || !user_bytes
                .iter()
                .enumerate()
                .all(|(i, b)| i == 4 || b.is_ascii_uppercase() || b.is_ascii_digit())
            || wire.verification_uri != VERIFICATION_URI
            || wire.expires_in == 0
            || wire.expires_in > MAX_DEVICE_SECONDS
        {
            return Err(ContractError::InvalidDeviceResponse);
        }
        if wire.interval == 0 || wire.interval > MAX_DEVICE_SECONDS {
            return Err(ContractError::InvalidInterval);
        }
        Ok(Self {
            device_code: wire.device_code,
            user_code: wire.user_code,
            expires_in: wire.expires_in,
            interval: wire.interval.max(MIN_POLL_SECONDS),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceTerminal {
    /// Token custody and authenticated account verification are still required.
    AwaitingAccountVerification,
    Cancelled,
    Denied,
    Expired,
    StaleGeneration,
    FailedClosed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PollDecision {
    WaitUntil(u64),
    RequestReady,
    InFlight,
    Terminal(DeviceTerminal),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PollOutcome {
    AuthorizationPending,
    SlowDown {
        provider_interval: Option<u64>,
    },
    /// A host-only event after protected token handling, not token JSON or proof.
    TokenReceived,
    AccessDenied,
    Expired,
    /// Includes unknown transport/write outcomes: never automatically retry.
    FailedClosed,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PollErrorWire {
    error: String,
    error_description: Option<String>,
    error_uri: Option<String>,
    interval: Option<u64>,
}

/// Parses only bounded OAuth errors. Successful token parsing/custody is absent.
pub fn parse_poll_error(bytes: &[u8]) -> Result<PollOutcome, ContractError> {
    if bytes.len() > MAX_DEVICE_RESPONSE_BYTES {
        return Err(ContractError::Oversized);
    }
    let wire: PollErrorWire =
        serde_json::from_slice(bytes).map_err(|_| ContractError::Malformed)?;
    if wire.error.len() > 80
        || wire
            .error_description
            .as_ref()
            .is_some_and(|s| s.len() > 512)
        || wire.error_uri.as_ref().is_some_and(|s| s.len() > 256)
    {
        return Err(ContractError::Malformed);
    }
    // error_uri is inert data and is never returned, visited or used as a route.
    Ok(match wire.error.as_str() {
        "authorization_pending" if wire.interval.is_none() => PollOutcome::AuthorizationPending,
        "slow_down" => PollOutcome::SlowDown {
            provider_interval: wire.interval,
        },
        "access_denied" => PollOutcome::AccessDenied,
        "expired_token" | "token_expired" => PollOutcome::Expired,
        _ => PollOutcome::FailedClosed,
    })
}

/// Tick values are host-owned monotonic seconds, never renderer/provider clocks.
/// Only one poll can be in flight. Terminal states cannot be revived.
pub struct DeviceSession {
    device_code: Zeroizing<String>,
    user_code: Zeroizing<String>,
    generation: u64,
    expires_at: u64,
    interval: u64,
    next_poll_at: u64,
    last_tick: u64,
    in_flight: bool,
    terminal: Option<DeviceTerminal>,
}

impl DeviceSession {
    /// Expiry is conservatively bound to request start, while the first poll is
    /// never earlier than the minimum interval after receiving the code.
    pub(crate) fn new_received(
        response: DeviceResponse,
        generation: u64,
        request_started: u64,
        received_at: u64,
    ) -> Result<Self, ContractError> {
        if received_at < request_started {
            return Err(ContractError::InvalidContext);
        }
        let interval = response.interval;
        let mut session = Self::new(response, generation, request_started)?;
        if received_at >= session.expires_at {
            return Err(ContractError::Expired);
        }
        session.next_poll_at = received_at
            .checked_add(interval)
            .ok_or(ContractError::InvalidContext)?;
        session.last_tick = received_at;
        Ok(session)
    }

    pub fn new(response: DeviceResponse, generation: u64, now: u64) -> Result<Self, ContractError> {
        if generation == 0 {
            return Err(ContractError::InvalidContext);
        }
        Ok(Self {
            expires_at: now
                .checked_add(response.expires_in)
                .ok_or(ContractError::InvalidContext)?,
            next_poll_at: now
                .checked_add(response.interval)
                .ok_or(ContractError::InvalidContext)?,
            device_code: response.device_code,
            user_code: response.user_code,
            generation,
            interval: response.interval,
            last_tick: now,
            in_flight: false,
            terminal: None,
        })
    }

    /// Only the fixed verification URI and this user code may be displayed.
    pub fn user_code(&self) -> Option<&str> {
        self.terminal.is_none().then_some(self.user_code.as_str())
    }

    /// Native adapter only, and only after this session admitted one poll.
    /// Never return this value through IPC, diagnostics or provider content.
    pub(crate) fn device_code_for_inflight(&self) -> Option<&str> {
        (self.terminal.is_none() && self.in_flight).then_some(self.device_code.as_str())
    }

    pub(crate) fn next_poll_seconds(&self, now: u64) -> u64 {
        self.next_poll_at.saturating_sub(now)
    }

    pub fn begin_poll(&mut self, now: u64, generation: u64) -> PollDecision {
        if let Some(terminal) = self.check_tick(now, generation) {
            return PollDecision::Terminal(terminal);
        }
        if self.in_flight {
            return PollDecision::InFlight;
        }
        if now < self.next_poll_at {
            return PollDecision::WaitUntil(self.next_poll_at);
        }
        self.in_flight = true;
        PollDecision::RequestReady
    }

    pub fn observe_result(
        &mut self,
        outcome: PollOutcome,
        now: u64,
        generation: u64,
    ) -> PollDecision {
        if let Some(terminal) = self.check_tick(now, generation) {
            return PollDecision::Terminal(terminal);
        }
        if !self.in_flight {
            return PollDecision::Terminal(self.finish(DeviceTerminal::FailedClosed));
        }
        self.in_flight = false;
        match outcome {
            PollOutcome::AuthorizationPending => {}
            PollOutcome::SlowDown { provider_interval } => {
                let Some(increased) = self.interval.checked_add(5) else {
                    return PollDecision::Terminal(self.finish(DeviceTerminal::FailedClosed));
                };
                if provider_interval.is_some_and(|v| v == 0 || v > MAX_DEVICE_SECONDS) {
                    return PollDecision::Terminal(self.finish(DeviceTerminal::FailedClosed));
                }
                self.interval = increased.max(provider_interval.unwrap_or(increased));
                if self.interval > MAX_DEVICE_SECONDS {
                    return PollDecision::Terminal(self.finish(DeviceTerminal::FailedClosed));
                }
            }
            PollOutcome::TokenReceived => {
                return PollDecision::Terminal(
                    self.finish(DeviceTerminal::AwaitingAccountVerification),
                );
            }
            PollOutcome::AccessDenied => {
                return PollDecision::Terminal(self.finish(DeviceTerminal::Denied));
            }
            PollOutcome::Expired => {
                return PollDecision::Terminal(self.finish(DeviceTerminal::Expired));
            }
            PollOutcome::FailedClosed => {
                return PollDecision::Terminal(self.finish(DeviceTerminal::FailedClosed));
            }
        }
        let Some(next_poll_at) = now.checked_add(self.interval) else {
            return PollDecision::Terminal(self.finish(DeviceTerminal::FailedClosed));
        };
        self.next_poll_at = next_poll_at;
        PollDecision::WaitUntil(self.next_poll_at)
    }

    pub fn cancel(&mut self) -> DeviceTerminal {
        self.terminal
            .unwrap_or_else(|| self.finish(DeviceTerminal::Cancelled))
    }

    fn check_tick(&mut self, now: u64, generation: u64) -> Option<DeviceTerminal> {
        if let Some(terminal) = self.terminal {
            return Some(terminal);
        }
        let terminal = if generation != self.generation {
            Some(DeviceTerminal::StaleGeneration)
        } else if now < self.last_tick {
            Some(DeviceTerminal::FailedClosed)
        } else if now >= self.expires_at {
            Some(DeviceTerminal::Expired)
        } else {
            None
        };
        self.last_tick = now;
        terminal.map(|value| self.finish(value))
    }

    fn finish(&mut self, terminal: DeviceTerminal) -> DeviceTerminal {
        self.device_code.zeroize();
        self.user_code.zeroize();
        self.in_flight = false;
        self.terminal = Some(terminal);
        terminal
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RepositoryIdentity {
    installation_id: u64,
    repository_id: u64,
    owner: String,
    name: String,
}

impl RepositoryIdentity {
    pub(crate) fn installation_id(&self) -> u64 {
        self.installation_id
    }

    pub(crate) fn repository_id(&self) -> u64 {
        self.repository_id
    }

    pub(crate) fn owner(&self) -> &str {
        &self.owner
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    /// Syntax-only object; installed/selected scope must come from the host.
    pub fn new(
        installation_id: u64,
        repository_id: u64,
        owner: &str,
        name: &str,
    ) -> Result<Self, ContractError> {
        if installation_id == 0 || repository_id == 0 || !login(owner) || !repo_name(name) {
            return Err(ContractError::InvalidIdentity);
        }
        Ok(Self {
            installation_id,
            repository_id,
            owner: owner.to_owned(),
            name: name.to_owned(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AccountBinding {
    generation: u64,
    account_id: u64,
    account_login: String,
    repository: RepositoryIdentity,
}

/// Volatile metadata only. No persisted receipt can restore this state.
pub struct ConnectionState {
    generation: u64,
    exhausted: bool,
    enabled: bool,
    awaiting_account: bool,
    account: Option<(u64, String)>,
    repository: Option<RepositoryIdentity>,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self {
            generation: 1,
            exhausted: false,
            enabled: false,
            awaiting_account: false,
            account: None,
            repository: None,
        }
    }
}

impl ConnectionState {
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// An enable intent does not authenticate or activate a live connector.
    pub fn set_enabled(&mut self, enabled: bool) -> Result<u64, ContractError> {
        self.invalidate()?;
        self.enabled = enabled;
        Ok(self.generation)
    }

    /// Reconnect/account switch invalidates old callbacks before new work.
    pub fn begin_account_connection(&mut self) -> Result<u64, ContractError> {
        if !self.enabled || self.exhausted {
            return Err(ContractError::Inactive);
        }
        self.invalidate()?;
        self.awaiting_account = true;
        Ok(self.generation)
    }

    /// Input must be host-verified account evidence, never a renderer assertion.
    pub fn record_verified_account(
        &mut self,
        generation: u64,
        account_id: u64,
        account_login: &str,
    ) -> Result<(), ContractError> {
        self.check_generation(generation)?;
        if !self.awaiting_account {
            return Err(ContractError::NoPendingAccount);
        }
        if account_id == 0 || !login(account_login) {
            return Err(ContractError::InvalidIdentity);
        }
        self.account = Some((account_id, account_login.to_owned()));
        self.awaiting_account = false;
        Ok(())
    }

    /// Selection is accepted only after independent installed-scope verification.
    pub fn select_verified_repository(
        &mut self,
        generation: u64,
        repository: RepositoryIdentity,
    ) -> Result<u64, ContractError> {
        self.check_generation(generation)?;
        let account = self.account.take().ok_or(ContractError::Inactive)?;
        self.invalidate()?;
        self.account = Some(account);
        self.repository = Some(repository);
        Ok(self.generation)
    }

    pub fn binding(&self) -> Result<AccountBinding, ContractError> {
        self.check_generation(self.generation)?;
        let (account_id, account_login) = self.account.as_ref().ok_or(ContractError::Inactive)?;
        Ok(AccountBinding {
            generation: self.generation,
            account_id: *account_id,
            account_login: account_login.clone(),
            repository: self.repository.clone().ok_or(ContractError::Inactive)?,
        })
    }

    /// Local metadata invalidation only; this performs no token deletion/revocation.
    /// Future custody must forget Aster's tokens after invalidating pending work.
    pub fn disconnect(&mut self) -> Result<u64, ContractError> {
        self.enabled = false;
        self.invalidate()?;
        Ok(self.generation)
    }

    fn check_generation(&self, generation: u64) -> Result<(), ContractError> {
        if self.exhausted || !self.enabled {
            return Err(ContractError::Inactive);
        }
        if self.generation != generation {
            return Err(ContractError::StaleGeneration);
        }
        Ok(())
    }

    fn invalidate(&mut self) -> Result<(), ContractError> {
        self.account = None;
        self.repository = None;
        self.awaiting_account = false;
        let Some(next_generation) = self.generation.checked_add(1) else {
            self.exhausted = true;
            self.enabled = false;
            return Err(ContractError::GenerationExhausted);
        };
        self.generation = next_generation;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MergeMethod {
    Merge,
    Squash,
    Rebase,
}

/// Closed resource contract; no URL, scope, bypass, force-write or confirm field.
/// File/issue publishers and asynchronous/queued merge dispatch remain deferred.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProposalIntent {
    ReadPullRequest {
        pull_number: u64,
        expected_head: String,
        expected_base: String,
        base_ref: String,
    },
    MergePullRequest {
        pull_number: u64,
        expected_head: String,
        expected_base: String,
        base_ref: String,
        method: MergeMethod,
        commit_title: String,
        commit_message: String,
    },
}

impl ProposalIntent {
    pub fn parse_json(bytes: &[u8]) -> Result<Self, ContractError> {
        if bytes.len() > MAX_PROPOSAL_BYTES {
            return Err(ContractError::Oversized);
        }
        let intent: Self = serde_json::from_slice(bytes).map_err(|_| ContractError::Malformed)?;
        intent.validate()?;
        Ok(intent)
    }

    fn resource(&self) -> (u64, &str, &str, &str) {
        match self {
            Self::ReadPullRequest {
                pull_number,
                expected_head,
                expected_base,
                base_ref,
            }
            | Self::MergePullRequest {
                pull_number,
                expected_head,
                expected_base,
                base_ref,
                ..
            } => (*pull_number, expected_head, expected_base, base_ref),
        }
    }

    fn validate(&self) -> Result<(), ContractError> {
        let (number, head, base, base_ref) = self.resource();
        if number == 0 || number > i32::MAX as u64 || !sha(head) || !sha(base) || !branch(base_ref)
        {
            return Err(ContractError::InvalidResource);
        }
        if let Self::MergePullRequest {
            commit_title,
            commit_message,
            ..
        } = self
        {
            if !bounded_text(commit_title, 256, 1_024)
                || commit_title.is_empty()
                || commit_title.contains(['\r', '\n'])
                || !bounded_text(commit_message, 2_000, 8_000)
            {
                return Err(ContractError::InvalidResource);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct HostContext {
    conversation_id: u64,
    request_id: u64,
}

impl HostContext {
    pub fn new(conversation_id: u64, request_id: u64) -> Result<Self, ContractError> {
        if conversation_id == 0 || request_id == 0 {
            return Err(ContractError::InvalidContext);
        }
        Ok(Self {
            conversation_id,
            request_id,
        })
    }
}

/// Immutable exact review data, not a confirmation receipt or reusable grant.
/// Deliberately not Deserialize: restoring display cannot restore authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProposalDescription {
    contract_version: u32,
    proposal_id: u64,
    binding: AccountBinding,
    context: HostContext,
    intent: ProposalIntent,
    created_at: u64,
    expires_at: u64,
    maximum_uses: u8,
}

pub struct Proposal {
    description: ProposalDescription,
    consumed: bool,
}

/// Host-owned monotonic issue IDs; IDs and active proposals are never restored.
/// Invalid admission also burns its ID. There is no renderer-supplied proposal ID.
pub struct ProposalIssuer {
    next_id: u64,
}

impl Default for ProposalIssuer {
    fn default() -> Self {
        Self { next_id: 1 }
    }
}

impl ProposalIssuer {
    pub fn issue(
        &mut self,
        intent: ProposalIntent,
        connection: &ConnectionState,
        context: HostContext,
        now: u64,
    ) -> Result<Proposal, ContractError> {
        let proposal_id = self.next_id;
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or(ContractError::GenerationExhausted)?;
        Proposal::admit(proposal_id, intent, connection, context, now)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeEligibility {
    /// Requires fresh complete rules, reviews, all checks AND explicit project gates.
    /// This fixture value alone proves none of those remote facts.
    Ready,
    Unknown,
    ChecksPending,
    ChecksFailed,
    ReviewsMissing,
    RulesUnavailable,
    StrongerProjectGateUnsatisfied,
    QueueRequired,
    BypassRequired,
}

/// Host observation only; no deserialisable approval or model-supplied evidence.
pub struct PullRequestSnapshot {
    pub binding: AccountBinding,
    pub pull_number: u64,
    pub head: String,
    pub base: String,
    pub base_ref: String,
    pub observed_at: u64,
    pub open: bool,
    pub draft: bool,
    pub eligibility: MergeEligibility,
}

impl Proposal {
    fn admit(
        proposal_id: u64,
        intent: ProposalIntent,
        connection: &ConnectionState,
        context: HostContext,
        now: u64,
    ) -> Result<Self, ContractError> {
        if proposal_id == 0 {
            return Err(ContractError::InvalidContext);
        }
        intent.validate()?;
        Ok(Self {
            description: ProposalDescription {
                contract_version: CONTRACT_VERSION,
                proposal_id,
                binding: connection.binding()?,
                context,
                intent,
                created_at: now,
                expires_at: now
                    .checked_add(MAX_PROPOSAL_SECONDS)
                    .ok_or(ContractError::InvalidContext)?,
                maximum_uses: 1,
            },
            consumed: false,
        })
    }

    pub fn description(&self) -> &ProposalDescription {
        &self.description
    }

    /// Pure offline comparison/consumption, returning NO dispatch authority.
    /// The future native host must separately authenticate caller/human consent
    /// and re-read remote policy. Neither model nor renderer can confirm a write.
    /// Every attempted consumption burns the proposal, including uncertain/failed
    /// revalidation; another operation needs a fresh exact native review.
    pub fn consume_once(
        &mut self,
        reviewed: &ProposalDescription,
        connection: &ConnectionState,
        context: HostContext,
        snapshot: &PullRequestSnapshot,
        now: u64,
    ) -> Result<(), ContractError> {
        if self.consumed {
            return Err(ContractError::AlreadyUsed);
        }
        self.consumed = true;
        if now < self.description.created_at || now >= self.description.expires_at {
            return Err(ContractError::Expired);
        }
        if reviewed != &self.description || context != self.description.context {
            return Err(ContractError::ReviewMismatch);
        }
        let binding = connection.binding()?;
        if binding != self.description.binding || snapshot.binding != binding {
            return Err(ContractError::StaleGeneration);
        }
        let (number, head, base, base_ref) = self.description.intent.resource();
        if snapshot.pull_number != number
            || snapshot.head != head
            || snapshot.base != base
            || snapshot.base_ref != base_ref
        {
            return Err(ContractError::StaleResource);
        }
        if snapshot.observed_at != now {
            return Err(ContractError::EvidenceUnavailable);
        }
        if matches!(
            self.description.intent,
            ProposalIntent::MergePullRequest { .. }
        ) && (!snapshot.open
            || snapshot.draft
            || snapshot.eligibility != MergeEligibility::Ready)
        {
            return Err(ContractError::EvidenceUnavailable);
        }
        Ok(())
    }
}

/// A reported/queued result is never verified merge evidence or retry permission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeObservation {
    Pending,
    Enqueued,
    ReportedMerged,
    Failed,
    Uncertain,
}

impl MergeObservation {
    pub fn requires_independent_merge_verification(self) -> bool {
        matches!(
            self,
            Self::Pending | Self::Enqueued | Self::ReportedMerged | Self::Uncertain
        )
    }
}

fn login(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 39
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        && !value.starts_with('-')
        && !value.ends_with('-')
        && !value.contains("--")
}

fn repo_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 100
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        && value != "."
        && value != ".."
        && !value.ends_with(".git")
}

fn sha(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn branch(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"/_-.".contains(&b))
        && !value.starts_with(['/', '.'])
        && !value.ends_with(['/', '.'])
        && !value.contains("..")
        && !value.contains("//")
        && value
            .split('/')
            .all(|part| !part.starts_with('.') && !part.ends_with(".lock"))
}

fn bounded_text(value: &str, scalars: usize, bytes: usize) -> bool {
    value.len() <= bytes
        && value.chars().count() <= scalars
        && !value
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\t')
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEAD: &str = "1111111111111111111111111111111111111111";
    const BASE: &str = "2222222222222222222222222222222222222222";

    fn device_json(interval: u64, expires: u64) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "device_code": "0123456789012345678901234567890123456789",
            "user_code": "ABCD-EFGH",
            "verification_uri": VERIFICATION_URI,
            "expires_in": expires,
            "interval": interval
        }))
        .unwrap()
    }

    fn session(interval: u64, expires: u64) -> DeviceSession {
        DeviceSession::new(
            DeviceResponse::parse_json(&device_json(interval, expires)).unwrap(),
            7,
            100,
        )
        .unwrap()
    }

    fn connected() -> ConnectionState {
        let mut state = ConnectionState::default();
        state.set_enabled(true).unwrap();
        let generation = state.begin_account_connection().unwrap();
        state
            .record_verified_account(generation, 10, "synthetic-user")
            .unwrap();
        state
            .select_verified_repository(
                generation,
                RepositoryIdentity::new(20, 30, "synthetic-owner", "fixture-repo").unwrap(),
            )
            .unwrap();
        state
    }

    fn intent() -> ProposalIntent {
        ProposalIntent::MergePullRequest {
            pull_number: 42,
            expected_head: HEAD.to_owned(),
            expected_base: BASE.to_owned(),
            base_ref: "main".to_owned(),
            method: MergeMethod::Squash,
            commit_title: "Synthetic fixture".to_owned(),
            commit_message: "No remote operation.".to_owned(),
        }
    }

    fn context() -> HostContext {
        HostContext::new(50, 60).unwrap()
    }

    fn snapshot(state: &ConnectionState) -> PullRequestSnapshot {
        PullRequestSnapshot {
            binding: state.binding().unwrap(),
            pull_number: 42,
            head: HEAD.to_owned(),
            base: BASE.to_owned(),
            base_ref: "main".to_owned(),
            observed_at: 101,
            open: true,
            draft: false,
            eligibility: MergeEligibility::Ready,
        }
    }

    // T-090 / TM-035: no secret/App-ID/URI substitution or malformed framing.
    #[test]
    fn registered_client_id_preserves_case_without_enabling_connection() {
        let mut connection = ConnectionState::default();
        let generation = connection.generation();
        let client = AppClientId::registered().unwrap();
        assert_eq!(REGISTERED_CLIENT_ID, "Iv23liKA8uYEYklVfzFF");
        assert_eq!(client.as_str(), "Iv23liKA8uYEYklVfzFF");
        assert_eq!(client.as_str().len(), 20);
        assert_eq!(
            AppClientId::parse("5256613"),
            Err(ContractError::InvalidClientId)
        );
        assert_eq!(connection.generation(), generation);
        assert_eq!(connection.binding(), Err(ContractError::Inactive));
        assert_eq!(
            connection.begin_account_connection(),
            Err(ContractError::Inactive)
        );
    }

    #[test]
    fn fixed_routes_and_bounded_device_response() {
        assert_eq!(AUTH_ORIGIN, "https://github.com");
        assert_eq!(RESOURCE_ORIGIN, "https://api.github.com");
        assert_eq!(VERIFICATION_URI, "https://github.com/login/device");
        for value in [
            "",
            "12345",
            "client secret",
            "https://github.com",
            "Iv1.id\n",
        ] {
            assert!(AppClientId::parse(value).is_err());
        }
        assert!(AppClientId::parse("Iv1.synthetic-id").is_ok());
        let valid = device_json(5, 900);
        let mut value: serde_json::Value = serde_json::from_slice(&valid).unwrap();
        for uri in [
            "https://github.com/login/device?code=ABCD-EFGH",
            "https://github.com.evil.invalid/login/device",
            "http://github.com/login/device",
            "https://user@github.com/login/device",
        ] {
            value["verification_uri"] = serde_json::json!(uri);
            assert!(DeviceResponse::parse_json(&serde_json::to_vec(&value).unwrap()).is_err());
        }
        value["verification_uri"] = serde_json::json!(VERIFICATION_URI);
        value["client_secret"] = serde_json::json!("synthetic-unknown-field");
        assert!(DeviceResponse::parse_json(&serde_json::to_vec(&value).unwrap()).is_err());
        assert!(DeviceResponse::parse_json(&vec![b' '; 4_097]).is_err());
        assert!(DeviceResponse::parse_json(&device_json(0, 900)).is_err());
        assert!(DeviceResponse::parse_json(&device_json(5, 901)).is_err());
        assert!(DeviceResponse::parse_json(br#"{"device_code":"x","device_code":"y"}"#).is_err());
    }

    #[test]
    fn device_receive_latency_cannot_extend_expiry_or_advance_first_poll() {
        let response = DeviceResponse::parse_json(&device_json(5, 30)).unwrap();
        let mut session = DeviceSession::new_received(response, 7, 100, 110).unwrap();
        assert!(session.device_code_for_inflight().is_none());
        assert_eq!(session.begin_poll(110, 7), PollDecision::WaitUntil(115));
        assert_eq!(session.begin_poll(115, 7), PollDecision::RequestReady);
        assert!(session.device_code_for_inflight().is_some());
        assert_eq!(
            session.begin_poll(130, 7),
            PollDecision::Terminal(DeviceTerminal::Expired)
        );
        assert!(session.device_code_for_inflight().is_none());
        assert!(DeviceSession::new_received(
            DeviceResponse::parse_json(&device_json(5, 30)).unwrap(),
            7,
            100,
            130
        )
        .is_err());
        assert!(DeviceSession::new_received(
            DeviceResponse::parse_json(&device_json(5, 30)).unwrap(),
            7,
            100,
            99
        )
        .is_err());
    }

    #[test]
    fn provider_interval_minimum_and_slow_down_are_monotonic() {
        let mut flow = session(1, 900);
        assert_eq!(flow.begin_poll(104, 7), PollDecision::WaitUntil(105));
        assert_eq!(flow.begin_poll(105, 7), PollDecision::RequestReady);
        assert_eq!(flow.begin_poll(105, 7), PollDecision::InFlight);
        assert_eq!(
            flow.observe_result(
                PollOutcome::SlowDown {
                    provider_interval: Some(2)
                },
                105,
                7
            ),
            PollDecision::WaitUntil(115)
        );
        assert_eq!(flow.begin_poll(114, 7), PollDecision::WaitUntil(115));
        assert_eq!(flow.begin_poll(115, 7), PollDecision::RequestReady);
        assert_eq!(
            flow.observe_result(
                PollOutcome::SlowDown {
                    provider_interval: Some(30)
                },
                115,
                7
            ),
            PollDecision::WaitUntil(145)
        );
        assert_eq!(flow.begin_poll(145, 7), PollDecision::RequestReady);
        assert_eq!(
            flow.observe_result(PollOutcome::AuthorizationPending, 146, 7),
            PollDecision::WaitUntil(176)
        );
        let mut provider = session(17, 900);
        assert_eq!(provider.begin_poll(116, 7), PollDecision::WaitUntil(117));
    }

    #[test]
    fn expiry_cancel_generation_and_unknown_outcomes_are_terminal() {
        let mut expired = session(5, 20);
        assert_eq!(
            expired.begin_poll(120, 7),
            PollDecision::Terminal(DeviceTerminal::Expired)
        );
        assert!(expired.device_code.is_empty());
        assert!(expired.user_code().is_none());
        let mut cancelled = session(5, 900);
        assert_eq!(cancelled.cancel(), DeviceTerminal::Cancelled);
        assert_eq!(
            cancelled.observe_result(PollOutcome::TokenReceived, 105, 7),
            PollDecision::Terminal(DeviceTerminal::Cancelled)
        );
        let mut stale = session(5, 900);
        assert_eq!(
            stale.begin_poll(105, 8),
            PollDecision::Terminal(DeviceTerminal::StaleGeneration)
        );
        let mut backwards = session(5, 900);
        assert_eq!(
            backwards.begin_poll(99, 7),
            PollDecision::Terminal(DeviceTerminal::FailedClosed)
        );
        let mut unknown = session(5, 900);
        unknown.begin_poll(105, 7);
        assert_eq!(
            unknown.observe_result(
                parse_poll_error(br#"{"error":"future_retry"}"#).unwrap(),
                106,
                7
            ),
            PollDecision::Terminal(DeviceTerminal::FailedClosed)
        );
        assert_eq!(
            unknown.begin_poll(500, 7),
            PollDecision::Terminal(DeviceTerminal::FailedClosed)
        );
        assert!(parse_poll_error(br#"{"error":"slow_down","interval":"10"}"#).is_err());
        assert!(parse_poll_error(br#"{"access_token":"synthetic-not-a-token"}"#).is_err());
    }

    #[test]
    fn unsolicited_results_and_overflow_cannot_start_or_revive_a_flow() {
        let mut unsolicited = session(5, 900);
        assert_eq!(
            unsolicited.observe_result(PollOutcome::TokenReceived, 101, 7),
            PollDecision::Terminal(DeviceTerminal::FailedClosed)
        );
        let response = DeviceResponse::parse_json(&device_json(5, 900)).unwrap();
        assert!(DeviceSession::new(response, 7, u64::MAX).is_err());
        let mut accepted = session(5, 900);
        accepted.begin_poll(105, 7);
        assert_eq!(
            accepted.observe_result(PollOutcome::TokenReceived, 106, 7),
            PollDecision::Terminal(DeviceTerminal::AwaitingAccountVerification)
        );
        assert!(accepted.device_code.is_empty());
    }

    // T-090/096 / TM-035/036: disable/disconnect/switch and late callbacks.
    #[test]
    fn account_and_repository_changes_invalidate_pending_generations() {
        let mut state = connected();
        let old = state.generation();
        assert_eq!(
            state.record_verified_account(old, 99, "another-user"),
            Err(ContractError::NoPendingAccount)
        );
        state.set_enabled(false).unwrap();
        assert!(state.binding().is_err());
        assert!(state
            .record_verified_account(old, 99, "another-user")
            .is_err());
        state.set_enabled(true).unwrap();
        let pending = state.begin_account_connection().unwrap();
        assert_eq!(
            state.record_verified_account(old, 99, "another-user"),
            Err(ContractError::StaleGeneration)
        );
        state
            .record_verified_account(pending, 99, "another-user")
            .unwrap();
        state.disconnect().unwrap();
        assert!(state.binding().is_err());
        assert!(state
            .record_verified_account(pending, 99, "another-user")
            .is_err());
        let mut exhausted = connected();
        exhausted.generation = u64::MAX;
        assert_eq!(
            exhausted.disconnect(),
            Err(ContractError::GenerationExhausted)
        );
        assert!(exhausted.binding().is_err());
        assert!(exhausted.set_enabled(true).is_err());
    }

    // T-091/092 / TM-036/037: exact review plus one-use, no model commit.
    #[test]
    fn exact_offline_match_is_one_use_and_cannot_restore_an_active_grant() {
        let state = connected();
        let mut issuer = ProposalIssuer::default();
        let mut proposal = issuer.issue(intent(), &state, context(), 100).unwrap();
        let reviewed = proposal.description().clone();
        assert_eq!(
            proposal.consume_once(&reviewed, &state, context(), &snapshot(&state), 101),
            Ok(())
        );
        assert_eq!(
            proposal.consume_once(&reviewed, &state, context(), &snapshot(&state), 101),
            Err(ContractError::AlreadyUsed)
        );
        assert_eq!(reviewed.maximum_uses, 1);
        assert!(serde_json::to_string(&reviewed).unwrap().contains("squash"));
        let next = issuer.issue(intent(), &state, context(), 100).unwrap();
        assert_ne!(reviewed.proposal_id, next.description().proposal_id);
        issuer.next_id = u64::MAX;
        assert!(issuer.issue(intent(), &state, context(), 100).is_err());
        assert!(issuer.issue(intent(), &state, context(), 100).is_err());
        // There is no Deserialize implementation for a Proposal or description.
    }

    #[test]
    fn changed_method_message_context_and_expiry_deny_review() {
        let state = connected();
        let mut proposal = Proposal::admit(70, intent(), &state, context(), 100).unwrap();
        let mut changed = proposal.description().clone();
        if let ProposalIntent::MergePullRequest {
            method,
            commit_message,
            ..
        } = &mut changed.intent
        {
            *method = MergeMethod::Merge;
            commit_message.push_str(" changed");
        }
        assert_eq!(
            proposal.consume_once(&changed, &state, context(), &snapshot(&state), 101),
            Err(ContractError::ReviewMismatch)
        );
        assert_eq!(
            proposal.consume_once(&changed, &state, context(), &snapshot(&state), 101),
            Err(ContractError::AlreadyUsed)
        );
        let mut proposal = Proposal::admit(71, intent(), &state, context(), 100).unwrap();
        let reviewed = proposal.description().clone();
        assert_eq!(
            proposal.consume_once(
                &reviewed,
                &state,
                HostContext::new(50, 61).unwrap(),
                &snapshot(&state),
                101
            ),
            Err(ContractError::ReviewMismatch)
        );
        let mut proposal = Proposal::admit(72, intent(), &state, context(), 100).unwrap();
        let reviewed = proposal.description().clone();
        assert_eq!(
            proposal.consume_once(&reviewed, &state, context(), &snapshot(&state), 220),
            Err(ContractError::Expired)
        );
    }

    #[test]
    fn head_base_scope_or_account_drift_deny_the_exact_proposal() {
        for field in 0..5 {
            let mut state = connected();
            let mut proposal = Proposal::admit(70, intent(), &state, context(), 100).unwrap();
            let reviewed = proposal.description().clone();
            let mut current = snapshot(&state);
            match field {
                0 => current.head = BASE.to_owned(),
                1 => current.base = HEAD.to_owned(),
                2 => current.base_ref = "another-base".to_owned(),
                3 => current.binding.repository.repository_id += 1,
                _ => {
                    state.begin_account_connection().unwrap();
                }
            }
            assert!(proposal
                .consume_once(&reviewed, &state, context(), &current, 101)
                .is_err());
        }
    }

    #[test]
    fn missing_pending_failing_bypass_queue_or_stale_evidence_denies_merge() {
        let state = connected();
        for eligibility in [
            MergeEligibility::Unknown,
            MergeEligibility::ChecksPending,
            MergeEligibility::ChecksFailed,
            MergeEligibility::ReviewsMissing,
            MergeEligibility::RulesUnavailable,
            MergeEligibility::StrongerProjectGateUnsatisfied,
            MergeEligibility::QueueRequired,
            MergeEligibility::BypassRequired,
        ] {
            let mut proposal = Proposal::admit(70, intent(), &state, context(), 100).unwrap();
            let reviewed = proposal.description().clone();
            let mut current = snapshot(&state);
            current.eligibility = eligibility;
            assert_eq!(
                proposal.consume_once(&reviewed, &state, context(), &current, 101),
                Err(ContractError::EvidenceUnavailable)
            );
        }
        for field in 0..3 {
            let mut proposal = Proposal::admit(70, intent(), &state, context(), 100).unwrap();
            let reviewed = proposal.description().clone();
            let mut current = snapshot(&state);
            match field {
                0 => current.open = false,
                1 => current.draft = true,
                _ => current.observed_at = 100,
            }
            assert_eq!(
                proposal.consume_once(&reviewed, &state, context(), &current, 101),
                Err(ContractError::EvidenceUnavailable)
            );
        }
        for observed in [
            MergeObservation::Pending,
            MergeObservation::Enqueued,
            MergeObservation::ReportedMerged,
            MergeObservation::Uncertain,
        ] {
            assert!(observed.requires_independent_merge_verification());
        }
    }

    #[test]
    fn closed_intents_reject_unknown_privileges_and_invalid_resource_text() {
        let mut value = serde_json::to_value(intent()).unwrap();
        for (field, text) in [
            ("url", "https://evil.invalid"),
            ("confirmed", "true"),
            ("bypass_rules", "true"),
            ("client_secret", "synthetic"),
        ] {
            value[field] = serde_json::json!(text);
            assert!(ProposalIntent::parse_json(&serde_json::to_vec(&value).unwrap()).is_err());
            value.as_object_mut().unwrap().remove(field);
        }
        for base_ref in [
            "../main",
            "main.lock",
            "a/.hidden",
            "a//b",
            "https://github.com",
            "main\n",
        ] {
            value["base_ref"] = serde_json::json!(base_ref);
            assert!(ProposalIntent::parse_json(&serde_json::to_vec(&value).unwrap()).is_err());
        }
        value["base_ref"] = serde_json::json!("main");
        value["expected_head"] = serde_json::json!("unknown");
        assert!(ProposalIntent::parse_json(&serde_json::to_vec(&value).unwrap()).is_err());
        value["expected_head"] = serde_json::json!(HEAD);
        value["commit_message"] = serde_json::json!("x".repeat(8_001));
        assert!(ProposalIntent::parse_json(&serde_json::to_vec(&value).unwrap()).is_err());
        assert!(RepositoryIdentity::new(20, 30, "synthetic-owner", "../repo").is_err());
        assert!(ProposalIntent::parse_json(&vec![b' '; 16_385]).is_err());
    }

    #[test]
    fn reads_also_require_exact_single_use_review() {
        let state = connected();
        let read = ProposalIntent::ReadPullRequest {
            pull_number: 42,
            expected_head: HEAD.to_owned(),
            expected_base: BASE.to_owned(),
            base_ref: "main".to_owned(),
        };
        let mut proposal = Proposal::admit(70, read, &state, context(), 100).unwrap();
        let reviewed = proposal.description().clone();
        let mut current = snapshot(&state);
        current.eligibility = MergeEligibility::Unknown;
        assert_eq!(
            proposal.consume_once(&reviewed, &state, context(), &current, 101),
            Ok(())
        );
        assert_eq!(
            proposal.consume_once(&reviewed, &state, context(), &current, 101),
            Err(ContractError::AlreadyUsed)
        );
    }
}
