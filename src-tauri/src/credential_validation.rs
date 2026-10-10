//! Authentication proof belongs to the native host, not key syntax or React.
//! The protected record binds a successful fixed-model GET to the exact key.
//! It contains only a digest; no key, model response, URL override or authority.

use crate::private_store::{Domain, PrivateStore, StoreError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ValidationState {
    Missing,
    Unverified,
    Valid,
    Invalid,
    Unavailable,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    version: u8,
    sequence: u64,
    previous: String,
    fingerprint: Option<String>,
    state: ValidationState,
}

pub(crate) struct CredentialValidation {
    store: PrivateStore,
    sequence: u64,
    head: String,
    fingerprint: Option<String>,
    state: ValidationState,
    poisoned: bool,
}

fn fingerprint(key: &str) -> String {
    let mut hash = Sha256::new();
    hash.update(b"Aster/Meta/CredentialProof/v1\0");
    hash.update(crate::policy::DEFAULT_MODEL.as_bytes());
    hash.update(crate::policy::POLICY_VERSION.to_le_bytes());
    hash.update(b"\0");
    hash.update(key.as_bytes());
    hash.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

impl CredentialValidation {
    pub(crate) fn open(base: &Path) -> Result<Self, StoreError> {
        Self::restore(PrivateStore::open(base, Domain::CredentialProof)?)
    }

    fn restore(store: PrivateStore) -> Result<Self, StoreError> {
        let (sequence, head, fingerprint, state) = Self::replay(&store)?;
        Ok(Self {
            store,
            sequence,
            head,
            fingerprint,
            state,
            poisoned: false,
        })
    }

    fn replay(
        store: &PrivateStore,
    ) -> Result<(u64, String, Option<String>, ValidationState), StoreError> {
        let mut records = store.records()?;
        records.sort_by(|a, b| a.0.cmp(&b.0));
        let (mut sequence, mut head, mut saved_fingerprint, mut state) =
            (0u64, String::new(), None, ValidationState::Unverified);
        for (name, bytes) in records {
            let record: Record = serde_json::from_slice(&bytes).map_err(|_| StoreError::Corrupt)?;
            if record.version != 1
                || record.sequence != sequence.checked_add(1).ok_or(StoreError::Corrupt)?
                || record.previous != head
                || name != format!("{:032x}", record.sequence)
                || !matches!(
                    record.state,
                    ValidationState::Unverified | ValidationState::Valid | ValidationState::Invalid
                )
                || (record.state == ValidationState::Unverified && record.fingerprint.is_some())
                || (record.state != ValidationState::Unverified && record.fingerprint.is_none())
                || record.fingerprint.as_ref().is_some_and(|hash| {
                    hash.len() != 64
                        || !hash
                            .bytes()
                            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                })
            {
                return Err(StoreError::Corrupt);
            }
            sequence = record.sequence;
            head = Sha256::digest(&bytes)
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect();
            saved_fingerprint = record.fingerprint;
            state = record.state;
        }
        Ok((sequence, head, saved_fingerprint, state))
    }

    fn healthy(&self) -> bool {
        !self.poisoned
            && Self::replay(&self.store).is_ok_and(|(sequence, head, hash, state)| {
                sequence == self.sequence
                    && head == self.head
                    && hash == self.fingerprint
                    && state == self.state
            })
    }

    pub(crate) fn state(&self, key: Option<&str>) -> ValidationState {
        let Some(key) = key else {
            return ValidationState::Missing;
        };
        if !self.healthy() {
            return ValidationState::Unavailable;
        }
        if self
            .fingerprint
            .as_ref()
            .is_some_and(|hash| *hash == fingerprint(key))
        {
            self.state
        } else {
            ValidationState::Unverified
        }
    }

    pub(crate) fn require(&self, key: &str) -> Result<(), &'static str> {
        if self.state(Some(key)) == ValidationState::Valid {
            Ok(())
        } else {
            Err("credential_validation_required")
        }
    }

    fn append(
        &mut self,
        fingerprint: Option<String>,
        state: ValidationState,
    ) -> Result<(), StoreError> {
        if !self.healthy() {
            return Err(StoreError::Corrupt);
        }
        let sequence = self.sequence.checked_add(1).ok_or(StoreError::Capacity)?;
        let record = Record {
            version: 1,
            sequence,
            previous: self.head.clone(),
            fingerprint,
            state,
        };
        let bytes = serde_json::to_vec(&record).map_err(|_| StoreError::Corrupt)?;
        // Latch closed before a potentially uncertain durable append.
        self.poisoned = true;
        self.store.append(&format!("{sequence:032x}"), &bytes)?;
        self.sequence = sequence;
        self.head = Sha256::digest(&bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        self.fingerprint = record.fingerprint;
        self.state = state;
        self.poisoned = false;
        Ok(())
    }

    pub(crate) fn revoke(&mut self) -> Result<(), StoreError> {
        self.append(None, ValidationState::Unverified)
    }

    pub(crate) fn record_result(&mut self, key: &str, valid: bool) -> Result<(), StoreError> {
        self.append(
            Some(fingerprint(key)),
            if valid {
                ValidationState::Valid
            } else {
                ValidationState::Invalid
            },
        )
    }

    pub(crate) fn can_verify(&self) -> Result<bool, StoreError> {
        if !self.healthy() {
            return Err(StoreError::Corrupt);
        }
        // Two records for revocation/result and one reserved deletion tombstone.
        self.store
            .has_capacity(Domain::CredentialProof, 3 * 1024, 3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const KEY: &str = "LLM_0_SYNTHETIC_NOT_A_SECRET";
    fn empty() -> CredentialValidation {
        CredentialValidation::restore(PrivateStore::memory(Domain::CredentialProof)).unwrap()
    }
    #[test]
    fn syntax_and_renderer_cannot_make_authentication_proof() {
        let proof = empty();
        assert_eq!(proof.state(None), ValidationState::Missing);
        assert_eq!(proof.state(Some(KEY)), ValidationState::Unverified);
        assert!(proof.require(KEY).is_err());
    }
    #[test]
    fn proof_is_exact_key_bound_and_replacement_deletion_revoke_it() {
        let mut proof = empty();
        proof.record_result(KEY, true).unwrap();
        assert_eq!(proof.require(KEY), Ok(()));
        assert_eq!(
            proof.state(Some("different-synthetic-key")),
            ValidationState::Unverified
        );
        proof.revoke().unwrap();
        assert!(proof.require(KEY).is_err());
        proof.record_result(KEY, false).unwrap();
        assert_eq!(proof.state(Some(KEY)), ValidationState::Invalid);
    }
    #[test]
    fn restored_proof_contains_no_key_and_restores_no_request_authority() {
        let mut proof = empty();
        proof.record_result(KEY, true).unwrap();
        let records = proof.store.records().unwrap();
        assert!(!String::from_utf8(records[0].1.clone())
            .unwrap()
            .contains(KEY));
        let restored = CredentialValidation::restore(proof.store).unwrap();
        assert_eq!(restored.require(KEY), Ok(()));
        assert!(restored.require("changed-key").is_err());
    }
    #[test]
    fn corrupt_or_non_contiguous_restore_fails_closed() {
        let mut store = PrivateStore::memory(Domain::CredentialProof);
        store.append(&format!("{:032x}", 2), br#"{"version":1,"sequence":2,"previous":"","fingerprint":null,"state":"unverified"}"#).unwrap();
        assert!(CredentialValidation::restore(store).is_err());
    }
    #[test]
    fn uncertain_revocation_never_keeps_a_live_valid_state() {
        for after_write in [false, true] {
            let mut proof = empty();
            proof.record_result(KEY, true).unwrap();
            proof.store.fail_next_append_for_test(after_write);
            assert!(proof.revoke().is_err());
            assert_eq!(proof.state(Some(KEY)), ValidationState::Unavailable);
            assert!(proof.require(KEY).is_err());
            // A committed tombstone is honoured after reopening; an append
            // that never happened is not falsely reported as durable deletion.
            if after_write {
                assert_eq!(
                    CredentialValidation::restore(proof.store)
                        .unwrap()
                        .state(Some(KEY)),
                    ValidationState::Unverified
                );
            }
        }
    }
    #[test]
    fn verification_reserves_room_for_revocation_result_and_deletion() {
        let mut proof = empty();
        for _ in 0..126 {
            proof.revoke().unwrap();
        }
        assert!(!proof.can_verify().unwrap());
        // The fixed record ceiling has not been raised; local deletion still
        // has room for a tombstone without a new validation network request.
        proof.revoke().unwrap();
        assert_eq!(proof.state(None), ValidationState::Missing);
        assert!(proof.require(KEY).is_err());
    }
    #[test]
    fn persisted_renderer_authority_or_invalid_state_shape_is_rejected() {
        for bytes in [
            br#"{"version":1,"sequence":1,"previous":"","fingerprint":null,"state":"valid"}"#.as_slice(),
            br#"{"version":1,"sequence":1,"previous":"","fingerprint":null,"state":"unavailable"}"#,
            br#"{"version":1,"sequence":1,"previous":"","fingerprint":null,"state":"unverified","confirmed":true}"#,
        ] {
            let mut store = PrivateStore::memory(Domain::CredentialProof);
            store.append(&format!("{:032x}", 1), bytes).unwrap();
            assert!(CredentialValidation::restore(store).is_err());
        }
    }
}
