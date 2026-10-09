//! Independently encrypted processed-only media. Never original PDF/image bytes.
use crate::attachment_processing::{InputKind, ProcessedContent, ProcessedPage};
use crate::attachments::{
    sha256, valid_id, valid_owner_id, validate_metadata, AdmittedAttachment, AttachmentKind,
    AttachmentMetadata, MAX_BATCH_PROCESSED, MAX_FILES, MAX_PDF_PAGES, MAX_PROCESSED,
};
use crate::private_store::{new_id, Domain, PrivateStore, StoreError};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
    path::Path,
    sync::atomic::AtomicBool,
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct StoredPage {
    pub width: u32,
    pub height: u32,
    pub png_base64: String,
    pub sha256: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum StoredContent {
    Text { text: String },
    Pages { pages: Vec<StoredPage> },
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct StoredProcessed {
    pub conversation_id: String,
    pub request_id: String,
    pub metadata: AttachmentMetadata,
    pub source_sha256: String,
    pub processed_sha256: String,
    pub data_class: String,
    pub content: StoredContent,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Record {
    version: u32,
    sequence: u64,
    previous_sha256: String,
    attachment: StoredProcessed,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ProcessedBinding {
    pub metadata: AttachmentMetadata,
    pub source_sha256: String,
    pub processed_sha256: String,
}
pub(crate) fn validate_binding(binding: &ProcessedBinding) -> Result<(), StoreError> {
    validate_metadata(&binding.metadata).map_err(|_| StoreError::Corrupt)?;
    if !valid_sha(&binding.source_sha256) || !valid_sha(&binding.processed_sha256) {
        return Err(StoreError::Corrupt);
    }
    Ok(())
}

fn valid_sha(hash: &str) -> bool {
    hash.len() == 64
        && hash
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn content_digest(content: &StoredContent) -> String {
    let mut h = Sha256::new();
    match content {
        StoredContent::Text { text } => {
            h.update(b"text-v1\0");
            h.update(text.as_bytes());
        }
        StoredContent::Pages { pages } => {
            h.update(b"pages-v1\0");
            h.update((pages.len() as u32).to_le_bytes());
            for page in pages {
                h.update(page.width.to_le_bytes());
                h.update(page.height.to_le_bytes());
                h.update(page.sha256.as_bytes());
            }
        }
    }
    h.finalize().iter().map(|n| format!("{n:02x}")).collect()
}
fn validate(item: &StoredProcessed) -> Result<(), StoreError> {
    validate_metadata(&item.metadata).map_err(|_| StoreError::Corrupt)?;
    if !valid_owner_id(&item.conversation_id)
        || !valid_owner_id(&item.request_id)
        || !valid_sha(&item.source_sha256)
        || !valid_sha(&item.processed_sha256)
        || !matches!(item.data_class.as_str(), "synthetic" | "public_eligible")
        || content_digest(&item.content) != item.processed_sha256
    {
        return Err(StoreError::Corrupt);
    }
    let bytes = match &item.content {
        StoredContent::Text { text } => {
            if item.metadata.kind != AttachmentKind::Text
                || text.is_empty()
                || text.len() > 128 * 1024
                || text.contains('\0')
            {
                return Err(StoreError::Corrupt);
            }
            text.len()
        }
        StoredContent::Pages { pages } => {
            if item.metadata.kind == AttachmentKind::Text
                || pages.len() != item.metadata.page_count
                || pages.is_empty()
                || pages.len() > MAX_PDF_PAGES
            {
                return Err(StoreError::Corrupt);
            }
            let mut sum = 0usize;
            let mut rgba = 0u64;
            for page in pages {
                if !valid_sha(&page.sha256)
                    || page.width == 0
                    || page.height == 0
                    || page.width > 4096
                    || page.height > 4096
                    || u64::from(page.width) * u64::from(page.height) > 4_194_304
                    || page.png_base64.is_empty()
                    || page.png_base64.len() > MAX_PROCESSED.div_ceil(3) * 4
                {
                    return Err(StoreError::Corrupt);
                }
                let png = STANDARD
                    .decode(&page.png_base64)
                    .map_err(|_| StoreError::Corrupt)?;
                if !png.starts_with(b"\x89PNG\r\n\x1a\n") || sha256(&png) != page.sha256 {
                    return Err(StoreError::Corrupt);
                }
                rgba = rgba
                    .checked_add(u64::from(page.width) * u64::from(page.height) * 4)
                    .ok_or(StoreError::Capacity)?;
                if rgba > 64 * 1024 * 1024 {
                    return Err(StoreError::Capacity);
                }
                sum = sum.checked_add(png.len()).ok_or(StoreError::Capacity)?;
                if sum > MAX_PROCESSED {
                    return Err(StoreError::Capacity);
                }
            }
            sum
        }
    };
    if bytes != item.metadata.processed_bytes || bytes > MAX_PROCESSED {
        return Err(StoreError::Corrupt);
    }
    Ok(())
}
impl StoredProcessed {
    /// Native viewer only. Decode each stored derivative in the same bounded
    /// worker, never feed these pages into a later provider request implicitly.
    pub(crate) fn materialise(&self, cancel: &AtomicBool) -> Result<ProcessedContent, StoreError> {
        validate(self)?;
        match &self.content {
            StoredContent::Text { text } => Ok(ProcessedContent::Text(text.clone())),
            StoredContent::Pages { pages } => {
                let mut result = Vec::with_capacity(pages.len());
                for page in pages {
                    let png = STANDARD
                        .decode(&page.png_base64)
                        .map_err(|_| StoreError::Corrupt)?;
                    let decoded = crate::attachment_processing::process_isolated(
                        InputKind::Png,
                        &png,
                        cancel,
                    )
                    .map_err(|_| StoreError::Corrupt)?;
                    let ProcessedContent::Pages(mut images) = decoded.content else {
                        return Err(StoreError::Corrupt);
                    };
                    if images.len() != 1 {
                        return Err(StoreError::Corrupt);
                    }
                    let image = images.remove(0);
                    if image.width != page.width || image.height != page.height {
                        return Err(StoreError::Corrupt);
                    }
                    result.push(ProcessedPage {
                        width: page.width,
                        height: page.height,
                        rgba: image.rgba,
                        png,
                    });
                }
                Ok(ProcessedContent::Pages(result))
            }
        }
    }
}
pub(crate) struct ProcessedHistory {
    store: PrivateStore,
    attachments: BTreeMap<String, StoredProcessed>,
    sequence: u64,
    previous_sha256: String,
    fault: bool,
}
impl ProcessedHistory {
    pub(crate) fn open(base: &Path) -> Result<Self, StoreError> {
        Self::restore(PrivateStore::open(base, Domain::ProcessedMedia)?)
    }
    /// Import requires a healthy replay and room for at least one maximum
    /// bounded file record. This neither restores a grant nor reserves quota;
    /// append_batch still performs exact current whole-batch admission.
    pub(crate) fn import_available(&self) -> Result<bool, StoreError> {
        self.ensure_current()?;
        self.store
            .has_capacity(Domain::ProcessedMedia, 6 * 1024 * 1024, 1)
    }
    fn restore(store: PrivateStore) -> Result<Self, StoreError> {
        let mut records = Vec::new();
        for (_, bytes) in store.records()? {
            let record: Record = serde_json::from_slice(&bytes).map_err(|_| StoreError::Corrupt)?;
            records.push((record, bytes));
        }
        records.sort_by_key(|(r, _)| r.sequence);
        let mut value = Self {
            store,
            attachments: BTreeMap::new(),
            sequence: 0,
            previous_sha256: String::new(),
            fault: false,
        };
        for (record, bytes) in records {
            if record.version != 1
                || record.sequence != value.sequence + 1
                || record.previous_sha256 != value.previous_sha256
            {
                return Err(StoreError::Corrupt);
            }
            value.check_insert(&record.attachment)?;
            value
                .attachments
                .insert(record.attachment.metadata.id.clone(), record.attachment);
            value.sequence = record.sequence;
            value.previous_sha256 = sha256(&bytes);
        }
        Ok(value)
    }
    fn ensure_current(&self) -> Result<(), StoreError> {
        if self.fault {
            return Err(StoreError::Corrupt);
        }
        let restored = Self::restore_records(&self.store)?;
        if restored.0 != self.sequence || restored.1 != self.previous_sha256 {
            return Err(StoreError::Corrupt);
        }
        Ok(())
    }
    fn restore_records(store: &PrivateStore) -> Result<(u64, String), StoreError> {
        let mut records = Vec::new();
        for (_, bytes) in store.records()? {
            let record: Record = serde_json::from_slice(&bytes).map_err(|_| StoreError::Corrupt)?;
            records.push((record, bytes));
        }
        records.sort_by_key(|(r, _)| r.sequence);
        let mut sequence = 0u64;
        let mut hash = String::new();
        let mut ids = HashSet::new();
        for (record, bytes) in records {
            if record.version != 1
                || record.sequence != sequence + 1
                || record.previous_sha256 != hash
                || !ids.insert(record.attachment.metadata.id.clone())
            {
                return Err(StoreError::Corrupt);
            }
            validate(&record.attachment)?;
            sequence = record.sequence;
            hash = sha256(&bytes);
        }
        Ok((sequence, hash))
    }
    fn check_insert(&self, item: &StoredProcessed) -> Result<(), StoreError> {
        validate(item)?;
        if self.attachments.contains_key(&item.metadata.id) {
            return Err(StoreError::Conflict);
        }
        let conversation_used = self
            .attachments
            .values()
            .filter(|a| a.conversation_id == item.conversation_id)
            .try_fold(0usize, |sum, a| {
                sum.checked_add(a.metadata.processed_bytes)
                    .ok_or(StoreError::Capacity)
            })?;
        if conversation_used
            .checked_add(item.metadata.processed_bytes)
            .is_none_or(|sum| sum > MAX_BATCH_PROCESSED)
        {
            return Err(StoreError::Capacity);
        }
        let same_turn: Vec<_> = self
            .attachments
            .values()
            .filter(|a| {
                a.conversation_id == item.conversation_id && a.request_id == item.request_id
            })
            .collect();
        let pages: usize = same_turn
            .iter()
            .filter(|a| a.metadata.kind == AttachmentKind::Pdf)
            .map(|a| a.metadata.page_count)
            .sum();
        if same_turn.len() >= MAX_FILES
            || pages
                + if item.metadata.kind == AttachmentKind::Pdf {
                    item.metadata.page_count
                } else {
                    0
                }
                > MAX_PDF_PAGES
        {
            return Err(StoreError::Capacity);
        }
        Ok(())
    }
    /// Append originals-free immutable records first. Call History::begin only
    /// after the complete batch succeeds; failures may leave unexposed orphans.
    pub(crate) fn append_batch(
        &mut self,
        conversation_id: &str,
        request_id: &str,
        data_class: &str,
        files: &[AdmittedAttachment],
    ) -> Result<Vec<ProcessedBinding>, StoreError> {
        self.ensure_current()?;
        if !valid_owner_id(conversation_id)
            || !valid_owner_id(request_id)
            || files.len() > MAX_FILES
            || !matches!(data_class, "synthetic" | "public_eligible")
        {
            return Err(StoreError::Denied);
        }
        if self
            .attachments
            .values()
            .any(|a| a.conversation_id == conversation_id && a.request_id == request_id)
        {
            return Err(StoreError::Conflict);
        }
        let mut batch = Vec::new();
        let mut ids = HashSet::new();
        let mut total = 0usize;
        let mut pdf_pages = 0usize;
        for file in files {
            if !ids.insert(file.metadata.id.clone()) || sha256(&file.original) != file.source_sha256
            {
                return Err(StoreError::Conflict);
            }
            let content = match &file.processed {
                ProcessedContent::Text(text) => StoredContent::Text { text: text.clone() },
                ProcessedContent::Pages(pages) => StoredContent::Pages {
                    pages: pages
                        .iter()
                        .map(|p| StoredPage {
                            width: p.width,
                            height: p.height,
                            png_base64: STANDARD.encode(&p.png),
                            sha256: sha256(&p.png),
                        })
                        .collect(),
                },
            };
            let item = StoredProcessed {
                conversation_id: conversation_id.into(),
                request_id: request_id.into(),
                metadata: file.metadata.clone(),
                source_sha256: file.source_sha256.clone(),
                processed_sha256: content_digest(&content),
                data_class: data_class.into(),
                content,
            };
            self.check_insert(&item)?;
            total = total
                .checked_add(item.metadata.processed_bytes)
                .ok_or(StoreError::Capacity)?;
            if item.metadata.kind == AttachmentKind::Pdf {
                pdf_pages += item.metadata.page_count;
            }
            batch.push(item);
        }
        let used = self
            .attachments
            .values()
            .filter(|a| a.conversation_id == conversation_id)
            .try_fold(0usize, |sum, a| {
                sum.checked_add(a.metadata.processed_bytes)
                    .ok_or(StoreError::Capacity)
            })?;
        if used
            .checked_add(total)
            .is_none_or(|sum| sum > MAX_BATCH_PROCESSED)
            || pdf_pages > MAX_PDF_PAGES
        {
            return Err(StoreError::Capacity);
        }
        if !self.store.has_capacity(
            Domain::ProcessedMedia,
            files
                .len()
                .checked_mul(6 * 1024 * 1024)
                .ok_or(StoreError::Capacity)?,
            files.len(),
        )? {
            return Err(StoreError::Capacity);
        }
        let bindings = batch
            .iter()
            .map(|item| ProcessedBinding {
                metadata: item.metadata.clone(),
                source_sha256: item.source_sha256.clone(),
                processed_sha256: item.processed_sha256.clone(),
            })
            .collect();
        for item in batch {
            let record = Record {
                version: 1,
                sequence: self.sequence.checked_add(1).ok_or(StoreError::Capacity)?,
                previous_sha256: self.previous_sha256.clone(),
                attachment: item,
            };
            let bytes = serde_json::to_vec(&record).map_err(|_| StoreError::Corrupt)?;
            if let Err(error) = self.store.append(&new_id()?, &bytes) {
                self.fault = true;
                return Err(error);
            }
            self.sequence = record.sequence;
            self.previous_sha256 = sha256(&bytes);
            self.attachments
                .insert(record.attachment.metadata.id.clone(), record.attachment);
        }
        Ok(bindings)
    }
    /// Caller must first obtain exact metadata/class from an undeleted History
    /// entry; a known media ID by itself is not authority to view an orphan.
    pub(crate) fn load_bound(
        &self,
        conversation_id: &str,
        request_id: &str,
        binding: &ProcessedBinding,
        data_class: &str,
    ) -> Result<StoredProcessed, StoreError> {
        self.ensure_current()?;
        validate_binding(binding)?;
        let metadata = &binding.metadata;
        if !valid_id(&metadata.id) {
            return Err(StoreError::Denied);
        }
        let item = self
            .attachments
            .get(&metadata.id)
            .ok_or(StoreError::Corrupt)?;
        if item.conversation_id != conversation_id
            || item.request_id != request_id
            || item.metadata != *metadata
            || item.source_sha256 != binding.source_sha256
            || item.processed_sha256 != binding.processed_sha256
            || item.data_class != data_class
        {
            return Err(StoreError::Corrupt);
        }
        validate(item)?;
        Ok(item.clone())
    }
    #[cfg(test)]
    pub(crate) fn memory() -> Self {
        Self::restore(PrivateStore::memory(Domain::ProcessedMedia)).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attachment_processing::ProcessedAttachment;
    use crate::attachments::SelectedOriginal;
    use zeroize::Zeroizing;
    fn fixture() -> AdmittedAttachment {
        AdmittedAttachment::from_processed(
            SelectedOriginal {
                kind: InputKind::Text,
                label: "fixture.txt".into(),
                bytes: Zeroizing::new(b"synthetic".to_vec()),
            },
            ProcessedAttachment {
                source_sha256: sha256(b"synthetic"),
                content: ProcessedContent::Text("synthetic".into()),
            },
        )
        .unwrap()
    }
    #[test]
    fn fresh_replay_binding_and_original_injection_are_closed() {
        let mut history = ProcessedHistory::memory();
        let file = fixture();
        let meta = history
            .append_batch("chat", "request", "synthetic", &[file])
            .unwrap()
            .remove(0);
        assert!(history
            .load_bound("other", "request", &meta, "synthetic")
            .is_err());
        assert!(history
            .load_bound("chat", "other", &meta, "synthetic")
            .is_err());
        assert!(history
            .load_bound("chat", "request", &meta, "public_eligible")
            .is_err());
        let mut mismatch = meta.clone();
        mismatch.source_sha256 = "0".repeat(64);
        assert!(history
            .load_bound("chat", "request", &mismatch, "synthetic")
            .is_err());
        let mut mismatch = meta.clone();
        mismatch.processed_sha256 = "0".repeat(64);
        assert!(history
            .load_bound("chat", "request", &mismatch, "synthetic")
            .is_err());
        let bytes = history.store.records().unwrap().remove(0).1;
        let mut injected: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        injected["attachment"]["originalPdf"] = serde_json::json!("hidden");
        let mut store = PrivateStore::memory(Domain::ProcessedMedia);
        store
            .append(&new_id().unwrap(), &serde_json::to_vec(&injected).unwrap())
            .unwrap();
        assert!(ProcessedHistory::restore(store).is_err());
        let restored = ProcessedHistory::restore(history.store).unwrap();
        assert!(restored
            .load_bound("chat", "request", &meta, "synthetic")
            .is_ok());
    }
    #[test]
    fn uncertain_append_poisons_cache_and_replay_never_creates_grants() {
        for after in [false, true] {
            let mut history = ProcessedHistory::memory();
            assert!(history.import_available().unwrap());
            let file = fixture();
            let meta = ProcessedBinding {
                metadata: file.metadata.clone(),
                source_sha256: file.source_sha256.clone(),
                processed_sha256: content_digest(&StoredContent::Text {
                    text: "synthetic".into(),
                }),
            };
            history.store.fail_next_append_for_test(after);
            assert_eq!(
                history
                    .append_batch("chat", "request", "synthetic", &[file])
                    .unwrap_err(),
                StoreError::Io
            );
            assert_eq!(history.import_available().unwrap_err(), StoreError::Corrupt);
            assert!(history
                .load_bound("chat", "request", &meta, "synthetic")
                .is_err());
            let restored = ProcessedHistory::restore(history.store).unwrap();
            assert_eq!(restored.attachments.len(), usize::from(after));
            assert!(restored.import_available().unwrap());
        }
    }
    #[test]
    fn corrupt_hash_duplicate_and_capacity_fail_without_eviction() {
        let mut history = ProcessedHistory::memory();
        history
            .append_batch("chat", "request", "synthetic", &[fixture()])
            .unwrap();
        let mut value: serde_json::Value =
            serde_json::from_slice(&history.store.records().unwrap()[0].1).unwrap();
        value["attachment"]["processedSha256"] = serde_json::json!("0".repeat(64));
        let mut store = PrivateStore::memory(Domain::ProcessedMedia);
        store
            .append(&new_id().unwrap(), &serde_json::to_vec(&value).unwrap())
            .unwrap();
        assert!(ProcessedHistory::restore(store).is_err());
        assert!(history
            .append_batch(
                "chat",
                "request",
                "synthetic",
                &(0..5).map(|_| fixture()).collect::<Vec<_>>()
            )
            .is_err());
        assert!(!history
            .store
            .has_capacity(Domain::ProcessedMedia, 128 * 1024 * 1024 + 1, 0)
            .unwrap());
    }
}
