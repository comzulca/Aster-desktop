//! Local project metadata only. No file paths, disclosure, model context or provider access.
use crate::private_store::{new_id, Domain, PrivateStore, StoreError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::Path};
use zeroize::Zeroizing;

const MAX_PROJECTS: usize = 64;
const MAX_LINKS: usize = 512;
const MAX_EVENTS: usize = 1024;

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Project {
    id: String,
    name: String,
}

#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    revision: u64,
    projects: Vec<Project>,
    assignments: BTreeMap<String, String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateIntent {
    pub expected_revision: u64,
    pub name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssignIntent {
    pub expected_revision: u64,
    pub conversation_id: String,
    pub project_id: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", tag = "operation", deny_unknown_fields)]
enum Change {
    Create(Project),
    Assign {
        conversation: String,
        project: Option<String>,
    },
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Record {
    version: u32,
    revision: u64,
    previous: String,
    change: Change,
}

fn valid_id(id: &str) -> bool {
    crate::admission::opaque_id(id)
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name == name.trim()
        && name.chars().count() <= 80
        && !name.chars().any(|c| {
            c.is_control() || matches!(c, '\u{061C}' | '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}')
        })
}

impl Snapshot {
    fn apply(&mut self, change: &Change) -> Result<(), StoreError> {
        match change {
            Change::Create(project) => {
                if !valid_id(&project.id) || !valid_name(&project.name) {
                    return Err(StoreError::Denied);
                }
                if self.projects.len() >= MAX_PROJECTS {
                    return Err(StoreError::Capacity);
                }
                if self.projects.iter().any(|item| {
                    item.id == project.id || item.name.to_lowercase() == project.name.to_lowercase()
                }) {
                    return Err(StoreError::Conflict);
                }
                self.projects.push(project.clone());
            }
            Change::Assign {
                conversation,
                project,
            } => {
                if !valid_id(conversation) {
                    return Err(StoreError::Denied);
                }
                if let Some(project) = project {
                    if !self.projects.iter().any(|item| item.id == *project) {
                        return Err(StoreError::Denied);
                    }
                    if self.assignments.len() >= MAX_LINKS
                        && !self.assignments.contains_key(conversation)
                    {
                        return Err(StoreError::Capacity);
                    }
                    self.assignments
                        .insert(conversation.clone(), project.clone());
                } else {
                    self.assignments.remove(conversation);
                }
            }
        }
        Ok(())
    }

    // Removed conversation IDs are not returned to the renderer.
    pub fn visible_for(mut self, visible: &[crate::history::ConversationSummary]) -> Self {
        self.assignments
            .retain(|id, _| visible.iter().any(|conversation| conversation.id == *id));
        self
    }
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub struct Projects {
    store: PrivateStore,
    current: Snapshot,
    head: String,
}

impl Projects {
    pub fn open(base: &Path) -> Result<Self, StoreError> {
        Self::restore(PrivateStore::open(base, Domain::Projects)?)
    }

    fn replay(store: &PrivateStore) -> Result<(Snapshot, String), StoreError> {
        let records = store.records()?;
        if records.len() > MAX_EVENTS {
            return Err(StoreError::Capacity);
        }
        let mut ordered = BTreeMap::new();
        for (_, bytes) in records {
            let bytes = Zeroizing::new(bytes);
            let record: Record = serde_json::from_slice(&bytes).map_err(|_| StoreError::Corrupt)?;
            let digest = digest(&bytes);
            if record.version != 1 || ordered.insert(record.revision, (record, digest)).is_some() {
                return Err(StoreError::Corrupt);
            }
        }
        let mut current = Snapshot::default();
        let mut head = String::new();
        for (revision, (record, digest)) in ordered {
            if revision != current.revision + 1 || record.previous != head {
                return Err(StoreError::Corrupt);
            }
            current
                .apply(&record.change)
                .map_err(|_| StoreError::Corrupt)?;
            current.revision = revision;
            head = digest;
        }
        Ok((current, head))
    }

    fn restore(store: PrivateStore) -> Result<Self, StoreError> {
        let (current, head) = Self::replay(&store)?;
        Ok(Self {
            store,
            current,
            head,
        })
    }

    pub fn snapshot(&self) -> Result<Snapshot, StoreError> {
        let (current, head) = Self::replay(&self.store)?;
        if current.revision != self.current.revision || head != self.head {
            return Err(StoreError::Corrupt);
        }
        Ok(self.current.clone())
    }

    pub fn create(&mut self, input: CreateIntent) -> Result<Snapshot, StoreError> {
        self.commit(
            input.expected_revision,
            Change::Create(Project {
                id: new_id()?,
                name: input.name,
            }),
        )
    }

    pub fn assign(&mut self, input: AssignIntent) -> Result<Snapshot, StoreError> {
        self.commit(
            input.expected_revision,
            Change::Assign {
                conversation: input.conversation_id,
                project: input.project_id,
            },
        )
    }

    fn commit(&mut self, expected: u64, change: Change) -> Result<Snapshot, StoreError> {
        let mut next = self.snapshot()?;
        if next.revision != expected {
            return Err(StoreError::Conflict);
        }
        next.apply(&change)?;
        if next.assignments == self.current.assignments
            && next.projects.len() == self.current.projects.len()
        {
            return Ok(next);
        }
        if next.revision >= MAX_EVENTS as u64 {
            return Err(StoreError::Capacity);
        }
        next.revision += 1;
        let record = Record {
            version: 1,
            revision: next.revision,
            previous: self.head.clone(),
            change,
        };
        let bytes = Zeroizing::new(serde_json::to_vec(&record).map_err(|_| StoreError::Corrupt)?);
        self.store.append(&new_id()?, &bytes)?;
        self.head = digest(&bytes);
        self.current = next;
        Ok(self.current.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create(name: &str, expected_revision: u64) -> CreateIntent {
        CreateIntent {
            expected_revision,
            name: name.to_owned(),
        }
    }

    #[test]
    fn restores_project_and_assignment_without_affecting_conversation_records() {
        let mut projects = Projects::restore(PrivateStore::memory(Domain::Projects)).unwrap();
        let created = projects.create(create("Atlas", 0)).unwrap();
        let id = created.projects[0].id.clone();
        let conversation = "a".repeat(32);
        projects
            .assign(AssignIntent {
                expected_revision: 1,
                conversation_id: conversation.clone(),
                project_id: Some(id.clone()),
            })
            .unwrap();
        let restored = Projects::restore(projects.store)
            .unwrap()
            .snapshot()
            .unwrap();
        assert_eq!(restored.projects[0].name, "Atlas");
        assert_eq!(restored.assignments.get(&conversation), Some(&id));
    }

    #[test]
    fn rejects_duplicate_invalid_and_stale_proposals() {
        let mut projects = Projects::restore(PrivateStore::memory(Domain::Projects)).unwrap();
        projects.create(create("Atlas", 0)).unwrap();
        assert_eq!(
            projects.create(create("atlas", 1)).err(),
            Some(StoreError::Conflict)
        );
        assert_eq!(
            projects.create(create("  ", 1)).err(),
            Some(StoreError::Denied)
        );
        assert_eq!(
            projects.create(create("Other", 0)).err(),
            Some(StoreError::Conflict)
        );
        assert_eq!(projects.snapshot().unwrap().revision, 1);
        assert!(serde_json::from_value::<CreateIntent>(serde_json::json!({
            "expectedRevision": 1, "name": "Other", "path": "not an IPC field"
        }))
        .is_err());
    }
}
