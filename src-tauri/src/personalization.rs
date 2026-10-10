//! Local-only profile custody (SEC-035 / T-093).
//!
//! This module deliberately has no provider, conversation, GitHub, export or
//! telemetry dependency. Its serialised DTOs are for the authenticated local
//! editor and its fixed encrypted store only. Clearing writes a tombstone:
//! prior immutable ciphertext and external backups are not forensic erasure.

use crate::private_store::{new_id, Domain, PrivateStore, StoreError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::Path};
use zeroize::{Zeroize, Zeroizing};

const SCHEMA_VERSION: u32 = 1;
const MAX_REVISIONS: usize = 512;
const CLEAR_RESERVE_BYTES: usize = 1_024;

/// Sensitive local editor fields. Do not derive Debug or attach this type to
/// an outbound request. Unicode bounds count scalar values, not UTF-16 units.
#[derive(Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Profile {
    pub custom_instructions: String,
    pub nickname: String,
    pub occupation: String,
    pub more_about_you: String,
}

impl Drop for Profile {
    fn drop(&mut self) {
        self.custom_instructions.zeroize();
        self.nickname.zeroize();
        self.occupation.zeroize();
        self.more_about_you.zeroize();
    }
}

impl Profile {
    pub fn validate(&self) -> Result<(), StoreError> {
        let fields = [
            (&self.custom_instructions, 2_000, 8_000, true),
            (&self.nickname, 80, 320, false),
            (&self.occupation, 120, 480, false),
            (&self.more_about_you, 1_000, 4_000, true),
        ];
        let mut total_scalars = 0usize;
        let mut total_bytes = 0usize;
        for (text, maximum_scalars, maximum_bytes, multiline) in fields {
            let scalars = text.chars().count();
            if scalars > maximum_scalars || text.len() > maximum_bytes {
                return Err(StoreError::Capacity);
            }
            if text.chars().any(|character| {
                character.is_control() && !(multiline && matches!(character, '\n' | '\r' | '\t'))
            }) {
                return Err(StoreError::Denied);
            }
            total_scalars = total_scalars
                .checked_add(scalars)
                .ok_or(StoreError::Capacity)?;
            total_bytes = total_bytes
                .checked_add(text.len())
                .ok_or(StoreError::Capacity)?;
        }
        if total_scalars > 3_200 || total_bytes > 12_800 {
            return Err(StoreError::Capacity);
        }
        Ok(())
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonalizationSnapshot {
    pub revision: u64,
    pub profile: Profile,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SavePersonalizationInput {
    pub expected_revision: u64,
    pub profile: Profile,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExpectedRevisionInput {
    pub expected_revision: u64,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", tag = "type", deny_unknown_fields)]
enum Event {
    Save { profile: Profile },
    Clear,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Record {
    schema_version: u32,
    revision: u64,
    previous: String,
    event: Event,
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub struct Personalization {
    store: PrivateStore,
    revision: u64,
    head: String,
    profile: Profile,
    fault: bool,
}

impl Personalization {
    pub fn open(base: &Path) -> Result<Self, StoreError> {
        Self::restore(PrivateStore::open(base, Domain::Personalization)?)
    }

    fn replay(store: &PrivateStore) -> Result<(u64, String, Profile), StoreError> {
        let records = store.records()?;
        if records.len() > MAX_REVISIONS {
            return Err(StoreError::Capacity);
        }
        let mut ordered = BTreeMap::new();
        for (_, bytes) in records {
            let bytes = Zeroizing::new(bytes);
            let record: Record = serde_json::from_slice(&bytes).map_err(|_| StoreError::Corrupt)?;
            if record.schema_version != SCHEMA_VERSION
                || record.revision == 0
                || record.revision > MAX_REVISIONS as u64
                || ordered.insert(record.revision, (record, bytes)).is_some()
            {
                return Err(StoreError::Corrupt);
            }
        }
        let mut revision = 0;
        let mut head = String::new();
        let mut profile = Profile::default();
        for (_, (record, bytes)) in ordered {
            if record.revision != revision + 1 || record.previous != head {
                return Err(StoreError::Corrupt);
            }
            profile = match record.event {
                Event::Save { profile } => {
                    profile.validate().map_err(|_| StoreError::Corrupt)?;
                    profile
                }
                Event::Clear => Profile::default(),
            };
            revision = record.revision;
            head = digest(&bytes);
        }
        Ok((revision, head, profile))
    }

    fn restore(store: PrivateStore) -> Result<Self, StoreError> {
        let (revision, head, profile) = Self::replay(&store)?;
        Ok(Self {
            store,
            revision,
            head,
            profile,
            fault: false,
        })
    }

    fn ensure_current(&self) -> Result<(), StoreError> {
        if self.fault {
            return Err(StoreError::Corrupt);
        }
        let (revision, head, _) = Self::replay(&self.store)?;
        if revision != self.revision || head != self.head {
            return Err(StoreError::Corrupt);
        }
        Ok(())
    }

    pub fn snapshot(&self) -> Result<PersonalizationSnapshot, StoreError> {
        self.ensure_current()?;
        Ok(PersonalizationSnapshot {
            revision: self.revision,
            profile: self.profile.clone(),
        })
    }

    fn commit(
        &mut self,
        expected_revision: u64,
        event: Event,
    ) -> Result<PersonalizationSnapshot, StoreError> {
        self.ensure_current()?;
        if expected_revision != self.revision {
            return Err(StoreError::Conflict);
        }
        let saving = matches!(&event, Event::Save { .. });
        if self.revision as usize >= MAX_REVISIONS
            || (saving && self.revision as usize >= MAX_REVISIONS - 1)
        {
            return Err(StoreError::Capacity);
        }
        let candidate = match &event {
            Event::Save { profile } => {
                profile.validate()?;
                profile.clone()
            }
            Event::Clear => Profile::default(),
        };
        let record = Record {
            schema_version: SCHEMA_VERSION,
            revision: self.revision + 1,
            previous: self.head.clone(),
            event,
        };
        let bytes = Zeroizing::new(serde_json::to_vec(&record).map_err(|_| StoreError::Corrupt)?);
        if !self.store.has_capacity(
            Domain::Personalization,
            bytes.len() + if saving { CLEAR_RESERVE_BYTES } else { 0 },
            if saving { 2 } else { 1 },
        )? {
            return Err(StoreError::Capacity);
        }
        if let Err(error) = self.store.append(&new_id()?, &bytes) {
            // A write may be durable despite an error. No retry/mutation is
            // accepted in this object; a fresh replay decides actual state.
            self.fault = true;
            return Err(error);
        }
        self.revision += 1;
        self.head = digest(&bytes);
        self.profile = candidate;
        self.snapshot()
    }

    pub fn save(
        &mut self,
        input: SavePersonalizationInput,
    ) -> Result<PersonalizationSnapshot, StoreError> {
        self.commit(
            input.expected_revision,
            Event::Save {
                profile: input.profile,
            },
        )
    }

    pub fn clear(
        &mut self,
        input: ExpectedRevisionInput,
    ) -> Result<PersonalizationSnapshot, StoreError> {
        self.commit(input.expected_revision, Event::Clear)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> Personalization {
        Personalization::restore(PrivateStore::memory(Domain::Personalization)).unwrap()
    }

    fn profile() -> Profile {
        Profile {
            custom_instructions: "LOCAL-ONLY-CANARY preferences".into(),
            nickname: "Synthetic nickname".into(),
            occupation: "Synthetic occupation".into(),
            more_about_you: "Synthetic profile\nSecond line".into(),
        }
    }

    #[test]
    fn fresh_save_clear_and_restore_preserve_only_active_local_profile() {
        let mut local = store();
        assert_eq!(local.snapshot().unwrap().revision, 0);
        let saved = local
            .save(SavePersonalizationInput {
                expected_revision: 0,
                profile: profile(),
            })
            .unwrap();
        assert_eq!(saved.revision, 1);
        assert!(saved.profile == profile());
        let mut local = Personalization::restore(local.store).unwrap();
        assert!(local.snapshot().unwrap().profile == profile());
        local
            .clear(ExpectedRevisionInput {
                expected_revision: 1,
            })
            .unwrap();
        assert_eq!(local.store.records().unwrap().len(), 2);
        let restored = Personalization::restore(local.store).unwrap();
        assert_eq!(restored.snapshot().unwrap().revision, 2);
        assert!(restored.snapshot().unwrap().profile == Profile::default());
    }

    #[test]
    fn exact_scalar_and_utf8_limits_count_supplementary_characters_once() {
        let maximum = Profile {
            custom_instructions: "🟦".repeat(2_000),
            nickname: "🟦".repeat(80),
            occupation: "🟦".repeat(120),
            more_about_you: "🟦".repeat(1_000),
        };
        assert_eq!(maximum.validate(), Ok(()));
        let mut excessive = maximum.clone();
        excessive.nickname.push('x');
        assert_eq!(excessive.validate(), Err(StoreError::Capacity));
        let mut excessive = maximum.clone();
        excessive.custom_instructions.push('x');
        assert_eq!(excessive.validate(), Err(StoreError::Capacity));
        let mut excessive = maximum.clone();
        excessive.occupation.push('x');
        assert_eq!(excessive.validate(), Err(StoreError::Capacity));
        let mut excessive = maximum;
        excessive.more_about_you.push('x');
        assert_eq!(excessive.validate(), Err(StoreError::Capacity));
    }

    #[test]
    fn control_fields_unknown_capabilities_and_fractional_revisions_are_rejected() {
        let mut invalid = profile();
        invalid.nickname.push('\n');
        assert_eq!(invalid.validate(), Err(StoreError::Denied));
        invalid.nickname = "x\0y".into();
        assert_eq!(invalid.validate(), Err(StoreError::Denied));
        assert!(serde_json::from_str::<SavePersonalizationInput>(
            r#"{"expectedRevision":0,"profile":{"customInstructions":"","nickname":"","occupation":"","moreAboutYou":"","sendToMeta":true}}"#
        ).is_err());
        assert!(
            serde_json::from_str::<ExpectedRevisionInput>(r#"{"expectedRevision":0.5}"#).is_err()
        );
        assert!(serde_json::from_str::<ExpectedRevisionInput>(
            r#"{"expectedRevision":0,"grant":true}"#
        )
        .is_err());
    }

    #[test]
    fn stale_revision_cannot_clear_or_overwrite_newer_profile() {
        let mut local = store();
        local
            .save(SavePersonalizationInput {
                expected_revision: 0,
                profile: profile(),
            })
            .unwrap();
        assert_eq!(
            local
                .clear(ExpectedRevisionInput {
                    expected_revision: 0
                })
                .err(),
            Some(StoreError::Conflict)
        );
        assert_eq!(
            local
                .save(SavePersonalizationInput {
                    expected_revision: 0,
                    profile: Profile::default()
                })
                .err(),
            Some(StoreError::Conflict)
        );
        assert_eq!(local.snapshot().unwrap().revision, 1);
    }

    #[test]
    fn malformed_gapped_duplicate_and_oversized_restores_fail_closed() {
        for payload in [
            b"{\"schemaVersion\":1".as_slice(),
            br#"{"schemaVersion":2,"revision":1,"previous":"","event":{"type":"clear"}}"#,
            br#"{"schemaVersion":1,"revision":2,"previous":"","event":{"type":"clear"}}"#,
            br#"{"schemaVersion":1,"revision":1,"previous":"forged","event":{"type":"clear"}}"#,
            br#"{"schemaVersion":1,"revision":1,"previous":"","event":{"type":"clear"},"restoreGrant":true}"#,
        ] {
            let mut local = store();
            local.store.append(&new_id().unwrap(), payload).unwrap();
            assert!(Personalization::restore(local.store).is_err());
        }
        let mut local = store();
        let mut oversized = Profile::default();
        oversized.nickname = "x".repeat(81);
        let bytes = serde_json::to_vec(&Record {
            schema_version: 1,
            revision: 1,
            previous: String::new(),
            event: Event::Save { profile: oversized },
        })
        .unwrap();
        local.store.append(&new_id().unwrap(), &bytes).unwrap();
        assert!(Personalization::restore(local.store).is_err());
        let mut local = store();
        let bytes = serde_json::to_vec(&Record {
            schema_version: 1,
            revision: 1,
            previous: String::new(),
            event: Event::Clear,
        })
        .unwrap();
        local.store.append(&new_id().unwrap(), &bytes).unwrap();
        local.store.append(&new_id().unwrap(), &bytes).unwrap();
        assert!(Personalization::restore(local.store).is_err());
    }

    #[test]
    fn append_failure_before_or_after_visibility_latches_without_false_success() {
        for after_write in [false, true] {
            let mut local = store();
            local.store.fail_next_append_for_test(after_write);
            assert_eq!(
                local
                    .save(SavePersonalizationInput {
                        expected_revision: 0,
                        profile: profile()
                    })
                    .err(),
                Some(StoreError::Io)
            );
            assert!(local.snapshot().is_err());
            assert!(local
                .clear(ExpectedRevisionInput {
                    expected_revision: 0
                })
                .is_err());
            let restored = Personalization::restore(local.store).unwrap();
            assert_eq!(
                restored.snapshot().unwrap().revision,
                u64::from(after_write)
            );
        }
    }

    #[test]
    fn module_has_no_outbound_constructor_or_dependency_and_counters_are_bounded() {
        // This is a source seam assertion, not proof of every application
        // egress. Root integration must add its independent canary fixtures.
        let source = include_str!("personalization.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        for forbidden in [
            "crate::provider",
            "crate::github",
            "reqwest::",
            "pub fn export",
            "pub fn prompt",
            "pub fn tool",
        ] {
            assert!(!production.contains(forbidden));
        }
        const { assert!(MAX_REVISIONS < 9_007_199_254_740_991usize) };
        let clear = serde_json::to_vec(&Record {
            schema_version: SCHEMA_VERSION,
            revision: MAX_REVISIONS as u64,
            previous: "f".repeat(64),
            event: Event::Clear,
        })
        .unwrap();
        assert!(clear.len() < CLEAR_RESERVE_BYTES);
    }
}
