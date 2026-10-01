//! Host-owned, bounded project access. No renderer value is an approval.
//!
//! A native folder dialog supplies the root. The separate native file grant,
//! read and proposal methods below are candidate internals: no IPC command
//! may expose project plaintext to the WebView until a native full-content
//! viewer and disclosure gate exist. An internal read consumes its one-use
//! selected-file grant; disclosure and proposals have separate one-use rights
//! on its exact ephemeral snapshot.
//!
//! File publication is deliberately unavailable. The Windows crash journal,
//! native full-diff approval surface and adversarial NTFS tests in ADR-0003
//! have not yet passed. A hash check followed by a rename is not a safe CAS.

use std::collections::HashMap;
#[cfg(windows)]
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;
#[cfg(windows)]
use std::time::Duration;
use std::time::Instant;

use serde::{Deserialize, Serialize};
#[cfg(windows)]
use sha2::{Digest, Sha256};

const MAX_READ_BYTES: u64 = 1024 * 1024;
const MAX_CHANGED_BYTES: usize = 256 * 1024;
const MAX_PATH_BYTES: usize = 4096;
const MAX_ID_BYTES: usize = 128;
const MAX_PROPOSAL_BYTES: usize = 2 * MAX_READ_BYTES as usize + 2 * MAX_PATH_BYTES + 256;
#[cfg(windows)]
const MAX_PENDING: usize = 4;
#[cfg(windows)]
const MAX_DIFF_BYTES: usize = 5 * 1024 * 1024;
#[cfg(windows)]
const GRANT_LIFETIME: Duration = Duration::from_secs(300);

#[cfg_attr(not(windows), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceError {
    Unavailable,
    Denied,
    Conflict,
    TooLarge,
    Io,
    AttentionRequired,
}

impl WorkspaceError {
    pub fn code(self) -> &'static str {
        match self {
            Self::Unavailable => "WORKSPACE_UNAVAILABLE",
            Self::Denied => "WORKSPACE_DENIED",
            Self::Conflict => "WORKSPACE_CONFLICT",
            Self::TooLarge => "WORKSPACE_LIMIT_EXCEEDED",
            Self::Io => "WORKSPACE_IO_ERROR",
            Self::AttentionRequired => "WORKSPACE_ATTENTION_REQUIRED",
        }
    }
}

impl std::fmt::Display for WorkspaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.code())
    }
}

impl std::error::Error for WorkspaceError {}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RootSelection {
    pub selected: bool,
    pub label: String,
    pub read_available: bool,
    pub apply_available: bool,
}

#[allow(dead_code)] // File read IPC is gated pending native full-content review.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSelection {
    pub relative_path: String,
    pub request_id: String,
}

#[allow(dead_code)] // Selected file plaintext must not cross into the WebView.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRead {
    pub relative_path: String,
    pub content: String,
    pub sha256: String,
}

#[allow(dead_code)] // Proposal IPC is gated pending native review and publication proof.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposalReview {
    pub status: String,
    pub operation: String,
    pub relative_path: String,
    pub old_sha256: Option<String>,
    pub new_sha256: String,
    /// JSON-encoded exact old/new UTF-8 contents; never truncate a diff.
    pub diff: String,
    pub proposal_id: String,
    pub apply_available: bool,
}

/// Model output is untrusted data. Parsing this closed object never creates
/// a grant, confirms a native review, or makes a file operation available.
#[allow(dead_code)] // The shipping model/proposal IPC remains disabled.
#[derive(Debug, Copy, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProposalOperation {
    ReplaceExisting,
    CreateNew,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProposedEdit {
    pub version: u32,
    pub operation: ProposalOperation,
    pub relative_path: String,
    pub expected_old_sha256: Option<String>,
    pub new_content: String,
}

#[allow(dead_code)]
pub fn parse_proposed_edit(json: &str) -> Result<ProposedEdit, WorkspaceError> {
    if json.len() > MAX_PROPOSAL_BYTES {
        return Err(WorkspaceError::TooLarge);
    }
    // serde rejects duplicate fields, unknown fields, arrays, surrounding
    // prose, concatenated objects and trailing non-whitespace data.
    let mut edit: ProposedEdit = serde_json::from_str(json).map_err(|_| WorkspaceError::Denied)?;
    if edit.version != 1 {
        return Err(WorkspaceError::Denied);
    }
    edit.relative_path = join_relative(&parse_relative_path(&edit.relative_path)?);
    match edit.operation {
        ProposalOperation::ReplaceExisting => {
            let digest = edit
                .expected_old_sha256
                .as_deref()
                .ok_or(WorkspaceError::Denied)?;
            if digest.len() != 64
                || !digest
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return Err(WorkspaceError::Denied);
            }
            validate_text(&edit.new_content, MAX_READ_BYTES)?;
        }
        ProposalOperation::CreateNew => {
            if edit.expected_old_sha256.is_some() {
                return Err(WorkspaceError::Denied);
            }
            validate_text(&edit.new_content, MAX_CHANGED_BYTES as u64)?;
        }
    }
    Ok(edit)
}

#[derive(Default)]
pub struct Workspace {
    state: Mutex<State>,
}

#[derive(Default)]
struct State {
    root: Option<Root>,
    epoch: u64,
    read_grants: HashMap<(String, String), ReadGrant>,
    snapshots: HashMap<(String, String), Snapshot>,
    proposals: HashMap<String, Proposal>,
    #[cfg(windows)]
    creations: HashMap<String, PendingCreation>,
}

#[cfg_attr(not(windows), allow(dead_code))]
struct Root {
    path: PathBuf,
    label: String,
    #[cfg(windows)]
    anchor: std::fs::File,
    #[cfg(windows)]
    id: windows_fs::FileIdentity,
    // Pin every checked ancestor for the lifetime of the selected root. A
    // root handle alone does not establish the stability of its path chain.
    #[cfg(windows)]
    ancestor_pins: Vec<std::fs::File>,
}

#[allow(dead_code)] // Read-grant service remains a Windows-only gated candidate.
struct ReadGrant {
    relative_path: String,
    epoch: u64,
    deadline: Instant,
    #[cfg(windows)]
    id: windows_fs::FileIdentity,
    sha256: String,
}

#[allow(dead_code)] // Snapshots remain host-only while disclosure is unavailable.
struct Snapshot {
    read: ProjectRead,
    epoch: u64,
    deadline: Instant,
    disclosure_remaining: bool,
    dispatch_remaining: bool,
    #[cfg(windows)]
    prepared_disclosure: Option<PreparedDisclosure>,
    proposal_remaining: bool,
    #[cfg(windows)]
    id: windows_fs::FileIdentity,
}

#[cfg(windows)]
struct PreparedDisclosure {
    exact_outbound_body: Vec<u8>,
    review_binding: String,
}

#[allow(dead_code)]
struct Proposal {
    conversation_id: String,
    request_id: String,
    relative_path: String,
    root_epoch: u64,
    old_sha256: String,
    new_sha256: String,
    complete_diff: String,
    deadline: Instant,
    remaining_uses: u8,
    #[cfg(windows)]
    old_id: windows_fs::FileIdentity,
}

#[cfg(windows)]
#[allow(dead_code)]
struct PendingCreation {
    conversation_id: String,
    request_id: String,
    relative_path: String,
    root_epoch: u64,
    root_id: windows_fs::FileIdentity,
    parent_id: windows_fs::FileIdentity,
    new_content: String,
    new_sha256: String,
    complete_diff: String,
    review_binding: String,
    journal_root: Root,
    deadline: Instant,
}

#[cfg(windows)]
#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CreationOutcome {
    Verified,
    AttentionRequired,
}

#[cfg(windows)]
#[derive(Copy, Clone, PartialEq, Eq)]
enum CreationStage {
    Intent,
    Created,
    IdentityRecorded,
    Written,
    Flushed,
    Complete,
}

impl Workspace {
    /// The caller MUST obtain `selected_path` from a trusted OS-native folder
    /// picker. Never expose this method as an IPC command accepting a path.
    pub fn set_native_root(&self, selected_path: PathBuf) -> Result<RootSelection, WorkspaceError> {
        #[cfg(windows)]
        {
            let root = windows_fs::select_root(selected_path)?;
            let response = RootSelection {
                selected: true,
                label: root.label.clone(),
                read_available: false, // Host-native viewer gate; no WebView plaintext.
                apply_available: false,
            };
            let mut state = self.state.lock().map_err(|_| WorkspaceError::Unavailable)?;
            state.epoch = state.epoch.wrapping_add(1);
            state.root = Some(root);
            state.read_grants.clear();
            state.snapshots.clear();
            state.proposals.clear();
            state.creations.clear();
            Ok(response)
        }
        #[cfg(not(windows))]
        {
            let _ = selected_path;
            Err(WorkspaceError::Unavailable)
        }
    }

    /// Only a non-sensitive label crosses IPC. A missing root yields `None`.
    pub fn selected_status(&self) -> Option<String> {
        self.state
            .lock()
            .ok()?
            .root
            .as_ref()
            .map(|r| r.label.clone())
    }

    /// The caller MUST obtain `selected_path` from a trusted OS-native FILE
    /// picker for this conversation and request, not from the renderer/model.
    #[allow(dead_code)] // Candidate disabled while a host-native file viewer is absent.
    pub fn grant_native_file_read(
        &self,
        conversation_id: &str,
        request_id: &str,
        selected_path: PathBuf,
    ) -> Result<FileSelection, WorkspaceError> {
        validate_id(conversation_id)?;
        validate_id(request_id)?;
        #[cfg(windows)]
        {
            let mut state = self.state.lock().map_err(|_| WorkspaceError::Unavailable)?;
            let root = state.root.as_ref().ok_or(WorkspaceError::Denied)?;
            let path = windows_fs::selected_relative_path(root, &selected_path)?;
            let (text, id) = windows_fs::read_guarded(root, &path)?;
            let relative_path = join_relative(&path);
            let sha256 = sha256_hex(text.as_bytes());
            let epoch = state.epoch;
            let key = (conversation_id.to_owned(), request_id.to_owned());
            let now = Instant::now();
            // One outstanding selected/read file per conversation. A new
            // explicit native choice revokes the previous snapshot and grant.
            state.read_grants.retain(|(conversation, _), value| {
                conversation != conversation_id && value.deadline > now
            });
            state.snapshots.retain(|(conversation, _), value| {
                conversation != conversation_id && value.deadline > now
            });
            state.proposals.retain(|_, value| {
                value.conversation_id != conversation_id && value.deadline > now
            });
            state.creations.retain(|_, value| {
                value.conversation_id != conversation_id && value.deadline > now
            });
            if !state.read_grants.contains_key(&key) && state.read_grants.len() >= MAX_PENDING {
                return Err(WorkspaceError::TooLarge);
            }
            state.snapshots.remove(&key);
            state.read_grants.insert(
                key,
                ReadGrant {
                    relative_path: relative_path.clone(),
                    epoch,
                    deadline: Instant::now() + GRANT_LIFETIME,
                    id,
                    sha256,
                },
            );
            Ok(FileSelection {
                relative_path,
                request_id: request_id.to_owned(),
            })
        }
        #[cfg(not(windows))]
        {
            let _ = selected_path;
            Err(WorkspaceError::Unavailable)
        }
    }

    /// Consume the exact selected-file grant once. This cannot read an
    /// arbitrary path merely because a project root is selected.
    #[allow(dead_code)] // Never expose project plaintext to the WebView via IPC.
    pub fn read_project_file(
        &self,
        conversation_id: &str,
        request_id: &str,
        relative_path: &str,
    ) -> Result<ProjectRead, WorkspaceError> {
        validate_id(conversation_id)?;
        validate_id(request_id)?;
        #[cfg(windows)]
        {
            let mut state = self.state.lock().map_err(|_| WorkspaceError::Unavailable)?;
            let key = (conversation_id.to_owned(), request_id.to_owned());
            // Consume even if later validation fails: stale or replayed grants
            // must never be revived by changing renderer input.
            let grant = state
                .read_grants
                .remove(&key)
                .ok_or(WorkspaceError::Denied)?;
            let components = parse_relative_path(relative_path)?;
            let normalized = join_relative(&components);
            if grant.relative_path != normalized
                || grant.deadline <= Instant::now()
                || grant.epoch != state.epoch
            {
                return Err(WorkspaceError::Denied);
            }
            let root = state.root.as_ref().ok_or(WorkspaceError::Denied)?;
            let (content, id) = windows_fs::read_guarded(root, &components)?;
            let sha256 = sha256_hex(content.as_bytes());
            if id != grant.id || sha256 != grant.sha256 {
                return Err(WorkspaceError::Conflict);
            }
            let read = ProjectRead {
                relative_path: normalized,
                content,
                sha256,
            };
            let epoch = state.epoch;
            let now = Instant::now();
            state.snapshots.retain(|_, value| value.deadline > now);
            if !state.snapshots.contains_key(&key) && state.snapshots.len() >= MAX_PENDING {
                return Err(WorkspaceError::TooLarge);
            }
            state.snapshots.insert(
                key,
                Snapshot {
                    read: read.clone(),
                    epoch,
                    deadline: Instant::now() + GRANT_LIFETIME,
                    disclosure_remaining: true,
                    dispatch_remaining: true,
                    prepared_disclosure: None,
                    proposal_remaining: true,
                    id,
                },
            );
            Ok(read)
        }
        #[cfg(not(windows))]
        {
            let _ = relative_path;
            Err(WorkspaceError::Unavailable)
        }
    }

    /// Consume this once BEFORE host-owned native disclosure confirmation.
    /// The caller shows the selected NVIDIA destination, file, byte count and
    /// exact digest, then transmits only on native confirmation. A refusal
    /// still consumes this right. Renderer-supplied content is never trusted.
    #[allow(dead_code)] // Disclosure remains disabled until native full-content review exists.
    pub fn take_disclosure(
        &self,
        conversation_id: &str,
        request_id: &str,
        relative_path: &str,
        expected_sha256: &str,
    ) -> Result<ProjectRead, WorkspaceError> {
        validate_id(conversation_id)?;
        validate_id(request_id)?;
        #[cfg(windows)]
        {
            let mut state = self.state.lock().map_err(|_| WorkspaceError::Unavailable)?;
            Self::take_disclosure_locked(
                &mut state,
                conversation_id,
                request_id,
                relative_path,
                expected_sha256,
            )
        }
        #[cfg(not(windows))]
        {
            let _ = (relative_path, expected_sha256);
            Err(WorkspaceError::Unavailable)
        }
    }

    #[cfg(windows)]
    fn take_disclosure_locked(
        state: &mut State,
        conversation_id: &str,
        request_id: &str,
        relative_path: &str,
        expected_sha256: &str,
    ) -> Result<ProjectRead, WorkspaceError> {
        let epoch = state.epoch;
        let (read, expected_id) = {
            let snapshot = state
                .snapshots
                .get_mut(&(conversation_id.to_owned(), request_id.to_owned()))
                .ok_or(WorkspaceError::Denied)?;
            let was_available = snapshot.disclosure_remaining;
            // A bad path, stale hash, expiry or refusal still consumes the
            // right. An untrusted retry cannot revive this snapshot.
            snapshot.disclosure_remaining = false;
            let normalized = join_relative(&parse_relative_path(relative_path)?);
            if !was_available
                || snapshot.epoch != epoch
                || snapshot.deadline <= Instant::now()
                || snapshot.read.relative_path != normalized
                || snapshot.read.sha256 != expected_sha256
            {
                return Err(WorkspaceError::Denied);
            }
            (snapshot.read.clone(), snapshot.id)
        };
        let root = state.root.as_ref().ok_or(WorkspaceError::Denied)?;
        let (current, current_id) =
            windows_fs::read_guarded(root, &parse_relative_path(&read.relative_path)?)?;
        if current_id != expected_id || sha256_hex(current.as_bytes()) != read.sha256 {
            return Err(WorkspaceError::Conflict);
        }
        Ok(read)
    }

    /// Prepare a request-scoped native review of the exact outbound bytes.
    /// This does not send, reserve a paid request, or make disclosure active.
    /// The host must use ephemeral admission and pass the actual serialized
    /// provider body; no renderer-supplied body or project bytes are accepted
    /// by a shipping command.
    #[cfg(windows)]
    #[allow(dead_code)]
    pub fn prepare_disclosure_candidate(
        &self,
        conversation_id: &str,
        request_id: &str,
        relative_path: &str,
        expected_sha256: &str,
        exact_outbound_body: Vec<u8>,
    ) -> Result<crate::native_review::ReviewDocument, WorkspaceError> {
        validate_id(conversation_id)?;
        validate_id(request_id)?;
        let mut state = self.state.lock().map_err(|_| WorkspaceError::Unavailable)?;
        // Consumption and preparation share ONE lock. A root reset or same-
        // request regrant cannot substitute another snapshot between phases.
        let read = Self::take_disclosure_locked(
            &mut state,
            conversation_id,
            request_id,
            relative_path,
            expected_sha256,
        )?;
        let epoch = state.epoch;
        let root = state.root.as_ref().ok_or(WorkspaceError::Denied)?;
        let root_id = root.id;
        let root_path = root.path.to_str().ok_or(WorkspaceError::Denied)?.to_owned();
        let parent =
            windows_fs::pin_creation_parent(root, &parse_relative_path(&read.relative_path)?)?;
        let key = (conversation_id.to_owned(), request_id.to_owned());
        let snapshot = state
            .snapshots
            .get_mut(&key)
            .ok_or(WorkspaceError::Denied)?;
        // Reserve preparation once, before parsing/encoding. Invalid material
        // cannot be corrected later using the consumed native file grant.
        let was_available = snapshot.dispatch_remaining;
        snapshot.dispatch_remaining = false;
        if !was_available
            || snapshot.disclosure_remaining
            || snapshot.prepared_disclosure.is_some()
            || snapshot.epoch != epoch
            || snapshot.deadline <= Instant::now()
        {
            return Err(WorkspaceError::Denied);
        }
        let metadata = serde_json::to_string(&serde_json::json!({
            "version": 1,
            "operation": "project_disclosure",
            "conversationId": conversation_id,
            "requestId": request_id,
            "rootEpoch": epoch,
            "rootIdentity": windows_fs::record_identity(root_id),
            "rootPath": root_path,
            "targetPath": parent.target_path.to_str().ok_or(WorkspaceError::Denied)?,
            "relativePath": read.relative_path,
            "fileIdentity": windows_fs::record_identity(snapshot.id),
            "fileSha256": read.sha256,
            "nonce": windows_fs::random_nonce()?,
            "expiresAfterSeconds": GRANT_LIFETIME.as_secs(),
            "uses": 1,
        }))
        .map_err(|_| WorkspaceError::Unavailable)?;
        let review = crate::native_review::ReviewDocument::disclosure_with_context(
            &exact_outbound_body,
            &metadata,
        )
        .map_err(|_| WorkspaceError::Denied)?;
        snapshot.prepared_disclosure = Some(PreparedDisclosure {
            exact_outbound_body,
            review_binding: review.binding_sha256().to_owned(),
        });
        snapshot.dispatch_remaining = true;
        Ok(review)
    }

    /// Final barrier AFTER native review: consume the exact approval, then
    /// recheck expiry/root generation/current file identity+hash. Only these
    /// exact stored bytes may be dispatched by a future provider integration.
    #[cfg(windows)]
    #[allow(dead_code)]
    pub fn finalize_disclosure_candidate(
        &self,
        conversation_id: &str,
        request_id: &str,
        approval: crate::native_review::NativeApproval,
    ) -> Result<Vec<u8>, WorkspaceError> {
        validate_id(conversation_id)?;
        validate_id(request_id)?;
        let mut state = self.state.lock().map_err(|_| WorkspaceError::Unavailable)?;
        let epoch = state.epoch;
        let (read, expected_id, prepared) = {
            let snapshot = state
                .snapshots
                .get_mut(&(conversation_id.to_owned(), request_id.to_owned()))
                .ok_or(WorkspaceError::Denied)?;
            let was_available = snapshot.dispatch_remaining;
            snapshot.dispatch_remaining = false;
            let prepared = snapshot
                .prepared_disclosure
                .take()
                .ok_or(WorkspaceError::Denied)?;
            approval
                .consume(&prepared.review_binding)
                .map_err(|_| WorkspaceError::Denied)?;
            if !was_available
                || snapshot.disclosure_remaining
                || snapshot.epoch != epoch
                || snapshot.deadline <= Instant::now()
            {
                return Err(WorkspaceError::Denied);
            }
            (snapshot.read.clone(), snapshot.id, prepared)
        };
        let root = state.root.as_ref().ok_or(WorkspaceError::Denied)?;
        let (current, id) =
            windows_fs::read_guarded(root, &parse_relative_path(&read.relative_path)?)?;
        if id != expected_id || sha256_hex(current.as_bytes()) != read.sha256 {
            return Err(WorkspaceError::Conflict);
        }
        Ok(prepared.exact_outbound_body)
    }

    /// Propose replacement of the exact selected/read file. Never publishes.
    /// `expected_old_sha256` is mandatory for an existing-file proposal.
    #[allow(dead_code)] // Needs a host-native full-content viewer to be usable.
    pub fn review_proposal(
        &self,
        conversation_id: &str,
        request_id: &str,
        relative_path: &str,
        new_content: &str,
        expected_old_sha256: Option<&str>,
    ) -> Result<ProposalReview, WorkspaceError> {
        validate_id(conversation_id)?;
        validate_id(request_id)?;
        // The resulting existing file may still be 1 MiB when only a small
        // region changes; the actual removed/added spans are bounded below.
        validate_text(new_content, MAX_READ_BYTES)?;
        let normalized = join_relative(&parse_relative_path(relative_path)?);
        let expected = expected_old_sha256.ok_or(WorkspaceError::Denied)?;
        #[cfg(windows)]
        {
            let mut state = self.state.lock().map_err(|_| WorkspaceError::Unavailable)?;
            let epoch = state.epoch;
            let key = (conversation_id.to_owned(), request_id.to_owned());
            let snapshot = state.snapshots.get(&key).ok_or(WorkspaceError::Denied)?;
            if !snapshot.proposal_remaining
                || snapshot.epoch != epoch
                || snapshot.deadline <= Instant::now()
                || snapshot.read.relative_path != normalized
                || snapshot.read.sha256 != expected
            {
                return Err(WorkspaceError::Denied);
            }
            let root = state.root.as_ref().ok_or(WorkspaceError::Denied)?;
            let (current, id) = windows_fs::read_guarded(root, &parse_relative_path(&normalized)?)?;
            if id != snapshot.id || sha256_hex(current.as_bytes()) != snapshot.read.sha256 {
                return Err(WorkspaceError::Conflict);
            }
            let (removed_bytes, added_bytes) = changed_bytes(&current, new_content);
            if removed_bytes > MAX_CHANGED_BYTES || added_bytes > MAX_CHANGED_BYTES {
                return Err(WorkspaceError::TooLarge);
            }
            if removed_bytes == 0 && added_bytes == 0 {
                return Err(WorkspaceError::Denied);
            }
            let old_sha256 = snapshot.read.sha256.clone();
            let new_sha256 = sha256_hex(new_content.as_bytes());
            let old_id = snapshot.id;
            let parent = windows_fs::pin_creation_parent(root, &parse_relative_path(&normalized)?)?;
            let proposal_id = windows_fs::random_nonce()?;
            // This structured JSON is lossless, distinguishes content from
            // review labels, and can be rendered inertly or parsed by a future
            // host-owned native full-diff viewer.
            let diff = serde_json::to_string(&serde_json::json!({
                "operation": "replace_existing",
                "version": 1,
                "conversationId": conversation_id,
                "requestId": request_id,
                "rootEpoch": epoch,
                "rootIdentity": windows_fs::record_identity(root.id),
                "parentIdentity": windows_fs::record_identity(parent.id),
                "fileIdentity": windows_fs::record_identity(old_id),
                "rootPath": root.path.to_str().ok_or(WorkspaceError::Denied)?,
                "targetPath": parent.target_path.to_str().ok_or(WorkspaceError::Denied)?,
                "nonce": proposal_id,
                "expiresAfterSeconds": GRANT_LIFETIME.as_secs(),
                "uses": 1,
                "relativePath": normalized.clone(),
                "before": {"sha256": old_sha256.clone(), "content": current},
                "after": {"sha256": new_sha256.clone(), "content": new_content},
            }))
            .map_err(|_| WorkspaceError::Unavailable)?;
            if diff.len() > MAX_DIFF_BYTES {
                return Err(WorkspaceError::TooLarge);
            }
            let now = Instant::now();
            state.proposals.retain(|_, value| {
                value.deadline > now && value.conversation_id != conversation_id
            });
            state.creations.retain(|_, value| {
                value.deadline > now && value.conversation_id != conversation_id
            });
            if state.proposals.len() + state.creations.len() >= MAX_PENDING {
                return Err(WorkspaceError::TooLarge);
            }
            if state.proposals.contains_key(&proposal_id) {
                return Err(WorkspaceError::Unavailable);
            }
            state
                .snapshots
                .get_mut(&key)
                .ok_or(WorkspaceError::Denied)?
                .proposal_remaining = false;
            state.proposals.insert(
                proposal_id.clone(),
                Proposal {
                    conversation_id: conversation_id.to_owned(),
                    request_id: request_id.to_owned(),
                    relative_path: normalized.clone(),
                    root_epoch: epoch,
                    old_sha256: old_sha256.clone(),
                    new_sha256: new_sha256.clone(),
                    complete_diff: diff.clone(),
                    deadline: Instant::now() + GRANT_LIFETIME,
                    remaining_uses: 1,
                    old_id,
                },
            );
            Ok(ProposalReview {
                status: "proposal_only".to_owned(),
                operation: "replace_existing".to_owned(),
                relative_path: normalized,
                old_sha256: Some(old_sha256),
                new_sha256,
                diff,
                proposal_id,
                apply_available: false,
            })
        }
        #[cfg(not(windows))]
        {
            let _ = (normalized, expected);
            Err(WorkspaceError::Unavailable)
        }
    }

    /// Bind an untrusted JSON response to host-supplied request/snapshot IDs.
    /// None of these IDs or approvals can be overridden by the parsed body.
    #[allow(dead_code)]
    pub fn review_model_replacement(
        &self,
        conversation_id: &str,
        request_id: &str,
        json: &str,
    ) -> Result<ProposalReview, WorkspaceError> {
        let edit = parse_proposed_edit(json)?;
        if edit.operation != ProposalOperation::ReplaceExisting {
            return Err(WorkspaceError::Denied);
        }
        self.review_proposal(
            conversation_id,
            request_id,
            &edit.relative_path,
            &edit.new_content,
            edit.expected_old_sha256.as_deref(),
        )
    }

    /// Windows-only, host-internal preparation. A proposal is not approval.
    /// The journal path comes from a trusted host configuration, is validated
    /// and pinned, and is displayed in the exact native review material.
    #[cfg(windows)]
    #[allow(dead_code)]
    pub fn prepare_new_file_proposal(
        &self,
        conversation_id: &str,
        request_id: &str,
        json: &str,
        journal_directory: PathBuf,
    ) -> Result<ProposalReview, WorkspaceError> {
        validate_id(conversation_id)?;
        validate_id(request_id)?;
        let edit = parse_proposed_edit(json)?;
        if edit.operation != ProposalOperation::CreateNew {
            return Err(WorkspaceError::Denied);
        }
        let journal_root = windows_fs::select_root(journal_directory)?;
        let journal = crate::new_file::Journal::in_pinned_directory(
            &journal_root.path,
            &journal_root.anchor,
            &journal_root.ancestor_pins,
            windows_fs::record_identity(journal_root.id),
        )?;
        let _journal_pins =
            windows_fs::pin_creation_parent(&journal_root, &["journal-guard".into()])?;
        // Conservative gate: every prior journal must verify against this
        // selected root before admitting another candidate. No cleanup or
        // recovery mutation is attempted by this proof seam.
        let mut state = self.state.lock().map_err(|_| WorkspaceError::Unavailable)?;
        let root = state.root.as_ref().ok_or(WorkspaceError::Denied)?;
        if !windows_fs::journal_is_resolved(root, &journal).unwrap_or(false) {
            return Err(WorkspaceError::AttentionRequired);
        }
        require_journal_capacity(journal.nonces()?.len())?;
        let parent =
            windows_fs::pin_creation_parent(root, &parse_relative_path(&edit.relative_path)?)?;
        if parent.contains_identity(journal_root.id)? {
            return Err(WorkspaceError::Denied);
        }
        windows_fs::require_absent(&parent.target_path)?;
        let new_sha256 = sha256_hex(edit.new_content.as_bytes());
        let root_epoch = state.epoch;
        let root_id = root.id;
        let parent_id = parent.id;
        let nonce = windows_fs::random_nonce()?;
        let diff = serde_json::to_string(&serde_json::json!({
            "version": 1,
            "operation": "create_new",
            "conversationId": conversation_id,
            "requestId": request_id,
            "rootEpoch": root_epoch,
            "rootIdentity": windows_fs::record_identity(root_id),
            "parentIdentity": windows_fs::record_identity(parent_id),
            "rootPath": root.path.to_str().ok_or(WorkspaceError::Denied)?,
            "targetPath": parent.target_path.to_str().ok_or(WorkspaceError::Denied)?,
            "relativePath": edit.relative_path,
            "nonce": nonce,
            "expiresAfterSeconds": GRANT_LIFETIME.as_secs(),
            "uses": 1,
            "journalDirectory": journal_root.path.to_str().ok_or(WorkspaceError::Denied)?,
            "journalPrivacy": "Windows DPAPI current-user encrypted metadata only; no file contents",
            "before": null,
            "after": {"sha256": new_sha256, "content": edit.new_content}
        })).map_err(|_| WorkspaceError::Unavailable)?;
        if diff.len() > MAX_DIFF_BYTES {
            return Err(WorkspaceError::TooLarge);
        }
        let review = crate::native_review::ReviewDocument::proposal(&diff)
            .map_err(|_| WorkspaceError::Unavailable)?;
        let review_binding = review.binding_sha256().to_owned();
        let now = Instant::now();
        state
            .creations
            .retain(|_, value| value.deadline > now && value.conversation_id != conversation_id);
        state
            .proposals
            .retain(|_, value| value.deadline > now && value.conversation_id != conversation_id);
        if state.creations.len() + state.proposals.len() >= MAX_PENDING {
            return Err(WorkspaceError::TooLarge);
        }
        if state.creations.contains_key(&nonce) || state.proposals.contains_key(&nonce) {
            return Err(WorkspaceError::Unavailable);
        }
        state.creations.insert(
            nonce.clone(),
            PendingCreation {
                conversation_id: conversation_id.to_owned(),
                request_id: request_id.to_owned(),
                relative_path: edit.relative_path.clone(),
                root_epoch,
                root_id,
                parent_id,
                new_content: edit.new_content,
                new_sha256: new_sha256.clone(),
                complete_diff: diff.clone(),
                review_binding,
                journal_root,
                deadline: now + GRANT_LIFETIME,
            },
        );
        Ok(ProposalReview {
            status: "candidate_only".into(),
            operation: "create_new".into(),
            relative_path: edit.relative_path,
            old_sha256: None,
            new_sha256,
            diff,
            proposal_id: nonce,
            apply_available: false,
        })
    }

    /// A non-serializable, move-only native approval is consumed in the host.
    /// This is an INTERNAL proof seam, never a registered command or feature.
    #[cfg(windows)]
    #[allow(dead_code)]
    pub fn apply_new_file_candidate(
        &self,
        proposal_id: &str,
        approval: crate::native_review::NativeApproval,
    ) -> Result<CreationOutcome, WorkspaceError> {
        self.apply_new_file_with_stage(proposal_id, approval, |_| Ok(()))
    }

    #[cfg(windows)]
    fn apply_new_file_with_stage<F>(
        &self,
        proposal_id: &str,
        approval: crate::native_review::NativeApproval,
        mut stage: F,
    ) -> Result<CreationOutcome, WorkspaceError>
    where
        F: FnMut(CreationStage) -> Result<(), WorkspaceError>,
    {
        let mut state = self.state.lock().map_err(|_| WorkspaceError::Unavailable)?;
        // Remove before all later validation. Refused/stale/failed operations
        // cannot be replayed with a corrected hash or a second token.
        let pending = state
            .creations
            .remove(proposal_id)
            .ok_or(WorkspaceError::Denied)?;
        approval
            .consume(&pending.review_binding)
            .map_err(|_| WorkspaceError::Denied)?;
        if pending.root_epoch != state.epoch || pending.deadline <= Instant::now() {
            return Err(WorkspaceError::Denied);
        }
        let root = state.root.as_ref().ok_or(WorkspaceError::Denied)?;
        if root.id != pending.root_id {
            return Err(WorkspaceError::Conflict);
        }
        windows_fs::check_root(&pending.journal_root)?;
        let _journal_pins =
            windows_fs::pin_creation_parent(&pending.journal_root, &["journal-guard".into()])?;
        let journal = crate::new_file::Journal::in_pinned_directory(
            &pending.journal_root.path,
            &pending.journal_root.anchor,
            &pending.journal_root.ancestor_pins,
            windows_fs::record_identity(pending.journal_root.id),
        )?;
        if !windows_fs::journal_is_resolved(root, &journal).unwrap_or(false) {
            return Err(WorkspaceError::AttentionRequired);
        }
        require_journal_capacity(journal.nonces()?.len())?;
        let parent =
            windows_fs::pin_creation_parent(root, &parse_relative_path(&pending.relative_path)?)?;
        if parent.contains_identity(pending.journal_root.id)? {
            return Err(WorkspaceError::Denied);
        }
        if parent.id != pending.parent_id {
            return Err(WorkspaceError::Conflict);
        }
        windows_fs::require_absent(&parent.target_path)?;
        let intent = crate::new_file::Intent {
            version: 1,
            nonce: proposal_id.to_owned(),
            conversation_id: pending.conversation_id,
            request_id: pending.request_id,
            root_epoch: pending.root_epoch,
            root_identity: windows_fs::record_identity(pending.root_id),
            parent_identity: windows_fs::record_identity(parent.id),
            relative_path: pending.relative_path,
            content_sha256: pending.new_sha256.clone(),
            content_bytes: pending.new_content.len(),
            review_binding_sha256: pending.review_binding,
        };
        // Any failure after durable intent is ATTENTION_REQUIRED. Never remove
        // a partially created target or an ambiguous immutable journal.
        let publish = (|| {
            journal.persist(proposal_id, "intent", &intent)?;
            stage(CreationStage::Intent)?;
            journal.revalidate()?;
            journal.nonces()?;
            let mut target = windows_fs::create_new_exclusive(&parent.target_path)?;
            stage(CreationStage::Created)?;
            let target_id = crate::new_file::identity(&target)?;
            journal.persist(
                proposal_id,
                "created",
                &crate::new_file::Created {
                    version: 1,
                    nonce: proposal_id.to_owned(),
                    identity: target_id,
                },
            )?;
            stage(CreationStage::IdentityRecorded)?;
            use std::io::Write as _;
            journal.revalidate()?;
            journal.nonces()?;
            target
                .write_all(pending.new_content.as_bytes())
                .map_err(|_| WorkspaceError::Io)?;
            stage(CreationStage::Written)?;
            target.sync_all().map_err(|_| WorkspaceError::Io)?;
            stage(CreationStage::Flushed)?;
            windows_fs::verify_created(
                &mut target,
                target_id,
                &pending.new_sha256,
                pending.new_content.len(),
            )?;
            journal.persist(
                proposal_id,
                "complete",
                &crate::new_file::Complete {
                    version: 1,
                    nonce: proposal_id.to_owned(),
                    identity: target_id,
                    content_sha256: pending.new_sha256,
                    content_bytes: pending.new_content.len(),
                },
            )?;
            stage(CreationStage::Complete)?;
            journal.revalidate()?;
            journal.nonces()?; // Recheck every immutable record's privacy before success.
            Ok::<(), WorkspaceError>(())
        })();
        Ok(if publish.is_ok() {
            CreationOutcome::Verified
        } else {
            CreationOutcome::AttentionRequired
        })
    }

    #[cfg(windows)]
    #[allow(dead_code)]
    pub fn inspect_creation_recovery(
        &self,
        journal_directory: PathBuf,
    ) -> Result<CreationOutcome, WorkspaceError> {
        let journal_root = windows_fs::select_root(journal_directory)?;
        let state = self.state.lock().map_err(|_| WorkspaceError::Unavailable)?;
        let root = state.root.as_ref().ok_or(WorkspaceError::Denied)?;
        let journal = match crate::new_file::Journal::in_pinned_directory(
            &journal_root.path,
            &journal_root.anchor,
            &journal_root.ancestor_pins,
            windows_fs::record_identity(journal_root.id),
        ) {
            Ok(journal) => journal,
            Err(_) => return Ok(CreationOutcome::AttentionRequired),
        };
        let _journal_pins =
            windows_fs::pin_creation_parent(&journal_root, &["journal-guard".into()])?;
        Ok(
            if windows_fs::journal_is_resolved(root, &journal).unwrap_or(false) {
                CreationOutcome::Verified
            } else {
                CreationOutcome::AttentionRequired
            },
        )
    }

    /// Kept as an explicit capability boundary. Never accept an IPC boolean
    /// or proposal ID as user approval. Native review, exclusive publication,
    /// durable recovery and adversarial Windows evidence are still required.
    #[allow(dead_code)]
    pub fn apply_proposal(&self, _proposal_id: &str) -> Result<(), WorkspaceError> {
        Err(WorkspaceError::Unavailable)
    }
}

fn validate_id(value: &str) -> Result<(), WorkspaceError> {
    if value.is_empty()
        || value.len() > MAX_ID_BYTES
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(WorkspaceError::Denied);
    }
    Ok(())
}

#[cfg(any(windows, test))]
fn require_journal_capacity(operations: usize) -> Result<(), WorkspaceError> {
    // Reserve one full triple before intent; never publish a 65th operation
    // beyond the journal reader's 192-entry recovery ceiling.
    if operations < 64 {
        Ok(())
    } else {
        Err(WorkspaceError::TooLarge)
    }
}

fn validate_text(value: &str, max_bytes: u64) -> Result<(), WorkspaceError> {
    if value.len() as u64 > max_bytes {
        return Err(WorkspaceError::TooLarge);
    }
    if value.chars().any(|ch| {
        ch == '\0'
            || risky_format_mark(ch)
            || (ch.is_control() && !matches!(ch, '\n' | '\r' | '\t'))
    }) {
        return Err(WorkspaceError::Denied);
    }
    Ok(())
}

fn parse_relative_path(path: &str) -> Result<Vec<String>, WorkspaceError> {
    if path.is_empty()
        || path.len() > MAX_PATH_BYTES
        || path.starts_with('/')
        || path.starts_with('\\')
    {
        return Err(WorkspaceError::Denied);
    }
    let mut result = Vec::new();
    for segment in path.split(['/', '\\']) {
        if segment.is_empty()
            || segment == "."
            || segment == ".."
            || segment.ends_with('.')
            || segment.ends_with(' ')
            || segment
                .chars()
                .any(|c| c.is_control() || risky_format_mark(c) || ":*?\"<>|~".contains(c))
        {
            return Err(WorkspaceError::Denied);
        }
        let low = segment.to_ascii_lowercase();
        let base = low.split('.').next().unwrap_or("");
        let reserved = matches!(
            base,
            "con" | "prn" | "aux" | "nul" | "clock$" | "conin$" | "conout$"
        ) || (base.len() == 4
            && (base.starts_with("com") || base.starts_with("lpt"))
            && base.as_bytes()[3].is_ascii_digit())
            || ["com¹", "com²", "com³", "lpt¹", "lpt²", "lpt³"].contains(&base);
        let sensitive = matches!(
            low.as_str(),
            ".git" | ".ssh" | ".aws" | ".npmrc" | ".pypirc" | ".netrc" | "id_rsa" | "id_ed25519"
        ) || low == ".env"
            || low.starts_with(".env.")
            || low.starts_with("credentials")
            || low.starts_with("secret")
            || [".pem", ".key", ".p12", ".pfx", ".kdbx"]
                .iter()
                .any(|s| low.ends_with(s));
        if reserved || sensitive {
            return Err(WorkspaceError::Denied);
        }
        result.push(segment.to_owned());
        if result.len() > 64 {
            return Err(WorkspaceError::TooLarge);
        }
    }
    Ok(result)
}

fn join_relative(components: &[String]) -> String {
    components.join("/")
}

/// Reject invisible/reordering Unicode formatting in paths and reviewed text.
/// A richer future native viewer may encode these visibly and revise policy.
fn risky_format_mark(ch: char) -> bool {
    matches!(ch,
        '\u{00ad}' | '\u{034f}' | '\u{061c}' | '\u{180e}' | '\u{2060}' | '\u{feff}'
            | '\u{200b}'..='\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}'
    )
}

#[cfg(windows)]
fn sha256_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut encoded, "{byte:02x}").expect("writing to a String cannot fail");
    }
    encoded
}

/// Return the lengths of the changed old/new UTF-8 slices, using common
/// Unicode scalar prefix and suffix boundaries (no byte-split UTF-8).
#[cfg(any(windows, test))]
fn changed_bytes(old: &str, new: &str) -> (usize, usize) {
    let mut prefix = 0;
    for (a, b) in old.chars().zip(new.chars()) {
        if a != b {
            break;
        }
        prefix += a.len_utf8();
    }
    let mut suffix = 0;
    for (a, b) in old[prefix..].chars().rev().zip(new[prefix..].chars().rev()) {
        if a != b {
            break;
        }
        suffix += a.len_utf8();
    }
    (old.len() - prefix - suffix, new.len() - prefix - suffix)
}

#[cfg(windows)]
#[allow(dead_code)] // Windows read/proposal candidate; no renderer IPC until native gate.
mod windows_fs {
    use super::*;
    use std::ffi::c_void;
    use std::fs::{File, OpenOptions};
    use std::io::Read;
    use std::mem::MaybeUninit;
    use std::os::windows::fs::OpenOptionsExt;
    use std::os::windows::io::AsRawHandle;

    // Win32 constants; OS handles are acquired by Rust OpenOptions with
    // OPEN_REPARSE_POINT and then independently checked by handle.
    const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x0000_0010;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    const FILE_SHARE_READ: u32 = 0x0000_0001;
    const FILE_SHARE_WRITE: u32 = 0x0000_0002;
    const DRIVE_FIXED: u32 = 3;
    const BCRYPT_USE_SYSTEM_PREFERRED_RNG: u32 = 0x0000_0002;

    #[repr(C)]
    struct FileTime {
        low: u32,
        high: u32,
    }
    #[repr(C)]
    struct ByHandleFileInformation {
        attributes: u32,
        created: FileTime,
        accessed: FileTime,
        modified: FileTime,
        volume: u32,
        size_high: u32,
        size_low: u32,
        links: u32,
        index_high: u32,
        index_low: u32,
    }
    #[allow(non_snake_case, clippy::too_many_arguments)]
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetFileInformationByHandle(
            handle: *mut c_void,
            info: *mut ByHandleFileInformation,
        ) -> i32;
        fn GetDriveTypeW(root_path: *const u16) -> u32;
        fn QueryDosDeviceW(device_name: *const u16, target_path: *mut u16, capacity: u32) -> u32;
        fn GetVolumeInformationW(
            root_path: *const u16,
            volume_name: *mut u16,
            volume_name_capacity: u32,
            volume_serial_number: *mut u32,
            max_component_length: *mut u32,
            file_system_flags: *mut u32,
            file_system_name: *mut u16,
            file_system_name_capacity: u32,
        ) -> i32;
    }
    #[allow(non_snake_case)]
    #[link(name = "bcrypt")]
    unsafe extern "system" {
        fn BCryptGenRandom(algorithm: *mut c_void, buffer: *mut u8, bytes: u32, flags: u32) -> i32;
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(super) struct FileIdentity {
        volume: u32,
        index: u64,
    }

    pub(super) fn record_identity(id: FileIdentity) -> crate::new_file::Identity {
        crate::new_file::Identity {
            volume: id.volume,
            index: id.index,
        }
    }

    fn file_info(file: &File) -> Result<ByHandleFileInformation, WorkspaceError> {
        let mut info = MaybeUninit::<ByHandleFileInformation>::uninit();
        // SAFETY: File's OS handle remains open for this call and points to a
        // correctly laid-out, writable BY_HANDLE_FILE_INFORMATION buffer.
        let ok = unsafe { GetFileInformationByHandle(file.as_raw_handle(), info.as_mut_ptr()) };
        if ok == 0 {
            return Err(WorkspaceError::Io);
        }
        // SAFETY: a successful Win32 call fully initialized the output.
        Ok(unsafe { info.assume_init() })
    }

    fn identity(info: &ByHandleFileInformation) -> FileIdentity {
        FileIdentity {
            volume: info.volume,
            index: ((info.index_high as u64) << 32) | info.index_low as u64,
        }
    }

    fn check_info(
        info: &ByHandleFileInformation,
        directory: bool,
    ) -> Result<FileIdentity, WorkspaceError> {
        if info.attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
            || ((info.attributes & FILE_ATTRIBUTE_DIRECTORY != 0) != directory)
            || (!directory && info.links != 1)
        {
            return Err(WorkspaceError::Denied);
        }
        Ok(identity(info))
    }

    fn open_dir(path: &Path, share: u32) -> Result<File, WorkspaceError> {
        let file = OpenOptions::new()
            .read(true)
            .share_mode(share)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)
            .map_err(|_| WorkspaceError::Io)?;
        check_info(&file_info(&file)?, true)?;
        Ok(file)
    }

    fn local_absolute(path: &Path) -> Result<Vec<String>, WorkspaceError> {
        let text = path.to_str().ok_or(WorkspaceError::Denied)?;
        let bytes = text.as_bytes();
        if bytes.len() < 3
            || bytes.len() > MAX_PATH_BYTES
            || !bytes[0].is_ascii_alphabetic()
            || bytes[1] != b':'
            || !matches!(bytes[2], b'\\' | b'/')
        {
            // Excludes UNC, \\?\ paths, NT namespaces, devices and drive-relative.
            return Err(WorkspaceError::Denied);
        }
        if text[3..].contains(':') {
            return Err(WorkspaceError::Denied);
        }
        let tail = text[3..].trim_end_matches(['/', '\\']);
        if tail.is_empty() {
            return Ok(Vec::new());
        }
        parse_relative_path(tail)
    }

    fn local_ntfs_drive(drive_prefix: &str) -> Result<u32, WorkspaceError> {
        let bytes = drive_prefix.as_bytes();
        let drive_root = [bytes[0] as u16, b':' as u16, b'\\' as u16, 0];
        // SAFETY: NUL-terminated drive root from validated ASCII drive letter.
        if unsafe { GetDriveTypeW(drive_root.as_ptr()) } != DRIVE_FIXED {
            // In particular DRIVE_REMOTE mapped SMB shares look like Z:\\...
            return Err(WorkspaceError::Denied);
        }
        // A SUBST drive can report DRIVE_FIXED while resolving through a DOS
        // device alias to an arbitrary path. Demand a direct harddisk volume
        // mapping, and still verify its volume serial against the open root.
        let drive_name = [bytes[0] as u16, b':' as u16, 0];
        let mut target = [0u16; 512];
        // SAFETY: valid NUL-terminated drive name and writable target buffer.
        let units = unsafe {
            QueryDosDeviceW(
                drive_name.as_ptr(),
                target.as_mut_ptr(),
                target.len() as u32,
            )
        } as usize;
        if units == 0 || units > target.len() {
            return Err(WorkspaceError::Denied);
        }
        let first_end = target[..units]
            .iter()
            .position(|unit| *unit == 0)
            .ok_or(WorkspaceError::Denied)?;
        let mapping =
            String::from_utf16(&target[..first_end]).map_err(|_| WorkspaceError::Denied)?;
        let suffix = mapping
            .strip_prefix(r"\Device\HarddiskVolume")
            .ok_or(WorkspaceError::Denied)?;
        if suffix.is_empty() || !suffix.bytes().all(|b| b.is_ascii_digit()) {
            return Err(WorkspaceError::Denied);
        }
        let mut serial = 0u32;
        let mut file_system = [0u16; 32];
        // SAFETY: each pointer is either null (unused output) or points to a
        // correctly-sized writable buffer with a declared element capacity.
        let ok = unsafe {
            GetVolumeInformationW(
                drive_root.as_ptr(),
                std::ptr::null_mut(),
                0,
                &mut serial,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                file_system.as_mut_ptr(),
                file_system.len() as u32,
            )
        };
        if ok == 0 {
            return Err(WorkspaceError::Unavailable);
        }
        let length = file_system
            .iter()
            .position(|unit| *unit == 0)
            .ok_or(WorkspaceError::Denied)?;
        if String::from_utf16(&file_system[..length]).map_err(|_| WorkspaceError::Denied)? != "NTFS"
        {
            return Err(WorkspaceError::Denied);
        }
        Ok(serial)
    }

    pub(super) fn select_root(path: PathBuf) -> Result<Root, WorkspaceError> {
        let segments = local_absolute(&path)?;
        let prefix = &path.to_str().ok_or(WorkspaceError::Denied)?[..3];
        let volume_serial = local_ntfs_drive(prefix)?;
        let mut cursor = PathBuf::from(prefix);
        // Keep each parent open without DELETE sharing until after pinning the
        // selected root. Busy or concurrently renamed directories fail closed.
        let mut parents = Vec::new();
        for part in &segments {
            parents.push(open_dir(&cursor, FILE_SHARE_READ | FILE_SHARE_WRITE)?);
            cursor.push(part);
        }
        let anchor = open_dir(&cursor, FILE_SHARE_READ | FILE_SHARE_WRITE)?;
        let id = check_info(&file_info(&anchor)?, true)?;
        if id.volume != volume_serial {
            return Err(WorkspaceError::Denied);
        }
        let label = segments
            .last()
            .cloned()
            .unwrap_or_else(|| prefix.to_owned());
        Ok(Root {
            path: cursor,
            label,
            anchor,
            id,
            ancestor_pins: parents,
        })
    }

    pub(super) fn selected_relative_path(
        root: &Root,
        selected: &Path,
    ) -> Result<Vec<String>, WorkspaceError> {
        let _ = local_absolute(selected)?;
        let raw = selected
            .strip_prefix(&root.path)
            .map_err(|_| WorkspaceError::Denied)?;
        let text = raw.to_str().ok_or(WorkspaceError::Denied)?;
        parse_relative_path(text)
    }

    pub(super) fn read_guarded(
        root: &Root,
        components: &[String],
    ) -> Result<(String, FileIdentity), WorkspaceError> {
        if components.is_empty() {
            return Err(WorkspaceError::Denied);
        }
        for ancestor in &root.ancestor_pins {
            check_info(&file_info(ancestor)?, true)?;
        }
        let mut dirs = Vec::with_capacity(components.len());
        // Verify the selected root remains the same directory object. The
        // anchor denies delete sharing and stays open for the session.
        let anchor_id = check_info(&file_info(&root.anchor)?, true)?;
        if anchor_id != root.id {
            return Err(WorkspaceError::Conflict);
        }
        let current_root = open_dir(&root.path, FILE_SHARE_READ | FILE_SHARE_WRITE)?;
        if check_info(&file_info(&current_root)?, true)? != root.id {
            return Err(WorkspaceError::Conflict);
        }
        let mut cursor = root.path.clone();
        for part in &components[..components.len() - 1] {
            cursor.push(part);
            // No DELETE sharing pins each checked directory against rename;
            // an existing conflicting handle causes denial.
            dirs.push(open_dir(&cursor, FILE_SHARE_READ | FILE_SHARE_WRITE)?);
        }
        cursor.push(components.last().ok_or(WorkspaceError::Denied)?);
        let mut file = OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(&cursor)
            .map_err(|_| WorkspaceError::Io)?;
        let info = file_info(&file)?;
        let id = check_info(&info, false)?;
        if (((info.size_high as u64) << 32) | info.size_low as u64) > MAX_READ_BYTES {
            return Err(WorkspaceError::TooLarge);
        }
        let mut bytes = Vec::new();
        (&mut file)
            .take(MAX_READ_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| WorkspaceError::Io)?;
        if bytes.len() as u64 > MAX_READ_BYTES {
            return Err(WorkspaceError::TooLarge);
        }
        if check_info(&file_info(&file)?, false)? != id {
            return Err(WorkspaceError::Conflict);
        }
        let text = String::from_utf8(bytes).map_err(|_| WorkspaceError::Denied)?;
        validate_text(&text, MAX_READ_BYTES)?;
        Ok((text, id))
    }

    pub(super) fn check_root(root: &Root) -> Result<(), WorkspaceError> {
        for ancestor in &root.ancestor_pins {
            check_info(&file_info(ancestor)?, true)?;
        }
        if check_info(&file_info(&root.anchor)?, true)? != root.id {
            return Err(WorkspaceError::Conflict);
        }
        let current = open_dir(&root.path, FILE_SHARE_READ | FILE_SHARE_WRITE)?;
        if check_info(&file_info(&current)?, true)? != root.id {
            return Err(WorkspaceError::Conflict);
        }
        Ok(())
    }

    pub(super) struct PinnedParent {
        // Fresh restrictive directory handles exclude write/delete handles
        // (including reparse mutation) while this operation is in progress.
        _pins: Vec<File>,
        pub target_path: PathBuf,
        pub id: FileIdentity,
    }

    impl PinnedParent {
        pub(super) fn contains_identity(
            &self,
            expected: FileIdentity,
        ) -> Result<bool, WorkspaceError> {
            for pin in &self._pins {
                if check_info(&file_info(pin)?, true)? == expected {
                    return Ok(true);
                }
            }
            Ok(false)
        }
    }

    pub(super) fn pin_creation_parent(
        root: &Root,
        components: &[String],
    ) -> Result<PinnedParent, WorkspaceError> {
        if components.is_empty() {
            return Err(WorkspaceError::Denied);
        }
        check_root(root)?;
        let ancestors = local_absolute(&root.path)?;
        let prefix = &root.path.to_str().ok_or(WorkspaceError::Denied)?[..3];
        let mut cursor = PathBuf::from(prefix);
        let mut pins = Vec::new();
        for part in ancestors {
            pins.push(open_dir(&cursor, FILE_SHARE_READ)?);
            cursor.push(part);
        }
        let root_pin = open_dir(&cursor, FILE_SHARE_READ)?;
        if check_info(&file_info(&root_pin)?, true)? != root.id {
            return Err(WorkspaceError::Conflict);
        }
        pins.push(root_pin);
        let mut parent_id = root.id;
        for part in &components[..components.len() - 1] {
            cursor.push(part);
            let pin = open_dir(&cursor, FILE_SHARE_READ)?;
            parent_id = check_info(&file_info(&pin)?, true)?;
            pins.push(pin);
        }
        cursor.push(components.last().ok_or(WorkspaceError::Denied)?);
        Ok(PinnedParent {
            _pins: pins,
            target_path: cursor,
            id: parent_id,
        })
    }

    pub(super) fn require_absent(path: &Path) -> Result<(), WorkspaceError> {
        match std::fs::symlink_metadata(path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Ok(_) => Err(WorkspaceError::Conflict),
            Err(_) => Err(WorkspaceError::Io),
        }
    }

    pub(super) fn create_new_exclusive(path: &Path) -> Result<File, WorkspaceError> {
        // OpenOptions::create_new maps to CREATE_NEW on Windows. share0 and
        // OPEN_REPARSE_POINT fail closed against pre-existing objects/handles.
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .share_mode(0)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)
            .map_err(|_| WorkspaceError::Conflict)?;
        check_info(&file_info(&file)?, false)?;
        Ok(file)
    }

    pub(super) fn verify_created(
        file: &mut File,
        expected_id: crate::new_file::Identity,
        expected_hash: &str,
        expected_bytes: usize,
    ) -> Result<(), WorkspaceError> {
        use std::io::{Seek as _, SeekFrom};
        if crate::new_file::identity(file)? != expected_id {
            return Err(WorkspaceError::Conflict);
        }
        file.seek(SeekFrom::Start(0))
            .map_err(|_| WorkspaceError::Io)?;
        let mut bytes = Vec::new();
        (&mut *file)
            .take(MAX_CHANGED_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| WorkspaceError::Io)?;
        if bytes.len() != expected_bytes
            || sha256_hex(&bytes) != expected_hash
            || crate::new_file::identity(file)? != expected_id
        {
            return Err(WorkspaceError::Conflict);
        }
        Ok(())
    }

    pub(super) fn journal_is_resolved(
        root: &Root,
        journal: &crate::new_file::Journal<'_>,
    ) -> Result<bool, WorkspaceError> {
        for nonce in journal.nonces()? {
            let intent: crate::new_file::Intent = journal.read(&nonce, "intent")?;
            let created: crate::new_file::Created = journal.read(&nonce, "created")?;
            let complete: crate::new_file::Complete = journal.read(&nonce, "complete")?;
            if intent.version != 1
                || created.version != 1
                || complete.version != 1
                || intent.nonce != nonce
                || created.nonce != nonce
                || complete.nonce != nonce
                || intent.root_identity != record_identity(root.id)
                || created.identity != complete.identity
                || intent.content_sha256 != complete.content_sha256
                || intent.content_bytes != complete.content_bytes
                || intent.content_bytes > MAX_CHANGED_BYTES
            {
                return Ok(false);
            }
            validate_id(&intent.conversation_id)?;
            validate_id(&intent.request_id)?;
            let components = parse_relative_path(&intent.relative_path)?;
            let parent = pin_creation_parent(root, &components)?;
            if record_identity(parent.id) != intent.parent_identity {
                return Ok(false);
            }
            let (content, current_id) = read_guarded(root, &components)?;
            if record_identity(current_id) != complete.identity
                || content.len() != complete.content_bytes
                || sha256_hex(content.as_bytes()) != complete.content_sha256
            {
                return Ok(false);
            }
        }
        journal.revalidate()?;
        journal.nonces()?;
        Ok(true)
    }

    pub(super) fn random_nonce() -> Result<String, WorkspaceError> {
        let mut bytes = [0u8; 16];
        // SAFETY: fixed writable 16-byte buffer, documented system RNG flag.
        let status = unsafe {
            BCryptGenRandom(
                std::ptr::null_mut(),
                bytes.as_mut_ptr(),
                16,
                BCRYPT_USE_SYSTEM_PREFERRED_RNG,
            )
        };
        if status != 0 {
            return Err(WorkspaceError::Unavailable);
        }
        Ok(bytes.iter().map(|b| format!("{b:02x}")).collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proposal_fixture(operation: &str) -> serde_json::Value {
        serde_json::json!({
            "version": 1,
            "operation": operation,
            "relativePath": "src/example.txt",
            "expectedOldSha256": if operation == "replace_existing" {
                Some("a".repeat(64))
            } else { None },
            "newContent": "exact new content\n"
        })
    }

    #[test]
    fn proposal_schema_is_closed_and_never_an_approval() {
        for operation in ["replace_existing", "create_new"] {
            let fixture = proposal_fixture(operation);
            let edit = parse_proposed_edit(&fixture.to_string()).unwrap();
            assert_eq!(edit.new_content, "exact new content\n");
            assert_eq!(edit.relative_path, "src/example.txt");
            for (key, value) in [
                ("approved", serde_json::json!(true)),
                ("url", serde_json::json!("https://example.invalid")),
                ("tool", serde_json::json!("shell")),
                ("root", serde_json::json!("C:\\private")),
                ("requestId", serde_json::json!("forged")),
            ] {
                let mut forged = fixture.clone();
                forged
                    .as_object_mut()
                    .unwrap()
                    .insert(key.to_owned(), value);
                assert!(parse_proposed_edit(&forged.to_string()).is_err());
            }
            let encoded = fixture.to_string();
            for malformed in [
                format!("Explanation: {encoded}"),
                format!("{encoded}{encoded}"),
                format!("[{encoded}]"),
                "{\"version\":1,\"version\":1}".to_owned(),
            ] {
                assert!(parse_proposed_edit(&malformed).is_err());
            }
        }
    }

    #[test]
    fn proposal_paths_hashes_operations_versions_and_content_are_bounded() {
        let fixture = proposal_fixture("replace_existing");
        for (key, value) in [
            ("version", serde_json::json!(2)),
            ("operation", serde_json::json!("delete")),
            ("relativePath", serde_json::json!("../outside.txt")),
            ("relativePath", serde_json::json!("config/.env")),
            ("expectedOldSha256", serde_json::json!(null)),
            ("expectedOldSha256", serde_json::json!("A".repeat(64))),
            ("expectedOldSha256", serde_json::json!("a".repeat(63))),
            ("newContent", serde_json::json!("hidden\u{202e}content")),
            ("newContent", serde_json::json!("bad\0content")),
            (
                "newContent",
                serde_json::json!("x".repeat(MAX_READ_BYTES as usize + 1)),
            ),
        ] {
            let mut forged = fixture.clone();
            forged
                .as_object_mut()
                .unwrap()
                .insert(key.to_owned(), value);
            assert!(
                parse_proposed_edit(&forged.to_string()).is_err(),
                "accepted {key}"
            );
        }
        let mut create = proposal_fixture("create_new");
        create["expectedOldSha256"] = serde_json::json!("a".repeat(64));
        assert!(parse_proposed_edit(&create.to_string()).is_err());
        create["expectedOldSha256"] = serde_json::Value::Null;
        create["newContent"] = serde_json::json!("x".repeat(MAX_CHANGED_BYTES + 1));
        assert!(parse_proposed_edit(&create.to_string()).is_err());
        assert_eq!(
            parse_proposed_edit(&" ".repeat(MAX_PROPOSAL_BYTES + 1)).unwrap_err(),
            WorkspaceError::TooLarge
        );
    }

    #[test]
    fn journal_capacity_reserves_the_whole_next_operation() {
        assert_eq!(require_journal_capacity(0), Ok(()));
        assert_eq!(require_journal_capacity(63), Ok(()));
        assert_eq!(require_journal_capacity(64), Err(WorkspaceError::TooLarge));
        assert_eq!(require_journal_capacity(65), Err(WorkspaceError::TooLarge));
    }

    #[cfg(windows)]
    #[test]
    fn sha256_hex_matches_known_vector() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn malicious_windows_paths_are_rejected_even_on_non_windows_hosts() {
        for path in [
            "../x",
            "a/../x",
            "a\\..\\x",
            "C:\\x",
            "\\\\srv\\file",
            "a:x",
            "a/.git/config",
            "A/.ENV.local",
            "foo~1.txt",
            "CON.txt",
            "COM¹.txt",
            "aux",
            "a/secret.key",
            "a/.ssh/id_rsa",
            "a//b",
            "a\\\\b",
            "a/naïve.",
            "re\u{202e}adme.txt",
        ] {
            assert!(parse_relative_path(path).is_err(), "accepted {path:?}");
        }
        for base in ["COM¹", "COM²", "COM³", "LPT¹", "LPT²", "LPT³"] {
            assert!(parse_relative_path(base).is_err(), "accepted {base}");
            assert!(
                parse_relative_path(&format!("dir/{base}.txt")).is_err(),
                "accepted {base}.txt"
            );
        }
        assert_eq!(
            join_relative(&parse_relative_path("src/lib.rs").unwrap()),
            "src/lib.rs"
        );
    }

    #[test]
    fn exact_changed_bytes_respect_unicode_boundaries_and_limits() {
        assert_eq!(changed_bytes("aéz", "a🦊z"), (2, 4));
        assert_eq!(changed_bytes("equal", "equal"), (0, 0));
        assert_eq!(changed_bytes("abc", "xbc"), (1, 1));
        assert!(
            validate_text(&"a".repeat(MAX_CHANGED_BYTES + 1), MAX_CHANGED_BYTES as u64).is_err()
        );
        assert_eq!(
            validate_text("safe\u{202e}unsafe", MAX_READ_BYTES),
            Err(WorkspaceError::Denied)
        );
    }

    #[test]
    fn no_renderer_boolean_or_proposal_nonce_can_publish() {
        let service = Workspace::default();
        assert_eq!(
            service.apply_proposal("attacker-value").unwrap_err(),
            WorkspaceError::Unavailable
        );
        assert_eq!(
            service
                .read_project_file("conv", "req", "src/lib.rs")
                .unwrap_err(),
            if cfg!(windows) {
                WorkspaceError::Denied
            } else {
                WorkspaceError::Unavailable
            }
        );
    }

    #[cfg(windows)]
    struct NativeFixture {
        dir: PathBuf,
        workspace: Workspace,
    }

    #[cfg(windows)]
    impl NativeFixture {
        fn new() -> Self {
            // Hosted runners may place TEMP on a volume or ancestor that the
            // production NTFS policy rightly rejects. Try the user's local
            // application-data directory first, then TEMP. A candidate must
            // pass the exact production root admission; no gate is bypassed.
            let mut attempts = Vec::new();
            for base in [
                std::env::var_os("LOCALAPPDATA").map(PathBuf::from),
                Some(std::env::temp_dir()),
            ]
            .into_iter()
            .flatten()
            {
                let dir = base.join(format!(
                    "aster-workspace-test-{}",
                    windows_fs::random_nonce().unwrap()
                ));
                if let Err(error) = std::fs::create_dir(&dir) {
                    attempts.push(format!("{}: create {error}", dir.display()));
                    continue;
                }
                let workspace = Workspace::default();
                match workspace.set_native_root(dir.clone()) {
                    Ok(_) => return Self { dir, workspace },
                    Err(error) => {
                        attempts.push(format!("{}: {error}", dir.display()));
                        std::fs::remove_dir(&dir).unwrap();
                    }
                }
            }
            panic!("No eligible fixed local NTFS fixture directory: {attempts:?}");
        }
    }

    #[cfg(windows)]
    impl Drop for NativeFixture {
        fn drop(&mut self) {
            // Drop the pinned root handle before removing the test directory.
            let previous = std::mem::take(&mut self.workspace);
            drop(previous);
            std::fs::remove_dir_all(&self.dir).unwrap();
        }
    }

    #[cfg(windows)]
    #[test]
    fn selected_file_grant_is_one_use_and_existing_proposal_never_mutates() {
        let fixture = NativeFixture::new();
        let path = fixture.dir.join("allowed.txt");
        std::fs::write(&path, "old content\n").unwrap();
        let selected = fixture
            .workspace
            .grant_native_file_read("conv", "req", path.clone())
            .unwrap();
        assert_eq!(selected.relative_path, "allowed.txt");
        let read = fixture
            .workspace
            .read_project_file("conv", "req", "allowed.txt")
            .unwrap();
        assert_eq!(read.content, "old content\n");
        assert_eq!(
            fixture
                .workspace
                .read_project_file("conv", "req", "allowed.txt")
                .unwrap_err(),
            WorkspaceError::Denied
        );
        let proposal = fixture
            .workspace
            .review_proposal(
                "conv",
                "req",
                "allowed.txt",
                "new content\n",
                Some(&read.sha256),
            )
            .unwrap();
        assert_eq!(proposal.status, "proposal_only");
        assert!(!proposal.apply_available);
        let exact: serde_json::Value = serde_json::from_str(&proposal.diff).unwrap();
        assert_eq!(exact["before"]["content"], "old content\n");
        assert_eq!(exact["after"]["content"], "new content\n");
        assert_eq!(
            fixture
                .workspace
                .apply_proposal(&proposal.proposal_id)
                .unwrap_err(),
            WorkspaceError::Unavailable
        );
        assert_eq!(std::fs::read_to_string(path).unwrap(), "old content\n");
        assert_eq!(
            fixture
                .workspace
                .review_proposal("conv", "req", "allowed.txt", "third\n", Some(&read.sha256))
                .unwrap_err(),
            WorkspaceError::Denied
        );
    }

    #[cfg(windows)]
    #[test]
    fn mutation_after_native_selection_fails_without_exposing_changed_contents() {
        let fixture = NativeFixture::new();
        let path = fixture.dir.join("drift.txt");
        std::fs::write(&path, "before").unwrap();
        fixture
            .workspace
            .grant_native_file_read("conv", "req", path.clone())
            .unwrap();
        std::fs::write(&path, "after").unwrap();
        assert_eq!(
            fixture
                .workspace
                .read_project_file("conv", "req", "drift.txt")
                .unwrap_err(),
            WorkspaceError::Conflict
        );
        assert_eq!(
            fixture
                .workspace
                .read_project_file("conv", "req", "drift.txt")
                .unwrap_err(),
            WorkspaceError::Denied
        );
    }

    #[cfg(windows)]
    #[test]
    fn disclosure_revalidates_identity_hash_and_consumes_failed_or_successful_use() {
        let fixture = NativeFixture::new();
        let path = fixture.dir.join("disclosure.txt");
        std::fs::write(&path, "original").unwrap();
        fixture
            .workspace
            .grant_native_file_read("conv", "req", path.clone())
            .unwrap();
        let read = fixture
            .workspace
            .read_project_file("conv", "req", "disclosure.txt")
            .unwrap();
        std::fs::write(&path, "changed").unwrap();
        assert_eq!(
            fixture
                .workspace
                .take_disclosure("conv", "req", "disclosure.txt", &read.sha256)
                .unwrap_err(),
            WorkspaceError::Conflict
        );
        std::fs::write(&path, "original").unwrap();
        assert_eq!(
            fixture
                .workspace
                .take_disclosure("conv", "req", "disclosure.txt", &read.sha256)
                .unwrap_err(),
            WorkspaceError::Denied
        );
        fixture
            .workspace
            .grant_native_file_read("conv", "req2", path)
            .unwrap();
        let read = fixture
            .workspace
            .read_project_file("conv", "req2", "disclosure.txt")
            .unwrap();
        assert_eq!(
            fixture
                .workspace
                .take_disclosure("conv", "req2", "disclosure.txt", &read.sha256)
                .unwrap()
                .content,
            "original"
        );
        assert_eq!(
            fixture
                .workspace
                .take_disclosure("conv", "req2", "disclosure.txt", &read.sha256)
                .unwrap_err(),
            WorkspaceError::Denied
        );
    }

    #[cfg(windows)]
    fn prepared_disclosure_fixture(
        fixture: &NativeFixture,
        conversation: &str,
        request: &str,
    ) -> (crate::native_review::ReviewDocument, Vec<u8>) {
        let path = fixture.dir.join("disclosure.txt");
        std::fs::write(&path, "synthetic selected content").unwrap();
        fixture
            .workspace
            .grant_native_file_read(conversation, request, path)
            .unwrap();
        let read = fixture
            .workspace
            .read_project_file(conversation, request, "disclosure.txt")
            .unwrap();
        let exact = crate::provider::exact_body(
            crate::policy::Model::Nemotron,
            &[crate::policy::ChatMessage {
                role: "user",
                content: format!("Review this explicit synthetic snapshot:\n{}", read.content),
            }],
        )
        .unwrap();
        let review = fixture
            .workspace
            .prepare_disclosure_candidate(
                conversation,
                request,
                "disclosure.txt",
                &read.sha256,
                exact.clone(),
            )
            .unwrap();
        (review, exact)
    }

    #[cfg(windows)]
    #[test]
    fn final_disclosure_barrier_returns_exact_bytes_once_and_denies_changed_file_root_and_expiry() {
        let fixture = NativeFixture::new();
        let (review, exact) = prepared_disclosure_fixture(&fixture, "conv", "req");
        let approved = crate::native_review::NativeApproval::for_fixture(review.binding_sha256());
        assert_eq!(
            fixture
                .workspace
                .finalize_disclosure_candidate("conv", "req", approved)
                .unwrap(),
            exact
        );
        assert_eq!(
            fixture
                .workspace
                .finalize_disclosure_candidate(
                    "conv",
                    "req",
                    crate::native_review::NativeApproval::for_fixture(review.binding_sha256())
                )
                .unwrap_err(),
            WorkspaceError::Denied
        );
        let (review, _) = prepared_disclosure_fixture(&fixture, "conv", "drift");
        std::fs::write(
            fixture.dir.join("disclosure.txt"),
            "changed after native review",
        )
        .unwrap();
        assert_eq!(
            fixture
                .workspace
                .finalize_disclosure_candidate(
                    "conv",
                    "drift",
                    crate::native_review::NativeApproval::for_fixture(review.binding_sha256())
                )
                .unwrap_err(),
            WorkspaceError::Conflict
        );
        let (review, _) = prepared_disclosure_fixture(&fixture, "conv", "root");
        fixture
            .workspace
            .set_native_root(fixture.dir.clone())
            .unwrap();
        assert_eq!(
            fixture
                .workspace
                .finalize_disclosure_candidate(
                    "conv",
                    "root",
                    crate::native_review::NativeApproval::for_fixture(review.binding_sha256())
                )
                .unwrap_err(),
            WorkspaceError::Denied
        );
        let (review, _) = prepared_disclosure_fixture(&fixture, "conv", "expiry");
        fixture
            .workspace
            .state
            .lock()
            .unwrap()
            .snapshots
            .get_mut(&("conv".into(), "expiry".into()))
            .unwrap()
            .deadline = Instant::now();
        assert_eq!(
            fixture
                .workspace
                .finalize_disclosure_candidate(
                    "conv",
                    "expiry",
                    crate::native_review::NativeApproval::for_fixture(review.binding_sha256())
                )
                .unwrap_err(),
            WorkspaceError::Denied
        );
    }

    #[cfg(windows)]
    #[test]
    fn same_outbound_body_cannot_reuse_native_disclosure_approval_across_requests() {
        let fixture = NativeFixture::new();
        let (review_one, body_one) = prepared_disclosure_fixture(&fixture, "conv_one", "one");
        let (review_two, body_two) = prepared_disclosure_fixture(&fixture, "conv_two", "two");
        assert_eq!(body_one, body_two);
        assert_ne!(review_one.binding_sha256(), review_two.binding_sha256());
        assert_eq!(
            fixture
                .workspace
                .finalize_disclosure_candidate(
                    "conv_two",
                    "two",
                    crate::native_review::NativeApproval::for_fixture(review_one.binding_sha256())
                )
                .unwrap_err(),
            WorkspaceError::Denied
        );
        assert_eq!(
            fixture
                .workspace
                .finalize_disclosure_candidate(
                    "conv_two",
                    "two",
                    crate::native_review::NativeApproval::for_fixture(review_two.binding_sha256())
                )
                .unwrap_err(),
            WorkspaceError::Denied
        );
        assert_eq!(
            fixture
                .workspace
                .finalize_disclosure_candidate(
                    "conv_one",
                    "one",
                    crate::native_review::NativeApproval::for_fixture(review_one.binding_sha256())
                )
                .unwrap(),
            body_one
        );
    }

    #[cfg(windows)]
    #[test]
    fn disclosure_preparation_denies_old_material_after_same_key_root_reset_and_regrant() {
        let fixture = NativeFixture::new();
        let old_path = fixture.dir.join("disclosure.txt");
        std::fs::write(&old_path, "old root material").unwrap();
        fixture
            .workspace
            .grant_native_file_read("conv", "req", old_path)
            .unwrap();
        let old = fixture
            .workspace
            .read_project_file("conv", "req", "disclosure.txt")
            .unwrap();
        let old_body = crate::provider::exact_body(
            crate::policy::Model::Nemotron,
            &[crate::policy::ChatMessage {
                role: "user",
                content: old.content,
            }],
        )
        .unwrap();
        let new_root = fixture.dir.join("different-root");
        std::fs::create_dir(&new_root).unwrap();
        let new_path = new_root.join("disclosure.txt");
        std::fs::write(&new_path, "new root material").unwrap();
        fixture.workspace.set_native_root(new_root).unwrap();
        fixture
            .workspace
            .grant_native_file_read("conv", "req", new_path)
            .unwrap();
        let new = fixture
            .workspace
            .read_project_file("conv", "req", "disclosure.txt")
            .unwrap();
        assert_ne!(old.sha256, new.sha256);
        assert!(matches!(
            fixture.workspace.prepare_disclosure_candidate(
                "conv",
                "req",
                "disclosure.txt",
                &old.sha256,
                old_body
            ),
            Err(WorkspaceError::Denied)
        ));
        assert_eq!(
            fixture
                .workspace
                .finalize_disclosure_candidate(
                    "conv",
                    "req",
                    crate::native_review::NativeApproval::for_fixture(&"0".repeat(64))
                )
                .unwrap_err(),
            WorkspaceError::Denied
        );
    }

    #[cfg(windows)]
    #[test]
    fn expired_grants_snapshots_and_changed_roots_cannot_be_reused() {
        let fixture = NativeFixture::new();
        let path = fixture.dir.join("expiry.txt");
        std::fs::write(&path, "content").unwrap();
        fixture
            .workspace
            .grant_native_file_read("conv", "expired", path.clone())
            .unwrap();
        fixture
            .workspace
            .state
            .lock()
            .unwrap()
            .read_grants
            .get_mut(&("conv".into(), "expired".into()))
            .unwrap()
            .deadline = Instant::now();
        assert_eq!(
            fixture
                .workspace
                .read_project_file("conv", "expired", "expiry.txt")
                .unwrap_err(),
            WorkspaceError::Denied
        );
        fixture
            .workspace
            .grant_native_file_read("conv", "snapshot", path.clone())
            .unwrap();
        let read = fixture
            .workspace
            .read_project_file("conv", "snapshot", "expiry.txt")
            .unwrap();
        fixture
            .workspace
            .state
            .lock()
            .unwrap()
            .snapshots
            .get_mut(&("conv".into(), "snapshot".into()))
            .unwrap()
            .deadline = Instant::now();
        assert_eq!(
            fixture
                .workspace
                .take_disclosure("conv", "snapshot", "expiry.txt", &read.sha256)
                .unwrap_err(),
            WorkspaceError::Denied
        );
        assert_eq!(
            fixture
                .workspace
                .review_proposal("conv", "snapshot", "expiry.txt", "new", Some(&read.sha256))
                .unwrap_err(),
            WorkspaceError::Denied
        );
        fixture
            .workspace
            .grant_native_file_read("conv", "root", path)
            .unwrap();
        let read = fixture
            .workspace
            .read_project_file("conv", "root", "expiry.txt")
            .unwrap();
        fixture
            .workspace
            .set_native_root(fixture.dir.clone())
            .unwrap();
        assert_eq!(
            fixture
                .workspace
                .take_disclosure("conv", "root", "expiry.txt", &read.sha256)
                .unwrap_err(),
            WorkspaceError::Denied
        );
        assert_eq!(
            fixture
                .workspace
                .review_proposal("conv", "root", "expiry.txt", "new", Some(&read.sha256))
                .unwrap_err(),
            WorkspaceError::Denied
        );
    }

    #[cfg(windows)]
    #[test]
    fn selected_root_pins_its_ancestor_chain_and_replacement_identity_conflicts() {
        let fixture = NativeFixture::new();
        let parent = fixture.dir.join("parent");
        let child = parent.join("child");
        std::fs::create_dir_all(&child).unwrap();
        fixture.workspace.set_native_root(child.clone()).unwrap();
        assert!(std::fs::rename(&parent, fixture.dir.join("moved-parent")).is_err());
        let path = child.join("identity.txt");
        std::fs::write(&path, "same content").unwrap();
        fixture
            .workspace
            .grant_native_file_read("conv", "req", path.clone())
            .unwrap();
        let read = fixture
            .workspace
            .read_project_file("conv", "req", "identity.txt")
            .unwrap();
        std::fs::rename(&path, child.join("old-identity.txt")).unwrap();
        std::fs::write(&path, "same content").unwrap();
        assert_eq!(
            fixture
                .workspace
                .take_disclosure("conv", "req", "identity.txt", &read.sha256)
                .unwrap_err(),
            WorkspaceError::Conflict
        );
        assert_eq!(
            fixture
                .workspace
                .review_proposal("conv", "req", "identity.txt", "new", Some(&read.sha256))
                .unwrap_err(),
            WorkspaceError::Conflict
        );
    }

    #[cfg(windows)]
    fn new_file_fixture(
        fixture: &NativeFixture,
        relative_path: &str,
        content: &str,
        journal_name: &str,
    ) -> (ProposalReview, PathBuf) {
        let journal = private_journal_fixture(fixture, journal_name);
        let json = serde_json::json!({
            "version": 1, "operation": "create_new",
            "relativePath": relative_path, "expectedOldSha256": null,
            "newContent": content,
        });
        let proposal = fixture
            .workspace
            .prepare_new_file_proposal("conv", "req", &json.to_string(), journal.clone())
            .unwrap();
        assert!(!proposal.apply_available);
        assert_eq!(proposal.status, "candidate_only");
        (proposal, journal)
    }

    #[cfg(windows)]
    fn private_journal_fixture(fixture: &NativeFixture, name: &str) -> PathBuf {
        let components = parse_relative_path(name).unwrap();
        assert_eq!(components.len(), 1);
        let state = fixture.workspace.state.lock().unwrap();
        let root = state.root.as_ref().unwrap();
        // Only a new synthetic child; the exact production parent/ancestor
        // pins stay alive through creation and admission. No existing ACL repair.
        let parent = windows_fs::pin_creation_parent(root, &components).unwrap();
        windows_fs::require_absent(&parent.target_path).unwrap();
        crate::journal_privacy::create_directory_exact(&parent.target_path).unwrap();
        let selected = windows_fs::select_root(parent.target_path.clone()).unwrap();
        let journal = crate::new_file::Journal::in_pinned_directory(
            &selected.path,
            &selected.anchor,
            &selected.ancestor_pins,
            windows_fs::record_identity(selected.id),
        )
        .unwrap();
        assert_eq!(journal.nonces().unwrap().len(), 0);
        selected.path.clone()
    }

    #[cfg(windows)]
    fn synthetic_approval(proposal: &ProposalReview) -> crate::native_review::NativeApproval {
        let review = crate::native_review::ReviewDocument::proposal(&proposal.diff).unwrap();
        // This bypasses native UI ONLY in cfg(test). It proves protocol shape,
        // never owner consent or Windows 11 UI acceptance.
        crate::native_review::NativeApproval::for_fixture(review.binding_sha256())
    }

    #[cfg(windows)]
    #[test]
    fn journal_privacy_rejects_weak_existing_objects_and_preserves_creation_collisions() {
        use crate::journal_privacy::{self, ObjectKind};
        use std::io::Write;
        let fixture = NativeFixture::new();
        let weak = fixture.dir.join("weak-journal");
        journal_privacy::create_directory_exact(&weak).unwrap();
        let acl = journal_privacy::fixture_acl_handle(&weak, ObjectKind::Directory).unwrap();
        let owner = journal_privacy::OwnerSid::current_process().unwrap();
        owner.validate_handle(&acl, ObjectKind::Directory).unwrap();
        // Deliberately weaken only the freshly owned disposable object.
        journal_privacy::fixture_set_private(&acl, ObjectKind::Directory, false).unwrap();
        // SetSecurityInfo propagates the parent's current inheritance model
        // to existing children. Create the sentinel only after that setup,
        // with an explicit protected descriptor at CREATE_NEW; never repair
        // an inherited child ACL after a failed preservation assertion.
        let sentinel_path = weak.join("sentinel.txt");
        let mut sentinel = journal_privacy::create_record_exact(&sentinel_path, &owner).unwrap();
        sentinel.write_all(b"unchanged synthetic marker").unwrap();
        sentinel.sync_all().unwrap();
        drop(sentinel);
        let sentinel_acl =
            journal_privacy::fixture_acl_handle(&sentinel_path, ObjectKind::Record).unwrap();
        let sentinel_before = journal_privacy::fixture_descriptor(&sentinel_acl).unwrap();
        let sentinel_identity =
            journal_privacy::object_identity(&sentinel_acl, ObjectKind::Record).unwrap();
        let sentinel_content = std::fs::read_to_string(&sentinel_path).unwrap();
        assert_eq!(sentinel_content, "unchanged synthetic marker");
        let before = journal_privacy::fixture_descriptor(&acl).unwrap();
        let identity = journal_privacy::object_identity(&acl, ObjectKind::Directory).unwrap();
        let selected = windows_fs::select_root(weak.clone()).unwrap();
        assert!(crate::new_file::Journal::in_pinned_directory(
            &selected.path,
            &selected.anchor,
            &selected.ancestor_pins,
            windows_fs::record_identity(selected.id),
        )
        .is_err());
        assert!(journal_privacy::create_directory_exact(&weak).is_err());
        assert_eq!(journal_privacy::fixture_descriptor(&acl).unwrap(), before);
        assert_eq!(
            journal_privacy::object_identity(&acl, ObjectKind::Directory).unwrap(),
            identity
        );
        assert_eq!(
            journal_privacy::fixture_descriptor(&sentinel_acl).unwrap(),
            sentinel_before
        );
        assert_eq!(
            journal_privacy::object_identity(&sentinel_acl, ObjectKind::Record).unwrap(),
            sentinel_identity
        );
        assert_eq!(
            std::fs::read_to_string(&sentinel_path).unwrap(),
            sentinel_content
        );

        let private = private_journal_fixture(&fixture, "private-journal");
        let acl = journal_privacy::fixture_acl_handle(&private, ObjectKind::Directory).unwrap();
        let before = journal_privacy::fixture_descriptor(&acl).unwrap();
        let identity = journal_privacy::object_identity(&acl, ObjectKind::Directory).unwrap();
        assert!(journal_privacy::create_directory_exact(&private).is_err());
        assert_eq!(journal_privacy::fixture_descriptor(&acl).unwrap(), before);
        assert_eq!(
            journal_privacy::object_identity(&acl, ObjectKind::Directory).unwrap(),
            identity
        );
        assert_eq!(std::fs::read_dir(private).unwrap().count(), 0);
    }

    #[cfg(windows)]
    #[test]
    fn journal_privacy_binds_path_identity_and_retains_directory_lifetime_pins() {
        let fixture = NativeFixture::new();
        let first = private_journal_fixture(&fixture, "first-journal");
        let second = private_journal_fixture(&fixture, "second-journal");
        let root = windows_fs::select_root(first.clone()).unwrap();
        assert!(crate::new_file::Journal::in_pinned_directory(
            &second,
            &root.anchor,
            &root.ancestor_pins,
            windows_fs::record_identity(root.id),
        )
        .is_err());
        let mut wrong = windows_fs::record_identity(root.id);
        wrong.index ^= 1;
        assert!(crate::new_file::Journal::in_pinned_directory(
            &first,
            &root.anchor,
            &root.ancestor_pins,
            wrong,
        )
        .is_err());
        let journal = crate::new_file::Journal::in_pinned_directory(
            &first,
            &root.anchor,
            &root.ancestor_pins,
            windows_fs::record_identity(root.id),
        )
        .unwrap();
        assert!(std::fs::rename(&first, fixture.dir.join("renamed-journal")).is_err());
        assert!(journal.nonces().unwrap().is_empty());
    }

    #[cfg(windows)]
    #[test]
    fn journal_privacy_directory_drift_denies_before_intent_without_target_or_records() {
        use crate::journal_privacy::{self, ObjectKind};
        let fixture = NativeFixture::new();
        let journal = private_journal_fixture(&fixture, "journal");
        let acl = journal_privacy::fixture_acl_handle(&journal, ObjectKind::Directory).unwrap();
        journal_privacy::fixture_set_private(&acl, ObjectKind::Directory, false).unwrap();
        let before = journal_privacy::fixture_descriptor(&acl).unwrap();
        let json = serde_json::json!({"version":1,"operation":"create_new","relativePath":"untouched.txt","expectedOldSha256":null,"newContent":"synthetic"});
        assert!(fixture
            .workspace
            .prepare_new_file_proposal("conv", "req", &json.to_string(), journal.clone())
            .is_err());
        assert!(!fixture.dir.join("untouched.txt").exists());
        assert_eq!(std::fs::read_dir(&journal).unwrap().count(), 0);
        assert_eq!(journal_privacy::fixture_descriptor(&acl).unwrap(), before);
        assert_eq!(
            fixture
                .workspace
                .inspect_creation_recovery(journal)
                .unwrap(),
            CreationOutcome::AttentionRequired
        );
    }

    #[cfg(windows)]
    #[test]
    fn journal_privacy_record_drift_blocks_read_recovery_and_followup_without_repair() {
        use crate::journal_privacy::{self, ObjectKind};
        let fixture = NativeFixture::new();
        let (proposal, directory) =
            new_file_fixture(&fixture, "created.txt", "synthetic content", "journal");
        assert_eq!(
            fixture
                .workspace
                .apply_new_file_candidate(&proposal.proposal_id, synthetic_approval(&proposal))
                .unwrap(),
            CreationOutcome::Verified
        );
        let path = directory.join(format!("{}.intent", proposal.proposal_id));
        let acl = journal_privacy::fixture_acl_handle(&path, ObjectKind::Record).unwrap();
        let content = std::fs::read(&path).unwrap();
        journal_privacy::fixture_set_private(&acl, ObjectKind::Record, false).unwrap();
        let before = journal_privacy::fixture_descriptor(&acl).unwrap();
        let selected = windows_fs::select_root(directory.clone()).unwrap();
        let journal = crate::new_file::Journal::in_pinned_directory(
            &selected.path,
            &selected.anchor,
            &selected.ancestor_pins,
            windows_fs::record_identity(selected.id),
        )
        .unwrap();
        assert!(journal
            .read::<crate::new_file::Intent>(&proposal.proposal_id, "intent")
            .is_err());
        assert!(journal.nonces().is_err());
        assert_eq!(
            fixture
                .workspace
                .inspect_creation_recovery(directory.clone())
                .unwrap(),
            CreationOutcome::AttentionRequired
        );
        let json = serde_json::json!({"version":1,"operation":"create_new","relativePath":"blocked.txt","expectedOldSha256":null,"newContent":"another"});
        assert_eq!(
            fixture
                .workspace
                .prepare_new_file_proposal("conv", "newreq", &json.to_string(), directory)
                .unwrap_err(),
            WorkspaceError::AttentionRequired
        );
        assert!(!fixture.dir.join("blocked.txt").exists());
        assert_eq!(std::fs::read(&path).unwrap(), content);
        assert_eq!(journal_privacy::fixture_descriptor(&acl).unwrap(), before);
        assert_eq!(
            std::fs::read_to_string(fixture.dir.join("created.txt")).unwrap(),
            "synthetic content"
        );
    }

    #[cfg(windows)]
    #[test]
    fn journal_privacy_drift_after_intent_or_creation_preserves_ambiguity_and_denies_followup() {
        use crate::journal_privacy::{self, ObjectKind};
        for boundary in [
            CreationStage::Intent,
            CreationStage::Created,
            CreationStage::IdentityRecorded,
            CreationStage::Complete,
        ] {
            let fixture = NativeFixture::new();
            let (proposal, directory) =
                new_file_fixture(&fixture, "interrupted.txt", "synthetic content", "journal");
            let acl =
                journal_privacy::fixture_acl_handle(&directory, ObjectKind::Directory).unwrap();
            let outcome = fixture
                .workspace
                .apply_new_file_with_stage(
                    &proposal.proposal_id,
                    synthetic_approval(&proposal),
                    |stage| {
                        if stage == boundary {
                            journal_privacy::fixture_set_private(
                                &acl,
                                ObjectKind::Directory,
                                false,
                            )?;
                        }
                        Ok(())
                    },
                )
                .unwrap();
            assert_eq!(outcome, CreationOutcome::AttentionRequired);
            assert_eq!(
                fixture.dir.join("interrupted.txt").exists(),
                boundary != CreationStage::Intent
            );
            let count = std::fs::read_dir(&directory).unwrap().count();
            assert!(count > 0);
            let records_before = synthetic_journal_snapshot(&directory);
            let target_before = std::fs::read(fixture.dir.join("interrupted.txt")).ok();
            let descriptor_before = journal_privacy::fixture_descriptor(&acl).unwrap();
            assert_eq!(
                fixture
                    .workspace
                    .inspect_creation_recovery(directory.clone())
                    .unwrap(),
                CreationOutcome::AttentionRequired
            );
            let json = serde_json::json!({"version":1,"operation":"create_new","relativePath":"followup.txt","expectedOldSha256":null,"newContent":"another synthetic file"});
            assert!(fixture
                .workspace
                .prepare_new_file_proposal("conv", "followup", &json.to_string(), directory.clone())
                .is_err());
            assert!(!fixture.dir.join("followup.txt").exists());
            assert_eq!(std::fs::read_dir(&directory).unwrap().count(), count);
            assert_eq!(synthetic_journal_snapshot(&directory), records_before);
            assert_eq!(
                std::fs::read(fixture.dir.join("interrupted.txt")).ok(),
                target_before
            );
            assert_eq!(
                journal_privacy::fixture_descriptor(&acl).unwrap(),
                descriptor_before
            );
        }
    }

    #[cfg(windows)]
    type SyntheticJournalSnapshot = std::collections::BTreeMap<String, (Vec<u8>, Vec<u8>)>;

    #[cfg(windows)]
    fn synthetic_journal_snapshot(directory: &std::path::Path) -> SyntheticJournalSnapshot {
        use crate::journal_privacy::{self, ObjectKind};
        std::fs::read_dir(directory)
            .unwrap()
            .map(|entry| {
                let entry = entry.unwrap();
                let handle =
                    journal_privacy::fixture_acl_handle(&entry.path(), ObjectKind::Record).unwrap();
                (
                    entry.file_name().into_string().unwrap(),
                    (
                        std::fs::read(entry.path()).unwrap(),
                        journal_privacy::fixture_descriptor(&handle).unwrap(),
                    ),
                )
            })
            .collect()
    }

    #[cfg(windows)]
    #[test]
    fn journal_privacy_intent_record_drift_denies_target_creation_or_content_and_followup() {
        use crate::journal_privacy::{self, ObjectKind};
        for boundary in [CreationStage::Intent, CreationStage::IdentityRecorded] {
            let fixture = NativeFixture::new();
            let (proposal, directory) =
                new_file_fixture(&fixture, "unchanged.txt", "must not be written", "journal");
            let intent_path = directory.join(format!("{}.intent", proposal.proposal_id));
            let mut retained_intent = None;
            let outcome = fixture
                .workspace
                .apply_new_file_with_stage(
                    &proposal.proposal_id,
                    synthetic_approval(&proposal),
                    |stage| {
                        if stage == boundary {
                            let handle = journal_privacy::fixture_acl_handle(
                                &intent_path,
                                ObjectKind::Record,
                            )?;
                            let ciphertext = std::fs::read(&intent_path).unwrap();
                            journal_privacy::fixture_set_private(
                                &handle,
                                ObjectKind::Record,
                                false,
                            )?;
                            retained_intent =
                                Some((ciphertext, journal_privacy::fixture_descriptor(&handle)?));
                        }
                        Ok(())
                    },
                )
                .unwrap();
            assert_eq!(outcome, CreationOutcome::AttentionRequired);
            let target_before = std::fs::read(fixture.dir.join("unchanged.txt")).ok();
            if boundary == CreationStage::Intent {
                assert_eq!(target_before, None);
                assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 1);
            } else {
                assert_eq!(target_before, Some(Vec::new()));
                assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 2);
            }
            assert!(!directory
                .join(format!("{}.complete", proposal.proposal_id))
                .exists());
            let records_before = synthetic_journal_snapshot(&directory);
            let (intent_ciphertext, intent_descriptor) = retained_intent.unwrap();
            assert_eq!(std::fs::read(&intent_path).unwrap(), intent_ciphertext);
            let handle =
                journal_privacy::fixture_acl_handle(&intent_path, ObjectKind::Record).unwrap();
            assert_eq!(
                journal_privacy::fixture_descriptor(&handle).unwrap(),
                intent_descriptor
            );
            assert_eq!(
                fixture
                    .workspace
                    .inspect_creation_recovery(directory.clone())
                    .unwrap(),
                CreationOutcome::AttentionRequired
            );
            let json = serde_json::json!({"version":1,"operation":"create_new","relativePath":"followup.txt","expectedOldSha256":null,"newContent":"another synthetic file"});
            assert_eq!(
                fixture
                    .workspace
                    .prepare_new_file_proposal(
                        "conv",
                        "followup",
                        &json.to_string(),
                        directory.clone()
                    )
                    .unwrap_err(),
                WorkspaceError::AttentionRequired
            );
            assert!(!fixture.dir.join("followup.txt").exists());
            assert_eq!(synthetic_journal_snapshot(&directory), records_before);
            assert_eq!(
                std::fs::read(fixture.dir.join("unchanged.txt")).ok(),
                target_before
            );
            assert_eq!(std::fs::read(&intent_path).unwrap(), intent_ciphertext);
            assert_eq!(
                journal_privacy::fixture_descriptor(&handle).unwrap(),
                intent_descriptor
            );
        }
    }

    #[cfg(windows)]
    #[test]
    fn new_file_candidate_uses_exact_one_use_approval_and_encrypted_immutable_records() {
        let fixture = NativeFixture::new();
        let (proposal, journal) = new_file_fixture(
            &fixture,
            "created.txt",
            "synthetic new file content\n",
            "journal",
        );
        let exact: serde_json::Value = serde_json::from_str(&proposal.diff).unwrap();
        assert_eq!(exact["after"]["content"], "synthetic new file content\n");
        assert_eq!(exact["uses"], 1);
        assert!(exact["rootIdentity"]["index"].is_u64());
        assert_eq!(exact["rootPath"], fixture.dir.to_str().unwrap());
        assert_eq!(
            exact["targetPath"],
            fixture.dir.join("created.txt").to_str().unwrap()
        );
        assert_eq!(
            fixture
                .workspace
                .apply_new_file_candidate(&proposal.proposal_id, synthetic_approval(&proposal))
                .unwrap(),
            CreationOutcome::Verified
        );
        assert_eq!(
            std::fs::read_to_string(fixture.dir.join("created.txt")).unwrap(),
            "synthetic new file content\n"
        );
        assert_eq!(
            fixture
                .workspace
                .apply_new_file_candidate(&proposal.proposal_id, synthetic_approval(&proposal))
                .unwrap_err(),
            WorkspaceError::Denied
        );
        assert_eq!(
            fixture
                .workspace
                .inspect_creation_recovery(journal.clone())
                .unwrap(),
            CreationOutcome::Verified
        );
        let records = std::fs::read_dir(&journal).unwrap().collect::<Vec<_>>();
        assert_eq!(records.len(), 3);
        for entry in records {
            let encrypted = std::fs::read(entry.unwrap().path()).unwrap();
            assert!(!encrypted
                .windows(b"synthetic new file content".len())
                .any(|w| w == b"synthetic new file content"));
        }
        // Recovery after a changed target never overwrites/restores it.
        std::fs::write(fixture.dir.join("created.txt"), "owner change").unwrap();
        assert_eq!(
            fixture
                .workspace
                .inspect_creation_recovery(journal)
                .unwrap(),
            CreationOutcome::AttentionRequired
        );
        assert_eq!(
            std::fs::read_to_string(fixture.dir.join("created.txt")).unwrap(),
            "owner change"
        );
    }

    #[cfg(windows)]
    #[test]
    fn intervening_creator_wrong_binding_expiry_and_parent_drift_are_denied() {
        let fixture = NativeFixture::new();
        let (proposal, journal) =
            new_file_fixture(&fixture, "existing.txt", "proposed", "journal-one");
        std::fs::write(fixture.dir.join("existing.txt"), "competing creator").unwrap();
        assert_eq!(
            fixture
                .workspace
                .apply_new_file_candidate(&proposal.proposal_id, synthetic_approval(&proposal))
                .unwrap_err(),
            WorkspaceError::Conflict
        );
        assert_eq!(
            std::fs::read_to_string(fixture.dir.join("existing.txt")).unwrap(),
            "competing creator"
        );
        assert_eq!(std::fs::read_dir(journal).unwrap().count(), 0);
        let (proposal, _) =
            new_file_fixture(&fixture, "wrong-binding.txt", "proposed", "journal-two");
        let wrong = crate::native_review::NativeApproval::for_fixture(&"0".repeat(64));
        assert_eq!(
            fixture
                .workspace
                .apply_new_file_candidate(&proposal.proposal_id, wrong)
                .unwrap_err(),
            WorkspaceError::Denied
        );
        assert!(!fixture.dir.join("wrong-binding.txt").exists());
        let (proposal, _) = new_file_fixture(&fixture, "expired.txt", "proposed", "journal-three");
        fixture
            .workspace
            .state
            .lock()
            .unwrap()
            .creations
            .get_mut(&proposal.proposal_id)
            .unwrap()
            .deadline = Instant::now();
        assert_eq!(
            fixture
                .workspace
                .apply_new_file_candidate(&proposal.proposal_id, synthetic_approval(&proposal))
                .unwrap_err(),
            WorkspaceError::Denied
        );
        assert!(!fixture.dir.join("expired.txt").exists());
        std::fs::create_dir(fixture.dir.join("parent")).unwrap();
        let (proposal, _) =
            new_file_fixture(&fixture, "parent/new.txt", "proposed", "journal-four");
        std::fs::rename(fixture.dir.join("parent"), fixture.dir.join("old-parent")).unwrap();
        std::fs::create_dir(fixture.dir.join("parent")).unwrap();
        assert_eq!(
            fixture
                .workspace
                .apply_new_file_candidate(&proposal.proposal_id, synthetic_approval(&proposal))
                .unwrap_err(),
            WorkspaceError::Conflict
        );
        assert!(!fixture.dir.join("parent/new.txt").exists());
    }

    #[cfg(windows)]
    #[test]
    fn creation_cannot_publish_into_its_own_journal_directory_even_through_case_alias() {
        let fixture = NativeFixture::new();
        let journal = private_journal_fixture(&fixture, "journal");
        for relative in ["journal/new.txt", "JOURNAL/new.txt"] {
            let json = serde_json::json!({"version":1, "operation":"create_new", "relativePath":relative, "expectedOldSha256":null, "newContent":"content"});
            assert_eq!(
                fixture
                    .workspace
                    .prepare_new_file_proposal("conv", "req", &json.to_string(), journal.clone())
                    .unwrap_err(),
                WorkspaceError::Denied
            );
        }
        assert_eq!(std::fs::read_dir(journal).unwrap().count(), 0);
    }

    #[cfg(windows)]
    #[test]
    fn cross_operation_candidate_preparation_revokes_old_ids_and_native_approval() {
        for create_first in [true, false] {
            let fixture = NativeFixture::new();
            let existing_path = fixture.dir.join("existing.txt");
            std::fs::write(&existing_path, "unchanged original").unwrap();
            fixture
                .workspace
                .grant_native_file_read("conv", "read", existing_path.clone())
                .unwrap();
            let read = fixture
                .workspace
                .read_project_file("conv", "read", "existing.txt")
                .unwrap();
            let (old, current, journal) = if create_first {
                let (create, journal) =
                    new_file_fixture(&fixture, "new.txt", "new candidate", "journal");
                let existing = fixture
                    .workspace
                    .review_proposal(
                        "conv",
                        "read",
                        "existing.txt",
                        "replacement candidate",
                        Some(&read.sha256),
                    )
                    .unwrap();
                (create, existing, journal)
            } else {
                let existing = fixture
                    .workspace
                    .review_proposal(
                        "conv",
                        "read",
                        "existing.txt",
                        "replacement candidate",
                        Some(&read.sha256),
                    )
                    .unwrap();
                let (create, journal) =
                    new_file_fixture(&fixture, "new.txt", "new candidate", "journal");
                (existing, create, journal)
            };
            {
                let state = fixture.workspace.state.lock().unwrap();
                assert!(!state.proposals.contains_key(&old.proposal_id));
                assert!(!state.creations.contains_key(&old.proposal_id));
                assert_eq!(state.proposals.len() + state.creations.len(), 1);
            }
            assert_eq!(
                fixture
                    .workspace
                    .apply_new_file_candidate(&old.proposal_id, synthetic_approval(&old))
                    .unwrap_err(),
                WorkspaceError::Denied
            );
            // An old operation's exact native approval cannot authorize the
            // new operation, even when the same conversation is reused.
            assert_eq!(
                fixture
                    .workspace
                    .apply_new_file_candidate(&current.proposal_id, synthetic_approval(&old))
                    .unwrap_err(),
                WorkspaceError::Denied
            );
            assert_eq!(
                fixture
                    .workspace
                    .apply_proposal(&current.proposal_id)
                    .unwrap_err(),
                WorkspaceError::Unavailable
            );
            assert_eq!(
                std::fs::read_to_string(existing_path).unwrap(),
                "unchanged original"
            );
            assert!(!fixture.dir.join("new.txt").exists());
            assert_eq!(std::fs::read_dir(journal).unwrap().count(), 0);
        }
    }

    #[cfg(windows)]
    #[test]
    fn interrupted_creation_is_attention_required_without_cleanup_and_blocks_more_writes() {
        for interruption in [
            CreationStage::Intent,
            CreationStage::Created,
            CreationStage::IdentityRecorded,
            CreationStage::Written,
            CreationStage::Flushed,
        ] {
            let fixture = NativeFixture::new();
            let (proposal, journal) =
                new_file_fixture(&fixture, "interrupted.txt", "synthetic content", "journal");
            let outcome = fixture
                .workspace
                .apply_new_file_with_stage(
                    &proposal.proposal_id,
                    synthetic_approval(&proposal),
                    |stage| {
                        if stage == interruption {
                            Err(WorkspaceError::Io)
                        } else {
                            Ok(())
                        }
                    },
                )
                .unwrap();
            assert_eq!(outcome, CreationOutcome::AttentionRequired);
            assert_eq!(
                fixture
                    .workspace
                    .inspect_creation_recovery(journal.clone())
                    .unwrap(),
                CreationOutcome::AttentionRequired
            );
            assert_eq!(
                fixture.dir.join("interrupted.txt").exists(),
                interruption != CreationStage::Intent
            );
            let json = serde_json::json!({"version":1, "operation":"create_new", "relativePath":"another.txt", "newContent":"another", "expectedOldSha256":null});
            assert_eq!(
                fixture
                    .workspace
                    .prepare_new_file_proposal("conv", "req2", &json.to_string(), journal)
                    .unwrap_err(),
                WorkspaceError::AttentionRequired
            );
            assert!(!fixture.dir.join("another.txt").exists());
        }
    }

    #[cfg(windows)]
    #[test]
    fn concurrent_creator_after_intent_cannot_be_overwritten_and_target_handle_excludes_writers() {
        let fixture = NativeFixture::new();
        let (proposal, journal) = new_file_fixture(&fixture, "race.txt", "proposed", "journal");
        let target = fixture.dir.join("race.txt");
        let outcome = fixture
            .workspace
            .apply_new_file_with_stage(
                &proposal.proposal_id,
                synthetic_approval(&proposal),
                |stage| {
                    if stage == CreationStage::Intent {
                        let path = target.clone();
                        std::thread::spawn(move || {
                            std::fs::write(path, "competing process fixture").unwrap()
                        })
                        .join()
                        .unwrap();
                    }
                    Ok(())
                },
            )
            .unwrap();
        assert_eq!(outcome, CreationOutcome::AttentionRequired);
        assert_eq!(
            std::fs::read_to_string(target).unwrap(),
            "competing process fixture"
        );
        assert_eq!(
            fixture
                .workspace
                .inspect_creation_recovery(journal)
                .unwrap(),
            CreationOutcome::AttentionRequired
        );

        let fixture = NativeFixture::new();
        let (proposal, _) = new_file_fixture(&fixture, "exclusive.txt", "proposed", "journal");
        let target = fixture.dir.join("exclusive.txt");
        let outcome = fixture
            .workspace
            .apply_new_file_with_stage(
                &proposal.proposal_id,
                synthetic_approval(&proposal),
                |stage| {
                    if stage == CreationStage::IdentityRecorded {
                        assert!(std::fs::write(&target, "conflicting writer").is_err());
                        assert!(std::fs::remove_file(&target).is_err());
                        assert!(std::fs::rename(&target, fixture.dir.join("renamed.txt")).is_err());
                    }
                    Ok(())
                },
            )
            .unwrap();
        assert_eq!(outcome, CreationOutcome::Verified);
        assert_eq!(std::fs::read_to_string(target).unwrap(), "proposed");
    }

    #[cfg(windows)]
    #[test]
    fn journal_corruption_foreign_objects_and_hardlinks_are_not_silently_recovered() {
        let fixture = NativeFixture::new();
        let (proposal, journal) = new_file_fixture(&fixture, "created.txt", "content", "journal");
        fixture
            .workspace
            .apply_new_file_candidate(&proposal.proposal_id, synthetic_approval(&proposal))
            .unwrap();
        let intent = journal.join(format!("{}.intent", proposal.proposal_id));
        let alias = fixture.dir.join("intent-alias.bin");
        std::fs::hard_link(&intent, &alias).unwrap();
        assert_eq!(
            fixture
                .workspace
                .inspect_creation_recovery(journal.clone())
                .unwrap(),
            CreationOutcome::AttentionRequired
        );
        std::fs::remove_file(alias).unwrap();
        std::fs::write(&intent, "truncated-not-encrypted").unwrap();
        assert_eq!(
            fixture
                .workspace
                .inspect_creation_recovery(journal.clone())
                .unwrap(),
            CreationOutcome::AttentionRequired
        );
        std::fs::write(journal.join("foreign.txt"), "foreign").unwrap();
        assert_eq!(
            fixture
                .workspace
                .inspect_creation_recovery(journal)
                .unwrap(),
            CreationOutcome::AttentionRequired
        );
        assert_eq!(
            std::fs::read_to_string(fixture.dir.join("created.txt")).unwrap(),
            "content"
        );
    }

    #[cfg(windows)]
    #[test]
    fn hardlinks_and_reparse_points_are_denied() {
        let fixture = NativeFixture::new();
        let real = fixture.dir.join("real.txt");
        let alias = fixture.dir.join("alias.txt");
        std::fs::write(&real, "data").unwrap();
        std::fs::hard_link(&real, &alias).unwrap();
        assert_eq!(
            fixture
                .workspace
                .grant_native_file_read("conv", "req", real.clone())
                .unwrap_err(),
            WorkspaceError::Denied
        );
        std::fs::remove_file(&alias).unwrap();
        // Symlink creation may require Developer Mode or elevation. A blocked
        // creation fails this required native gate; it is never silently PASS.
        std::os::windows::fs::symlink_file(&real, &alias).unwrap();
        assert_eq!(
            fixture
                .workspace
                .grant_native_file_read("conv", "req", alias)
                .unwrap_err(),
            WorkspaceError::Denied
        );
    }

    // T-009/T-010/T-024/T-025/T-026 Windows NTFS integration cases are NOT
    // RUN in Linux. The native gate must exercise symlinks/junctions,
    // hardlinks, 8.3 aliases, simultaneous writers, delete/rename attempts,
    // reused grant IDs, identity drift and each crash point before apply.
}
