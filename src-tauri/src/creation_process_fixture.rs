//! Windows-only independent-process crash/restart fixtures for new-file creation.
//!
//! This module is compiled only into the Rust test binary. It deliberately
//! re-enters that exact test binary as a bounded child role; it does not add a
//! production argument, IPC command, credential path, provider call, or
//! general-purpose process tool.

use crate::native_review::{NativeApproval, ReviewDocument};
use crate::workspace::{CreationOutcome, Workspace, WorkspaceError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const TEST_NAME: &str =
    "creation_process_fixture::independent_process_creation_interruption_restart_is_conservative";
const ROLE_ENV: &str = "ASTER_CREATION_PROCESS_FIXTURE_ROLE";
const ROOT_ENV: &str = "ASTER_CREATION_PROCESS_FIXTURE_ROOT";
const STAGE_ENV: &str = "ASTER_CREATION_PROCESS_FIXTURE_STAGE";
const CHILD_PUBLISHER: &str = "publisher";
const CHILD_INSPECTOR: &str = "inspector";
const ROOT_PREFIX: &str = "aster-creation-process-test-";
const MARKER_NAME: &str = "stage-reached.signal";
const RESULT_NAME: &str = "inspection-result.json";
const TARGET_NAME: &str = "interrupted.txt";
const FOLLOWUP_NAME: &str = "followup.txt";
const JOURNAL_NAME: &str = "journal";
const CONTENT: &str = "independent process synthetic content\n";
const SENTINEL_CONTENT: &str = "owner sentinel must remain unchanged\n";
const CHILD_DEADLINE: Duration = Duration::from_secs(30);

static FIXTURE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum StopStage {
    Intent,
    Created,
    IdentityRecorded,
    Written,
    Flushed,
    Complete,
}

impl StopStage {
    const ALL: [Self; 6] = [
        Self::Intent,
        Self::Created,
        Self::IdentityRecorded,
        Self::Written,
        Self::Flushed,
        Self::Complete,
    ];

    fn as_str(self) -> &'static str {
        match self {
            Self::Intent => "intent",
            Self::Created => "created",
            Self::IdentityRecorded => "identity-recorded",
            Self::Written => "written",
            Self::Flushed => "flushed",
            Self::Complete => "complete",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|candidate| candidate.as_str() == value)
    }

    fn expected_record_count(self) -> usize {
        match self {
            Self::Intent | Self::Created => 1,
            Self::IdentityRecorded | Self::Written | Self::Flushed => 2,
            Self::Complete => 3,
        }
    }

    fn target_should_exist(self) -> bool {
        self != Self::Intent
    }

    fn recovery_should_verify(self) -> bool {
        self == Self::Complete
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DurableSnapshot {
    journal_records: BTreeMap<OsString, Vec<u8>>,
    target: Option<Vec<u8>>,
    sentinel: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
struct InspectionResult {
    process_id: u32,
    recovery: String,
    followup: String,
}

struct FixtureRoot {
    path: PathBuf,
}

impl FixtureRoot {
    fn create(stage: StopStage) -> Self {
        let mut failures = Vec::new();
        for base in [
            std::env::current_dir().ok(),
            std::env::var_os("LOCALAPPDATA").map(PathBuf::from),
            Some(std::env::temp_dir()),
        ]
        .into_iter()
        .flatten()
        {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time must follow the Unix epoch")
                .as_nanos();
            let counter = FIXTURE_COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = base.join(format!(
                "{ROOT_PREFIX}{}-{}-{nonce}-{counter}",
                std::process::id(),
                stage.as_str()
            ));
            if let Err(error) = std::fs::create_dir(&path) {
                failures.push(format!("{}: create {error}", path.display()));
                continue;
            }
            let workspace = Workspace::default();
            match workspace.set_native_root(path.clone()) {
                Ok(_) => {
                    drop(workspace);
                    return Self { path };
                }
                Err(error) => {
                    failures.push(format!("{}: root admission {error:?}", path.display()));
                    std::fs::remove_dir(&path)
                        .expect("rejected empty fixture root must be removable");
                }
            }
        }
        panic!("no eligible fixed local NTFS fixture root: {failures:?}");
    }
}

impl Drop for FixtureRoot {
    fn drop(&mut self) {
        if self.path.exists() {
            std::fs::remove_dir_all(&self.path)
                .expect("independent-process fixture must clean up after all handles close");
        }
    }
}

struct ChildGuard {
    child: Option<Child>,
}

impl ChildGuard {
    fn spawn(role: &str, root: &Path, stage: StopStage) -> Self {
        let child = Command::new(std::env::current_exe().expect("test executable path"))
            .args(["--exact", TEST_NAME, "--nocapture", "--test-threads=1"])
            .env(ROLE_ENV, role)
            .env(ROOT_ENV, root)
            .env(STAGE_ENV, stage.as_str())
            .env("ASTER_DISABLE_LIVE", "1")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn exact bounded child test role");
        Self { child: Some(child) }
    }

    fn child_mut(&mut self) -> &mut Child {
        self.child.as_mut().expect("child process still owned")
    }

    fn wait_for_marker(&mut self, marker: &Path) {
        let deadline = Instant::now() + CHILD_DEADLINE;
        loop {
            if marker.is_file() {
                return;
            }
            if let Some(status) = self
                .child_mut()
                .try_wait()
                .expect("query publisher child status")
            {
                let output = self.take_output();
                panic!(
                    "publisher exited before stage marker: status={status}; stdout={}; stderr={}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            if Instant::now() >= deadline {
                let output = self.terminate_in_place();
                panic!(
                    "publisher timed out before stage marker: stdout={}; stderr={}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn terminate(mut self) -> Output {
        self.terminate_in_place()
    }

    fn terminate_in_place(&mut self) -> Output {
        let mut child = self.child.take().expect("child process still owned");
        child.kill().expect("terminate publisher at selected stage");
        child
            .wait_with_output()
            .expect("collect terminated publisher output")
    }

    fn wait(mut self) -> Output {
        self.child
            .take()
            .expect("child process still owned")
            .wait_with_output()
            .expect("collect child process output")
    }

    fn take_output(&mut self) -> Output {
        self.child
            .take()
            .expect("child process still owned")
            .wait_with_output()
            .expect("collect exited child output")
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            if child.try_wait().ok().flatten().is_none() {
                let _ = child.kill();
            }
            let _ = child.wait();
        }
    }
}

#[test]
fn independent_process_creation_interruption_restart_is_conservative() {
    match std::env::var(ROLE_ENV).ok().as_deref() {
        Some(CHILD_PUBLISHER) => publisher_child(),
        Some(CHILD_INSPECTOR) => inspector_child(),
        Some(other) => panic!("unknown bounded child role: {other}"),
        None => parent_orchestrator(),
    }
}

fn parent_orchestrator() {
    for stage in StopStage::ALL {
        let fixture = FixtureRoot::create(stage);
        let marker = fixture.path.join(MARKER_NAME);
        let result = fixture.path.join(RESULT_NAME);
        let sentinel = fixture.path.join("sentinel.txt");
        std::fs::write(&sentinel, SENTINEL_CONTENT).expect("write synthetic sentinel");

        let mut publisher = ChildGuard::spawn(CHILD_PUBLISHER, &fixture.path, stage);
        let publisher_process_id = publisher.child_mut().id();
        assert_ne!(publisher_process_id, std::process::id());
        publisher.wait_for_marker(&marker);
        let marker_value = std::fs::read_to_string(&marker).expect("read durable stage marker");
        assert_eq!(
            marker_value,
            format!("{}\n{publisher_process_id}\n", stage.as_str())
        );
        let terminated = publisher.terminate();
        assert!(
            !terminated.status.success(),
            "publisher must be terminated, not finish normally"
        );

        let before = durable_snapshot(&fixture.path);
        assert_eq!(before.journal_records.len(), stage.expected_record_count());
        assert_eq!(before.target.is_some(), stage.target_should_exist());
        assert_eq!(before.sentinel, SENTINEL_CONTENT.as_bytes());
        if matches!(stage, StopStage::Flushed | StopStage::Complete) {
            assert_eq!(before.target.as_deref(), Some(CONTENT.as_bytes()));
        }

        let inspector = ChildGuard::spawn(CHILD_INSPECTOR, &fixture.path, stage);
        let inspected = inspector.wait();
        assert!(
            inspected.status.success(),
            "inspector failed: stdout={}; stderr={}",
            String::from_utf8_lossy(&inspected.stdout),
            String::from_utf8_lossy(&inspected.stderr)
        );
        let inspection: InspectionResult =
            serde_json::from_slice(&std::fs::read(&result).expect("read bounded inspector result"))
                .expect("parse bounded inspector result");
        assert_ne!(inspection.process_id, std::process::id());
        assert_ne!(inspection.process_id, publisher_process_id);
        let after = durable_snapshot(&fixture.path);
        assert_eq!(
            after, before,
            "recovery inspection must not mutate evidence"
        );
        assert!(!fixture.path.join(FOLLOWUP_NAME).exists());

        if stage.recovery_should_verify() {
            assert_eq!(inspection.recovery, "verified");
            assert_eq!(inspection.followup, "candidate-only");
        } else {
            assert_eq!(inspection.recovery, "attention-required");
            assert_eq!(inspection.followup, "attention-required");
        }
    }
}

fn publisher_child() {
    let root = required_fixture_root();
    let marker = root.join(MARKER_NAME);
    let stop = required_stage();
    let workspace = Workspace::default();
    workspace
        .set_native_root(root.clone())
        .expect("publisher admits exact synthetic root");
    let journal = root.join(JOURNAL_NAME);
    crate::journal_privacy::create_directory_exact(&journal)
        .expect("create new owner-private synthetic journal");
    let proposal = workspace
        .prepare_new_file_proposal(
            "process-fixture-conversation",
            "process-fixture-request",
            &proposal_json(TARGET_NAME, CONTENT),
            journal,
        )
        .expect("prepare bounded synthetic create-new candidate");
    let review = ReviewDocument::proposal(&proposal.diff).expect("bind exact synthetic review");
    let approval = NativeApproval::for_fixture(review.binding_sha256());

    let outcome = workspace.apply_new_file_candidate_with_test_stage(
        &proposal.proposal_id,
        approval,
        |current| {
            if current == stop.as_str() {
                persist_signal(&marker, current)?;
                loop {
                    std::thread::park_timeout(Duration::from_secs(60));
                }
            }
            Ok(())
        },
    );
    panic!("publisher unexpectedly returned before termination: {outcome:?}");
}

fn inspector_child() {
    let root = required_fixture_root();
    let result = root.join(RESULT_NAME);
    let workspace = Workspace::default();
    workspace
        .set_native_root(root.clone())
        .expect("inspector admits exact synthetic root");
    let recovery = match workspace
        .inspect_creation_recovery(root.join(JOURNAL_NAME))
        .expect("inspect exact synthetic journal")
    {
        CreationOutcome::Verified => "verified",
        CreationOutcome::AttentionRequired => "attention-required",
    };
    let followup = match workspace.prepare_new_file_proposal(
        "process-fixture-conversation",
        "process-fixture-followup",
        &proposal_json(FOLLOWUP_NAME, "must never be published\n"),
        root.join(JOURNAL_NAME),
    ) {
        Ok(candidate) => {
            assert_eq!(candidate.status, "candidate_only");
            "candidate-only"
        }
        Err(WorkspaceError::AttentionRequired) => "attention-required",
        Err(other) => panic!("unexpected follow-up classification: {other:?}"),
    };
    let encoded = serde_json::to_vec(&InspectionResult {
        process_id: std::process::id(),
        recovery: recovery.to_owned(),
        followup: followup.to_owned(),
    })
    .expect("encode bounded inspector result");
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(result)
        .expect("create inspector result exactly once");
    file.write_all(&encoded)
        .expect("write bounded inspector result");
    file.sync_all().expect("flush bounded inspector result");
}

fn proposal_json(relative_path: &str, content: &str) -> String {
    serde_json::json!({
        "version": 1,
        "operation": "create_new",
        "relativePath": relative_path,
        "expectedOldSha256": null,
        "newContent": content,
    })
    .to_string()
}

fn required_path(name: &str) -> PathBuf {
    std::env::var_os(name)
        .map(PathBuf::from)
        .unwrap_or_else(|| panic!("missing bounded fixture path: {name}"))
}

fn required_fixture_root() -> PathBuf {
    let root = required_path(ROOT_ENV);
    let filename = root
        .file_name()
        .and_then(|value| value.to_str())
        .expect("synthetic fixture root has a UTF-8 filename");
    assert!(
        filename.starts_with(ROOT_PREFIX),
        "child roles accept only a bounded synthetic fixture root"
    );
    root
}

fn required_stage() -> StopStage {
    let value = std::env::var(STAGE_ENV).expect("missing bounded fixture stage");
    StopStage::parse(&value).unwrap_or_else(|| panic!("invalid bounded fixture stage: {value}"))
}

fn persist_signal(path: &Path, stage: &str) -> Result<(), WorkspaceError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|_| WorkspaceError::Io)?;
    file.write_all(format!("{stage}\n{}\n", std::process::id()).as_bytes())
        .map_err(|_| WorkspaceError::Io)?;
    file.sync_all().map_err(|_| WorkspaceError::Io)
}

fn durable_snapshot(root: &Path) -> DurableSnapshot {
    let journal = root.join(JOURNAL_NAME);
    let mut journal_records = BTreeMap::new();
    for entry in std::fs::read_dir(&journal).expect("read synthetic journal directory") {
        let entry = entry.expect("read synthetic journal entry");
        assert!(entry
            .file_type()
            .expect("read journal entry type")
            .is_file());
        journal_records.insert(
            entry.file_name(),
            std::fs::read(entry.path()).expect("read immutable journal record"),
        );
    }
    let target_path = root.join(TARGET_NAME);
    let target = target_path
        .is_file()
        .then(|| std::fs::read(target_path).expect("read interrupted target"));
    DurableSnapshot {
        journal_records,
        target,
        sentinel: std::fs::read(root.join("sentinel.txt")).expect("read owner sentinel"),
    }
}
