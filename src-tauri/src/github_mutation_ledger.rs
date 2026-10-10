//! Metadata-only append ledger. The private backend has NO native implementation
//! in this slice: no store/domain opening, encryption or owner-state mutation.
//! Future native integration must use the sole reviewed github-mutations-v1
//! domain and independently qualify immutable encrypted publication/recovery.
use super::github_mutation_validation::EffectIds;
use super::{digest, hash, Binding, Error, Operation, Plan, Resource, CONTRACT_VERSION};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const MAX_RECORD_BYTES: usize = 16 * 1024;
const MAX_TOTAL_BYTES: usize = 4 * 1024 * 1024;
const MAX_RECORDS: usize = 512;

pub(super) fn reserve_capacity(count: usize, used_bytes: usize) -> Result<(), Error> {
    // This exact worst-case reservation is shared by admission and pure
    // boundary fixtures; it neither allocates nor opens a store.
    if count.checked_add(2).is_none_or(|n| n > MAX_RECORDS)
        || used_bytes
            .checked_add(2 * MAX_RECORD_BYTES)
            .is_none_or(|n| n > MAX_TOTAL_BYTES)
    {
        Err(Error::Bounds)
    } else {
        Ok(())
    }
}

pub(super) trait RecordBackend {
    fn records(&self) -> Result<Vec<(String, Vec<u8>)>, Error>;
    /// Failure does not promise that no immutable record became visible.
    fn append(&mut self, name: &str, bytes: &[u8]) -> Result<(), Error>;
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(super) enum Terminal {
    Verified(EffectIds),
    RejectedNoChange,
    CancelledBeforeDispatch,
    AttentionRequired,
}
impl Terminal {
    fn valid(&self) -> bool {
        match self {
            Self::Verified(ids) => ids.valid(),
            _ => true,
        }
    }
    fn unresolved(&self) -> bool {
        matches!(self, Self::AttentionRequired)
    }
}

#[derive(Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Metadata {
    plan_id: String,
    epoch: u64,
    binding: Binding,
    operation: Operation,
    resource: Resource,
    payload_hash: String,
    review_hash: String,
    snapshot_hash: String,
    issued_ms: u64,
    expires_ms: u64,
    dispatch_ms: u64,
}
impl Metadata {
    fn valid(&self) -> bool {
        valid_plan_id(&self.plan_id)
            && self.epoch > 0
            && self.binding.valid()
            && self.resource.valid()
            && digest(&self.payload_hash)
            && digest(&self.review_hash)
            && digest(&self.snapshot_hash)
            && self.dispatch_ms >= self.issued_ms
            && self.dispatch_ms < self.expires_ms
            && self.expires_ms.checked_sub(self.issued_ms) == Some(super::MAX_PLAN_MS)
            && matches!(
                (self.operation, &self.resource),
                (Operation::CreateBranch, Resource::Branch { .. })
                    | (
                        Operation::CreateFile | Operation::UpdateFile,
                        Resource::File { .. }
                    )
                    | (Operation::CreateIssue, Resource::NewIssue)
                    | (
                        Operation::CommentIssue | Operation::CloseIssue,
                        Resource::Issue { .. }
                    )
                    | (Operation::CreatePullRequest, Resource::NewPull { .. })
                    | (Operation::MergePullRequest, Resource::Pull { .. })
            )
    }
    fn valid_terminal(&self, terminal: &Terminal) -> bool {
        if !terminal.valid() {
            return false;
        }
        let Terminal::Verified(effect) = terminal else {
            return true;
        };
        match (&self.resource, self.operation) {
            (Resource::Branch { .. }, Operation::CreateBranch)
            | (Resource::File { .. }, Operation::CreateFile | Operation::UpdateFile) => {
                effect.commit.is_some() && effect.resource_id.is_none() && effect.number.is_none()
            }
            (Resource::NewIssue, Operation::CreateIssue)
            | (Resource::NewPull { .. }, Operation::CreatePullRequest) => {
                effect.resource_id.is_some() && effect.number.is_some() && effect.commit.is_none()
            }
            (Resource::Issue { .. }, Operation::CommentIssue) => {
                effect.resource_id.is_some() && effect.number.is_none() && effect.commit.is_none()
            }
            (Resource::Issue { id, number }, Operation::CloseIssue) => {
                effect.resource_id == Some(*id)
                    && effect.number == Some(*number)
                    && effect.commit.is_none()
            }
            (Resource::Pull { id, number }, Operation::MergePullRequest) => {
                effect.resource_id == Some(*id)
                    && effect.number == Some(*number)
                    && effect.commit.is_some()
            }
            _ => false,
        }
    }
}

#[derive(Copy, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum RecordKind {
    DispatchIntent,
    Terminal,
}
#[derive(Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Record {
    schema_version: u8,
    sequence: u64,
    operation_id: String,
    kind: RecordKind,
    metadata: Option<Metadata>,
    terminal: Option<Terminal>,
}
impl Record {
    fn parse(name: &str, bytes: &[u8]) -> Result<Self, Error> {
        if bytes.is_empty() || bytes.len() > MAX_RECORD_BYTES {
            return Err(Error::Ledger);
        }
        let value: Self = serde_json::from_slice(bytes).map_err(|_| Error::Ledger)?;
        if value.schema_version != CONTRACT_VERSION
            || value.sequence == 0
            || name != format!("{:032x}", value.sequence)
            || !valid_operation_id(&value.operation_id)
            || serde_json::to_vec(&value).map_err(|_| Error::Ledger)? != bytes
        {
            return Err(Error::Ledger);
        }
        match (value.kind, &value.metadata, &value.terminal) {
            (RecordKind::DispatchIntent, Some(metadata), None)
                if metadata.valid()
                    && value
                        .operation_id
                        .starts_with(&format!("{}:", metadata.plan_id)) => {}
            (RecordKind::Terminal, None, Some(terminal)) if terminal.valid() => {}
            _ => return Err(Error::Ledger),
        }
        Ok(value)
    }
}
fn valid_plan_id(value: &str) -> bool {
    let Some((nonce, sequence)) = value.split_once('-') else {
        return false;
    };
    super::hex(nonce, 32)
        && !sequence.is_empty()
        && !sequence.starts_with('0')
        && sequence.bytes().all(|b| b.is_ascii_digit())
        && sequence.parse::<u64>().is_ok_and(|sequence| sequence > 0)
}
fn valid_operation_id(value: &str) -> bool {
    let Some((plan, sequence)) = value.split_once(':') else {
        return false;
    };
    valid_plan_id(plan)
        && !sequence.starts_with('0')
        && sequence.bytes().all(|b| b.is_ascii_digit())
        && sequence.parse::<u64>().is_ok_and(|sequence| sequence > 0)
}

struct Entry {
    metadata: Metadata,
    terminal: Option<Terminal>,
}
pub(super) struct Ledger<B: RecordBackend> {
    backend: B,
    next_sequence: u64,
    used_bytes: usize,
    count: usize,
    entries: BTreeMap<String, Entry>,
    record_hashes: BTreeMap<u64, String>,
    poisoned: bool,
}
impl<B: RecordBackend> Ledger<B> {
    pub(super) fn restore(backend: B) -> Result<Self, Error> {
        let bodies = backend.records()?;
        if bodies.len() > MAX_RECORDS {
            return Err(Error::Ledger);
        }
        let mut records = BTreeMap::new();
        let mut names = BTreeSet::new();
        let mut record_hashes = BTreeMap::new();
        let mut used_bytes = 0usize;
        for (name, bytes) in bodies {
            used_bytes = used_bytes.checked_add(bytes.len()).ok_or(Error::Ledger)?;
            if used_bytes > MAX_TOTAL_BYTES || !names.insert(name.clone()) {
                return Err(Error::Ledger);
            }
            let record = Record::parse(&name, &bytes)?;
            record_hashes.insert(record.sequence, hash(&bytes));
            if records.insert(record.sequence, record).is_some() {
                return Err(Error::Ledger);
            }
        }
        let mut entries: BTreeMap<String, Entry> = BTreeMap::new();
        let mut next_sequence = 1u64;
        for (sequence, record) in &records {
            if *sequence != next_sequence {
                return Err(Error::Ledger);
            }
            next_sequence = next_sequence
                .checked_add(1)
                .ok_or(Error::SequenceExhausted)?;
            match record.kind {
                RecordKind::DispatchIntent => {
                    let metadata = record.metadata.clone().ok_or(Error::Ledger)?;
                    if entries
                        .values()
                        .any(|entry| entry.metadata.plan_id == metadata.plan_id)
                        || record.operation_id != format!("{}:{sequence}", metadata.plan_id)
                        || entries
                            .insert(
                                record.operation_id.clone(),
                                Entry {
                                    metadata,
                                    terminal: None,
                                },
                            )
                            .is_some()
                    {
                        return Err(Error::Ledger);
                    }
                }
                RecordKind::Terminal => {
                    let entry = entries.get_mut(&record.operation_id).ok_or(Error::Ledger)?;
                    let terminal = record.terminal.clone().ok_or(Error::Ledger)?;
                    if entry.terminal.is_some() || !entry.metadata.valid_terminal(&terminal) {
                        return Err(Error::Ledger);
                    }
                    entry.terminal = Some(terminal);
                }
            }
        }
        Ok(Self {
            backend,
            next_sequence,
            used_bytes,
            count: records.len(),
            entries,
            record_hashes,
            poisoned: false,
        })
    }
    pub(super) fn admit(&self, binding: &Binding) -> Result<(), Error> {
        if self.poisoned || !binding.valid() {
            return Err(Error::Ledger);
        }
        if self.entries.values().any(|entry| {
            entry.metadata.binding.repository_id == binding.repository_id
                && entry.terminal.as_ref().is_none_or(Terminal::unresolved)
        }) {
            return Err(Error::Uncertainty);
        }
        // Reserve worst-case intent AND terminal before any remote effect.
        reserve_capacity(self.count, self.used_bytes)
    }
    fn append_verified(&mut self, record: &Record) -> Result<(), Error> {
        if self.poisoned {
            return Err(Error::Ledger);
        }
        let bytes = serde_json::to_vec(record).map_err(|_| Error::Ledger)?;
        let name = format!("{:032x}", record.sequence);
        Record::parse(&name, &bytes)?;
        if self.count >= MAX_RECORDS
            || self
                .used_bytes
                .checked_add(bytes.len())
                .is_none_or(|n| n > MAX_TOTAL_BYTES)
        {
            return Err(Error::Bounds);
        }
        let outcome = self.backend.append(&name, &bytes).and_then(|()| {
            let records = self.backend.records()?;
            // Verify the complete immutable ledger, not merely an existence bit.
            if records.len() != self.count + 1
                || records
                    .iter()
                    .filter(|(n, b)| *n == name && *b == bytes)
                    .count()
                    != 1
            {
                return Err(Error::Ledger);
            }
            let total = records.iter().try_fold(0usize, |sum, (_, body)| {
                sum.checked_add(body.len()).ok_or(Error::Ledger)
            })?;
            if total != self.used_bytes + bytes.len() {
                return Err(Error::Ledger);
            }
            // All prior records must remain byte-identical and parseable.
            let mut seen = BTreeSet::new();
            for (name, body) in records {
                let parsed = Record::parse(&name, &body)?;
                if !seen.insert(parsed.sequence)
                    || parsed.sequence > record.sequence
                    || (parsed.sequence < record.sequence
                        && self.record_hashes.get(&parsed.sequence) != Some(&hash(&body)))
                {
                    return Err(Error::Ledger);
                }
            }
            if seen.len() != self.count + 1
                || seen.first() != Some(&1)
                || seen.last() != Some(&record.sequence)
            {
                return Err(Error::Ledger);
            }
            Ok(())
        });
        if outcome.is_err() {
            self.poisoned = true;
            return Err(Error::Ledger);
        }
        self.count += 1;
        self.used_bytes += bytes.len();
        self.record_hashes.insert(record.sequence, hash(&bytes));
        self.next_sequence = self
            .next_sequence
            .checked_add(1)
            .ok_or(Error::SequenceExhausted)?;
        Ok(())
    }
    pub(super) fn begin(&mut self, plan: &Plan, now_ms: u64) -> Result<String, Error> {
        self.admit(&plan.snapshot.binding)?;
        if self
            .entries
            .values()
            .any(|entry| entry.metadata.plan_id == plan.id)
        {
            return Err(Error::Replay);
        }
        let operation_id = format!("{}:{}", plan.id, self.next_sequence);
        let metadata = Metadata {
            plan_id: plan.id.clone(),
            epoch: plan.epoch,
            binding: plan.snapshot.binding.clone(),
            operation: plan.intent.operation(),
            resource: plan.intent.resource(),
            payload_hash: hash(&plan.request.payload),
            review_hash: plan.review_hash.clone(),
            snapshot_hash: hash(&serde_json::to_vec(&plan.snapshot).map_err(|_| Error::Malformed)?),
            issued_ms: plan.issued_ms,
            expires_ms: plan.expires_ms,
            dispatch_ms: now_ms,
        };
        if !metadata.valid() {
            return Err(Error::Ledger);
        }
        let record = Record {
            schema_version: CONTRACT_VERSION,
            sequence: self.next_sequence,
            operation_id: operation_id.clone(),
            kind: RecordKind::DispatchIntent,
            metadata: Some(metadata.clone()),
            terminal: None,
        };
        self.append_verified(&record)?;
        self.entries.insert(
            operation_id.clone(),
            Entry {
                metadata,
                terminal: None,
            },
        );
        Ok(operation_id)
    }
    pub(super) fn finish(&mut self, operation_id: &str, terminal: Terminal) -> Result<(), Error> {
        let entry = self.entries.get(operation_id).ok_or(Error::Replay)?;
        if entry.terminal.is_some() || !entry.metadata.valid_terminal(&terminal) {
            return Err(Error::Replay);
        }
        let record = Record {
            schema_version: CONTRACT_VERSION,
            sequence: self.next_sequence,
            operation_id: operation_id.to_owned(),
            kind: RecordKind::Terminal,
            metadata: None,
            terminal: Some(terminal.clone()),
        };
        self.append_verified(&record)?;
        self.entries
            .get_mut(operation_id)
            .ok_or(Error::Ledger)?
            .terminal = Some(terminal);
        Ok(())
    }
    #[cfg(test)]
    pub(super) fn into_backend(self) -> B {
        self.backend
    }
}

#[cfg(test)]
#[derive(Clone, Default)]
pub(super) struct Memory {
    pub records: Vec<(String, Vec<u8>)>,
    pub fail_append: Option<(usize, bool)>,
    pub append_calls: usize,
}
#[cfg(test)]
impl RecordBackend for Memory {
    fn records(&self) -> Result<Vec<(String, Vec<u8>)>, Error> {
        Ok(self.records.clone())
    }
    fn append(&mut self, name: &str, bytes: &[u8]) -> Result<(), Error> {
        self.append_calls += 1;
        let failure = self
            .fail_append
            .filter(|(call, _)| *call == self.append_calls);
        if failure == Some((self.append_calls, false)) {
            return Err(Error::Ledger);
        }
        if self.records.iter().any(|(existing, _)| existing == name) {
            return Err(Error::Ledger);
        }
        self.records.push((name.to_owned(), bytes.to_vec()));
        if failure.is_some() {
            return Err(Error::Ledger);
        }
        Ok(())
    }
}
