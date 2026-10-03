//! Host-authored Meta-only persistent display history. Stored display metadata
//! never grants disclosure, financial authority, approvals or active requests.
//! Removing visible history is a tombstone; encrypted recovery records remain.

use crate::private_store::{new_id, Domain, PrivateStore, StoreError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
    path::Path,
};

const PROVIDER: &str = "meta_model_api";
const MODEL: &str = "muse-spark-1.3-contributor";
const MAX_CONVERSATIONS: usize = 128;
const MAX_TEXT: usize = 512 * 1024;
const MAX_ENTRIES: usize = 256;
const MAX_COMPLETE_TURNS: usize = 20;

#[derive(Debug, Copy, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum EntryState {
    Completed,
    Failed,
    Cancelled,
    Truncated,
    Refused,
    Interrupted,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Entry {
    pub id: String,
    pub role: String,
    pub content: String,
    pub request_id: String,
    pub state: EntryState,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Conversation {
    pub id: String,
    pub title: String,
    pub updated_at: u64,
    pub entries: Vec<Entry>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ConversationSummary {
    pub id: String,
    pub title: String,
    pub updated_at: u64,
    pub entry_count: usize,
}

pub(crate) struct EligiblePair {
    pub prompt: String,
    pub answer: String,
    pub data_class: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Turn {
    request_id: String,
    user_entry: String,
    assistant_entry: Option<String>,
    data_class: String,
    eligible: bool,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredConversation {
    version: u32,
    provider: String,
    model: String,
    display: Conversation,
    turns: Vec<Turn>,
    pending_request: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Event {
    Snapshot { conversation: StoredConversation },
    Remove { id: String, updated_at: u64 },
    Invalidate { updated_at: u64 },
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Record {
    version: u32,
    sequence: u64,
    previous_sha256: String,
    event: Event,
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn valid_id(id: &str) -> bool {
    (1..=80).contains(&id.len())
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn eligible_class(class: &str) -> bool {
    matches!(class, "synthetic" | "public_eligible")
}

fn title(prompt: &str) -> String {
    let text = prompt.split_whitespace().collect::<Vec<_>>().join(" ");
    let text: String = text
        .chars()
        .filter(|ch| !ch.is_control() && !risky_format_mark(*ch))
        .take(64)
        .collect();
    if text.is_empty() {
        "New conversation".to_owned()
    } else {
        text
    }
}

fn risky_format_mark(ch: char) -> bool {
    matches!(ch,
        '\u{00ad}' | '\u{034f}' | '\u{061c}' | '\u{180e}' | '\u{2060}' | '\u{feff}'
        | '\u{200b}'..='\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}'
    )
}

fn text_bytes(conversation: &Conversation) -> Result<usize, StoreError> {
    conversation.entries.iter().try_fold(0usize, |sum, entry| {
        sum.checked_add(entry.content.len())
            .ok_or(StoreError::Capacity)
    })
}

fn validate(conversation: &StoredConversation) -> Result<(), StoreError> {
    let display = &conversation.display;
    if conversation.version != 1
        || conversation.provider != PROVIDER
        || conversation.model != MODEL
        || !valid_id(&display.id)
        || display.title.is_empty()
        || display.title.chars().count() > 64
        || display.title.chars().any(char::is_control)
        || display.title.chars().any(risky_format_mark)
        || display.entries.len() > MAX_ENTRIES
        || text_bytes(display)? > MAX_TEXT
        || conversation.turns.len() > MAX_ENTRIES / 2
    {
        return Err(StoreError::Corrupt);
    }
    let mut entry_ids = HashSet::new();
    for entry in &display.entries {
        if !valid_id(&entry.id)
            || !valid_id(&entry.request_id)
            || !matches!(entry.role.as_str(), "user" | "assistant")
            || entry.content.contains('\0')
            || !entry_ids.insert(entry.id.as_str())
        {
            return Err(StoreError::Corrupt);
        }
    }
    let mut request_ids = HashSet::new();
    let mut claimed = HashSet::new();
    let mut eligible_count = 0usize;
    let mut pending_count = 0usize;
    for turn in &conversation.turns {
        if !valid_id(&turn.request_id)
            || !eligible_class(&turn.data_class)
            || !request_ids.insert(turn.request_id.as_str())
            || !claimed.insert(turn.user_entry.as_str())
        {
            return Err(StoreError::Corrupt);
        }
        let user = display
            .entries
            .iter()
            .find(|entry| entry.id == turn.user_entry)
            .ok_or(StoreError::Corrupt)?;
        if user.role != "user"
            || user.request_id != turn.request_id
            || user.content.trim().is_empty()
            || user.content.len() > 32 * 1024
        {
            return Err(StoreError::Corrupt);
        }
        match &turn.assistant_entry {
            Some(id) => {
                if !claimed.insert(id.as_str()) {
                    return Err(StoreError::Corrupt);
                }
                let answer = display
                    .entries
                    .iter()
                    .find(|entry| entry.id == *id)
                    .ok_or(StoreError::Corrupt)?;
                if answer.role != "assistant" || answer.request_id != turn.request_id {
                    return Err(StoreError::Corrupt);
                }
                if turn.eligible {
                    if user.state != EntryState::Completed
                        || answer.state != EntryState::Completed
                        || answer.content.trim().is_empty()
                    {
                        return Err(StoreError::Corrupt);
                    }
                    eligible_count += 1;
                }
            }
            None => {
                if turn.eligible {
                    return Err(StoreError::Corrupt);
                }
                if conversation.pending_request.as_deref() == Some(turn.request_id.as_str()) {
                    if user.state != EntryState::Completed {
                        return Err(StoreError::Corrupt);
                    }
                    pending_count += 1;
                } else if user.state != EntryState::Interrupted {
                    return Err(StoreError::Corrupt);
                }
            }
        }
    }
    if claimed.len() != display.entries.len()
        || eligible_count > MAX_COMPLETE_TURNS
        || pending_count != usize::from(conversation.pending_request.is_some())
        || conversation
            .pending_request
            .as_ref()
            .is_some_and(|id| !valid_id(id))
    {
        return Err(StoreError::Corrupt);
    }
    Ok(())
}

pub(crate) struct History {
    store: PrivateStore,
    conversations: BTreeMap<String, StoredConversation>,
    removed: HashSet<String>,
    sequence: u64,
    previous_sha256: String,
    // An immutable append can fail before or after durable visibility. Until
    // a fresh replay validates the actual records, cached state must not be
    // exposed as healthy or used to continue another request.
    fault: bool,
}

impl History {
    pub(crate) fn open(base: &Path) -> Result<Self, StoreError> {
        Self::restore(PrivateStore::open(base, Domain::History)?)
    }

    fn restore(store: PrivateStore) -> Result<Self, StoreError> {
        let mut records = Vec::new();
        for (_, bytes) in store.records()? {
            let record: Record = serde_json::from_slice(&bytes).map_err(|_| StoreError::Corrupt)?;
            records.push((record, bytes));
        }
        records.sort_by_key(|(record, _)| record.sequence);
        let mut history = Self {
            store,
            conversations: BTreeMap::new(),
            removed: HashSet::new(),
            sequence: 0,
            previous_sha256: String::new(),
            fault: false,
        };
        for (record, bytes) in records {
            if record.version != 1
                || record.sequence
                    != history
                        .sequence
                        .checked_add(1)
                        .ok_or(StoreError::Capacity)?
                || record.previous_sha256 != history.previous_sha256
            {
                return Err(StoreError::Corrupt);
            }
            history.apply(record.event)?;
            history.sequence = record.sequence;
            history.previous_sha256 = digest(&bytes);
        }
        let interrupted: Vec<_> = history
            .conversations
            .values()
            .filter(|conversation| conversation.pending_request.is_some())
            .cloned()
            .collect();
        for mut conversation in interrupted {
            let request = conversation
                .pending_request
                .take()
                .ok_or(StoreError::Corrupt)?;
            let turn = conversation
                .turns
                .iter()
                .find(|turn| turn.request_id == request)
                .ok_or(StoreError::Corrupt)?;
            let user = conversation
                .display
                .entries
                .iter_mut()
                .find(|entry| entry.id == turn.user_entry)
                .ok_or(StoreError::Corrupt)?;
            user.state = EntryState::Interrupted;
            // Preserve recorded time; an untrusted system-clock change cannot
            // revive request state. No active worker or approval is restored.
            history.persist(Event::Snapshot { conversation })?;
        }
        Ok(history)
    }

    fn apply(&mut self, event: Event) -> Result<(), StoreError> {
        match event {
            Event::Snapshot { conversation } => {
                validate(&conversation)?;
                if self.removed.contains(&conversation.display.id)
                    || (!self.conversations.contains_key(&conversation.display.id)
                        && self.conversations.len() >= MAX_CONVERSATIONS)
                {
                    return Err(StoreError::Corrupt);
                }
                self.conversations
                    .insert(conversation.display.id.clone(), conversation);
            }
            Event::Remove { id, updated_at: _ } => {
                if !valid_id(&id)
                    || self.conversations.remove(&id).is_none()
                    || !self.removed.insert(id)
                {
                    return Err(StoreError::Corrupt);
                }
            }
            Event::Invalidate { updated_at: _ } => {
                for conversation in self.conversations.values_mut() {
                    if conversation.pending_request.is_some() {
                        return Err(StoreError::Corrupt);
                    }
                    for turn in &mut conversation.turns {
                        turn.eligible = false;
                    }
                }
            }
        }
        Ok(())
    }

    fn ensure_current(&self) -> Result<(), StoreError> {
        if self.fault {
            return Err(StoreError::Corrupt);
        }
        let mut records = Vec::new();
        for (_, bytes) in self.store.records()? {
            let record: Record = serde_json::from_slice(&bytes).map_err(|_| StoreError::Corrupt)?;
            records.push((record, bytes));
        }
        if u64::try_from(records.len()).map_err(|_| StoreError::Capacity)? != self.sequence {
            return Err(StoreError::Corrupt);
        }
        records.sort_by_key(|(record, _)| record.sequence);
        let mut previous = String::new();
        for (offset, (record, bytes)) in records.into_iter().enumerate() {
            if record.version != 1
                || record.sequence != u64::try_from(offset + 1).map_err(|_| StoreError::Capacity)?
                || record.previous_sha256 != previous
            {
                return Err(StoreError::Corrupt);
            }
            previous = digest(&bytes);
        }
        if previous != self.previous_sha256 {
            return Err(StoreError::Corrupt);
        }
        Ok(())
    }

    fn persist(&mut self, event: Event) -> Result<(), StoreError> {
        self.ensure_current()?;
        match &event {
            Event::Snapshot { conversation } => {
                validate(conversation)?;
                if self.removed.contains(&conversation.display.id) {
                    return Err(StoreError::Denied);
                }
                if !self.conversations.contains_key(&conversation.display.id)
                    && self.conversations.len() >= MAX_CONVERSATIONS
                {
                    return Err(StoreError::Capacity);
                }
            }
            Event::Remove { id, .. } => {
                if !self.conversations.contains_key(id) {
                    return Err(StoreError::Denied);
                }
            }
            Event::Invalidate { .. } => {
                if self
                    .conversations
                    .values()
                    .any(|conversation| conversation.pending_request.is_some())
                {
                    return Err(StoreError::Conflict);
                }
            }
        }
        let record = Record {
            version: 1,
            sequence: self.sequence.checked_add(1).ok_or(StoreError::Capacity)?,
            previous_sha256: self.previous_sha256.clone(),
            event,
        };
        let bytes = serde_json::to_vec(&record).map_err(|_| StoreError::Corrupt)?;
        if let Err(error) = self.store.append(&new_id()?, &bytes) {
            self.fault = true;
            return Err(error);
        }
        let sequence = record.sequence;
        if let Err(error) = self.apply(record.event) {
            self.fault = true;
            return Err(error);
        }
        self.sequence = sequence;
        self.previous_sha256 = digest(&bytes);
        Ok(())
    }

    pub(crate) fn list(&self) -> Result<Vec<ConversationSummary>, StoreError> {
        self.ensure_current()?;
        let mut summaries: Vec<_> = self
            .conversations
            .values()
            .map(|conversation| {
                let display = &conversation.display;
                ConversationSummary {
                    id: display.id.clone(),
                    title: display.title.clone(),
                    updated_at: display.updated_at,
                    entry_count: display.entries.len(),
                }
            })
            .collect();
        summaries.sort_by(|a, b| {
            b.updated_at
                .cmp(&a.updated_at)
                .then_with(|| a.id.cmp(&b.id))
        });
        Ok(summaries)
    }
    pub(crate) fn can_append_request(&self) -> Result<bool, StoreError> {
        self.ensure_current()?;
        // Space for two maximum JSON-escaped snapshots. No automatic eviction.
        self.store.has_capacity(Domain::History, 8 * 1024 * 1024, 2)
    }

    pub(crate) fn create(&mut self, now: u64) -> Result<Conversation, StoreError> {
        if self.conversations.len() >= MAX_CONVERSATIONS {
            return Err(StoreError::Capacity);
        }
        let display = Conversation {
            id: new_id()?,
            title: "New conversation".to_owned(),
            updated_at: now,
            entries: Vec::new(),
        };
        self.persist(Event::Snapshot {
            conversation: StoredConversation {
                version: 1,
                provider: PROVIDER.to_owned(),
                model: MODEL.to_owned(),
                display: display.clone(),
                turns: Vec::new(),
                pending_request: None,
            },
        })?;
        Ok(display)
    }

    pub(crate) fn load(&self, id: &str) -> Result<Conversation, StoreError> {
        self.ensure_current()?;
        Ok(self
            .conversations
            .get(id)
            .ok_or(StoreError::Denied)?
            .display
            .clone())
    }

    pub(crate) fn remaining_text(&self, id: &str) -> Result<usize, StoreError> {
        self.ensure_current()?;
        let conversation = self.conversations.get(id).ok_or(StoreError::Denied)?;
        MAX_TEXT
            .checked_sub(text_bytes(&conversation.display)?)
            .ok_or(StoreError::Corrupt)
    }

    pub(crate) fn begin_request(
        &mut self,
        id: &str,
        request_id: &str,
        prompt: &str,
        data_class: &str,
        now: u64,
    ) -> Result<Conversation, StoreError> {
        if !valid_id(request_id)
            || !eligible_class(data_class)
            || prompt.trim().is_empty()
            || prompt.len() > 32 * 1024
            || prompt.contains('\0')
        {
            return Err(StoreError::Denied);
        }
        let mut conversation = self
            .conversations
            .get(id)
            .ok_or(StoreError::Denied)?
            .clone();
        if conversation.pending_request.is_some()
            || conversation
                .turns
                .iter()
                .any(|turn| turn.request_id == request_id)
        {
            return Err(StoreError::Conflict);
        }
        if conversation.display.entries.len() + 2 > MAX_ENTRIES
            || conversation
                .turns
                .iter()
                .filter(|turn| turn.eligible)
                .count()
                >= MAX_COMPLETE_TURNS
            || text_bytes(&conversation.display)?
                .checked_add(prompt.len())
                .is_none_or(|sum| sum > MAX_TEXT)
        {
            return Err(StoreError::Capacity);
        }
        if conversation.display.entries.is_empty() {
            conversation.display.title = title(prompt);
        }
        let entry_id = new_id()?;
        conversation.display.entries.push(Entry {
            id: entry_id.clone(),
            role: "user".to_owned(),
            content: prompt.to_owned(),
            request_id: request_id.to_owned(),
            state: EntryState::Completed,
        });
        conversation.turns.push(Turn {
            request_id: request_id.to_owned(),
            user_entry: entry_id,
            assistant_entry: None,
            data_class: data_class.to_owned(),
            eligible: false,
        });
        conversation.pending_request = Some(request_id.to_owned());
        conversation.display.updated_at = conversation.display.updated_at.max(now);
        let display = conversation.display.clone();
        self.persist(Event::Snapshot { conversation })?;
        Ok(display)
    }

    #[allow(clippy::too_many_arguments)] // Keep bound request and terminal eligibility explicit.
    pub(crate) fn finish_request(
        &mut self,
        id: &str,
        request_id: &str,
        answer: &str,
        state: EntryState,
        eligible: bool,
        now: u64,
    ) -> Result<Conversation, StoreError> {
        let mut conversation = self
            .conversations
            .get(id)
            .ok_or(StoreError::Denied)?
            .clone();
        if conversation.pending_request.as_deref() != Some(request_id) {
            return Err(StoreError::Conflict);
        }
        if answer.contains('\0')
            || text_bytes(&conversation.display)?
                .checked_add(answer.len())
                .is_none_or(|sum| sum > MAX_TEXT)
        {
            return Err(StoreError::Capacity);
        }
        if eligible && (state != EntryState::Completed || answer.trim().is_empty()) {
            return Err(StoreError::Denied);
        }
        let turn = conversation
            .turns
            .iter_mut()
            .find(|turn| turn.request_id == request_id)
            .ok_or(StoreError::Corrupt)?;
        let entry_id = new_id()?;
        turn.assistant_entry = Some(entry_id.clone());
        turn.eligible = eligible;
        conversation.display.entries.push(Entry {
            id: entry_id,
            role: "assistant".to_owned(),
            content: answer.to_owned(),
            request_id: request_id.to_owned(),
            state,
        });
        conversation.pending_request = None;
        conversation.display.updated_at = conversation.display.updated_at.max(now);
        let display = conversation.display.clone();
        self.persist(Event::Snapshot { conversation })?;
        Ok(display)
    }

    pub(crate) fn eligible_pairs(&self, id: &str) -> Result<Vec<EligiblePair>, StoreError> {
        self.ensure_current()?;
        let conversation = self.conversations.get(id).ok_or(StoreError::Denied)?;
        validate(conversation)?;
        let mut pairs = Vec::new();
        for turn in conversation.turns.iter().filter(|turn| turn.eligible) {
            let prompt = conversation
                .display
                .entries
                .iter()
                .find(|entry| entry.id == turn.user_entry)
                .ok_or(StoreError::Corrupt)?;
            let answer = conversation
                .display
                .entries
                .iter()
                .find(|entry| Some(&entry.id) == turn.assistant_entry.as_ref())
                .ok_or(StoreError::Corrupt)?;
            pairs.push(EligiblePair {
                prompt: prompt.content.clone(),
                answer: answer.content.clone(),
                data_class: turn.data_class.clone(),
            });
        }
        Ok(pairs)
    }

    pub(crate) fn invalidate_continuation(&mut self, now: u64) -> Result<(), StoreError> {
        self.persist(Event::Invalidate { updated_at: now })
    }

    /// Internal host call only AFTER separate native confirmation. This is a
    /// logical visibility tombstone, not secure erasure or quota reclamation.
    pub(crate) fn remove_after_native_confirmation(
        &mut self,
        id: &str,
        now: u64,
    ) -> Result<(), StoreError> {
        let conversation = self.conversations.get(id).ok_or(StoreError::Denied)?;
        if conversation.pending_request.is_some() {
            return Err(StoreError::Conflict);
        }
        self.persist(Event::Remove {
            id: id.to_owned(),
            updated_at: now,
        })
    }

    #[cfg(test)]
    pub(crate) fn memory() -> Self {
        Self::restore(PrivateStore::memory(Domain::History)).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_fault_blocks_cached_history(history: &mut History, id: &str) {
        assert_eq!(history.list().unwrap_err(), StoreError::Corrupt);
        assert_eq!(history.load(id).unwrap_err(), StoreError::Corrupt);
        assert_eq!(history.remaining_text(id).unwrap_err(), StoreError::Corrupt);
        assert!(matches!(
            history.eligible_pairs(id),
            Err(StoreError::Corrupt)
        ));
        assert_eq!(
            history.can_append_request().unwrap_err(),
            StoreError::Corrupt
        );
        assert_eq!(history.create(4).unwrap_err(), StoreError::Corrupt);
    }

    #[test]
    fn failed_terminal_append_before_write_blocks_until_replay_marks_interrupted() {
        let mut history = History::memory();
        let id = history.create(1).unwrap().id;
        history
            .begin_request(&id, "request", "Synthetic prompt", "synthetic", 2)
            .unwrap();
        let records_before = history.store.records().unwrap().len();
        history.store.fail_next_append_for_test(false);
        assert_eq!(
            history
                .finish_request(
                    &id,
                    "request",
                    "Synthetic answer",
                    EntryState::Completed,
                    true,
                    3,
                )
                .unwrap_err(),
            StoreError::Io
        );
        assert_eq!(history.store.records().unwrap().len(), records_before);
        assert_fault_blocks_cached_history(&mut history, &id);

        // A clean restart replays the unchanged pending intent, never the
        // failed cached completion. It appends only the interrupted state.
        let mut restored = History::restore(history.store).unwrap();
        let saved = restored.load(&id).unwrap();
        assert_eq!(saved.entries.len(), 1);
        assert_eq!(saved.entries[0].state, EntryState::Interrupted);
        assert!(restored.eligible_pairs(&id).unwrap().is_empty());
        assert_eq!(restored.store.records().unwrap().len(), records_before + 1);
        restored
            .begin_request(&id, "fresh", "Fresh synthetic prompt", "synthetic", 5)
            .unwrap();
    }

    #[test]
    fn unacknowledged_terminal_append_blocks_cache_but_replay_preserves_valid_record() {
        let mut history = History::memory();
        let id = history.create(1).unwrap().id;
        history
            .begin_request(&id, "request", "Synthetic prompt", "synthetic", 2)
            .unwrap();
        let records_before = history.store.records().unwrap().len();
        history.store.fail_next_append_for_test(true);
        assert_eq!(
            history
                .finish_request(
                    &id,
                    "request",
                    "Synthetic answer",
                    EntryState::Completed,
                    true,
                    3,
                )
                .unwrap_err(),
            StoreError::Io
        );
        assert_eq!(history.store.records().unwrap().len(), records_before + 1);
        assert_fault_blocks_cached_history(&mut history, &id);

        // Validated immutable records, not an assumed refund or cached state,
        // decide recovery. Restoration still grants no active request/review.
        let mut restored = History::restore(history.store).unwrap();
        let saved = restored.load(&id).unwrap();
        assert_eq!(saved.entries.len(), 2);
        assert_eq!(saved.entries[1].state, EntryState::Completed);
        assert_eq!(restored.eligible_pairs(&id).unwrap().len(), 1);
        assert_eq!(restored.store.records().unwrap().len(), records_before + 1);
        restored
            .begin_request(&id, "fresh", "Fresh synthetic prompt", "synthetic", 5)
            .unwrap();
    }

    #[test]
    fn completed_pairs_restore_but_pending_requests_become_interrupted() {
        let mut history = History::memory();
        let id = history.create(1).unwrap().id;
        history
            .begin_request(&id, "synthetic-1", "Synthetic first prompt", "synthetic", 2)
            .unwrap();
        history
            .finish_request(
                &id,
                "synthetic-1",
                "Synthetic answer",
                EntryState::Completed,
                true,
                3,
            )
            .unwrap();
        history
            .begin_request(
                &id,
                "synthetic-2",
                "Second synthetic prompt",
                "synthetic",
                4,
            )
            .unwrap();
        let mut restored = History::restore(history.store).unwrap();
        assert_eq!(
            restored.load(&id).unwrap().entries.last().unwrap().state,
            EntryState::Interrupted
        );
        assert_eq!(restored.eligible_pairs(&id).unwrap().len(), 1);
        assert!(restored
            .finish_request(&id, "synthetic-2", "", EntryState::Failed, false, 5)
            .is_err());
        restored
            .begin_request(
                &id,
                "synthetic-3",
                "Fresh synthetic request",
                "synthetic",
                6,
            )
            .unwrap();
    }

    #[test]
    fn partial_answers_never_continue_and_credential_change_invalidates_complete_pairs() {
        let mut history = History::memory();
        let id = history.create(1).unwrap().id;
        for (number, state) in [
            EntryState::Cancelled,
            EntryState::Truncated,
            EntryState::Refused,
            EntryState::Failed,
        ]
        .into_iter()
        .enumerate()
        {
            let request = format!("request-{number}");
            history
                .begin_request(&id, &request, "Synthetic prompt", "synthetic", 2)
                .unwrap();
            assert!(history
                .finish_request(&id, &request, "Synthetic partial", state, true, 3)
                .is_err());
            history
                .finish_request(&id, &request, "Synthetic partial", state, false, 3)
                .unwrap();
        }
        assert!(history.eligible_pairs(&id).unwrap().is_empty());
        history
            .begin_request(&id, "complete", "Synthetic prompt", "synthetic", 4)
            .unwrap();
        history
            .finish_request(
                &id,
                "complete",
                "Synthetic answer",
                EntryState::Completed,
                true,
                5,
            )
            .unwrap();
        history.invalidate_continuation(6).unwrap();
        let restored = History::restore(history.store).unwrap();
        assert!(restored.eligible_pairs(&id).unwrap().is_empty());
        assert_eq!(restored.load(&id).unwrap().entries.len(), 10);
    }

    #[test]
    fn deletion_is_only_visible_history_and_preserves_encrypted_event_records() {
        let mut history = History::memory();
        let id = history.create(1).unwrap().id;
        let count = history.store.records().unwrap().len();
        history.remove_after_native_confirmation(&id, 2).unwrap();
        assert_eq!(history.store.records().unwrap().len(), count + 1);
        assert!(history.load(&id).is_err());
        let restored = History::restore(history.store).unwrap();
        assert!(restored.list().unwrap().is_empty());
    }

    #[test]
    fn legacy_pairs_unknown_fields_and_broken_chains_deny_without_reset() {
        let mut history = History::memory();
        history.create(1).unwrap();
        let original = history.store.records().unwrap()[0].1.clone();
        for (path, value) in [
            ("model", serde_json::json!("retired-model")),
            ("provider", serde_json::json!("retired-provider")),
            ("approval", serde_json::json!(true)),
        ] {
            let mut data: serde_json::Value = serde_json::from_slice(&original).unwrap();
            data["event"]["conversation"][path] = value;
            let mut store = PrivateStore::memory(Domain::History);
            store
                .append(&new_id().unwrap(), &serde_json::to_vec(&data).unwrap())
                .unwrap();
            assert!(History::restore(store).is_err());
        }
        let mut data: serde_json::Value = serde_json::from_slice(&original).unwrap();
        data["previousSha256"] = serde_json::json!("not the genesis");
        let mut store = PrivateStore::memory(Domain::History);
        store
            .append(&new_id().unwrap(), &serde_json::to_vec(&data).unwrap())
            .unwrap();
        assert!(History::restore(store).is_err());
    }

    #[test]
    fn history_limits_deny_without_silent_truncation_or_eviction() {
        let mut history = History::memory();
        let id = history.create(1).unwrap().id;
        history
            .begin_request(&id, "request", "Synthetic prompt", "synthetic", 2)
            .unwrap();
        let too_large = "x".repeat(MAX_TEXT);
        assert_eq!(
            history
                .finish_request(&id, "request", &too_large, EntryState::Completed, true, 3)
                .unwrap_err(),
            StoreError::Capacity
        );
        assert_eq!(history.load(&id).unwrap().entries.len(), 1);
        history
            .finish_request(
                &id,
                "request",
                "Synthetic answer",
                EntryState::Completed,
                true,
                3,
            )
            .unwrap();
        assert!(history
            .begin_request(&id, "request", "Replayed", "synthetic", 4)
            .is_err());
        assert!(history
            .begin_request(&id, "unknown", "Synthetic prompt", "unknown", 4)
            .is_err());
    }
}
