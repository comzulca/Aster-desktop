//! Host-only evidence validation, never a deserializable approval DTO.
use super::{branch, digest, hash, id, number, sha, text, Binding, Error, Intent, Plan};
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Copy, Clone, Eq, PartialEq, Serialize)]
pub(super) enum Access {
    Unknown,
    None,
    Read,
    Write,
}
impl Access {
    fn readable(self) -> bool {
        matches!(self, Self::Read | Self::Write)
    }
}
#[derive(Clone, Eq, PartialEq, Serialize)]
pub(super) struct Permissions {
    pub contents: Access,
    pub issues: Access,
    pub pulls: Access,
    pub checks: Access,
    pub statuses: Access,
    pub actions: Access,
}
#[derive(Clone, Eq, PartialEq, Serialize)]
pub(super) struct Complete<T> {
    pub complete: bool,
    pub total: usize,
    pub items: Vec<T>,
}
impl<T> Complete<T> {
    fn bounded_complete(&self) -> bool {
        self.complete && self.total == self.items.len() && self.total <= 1_000
    }
}
#[derive(Clone, Eq, PartialEq, Serialize)]
pub(super) struct Branch {
    pub name: String,
    pub head: String,
    pub protected: Option<bool>,
    pub active_rules: Complete<String>,
}
impl Branch {
    fn valid(&self) -> bool {
        branch(&self.name) && sha(&self.head)
    }
    fn unprotected(&self) -> bool {
        self.valid()
            && self.protected == Some(false)
            && self.active_rules.bounded_complete()
            && self.active_rules.items.is_empty()
    }
}
#[derive(Clone, Eq, PartialEq, Serialize)]
pub(super) struct TreeEntry {
    pub path: String,
    pub mode: String,
    pub kind: String,
    pub sha: String,
}
#[derive(Clone, Eq, PartialEq, Serialize)]
pub(super) struct Tree {
    pub commit: String,
    pub truncated: bool,
    pub entries: Complete<TreeEntry>,
}
#[derive(Clone, Eq, PartialEq, Serialize)]
pub(super) struct Blob {
    pub sha: String,
    pub content: String,
}
#[derive(Clone, Eq, PartialEq, Serialize)]
pub(super) struct Issue {
    pub repository_id: u64,
    pub id: u64,
    pub number: u64,
    pub is_pull_request: bool,
    pub open: bool,
    pub title: String,
    pub body: String,
    pub updated_at: String,
}
impl Issue {
    fn valid(&self, repository_id: u64) -> bool {
        self.repository_id == repository_id
            && id(self.id)
            && number(self.number)
            && !self.is_pull_request
            && !self.title.trim().is_empty()
            && text(&self.title, 16_384, 65_536, false)
            && text(&self.body, 65_536, 65_536, true)
            && self.title.len() + self.body.len() <= 65_536
            && utc_timestamp(&self.updated_at)
    }
    pub(super) fn fingerprint(&self) -> Result<String, Error> {
        Ok(hash(
            &serde_json::to_vec(self).map_err(|_| Error::Malformed)?,
        ))
    }
}
fn utc_timestamp(value: &str) -> bool {
    // Fixed upstream second-resolution UTC shape, not a permissive prose field.
    let bytes = value.as_bytes();
    let shape = bytes.len() == 20
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b'T'
        && bytes[13] == b':'
        && bytes[16] == b':'
        && bytes[19] == b'Z'
        && bytes
            .iter()
            .enumerate()
            .all(|(i, b)| [4, 7, 10, 13, 16, 19].contains(&i) || b.is_ascii_digit());
    if !shape {
        return false;
    }
    let numeric = |range: std::ops::Range<usize>| value[range].parse::<u32>().unwrap_or(u32::MAX);
    let year = numeric(0..4);
    let month = numeric(5..7);
    let day = numeric(8..10);
    let maximum_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        _ => 0,
    };
    year >= 1970
        && (1..=maximum_day).contains(&day)
        && numeric(11..13) <= 23
        && numeric(14..16) <= 59
        && numeric(17..19) <= 59
}
#[derive(Clone, Eq, PartialEq, Serialize)]
pub(super) struct Pull {
    pub repository_id: u64,
    pub id: u64,
    pub number: u64,
    pub head_repository_id: u64,
    pub base_repository_id: u64,
    pub head_branch: String,
    pub base_branch: String,
    pub head: String,
    pub base: String,
    pub open: bool,
    pub draft: bool,
    pub stacked: Option<bool>,
    pub queued: Option<bool>,
    pub mergeable: Option<bool>,
    pub title: String,
    pub body: String,
}
impl Pull {
    fn valid(&self, repository_id: u64) -> bool {
        self.repository_id == repository_id
            && self.head_repository_id == repository_id
            && self.base_repository_id == repository_id
            && id(self.id)
            && number(self.number)
            && branch(&self.head_branch)
            && branch(&self.base_branch)
            && self.head_branch != self.base_branch
            && sha(&self.head)
            && sha(&self.base)
            && !self.title.trim().is_empty()
            && text(&self.title, 16_384, 65_536, false)
            && text(&self.body, 65_536, 65_536, true)
            && self.title.len() + self.body.len() <= 65_536
    }
}
#[derive(Copy, Clone, Eq, PartialEq, Serialize)]
pub(super) enum Execution {
    Queued,
    InProgress,
    Completed,
}
#[derive(Copy, Clone, Eq, PartialEq, Serialize)]
pub(super) enum Conclusion {
    Success,
    Failure,
    Neutral,
    Skipped,
    Cancelled,
    TimedOut,
    ActionRequired,
    Unknown,
}
#[derive(Clone, Eq, PartialEq, Serialize)]
pub(super) struct Check {
    pub id: u64,
    pub suite_id: u64,
    pub app_id: u64,
    pub head: String,
    pub name: String,
    pub status: Execution,
    pub conclusion: Conclusion,
}
#[derive(Clone, Eq, PartialEq, Serialize)]
pub(super) struct Suite {
    pub id: u64,
    pub app_id: u64,
    pub head: String,
    pub status: Execution,
    pub conclusion: Conclusion,
}
#[derive(Copy, Clone, Eq, PartialEq, Serialize)]
pub(super) enum StatusState {
    Pending,
    Success,
    Failure,
    Error,
}
#[derive(Clone, Eq, PartialEq, Serialize)]
pub(super) struct Status {
    pub id: u64,
    pub context: String,
    pub head: String,
    pub state: StatusState,
}
#[derive(Copy, Clone, Eq, PartialEq, Serialize)]
pub(super) enum ReviewState {
    Approved,
    Commented,
    ChangesRequested,
    Dismissed,
    Pending,
    Unknown,
}
#[derive(Clone, Eq, PartialEq, Serialize)]
pub(super) struct Review {
    pub id: u64,
    pub user_id: u64,
    pub head: String,
    pub state: ReviewState,
}
#[derive(Clone, Eq, PartialEq, Serialize)]
pub(super) struct MergeEvidence {
    pub checks: Complete<Check>,
    pub suites: Complete<Suite>,
    pub statuses: Complete<Status>,
    pub combined_status: StatusState,
    pub reviews: Complete<Review>,
    // This is a host-owned project policy decision, not imported repo prose.
    pub allow_no_results: bool,
}
#[derive(Clone, Eq, PartialEq, Serialize)]
pub(super) enum ResourceSnapshot {
    CreateBranch {
        source: Branch,
        target_absent: bool,
    },
    File {
        branch: Branch,
        tree: Tree,
        existing: Option<Blob>,
    },
    NewIssue,
    Issue(Issue),
    NewPull {
        head: Branch,
        base: Branch,
        no_existing_pull: bool,
    },
    Merge {
        pull: Box<Pull>,
        head: Branch,
        base: Branch,
        evidence: MergeEvidence,
    },
}
#[derive(Clone, Eq, PartialEq, Serialize)]
pub(super) struct Snapshot {
    pub binding: Binding,
    pub enabled: bool,
    pub token_live: bool,
    // Fresh independently enumerated dedicated-App scope must match binding.
    pub installed_binding: Option<Binding>,
    pub permissions: Permissions,
    pub default_branch: String,
    pub issues_enabled: bool,
    pub resource: ResourceSnapshot,
}

pub(super) fn validate_snapshot(intent: &Intent, value: &Snapshot) -> Result<(), Error> {
    intent.validate()?;
    if !value.enabled || !value.token_live {
        return Err(Error::Disabled);
    }
    if !value.binding.valid()
        || value.installed_binding.as_ref() != Some(&value.binding)
        || !branch(&value.default_branch)
    {
        return Err(Error::Conflict);
    }
    let permission = match intent {
        Intent::CreateBranch { .. }
        | Intent::CreateFile { .. }
        | Intent::UpdateFile { .. }
        | Intent::MergePullRequest { .. } => value.permissions.contents,
        Intent::CreateIssue { .. } | Intent::CommentIssue { .. } | Intent::CloseIssue { .. } => {
            value.permissions.issues
        }
        Intent::CreatePullRequest { .. } => value.permissions.pulls,
    };
    if permission != Access::Write {
        return Err(Error::Permission);
    }
    // Evidence permissions are independently required by the closed host
    // contract; public endpoint success cannot substitute installed-App scope.
    if (matches!(intent, Intent::CreatePullRequest { .. })
        && !value.permissions.contents.readable())
        || (matches!(intent, Intent::MergePullRequest { .. })
            && !value.permissions.pulls.readable())
    {
        return Err(Error::Permission);
    }
    let valid = match (intent, &value.resource) {
        (
            Intent::CreateBranch {
                source_branch,
                expected_head,
                new_branch,
            },
            ResourceSnapshot::CreateBranch {
                source,
                target_absent,
            },
        ) => {
            source.valid()
                && &source.name == source_branch
                && &source.head == expected_head
                && *target_absent
                && new_branch != &value.default_branch
        }
        (
            Intent::CreateFile {
                branch,
                expected_head,
                path,
                ..
            },
            ResourceSnapshot::File {
                branch: observed,
                tree,
                existing,
            },
        ) => {
            observed.unprotected()
                && &observed.name == branch
                && &observed.head == expected_head
                && branch != &value.default_branch
                && existing.is_none()
                && validate_tree(tree, path, expected_head, None)
        }
        (
            Intent::UpdateFile {
                branch,
                expected_head,
                path,
                expected_blob,
                ..
            },
            ResourceSnapshot::File {
                branch: observed,
                tree,
                existing,
            },
        ) => {
            observed.unprotected()
                && &observed.name == branch
                && &observed.head == expected_head
                && branch != &value.default_branch
                && existing.as_ref().is_some_and(|blob| {
                    &blob.sha == expected_blob && text(&blob.content, 65_536, 65_536, true)
                })
                && validate_tree(tree, path, expected_head, Some(expected_blob))
        }
        (Intent::CreateIssue { .. }, ResourceSnapshot::NewIssue) => value.issues_enabled,
        (
            Intent::CommentIssue {
                number,
                expected_id,
                expected_snapshot,
                ..
            },
            ResourceSnapshot::Issue(issue),
        )
        | (
            Intent::CloseIssue {
                number,
                expected_id,
                expected_snapshot,
                ..
            },
            ResourceSnapshot::Issue(issue),
        ) => {
            value.issues_enabled
                && issue.valid(value.binding.repository_id)
                && issue.id == *expected_id
                && issue.number == *number
                && issue.fingerprint()? == *expected_snapshot
                && (!matches!(intent, Intent::CloseIssue { .. }) || issue.open)
        }
        (
            Intent::CreatePullRequest {
                head_branch,
                base_branch,
                expected_head,
                expected_base,
                ..
            },
            ResourceSnapshot::NewPull {
                head,
                base,
                no_existing_pull,
            },
        ) => {
            head.valid()
                && base.valid()
                && &head.name == head_branch
                && &base.name == base_branch
                && &head.head == expected_head
                && &base.head == expected_base
                && *no_existing_pull
        }
        (
            Intent::MergePullRequest {
                number,
                expected_id,
                head_branch,
                base_branch,
                expected_head,
                expected_base,
                ..
            },
            ResourceSnapshot::Merge {
                pull,
                head,
                base,
                evidence,
            },
        ) => {
            if value.binding.aster_project() {
                // The five exact jobs/15+6/causal CPU receipt verifier is absent;
                // a boolean or job-name list must never qualify that stronger gate.
                return Err(Error::GateUnavailable);
            }
            if !value.permissions.checks.readable() || !value.permissions.statuses.readable() {
                return Err(Error::Permission);
            }
            pull.valid(value.binding.repository_id)
                && pull.id == *expected_id
                && pull.number == *number
                && pull.open
                && !pull.draft
                && pull.stacked == Some(false)
                && pull.queued == Some(false)
                && pull.mergeable == Some(true)
                && &pull.head_branch == head_branch
                && &pull.base_branch == base_branch
                && &pull.head == expected_head
                && &pull.base == expected_base
                && head.valid()
                && base.unprotected()
                && &head.name == head_branch
                && &base.name == base_branch
                && &head.head == expected_head
                && &base.head == expected_base
                && validate_merge(evidence, expected_head)
        }
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(Error::Conflict)
    }
}

fn evidence_path(value: &str) -> bool {
    // Tree evidence is remote data, never a local path or a mutation target.
    // Unrelated .github/.env, Unicode, spaces and Windows-reserved names are
    // legitimate Git entries. Bound their exact spelling without interpreting
    // it; the much narrower mutation-path policy is already applied to intent.
    !value.is_empty()
        && value.len() <= 512
        && !value.contains('\\')
        && text(value, 512, 512, false)
        && value
            .split('/')
            .all(|part| !part.is_empty() && part.len() <= 128 && part != "." && part != "..")
}

fn evidence_alias(value: &str) -> String {
    // Conservative target/ancestor alias screening only. Nothing is opened or
    // materialised on Windows, and this is not a universal filesystem
    // normalisation claim. Keep exact remote bytes for lookup and review.
    value
        .split('/')
        .map(|part| {
            part.trim_end_matches(['.', ' '])
                .chars()
                .flat_map(char::to_lowercase)
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn validate_tree(
    tree: &Tree,
    target: &str,
    expected_head: &str,
    expected_blob: Option<&str>,
) -> bool {
    if tree.commit != expected_head || tree.truncated || !tree.entries.bounded_complete() {
        return false;
    }
    let mut paths = BTreeSet::new();
    for entry in &tree.entries.items {
        if !evidence_path(&entry.path)
            || !sha(&entry.sha)
            || !paths.insert(entry.path.as_str())
            || !matches!(
                (entry.mode.as_str(), entry.kind.as_str()),
                ("040000", "tree")
                    | ("100644" | "100755" | "120000", "blob")
                    | ("160000", "commit")
            )
        {
            return false;
        }
    }
    let pieces: Vec<_> = target.split('/').collect();
    for index in 1..pieces.len() {
        let parent = pieces[..index].join("/");
        if tree.entries.items.iter().any(|entry| {
            let prefix = entry
                .path
                .split('/')
                .take(index)
                .collect::<Vec<_>>()
                .join("/");
            prefix != parent && evidence_alias(&prefix) == evidence_alias(&parent)
        }) || !tree
            .entries
            .items
            .iter()
            .any(|entry| entry.path == parent && entry.mode == "040000" && entry.kind == "tree")
        {
            return false;
        }
    }
    let entry = tree.entries.items.iter().find(|entry| entry.path == target);
    if tree
        .entries
        .items
        .iter()
        .any(|entry| entry.path != target && evidence_alias(&entry.path) == evidence_alias(target))
        || tree.entries.items.iter().any(|entry| {
            evidence_alias(&entry.path).starts_with(&format!("{}/", evidence_alias(target)))
        })
    {
        return false;
    }
    match (expected_blob, entry) {
        (None, None) => true,
        (Some(expected), Some(entry)) => {
            entry.mode == "100644" && entry.kind == "blob" && entry.sha == expected
        }
        _ => false,
    }
}

fn validate_merge(evidence: &MergeEvidence, head: &str) -> bool {
    if !evidence.checks.bounded_complete()
        || !evidence.suites.bounded_complete()
        || !evidence.statuses.bounded_complete()
        || !evidence.reviews.bounded_complete()
    {
        return false;
    }
    if evidence.checks.items.is_empty()
        && evidence.suites.items.is_empty()
        && evidence.statuses.items.is_empty()
        && !evidence.allow_no_results
    {
        return false;
    }
    let mut ids = BTreeSet::new();
    for suite in &evidence.suites.items {
        if !id(suite.id)
            || !id(suite.app_id)
            || suite.head != head
            || suite.status != Execution::Completed
            || suite.conclusion != Conclusion::Success
            || !ids.insert(suite.id)
        {
            return false;
        }
    }
    let mut check_ids = BTreeSet::new();
    let mut contexts = BTreeSet::new();
    for check in &evidence.checks.items {
        if !id(check.id)
            || !id(check.app_id)
            || check.head != head
            || check.status != Execution::Completed
            || check.conclusion != Conclusion::Success
            || !text(&check.name, 256, 1_024, false)
            || check.name.trim().is_empty()
            || !check_ids.insert(check.id)
            || !contexts.insert((check.app_id, check.name.as_str()))
            || !evidence
                .suites
                .items
                .iter()
                .any(|suite| suite.id == check.suite_id && suite.app_id == check.app_id)
        {
            return false;
        }
    }
    // A successful suite with no bounded current checks is ambiguous, not PASS.
    if evidence.suites.items.iter().any(|suite| {
        !evidence
            .checks
            .items
            .iter()
            .any(|check| check.suite_id == suite.id)
    }) {
        return false;
    }
    let mut contexts = BTreeSet::new();
    let mut status_ids = BTreeSet::new();
    for status in &evidence.statuses.items {
        if !id(status.id)
            || status.head != head
            || status.state != StatusState::Success
            || status.context.trim().is_empty()
            || !text(&status.context, 256, 1_024, false)
            || !status_ids.insert(status.id)
            || !contexts.insert(status.context.as_str())
        {
            return false;
        }
    }
    if evidence.statuses.items.is_empty() {
        if evidence.combined_status != StatusState::Pending {
            return false;
        } // Distinct documented NoStatuses.
    } else if evidence.combined_status != StatusState::Success {
        return false;
    }
    let mut reviewers = BTreeSet::new();
    let mut review_ids = BTreeSet::new();
    for review in &evidence.reviews.items {
        if !id(review.id)
            || !id(review.user_id)
            || !sha(&review.head)
            || !review_ids.insert(review.id)
            || !reviewers.insert(review.user_id)
            || matches!(
                review.state,
                ReviewState::ChangesRequested | ReviewState::Pending | ReviewState::Unknown
            )
            || (review.state != ReviewState::Dismissed && review.head != head)
        {
            return false;
        }
    }
    true
}

#[derive(Clone)]
pub(super) enum Readback {
    Branch {
        binding: Binding,
        name: String,
        head: String,
    },
    File {
        binding: Binding,
        branch: String,
        path: String,
        response_commit: String,
        observed_commit: String,
        parents: Vec<String>,
        blob_sha: String,
        mode: String,
        content_hash: String,
        commit_message: String,
    },
    Issue {
        binding: Binding,
        response_id: u64,
        issue: Issue,
    },
    Comment {
        binding: Binding,
        issue_id: u64,
        response_id: u64,
        comment_id: u64,
        body_hash: String,
    },
    ClosedIssue {
        binding: Binding,
        issue: Issue,
        reason: super::CloseReason,
    },
    Pull {
        binding: Binding,
        response_id: u64,
        pull: Pull,
    },
    Merge {
        binding: Binding,
        http_status: u16,
        merged: bool,
        response_commit: String,
        observed_commit: String,
        observed_merge_commit: String,
        observed_merged: bool,
        parents: Vec<String>,
        pull: Pull,
        commit_title: String,
        commit_message: String,
    },
}
#[derive(Clone, Debug, serde::Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct EffectIds {
    pub resource_id: Option<u64>,
    pub number: Option<u64>,
    pub commit: Option<String>,
}
impl EffectIds {
    pub(super) fn valid(&self) -> bool {
        (self.resource_id.is_some() || self.commit.is_some())
            && self.resource_id.is_none_or(id)
            && self.number.is_none_or(number)
            && self.commit.as_deref().is_none_or(sha)
    }
}
pub(super) fn verify_readback(plan: &Plan, readback: &Readback) -> Result<EffectIds, Error> {
    let expected_binding = &plan.snapshot.binding;
    let none = || EffectIds {
        resource_id: None,
        number: None,
        commit: None,
    };
    let effect = match (&plan.intent, readback) {
        (
            Intent::CreateBranch {
                new_branch,
                expected_head,
                ..
            },
            Readback::Branch {
                binding,
                name,
                head,
            },
        ) if binding == expected_binding && name == new_branch && head == expected_head => {
            EffectIds {
                commit: Some(head.clone()),
                ..none()
            }
        }
        (
            Intent::CreateFile {
                branch,
                path,
                expected_head,
                content,
                commit_title,
                commit_message,
            }
            | Intent::UpdateFile {
                branch,
                path,
                expected_head,
                content,
                commit_title,
                commit_message,
                ..
            },
            Readback::File {
                binding,
                branch: observed_branch,
                path: observed_path,
                response_commit,
                observed_commit,
                parents,
                blob_sha,
                mode,
                content_hash,
                commit_message: observed_message,
            },
        ) if binding == expected_binding
            && branch == observed_branch
            && path == observed_path
            && sha(response_commit)
            && response_commit == observed_commit
            && parents.len() == 1
            && parents[0] == *expected_head
            && sha(blob_sha)
            && mode == "100644"
            && digest(content_hash)
            && *content_hash == hash(content.as_bytes())
            && *observed_message == super::commit_message_text(commit_title, commit_message) =>
        {
            EffectIds {
                commit: Some(response_commit.clone()),
                ..none()
            }
        }
        (
            Intent::CreateIssue { title, body },
            Readback::Issue {
                binding,
                response_id,
                issue,
            },
        ) if binding == expected_binding
            && issue.valid(binding.repository_id)
            && *response_id == issue.id
            && issue.open
            && &issue.title == title
            && &issue.body == body =>
        {
            EffectIds {
                resource_id: Some(issue.id),
                number: Some(issue.number),
                ..none()
            }
        }
        (
            Intent::CommentIssue {
                expected_id, body, ..
            },
            Readback::Comment {
                binding,
                issue_id,
                response_id,
                comment_id,
                body_hash,
            },
        ) if binding == expected_binding
            && issue_id == expected_id
            && id(*response_id)
            && response_id == comment_id
            && *body_hash == hash(body.as_bytes()) =>
        {
            EffectIds {
                resource_id: Some(*comment_id),
                ..none()
            }
        }
        (
            Intent::CloseIssue {
                expected_id,
                number,
                reason,
                ..
            },
            Readback::ClosedIssue {
                binding,
                issue,
                reason: observed_reason,
            },
        ) if binding == expected_binding
            && issue.valid(binding.repository_id)
            && issue.id == *expected_id
            && issue.number == *number
            && !issue.open
            && reason == observed_reason
            && matches!(&plan.snapshot.resource,ResourceSnapshot::Issue(old) if old.title == issue.title && old.body == issue.body) =>
        {
            EffectIds {
                resource_id: Some(issue.id),
                number: Some(issue.number),
                ..none()
            }
        }
        (
            Intent::CreatePullRequest {
                head_branch,
                base_branch,
                expected_head,
                expected_base,
                title,
                body,
                draft,
            },
            Readback::Pull {
                binding,
                response_id,
                pull,
            },
        ) if binding == expected_binding
            && pull.valid(binding.repository_id)
            && *response_id == pull.id
            && pull.open
            && pull.draft == *draft
            && &pull.head_branch == head_branch
            && &pull.base_branch == base_branch
            && &pull.head == expected_head
            && &pull.base == expected_base
            && &pull.title == title
            && &pull.body == body =>
        {
            EffectIds {
                resource_id: Some(pull.id),
                number: Some(pull.number),
                ..none()
            }
        }
        (
            Intent::MergePullRequest {
                expected_id,
                number,
                expected_head,
                expected_base,
                commit_title,
                commit_message,
                ..
            },
            Readback::Merge {
                binding,
                http_status,
                merged,
                response_commit,
                observed_commit,
                observed_merge_commit,
                observed_merged,
                parents,
                pull,
                commit_title: observed_title,
                commit_message: observed_message,
            },
        ) if binding == expected_binding
            && *http_status == 200
            && *merged
            && sha(response_commit)
            && response_commit == observed_commit
            && *observed_merged
            && observed_merge_commit == response_commit
            && observed_title == commit_title
            && observed_message == commit_message
            && parents.len() == 2
            && parents[0] == *expected_base
            && parents[1] == *expected_head
            && pull.valid(binding.repository_id)
            && pull.id == *expected_id
            && pull.number == *number
            && !pull.open
            && &pull.head == expected_head
            && &pull.base == expected_base
            && matches!(&plan.snapshot.resource,ResourceSnapshot::Merge{pull:old,..} if old.head_branch == pull.head_branch && old.base_branch == pull.base_branch) =>
        {
            EffectIds {
                resource_id: Some(pull.id),
                number: Some(pull.number),
                commit: Some(response_commit.clone()),
            }
        }
        _ => return Err(Error::Conflict),
    };
    if effect.valid() {
        Ok(effect)
    } else {
        Err(Error::Conflict)
    }
}
