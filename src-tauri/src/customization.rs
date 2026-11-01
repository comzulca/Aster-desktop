//! Bounded, inert skill/plugin catalogue (SEC-031 / T-088, T-089).
//!
//! Package text is lower-trust data. No executable path, hook, dependency,
//! endpoint, credential, MCP server or permission is represented. Enabling a
//! catalogue item is only a persisted preference, never host tool authority.

use crate::private_store::{new_id, Domain, PrivateStore, StoreError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::Path};
use zeroize::Zeroizing;

pub const MAX_IMPORT_BYTES: usize = 64 * 1024;
pub const MAX_PACKAGES: usize = 64;
const MAX_REVISIONS: usize = 512;
const SCHEMA_VERSION: u32 = 1;
pub const BUILTIN_GITHUB_ID: &str = "builtin:github";
pub const BUILTIN_ASTER_BUILDER_ID: &str = "skill:aster-builder";
const ASTER_BUILDER_INSTRUCTIONS: &str = include_str!("../resources/aster-builder/SKILL.md");

#[derive(Debug, Copy, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PackageKind {
    Skill,
    Plugin,
}

impl PackageKind {
    fn prefix(self) -> &'static str {
        match self {
            Self::Skill => "skill",
            Self::Plugin => "plugin",
        }
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageDraft {
    pub kind: PackageKind,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub instructions: String,
}

fn bounded_text(text: &str, scalars: usize, bytes: usize, multiline: bool) -> bool {
    text.len() <= bytes
        && text.chars().count() <= scalars
        && !text.chars().any(|character| {
            character.is_control() && !(multiline && matches!(character, '\r' | '\n' | '\t'))
        })
}

impl PackageDraft {
    pub fn validate(&self) -> Result<(), StoreError> {
        if !(3..=64).contains(&self.slug.len())
            || !self.slug.as_bytes()[0].is_ascii_lowercase()
            || self.slug.ends_with('-')
            || self.slug.contains("--")
            || !self
                .slug
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        {
            return Err(StoreError::Denied);
        }
        if self.name.trim().is_empty()
            || self.instructions.trim().is_empty()
            || !bounded_text(&self.name, 80, 320, false)
            || !bounded_text(&self.description, 320, 1_280, false)
            || !bounded_text(&self.instructions, 12_000, 48_000, true)
        {
            return Err(StoreError::Capacity);
        }
        Ok(())
    }

    pub fn id(&self) -> String {
        format!("{}:{}", self.kind.prefix(), self.slug)
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Availability {
    InstructionsOnly,
    BuiltInConnector,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogueEntryMetadata {
    pub id: String,
    pub kind: PackageKind,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub read_only: bool,
    pub availability: Availability,
    pub entry_revision: u64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogueSnapshot {
    pub revision: u64,
    pub entries: Vec<CatalogueEntryMetadata>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageView {
    pub id: String,
    pub kind: PackageKind,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub instructions: String,
    pub enabled: bool,
    pub entry_revision: u64,
    pub read_only: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageIdInput {
    pub id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SavePackageInput {
    pub expected_revision: u64,
    pub package: PackageDraft,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SetPackageEnabledInput {
    pub expected_revision: u64,
    pub id: String,
    pub enabled: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PackageEnvelope {
    schema_version: u32,
    package: PackageDraft,
}

/// Strict import/generation seam. Callers must pass a native-selected bounded
/// file or a separately admitted provider draft, never arbitrary JSON fields.
pub fn parse_package_json(bytes: &[u8]) -> Result<PackageDraft, StoreError> {
    if bytes.is_empty() || bytes.len() > MAX_IMPORT_BYTES {
        return Err(StoreError::Capacity);
    }
    let envelope: PackageEnvelope =
        serde_json::from_slice(bytes).map_err(|_| StoreError::Corrupt)?;
    if envelope.schema_version != SCHEMA_VERSION {
        return Err(StoreError::Denied);
    }
    envelope.package.validate()?;
    Ok(envelope.package)
}

/// Test-only closed inert resource inventory; there is no resource-read IPC.
/// Production packaging lists these files explicitly in tauri.conf.json.
#[cfg(test)]
pub fn aster_builder_resource(name: &str) -> Result<&'static str, StoreError> {
    match name {
        "references/architecture.md" => Ok(include_str!(
            "../resources/aster-builder/references/architecture.md"
        )),
        "references/spec-threats.md" => Ok(include_str!(
            "../resources/aster-builder/references/spec-threats.md"
        )),
        "references/provider-contracts.md" => Ok(include_str!(
            "../resources/aster-builder/references/provider-contracts.md"
        )),
        "references/storage-tools.md" => Ok(include_str!(
            "../resources/aster-builder/references/storage-tools.md"
        )),
        "references/ui.md" => Ok(include_str!("../resources/aster-builder/references/ui.md")),
        "references/verification.md" => Ok(include_str!(
            "../resources/aster-builder/references/verification.md"
        )),
        "references/aster-case-study.md" => Ok(include_str!(
            "../resources/aster-builder/references/aster-case-study.md"
        )),
        "references/ledger-schema.md" => Ok(include_str!(
            "../resources/aster-builder/references/ledger-schema.md"
        )),
        "assets/icon.svg" => Ok(include_str!("../resources/aster-builder/assets/icon.svg")),
        _ => Err(StoreError::Denied),
    }
}

#[derive(Clone)]
struct Entry {
    package: PackageDraft,
    enabled: bool,
    revision: u64,
    read_only: bool,
}

#[derive(Clone)]
struct Catalogue {
    packages: BTreeMap<String, Entry>,
    github_enabled: bool,
    github_revision: u64,
}

impl Default for Catalogue {
    fn default() -> Self {
        let mut packages = BTreeMap::new();
        packages.insert(BUILTIN_ASTER_BUILDER_ID.into(), Entry {
            package: PackageDraft {
                kind: PackageKind::Skill,
                slug: "aster-builder".into(),
                name: "Aster Builder".into(),
                description: "Owner-authorised bundled inert engineering instructions. Scripts, installation hooks and the old UI image are omitted.".into(),
                instructions: ASTER_BUILDER_INSTRUCTIONS.into(),
            },
            enabled: false,
            revision: 0,
            read_only: true,
        });
        Self {
            packages,
            github_enabled: false,
            github_revision: 0,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", tag = "type", deny_unknown_fields)]
enum Event {
    Save { package: PackageDraft },
    SetEnabled { id: String, enabled: bool },
}

impl Catalogue {
    fn apply(&mut self, event: &Event, revision: u64) -> Result<(), StoreError> {
        match event {
            Event::Save { package } => {
                package.validate()?;
                let id = package.id();
                if self.packages.get(&id).is_some_and(|entry| entry.read_only) {
                    return Err(StoreError::Denied);
                }
                if !self.packages.contains_key(&id) && self.packages.len() > MAX_PACKAGES {
                    return Err(StoreError::Capacity);
                }
                // Editing invalidates the previous enabled preference. The
                // changed full instructions require a fresh local selection.
                self.packages.insert(
                    id,
                    Entry {
                        package: package.clone(),
                        enabled: false,
                        revision,
                        read_only: false,
                    },
                );
            }
            Event::SetEnabled { id, enabled } => {
                if id == BUILTIN_GITHUB_ID {
                    self.github_enabled = *enabled;
                    self.github_revision = revision;
                } else {
                    let entry = self.packages.get_mut(id).ok_or(StoreError::Denied)?;
                    entry.enabled = *enabled;
                    entry.revision = revision;
                }
            }
        }
        Ok(())
    }
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

pub struct Customization {
    store: PrivateStore,
    revision: u64,
    head: String,
    catalogue: Catalogue,
    fault: bool,
}

impl Customization {
    pub fn open(base: &Path) -> Result<Self, StoreError> {
        Self::restore(PrivateStore::open(base, Domain::Customization)?)
    }

    fn replay(store: &PrivateStore) -> Result<(u64, String, Catalogue), StoreError> {
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
        let mut catalogue = Catalogue::default();
        for (_, (record, bytes)) in ordered {
            if record.revision != revision + 1 || record.previous != head {
                return Err(StoreError::Corrupt);
            }
            catalogue
                .apply(&record.event, record.revision)
                .map_err(|_| StoreError::Corrupt)?;
            revision = record.revision;
            head = digest(&bytes);
        }
        Ok((revision, head, catalogue))
    }

    fn restore(store: PrivateStore) -> Result<Self, StoreError> {
        let (revision, head, catalogue) = Self::replay(&store)?;
        Ok(Self {
            store,
            revision,
            head,
            catalogue,
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

    pub fn snapshot(&self) -> Result<CatalogueSnapshot, StoreError> {
        self.ensure_current()?;
        let mut entries = vec![CatalogueEntryMetadata {
            id: BUILTIN_GITHUB_ID.into(),
            kind: PackageKind::Plugin,
            name: "GitHub".into(),
            description: "Built-in reviewed GitHub connector. Enabling is not account authorisation or mutation consent.".into(),
            enabled: self.catalogue.github_enabled,
            read_only: true,
            availability: Availability::BuiltInConnector,
            entry_revision: self.catalogue.github_revision,
        }];
        entries.extend(
            self.catalogue
                .packages
                .iter()
                .map(|(id, entry)| CatalogueEntryMetadata {
                    id: id.clone(),
                    kind: entry.package.kind,
                    name: entry.package.name.clone(),
                    description: entry.package.description.clone(),
                    enabled: entry.enabled,
                    read_only: entry.read_only,
                    availability: Availability::InstructionsOnly,
                    entry_revision: entry.revision,
                }),
        );
        Ok(CatalogueSnapshot {
            revision: self.revision,
            entries,
        })
    }

    pub fn package(&self, id: &str) -> Result<PackageView, StoreError> {
        self.ensure_current()?;
        if id.len() > 72 {
            return Err(StoreError::Denied);
        }
        if id == BUILTIN_GITHUB_ID {
            return Ok(PackageView {
                id: id.into(), kind: PackageKind::Plugin, slug: "github".into(),
                name: "GitHub".into(),
                description: "Built-in host connector; package instructions cannot alter its routes or permissions.".into(),
                instructions: String::new(), enabled: self.catalogue.github_enabled,
                entry_revision: self.catalogue.github_revision, read_only: true,
            });
        }
        let entry = self.catalogue.packages.get(id).ok_or(StoreError::Denied)?;
        Ok(PackageView {
            id: id.into(),
            kind: entry.package.kind,
            slug: entry.package.slug.clone(),
            name: entry.package.name.clone(),
            description: entry.package.description.clone(),
            instructions: entry.package.instructions.clone(),
            enabled: entry.enabled,
            entry_revision: entry.revision,
            read_only: entry.read_only,
        })
    }

    fn commit(
        &mut self,
        expected_revision: u64,
        event: Event,
    ) -> Result<CatalogueSnapshot, StoreError> {
        self.ensure_current()?;
        if expected_revision != self.revision {
            return Err(StoreError::Conflict);
        }
        if self.revision as usize >= MAX_REVISIONS {
            return Err(StoreError::Capacity);
        }
        let mut candidate = self.catalogue.clone();
        candidate.apply(&event, self.revision + 1)?;
        let record = Record {
            schema_version: SCHEMA_VERSION,
            revision: self.revision + 1,
            previous: self.head.clone(),
            event,
        };
        let bytes = Zeroizing::new(serde_json::to_vec(&record).map_err(|_| StoreError::Corrupt)?);
        if !self
            .store
            .has_capacity(Domain::Customization, bytes.len(), 1)?
        {
            return Err(StoreError::Capacity);
        }
        if let Err(error) = self.store.append(&new_id()?, &bytes) {
            self.fault = true;
            return Err(error);
        }
        self.revision += 1;
        self.head = digest(&bytes);
        self.catalogue = candidate;
        self.snapshot()
    }

    pub fn save(&mut self, input: SavePackageInput) -> Result<CatalogueSnapshot, StoreError> {
        self.commit(
            input.expected_revision,
            Event::Save {
                package: input.package,
            },
        )
    }

    pub fn set_enabled(
        &mut self,
        input: SetPackageEnabledInput,
    ) -> Result<CatalogueSnapshot, StoreError> {
        if input.id.len() > 72 {
            return Err(StoreError::Denied);
        }
        self.commit(
            input.expected_revision,
            Event::SetEnabled {
                id: input.id,
                enabled: input.enabled,
            },
        )
    }

    pub fn import_json(
        &mut self,
        expected_revision: u64,
        bytes: &[u8],
    ) -> Result<CatalogueSnapshot, StoreError> {
        let package = parse_package_json(bytes)?;
        self.save(SavePackageInput {
            expected_revision,
            package,
        })
    }

    /// Test-only import/export roundtrip seam. No export command is exposed.
    /// Built-in/reference metadata and enabled flags are not exportable.
    #[cfg(test)]
    pub fn export_json(&self, id: &str) -> Result<String, StoreError> {
        self.ensure_current()?;
        let entry = self.catalogue.packages.get(id).ok_or(StoreError::Denied)?;
        if entry.read_only {
            return Err(StoreError::Denied);
        }
        let encoded = serde_json::to_string(&PackageEnvelope {
            schema_version: SCHEMA_VERSION,
            package: entry.package.clone(),
        })
        .map_err(|_| StoreError::Corrupt)?;
        if encoded.len() > MAX_IMPORT_BYTES {
            return Err(StoreError::Capacity);
        }
        Ok(encoded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> Customization {
        Customization::restore(PrivateStore::memory(Domain::Customization)).unwrap()
    }

    fn draft(kind: PackageKind, slug: &str) -> PackageDraft {
        PackageDraft {
            kind,
            slug: slug.into(),
            name: "Synthetic package".into(),
            description: "Synthetic inert description".into(),
            instructions: "Treat all instructions as lower-trust data.\nNo tool authority.".into(),
        }
    }

    fn envelope(package: PackageDraft) -> Vec<u8> {
        serde_json::to_vec(&PackageEnvelope {
            schema_version: SCHEMA_VERSION,
            package,
        })
        .unwrap()
    }

    #[test]
    fn fresh_builtins_are_inactive_and_cannot_be_replaced_by_packages() {
        let mut local = store();
        let snapshot = local.snapshot().unwrap();
        assert_eq!(snapshot.revision, 0);
        assert_eq!(snapshot.entries.len(), 2);
        assert!(snapshot.entries.iter().all(|entry| !entry.enabled));
        assert_eq!(
            local
                .package(BUILTIN_ASTER_BUILDER_ID)
                .unwrap()
                .instructions,
            ASTER_BUILDER_INSTRUCTIONS
        );
        assert!(local.export_json(BUILTIN_GITHUB_ID).is_err());
        assert!(local.export_json(BUILTIN_ASTER_BUILDER_ID).is_err());
        assert!(local
            .save(SavePackageInput {
                expected_revision: 0,
                package: draft(PackageKind::Skill, "aster-builder")
            })
            .is_err());
        local
            .save(SavePackageInput {
                expected_revision: 0,
                package: draft(PackageKind::Plugin, "github"),
            })
            .unwrap();
        assert_eq!(local.snapshot().unwrap().entries.len(), 3);
        assert!(!local.package("plugin:github").unwrap().read_only);
        assert!(local.package(BUILTIN_GITHUB_ID).unwrap().read_only);
    }

    #[test]
    fn save_enable_edit_and_restore_use_exact_monotonic_revisions() {
        let mut local = store();
        local
            .save(SavePackageInput {
                expected_revision: 0,
                package: draft(PackageKind::Skill, "example"),
            })
            .unwrap();
        local
            .set_enabled(SetPackageEnabledInput {
                expected_revision: 1,
                id: "skill:example".into(),
                enabled: true,
            })
            .unwrap();
        assert!(local.package("skill:example").unwrap().enabled);
        assert!(local
            .save(SavePackageInput {
                expected_revision: 1,
                package: draft(PackageKind::Skill, "example")
            })
            .is_err());
        let mut edited = draft(PackageKind::Skill, "example");
        edited.instructions.push_str("\nChanged revision.");
        local
            .save(SavePackageInput {
                expected_revision: 2,
                package: edited.clone(),
            })
            .unwrap();
        assert!(!local.package("skill:example").unwrap().enabled);
        assert_eq!(local.package("skill:example").unwrap().entry_revision, 3);
        let restored = Customization::restore(local.store).unwrap();
        assert_eq!(restored.snapshot().unwrap().revision, 3);
        assert_eq!(
            restored.package("skill:example").unwrap().instructions,
            edited.instructions
        );
    }

    #[test]
    fn inert_import_export_excludes_enabled_state_and_bundled_script_paths() {
        let mut local = store();
        local
            .import_json(0, &envelope(draft(PackageKind::Skill, "example")))
            .unwrap();
        assert!(!local.package("skill:example").unwrap().enabled);
        let export = local.export_json("skill:example").unwrap();
        assert!(parse_package_json(export.as_bytes()).is_ok());
        assert!(!export.contains("enabled"));
        assert!(!export.contains("account"));
        for name in [
            "../SKILL.md",
            "scripts/harness_gate.py",
            "agents/openai.yaml",
            "assets/default-ui.jpg",
            "C:\\evil",
            "https://evil.example",
        ] {
            assert!(aster_builder_resource(name).is_err());
        }
        assert!(aster_builder_resource("references/architecture.md")
            .unwrap()
            .starts_with("# Authority"));
    }

    #[test]
    fn malformed_json_unknown_fields_active_capabilities_duplicates_and_bombs_are_rejected() {
        for input in [
            br#"{"schemaVersion":1,"package":{"kind":"skill","slug":"example","name":"Example","description":"","instructions":"x","hooks":["evil"]}}"#.as_slice(),
            br#"{"schemaVersion":1,"package":{"kind":"plugin","slug":"example","name":"Example","description":"","instructions":"x","endpoint":"https://evil.example"}}"#,
            br#"{"schemaVersion":1,"package":{"kind":"skill","slug":"example","name":"Example","description":"","instructions":"x","enabled":true}}"#,
            br#"{"schemaVersion":1,"package":{"kind":"skill","slug":"example","name":"Example","description":"","instructions":"x"},"grant":true}"#,
            br#"{"schemaVersion":1,"schemaVersion":1,"package":{"kind":"skill","slug":"example","name":"Example","description":"","instructions":"x"}}"#,
            br#"{"schemaVersion":2,"package":{"kind":"skill","slug":"example","name":"Example","description":"","instructions":"x"}}"#,
            b"PK\x03\x04zip-not-json",
            b"{",
            b"\xff",
        ] {
            assert!(parse_package_json(input).is_err());
        }
        assert_eq!(
            parse_package_json(&vec![b' '; MAX_IMPORT_BYTES + 1]).err(),
            Some(StoreError::Capacity)
        );
    }

    #[test]
    fn path_aliases_control_bytes_and_unicode_overflow_do_not_form_package_identity() {
        for slug in [
            "../x",
            "x:y",
            "C-drive",
            "nul",
            "x/evil",
            "abc.",
            "abc ",
            "abc--def",
            "abc-",
            "a",
            "🟦xyz",
            "github.com",
            "\\server",
            "1example",
        ] {
            let candidate = draft(PackageKind::Skill, slug);
            // "nul" is an inert identifier, not a filesystem path. It is
            // intentionally admissible because no extraction path exists.
            assert_eq!(candidate.validate().is_ok(), slug == "nul");
        }
        let mut candidate = draft(PackageKind::Skill, "example");
        candidate.instructions = "🟦".repeat(12_000);
        assert_eq!(candidate.validate(), Ok(()));
        candidate.instructions.push('x');
        assert_eq!(candidate.validate(), Err(StoreError::Capacity));
        candidate.instructions = "x\0y".into();
        assert_eq!(candidate.validate(), Err(StoreError::Capacity));
    }

    #[test]
    fn built_in_enabled_preference_restores_no_account_or_mutation_grant() {
        let mut local = store();
        local
            .set_enabled(SetPackageEnabledInput {
                expected_revision: 0,
                id: BUILTIN_GITHUB_ID.into(),
                enabled: true,
            })
            .unwrap();
        let restored = Customization::restore(local.store).unwrap();
        let metadata = serde_json::to_string(&restored.snapshot().unwrap()).unwrap();
        assert!(restored.package(BUILTIN_GITHUB_ID).unwrap().enabled);
        for forbidden in [
            "accessToken",
            "refreshToken",
            "deviceCode",
            "confirmation",
            "accountGeneration",
            "repositoryId",
            "grant",
        ] {
            assert!(!metadata.contains(forbidden));
        }
    }

    #[test]
    fn duplicate_gap_unknown_event_and_invalid_catalogue_restore_are_corrupt() {
        for payload in [
            br#"{"schemaVersion":1,"revision":2,"previous":"","event":{"type":"setEnabled","id":"builtin:github","enabled":true}}"#.as_slice(),
            br#"{"schemaVersion":1,"revision":1,"previous":"","event":{"type":"setEnabled","id":"skill:missing","enabled":true}}"#,
            br#"{"schemaVersion":1,"revision":1,"previous":"","event":{"type":"runScript","command":"evil"}}"#,
            br#"{"schemaVersion":1,"revision":1,"previous":"","event":{"type":"setEnabled","id":"builtin:github","enabled":true,"token":"evil"}}"#,
        ] {
            let mut local = store();
            local.store.append(&new_id().unwrap(), payload).unwrap();
            assert!(Customization::restore(local.store).is_err());
        }
        let mut local = store();
        let payload = serde_json::to_vec(&Record {
            schema_version: 1,
            revision: 1,
            previous: String::new(),
            event: Event::Save {
                package: draft(PackageKind::Skill, "example"),
            },
        })
        .unwrap();
        local.store.append(&new_id().unwrap(), &payload).unwrap();
        local.store.append(&new_id().unwrap(), &payload).unwrap();
        assert!(Customization::restore(local.store).is_err());
    }

    #[test]
    fn failed_append_is_not_retried_and_durable_replay_decides_result() {
        for after_write in [false, true] {
            let mut local = store();
            local.store.fail_next_append_for_test(after_write);
            assert_eq!(
                local
                    .save(SavePackageInput {
                        expected_revision: 0,
                        package: draft(PackageKind::Skill, "example")
                    })
                    .err(),
                Some(StoreError::Io)
            );
            assert!(local.snapshot().is_err());
            assert!(local
                .save(SavePackageInput {
                    expected_revision: 0,
                    package: draft(PackageKind::Skill, "example")
                })
                .is_err());
            let restored = Customization::restore(local.store).unwrap();
            assert_eq!(
                restored.snapshot().unwrap().revision,
                u64::from(after_write)
            );
            assert_eq!(restored.package("skill:example").is_ok(), after_write);
        }
    }

    #[test]
    fn bounded_catalogue_capacity_does_not_silently_drop_previous_entries() {
        let mut local = store();
        for n in 0..MAX_PACKAGES {
            local
                .save(SavePackageInput {
                    expected_revision: n as u64,
                    package: draft(PackageKind::Skill, &format!("example-{n}")),
                })
                .unwrap();
        }
        assert_eq!(
            local
                .save(SavePackageInput {
                    expected_revision: MAX_PACKAGES as u64,
                    package: draft(PackageKind::Skill, "one-too-many")
                })
                .err(),
            Some(StoreError::Capacity)
        );
        assert_eq!(local.snapshot().unwrap().entries.len(), MAX_PACKAGES + 2);
    }

    #[test]
    fn catalogue_module_cannot_construct_provider_or_process_payloads() {
        // Domain-level seam only, not evidence for all application egress.
        let source = include_str!("customization.rs");
        let production = source.split("\nmod tests {").next().unwrap();
        for forbidden in [
            "crate::personalization",
            "crate::provider",
            "reqwest::",
            "std::process",
            "Command::",
            "pub fn commit_tool",
        ] {
            assert!(!production.contains(forbidden));
        }
        const { assert!(MAX_REVISIONS < 9_007_199_254_740_991usize) };
    }
}
