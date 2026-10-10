//! PRIVATE pure mutation core (T-091/092/096). This file has no IPC, HTTP,
//! credential, native dialogue or provider route. Root links this first slice
//! under cfg(test) only; runtime mutationsAvailable remains false.
//! Snapshots/review decisions are trusted-host seams, not wire authority.
#[path = "github_mutation_ledger.rs"]
mod github_mutation_ledger;
#[path = "github_mutation_validation.rs"]
mod github_mutation_validation;

use base64::{engine::general_purpose::STANDARD, Engine};
use github_mutation_ledger::{Ledger, RecordBackend, Terminal};
use github_mutation_validation::{validate_snapshot, Snapshot};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

const CONTRACT_VERSION: u8 = 1;
const APP_ID: u64 = 5_256_613;
const MAX_INTENT_BYTES: usize = 160 * 1024;
const MAX_REVIEW_BYTES: usize = 512 * 1024;
const MAX_PLAN_MS: u64 = 120_000;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Error {
    Malformed,
    Bounds,
    InvalidResource,
    Disabled,
    Permission,
    Conflict,
    GateUnavailable,
    ReviewMismatch,
    Expired,
    ClockRollback,
    Replay,
    Busy,
    Uncertainty,
    Ledger,
    SequenceExhausted,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Operation {
    CreateBranch,
    CreateFile,
    UpdateFile,
    CreateIssue,
    CommentIssue,
    CloseIssue,
    CreatePullRequest,
    MergePullRequest,
}

#[derive(Copy, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum CloseReason {
    Completed,
    NotPlanned,
}

/// No URL/method/header/token/approval field; duplicate or extra JSON fields deny.
#[derive(Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
enum Intent {
    CreateBranch {
        source_branch: String,
        expected_head: String,
        new_branch: String,
    },
    CreateFile {
        branch: String,
        expected_head: String,
        path: String,
        content: String,
        commit_title: String,
        commit_message: String,
    },
    UpdateFile {
        branch: String,
        expected_head: String,
        path: String,
        expected_blob: String,
        content: String,
        commit_title: String,
        commit_message: String,
    },
    CreateIssue {
        title: String,
        body: String,
    },
    CommentIssue {
        number: u64,
        expected_id: u64,
        expected_snapshot: String,
        body: String,
    },
    CloseIssue {
        number: u64,
        expected_id: u64,
        expected_snapshot: String,
        reason: CloseReason,
    },
    CreatePullRequest {
        head_branch: String,
        base_branch: String,
        expected_head: String,
        expected_base: String,
        title: String,
        body: String,
        draft: bool,
    },
    MergePullRequest {
        number: u64,
        expected_id: u64,
        head_branch: String,
        base_branch: String,
        expected_head: String,
        expected_base: String,
        // Ordinary merge only; squash/rebase/async/queue remain unsupported.
        commit_title: String,
        commit_message: String,
    },
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct IntentEnvelope {
    schema_version: u8,
    change: Intent,
}

impl Intent {
    fn parse(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.is_empty() || bytes.len() > MAX_INTENT_BYTES {
            return Err(Error::Bounds);
        }
        let envelope: IntentEnvelope =
            serde_json::from_slice(bytes).map_err(|_| Error::Malformed)?;
        if envelope.schema_version != CONTRACT_VERSION {
            return Err(Error::Malformed);
        }
        envelope.change.validate()?;
        Ok(envelope.change)
    }

    fn operation(&self) -> Operation {
        match self {
            Self::CreateBranch { .. } => Operation::CreateBranch,
            Self::CreateFile { .. } => Operation::CreateFile,
            Self::UpdateFile { .. } => Operation::UpdateFile,
            Self::CreateIssue { .. } => Operation::CreateIssue,
            Self::CommentIssue { .. } => Operation::CommentIssue,
            Self::CloseIssue { .. } => Operation::CloseIssue,
            Self::CreatePullRequest { .. } => Operation::CreatePullRequest,
            Self::MergePullRequest { .. } => Operation::MergePullRequest,
        }
    }

    fn validate(&self) -> Result<(), Error> {
        use Intent::*;
        let invalid = match self {
            CreateBranch {
                source_branch,
                expected_head,
                new_branch,
            } => {
                !branch(source_branch)
                    || !sha(expected_head)
                    || !feature_branch(new_branch)
                    || source_branch == new_branch
            }
            CreateFile {
                branch: target,
                expected_head,
                path: target_path,
                content,
                commit_title,
                commit_message,
            }
            | UpdateFile {
                branch: target,
                expected_head,
                path: target_path,
                content,
                commit_title,
                commit_message,
                ..
            } => {
                !branch(target)
                    || !sha(expected_head)
                    || !path(target_path)
                    || !text(content, 65_536, 65_536, true)
                    || !commit_text(commit_title, commit_message)
                    || matches!(self, UpdateFile { expected_blob, .. } if !sha(expected_blob))
            }
            CreateIssue { title, body } => !title_body(title, body),
            CommentIssue {
                number: n,
                expected_id,
                expected_snapshot,
                body,
            } => {
                !number(*n)
                    || !id(*expected_id)
                    || !digest(expected_snapshot)
                    || body.trim().is_empty()
                    || !text(body, 4_000, 16_000, true)
            }
            CloseIssue {
                number: n,
                expected_id,
                expected_snapshot,
                ..
            } => !number(*n) || !id(*expected_id) || !digest(expected_snapshot),
            CreatePullRequest {
                head_branch,
                base_branch,
                expected_head,
                expected_base,
                title,
                body,
                ..
            } => {
                !branch(head_branch)
                    || !branch(base_branch)
                    || head_branch == base_branch
                    || !sha(expected_head)
                    || !sha(expected_base)
                    || !title_body(title, body)
            }
            MergePullRequest {
                number: n,
                expected_id,
                head_branch,
                base_branch,
                expected_head,
                expected_base,
                commit_title,
                commit_message,
            } => {
                !number(*n)
                    || !id(*expected_id)
                    || !branch(head_branch)
                    || !branch(base_branch)
                    || head_branch == base_branch
                    || !sha(expected_head)
                    || !sha(expected_base)
                    || !commit_text(commit_title, commit_message)
            }
        };
        if invalid {
            Err(Error::InvalidResource)
        } else {
            Ok(())
        }
    }

    fn resource(&self) -> Resource {
        match self {
            Self::CreateBranch { new_branch, .. } => Resource::Branch {
                name: new_branch.clone(),
            },
            Self::CreateFile { branch, path, .. } | Self::UpdateFile { branch, path, .. } => {
                Resource::File {
                    branch: branch.clone(),
                    path: path.clone(),
                }
            }
            Self::CreateIssue { .. } => Resource::NewIssue,
            Self::CommentIssue {
                number,
                expected_id,
                ..
            }
            | Self::CloseIssue {
                number,
                expected_id,
                ..
            } => Resource::Issue {
                number: *number,
                id: *expected_id,
            },
            Self::CreatePullRequest {
                head_branch,
                base_branch,
                ..
            } => Resource::NewPull {
                head: head_branch.clone(),
                base: base_branch.clone(),
            },
            Self::MergePullRequest {
                number,
                expected_id,
                ..
            } => Resource::Pull {
                number: *number,
                id: *expected_id,
            },
        }
    }

    fn request(&self) -> Result<FixedRequest, Error> {
        // These are operation-specific payloads, not a caller-provided JSON body.
        let value = match self {
            Self::CreateBranch {
                expected_head,
                new_branch,
                ..
            } => {
                serde_json::json!({"ref":format!("refs/heads/{new_branch}"),"sha":expected_head})
            }
            Self::CreateFile {
                branch,
                path: _,
                content,
                commit_title,
                commit_message,
                ..
            } => {
                serde_json::json!({"branch":branch,"content":STANDARD.encode(content.as_bytes()),"message":commit_message_text(commit_title,commit_message)})
            }
            Self::UpdateFile {
                branch,
                content,
                expected_blob,
                commit_title,
                commit_message,
                ..
            } => {
                serde_json::json!({"branch":branch,"content":STANDARD.encode(content.as_bytes()),"message":commit_message_text(commit_title,commit_message),"sha":expected_blob})
            }
            Self::CreateIssue { title, body } => serde_json::json!({"title":title,"body":body}),
            Self::CommentIssue { body, .. } => serde_json::json!({"body":body}),
            Self::CloseIssue { reason, .. } => {
                serde_json::json!({"state":"closed","state_reason":reason})
            }
            Self::CreatePullRequest {
                head_branch,
                base_branch,
                title,
                body,
                draft,
                ..
            } => {
                serde_json::json!({"head":head_branch,"base":base_branch,"title":title,"body":body,"draft":draft})
            }
            Self::MergePullRequest {
                expected_head,
                commit_title,
                commit_message,
                ..
            } => {
                serde_json::json!({"sha":expected_head,"merge_method":"merge","commit_title":commit_title,"commit_message":commit_message})
            }
        };
        Ok(FixedRequest {
            operation: self.operation(),
            resource: self.resource(),
            payload: Zeroizing::new(serde_json::to_vec(&value).map_err(|_| Error::Malformed)?),
        })
    }
}

fn commit_message_text(title: &str, message: &str) -> String {
    if message.is_empty() {
        title.to_owned()
    } else {
        format!("{title}\n\n{message}")
    }
}

fn text(value: &str, scalars: usize, bytes: usize, multiline: bool) -> bool {
    value.len() <= bytes && value.chars().count() <= scalars
        && !value.chars().any(|c| (c.is_control() && !(multiline && matches!(c, '\r' | '\n' | '\t')))
            || matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}' | '\u{200b}' | '\u{feff}'))
}
fn title_body(title: &str, body: &str) -> bool {
    !title.trim().is_empty() && text(title, 256, 1_024, false) && text(body, 4_000, 16_000, true)
}
fn commit_text(title: &str, message: &str) -> bool {
    !title.trim().is_empty() && text(title, 256, 1_024, false) && text(message, 2_000, 8_000, true)
}
fn id(value: u64) -> bool {
    value > 0 && value <= i64::MAX as u64
}
fn number(value: u64) -> bool {
    value > 0 && value <= i32::MAX as u64
}
fn hex(value: &str, size: usize) -> bool {
    value.len() == size
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn sha(value: &str) -> bool {
    hex(value, 40)
}
fn digest(value: &str) -> bool {
    hex(value, 64)
}
fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
fn branch(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value != "HEAD"
        && !value.starts_with("refs/")
        && value.split('/').all(|part| {
            !part.is_empty()
                && !part.starts_with(['.', '-'])
                && !part.ends_with('.')
                && !part.ends_with(".lock")
                && !part.contains("..")
                && part
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        })
}
fn feature_branch(value: &str) -> bool {
    branch(value)
        && value
            .strip_prefix("aster/")
            .is_some_and(|name| !name.contains('/') && name.len() <= 64)
}
fn path(value: &str) -> bool {
    if value.is_empty() || value.len() > 512 || value.starts_with('/') || value.ends_with('/') {
        return false;
    }
    value.split('/').all(|part| {
        let lower = part.to_ascii_lowercase();
        let stem = lower.split('.').next().unwrap_or_default();
        !part.is_empty()
            && part.len() <= 128
            && !part.ends_with('.')
            && !part.contains("..")
            && part
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
            && !matches!(
                lower.as_str(),
                ".git"
                    | ".github"
                    | ".ssh"
                    | ".aws"
                    | ".azure"
                    | ".codex"
                    | ".agents"
                    | ".npmrc"
                    | ".pypirc"
                    | "credentials"
                    | "id_rsa"
                    | "id_ed25519"
            )
            && !lower.starts_with(".env")
            && !lower.starts_with("credentials.")
            && !lower.ends_with(".pem")
            && !lower.ends_with(".key")
            && !lower.ends_with(".pfx")
            && !matches!(stem, "con" | "prn" | "aux" | "nul")
            && !(stem.len() == 4
                && (stem.starts_with("com") || stem.starts_with("lpt"))
                && matches!(stem.as_bytes()[3], b'1'..=b'9'))
    })
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Resource {
    Branch { name: String },
    File { branch: String, path: String },
    NewIssue,
    Issue { number: u64, id: u64 },
    NewPull { head: String, base: String },
    Pull { number: u64, id: u64 },
}
impl Resource {
    fn valid(&self) -> bool {
        match self {
            Self::Branch { name } => feature_branch(name),
            Self::File {
                branch: target,
                path: target_path,
            } => branch(target) && path(target_path),
            Self::NewIssue => true,
            Self::Issue {
                number: n,
                id: identity,
            }
            | Self::Pull {
                number: n,
                id: identity,
            } => number(*n) && id(*identity),
            Self::NewPull { head, base } => branch(head) && branch(base) && head != base,
        }
    }
}

/// Only metadata is serializable for encrypted ledger records. No account token.
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Binding {
    generation: u64,
    account_id: u64,
    account_login: String,
    app_id: u64,
    installation_id: u64,
    repository_id: u64,
    owner: String,
    name: String,
    private: bool,
}
impl Binding {
    fn valid(&self) -> bool {
        let login = |v: &str| {
            !v.is_empty()
                && v.len() <= 39
                && !v.starts_with('-')
                && !v.ends_with('-')
                && !v.contains("--")
                && v.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        };
        self.generation > 0
            && id(self.account_id)
            && self.app_id == APP_ID
            && id(self.installation_id)
            && id(self.repository_id)
            && login(&self.account_login)
            && login(&self.owner)
            && !self.name.is_empty()
            && self.name.len() <= 100
            && self.name != "."
            && self.name != ".."
            && !self.name.ends_with(".git")
            && self
                .name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
    }
    fn aster_project(&self) -> bool {
        self.owner.eq_ignore_ascii_case("comzulca")
            && self.name.eq_ignore_ascii_case("Aster-desktop")
    }
}

struct FixedRequest {
    operation: Operation,
    resource: Resource,
    payload: Zeroizing<Vec<u8>>,
}

/// No Clone/Deserialize/Debug: full private review data cannot be restored as a grant.
struct Plan {
    id: String,
    epoch: u64,
    issued_ms: u64,
    expires_ms: u64,
    intent: Intent,
    snapshot: Snapshot,
    request: FixedRequest,
    review: Zeroizing<Vec<u8>>,
    review_hash: String,
}
impl Plan {
    fn canonical_review(&self) -> Result<Zeroizing<Vec<u8>>, Error> {
        Ok(Zeroizing::new(serde_json::to_vec(&serde_json::json!({
            "contractVersion":CONTRACT_VERSION,"planId":self.id,"epoch":self.epoch,
            "issuedMs":self.issued_ms,"expiresMs":self.expires_ms,"maximumUses":1,"maximumMutations":1,
            "destination":"api.github.com","binding":self.snapshot.binding,
            "intent":self.intent,"snapshot":self.snapshot,"exactPayload":std::str::from_utf8(&self.request.payload).map_err(|_| Error::Malformed)?,
            "guardLimitations":"REST host preconditions are not universal server CAS; a received request cannot be undone."
        })).map_err(|_| Error::Malformed)?))
    }
}
struct ReviewTicket {
    plan: Plan,
}
struct DispatchPermit {
    plan: Plan,
}

#[derive(Clone)]
struct Runtime {
    enabled: bool,
    token_live: bool,
    binding: Option<Binding>,
}

/// The opaque boot nonce must come from the future trusted host's OS RNG, never
/// IPC. Tests provide a synthetic nonce; no random/credential/native I/O here.
struct PlanBook {
    boot_nonce: String,
    next_id: u64,
    epoch: u64,
    last_ms: u64,
    pending: Option<Plan>,
    active_id: Option<String>,
    exhausted: bool,
}
impl PlanBook {
    fn new(boot_nonce: &str) -> Result<Self, Error> {
        if !hex(boot_nonce, 32) {
            return Err(Error::InvalidResource);
        }
        Ok(Self {
            boot_nonce: boot_nonce.to_owned(),
            next_id: 1,
            epoch: 1,
            last_ms: 0,
            pending: None,
            active_id: None,
            exhausted: false,
        })
    }
    fn tick(&mut self, now_ms: u64) -> Result<(), Error> {
        if self.exhausted {
            return Err(Error::SequenceExhausted);
        }
        if now_ms < self.last_ms {
            self.invalidate()?;
            return Err(Error::ClockRollback);
        }
        self.last_ms = now_ms;
        Ok(())
    }
    fn invalidate(&mut self) -> Result<(), Error> {
        self.pending = None;
        let Some(next_epoch) = self.epoch.checked_add(1) else {
            self.exhausted = true;
            return Err(Error::SequenceExhausted);
        };
        self.epoch = next_epoch;
        // An existing review/dispatch remains single-flight until its owner
        // drains it; no cancellation can release an actual native guard early.
        Ok(())
    }
    fn prepare<B: RecordBackend>(
        &mut self,
        intent: Intent,
        snapshot: Snapshot,
        ledger: &Ledger<B>,
        now_ms: u64,
    ) -> Result<String, Error> {
        self.tick(now_ms)?;
        if self.active_id.is_some() {
            return Err(Error::Busy);
        }
        self.pending = None; // Replacing or failed preparation burns older plans.
        intent.validate()?;
        validate_snapshot(&intent, &snapshot)?;
        ledger.admit(&snapshot.binding)?;
        let sequence = self.next_id;
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or(Error::SequenceExhausted)?;
        let id = format!("{}-{sequence}", self.boot_nonce);
        let expires_ms = now_ms.checked_add(MAX_PLAN_MS).ok_or(Error::Expired)?;
        let request = intent.request()?;
        let mut plan = Plan {
            id: id.clone(),
            epoch: self.epoch,
            issued_ms: now_ms,
            expires_ms,
            intent,
            snapshot,
            request,
            review: Zeroizing::new(Vec::new()),
            review_hash: String::new(),
        };
        plan.review = plan.canonical_review()?;
        if plan.review.len() > MAX_REVIEW_BYTES {
            return Err(Error::Bounds);
        }
        plan.review_hash = hash(&plan.review);
        self.pending = Some(plan);
        Ok(id)
    }
    fn begin_review(&mut self, plan_id: &str, now_ms: u64) -> Result<ReviewTicket, Error> {
        self.tick(now_ms)?;
        if self.active_id.is_some() {
            return Err(Error::Busy);
        }
        let plan = self.pending.take().ok_or(Error::Replay)?;
        if plan.id != plan_id {
            return Err(Error::ReviewMismatch);
        }
        if now_ms >= plan.expires_ms || now_ms < plan.issued_ms {
            return Err(Error::Expired);
        }
        self.active_id = Some(plan.id.clone());
        Ok(ReviewTicket { plan })
    }
    fn finish_review(
        &mut self,
        ticket: ReviewTicket,
        native_review_hash: Option<&str>,
        fresh: Snapshot,
        now_ms: u64,
    ) -> Result<DispatchPermit, Error> {
        let plan = ticket.plan;
        let result = self.tick(now_ms).and_then(|()| {
            if self.active_id.as_ref() != Some(&plan.id) || self.epoch != plan.epoch {
                return Err(Error::Replay);
            }
            if now_ms >= plan.expires_ms || now_ms < plan.issued_ms {
                return Err(Error::Expired);
            }
            if native_review_hash != Some(plan.review_hash.as_str())
                || hash(&plan.review) != plan.review_hash
                || plan.canonical_review()?.as_slice() != plan.review.as_slice()
            {
                return Err(Error::ReviewMismatch);
            }
            validate_snapshot(&plan.intent, &fresh)?;
            if fresh != plan.snapshot
                || plan.request.operation != plan.intent.operation()
                || plan.request.resource != plan.intent.resource()
                || hash(&plan.request.payload) != hash(&plan.intent.request()?.payload)
            {
                return Err(Error::Conflict);
            }
            Ok(())
        });
        if let Err(error) = result {
            self.active_id = None;
            return Err(error);
        }
        Ok(DispatchPermit { plan })
    }
    fn live(&mut self, plan: &Plan, runtime: &Runtime, now_ms: u64) -> Result<(), Error> {
        self.tick(now_ms)?;
        if !runtime.enabled || !runtime.token_live {
            return Err(Error::Disabled);
        }
        if self.active_id.as_ref() != Some(&plan.id) || plan.epoch != self.epoch {
            return Err(Error::Replay);
        }
        if now_ms >= plan.expires_ms || now_ms < plan.issued_ms {
            return Err(Error::Expired);
        }
        if runtime.binding.as_ref() != Some(&plan.snapshot.binding) {
            return Err(Error::Conflict);
        }
        Ok(())
    }
}

/// This seam has no real implementation. Exact adapters/readback/native consent
/// require root review before any future runtime linking or route activation.
trait FakeableTransport {
    fn send_one(&mut self, request: &FixedRequest) -> Observation;
}
trait Checkpoints {
    fn current(&mut self) -> (Runtime, u64);
}
enum Observation {
    ProvablyNotStarted,
    DefinitiveNoChange,
    Uncertain,
    Readback(Box<github_mutation_validation::Readback>),
}

impl PlanBook {
    fn dispatch_once<B: RecordBackend, T: FakeableTransport, C: Checkpoints>(
        &mut self,
        permit: DispatchPermit,
        ledger: &mut Ledger<B>,
        transport: &mut T,
        checkpoints: &mut C,
    ) -> Result<Terminal, Error> {
        let plan = permit.plan;
        let (runtime, now) = checkpoints.current();
        if let Err(error) = self.live(&plan, &runtime, now) {
            self.active_id = None;
            return Err(error);
        }
        let operation_id = match ledger.begin(&plan, now) {
            Ok(value) => value,
            Err(error) => {
                self.active_id = None;
                return Err(error);
            }
        };
        let (runtime, now) = checkpoints.current();
        let terminal = if self.live(&plan, &runtime, now).is_err() {
            Terminal::CancelledBeforeDispatch
        } else {
            // Exactly one remote-change seam; no retry, rollback or follow-up.
            let result = transport.send_one(&plan.request);
            let (runtime, now) = checkpoints.current();
            if self.live(&plan, &runtime, now).is_err() {
                Terminal::AttentionRequired
            } else {
                match result {
                    Observation::ProvablyNotStarted => Terminal::CancelledBeforeDispatch,
                    Observation::DefinitiveNoChange => Terminal::RejectedNoChange,
                    Observation::Uncertain => Terminal::AttentionRequired,
                    Observation::Readback(readback) => {
                        github_mutation_validation::verify_readback(&plan, readback.as_ref())
                            .map(Terminal::Verified)
                            .unwrap_or(Terminal::AttentionRequired)
                    }
                }
            }
        };
        self.active_id = None;
        match ledger.finish(&operation_id, terminal.clone()) {
            Ok(()) => Ok(terminal),
            Err(_) => Ok(Terminal::AttentionRequired),
        }
    }
}

#[cfg(test)]
#[path = "github_mutation_tests.rs"]
mod tests;
