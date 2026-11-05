//! Native-confirmed, dated owner attestation; never proof of Meta entitlement.
use crate::{
    contributor_policy,
    private_store::{new_id, Domain, PrivateStore, StoreError},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::Path};

pub(crate) use crate::contributor_policy::TERMS_REVISION;
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContributorStatus {
    pub terms_revision: &'static str,
    pub accepted: bool,
    pub eligible: bool,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Record {
    version: u32,
    sequence: u64,
    previous: String,
    revision: String,
    accepted: bool,
    timestamp: u64,
}
fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
pub struct Contributor {
    store: PrivateStore,
    sequence: u64,
    head: String,
    accepted: bool,
    maximum_time: u64,
    fault: bool,
}
impl Contributor {
    pub fn open(base: &Path) -> Result<Self, StoreError> {
        Self::restore(PrivateStore::open(base, Domain::Terms)?)
    }
    fn replay(store: &PrivateStore) -> Result<(u64, String, bool, u64), StoreError> {
        let mut ordered = BTreeMap::new();
        for (_, bytes) in store.records()? {
            let r: Record = serde_json::from_slice(&bytes).map_err(|_| StoreError::Corrupt)?;
            if ordered.insert(r.sequence, (r, bytes)).is_some() {
                return Err(StoreError::Corrupt);
            }
        }
        let (mut sequence, mut head, mut accepted, mut time) = (0, String::new(), false, 0);
        for (_, (r, bytes)) in ordered {
            if r.version != 1
                || r.sequence != sequence + 1
                || r.previous != head
                || r.timestamp < time
                || ![TERMS_REVISION, contributor_policy::LEGACY_REVISION]
                    .contains(&r.revision.as_str())
                || (r.accepted
                    && !contributor_policy::accepted_record_valid(&r.revision, r.timestamp))
            {
                return Err(StoreError::Corrupt);
            }
            sequence += 1;
            head = hash(&bytes);
            accepted = r.accepted && r.revision == TERMS_REVISION;
            time = r.timestamp;
        }
        Ok((sequence, head, accepted, time))
    }
    fn restore(store: PrivateStore) -> Result<Self, StoreError> {
        let (sequence, head, accepted, maximum_time) = Self::replay(&store)?;
        Ok(Self {
            store,
            sequence,
            head,
            accepted,
            maximum_time,
            fault: false,
        })
    }
    fn healthy(&self) -> bool {
        !self.fault
            && Self::replay(&self.store)
                .is_ok_and(|(n, h, _, _)| n == self.sequence && h == self.head)
    }
    pub fn set(&mut self, accepted: bool, now: u64) -> Result<(), StoreError> {
        if !self.healthy()
            || now < self.maximum_time
            || (accepted && !contributor_policy::current(now))
        {
            return Err(StoreError::Denied);
        }
        let r = Record {
            version: 1,
            sequence: self.sequence + 1,
            previous: self.head.clone(),
            revision: TERMS_REVISION.into(),
            accepted,
            timestamp: now,
        };
        let bytes = serde_json::to_vec(&r).map_err(|_| StoreError::Corrupt)?;
        if let Err(e) = self.store.append(&new_id()?, &bytes) {
            self.fault = true;
            return Err(e);
        }
        self.sequence += 1;
        self.head = hash(&bytes);
        self.accepted = accepted;
        self.maximum_time = now;
        Ok(())
    }
    pub fn status(&self, now: u64) -> ContributorStatus {
        let accepted = self.healthy()
            && self.accepted
            && now >= self.maximum_time
            && contributor_policy::current(now);
        ContributorStatus {
            terms_revision: TERMS_REVISION,
            accepted,
            eligible: accepted,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn terms_restore_and_revocation_never_grant_live_entitlement() {
        let now = contributor_policy::REVIEWED_AT + 100;
        let mut c = Contributor::restore(PrivateStore::memory(Domain::Terms)).unwrap();
        assert!(!c.status(now).accepted);
        c.set(true, now).unwrap();
        let mut c = Contributor::restore(c.store).unwrap();
        assert!(c.status(now).accepted);
        assert!(!c.status(contributor_policy::EXPIRES_AT).accepted);
        assert!(!c.status(now - 1).accepted);
        c.set(false, now).unwrap();
        assert!(!c.status(now).accepted);
        assert!(c.set(true, contributor_policy::EXPIRES_AT).is_err());
    }
    #[test]
    fn legacy_acceptance_replays_without_reset_but_requires_fresh_review() {
        let mut store = PrivateStore::memory(Domain::Terms);
        let record = Record {
            version: 1,
            sequence: 1,
            previous: String::new(),
            revision: contributor_policy::LEGACY_REVISION.into(),
            accepted: true,
            timestamp: 1_790_978_400,
        };
        let bytes = serde_json::to_vec(&record).unwrap();
        store.append(&new_id().unwrap(), &bytes).unwrap();
        let mut c = Contributor::restore(store).unwrap();
        let now = contributor_policy::REVIEWED_AT;
        assert_eq!(c.sequence, 1);
        assert!(!c.status(now).accepted);
        c.set(true, now).unwrap();
        assert_eq!(c.sequence, 2);
        let mut restored = Contributor::restore(c.store).unwrap();
        assert!(restored.status(now).accepted);
        assert!(restored.set(true, now - 1).is_err());
        assert_eq!(restored.sequence, 2);
    }
    #[test]
    fn unknown_revision_or_out_of_window_legacy_record_denies_replay() {
        for (revision, timestamp) in [
            ("unknown", contributor_policy::REVIEWED_AT),
            (
                contributor_policy::LEGACY_REVISION,
                contributor_policy::REVIEWED_AT,
            ),
        ] {
            let mut store = PrivateStore::memory(Domain::Terms);
            let record = Record {
                version: 1,
                sequence: 1,
                previous: String::new(),
                revision: revision.into(),
                accepted: true,
                timestamp,
            };
            store
                .append(&new_id().unwrap(), &serde_json::to_vec(&record).unwrap())
                .unwrap();
            assert!(Contributor::restore(store).is_err());
        }
    }
}
