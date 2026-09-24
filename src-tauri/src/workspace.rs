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

use serde::Serialize;
#[cfg(windows)]
use sha2::{Digest, Sha256};

const MAX_READ_BYTES: u64 = 1024 * 1024;
#[cfg(any(windows, test))]
const MAX_CHANGED_BYTES: usize = 256 * 1024;
const MAX_PATH_BYTES: usize = 4096;
const MAX_ID_BYTES: usize = 128;
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
}

impl WorkspaceError {
    pub fn code(self) -> &'static str {
        match self {
            Self::Unavailable => "WORKSPACE_UNAVAILABLE",
            Self::Denied => "WORKSPACE_DENIED",
            Self::Conflict => "WORKSPACE_CONFLICT",
            Self::TooLarge => "WORKSPACE_LIMIT_EXCEEDED",
            Self::Io => "WORKSPACE_IO_ERROR",
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
}

#[cfg_attr(not(windows), allow(dead_code))]
struct Root {
    path: PathBuf,
    label: String,
    #[cfg(windows)]
    anchor: std::fs::File,
    #[cfg(windows)]
    id: windows_fs::FileIdentity,
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
    proposal_remaining: bool,
    #[cfg(windows)]
    id: windows_fs::FileIdentity,
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
        let mut state = self.state.lock().map_err(|_| WorkspaceError::Unavailable)?;
        let epoch = state.epoch;
        let snapshot = state
            .snapshots
            .get_mut(&(conversation_id.to_owned(), request_id.to_owned()))
            .ok_or(WorkspaceError::Denied)?;
        let was_available = snapshot.disclosure_remaining;
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
        Ok(snapshot.read.clone())
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
            // This structured JSON is lossless, distinguishes content from
            // review labels, and can be rendered inertly or parsed by a future
            // host-owned native full-diff viewer.
            let diff = serde_json::to_string(&serde_json::json!({
                "operation": "replace_existing",
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
            if state.proposals.len() >= MAX_PENDING {
                return Err(WorkspaceError::TooLarge);
            }
            let proposal_id = windows_fs::random_nonce()?;
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
    format!("{:x}", Sha256::digest(bytes))
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
        drop(parents);
        let label = segments
            .last()
            .cloned()
            .unwrap_or_else(|| prefix.to_owned());
        Ok(Root {
            path: cursor,
            label,
            anchor,
            id,
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
            WorkspaceError::Unavailable
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
            let dir = std::env::temp_dir().join(format!(
                "aster-workspace-test-{}",
                windows_fs::random_nonce().unwrap()
            ));
            std::fs::create_dir(&dir).unwrap();
            let workspace = Workspace::default();
            workspace.set_native_root(dir.clone()).unwrap();
            Self { dir, workspace }
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
