//! A separate Markdown vault, not a generic project or provider capability.
//! Only an OS-picked root enters state. All authority is session-local.

#[cfg(windows)]
use super::{join_relative, windows_fs};
use super::{parse_relative_path, validate_text, Workspace, WorkspaceError};
use serde::{Deserialize, Serialize};
#[cfg(windows)]
use std::path::PathBuf;

#[cfg(windows)]
const MAX_SCAN_ENTRIES: usize = 128;
#[cfg(windows)]
const MAX_RESULTS: usize = 3;
const MAX_NOTE_BYTES: u64 = 16 * 1024;
const MAX_NOTE_WORDS: usize = 1_200;

#[derive(Default)]
pub(crate) struct Vault(Workspace);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Status {
    selected: bool,
    label: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Search {
    topic: String,
    query: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct OpenNote {
    relative_path: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CreateTopic {
    topic: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CreateNote {
    topic: String,
    title: String,
    content: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RelativePath {
    relative_path: String,
}

#[derive(Serialize)]
pub(crate) struct SearchResults {
    notes: Vec<RelativePath>,
    limited: bool,
}

#[cfg(windows)]
pub(crate) struct TopicCandidate {
    pub(crate) relative_path: String,
    document: serde_json::Value,
    epoch: u64,
    id: windows_fs::FileIdentity,
}

#[cfg(windows)]
impl TopicCandidate {
    pub(crate) fn review_document(&self) -> Result<crate::native_review::ReviewDocument, String> {
        crate::operation(self.document.clone())
    }
}

pub(crate) fn error(error: WorkspaceError) -> String {
    match error {
        WorkspaceError::Unavailable => "obsidian_unavailable",
        WorkspaceError::Denied => "obsidian_invalid",
        WorkspaceError::Conflict => "obsidian_conflict",
        WorkspaceError::TooLarge => "obsidian_limit",
        WorkspaceError::Io => "obsidian_io_error",
        WorkspaceError::AttentionRequired => "obsidian_verify_note",
    }
    .into()
}

fn component(value: &str) -> Result<String, WorkspaceError> {
    if value.starts_with('.') || value.trim() != value || value.chars().count() > 80 {
        return Err(WorkspaceError::Denied);
    }
    let mut parts = parse_relative_path(value)?;
    if parts.len() != 1 {
        return Err(WorkspaceError::Denied);
    }
    parts.pop().ok_or(WorkspaceError::Denied)
}

fn note_components(value: &str) -> Result<Vec<String>, WorkspaceError> {
    // Canonical IPC uses slash only; paths are never passed into Windows APIs
    // directly, not even after validating the extension.
    let parts = parse_relative_path(value)?;
    if parts.len() != 2 || parts.join("/") != value {
        return Err(WorkspaceError::Denied);
    }
    component(&parts[0])?;
    let (stem, ext) = parts[1].rsplit_once('.').ok_or(WorkspaceError::Denied)?;
    if !ext.eq_ignore_ascii_case("md") {
        return Err(WorkspaceError::Denied);
    }
    component(stem)?;
    Ok(parts)
}

fn note_content(value: &str) -> Result<(), WorkspaceError> {
    validate_text(value, MAX_NOTE_BYTES)?;
    if value.trim().is_empty() {
        return Err(WorkspaceError::Denied);
    }
    if value.split_whitespace().take(MAX_NOTE_WORDS + 1).count() > MAX_NOTE_WORDS {
        return Err(WorkspaceError::TooLarge);
    }
    Ok(())
}

impl Vault {
    pub(crate) fn status(&self) -> Result<Status, String> {
        let state = self.0.state.lock().map_err(|_| "obsidian_unavailable")?;
        Ok(Status {
            selected: state.root.is_some(),
            label: state
                .root
                .as_ref()
                .map(|r| r.label.clone())
                .unwrap_or_default(),
        })
    }

    #[cfg(windows)]
    pub(crate) fn select(&self, path: PathBuf, local_data: PathBuf) -> Result<Status, String> {
        // The trusted picker supplies the only root; do not grant a whole drive
        // or any ancestor/descendant of Aster's private store or journal.
        if path.parent().is_none() {
            return Err("obsidian_invalid".into());
        }
        let root = windows_fs::select_root(path).map_err(error)?;
        component(&root.label).map_err(error)?;
        let data = windows_fs::select_root(local_data).map_err(error)?;
        if windows_fs::root_or_ancestor_contains(&data, root.id).map_err(error)? {
            return Err("obsidian_invalid".into());
        }
        for protected in [
            "com.comzulca.aster-desktop-private-v1",
            super::JOURNAL_CONTAINER,
        ] {
            let target = data.path.join(protected);
            match std::fs::symlink_metadata(&target) {
                Ok(_) => {
                    let blocked = windows_fs::select_root(target).map_err(error)?;
                    if windows_fs::roots_overlap(&root, &blocked).map_err(error)? {
                        return Err("obsidian_invalid".into());
                    }
                }
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                Err(_) => return Err("obsidian_io_error".into()),
            }
        }
        let label = root.label.clone();
        let mut state = self.0.state.lock().map_err(|_| "obsidian_unavailable")?;
        let epoch = state.epoch.wrapping_add(1);
        *state = super::State {
            root: Some(root),
            epoch,
            ..Default::default()
        };
        Ok(Status {
            selected: true,
            label,
        })
    }

    pub(crate) fn clear(&self) -> Result<Status, String> {
        let mut state = self.0.state.lock().map_err(|_| "obsidian_unavailable")?;
        // Dropping releases every root/ancestor handle and pending allocation.
        let epoch = state.epoch.wrapping_add(1);
        *state = super::State {
            epoch,
            ..Default::default()
        };
        Ok(Status {
            selected: false,
            label: String::new(),
        })
    }

    pub(crate) fn search(&self, input: Search) -> Result<SearchResults, String> {
        let topic = component(&input.topic).map_err(error)?;
        validate_text(&input.query, 256).map_err(error)?;
        let query = input.query.trim().to_lowercase();
        #[cfg(windows)]
        {
            let state = self.0.state.lock().map_err(|_| "obsidian_unavailable")?;
            let root = state.root.as_ref().ok_or("obsidian_not_selected")?;
            // By-handle chain is kept pinned until enumeration fully returns.
            let Some(pinned) = windows_fs::pin_vault_topic(root, &topic).map_err(error)? else {
                return Ok(SearchResults {
                    notes: Vec::new(),
                    limited: false,
                });
            };
            let parent = pinned.target_path.parent().ok_or("obsidian_invalid")?;
            let mut names = Vec::new();
            let mut limited = false;
            for (index, file) in std::fs::read_dir(parent)
                .map_err(|_| "obsidian_io_error")?
                .enumerate()
            {
                if index == MAX_SCAN_ENTRIES {
                    limited = true;
                    break;
                }
                let file = file.map_err(|_| "obsidian_io_error")?;
                let Ok(name) = file.file_name().into_string() else {
                    continue;
                };
                let relative_path = format!("{topic}/{name}");
                if note_components(&relative_path).is_err()
                    || !name.to_lowercase().contains(&query)
                    || !windows_fs::is_vault_note(&file.path(), root.id, MAX_NOTE_BYTES)
                {
                    continue;
                }
                names.push(RelativePath { relative_path });
            }
            names.sort_by(|a, b| {
                a.relative_path
                    .to_lowercase()
                    .cmp(&b.relative_path.to_lowercase())
                    .then_with(|| a.relative_path.cmp(&b.relative_path))
            });
            limited |= names.len() > MAX_RESULTS;
            names.truncate(MAX_RESULTS);
            Ok(SearchResults {
                notes: names,
                limited,
            })
        }
        #[cfg(not(windows))]
        {
            let _ = (topic, query);
            Err("obsidian_unavailable".into())
        }
    }

    pub(crate) fn read(
        &self,
        input: OpenNote,
    ) -> Result<(String, zeroize::Zeroizing<String>), String> {
        let parts = note_components(&input.relative_path).map_err(error)?;
        #[cfg(windows)]
        {
            let state = self.0.state.lock().map_err(|_| "obsidian_unavailable")?;
            let root = state.root.as_ref().ok_or("obsidian_not_selected")?;
            let _pinned_topic = windows_fs::pin_vault_topic(root, &parts[0])
                .map_err(error)?
                .ok_or("obsidian_topic_missing")?;
            let (text, _) =
                windows_fs::read_vault_note(root, &parts, MAX_NOTE_BYTES).map_err(error)?;
            let text = zeroize::Zeroizing::new(text);
            note_content(&text).map_err(error)?;
            Ok((join_relative(&parts), text))
        }
        #[cfg(not(windows))]
        {
            let _ = parts;
            Err("obsidian_unavailable".into())
        }
    }

    #[cfg(windows)]
    pub(crate) fn prepare_topic(&self, input: CreateTopic) -> Result<TopicCandidate, String> {
        let topic = component(&input.topic).map_err(error)?;
        let state = self.0.state.lock().map_err(|_| "obsidian_unavailable")?;
        let root = state.root.as_ref().ok_or("obsidian_not_selected")?;
        let parent =
            windows_fs::pin_creation_parent(root, std::slice::from_ref(&topic)).map_err(error)?;
        windows_fs::require_absent(&parent.target_path).map_err(error)?;
        let document = serde_json::json!({
            "operation": "create_obsidian_topic",
            "targetPath": parent.target_path.to_str().ok_or("obsidian_invalid")?,
            "relativePath": topic,
            "scope": "One local folder only. No disclosure or provider request.",
            "nonce": windows_fs::random_nonce().map_err(error)?,
            "rootEpoch": state.epoch,
            "rootIdentity": windows_fs::record_identity(root.id),
        });
        Ok(TopicCandidate {
            relative_path: topic,
            document,
            epoch: state.epoch,
            id: root.id,
        })
    }

    #[cfg(windows)]
    pub(crate) fn create_topic(
        &self,
        candidate: TopicCandidate,
        approval: crate::native_review::NativeApproval,
    ) -> Result<RelativePath, String> {
        approval
            .consume(candidate.review_document()?.binding_sha256())
            .map_err(|_| "obsidian_conflict")?;
        let state = self.0.state.lock().map_err(|_| "obsidian_unavailable")?;
        let root = state.root.as_ref().ok_or("obsidian_not_selected")?;
        if root.id != candidate.id || state.epoch != candidate.epoch {
            return Err("obsidian_conflict".into());
        }
        let pinned =
            windows_fs::pin_creation_parent(root, std::slice::from_ref(&candidate.relative_path))
                .map_err(error)?;
        windows_fs::require_absent(&pinned.target_path).map_err(error)?;
        // create_dir does not replace an existing namespace entry even if
        // another process creates it after require_absent.
        std::fs::create_dir(&pinned.target_path).map_err(|err| {
            if err.kind() == std::io::ErrorKind::AlreadyExists {
                "obsidian_conflict"
            } else {
                "obsidian_io_error"
            }
        })?;
        // Re-open/verify, reporting uncertain external changes without cleanup.
        windows_fs::pin_creation_parent(root, &[candidate.relative_path.clone(), "verify".into()])
            .map_err(|_| "obsidian_verify_note")?;
        Ok(RelativePath {
            relative_path: candidate.relative_path,
        })
    }

    #[cfg(windows)]
    fn pin_topic(&self, topic: &str) -> Result<windows_fs::PinnedParent, String> {
        let state = self.0.state.lock().map_err(|_| "obsidian_unavailable")?;
        let root = state.root.as_ref().ok_or("obsidian_not_selected")?;
        windows_fs::pin_vault_topic(root, topic)
            .map_err(error)?
            .ok_or_else(|| "obsidian_topic_missing".to_owned())
    }

    #[cfg(windows)]
    pub(crate) fn prepare_note(
        &self,
        input: &CreateNote,
        local_data: PathBuf,
    ) -> Result<super::ProposalReview, String> {
        let topic = component(&input.topic).map_err(error)?;
        let title = component(&input.title).map_err(error)?;
        let relative_path = format!("{topic}/{title}.md");
        note_components(&relative_path).map_err(error)?;
        note_content(&input.content).map_err(error)?;
        // Do not hold restrictive topic ancestor pins while provisioning the
        // trusted journal. Resolve the same visible topic again before the
        // existing proposal core records its pinned parent identity.
        drop(self.pin_topic(&topic)?);
        let journal = self.0.journal_directory(local_data).map_err(error)?;
        let _pin = self.pin_topic(&topic)?;
        let candidate = serde_json::json!({
            "version": 1,
            "operation": "create_new",
            "relativePath": relative_path,
            "expectedOldSha256": null,
            "newContent": input.content,
        });
        self.0
            .prepare_new_file_proposal(
                "aster-obsidian",
                "create-note",
                &candidate.to_string(),
                journal,
            )
            .map_err(error)
    }

    #[cfg(windows)]
    pub(crate) fn discard_note(&self) {
        let _ = self.0.revoke_pending("aster-obsidian", "create-note");
    }

    #[cfg(windows)]
    pub(crate) fn create_note(
        &self,
        candidate: super::ProposalReview,
        approval: crate::native_review::NativeApproval,
    ) -> Result<RelativePath, String> {
        let parts = note_components(&candidate.relative_path).map_err(error)?;
        // Recheck visibility (not only identity) at apply, with the Windows
        // directory chain pinned until the create-only operation finishes.
        let _pin = self
            .pin_topic(&parts[0])
            .inspect_err(|_| self.discard_note())?;
        match self
            .0
            .apply_new_file_candidate(&candidate.proposal_id, approval)
            .map_err(error)?
        {
            super::CreationOutcome::Verified => Ok(RelativePath {
                relative_path: candidate.relative_path,
            }),
            super::CreationOutcome::AttentionRequired => Err("obsidian_verify_note".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_one_visible_topic_and_markdown_leaf_are_allowed() {
        for bad in [
            "../Python/Atlas.md",
            "Python/atlas.txt",
            "Python/NUL.md",
            "Python/.hidden.md",
            "Python/Atlas.md:stream",
            "Python/secret.md",
            "Python/Atlas.md.",
            "Python/Sub/Atlas.md",
            r"Python\Atlas.md",
            r"C:\Python\Atlas.md",
            ".obsidian/Atlas.md",
            "Python?/Atlas.md",
            "Python/ Atlas.md",
        ] {
            assert_eq!(note_components(bad), Err(WorkspaceError::Denied), "{bad}");
        }
        assert_eq!(
            note_components("Guías/Atlas - Setup.MD").unwrap(),
            ["Guías", "Atlas - Setup.MD"]
        );
    }

    #[test]
    fn note_body_is_bounded_without_truncation() {
        assert_eq!(
            note_content(&"x ".repeat(1_201)),
            Err(WorkspaceError::TooLarge)
        );
        assert_eq!(
            note_content(&"x".repeat(MAX_NOTE_BYTES as usize + 1)),
            Err(WorkspaceError::TooLarge)
        );
        assert_eq!(note_content("note\0"), Err(WorkspaceError::Denied));
        assert_eq!(note_content("# Atlas\nPython 3.12"), Ok(()));
    }

    #[test]
    fn intent_shapes_reject_absolute_path_approval_and_extra_fields() {
        assert!(serde_json::from_value::<CreateNote>(serde_json::json!({
            "topic": "Python", "title": "Atlas", "content": "synthetic", "approved": true
        }))
        .is_err());
        assert!(serde_json::from_value::<OpenNote>(serde_json::json!({
            "relativePath": "Python/Atlas.md", "root": r"C:\Somewhere"
        }))
        .is_err());
        assert!(serde_json::from_value::<Search>(serde_json::json!({
            "topic": "Python", "query": "Atlas", "recursive": true
        }))
        .is_err());
    }

    #[cfg(windows)]
    struct VaultFixture {
        base: PathBuf,
        vault: PathBuf,
        app: PathBuf,
        state: Vault,
    }

    #[cfg(windows)]
    impl VaultFixture {
        fn new() -> Self {
            // As with the workspace fixtures, TEMP may use an ineligible
            // volume, namespace or parent. Try only disposable directories,
            // and require normal production admission for both siblings.
            let mut failures = Vec::new();
            for (location, candidate) in [
                ("checkout", std::env::current_dir().ok()),
                (
                    "local app data",
                    std::env::var_os("LOCALAPPDATA").map(PathBuf::from),
                ),
                ("temp", Some(std::env::temp_dir())),
            ] {
                let Some(candidate) = candidate else { continue };
                let base = candidate.join(format!(
                    "aster-obsidian-test-{}",
                    windows_fs::random_nonce().unwrap()
                ));
                if let Err(err) = std::fs::create_dir(&base) {
                    failures.push(format!("{location}: create root: {:?}", err.kind()));
                    continue;
                }
                let vault = base.join("Vault");
                let app = base.join("LocalData");
                let state = Vault::default();
                let ready = std::fs::create_dir(&vault)
                    .and_then(|_| std::fs::create_dir(&app))
                    .map_err(|err| format!("create siblings: {:?}", err.kind()))
                    .and_then(|()| state.select(vault.clone(), app.clone()).map(|_| ()));
                match ready {
                    Ok(()) => {
                        return Self {
                            base,
                            vault,
                            app,
                            state,
                        }
                    }
                    Err(reason) => {
                        failures.push(format!("{location}: {reason}"));
                        drop(state);
                        std::fs::remove_dir_all(&base).unwrap();
                    }
                }
            }
            panic!("No eligible local NTFS vault fixture: {failures:?}");
        }
    }

    #[cfg(windows)]
    impl Drop for VaultFixture {
        fn drop(&mut self) {
            // Close the selected root and all ancestor handles first.
            drop(std::mem::take(&mut self.state));
            let cleanup = std::fs::remove_dir_all(&self.base);
            if !std::thread::panicking() {
                cleanup.expect("remove disposable vault fixture");
            }
        }
    }

    #[cfg(windows)]
    #[test]
    fn windows_fixture_is_bounded_to_the_selected_topic_and_native_scope() {
        let fixture = VaultFixture::new();
        let docs = fixture.vault.join("Python");
        std::fs::create_dir(&docs).unwrap();
        std::fs::create_dir(fixture.vault.join("Rust")).unwrap();
        for index in 1..=4 {
            std::fs::write(
                docs.join(format!("Atlas {index}.md")),
                "# One retained decision",
            )
            .unwrap();
        }
        std::fs::write(fixture.vault.join("Rust/Atlas 0.md"), "Other topic").unwrap();
        std::fs::write(docs.join(".Atlas.md"), "Hidden").unwrap();
        let state = &fixture.state;
        assert!(state
            .select(fixture.base.clone(), fixture.app.clone())
            .is_err());
        let result = state
            .search(Search {
                topic: "Python".into(),
                query: "atlas".into(),
            })
            .unwrap();
        assert!(result.limited);
        assert_eq!(result.notes.len(), 3);
        assert_eq!(result.notes[0].relative_path, "Python/Atlas 1.md");
        let result = state
            .search(Search {
                topic: "Unknown".into(),
                query: "atlas".into(),
            })
            .unwrap();
        assert!(result.notes.is_empty());
        let (path, content) = state
            .read(OpenNote {
                relative_path: "Python/Atlas 1.md".into(),
            })
            .unwrap();
        assert_eq!(path, "Python/Atlas 1.md");
        assert_eq!(content.as_str(), "# One retained decision");
        assert!(state
            .read(OpenNote {
                relative_path: "Python/.Atlas.md".into()
            })
            .is_err());
        state.clear().unwrap();
        assert!(state
            .read(OpenNote {
                relative_path: "Python/Atlas 1.md".into()
            })
            .is_err());
    }

    #[cfg(windows)]
    #[test]
    fn windows_fixture_requires_exact_native_review_and_never_replaces_notes() {
        use crate::native_review::{NativeApproval, ReviewDocument};

        let fixture = VaultFixture::new();
        let state = &fixture.state;
        let vault = &fixture.vault;
        let intent = || CreateTopic {
            topic: "Rust".into(),
        };
        let rejected = state.prepare_topic(intent()).unwrap();
        assert!(state
            .create_topic(rejected, NativeApproval::for_fixture("not reviewed"))
            .is_err());
        assert!(!vault.join("Rust").exists());

        let folder = state.prepare_topic(intent()).unwrap();
        let document = folder.review_document().unwrap();
        let accepted = NativeApproval::for_fixture(document.binding_sha256());
        assert_eq!(
            state.create_topic(folder, accepted).unwrap().relative_path,
            "Rust"
        );

        let intent = || CreateNote {
            topic: "Rust".into(),
            title: "Atlas".into(),
            content: "# Exact note".into(),
        };
        let note = state.prepare_note(&intent(), fixture.app.clone()).unwrap();
        let document = ReviewDocument::proposal(&note.diff).unwrap();
        let accepted = NativeApproval::for_fixture(document.binding_sha256());
        assert_eq!(
            state.create_note(note, accepted).unwrap().relative_path,
            "Rust/Atlas.md"
        );
        assert_eq!(
            std::fs::read_to_string(vault.join("Rust/Atlas.md")).unwrap(),
            "# Exact note"
        );
        assert!(state.prepare_note(&intent(), fixture.app.clone()).is_err());
        assert_eq!(
            std::fs::read_to_string(vault.join("Rust/Atlas.md")).unwrap(),
            "# Exact note"
        );
        state.clear().unwrap();
    }

    #[test]
    fn no_root_is_granted_by_default_or_by_clear() {
        let v = Vault::default();
        assert!(!v.status().unwrap().selected);
        assert!(!v.clear().unwrap().selected);
        assert_eq!(v.status().unwrap().label, "");
    }
}
