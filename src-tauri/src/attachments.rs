//! Native-selected, process-local, one-use attachment custody. No path IPC.
use crate::attachment_processing::{InputKind, ProcessedAttachment, ProcessedContent};
use crate::private_store::new_id;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    path::Path,
    time::{Duration, Instant},
};
use zeroize::Zeroizing;

pub(crate) const MAX_SOURCE: usize = 8 * 1024 * 1024;
pub(crate) const MAX_BATCH_SOURCE: usize = crate::attachment_processing::MAX_BATCH_SOURCE_BYTES;
pub(crate) const MAX_FILES: usize = crate::attachment_processing::MAX_BATCH_FILES;
pub(crate) const MAX_PROCESSED: usize = 4 * 1024 * 1024;
pub(crate) const MAX_BATCH_PROCESSED: usize = 16 * 1024 * 1024;
pub(crate) const MAX_BATCH_RGBA: usize = 64 * 1024 * 1024;
pub(crate) const MAX_PDF_PAGES: usize = 20;
const LIFETIME: Duration = Duration::from_secs(300);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AttachmentError {
    Denied,
    ConversionRequired,
    Capacity,
    Conflict,
    Unavailable,
}
impl AttachmentError {
    pub(crate) fn code(self) -> &'static str {
        match self {
            Self::Denied => "attachment_denied",
            Self::ConversionRequired => "attachment_convert_to_pdf",
            Self::Capacity => "attachment_capacity",
            Self::Conflict => "attachment_conflict",
            Self::Unavailable => "attachment_unavailable",
        }
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AttachmentKind {
    Text,
    Image,
    Pdf,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AttachmentMetadata {
    pub id: String,
    pub kind: AttachmentKind,
    pub label: String,
    pub source_bytes: usize,
    pub processed_bytes: usize,
    pub page_count: usize,
}
pub(crate) fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|n| format!("{n:02x}"))
        .collect()
}
pub(crate) fn valid_id(id: &str) -> bool {
    id.len() == 32
        && id
            .bytes()
            .all(|n| n.is_ascii_digit() || (b'a'..=b'f').contains(&n))
}
pub(crate) fn valid_owner_id(id: &str) -> bool {
    (1..=80).contains(&id.len())
        && id
            .bytes()
            .all(|n| n.is_ascii_alphanumeric() || matches!(n, b'-' | b'_'))
}
pub(crate) fn valid_label(label: &str) -> bool {
    !label.is_empty() && label.len() <= 120 && !label.chars().any(|c| c.is_control() || "\\/:*?\"<>|".contains(c)
        || matches!(c, '\u{00ad}' | '\u{034f}' | '\u{061c}' | '\u{180e}' | '\u{2060}' | '\u{feff}'
            | '\u{200b}'..='\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}'))
}
pub(crate) fn validate_metadata(meta: &AttachmentMetadata) -> Result<(), AttachmentError> {
    if !valid_id(&meta.id)
        || !valid_label(&meta.label)
        || meta.source_bytes == 0
        || meta.source_bytes > MAX_SOURCE
        || meta.processed_bytes == 0
        || meta.processed_bytes > MAX_PROCESSED
        || !match meta.kind {
            AttachmentKind::Text => meta.page_count == 0,
            AttachmentKind::Image => meta.page_count == 1,
            AttachmentKind::Pdf => (1..=MAX_PDF_PAGES).contains(&meta.page_count),
        }
    {
        return Err(AttachmentError::Denied);
    }
    Ok(())
}
pub(crate) struct SelectedOriginal {
    pub kind: InputKind,
    pub label: String,
    pub bytes: Zeroizing<Vec<u8>>,
}
/// `path` must originate from the host-owned native selector, never an IPC DTO.
pub(crate) fn read_selected_native(path: &Path) -> Result<SelectedOriginal, AttachmentError> {
    read_selected_native_bounded(path, MAX_SOURCE)
}

/// Fixed host callers may use a stricter local-package limit. No renderer path
/// or arbitrary size parameter is exposed through IPC.
pub(crate) fn read_selected_native_bounded(
    path: &Path,
    maximum: usize,
) -> Result<SelectedOriginal, AttachmentError> {
    if maximum == 0 || maximum > MAX_SOURCE {
        return Err(AttachmentError::Denied);
    }
    let label = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or(AttachmentError::Denied)?;
    if !valid_label(label) {
        return Err(AttachmentError::Denied);
    }
    let raw = path.to_str().ok_or(AttachmentError::Denied)?;
    for component in raw.split(['/', '\\']) {
        let low = component.to_ascii_lowercase();
        let compact: String = low
            .chars()
            .filter(|c| !matches!(c, ' ' | '_' | '-'))
            .collect();
        if matches!(
            low.as_str(),
            "appdata"
                | "windows"
                | "program files"
                | "program files (x86)"
                | "programdata"
                | ".codex"
                | ".agents"
                | ".ssh"
                | ".aws"
                | ".git"
        ) || low.contains("api key")
            || low.contains("api_key")
            || compact.contains("apikey")
            || compact.contains("accesstoken")
            || low.starts_with("credentials")
            || low.starts_with("secret")
        {
            return Err(AttachmentError::Denied);
        }
    }
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .ok_or(AttachmentError::Denied)?
        .to_ascii_lowercase();
    let kind = match extension.as_str() {
        "txt" | "md" | "json" | "xml" | "yaml" | "csv" | "tsv" => InputKind::Text,
        "png" => InputKind::Png,
        "jpg" | "jpeg" => InputKind::Jpeg,
        "gif" => InputKind::Gif,
        "pdf" => InputKind::Pdf,
        "docx" | "xlsx" | "pptx" => return Err(AttachmentError::ConversionRequired),
        _ => return Err(AttachmentError::Denied),
    };
    let bytes = crate::workspace::read_selected_attachment(path, maximum).map_err(|e| match e {
        crate::workspace::WorkspaceError::TooLarge => AttachmentError::Capacity,
        crate::workspace::WorkspaceError::Conflict => AttachmentError::Conflict,
        crate::workspace::WorkspaceError::Unavailable => AttachmentError::Unavailable,
        _ => AttachmentError::Denied,
    })?;
    Ok(SelectedOriginal {
        kind,
        label: label.to_owned(),
        bytes,
    })
}
pub(crate) struct AdmittedAttachment {
    pub metadata: AttachmentMetadata,
    pub original: Zeroizing<Vec<u8>>,
    pub source_sha256: String,
    pub processed: ProcessedContent,
    deadline: Instant,
}
impl AdmittedAttachment {
    /// Immutable import deadline for native review's upper bound. Viewing or
    /// consuming the allocation does not renew its original-source lifetime.
    pub(crate) fn deadline(&self) -> Instant {
        self.deadline
    }
    pub(crate) fn still_valid(&self) -> bool {
        Instant::now() <= self.deadline && sha256(&self.original) == self.source_sha256
    }
    pub(crate) fn from_processed(
        selected: SelectedOriginal,
        processed: ProcessedAttachment,
    ) -> Result<Self, AttachmentError> {
        let source_sha256 = sha256(&selected.bytes);
        if source_sha256 != processed.source_sha256 {
            return Err(AttachmentError::Conflict);
        }
        let kind = match selected.kind {
            InputKind::Text => AttachmentKind::Text,
            InputKind::Pdf => AttachmentKind::Pdf,
            _ => AttachmentKind::Image,
        };
        let (processed_bytes, page_count) = match &processed.content {
            ProcessedContent::Text(text)
                if kind == AttachmentKind::Text && !text.is_empty() && !text.contains('\0') =>
            {
                (text.len(), 0)
            }
            ProcessedContent::Pages(pages) if kind != AttachmentKind::Text => {
                let mut total = 0usize;
                for page in pages {
                    if page.width == 0
                        || page.height == 0
                        || page.width > 4096
                        || page.height > 4096
                        || u64::from(page.width) * u64::from(page.height) > 4_194_304
                        || page.rgba.len() as u64
                            != u64::from(page.width) * u64::from(page.height) * 4
                        || !page.png.starts_with(b"\x89PNG\r\n\x1a\n")
                    {
                        return Err(AttachmentError::Denied);
                    }
                    total = total
                        .checked_add(page.png.len())
                        .ok_or(AttachmentError::Capacity)?;
                }
                (total, pages.len())
            }
            _ => return Err(AttachmentError::Denied),
        };
        let metadata = AttachmentMetadata {
            id: new_id().map_err(|_| AttachmentError::Unavailable)?,
            kind,
            label: selected.label,
            source_bytes: selected.bytes.len(),
            processed_bytes,
            page_count,
        };
        validate_metadata(&metadata)?;
        Ok(Self {
            metadata,
            original: selected.bytes,
            source_sha256,
            processed: processed.content,
            deadline: Instant::now() + LIFETIME,
        })
    }
}
struct Pending {
    conversation_id: String,
    deadline: Instant,
    allocations: Vec<AdmittedAttachment>,
}
#[derive(Default)]
pub(crate) struct AttachmentRegistry {
    pending: Option<Pending>,
}
impl AttachmentRegistry {
    pub(crate) fn invalidate(&mut self) {
        self.pending = None;
    }
    /// Called by the independent host-owned expiry monitor. Grant expiry is
    /// immediate; dropping original buffers occurs on the next scheduled tick.
    /// Zeroising owned buffers is not OS paging/dump secure erasure.
    pub(crate) fn prune_expired(&mut self) -> bool {
        if self
            .pending
            .as_ref()
            .is_some_and(|pending| Instant::now() >= pending.deadline)
        {
            self.invalidate();
            true
        } else {
            false
        }
    }
    pub(crate) fn insert_batch(
        &mut self,
        conversation_id: &str,
        mut batch: Vec<AdmittedAttachment>,
    ) -> Result<Vec<AttachmentMetadata>, AttachmentError> {
        if !valid_owner_id(conversation_id) || batch.is_empty() || batch.len() > MAX_FILES {
            return Err(AttachmentError::Denied);
        }
        let mut ids = HashSet::new();
        let mut source = 0usize;
        let mut processed = 0usize;
        let mut rgba = 0usize;
        let mut pages = 0usize;
        let mut text = 0usize;
        for file in &batch {
            validate_metadata(&file.metadata)?;
            if !ids.insert(file.metadata.id.as_str())
                || file.original.len() != file.metadata.source_bytes
                || sha256(&file.original) != file.source_sha256
            {
                return Err(AttachmentError::Conflict);
            }
            source = source
                .checked_add(file.original.len())
                .ok_or(AttachmentError::Capacity)?;
            processed = processed
                .checked_add(file.metadata.processed_bytes)
                .ok_or(AttachmentError::Capacity)?;
            if file.metadata.kind == AttachmentKind::Pdf {
                pages += file.metadata.page_count;
            }
            if let ProcessedContent::Pages(ps) = &file.processed {
                for p in ps {
                    rgba = rgba
                        .checked_add(p.rgba.len())
                        .ok_or(AttachmentError::Capacity)?;
                }
            }
            if let ProcessedContent::Text(value) = &file.processed {
                if value.len() > 128 * 1024 {
                    return Err(AttachmentError::Capacity);
                }
                text = text
                    .checked_add(value.len())
                    .ok_or(AttachmentError::Capacity)?;
            }
        }
        if source > MAX_BATCH_SOURCE
            || processed > MAX_BATCH_PROCESSED
            || rgba > MAX_BATCH_RGBA
            || pages > MAX_PDF_PAGES
            || text > crate::attachment_processing::MAX_BATCH_TEXT_BYTES
        {
            return Err(AttachmentError::Capacity);
        }
        let deadline = Instant::now() + LIFETIME;
        for allocation in &mut batch {
            allocation.deadline = deadline;
        }
        let metadata = batch.iter().map(|a| a.metadata.clone()).collect();
        self.pending = Some(Pending {
            conversation_id: conversation_id.to_owned(),
            deadline,
            allocations: batch,
        });
        Ok(metadata)
    }
    pub(crate) fn remove(
        &mut self,
        conversation_id: &str,
        id: &str,
    ) -> Result<Vec<AttachmentMetadata>, AttachmentError> {
        let pending = self.pending.as_mut().ok_or(AttachmentError::Denied)?;
        if pending.conversation_id != conversation_id
            || Instant::now() > pending.deadline
            || !valid_id(id)
        {
            self.invalidate();
            return Err(AttachmentError::Denied);
        }
        let index = pending
            .allocations
            .iter()
            .position(|a| a.metadata.id == id)
            .ok_or(AttachmentError::Denied)?;
        pending.allocations.remove(index);
        Ok(pending
            .allocations
            .iter()
            .map(|a| a.metadata.clone())
            .collect())
    }
    /// Move all originals once, before review. A denied/malformed selection
    /// cannot retain original allocations for later replay.
    pub(crate) fn consume(
        &mut self,
        conversation_id: &str,
        request_id: &str,
        ids: &[String],
    ) -> Result<Vec<AdmittedAttachment>, AttachmentError> {
        let pending = self.pending.take();
        if !valid_owner_id(conversation_id)
            || !valid_owner_id(request_id)
            || ids.len() > MAX_FILES
            || ids.iter().any(|id| !valid_id(id))
            || ids.iter().collect::<HashSet<_>>().len() != ids.len()
        {
            return Err(AttachmentError::Denied);
        }
        let Some(pending) = pending else {
            return if ids.is_empty() {
                Ok(Vec::new())
            } else {
                Err(AttachmentError::Denied)
            };
        };
        if ids.is_empty() && pending.allocations.is_empty() {
            return Ok(Vec::new());
        }
        if pending.conversation_id != conversation_id
            || Instant::now() > pending.deadline
            || ids.len() != pending.allocations.len()
            || pending
                .allocations
                .iter()
                .any(|a| !ids.contains(&a.metadata.id))
        {
            return Err(AttachmentError::Denied);
        }
        Ok(pending.allocations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> AdmittedAttachment {
        let bytes = Zeroizing::new(b"synthetic".to_vec());
        AdmittedAttachment::from_processed(
            SelectedOriginal {
                kind: InputKind::Text,
                label: "fixture.txt".into(),
                bytes,
            },
            ProcessedAttachment {
                source_sha256: sha256(b"synthetic"),
                content: ProcessedContent::Text("synthetic".into()),
            },
        )
        .unwrap()
    }
    #[test]
    fn one_use_restart_cross_chat_and_duplicate_ids_deny() {
        let mut registry = AttachmentRegistry::default();
        let meta = registry.insert_batch("chat", vec![fixture()]).unwrap();
        let ids = vec![meta[0].id.clone()];
        assert!(AttachmentRegistry::default()
            .consume("chat", "req", &ids)
            .is_err());
        assert!(registry.consume("other", "req", &ids).is_err());
        assert!(registry.consume("chat", "req", &ids).is_err());
        let meta = registry.insert_batch("chat", vec![fixture()]).unwrap();
        let duplicate = vec![meta[0].id.clone(), meta[0].id.clone()];
        assert!(registry.consume("chat", "req", &duplicate).is_err());
        let meta = registry.insert_batch("chat", vec![fixture()]).unwrap();
        let ids = vec![meta[0].id.clone()];
        assert_eq!(registry.consume("chat", "req", &ids).unwrap().len(), 1);
        assert!(registry.consume("chat", "req", &ids).is_err());
    }
    #[test]
    fn removal_expiry_hash_mismatch_and_overcount_deny() {
        let mut registry = AttachmentRegistry::default();
        let meta = registry.insert_batch("chat", vec![fixture()]).unwrap();
        assert!(registry.remove("chat", &meta[0].id).unwrap().is_empty());
        assert!(registry
            .consume("chat", "req", &[meta[0].id.clone()])
            .is_err());
        let meta = registry.insert_batch("chat", vec![fixture()]).unwrap();
        registry.pending.as_mut().unwrap().deadline = Instant::now() - Duration::from_secs(1);
        assert!(registry
            .consume("chat", "req", &[meta[0].id.clone()])
            .is_err());
        let mut broken = fixture();
        broken.source_sha256 = "0".repeat(64);
        assert!(registry.insert_batch("chat", vec![broken]).is_err());
        assert!(registry
            .insert_batch("chat", (0..5).map(|_| fixture()).collect())
            .is_err());
        let mut expired = fixture();
        expired.deadline = Instant::now() - Duration::from_secs(1);
        assert!(!expired.still_valid());
    }
    #[test]
    fn host_expiry_tick_discards_pending_originals_without_another_user_action() {
        let mut registry = AttachmentRegistry::default();
        let metadata = registry.insert_batch("chat", vec![fixture()]).unwrap();
        assert!(!registry.prune_expired());
        registry.pending.as_mut().unwrap().deadline = Instant::now() - Duration::from_secs(1);
        assert!(registry.prune_expired());
        assert!(registry.pending.is_none());
        assert!(!registry.prune_expired());
        assert!(registry
            .consume("chat", "request", &[metadata[0].id.clone()])
            .is_err());
    }
    #[test]
    fn consuming_source_preserves_its_deadline_without_renewal() {
        let mut registry = AttachmentRegistry::default();
        let metadata = registry.insert_batch("chat", vec![fixture()]).unwrap();
        let original_deadline = registry.pending.as_ref().unwrap().allocations[0].deadline();
        let allocations = registry
            .consume("chat", "request", &[metadata[0].id.clone()])
            .unwrap();
        assert_eq!(allocations[0].deadline(), original_deadline);
        assert_eq!(allocations[0].deadline(), allocations[0].deadline());
    }
    #[test]
    fn metadata_labels_and_unknown_fields_are_closed() {
        assert!(!valid_label("bad\u{202e}.pdf"));
        assert!(!valid_label("bad\n.pdf"));
        let meta = fixture().metadata;
        let mut json = serde_json::to_value(meta).unwrap();
        json["original"] = serde_json::json!("injected");
        assert!(serde_json::from_value::<AttachmentMetadata>(json).is_err());
    }

    #[test]
    fn office_conversion_and_sensitive_native_selection_are_denied_before_read() {
        assert!(matches!(
            read_selected_native(Path::new("synthetic.docx")),
            Err(AttachmentError::ConversionRequired)
        ));
        for path in [
            r"C:\Safe\Meta API key.txt",
            r"C:\Users\Owner\.codex\data.txt",
            r"C:\Windows\data.txt",
            r"C:\Safe\credentials.txt",
        ] {
            assert!(matches!(
                read_selected_native(Path::new(path)),
                Err(AttachmentError::Denied)
            ));
        }
    }

    #[cfg(windows)]
    #[test]
    fn native_identity_reader_refuses_hardlinks_ads_aliases_and_writable_handles() {
        use std::{
            fs::{self, OpenOptions},
            io::{ErrorKind, Write},
            os::windows::fs::OpenOptionsExt,
            path::PathBuf,
        };
        struct NativeFixture {
            directory: PathBuf,
            root: Option<crate::workspace::Workspace>,
            source_created: bool,
            link_created: bool,
        }
        impl Drop for NativeFixture {
            fn drop(&mut self) {
                // Release production ancestor/root pins before exact, non-recursive
                // cleanup. Never remove unrelated files if the test is interrupted.
                drop(self.root.take());
                let mut failed = false;
                for (created, name) in [
                    (self.link_created, "linked.txt"),
                    (self.source_created, "synthetic.txt"),
                ] {
                    if !created {
                        continue;
                    }
                    if let Err(error) = fs::remove_file(self.directory.join(name)) {
                        failed |= error.kind() != ErrorKind::NotFound;
                    }
                }
                if let Err(error) = fs::remove_dir(&self.directory) {
                    failed |= error.kind() != ErrorKind::NotFound;
                }
                if failed {
                    if std::thread::panicking() {
                        eprintln!("attachment_fixture_cleanup=failed");
                    } else {
                        panic!("Synthetic attachment fixture cleanup failed");
                    }
                }
            }
        }
        // Use the checkout candidate already exercised by workspace's native
        // fixtures. TEMP is not an admitted root merely because it is writable.
        // Exact production root admission below remains mandatory; no fallback,
        // guard relaxation, alias canonicalisation or skipped assertion is used.
        let directory = std::env::current_dir()
            .unwrap()
            .join(format!("aster-workspace-test-{}", new_id().unwrap()));
        fs::create_dir(&directory).unwrap();
        let mut fixture = NativeFixture {
            directory,
            root: Some(crate::workspace::Workspace::default()),
            source_created: false,
            link_created: false,
        };
        fixture
            .root
            .as_ref()
            .unwrap()
            .set_native_root(fixture.directory.clone())
            .expect("Synthetic attachment fixture must pass production root admission");
        let source = fixture.directory.join("synthetic.txt");
        let link = fixture.directory.join("linked.txt");
        let mut original = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&source)
            .unwrap();
        fixture.source_created = true;
        original.write_all(b"synthetic").unwrap();
        drop(original);
        assert_eq!(
            &*crate::workspace::read_selected_attachment(&source, MAX_SOURCE).unwrap(),
            b"synthetic"
        );
        let writer = OpenOptions::new()
            .write(true)
            .share_mode(3)
            .open(&source)
            .unwrap();
        assert!(crate::workspace::read_selected_attachment(&source, MAX_SOURCE).is_err());
        drop(writer);
        fs::hard_link(&source, &link).unwrap();
        fixture.link_created = true;
        assert!(crate::workspace::read_selected_attachment(&source, MAX_SOURCE).is_err());
        fs::remove_file(&link).unwrap();
        fixture.link_created = false;
        let ads = Path::new(&format!("{}:extra", source.display())).to_path_buf();
        assert!(crate::workspace::read_selected_attachment(&ads, MAX_SOURCE).is_err());
        let device = Path::new(&format!(r"\\?\{}", source.display())).to_path_buf();
        assert!(crate::workspace::read_selected_attachment(&device, MAX_SOURCE).is_err());
    }
}
