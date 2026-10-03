//! Native-confirmed, dated owner attestation; never proof of Meta entitlement.
use crate::private_store::{new_id, Domain, PrivateStore, StoreError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::Path};

pub const TERMS_REVISION: &str = "2026-10-02";
const EXPIRY: u64 = 1_793_577_600;
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
                || r.revision != TERMS_REVISION
            {
                return Err(StoreError::Corrupt);
            }
            sequence += 1;
            head = hash(&bytes);
            accepted = r.accepted;
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
            || (accepted && !(1_790_899_200..EXPIRY).contains(&now))
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
        let accepted = self.healthy() && self.accepted && now >= self.maximum_time && now < EXPIRY;
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
        let mut c = Contributor::restore(PrivateStore::memory(Domain::Terms)).unwrap();
        assert!(!c.status(1_790_978_400).accepted);
        c.set(true, 1_790_978_400).unwrap();
        let mut c = Contributor::restore(c.store).unwrap();
        assert!(c.status(1_790_978_400).accepted);
        assert!(!c.status(EXPIRY).accepted);
        assert!(!c.status(1_790_978_399).accepted);
        c.set(false, 1_790_978_400).unwrap();
        assert!(!c.status(1_790_978_400).accepted);
    }
}
