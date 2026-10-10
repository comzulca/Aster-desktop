//! Synthetic core fixtures, with additive hosted-Windows protected-store cases.
//! No owner-state, Credential Manager, runtime approval or live network access.
use super::*;
use github_mutation_ledger::{reserve_capacity, Memory};
use github_mutation_validation::*;
use std::{
    collections::VecDeque,
    panic::{catch_unwind, AssertUnwindSafe},
};

const NONCE: &str = "11111111111111111111111111111111";
fn head() -> String {
    "a".repeat(40)
}
fn blob_sha() -> String {
    "b".repeat(40)
}
fn base_sha() -> String {
    "c".repeat(40)
}
fn commit_sha() -> String {
    "d".repeat(40)
}
fn binding() -> Binding {
    Binding {
        generation: 7,
        account_id: 41,
        account_login: "synthetic-user".into(),
        app_id: APP_ID,
        installation_id: 51,
        repository_id: 1_412_554_572,
        owner: "comzulca".into(),
        name: "aster-v024-github-test-20261010".into(),
        private: true,
    }
}
fn complete<T>(items: Vec<T>) -> Complete<T> {
    Complete {
        complete: true,
        total: items.len(),
        items,
    }
}
fn branch_state(name: &str, sha: String) -> Branch {
    Branch {
        name: name.into(),
        head: sha,
        protected: Some(false),
        active_rules: complete(vec![]),
    }
}
fn issue() -> Issue {
    Issue {
        repository_id: binding().repository_id,
        id: 71,
        number: 3,
        is_pull_request: false,
        open: true,
        title: "Old synthetic issue".into(),
        body: "PRIVATE_REVIEW_CANARY".into(),
        updated_at: "2026-10-10T12:00:00Z".into(),
    }
}
fn pull() -> Pull {
    Pull {
        repository_id: binding().repository_id,
        id: 81,
        number: 4,
        head_repository_id: binding().repository_id,
        base_repository_id: binding().repository_id,
        head_branch: "aster/topic".into(),
        base_branch: "engineering-target".into(),
        head: head(),
        base: base_sha(),
        open: true,
        draft: false,
        stacked: Some(false),
        queued: Some(false),
        mergeable: Some(true),
        title: "Synthetic PR".into(),
        body: "Synthetic diff only.".into(),
    }
}
fn merge_evidence() -> MergeEvidence {
    MergeEvidence {
        checks: complete(vec![Check {
            id: 91,
            suite_id: 92,
            app_id: 93,
            head: head(),
            name: "synthetic-check".into(),
            status: Execution::Completed,
            conclusion: Conclusion::Success,
        }]),
        suites: complete(vec![Suite {
            id: 92,
            app_id: 93,
            head: head(),
            status: Execution::Completed,
            conclusion: Conclusion::Success,
        }]),
        statuses: complete(vec![]),
        combined_status: StatusState::Pending,
        reviews: complete(vec![Review {
            id: 94,
            user_id: 95,
            head: head(),
            state: ReviewState::Approved,
        }]),
        allow_no_results: false,
    }
}
fn snapshot(intent: &Intent) -> Snapshot {
    let resource = match intent {
        Intent::CreateBranch {
            source_branch,
            expected_head,
            ..
        } => ResourceSnapshot::CreateBranch {
            source: branch_state(source_branch, expected_head.clone()),
            target_absent: true,
        },
        Intent::CreateFile {
            branch,
            path: _,
            expected_head,
            ..
        } => ResourceSnapshot::File {
            branch: branch_state(branch, expected_head.clone()),
            tree: Tree {
                commit: expected_head.clone(),
                truncated: false,
                entries: complete(vec![]),
            },
            existing: None,
        },
        Intent::UpdateFile {
            branch,
            path,
            expected_head,
            expected_blob,
            ..
        } => ResourceSnapshot::File {
            branch: branch_state(branch, expected_head.clone()),
            tree: Tree {
                commit: expected_head.clone(),
                truncated: false,
                entries: complete(vec![TreeEntry {
                    path: path.clone(),
                    mode: "100644".into(),
                    kind: "blob".into(),
                    sha: expected_blob.clone(),
                }]),
            },
            existing: Some(Blob {
                sha: expected_blob.clone(),
                content: "Old synthetic file".into(),
            }),
        },
        Intent::CreateIssue { .. } => ResourceSnapshot::NewIssue,
        Intent::CommentIssue { .. } | Intent::CloseIssue { .. } => ResourceSnapshot::Issue(issue()),
        Intent::CreatePullRequest {
            head_branch,
            base_branch,
            expected_head,
            expected_base,
            ..
        } => ResourceSnapshot::NewPull {
            head: branch_state(head_branch, expected_head.clone()),
            base: branch_state(base_branch, expected_base.clone()),
            no_existing_pull: true,
        },
        Intent::MergePullRequest { .. } => ResourceSnapshot::Merge {
            pull: Box::new(pull()),
            head: branch_state("aster/topic", head()),
            base: branch_state("engineering-target", base_sha()),
            evidence: merge_evidence(),
        },
    };
    Snapshot {
        binding: binding(),
        enabled: true,
        token_live: true,
        installed_binding: Some(binding()),
        permissions: Permissions {
            contents: Access::Write,
            issues: Access::Write,
            pulls: Access::Write,
            checks: Access::Read,
            statuses: Access::Read,
            actions: Access::Read,
        },
        default_branch: "main".into(),
        issues_enabled: true,
        resource,
    }
}
fn intents() -> Vec<Intent> {
    vec![
        Intent::CreateBranch {
            source_branch: "main".into(),
            expected_head: head(),
            new_branch: "aster/new".into(),
        },
        Intent::CreateFile {
            branch: "aster/topic".into(),
            expected_head: head(),
            path: "README.md".into(),
            content: "new synthetic content".into(),
            commit_title: "Change synthetic file".into(),
            commit_message: "Synthetic fixture only.".into(),
        },
        Intent::UpdateFile {
            branch: "aster/topic".into(),
            expected_head: head(),
            path: "README.md".into(),
            expected_blob: blob_sha(),
            content: "new synthetic content".into(),
            commit_title: "Change synthetic file".into(),
            commit_message: "Synthetic fixture only.".into(),
        },
        Intent::CreateIssue {
            title: "Synthetic issue".into(),
            body: "Synthetic text only.".into(),
        },
        Intent::CommentIssue {
            number: 3,
            expected_id: 71,
            expected_snapshot: issue().fingerprint().unwrap(),
            body: "Reviewed synthetic comment".into(),
        },
        Intent::CloseIssue {
            number: 3,
            expected_id: 71,
            expected_snapshot: issue().fingerprint().unwrap(),
            reason: CloseReason::Completed,
        },
        Intent::CreatePullRequest {
            head_branch: "aster/topic".into(),
            base_branch: "engineering-target".into(),
            expected_head: head(),
            expected_base: base_sha(),
            title: "Synthetic PR".into(),
            body: "Synthetic diff only.".into(),
            draft: false,
        },
        Intent::MergePullRequest {
            number: 4,
            expected_id: 81,
            head_branch: "aster/topic".into(),
            base_branch: "engineering-target".into(),
            expected_head: head(),
            expected_base: base_sha(),
            commit_title: "Merge synthetic PR".into(),
            commit_message: "Synthetic verification only.".into(),
        },
    ]
}
fn expected_payload(operation: Operation) -> serde_json::Value {
    match operation {
        Operation::CreateBranch => serde_json::json!({"ref":"refs/heads/aster/new","sha":head()}),
        Operation::CreateFile => {
            serde_json::json!({"branch":"aster/topic","content":"bmV3IHN5bnRoZXRpYyBjb250ZW50","message":"Change synthetic file\n\nSynthetic fixture only."})
        }
        Operation::UpdateFile => {
            serde_json::json!({"branch":"aster/topic","content":"bmV3IHN5bnRoZXRpYyBjb250ZW50","message":"Change synthetic file\n\nSynthetic fixture only.","sha":blob_sha()})
        }
        Operation::CreateIssue => {
            serde_json::json!({"title":"Synthetic issue","body":"Synthetic text only."})
        }
        Operation::CommentIssue => serde_json::json!({"body":"Reviewed synthetic comment"}),
        Operation::CloseIssue => serde_json::json!({"state":"closed","state_reason":"completed"}),
        Operation::CreatePullRequest => {
            serde_json::json!({"head":"aster/topic","base":"engineering-target","title":"Synthetic PR","body":"Synthetic diff only.","draft":false})
        }
        Operation::MergePullRequest => {
            serde_json::json!({"sha":head(),"merge_method":"merge","commit_title":"Merge synthetic PR","commit_message":"Synthetic verification only."})
        }
    }
}
fn readback(intent: &Intent) -> Readback {
    match intent {
        Intent::CreateBranch {
            new_branch,
            expected_head,
            ..
        } => Readback::Branch {
            binding: binding(),
            name: new_branch.clone(),
            head: expected_head.clone(),
        },
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
        } => Readback::File {
            binding: binding(),
            branch: branch.clone(),
            path: path.clone(),
            response_commit: commit_sha(),
            observed_commit: commit_sha(),
            parents: vec![expected_head.clone()],
            blob_sha: blob_sha(),
            mode: "100644".into(),
            content_hash: hash(content.as_bytes()),
            commit_message: commit_message_text(commit_title, commit_message),
        },
        Intent::CreateIssue { title, body } => {
            let mut created = issue();
            created.id = 101;
            created.number = 5;
            created.title = title.clone();
            created.body = body.clone();
            Readback::Issue {
                binding: binding(),
                response_id: 101,
                issue: created,
            }
        }
        Intent::CommentIssue {
            expected_id, body, ..
        } => Readback::Comment {
            binding: binding(),
            issue_id: *expected_id,
            response_id: 102,
            comment_id: 102,
            body_hash: hash(body.as_bytes()),
        },
        Intent::CloseIssue { reason, .. } => {
            let mut closed = issue();
            closed.open = false;
            Readback::ClosedIssue {
                binding: binding(),
                issue: closed,
                reason: *reason,
            }
        }
        Intent::CreatePullRequest { .. } => Readback::Pull {
            binding: binding(),
            response_id: 81,
            pull: pull(),
        },
        Intent::MergePullRequest {
            commit_title,
            commit_message,
            ..
        } => {
            let mut merged = pull();
            merged.open = false;
            Readback::Merge {
                binding: binding(),
                http_status: 200,
                merged: true,
                response_commit: commit_sha(),
                observed_commit: commit_sha(),
                observed_merge_commit: commit_sha(),
                observed_merged: true,
                parents: vec![base_sha(), head()],
                pull: merged,
                commit_title: commit_title.clone(),
                commit_message: commit_message.clone(),
            }
        }
    }
}
fn runtime() -> Runtime {
    Runtime {
        enabled: true,
        token_live: true,
        binding: Some(binding()),
    }
}
struct Checks {
    states: VecDeque<(Runtime, u64)>,
}
impl Checks {
    fn ordinary() -> Self {
        Self {
            states: VecDeque::from([(runtime(), 1_003), (runtime(), 1_004), (runtime(), 1_005)]),
        }
    }
}
impl Checkpoints for Checks {
    fn current(&mut self) -> (Runtime, u64) {
        self.states
            .pop_front()
            .expect("fixture checkpoint exhausted")
    }
}
struct Fake {
    operation: Operation,
    resource: Resource,
    outcome: Option<Observation>,
    calls: usize,
    panic_after_start: bool,
}
impl Fake {
    fn new(intent: &Intent, outcome: Observation) -> Self {
        Self {
            operation: intent.operation(),
            resource: intent.resource(),
            outcome: Some(outcome),
            calls: 0,
            panic_after_start: false,
        }
    }
}
impl FakeableTransport for Fake {
    fn send_one(&mut self, request: &FixedRequest) -> Observation {
        self.calls += 1;
        assert_eq!(
            self.calls, 1,
            "an exact confirmation permits one change only"
        );
        assert_eq!(request.operation, self.operation);
        assert_eq!(request.resource, self.resource);
        let canonical_expected = serde_json::to_vec(&expected_payload(self.operation)).unwrap();
        assert_eq!(request.payload.as_slice(), canonical_expected.as_slice());
        let payload: serde_json::Value = serde_json::from_slice(&request.payload).unwrap();
        assert_eq!(payload, expected_payload(self.operation));
        assert!(
            payload.get("token").is_none()
                && payload.get("force").is_none()
                && payload.get("approved").is_none()
        );
        assert!(!self.panic_after_start, "synthetic interrupted dispatch");
        self.outcome.take().expect("fixture request replay")
    }
}
fn permit<B: RecordBackend>(
    book: &mut PlanBook,
    ledger: &Ledger<B>,
    intent: &Intent,
) -> DispatchPermit {
    let fresh = snapshot(intent);
    let id = book
        .prepare(intent.clone(), fresh.clone(), ledger, 1_000)
        .unwrap();
    let ticket = book.begin_review(&id, 1_001).unwrap();
    let native_hash = ticket.plan.review_hash.clone();
    book.finish_review(ticket, Some(&native_hash), fresh, 1_002)
        .unwrap()
}

#[test]
fn sha256_known_vectors_preserve_exact_lowercase_width_and_leading_zero_bytes() {
    for (input, expected) in [
        (
            b"".as_slice(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ),
        (
            b"abc".as_slice(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        ),
    ] {
        let actual = hash(input);
        assert_eq!(actual, expected);
        assert_eq!(actual.len(), 64);
        assert!(actual
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
    }
    // The independently known ASCII vector includes 0x01, 0x03 and 0x00;
    // their two-digit encodings must not lose any leading-zero nibble.
    let ascii = hash(b"abc");
    assert_eq!(&ascii[10..12], "01");
    assert_eq!(&ascii[34..36], "03");
    assert_eq!(&ascii[58..60], "00");
}

#[test]
fn owned_merge_indirection_preserves_exact_snapshot_bytes_hashes_and_deep_clone() {
    // This independent borrowed representation retains the original unboxed
    // Merge fields and their order; it is never a runtime evidence or grant DTO.
    #[derive(serde::Serialize)]
    enum UnboxedMerge<'a> {
        Merge {
            pull: &'a Pull,
            head: &'a Branch,
            base: &'a Branch,
            evidence: &'a MergeEvidence,
        },
    }
    let intent = intents().remove(7);
    let original = snapshot(&intent);
    let ResourceSnapshot::Merge {
        pull: original_pull,
        head,
        base,
        evidence,
    } = &original.resource
    else {
        panic!("synthetic merge snapshot expected");
    };
    let unboxed = UnboxedMerge::Merge {
        pull: original_pull.as_ref(),
        head,
        base,
        evidence,
    };
    let exact = serde_json::to_vec(&original.resource).unwrap();
    let prior_shape = serde_json::to_vec(&unboxed).unwrap();
    assert_eq!(exact, prior_shape);
    assert_eq!(hash(&exact), hash(&prior_shape));

    let original_title = original_pull.title.clone();
    let mut cloned = original.clone();
    assert!(cloned == original);
    let ResourceSnapshot::Merge { pull, .. } = &mut cloned.resource else {
        panic!("synthetic cloned merge snapshot expected");
    };
    pull.as_mut().title.push_str(" changed clone");
    assert!(cloned != original);
    assert_eq!(original_pull.title, original_title);
    assert_eq!(serde_json::to_vec(&original.resource).unwrap(), exact);
}

#[test]
fn all_eight_closed_operations_emit_one_exact_request_and_metadata_only_terminal() {
    for intent in intents() {
        let wire =
            serde_json::to_vec(&serde_json::json!({"schemaVersion":1,"change":intent})).unwrap();
        let parsed = Intent::parse(&wire).unwrap();
        let mut book = PlanBook::new(NONCE).unwrap();
        let mut ledger = Ledger::restore(Memory::default()).unwrap();
        let permit = permit(&mut book, &ledger, &parsed);
        let mut fake = Fake::new(&parsed, Observation::Readback(Box::new(readback(&parsed))));
        let result = book
            .dispatch_once(permit, &mut ledger, &mut fake, &mut Checks::ordinary())
            .unwrap();
        assert!(matches!(result, Terminal::Verified(_)));
        assert_eq!(fake.calls, 1);
        let memory = ledger.into_backend();
        assert_eq!(memory.records.len(), 2);
        for (_, record) in &memory.records {
            let record = std::str::from_utf8(record).unwrap();
            for canary in [
                "PRIVATE_REVIEW_CANARY",
                "Synthetic text only.",
                "new synthetic content",
                "exactPayload",
                "access_token",
                "refresh_token",
                "PROFILE_CANARY",
                "approved",
            ] {
                assert!(
                    !record.contains(canary),
                    "private payload/authority leaked into metadata"
                );
            }
        }
        let restored = Ledger::restore(memory).unwrap();
        restored.admit(&binding()).unwrap(); // Historical terminal grants nothing.
        assert!(book.pending.is_none() && book.active_id.is_none());
        assert!(matches!(
            book.begin_review("unrestored-plan", 1_006),
            Err(Error::Replay)
        ));
    }
}

#[test]
fn forged_fields_duplicates_null_types_unicode_and_size_never_prepare() {
    for wire in [
        r#"{"schemaVersion":1,"change":{"operation":"create_issue","title":"x","body":"y","approved":true}}"#,
        r#"{"schemaVersion":1,"schemaVersion":1,"change":{"operation":"create_issue","title":"x","body":"y"}}"#,
        r#"{"schemaVersion":1,"change":{"operation":"create_issue","title":"x","title":"x","body":"y"}}"#,
        r#"{"schemaVersion":1,"change":{"operation":"create_issue","title":null,"body":"y"}}"#,
        r#"{"schemaVersion":1,"change":{"operation":"create_issue","title":"\ud800","body":"y"}}"#,
        r#"{"schemaVersion":1,"change":{"operation":"create_issue","title":"x","body":"y","endpoint":"https://evil.invalid"}}"#,
        r#"{"schemaVersion":1,"change":{"operation":"create_issue","title":"x","body":"y","token":"fake"}}"#,
        r#"{"schemaVersion":1,"change":{"operation":"delete_repository"}}"#,
        r#"{"schemaVersion":2,"change":{"operation":"create_issue","title":"x","body":"y"}}"#,
        "[]",
    ] {
        assert!(Intent::parse(wire.as_bytes()).is_err());
    }
    assert!(matches!(
        Intent::parse(&vec![b' '; MAX_INTENT_BYTES + 1]),
        Err(Error::Bounds)
    ));
    assert!(Intent::CreateIssue {
        title: "é".repeat(257),
        body: String::new()
    }
    .validate()
    .is_err());
    assert!(Intent::CreateIssue {
        title: "x".into(),
        body: "😀".repeat(4_001)
    }
    .validate()
    .is_err());
    assert!(Intent::CreateIssue {
        title: "spoof\u{202e}".into(),
        body: String::new()
    }
    .validate()
    .is_err());
}

#[test]
fn unsafe_paths_refs_modes_aliases_symlinks_and_default_file_branch_deny() {
    for value in [
        "../file",
        "/file",
        "a\\b",
        "a%2fb",
        "a:b",
        ".git/config",
        ".github/workflows/ci.yml",
        ".env",
        "keys.pem",
        "credentials.json",
        "CON.txt",
        "a/../b",
        "readme.md.",
        "a//b",
    ] {
        assert!(!path(value));
    }
    for value in [
        "HEAD",
        "refs/heads/main",
        "a..b",
        "a@{b}",
        "-topic",
        "topic.lock",
        "a//b",
    ] {
        assert!(!branch(value));
    }
    let intent = intents().remove(2);
    for mode in ["100755", "120000", "160000"] {
        let mut value = snapshot(&intent);
        if let ResourceSnapshot::File { tree, .. } = &mut value.resource {
            tree.entries.items[0].mode = mode.into();
            if mode == "160000" {
                tree.entries.items[0].kind = "commit".into();
            }
        }
        assert!(validate_snapshot(&intent, &value).is_err());
    }
    let mut value = snapshot(&intent);
    value.default_branch = "aster/topic".into();
    assert!(validate_snapshot(&intent, &value).is_err());
    let create = intents().remove(1);
    let mut value = snapshot(&create);
    if let ResourceSnapshot::File { tree, .. } = &mut value.resource {
        tree.entries = complete(vec![TreeEntry {
            path: "readme.md".into(),
            mode: "100644".into(),
            kind: "blob".into(),
            sha: blob_sha(),
        }]);
    }
    assert!(validate_snapshot(&create, &value).is_err());
    let mut value = snapshot(&intent);
    if let ResourceSnapshot::File { tree, .. } = &mut value.resource {
        tree.truncated = true;
    }
    assert!(validate_snapshot(&intent, &value).is_err());
}

#[test]
fn unrelated_sensitive_unicode_space_case_alias_and_special_mode_tree_data_is_not_a_target() {
    let intent = intents().remove(2);
    let mut value = snapshot(&intent);
    if let ResourceSnapshot::File { tree, .. } = &mut value.resource {
        for (name, mode, kind) in [
            (".github", "040000", "tree"),
            (".github/workflows", "040000", "tree"),
            (".github/workflows/ci.yml", "100644", "blob"),
            (".env", "100644", "blob"),
            ("credentials.json", "100644", "blob"),
            ("keys.pem", "100644", "blob"),
            ("CON.txt", "100644", "blob"),
            ("Documentación de prueba.txt", "100644", "blob"),
            ("Other.txt", "100644", "blob"),
            ("other.txt", "100644", "blob"),
            ("unrelated-script", "100755", "blob"),
            ("unrelated-symlink", "120000", "blob"),
            ("unrelated-submodule", "160000", "commit"),
        ] {
            tree.entries.items.push(TreeEntry {
                path: name.into(),
                mode: mode.into(),
                kind: kind.into(),
                sha: blob_sha(),
            });
        }
        tree.entries.total = tree.entries.items.len();
    }
    validate_snapshot(&intent, &value).unwrap();
    let mut book = PlanBook::new(NONCE).unwrap();
    let ledger = Ledger::restore(Memory::default()).unwrap();
    book.prepare(intent, value, &ledger, 100).unwrap();
    // This did not expand the mutation-path policy itself.
    for name in [".env", ".github/workflows/ci.yml", "keys.pem", "CON.txt"] {
        assert!(!path(name));
    }
}

#[test]
fn creation_and_update_deny_exact_or_aliased_target_descendants_not_unrelated_prefixes() {
    for index in [1, 2] {
        let intent = intents().remove(index);
        for spelling in [
            "README.md/child",
            "readme.md/child",
            "README.md./child",
            "README.md /child",
        ] {
            let mut value = snapshot(&intent);
            if let ResourceSnapshot::File { tree, .. } = &mut value.resource {
                tree.entries.items.push(TreeEntry {
                    path: spelling.into(),
                    mode: "100644".into(),
                    kind: "blob".into(),
                    sha: blob_sha(),
                });
                tree.entries.total += 1;
            }
            assert!(
                validate_snapshot(&intent, &value).is_err(),
                "operation {index}, contradictory target descendant {spelling}"
            );
        }
        let mut value = snapshot(&intent);
        if let ResourceSnapshot::File { tree, .. } = &mut value.resource {
            tree.entries.items.extend([
                TreeEntry {
                    path: "README.md-other".into(),
                    mode: "040000".into(),
                    kind: "tree".into(),
                    sha: blob_sha(),
                },
                TreeEntry {
                    path: "README.md-other/child".into(),
                    mode: "100644".into(),
                    kind: "blob".into(),
                    sha: blob_sha(),
                },
            ]);
            tree.entries.total += 2;
        }
        validate_snapshot(&intent, &value).unwrap();
    }
}

#[test]
fn fake_exact_request_guard_denies_semantically_identical_noncanonical_bytes() {
    let intent = intents().remove(3);
    let mut request = intent.request().unwrap();
    request.payload.push(b' ');
    let semantic: serde_json::Value = serde_json::from_slice(&request.payload).unwrap();
    assert_eq!(semantic, expected_payload(intent.operation()));
    let mut fake = Fake::new(&intent, Observation::Uncertain);
    assert!(catch_unwind(AssertUnwindSafe(|| fake.send_one(&request))).is_err());
    assert_eq!(fake.calls, 1);
}

#[test]
fn target_and_ancestor_aliases_or_modes_and_incomplete_remote_tree_still_deny() {
    let create = intents().remove(1);
    for spelling in ["readme.md", "README.md.", "README.md ", "readme.md./child"] {
        let mut value = snapshot(&create);
        if let ResourceSnapshot::File { tree, .. } = &mut value.resource {
            tree.entries = complete(vec![TreeEntry {
                path: spelling.into(),
                mode: "100644".into(),
                kind: "blob".into(),
                sha: blob_sha(),
            }]);
        }
        assert!(
            validate_snapshot(&create, &value).is_err(),
            "target alias {spelling}"
        );
    }
    let mut nested = create;
    if let Intent::CreateFile { path, .. } = &mut nested {
        *path = "docs/README.md".into();
    }
    let mut valid = snapshot(&nested);
    if let ResourceSnapshot::File { tree, .. } = &mut valid.resource {
        tree.entries = complete(vec![TreeEntry {
            path: "docs".into(),
            mode: "040000".into(),
            kind: "tree".into(),
            sha: blob_sha(),
        }]);
    }
    validate_snapshot(&nested, &valid).unwrap();
    for alias in ["DOCS", "docs.", "docs ", "DOCS/other"] {
        let mut value = valid.clone();
        if let ResourceSnapshot::File { tree, .. } = &mut value.resource {
            tree.entries.items.push(TreeEntry {
                path: alias.into(),
                mode: "040000".into(),
                kind: "tree".into(),
                sha: blob_sha(),
            });
            tree.entries.total += 1;
        }
        assert!(
            validate_snapshot(&nested, &value).is_err(),
            "ancestor alias {alias}"
        );
    }
    for (mode, kind) in [
        ("100644", "blob"),
        ("100755", "blob"),
        ("120000", "blob"),
        ("160000", "commit"),
    ] {
        let mut value = valid.clone();
        if let ResourceSnapshot::File { tree, .. } = &mut value.resource {
            tree.entries.items[0].mode = mode.into();
            tree.entries.items[0].kind = kind.into();
        }
        assert!(validate_snapshot(&nested, &value).is_err());
    }
    for malformed in [
        "a//b".into(),
        "../remote".into(),
        "a\\b".into(),
        "x".repeat(129),
        "x".repeat(513),
        "control\nname".into(),
    ] {
        let mut value = valid.clone();
        if let ResourceSnapshot::File { tree, .. } = &mut value.resource {
            tree.entries.items.push(TreeEntry {
                path: malformed,
                mode: "100644".into(),
                kind: "blob".into(),
                sha: blob_sha(),
            });
            tree.entries.total += 1;
        }
        assert!(validate_snapshot(&nested, &value).is_err());
    }
    for case in 0..4 {
        let mut value = valid.clone();
        if let ResourceSnapshot::File { tree, .. } = &mut value.resource {
            match case {
                0 => tree.entries.complete = false,
                1 => tree.entries.total += 1,
                2 => {
                    tree.entries.items.push(tree.entries.items[0].clone());
                    tree.entries.total += 1;
                }
                _ => tree.entries.items[0].mode = "unknown".into(),
            }
        }
        assert!(validate_snapshot(&nested, &value).is_err());
    }
}

#[test]
fn fresh_scope_identity_permission_privacy_resource_and_native_denial_burn_plans() {
    let intent = intents().remove(4);
    for mutation in 0..9 {
        let mut book = PlanBook::new(NONCE).unwrap();
        let ledger = Ledger::restore(Memory::default()).unwrap();
        let id = book
            .prepare(intent.clone(), snapshot(&intent), &ledger, 100)
            .unwrap();
        let ticket = book.begin_review(&id, 101).unwrap();
        let native_hash = ticket.plan.review_hash.clone();
        assert!(std::str::from_utf8(&ticket.plan.review)
            .unwrap()
            .contains("PRIVATE_REVIEW_CANARY"));
        let mut fresh = snapshot(&intent);
        match mutation {
            0 => fresh.enabled = false,
            1 => fresh.token_live = false,
            2 => fresh.installed_binding = None,
            3 => {
                fresh.binding.generation += 1;
                fresh.installed_binding = Some(fresh.binding.clone());
            }
            4 => {
                fresh.binding.private = false;
                fresh.installed_binding = Some(fresh.binding.clone());
            }
            5 => {
                fresh.binding.name = "renamed".into();
                fresh.installed_binding = Some(fresh.binding.clone());
            }
            6 => fresh.permissions.issues = Access::Read,
            7 => {
                if let ResourceSnapshot::Issue(issue) = &mut fresh.resource {
                    issue.body.push_str(" concurrent change");
                }
            }
            _ => {}
        }
        let native = if mutation == 8 {
            None
        } else {
            Some(native_hash.as_str())
        };
        assert!(book.finish_review(ticket, native, fresh, 102).is_err());
        assert!(matches!(book.begin_review(&id, 103), Err(Error::Replay)));
        assert_eq!(ledger.into_backend().append_calls, 0);
    }
}

#[test]
fn immutable_payload_hash_review_hash_and_wrong_id_cannot_be_replaced() {
    let intent = intents().remove(3);
    for mutate_payload in [false, true] {
        let mut book = PlanBook::new(NONCE).unwrap();
        let ledger = Ledger::restore(Memory::default()).unwrap();
        let id = book
            .prepare(intent.clone(), snapshot(&intent), &ledger, 100)
            .unwrap();
        let mut ticket = book.begin_review(&id, 101).unwrap();
        let native_hash = ticket.plan.review_hash.clone();
        if mutate_payload {
            ticket.plan.request.payload.push(b' ');
        } else {
            ticket.plan.review.push(b' ');
        }
        assert!(book
            .finish_review(ticket, Some(&native_hash), snapshot(&intent), 102)
            .is_err());
    }
    let mut book = PlanBook::new(NONCE).unwrap();
    let ledger = Ledger::restore(Memory::default()).unwrap();
    let id = book
        .prepare(intent.clone(), snapshot(&intent), &ledger, 100)
        .unwrap();
    assert!(matches!(
        book.begin_review("wrong-operation-plan", 101),
        Err(Error::ReviewMismatch)
    ));
    assert!(matches!(book.begin_review(&id, 102), Err(Error::Replay)));
}

#[test]
fn private_intent_path_snapshot_and_request_metadata_cannot_escape_exact_native_review() {
    let intent = intents().remove(1);
    for case in 0..4 {
        let mut book = PlanBook::new(NONCE).unwrap();
        let ledger = Ledger::restore(Memory::default()).unwrap();
        let id = book
            .prepare(intent.clone(), snapshot(&intent), &ledger, 100)
            .unwrap();
        let mut ticket = book.begin_review(&id, 101).unwrap();
        let native_hash = ticket.plan.review_hash.clone();
        match case {
            0 => {
                if let Intent::CreateFile { path, .. } = &mut ticket.plan.intent {
                    *path = "OTHER.md".into();
                }
                ticket.plan.request.resource = ticket.plan.intent.resource();
            }
            1 => {
                ticket.plan.snapshot.binding.private = false;
                ticket.plan.snapshot.installed_binding = Some(ticket.plan.snapshot.binding.clone());
            }
            2 => ticket.plan.request.operation = Operation::UpdateFile,
            _ => {
                ticket.plan.request.resource = Resource::File {
                    branch: "aster/topic".into(),
                    path: "OTHER.md".into(),
                }
            }
        }
        // Cases zero/one can preserve the exact HTTP payload bytes. Rebuilding
        // the native document and separately checking route metadata deny them.
        let fresh = ticket.plan.snapshot.clone();
        assert!(book
            .finish_review(ticket, Some(&native_hash), fresh, 102)
            .is_err());
        assert!(matches!(book.begin_review(&id, 103), Err(Error::Replay)));
        assert_eq!(ledger.into_backend().append_calls, 0);
    }
}

#[test]
fn expiry_rollback_cancellation_sequence_exhaustion_and_review_drain_are_closed() {
    let intent = intents().remove(3);
    let ledger = Ledger::restore(Memory::default()).unwrap();
    let mut book = PlanBook::new(NONCE).unwrap();
    let id = book
        .prepare(intent.clone(), snapshot(&intent), &ledger, 100)
        .unwrap();
    assert!(matches!(
        book.begin_review(&id, 100 + MAX_PLAN_MS),
        Err(Error::Expired)
    ));
    let mut book = PlanBook::new(NONCE).unwrap();
    let id = book
        .prepare(intent.clone(), snapshot(&intent), &ledger, 100)
        .unwrap();
    let ticket = book.begin_review(&id, 101).unwrap();
    let native_hash = ticket.plan.review_hash.clone();
    assert!(matches!(book.tick(99), Err(Error::ClockRollback)));
    assert!(book
        .finish_review(ticket, Some(&native_hash), snapshot(&intent), 102)
        .is_err());
    let mut book = PlanBook::new(NONCE).unwrap();
    let id = book
        .prepare(intent.clone(), snapshot(&intent), &ledger, 100)
        .unwrap();
    let ticket = book.begin_review(&id, 101).unwrap();
    let native_hash = ticket.plan.review_hash.clone();
    book.invalidate().unwrap();
    assert!(matches!(
        book.prepare(intent.clone(), snapshot(&intent), &ledger, 102),
        Err(Error::Busy)
    ));
    assert!(book
        .finish_review(ticket, Some(&native_hash), snapshot(&intent), 103)
        .is_err());
    book.prepare(intent.clone(), snapshot(&intent), &ledger, 104)
        .unwrap();
    book.epoch = u64::MAX;
    assert!(matches!(book.invalidate(), Err(Error::SequenceExhausted)));
    assert!(matches!(
        book.prepare(intent.clone(), snapshot(&intent), &ledger, 105),
        Err(Error::SequenceExhausted)
    ));
    assert!(PlanBook::new("caller-id").is_err());
}

#[test]
fn cancellation_after_intent_is_durable_unsent_and_no_transport_call() {
    let intent = intents().remove(3);
    let mut book = PlanBook::new(NONCE).unwrap();
    let mut ledger = Ledger::restore(Memory::default()).unwrap();
    let permit = permit(&mut book, &ledger, &intent);
    let mut disabled = runtime();
    disabled.enabled = false;
    let mut checks = Checks {
        states: VecDeque::from([(runtime(), 1_003), (disabled, 1_004)]),
    };
    let mut fake = Fake::new(&intent, Observation::Readback(Box::new(readback(&intent))));
    assert_eq!(
        book.dispatch_once(permit, &mut ledger, &mut fake, &mut checks)
            .unwrap(),
        Terminal::CancelledBeforeDispatch
    );
    assert_eq!(fake.calls, 0);
    let restored = Ledger::restore(ledger.into_backend()).unwrap();
    restored.admit(&binding()).unwrap();
}

#[test]
fn predispatch_append_failure_never_sends_and_visible_intent_restores_uncertainty() {
    let intent = intents().remove(3);
    for after_write in [false, true] {
        let mut book = PlanBook::new(NONCE).unwrap();
        let memory = Memory {
            fail_append: Some((1, after_write)),
            ..Memory::default()
        };
        let mut ledger = Ledger::restore(memory).unwrap();
        let permit = permit(&mut book, &ledger, &intent);
        let mut fake = Fake::new(&intent, Observation::Readback(Box::new(readback(&intent))));
        assert!(matches!(
            book.dispatch_once(permit, &mut ledger, &mut fake, &mut Checks::ordinary()),
            Err(Error::Ledger)
        ));
        assert_eq!(fake.calls, 0);
        assert!(ledger.admit(&binding()).is_err());
        let restored = Ledger::restore(ledger.into_backend()).unwrap();
        assert_eq!(restored.admit(&binding()).is_err(), after_write);
    }
}

#[test]
fn possible_write_timeout_late_old_account_and_lost_terminal_never_retry() {
    let intent = intents().remove(3);
    for case in 0..3 {
        let mut book = PlanBook::new(NONCE).unwrap();
        let memory = Memory {
            fail_append: if case == 2 { Some((2, false)) } else { None },
            ..Memory::default()
        };
        let mut ledger = Ledger::restore(memory).unwrap();
        let permit = permit(&mut book, &ledger, &intent);
        let mut fake = Fake::new(
            &intent,
            if case == 0 {
                Observation::Uncertain
            } else {
                Observation::Readback(Box::new(readback(&intent)))
            },
        );
        let mut checks = Checks::ordinary();
        if case == 1 {
            checks.states[2].0.binding.as_mut().unwrap().account_id += 1;
        }
        assert_eq!(
            book.dispatch_once(permit, &mut ledger, &mut fake, &mut checks)
                .unwrap(),
            Terminal::AttentionRequired
        );
        assert_eq!(fake.calls, 1);
        assert!(book
            .prepare(intent.clone(), snapshot(&intent), &ledger, 1_006)
            .is_err());
        let restored = Ledger::restore(ledger.into_backend()).unwrap();
        assert!(matches!(
            restored.admit(&binding()),
            Err(Error::Uncertainty)
        ));
        let mut another_account = binding();
        another_account.account_id += 1;
        assert!(restored.admit(&another_account).is_err());
    }
}

#[test]
fn unwind_after_intent_restores_only_uncertainty_not_plan_or_network_work() {
    let intent = intents().remove(3);
    let mut book = PlanBook::new(NONCE).unwrap();
    let mut ledger = Ledger::restore(Memory::default()).unwrap();
    let permit = permit(&mut book, &ledger, &intent);
    let mut fake = Fake::new(&intent, Observation::Uncertain);
    fake.panic_after_start = true;
    assert!(catch_unwind(AssertUnwindSafe(|| book.dispatch_once(
        permit,
        &mut ledger,
        &mut fake,
        &mut Checks::ordinary()
    )))
    .is_err());
    assert_eq!(fake.calls, 1);
    let restored = Ledger::restore(ledger.into_backend()).unwrap();
    assert!(matches!(
        restored.admit(&binding()),
        Err(Error::Uncertainty)
    ));
    let mut new_book = PlanBook::new("22222222222222222222222222222222").unwrap();
    assert!(matches!(
        new_book.prepare(intent.clone(), snapshot(&intent), &restored, 1),
        Err(Error::Uncertainty)
    ));
    assert!(new_book.pending.is_none() && new_book.active_id.is_none());
}

#[test]
fn definitive_no_change_and_provably_unstarted_are_distinct_not_success() {
    let intent = intents().remove(3);
    for (observation, expected) in [
        (Observation::DefinitiveNoChange, Terminal::RejectedNoChange),
        (
            Observation::ProvablyNotStarted,
            Terminal::CancelledBeforeDispatch,
        ),
    ] {
        let mut book = PlanBook::new(NONCE).unwrap();
        let mut ledger = Ledger::restore(Memory::default()).unwrap();
        let permit = permit(&mut book, &ledger, &intent);
        let mut fake = Fake::new(&intent, observation);
        assert_eq!(
            book.dispatch_once(permit, &mut ledger, &mut fake, &mut Checks::ordinary())
                .unwrap(),
            expected
        );
        assert_eq!(fake.calls, 1);
        Ledger::restore(ledger.into_backend())
            .unwrap()
            .admit(&binding())
            .unwrap();
    }
}

#[test]
fn create_pull_requires_contents_read_evidence_in_addition_to_pull_write() {
    let intent = intents().remove(6);
    for access in [Access::Read, Access::Write] {
        let mut value = snapshot(&intent);
        value.permissions.contents = access;
        validate_snapshot(&intent, &value).unwrap();
        let mut book = PlanBook::new(NONCE).unwrap();
        let ledger = Ledger::restore(Memory::default()).unwrap();
        let id = book
            .prepare(intent.clone(), value.clone(), &ledger, 100)
            .unwrap();
        let ticket = book.begin_review(&id, 101).unwrap();
        let native_hash = ticket.plan.review_hash.clone();
        let permit = book
            .finish_review(ticket, Some(&native_hash), value, 102)
            .unwrap();
        assert_eq!(permit.plan.request.operation, Operation::CreatePullRequest);
        assert_eq!(ledger.into_backend().append_calls, 0);
    }
    for access in [Access::Unknown, Access::None] {
        for private in [false, true] {
            let mut value = snapshot(&intent);
            value.permissions.contents = access;
            value.binding.private = private;
            value.installed_binding = Some(value.binding.clone());
            assert!(matches!(
                validate_snapshot(&intent, &value),
                Err(Error::Permission)
            ));
            let mut book = PlanBook::new(NONCE).unwrap();
            let ledger = Ledger::restore(Memory::default()).unwrap();
            assert!(matches!(
                book.prepare(intent.clone(), value, &ledger, 100),
                Err(Error::Permission)
            ));
            let id = book
                .prepare(intent.clone(), snapshot(&intent), &ledger, 101)
                .unwrap();
            let ticket = book.begin_review(&id, 102).unwrap();
            let native_hash = ticket.plan.review_hash.clone();
            let mut fresh = snapshot(&intent);
            fresh.permissions.contents = access;
            assert!(matches!(
                book.finish_review(ticket, Some(&native_hash), fresh, 103),
                Err(Error::Permission)
            ));
            assert!(matches!(book.begin_review(&id, 104), Err(Error::Replay)));
            assert_eq!(ledger.into_backend().append_calls, 0);
        }
    }
    let mut value = snapshot(&intent);
    value.permissions.pulls = Access::Read;
    assert!(matches!(
        validate_snapshot(&intent, &value),
        Err(Error::Permission)
    ));
}

#[test]
fn merge_pull_requires_pull_read_evidence_in_addition_to_contents_write() {
    let intent = intents().remove(7);
    for access in [Access::Read, Access::Write] {
        let mut value = snapshot(&intent);
        value.permissions.pulls = access;
        validate_snapshot(&intent, &value).unwrap();
        let mut book = PlanBook::new(NONCE).unwrap();
        let ledger = Ledger::restore(Memory::default()).unwrap();
        let id = book
            .prepare(intent.clone(), value.clone(), &ledger, 100)
            .unwrap();
        let ticket = book.begin_review(&id, 101).unwrap();
        let native_hash = ticket.plan.review_hash.clone();
        let permit = book
            .finish_review(ticket, Some(&native_hash), value, 102)
            .unwrap();
        assert_eq!(permit.plan.request.operation, Operation::MergePullRequest);
        assert_eq!(ledger.into_backend().append_calls, 0);
    }
    for access in [Access::Unknown, Access::None] {
        for private in [false, true] {
            let mut value = snapshot(&intent);
            value.permissions.pulls = access;
            value.binding.private = private;
            value.installed_binding = Some(value.binding.clone());
            assert!(matches!(
                validate_snapshot(&intent, &value),
                Err(Error::Permission)
            ));
            let mut book = PlanBook::new(NONCE).unwrap();
            let ledger = Ledger::restore(Memory::default()).unwrap();
            assert!(matches!(
                book.prepare(intent.clone(), value, &ledger, 100),
                Err(Error::Permission)
            ));
            let id = book
                .prepare(intent.clone(), snapshot(&intent), &ledger, 101)
                .unwrap();
            let ticket = book.begin_review(&id, 102).unwrap();
            let native_hash = ticket.plan.review_hash.clone();
            let mut fresh = snapshot(&intent);
            fresh.permissions.pulls = access;
            assert!(matches!(
                book.finish_review(ticket, Some(&native_hash), fresh, 103),
                Err(Error::Permission)
            ));
            assert!(matches!(book.begin_review(&id, 104), Err(Error::Replay)));
            assert_eq!(ledger.into_backend().append_calls, 0);
        }
    }
    let mut value = snapshot(&intent);
    value.permissions.contents = Access::Read;
    assert!(matches!(
        validate_snapshot(&intent, &value),
        Err(Error::Permission)
    ));
}

#[test]
fn protected_unknown_rules_gate_failures_current_checks_reviews_and_no_status_distinction() {
    let intent = intents().remove(7);
    for case in 0..17 {
        let mut fresh = snapshot(&intent);
        if let ResourceSnapshot::Merge {
            pull,
            base,
            evidence,
            ..
        } = &mut fresh.resource
        {
            match case {
                0 => base.protected = Some(true),
                1 => base.protected = None,
                2 => base.active_rules = complete(vec!["required_status_checks".into()]),
                3 => base.active_rules.complete = false,
                4 => pull.draft = true,
                5 => pull.stacked = None,
                6 => pull.queued = Some(true),
                7 => pull.mergeable = None,
                8 => pull.head_repository_id += 1,
                9 => evidence.checks.items[0].status = Execution::Queued,
                10 => evidence.suites.items[0].status = Execution::InProgress,
                11 => evidence.checks.items[0].conclusion = Conclusion::Skipped,
                12 => evidence.checks.complete = false,
                13 => evidence.reviews.items[0].state = ReviewState::ChangesRequested,
                14 => evidence.reviews.items[0].head = blob_sha(),
                15 => {
                    evidence.statuses = complete(vec![Status {
                        id: 96,
                        context: "pending-context".into(),
                        head: head(),
                        state: StatusState::Pending,
                    }])
                }
                _ => {
                    evidence.checks.items.push(evidence.checks.items[0].clone());
                    evidence.checks.total += 1;
                }
            }
        }
        assert!(
            validate_snapshot(&intent, &fresh).is_err(),
            "merge case {case} was not denied"
        );
    }
    let mut fresh = snapshot(&intent);
    if let ResourceSnapshot::Merge { evidence, .. } = &mut fresh.resource {
        evidence.checks = complete(vec![]);
        evidence.suites = complete(vec![]);
        evidence.reviews = complete(vec![]);
        evidence.allow_no_results = true;
    }
    validate_snapshot(&intent, &fresh).unwrap();
    if let ResourceSnapshot::Merge { evidence, .. } = &mut fresh.resource {
        evidence.combined_status = StatusState::Success;
    }
    assert!(
        validate_snapshot(&intent, &fresh).is_err(),
        "no contexts must not invent success"
    );
    let mut fresh = snapshot(&intent);
    fresh.binding.name = "Aster-desktop".into();
    fresh.installed_binding = Some(fresh.binding.clone());
    assert!(matches!(
        validate_snapshot(&intent, &fresh),
        Err(Error::GateUnavailable)
    ));
}

#[test]
fn every_non_success_check_status_and_unsupported_review_state_is_denied() {
    let intent = intents().remove(7);
    for conclusion in [
        Conclusion::Failure,
        Conclusion::Neutral,
        Conclusion::Skipped,
        Conclusion::Cancelled,
        Conclusion::TimedOut,
        Conclusion::ActionRequired,
        Conclusion::Unknown,
    ] {
        let mut fresh = snapshot(&intent);
        if let ResourceSnapshot::Merge { evidence, .. } = &mut fresh.resource {
            evidence.checks.items[0].conclusion = conclusion;
        }
        assert!(validate_snapshot(&intent, &fresh).is_err());
    }
    for state in [
        StatusState::Pending,
        StatusState::Failure,
        StatusState::Error,
    ] {
        let mut fresh = snapshot(&intent);
        if let ResourceSnapshot::Merge { evidence, .. } = &mut fresh.resource {
            evidence.statuses = complete(vec![Status {
                id: 96,
                context: "context".into(),
                head: head(),
                state,
            }]);
            evidence.combined_status = state;
        }
        assert!(validate_snapshot(&intent, &fresh).is_err());
    }
    for state in [ReviewState::Pending, ReviewState::Unknown] {
        let mut fresh = snapshot(&intent);
        if let ResourceSnapshot::Merge { evidence, .. } = &mut fresh.resource {
            evidence.reviews.items[0].state = state;
        }
        assert!(validate_snapshot(&intent, &fresh).is_err());
    }
    for state in [ReviewState::Commented, ReviewState::Dismissed] {
        let mut fresh = snapshot(&intent);
        if let ResourceSnapshot::Merge { evidence, .. } = &mut fresh.resource {
            evidence.reviews.items[0].state = state;
        }
        validate_snapshot(&intent, &fresh).unwrap();
    }
    for access in [Access::Unknown, Access::None] {
        let mut fresh = snapshot(&intent);
        fresh.permissions.checks = access;
        assert!(validate_snapshot(&intent, &fresh).is_err());
    }
}

#[test]
fn misleading_merge_responses_moved_base_and_unknown_readback_are_uncertain() {
    let intent = intents().remove(7);
    for case in 0..7 {
        let mut result = readback(&intent);
        if let Readback::Merge {
            http_status,
            merged,
            parents,
            observed_merged,
            observed_merge_commit,
            commit_message,
            pull,
            ..
        } = &mut result
        {
            match case {
                0 => *http_status = 202,
                1 => *merged = false,
                2 => parents[0] = blob_sha(),
                3 => *observed_merged = false,
                4 => *observed_merge_commit = blob_sha(),
                5 => commit_message.push_str(" changed"),
                _ => pull.id += 1,
            }
        }
        let mut book = PlanBook::new(NONCE).unwrap();
        let mut ledger = Ledger::restore(Memory::default()).unwrap();
        let permit = permit(&mut book, &ledger, &intent);
        let mut fake = Fake::new(&intent, Observation::Readback(Box::new(result)));
        assert_eq!(
            book.dispatch_once(permit, &mut ledger, &mut fake, &mut Checks::ordinary())
                .unwrap(),
            Terminal::AttentionRequired
        );
        assert_eq!(fake.calls, 1);
        assert!(Ledger::restore(ledger.into_backend())
            .unwrap()
            .admit(&binding())
            .is_err());
    }
}

#[test]
fn ledger_corruption_duplicate_records_sequence_gaps_and_capacity_fail_closed() {
    let intent = intents().remove(3);
    let mut book = PlanBook::new(NONCE).unwrap();
    let mut ledger = Ledger::restore(Memory::default()).unwrap();
    let permit = permit(&mut book, &ledger, &intent);
    let mut fake = Fake::new(&intent, Observation::Uncertain);
    book.dispatch_once(permit, &mut ledger, &mut fake, &mut Checks::ordinary())
        .unwrap();
    let source = ledger.into_backend();
    let mut duplicate = source.clone();
    duplicate.records.push(duplicate.records[0].clone());
    assert!(Ledger::restore(duplicate).is_err());
    let mut gap = source.clone();
    gap.records.remove(0);
    assert!(Ledger::restore(gap).is_err());
    let mut corrupt = source.clone();
    corrupt.records[0].1.push(b' ');
    assert!(Ledger::restore(corrupt).is_err());
    let mut forged = source.clone();
    let mut wire: serde_json::Value = serde_json::from_slice(&forged.records[0].1).unwrap();
    wire["token"] = serde_json::json!("fake");
    forged.records[0].1 = serde_json::to_vec(&wire).unwrap();
    assert!(Ledger::restore(forged).is_err());
    let capacity = Memory {
        records: vec![("0".repeat(32), b"{}".to_vec()); 513],
        ..Memory::default()
    };
    assert!(Ledger::restore(capacity).is_err());
}

#[test]
fn valid_terminal_json_with_wrong_operation_effect_shape_is_not_historical_success() {
    for intent in intents() {
        let mut book = PlanBook::new(NONCE).unwrap();
        let mut ledger = Ledger::restore(Memory::default()).unwrap();
        let permit = permit(&mut book, &ledger, &intent);
        let mut fake = Fake::new(&intent, Observation::Uncertain);
        book.dispatch_once(permit, &mut ledger, &mut fake, &mut Checks::ordinary())
            .unwrap();
        let mut source = ledger.into_backend();
        let effect = match intent.operation() {
            Operation::CreateBranch | Operation::CreateFile | Operation::UpdateFile => EffectIds {
                resource_id: Some(71),
                number: Some(3),
                commit: None,
            },
            Operation::CreateIssue | Operation::CreatePullRequest => EffectIds {
                resource_id: None,
                number: None,
                commit: Some(commit_sha()),
            },
            Operation::CommentIssue => EffectIds {
                resource_id: Some(71),
                number: Some(3),
                commit: None,
            },
            Operation::CloseIssue => EffectIds {
                resource_id: Some(72),
                number: Some(3),
                commit: None,
            },
            Operation::MergePullRequest => EffectIds {
                resource_id: Some(82),
                number: Some(4),
                commit: Some(commit_sha()),
            },
        };
        assert!(
            effect.valid(),
            "fixture must reach the operation-specific shape gate"
        );
        let wrong = Terminal::Verified(effect);
        let original = String::from_utf8(source.records[1].1.clone()).unwrap();
        let marker = "\"terminal\":\"attention_required\"";
        assert_eq!(original.matches(marker).count(), 1);
        source.records[1].1 = original
            .replace(
                marker,
                &format!("\"terminal\":{}", serde_json::to_string(&wrong).unwrap()),
            )
            .into_bytes();
        // Field order and JSON canonical encoding remain unchanged. This is a
        // valid terminal schema whose IDs are incompatible with its intent.
        assert!(Ledger::restore(source.clone()).is_err());
        source.records.pop();
        let wire: serde_json::Value = serde_json::from_slice(&source.records[0].1).unwrap();
        let operation_id = wire["operationId"].as_str().unwrap();
        let mut unresolved = Ledger::restore(source.clone()).unwrap();
        assert!(matches!(
            unresolved.finish(operation_id, wrong),
            Err(Error::Replay)
        ));
        assert_eq!(unresolved.into_backend().records.len(), 1);
    }
}

struct PriorRewrite {
    inner: Memory,
}
impl RecordBackend for PriorRewrite {
    fn records(&self) -> Result<Vec<(String, Vec<u8>)>, Error> {
        self.inner.records()
    }
    fn append(&mut self, name: &str, bytes: &[u8]) -> Result<(), Error> {
        self.inner.append(name, bytes)?;
        if self.inner.records.len() == 2 {
            let body = &mut self.inner.records[0].1;
            let marker = b"\"payloadHash\":\"";
            let at = body
                .windows(marker.len())
                .position(|part| part == marker)
                .unwrap()
                + marker.len();
            body[at] = if body[at] == b'a' { b'b' } else { b'a' };
        }
        Ok(())
    }
}

#[test]
fn canonical_same_length_prior_record_rewrite_poisons_current_ledger_without_retry() {
    let intent = intents().remove(3);
    let mut book = PlanBook::new(NONCE).unwrap();
    let mut ledger = Ledger::restore(PriorRewrite {
        inner: Memory::default(),
    })
    .unwrap();
    let permit = permit(&mut book, &ledger, &intent);
    let mut fake = Fake::new(&intent, Observation::Readback(Box::new(readback(&intent))));
    assert_eq!(
        book.dispatch_once(permit, &mut ledger, &mut fake, &mut Checks::ordinary())
            .unwrap(),
        Terminal::AttentionRequired
    );
    assert_eq!(fake.calls, 1);
    assert!(matches!(ledger.admit(&binding()), Err(Error::Ledger)));
    assert!(book
        .prepare(intent.clone(), snapshot(&intent), &ledger, 1_006)
        .is_err());
    let backend = ledger.into_backend();
    assert_eq!(backend.inner.append_calls, 2);
    let wire: serde_json::Value = serde_json::from_slice(&backend.inner.records[0].1).unwrap();
    assert!(digest(wire["metadata"]["payloadHash"].as_str().unwrap()));
    // The pure memory backend supplies no cryptographic at-rest integrity or
    // pre-restart hash anchor. Authentic encrypted native recovery remains a
    // distinct integration obligation, not a claim of this fixture.
}

#[test]
fn exactly_two_record_and_worst_case_byte_slots_are_reserved_before_dispatch() {
    let bytes = 4 * 1024 * 1024;
    let pair = 2 * 16 * 1024;
    reserve_capacity(510, bytes - pair).unwrap();
    for (count, used) in [
        (511, 0),
        (512, 0),
        (usize::MAX, 0),
        (0, bytes - pair + 1),
        (510, bytes - pair + 1),
        (0, bytes),
        (0, usize::MAX),
    ] {
        assert!(matches!(reserve_capacity(count, used), Err(Error::Bounds)));
    }
}

// Additive protected-backend fixtures. The original 27 core fixtures above
// remain unchanged. These native cases are authored for hosted Windows only;
// no local agent execution or owner store/credential access is implied.

#[test]
fn protected_memory_backend_preserves_exact_bytes_names_and_store_bounds() {
    use github_mutation_store::ProtectedBackend;
    let mut backend = ProtectedBackend::memory();
    let name = format!("{:032x}", 1);
    let bytes = b"{\"synthetic\":true}";
    backend.append(&name, bytes).unwrap();
    assert_eq!(
        backend.records().unwrap(),
        vec![(name.clone(), bytes.to_vec())]
    );
    assert_eq!(backend.append(&name, b"{}"), Err(Error::Ledger));
    for hostile in ["", "../x", "A", "github-mutations-v1", "a:stream"] {
        assert_eq!(backend.append(hostile, b"{}"), Err(Error::Ledger));
    }
    for hostile in [Vec::new(), vec![0xff], vec![b'x'; 16 * 1024 + 1]] {
        assert_eq!(
            backend.append(&format!("{:032x}", 2), &hostile),
            Err(Error::Ledger)
        );
    }
    assert_eq!(
        backend.records().unwrap(),
        vec![(name.clone(), bytes.to_vec())]
    );
    let other = crate::private_store::PrivateStore::memory(crate::private_store::Domain::Terms);
    assert!(other.records().unwrap().is_empty());

    let mut bytes_full = ProtectedBackend::memory();
    let maximum = vec![b'x'; 16 * 1024];
    for sequence in 1..=256 {
        bytes_full
            .append(&format!("{sequence:032x}"), &maximum)
            .unwrap();
    }
    let exact = bytes_full.records().unwrap();
    assert_eq!(exact.len(), 256);
    assert_eq!(
        exact.iter().map(|(_, body)| body.len()).sum::<usize>(),
        4 * 1024 * 1024
    );
    assert_eq!(
        bytes_full.append(&format!("{:032x}", 257), b"x"),
        Err(Error::Ledger)
    );
    assert_eq!(bytes_full.records().unwrap(), exact);

    let mut count_full = ProtectedBackend::memory();
    for sequence in 1..=512 {
        count_full
            .append(&format!("{sequence:032x}"), b"x")
            .unwrap();
    }
    assert_eq!(count_full.records().unwrap().len(), 512);
    assert_eq!(
        count_full.append(&format!("{:032x}", 513), b"x"),
        Err(Error::Ledger)
    );
    assert_eq!(count_full.records().unwrap().len(), 512);
    // Valid bounded UTF-8 backend bytes are not a canonical ledger by themselves.
    assert!(matches!(Ledger::restore(count_full), Err(Error::Ledger)));
}

fn canonical_mutation_fixture_records(verified: bool) -> Vec<(String, Vec<u8>)> {
    let intent = intents().remove(3);
    let mut book = PlanBook::new(NONCE).unwrap();
    let mut ledger = Ledger::restore(Memory::default()).unwrap();
    let permit = permit(&mut book, &ledger, &intent);
    let operation_id = ledger.begin(&permit.plan, 1_003).unwrap();
    if verified {
        let effect = verify_readback(&permit.plan, &readback(&intent)).unwrap();
        ledger
            .finish(&operation_id, Terminal::Verified(effect))
            .unwrap();
    }
    ledger.into_backend().records
}

fn assert_no_mutation_grant_restored<B: RecordBackend>(ledger: &Ledger<B>, verified: bool) {
    let mut fresh = PlanBook::new("22222222222222222222222222222222").unwrap();
    assert!(fresh.pending.is_none() && fresh.active_id.is_none());
    assert!(matches!(
        fresh.begin_review(&format!("{NONCE}-1"), 1),
        Err(Error::Replay)
    ));
    if verified {
        ledger.admit(&binding()).unwrap();
    } else {
        assert!(matches!(ledger.admit(&binding()), Err(Error::Uncertainty)));
        let intent = intents().remove(3);
        assert!(matches!(
            fresh.prepare(intent.clone(), snapshot(&intent), ledger, 2),
            Err(Error::Uncertainty)
        ));
        assert!(fresh.pending.is_none() && fresh.active_id.is_none());
    }
}

#[test]
fn protected_memory_backend_requires_canonical_ledger_replay_and_restores_no_grants() {
    use github_mutation_store::ProtectedBackend;
    for verified in [false, true] {
        let source = canonical_mutation_fixture_records(verified);
        let mut backend = ProtectedBackend::memory();
        for (name, body) in &source {
            backend.append(name, body).unwrap();
        }
        let ledger = Ledger::restore(backend).unwrap();
        assert_no_mutation_grant_restored(&ledger, verified);
        assert_eq!(ledger.into_backend().records().unwrap(), source);
    }
    for hostile in [b"{\"synthetic\":true}".to_vec(), {
        let mut canonical = canonical_mutation_fixture_records(false).remove(0).1;
        canonical.push(b' ');
        canonical
    }] {
        let mut backend = ProtectedBackend::memory();
        backend.append(&format!("{:032x}", 1), &hostile).unwrap();
        assert!(matches!(Ledger::restore(backend), Err(Error::Ledger)));
    }
}

struct ProtectedFailureFixture {
    backend: github_mutation_store::ProtectedBackend,
    calls: usize,
    fail_at: usize,
    after_write: bool,
}
impl RecordBackend for ProtectedFailureFixture {
    fn records(&self) -> Result<Vec<(String, Vec<u8>)>, Error> {
        self.backend.records()
    }
    fn append(&mut self, name: &str, bytes: &[u8]) -> Result<(), Error> {
        self.calls += 1;
        if self.calls == self.fail_at {
            self.backend.fail_next_append_for_test(self.after_write);
        }
        self.backend.append(name, bytes)
    }
}

#[test]
fn protected_memory_intent_append_failure_poison_never_dispatches_or_retries() {
    use github_mutation_store::ProtectedBackend;
    let intent = intents().remove(3);
    for after_write in [false, true] {
        let mut book = PlanBook::new(NONCE).unwrap();
        let mut ledger = Ledger::restore(ProtectedFailureFixture {
            backend: ProtectedBackend::memory(),
            calls: 0,
            fail_at: 1,
            after_write,
        })
        .unwrap();
        let permit = permit(&mut book, &ledger, &intent);
        let mut fake = Fake::new(&intent, Observation::Readback(Box::new(readback(&intent))));
        assert!(matches!(
            book.dispatch_once(permit, &mut ledger, &mut fake, &mut Checks::ordinary()),
            Err(Error::Ledger)
        ));
        assert_eq!(fake.calls, 0);
        assert!(matches!(ledger.admit(&binding()), Err(Error::Ledger)));
        assert!(book
            .prepare(intent.clone(), snapshot(&intent), &ledger, 1_006)
            .is_err());
        let backend = ledger.into_backend();
        assert_eq!(backend.calls, 1);
        assert_eq!(backend.records().unwrap().len(), usize::from(after_write));
        let restored = Ledger::restore(backend).unwrap();
        assert_eq!(restored.admit(&binding()).is_err(), after_write);
        assert_eq!(restored.into_backend().calls, 1);
    }
}

#[test]
fn protected_memory_lost_terminal_acknowledgement_poison_never_retries() {
    use github_mutation_store::ProtectedBackend;
    let intent = intents().remove(3);
    for after_write in [false, true] {
        let mut book = PlanBook::new(NONCE).unwrap();
        let mut ledger = Ledger::restore(ProtectedFailureFixture {
            backend: ProtectedBackend::memory(),
            calls: 0,
            fail_at: 2,
            after_write,
        })
        .unwrap();
        let permit = permit(&mut book, &ledger, &intent);
        let mut fake = Fake::new(&intent, Observation::Readback(Box::new(readback(&intent))));
        assert_eq!(
            book.dispatch_once(permit, &mut ledger, &mut fake, &mut Checks::ordinary())
                .unwrap(),
            Terminal::AttentionRequired
        );
        assert_eq!(fake.calls, 1);
        assert!(matches!(ledger.admit(&binding()), Err(Error::Ledger)));
        assert!(book
            .prepare(intent.clone(), snapshot(&intent), &ledger, 1_006)
            .is_err());
        let backend = ledger.into_backend();
        assert_eq!(backend.calls, 2);
        assert_eq!(
            backend.records().unwrap().len(),
            1 + usize::from(after_write)
        );
        // Recovery may accept a visible canonical verified terminal, never
        // re-dispatch or restore the old permit; an absent terminal is uncertain.
        let restored = Ledger::restore(backend).unwrap();
        assert_no_mutation_grant_restored(&restored, after_write);
        assert_eq!(restored.into_backend().calls, 2);
    }
}

#[cfg(windows)]
struct NativeMutationFixture {
    base: std::path::PathBuf,
    identity: crate::new_file::Identity,
}
#[cfg(windows)]
impl NativeMutationFixture {
    fn create() -> Self {
        use crate::journal_privacy::{object_identity, open_directory, ObjectKind};
        let opaque = crate::private_store::new_id().unwrap();
        assert!(hex(&opaque, 32));
        let base = std::env::temp_dir().join(format!("aster-mutation-fixture-{opaque}"));
        std::fs::create_dir(&base).unwrap();
        let handle = open_directory(&base).unwrap();
        let identity = object_identity(&handle, ObjectKind::Directory).unwrap();
        drop(handle);
        Self { base, identity }
    }
    fn record_path(&self, domain: &str, name: &str) -> std::path::PathBuf {
        assert!(matches!(domain, "github-mutations-v1" | "terms-v1"));
        assert!(hex(name, 32));
        self.base
            .join("com.comzulca.aster-desktop-private-v1")
            .join(domain)
            .join(format!("{name}.dpapi"))
    }
    fn cleanup(self) {
        use crate::journal_privacy::{object_identity, open_directory, ObjectKind};
        use std::os::windows::fs::MetadataExt;
        let temporary = std::env::temp_dir();
        assert_eq!(self.base.parent(), Some(temporary.as_path()));
        let name = self.base.file_name().unwrap().to_str().unwrap();
        assert!(hex(
            name.strip_prefix("aster-mutation-fixture-").unwrap(),
            32
        ));
        let metadata = std::fs::symlink_metadata(&self.base).unwrap();
        assert!(metadata.is_dir() && metadata.file_attributes() & 0x400 == 0);
        assert_eq!(
            std::fs::canonicalize(&self.base).unwrap(),
            std::fs::canonicalize(&temporary).unwrap().join(name)
        );
        let handle = open_directory(&self.base).unwrap();
        assert_eq!(
            object_identity(&handle, ObjectKind::Directory).unwrap(),
            self.identity
        );
        drop(handle);
        // All store/record/directory handles are already released. The exact
        // target is this fresh opaque root, not temp, owner data or a workspace.
        std::fs::remove_dir_all(&self.base).unwrap();
    }
}

#[cfg(windows)]
#[test]
fn hosted_native_mutation_ciphertext_roundtrip_replays_unmatched_and_verified_without_grants() {
    use github_mutation_store::ProtectedBackend;
    for verified in [false, true] {
        let fixture = NativeMutationFixture::create();
        let source = canonical_mutation_fixture_records(verified);
        {
            let mut backend = ProtectedBackend::open_synthetic_root(&fixture.base).unwrap();
            for (name, body) in &source {
                backend.append(name, body).unwrap();
                let disk = std::fs::read(fixture.record_path("github-mutations-v1", name)).unwrap();
                assert!(!disk.windows(body.len()).any(|part| part == body.as_slice()));
            }
            assert_eq!(backend.records().unwrap(), source);
            assert!(ProtectedBackend::open_synthetic_root(&fixture.base).is_err());
        }
        {
            let backend = ProtectedBackend::open_synthetic_root(&fixture.base).unwrap();
            let ledger = Ledger::restore(backend).unwrap();
            assert_no_mutation_grant_restored(&ledger, verified);
            assert_eq!(ledger.into_backend().records().unwrap(), source);
        }
        fixture.cleanup();
    }
}

#[cfg(windows)]
#[test]
fn hosted_native_mutation_foreign_domain_ciphertext_is_preserved_and_denied() {
    use crate::journal_privacy::{create_record_exact, OwnerSid};
    use github_mutation_store::ProtectedBackend;
    use std::io::Write;
    let fixture = NativeMutationFixture::create();
    let source = canonical_mutation_fixture_records(false);
    let (name, body) = &source[0];
    {
        let mut terms = crate::private_store::PrivateStore::open(
            &fixture.base,
            crate::private_store::Domain::Terms,
        )
        .unwrap();
        terms.append(name, body).unwrap();
    }
    let original = std::fs::read(fixture.record_path("terms-v1", name)).unwrap();
    {
        let backend = ProtectedBackend::open_synthetic_root(&fixture.base).unwrap();
        assert!(backend.records().unwrap().is_empty());
    }
    let foreign = fixture.record_path("github-mutations-v1", name);
    {
        let owner = OwnerSid::current_process().unwrap();
        let mut record = create_record_exact(&foreign, &owner).unwrap();
        record.write_all(&original).unwrap();
        record.sync_all().unwrap();
    }
    assert!(ProtectedBackend::open_synthetic_root(&fixture.base).is_err());
    assert_eq!(std::fs::read(&foreign).unwrap(), original);
    fixture.cleanup();
}

#[cfg(windows)]
#[test]
fn hosted_native_mutation_corrupt_ciphertext_and_raw_noncanonical_payload_are_not_evidence() {
    use github_mutation_store::ProtectedBackend;
    for corrupt_ciphertext in [false, true] {
        let fixture = NativeMutationFixture::create();
        let name = format!("{:032x}", 1);
        {
            let mut backend = ProtectedBackend::open_synthetic_root(&fixture.base).unwrap();
            backend.append(&name, b"{\"synthetic\":true}").unwrap();
        }
        let path = fixture.record_path("github-mutations-v1", &name);
        if corrupt_ciphertext {
            std::fs::write(&path, b"synthetic corrupt ciphertext").unwrap();
            assert!(ProtectedBackend::open_synthetic_root(&fixture.base).is_err());
            assert_eq!(
                std::fs::read(&path).unwrap(),
                b"synthetic corrupt ciphertext"
            );
        } else {
            let original = std::fs::read(&path).unwrap();
            let backend = ProtectedBackend::open_synthetic_root(&fixture.base).unwrap();
            assert!(matches!(Ledger::restore(backend), Err(Error::Ledger)));
            assert_eq!(std::fs::read(&path).unwrap(), original);
        }
        fixture.cleanup();
    }
}
