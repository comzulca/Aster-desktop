//! Explicit owner-run, offline credential diagnostic (ADR-0005 / T-033).
//! No production store, renderer, provider, network or configurable target.

pub(crate) const FLAG: &str = "--aster-credential-gui-fixture";
const DIAGNOSTIC_PREFIX: &str = "--aster-credential";
#[cfg(any(windows, test))]
const NAMESPACE: &str = "AsterDesktop/MetaModelAPI/credential-gui-fixture/";
#[cfg(any(windows, test))]
pub(crate) const SYNTHETIC_KEY: &str = "LLM|0|synthetic-fixture-key-123";
#[cfg(windows)]
pub(crate) const SYNTHETIC_USERNAME: &str = "aster-synthetic-fixture";

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Invocation {
    NotRequested,
    Requested,
    Invalid,
}

pub(crate) fn invocation(arguments: &[String]) -> Invocation {
    if !arguments
        .iter()
        .any(|value| value.to_ascii_lowercase().starts_with(DIAGNOSTIC_PREFIX))
    {
        return Invocation::NotRequested;
    }
    if arguments.len() == 1 && arguments[0] == FLAG {
        Invocation::Requested
    } else {
        Invocation::Invalid
    }
}

// The field and constructors are private. There is no From<String>, serde,
// mutable name, argument/env override, or production target in this module.
#[cfg(any(windows, test))]
pub(crate) struct FixtureTarget(String);

// Only non-secret, already admitted fixture metadata is retained across native
// consent. This snapshot is host-only and never serialized or returned by IPC.
#[cfg(any(windows, test))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FixtureSnapshot {
    pub(crate) target_name: String,
    pub(crate) last_written: u64,
    pub(crate) persistence: u32,
}

#[cfg(any(windows, test))]
impl FixtureTarget {
    fn from_nonce(pid: u32, nanos: u128, counter: u64) -> Self {
        Self(format!("{NAMESPACE}{pid:08x}-{nanos:032x}-{counter:016x}"))
    }

    #[cfg(windows)]
    fn generate() -> Result<Self, FixtureError> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| FixtureError::Native)?
            .as_nanos();
        let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
        Ok(Self::from_nonce(std::process::id(), nanos, counter))
    }

    pub(crate) fn validated_name(&self) -> Option<&str> {
        let suffix = self.0.strip_prefix(NAMESPACE)?;
        if suffix.len() != 58 {
            return None;
        }
        for (index, byte) in suffix.bytes().enumerate() {
            if index == 8 || index == 41 {
                if byte != b'-' {
                    return None;
                }
            } else if !byte.is_ascii_digit() && !(b'a'..=b'f').contains(&byte) {
                return None;
            }
        }
        Some(&self.0)
    }
}

#[cfg(any(windows, test))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum FixtureError {
    Target,
    Native,
    UnexpectedEntry,
    ExpectedSynthetic,
    ExpectedCancellation,
    Changed,
}

#[cfg(any(windows, test))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Outcome {
    Completed,
    Denied,
}

#[cfg(any(windows, test))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Step {
    AddInControlPanel,
    RemoveInControlPanel,
    CancelNativeEntry,
    AcceptNativeEntry,
    DeclineDeletion,
    AcceptDeletion,
}

#[cfg(any(windows, test))]
trait Backend {
    fn instruction(&mut self, target: &FixtureTarget, step: Step) -> Result<bool, FixtureError>;
    fn present(&mut self, target: &FixtureTarget) -> Result<bool, FixtureError>;
    fn snapshot(&mut self, target: &FixtureTarget) -> Result<FixtureSnapshot, FixtureError>;
    fn prompt(&mut self, target: &FixtureTarget) -> Result<bool, FixtureError>;
    fn write(&mut self, target: &FixtureTarget) -> Result<(), FixtureError>;
    fn confirm_deletion(
        &mut self,
        target: &FixtureTarget,
        step: Step,
    ) -> Result<bool, FixtureError>;
    fn delete(
        &mut self,
        target: &FixtureTarget,
        expected: &FixtureSnapshot,
    ) -> Result<(), FixtureError>;
}

#[cfg(any(windows, test))]
fn expect_absent<B: Backend>(backend: &mut B, target: &FixtureTarget) -> Result<(), FixtureError> {
    if backend.present(target)? {
        return Err(FixtureError::UnexpectedEntry);
    }
    Ok(())
}

#[cfg(any(windows, test))]
fn expect_synthetic<B: Backend>(
    backend: &mut B,
    target: &FixtureTarget,
) -> Result<FixtureSnapshot, FixtureError> {
    if !backend.present(target)? {
        return Err(FixtureError::ExpectedSynthetic);
    }
    backend.snapshot(target)
}

#[cfg(any(windows, test))]
fn execute<B: Backend>(backend: &mut B, target: &FixtureTarget) -> Result<Outcome, FixtureError> {
    target.validated_name().ok_or(FixtureError::Target)?;
    expect_absent(backend, target)?;
    if !backend.instruction(target, Step::AddInControlPanel)? {
        return Ok(Outcome::Denied);
    }
    expect_synthetic(backend, target)?;
    if !backend.instruction(target, Step::RemoveInControlPanel)? {
        return Ok(Outcome::Denied);
    }
    expect_absent(backend, target)?;

    if !backend.instruction(target, Step::CancelNativeEntry)? {
        return Ok(Outcome::Denied);
    }
    if backend.prompt(target)? {
        return Err(FixtureError::ExpectedCancellation);
    }
    expect_absent(backend, target)?;
    if !backend.instruction(target, Step::AcceptNativeEntry)? {
        return Ok(Outcome::Denied);
    }
    if !backend.prompt(target)? {
        return Ok(Outcome::Denied);
    }
    expect_absent(backend, target)?;
    backend.write(target)?;
    let written = expect_synthetic(backend, target)?;

    // The denial stage never mutates, even if the owner accidentally presses
    // the affirmative button instead of the requested Cancel.
    if backend.confirm_deletion(target, Step::DeclineDeletion)? {
        return Err(FixtureError::ExpectedCancellation);
    }
    if expect_synthetic(backend, target)? != written {
        return Err(FixtureError::Changed);
    }
    if !backend.confirm_deletion(target, Step::AcceptDeletion)? {
        return Ok(Outcome::Denied);
    }
    if expect_synthetic(backend, target)? != written {
        return Err(FixtureError::Changed);
    }
    backend.delete(target, &written)?;
    expect_absent(backend, target)?;
    Ok(Outcome::Completed)
}

#[cfg(windows)]
mod windows {
    use super::*;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        MessageBoxW, IDCANCEL, IDOK, MB_DEFBUTTON2, MB_ICONINFORMATION, MB_OK, MB_OKCANCEL,
    };

    fn message(text: &str, consent: bool) -> Result<bool, FixtureError> {
        let text: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let caption: Vec<u16> = "Aster: OFFLINE synthetic credential test"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let buttons = if consent {
            MB_OKCANCEL | MB_DEFBUTTON2
        } else {
            MB_OK
        };
        // SAFETY: bounded internally generated text, live NUL-terminated
        // buffers, no owner WebView or production host is initialized.
        match unsafe {
            MessageBoxW(
                std::ptr::null_mut(),
                text.as_ptr(),
                caption.as_ptr(),
                buttons | MB_ICONINFORMATION,
            )
        } {
            IDOK => Ok(true),
            IDCANCEL => Ok(false),
            _ => Err(FixtureError::Native),
        }
    }

    struct NativeBackend;

    impl Backend for NativeBackend {
        fn instruction(
            &mut self,
            target: &FixtureTarget,
            step: Step,
        ) -> Result<bool, FixtureError> {
            let name = target.validated_name().ok_or(FixtureError::Target)?;
            let instruction = match step {
                Step::AddInControlPanel => "In Windows Credential Manager > Windows Credentials > Add a generic credential, add ONLY the synthetic target below with the supplied fake username and password. Never enter or change a real key. Return here and choose OK to validate; Cancel ends without automatic clean-up.",
                Step::RemoveInControlPanel => "The manual entry matches the expected synthetic value. Delete ONLY this synthetic target in Windows Credential Manager. Do not touch any other credential. Choose OK to verify absence; Cancel ends without automatic clean-up.",
                Step::CancelNativeEntry => "Absence verified. Choose OK to open a native entry dialogue. Cancel or close that dialogue; do not enter a key. This stage must end without saving anything.",
                Step::AcceptNativeEntry => "Native cancellation and absence verified. Choose OK to open another native dialogue. Enter ONLY the supplied fake password. Other values are rejected and cleared from memory; never use a real key.",
                _ => return Err(FixtureError::Native),
            };
            message(
                &format!("{instruction}\n\nFull synthetic target:\n{name}\n\nUsername: {SYNTHETIC_USERNAME}\nNON-SECRET fake password: {SYNTHETIC_KEY}\n\nNo request is sent to Meta."),
                true,
            )
        }

        fn present(&mut self, target: &FixtureTarget) -> Result<bool, FixtureError> {
            crate::credential::fixture_present(target).map_err(|_| FixtureError::Native)
        }

        fn snapshot(&mut self, target: &FixtureTarget) -> Result<FixtureSnapshot, FixtureError> {
            crate::credential::fixture_snapshot(target).map_err(|_| FixtureError::ExpectedSynthetic)
        }

        fn prompt(&mut self, target: &FixtureTarget) -> Result<bool, FixtureError> {
            crate::credential::fixture_prompt(target).map_err(|_| FixtureError::Native)
        }

        fn write(&mut self, target: &FixtureTarget) -> Result<(), FixtureError> {
            crate::credential::fixture_write(target).map_err(|_| FixtureError::Native)
        }

        fn confirm_deletion(
            &mut self,
            target: &FixtureTarget,
            step: Step,
        ) -> Result<bool, FixtureError> {
            let name = target.validated_name().ok_or(FixtureError::Target)?;
            let instruction = match step {
                Step::DeclineDeletion => "DENIAL TEST: choose Cancel or close. The synthetic credential must remain. Choosing OK fails this stage and does NOT delete anything.",
                Step::AcceptDeletion => "Denial verified: the synthetic credential remains intact. This separate confirmation allows deletion of ONLY the synthetic target below. OK deletes after revalidation; Cancel ends and preserves it.",
                _ => return Err(FixtureError::Native),
            };
            message(&format!("{instruction}\n\n{name}\n\nNo other target is modified; there is no automatic clean-up."), true)
        }

        fn delete(
            &mut self,
            target: &FixtureTarget,
            expected: &FixtureSnapshot,
        ) -> Result<(), FixtureError> {
            crate::credential::fixture_delete(target, expected).map_err(|_| FixtureError::Native)
        }
    }

    pub(super) fn run() -> (&'static str, i32) {
        let target = match FixtureTarget::generate() {
            Ok(target) => target,
            Err(_) => return ("synthetic_gui_failed_closed", 3),
        };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            execute(&mut NativeBackend, &target)
        }));
        let (classification, exit_code, summary) = match result {
            Ok(Ok(Outcome::Completed)) => (
                "synthetic_gui_completed",
                0,
                "All synthetic stages completed; final absence verified. This does not certify the production Tauri handler or a Meta connection.",
            ),
            Ok(Ok(Outcome::Denied)) => (
                "synthetic_gui_denied_no_cleanup",
                2,
                "Test declined or cancelled. There is no automatic clean-up. The supplied synthetic target may remain; do not delete ambiguous or unrelated entries.",
            ),
            _ => (
                "synthetic_gui_failed_closed",
                3,
                "Failed closed; review required. No automatic clean-up or data repair occurs. The supplied synthetic target may remain; do not delete ambiguous or unrelated entries.",
            ),
        };
        if let Some(name) = target.validated_name() {
            let _ = message(&format!("Result: {classification}\n\n{summary}\n\nSynthetic target for this test:\n{name}"), false);
        }
        (classification, exit_code)
    }
}

pub(crate) fn run() -> (&'static str, i32) {
    #[cfg(windows)]
    {
        windows::run()
    }
    #[cfg(not(windows))]
    {
        ("synthetic_gui_windows_only", 3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct FakeBackend {
        entry: Option<bool>,
        prompts: usize,
        writes: usize,
        deletes: usize,
        targets: Vec<String>,
        denied_step: Option<Step>,
        wrong_gui: bool,
        accept_cancel_prompt: bool,
        cancel_accept_prompt: bool,
        prompt_error: bool,
        decline_affirmative: bool,
        decline_delete_accept: bool,
        foreign_before_delete: bool,
        changed_before_delete: bool,
        generation: u64,
        presence_error: bool,
        panic_before_delete: bool,
    }

    impl FakeBackend {
        fn record(&mut self, target: &FixtureTarget) {
            self.targets
                .push(target.validated_name().unwrap().to_owned());
        }
    }

    impl Backend for FakeBackend {
        fn instruction(
            &mut self,
            target: &FixtureTarget,
            step: Step,
        ) -> Result<bool, FixtureError> {
            self.record(target);
            if self.denied_step == Some(step) {
                return Ok(false);
            }
            match step {
                Step::AddInControlPanel => {
                    self.entry = Some(!self.wrong_gui);
                    self.generation += 1;
                }
                Step::RemoveInControlPanel => self.entry = None,
                _ => (),
            }
            Ok(true)
        }
        fn present(&mut self, target: &FixtureTarget) -> Result<bool, FixtureError> {
            self.record(target);
            if self.presence_error {
                return Err(FixtureError::Native);
            }
            Ok(self.entry.is_some())
        }
        fn snapshot(&mut self, target: &FixtureTarget) -> Result<FixtureSnapshot, FixtureError> {
            self.record(target);
            if self.entry != Some(true) {
                return Err(FixtureError::ExpectedSynthetic);
            }
            Ok(FixtureSnapshot {
                target_name: target.0.clone(),
                last_written: self.generation,
                persistence: 2,
            })
        }
        fn prompt(&mut self, target: &FixtureTarget) -> Result<bool, FixtureError> {
            self.record(target);
            self.prompts += 1;
            if self.prompt_error {
                return Err(FixtureError::Native);
            }
            Ok(if self.prompts == 1 {
                self.accept_cancel_prompt
            } else {
                !self.cancel_accept_prompt
            })
        }
        fn write(&mut self, target: &FixtureTarget) -> Result<(), FixtureError> {
            self.record(target);
            assert!(self.entry.is_none());
            self.writes += 1;
            self.entry = Some(true);
            self.generation += 1;
            Ok(())
        }
        fn confirm_deletion(
            &mut self,
            target: &FixtureTarget,
            step: Step,
        ) -> Result<bool, FixtureError> {
            self.record(target);
            if step == Step::DeclineDeletion {
                return Ok(self.decline_affirmative);
            }
            if self.foreign_before_delete {
                self.entry = Some(false);
            }
            if self.changed_before_delete {
                self.generation += 1;
            }
            if self.panic_before_delete {
                panic!("synthetic injected interruption");
            }
            Ok(!self.decline_delete_accept)
        }
        fn delete(
            &mut self,
            target: &FixtureTarget,
            expected: &FixtureSnapshot,
        ) -> Result<(), FixtureError> {
            self.record(target);
            assert_eq!(self.entry, Some(true));
            assert_eq!(&self.snapshot(target)?, expected);
            self.deletes += 1;
            self.entry = None;
            Ok(())
        }
    }

    fn target() -> FixtureTarget {
        FixtureTarget::from_nonce(123, 456, 7)
    }

    #[test]
    fn exact_diagnostic_invocation_and_overrides_fail_closed() {
        let args = |values: &[&str]| {
            values
                .iter()
                .map(|value| (*value).to_owned())
                .collect::<Vec<_>>()
        };
        assert_eq!(invocation(&[]), Invocation::NotRequested);
        assert_eq!(invocation(&args(&[FLAG])), Invocation::Requested);
        for values in [
            vec![FLAG, "extra"],
            vec!["other", FLAG],
            vec!["--aster-credential-gui-fixture=override"],
            vec!["--aster-credential-gui-fixture-other"],
            vec!["--aster-credential-gui-fixtur"],
            vec!["--ASTER-CREDENTIAL-GUI-FIXTURE"],
            vec![FLAG, "--target=override"],
            vec![FLAG, "--password=override"],
            vec![FLAG, "--url=override"],
        ] {
            assert_eq!(invocation(&args(&values)), Invocation::Invalid);
        }
    }

    #[test]
    fn generated_target_is_opaque_closed_and_distinct() {
        let first = target();
        assert!(first.validated_name().unwrap().starts_with(NAMESPACE));
        assert_ne!(first.0, FixtureTarget::from_nonce(123, 456, 8).0);
        for value in [
            format!("{NAMESPACE}123"),
            format!("{}\0", first.0),
            first.0.to_uppercase(),
            first.0.replace('-', "/"),
            "unrelated-target".to_owned(),
        ] {
            assert!(FixtureTarget(value).validated_name().is_none());
        }
    }

    #[test]
    fn invalid_target_is_denied_before_any_backend_call() {
        let mut backend = FakeBackend::default();
        assert_eq!(
            execute(&mut backend, &FixtureTarget("unrelated-target".into())),
            Err(FixtureError::Target)
        );
        assert!(backend.targets.is_empty());
    }

    #[test]
    fn complete_journey_has_one_write_one_delete_and_only_generated_target() {
        let target = target();
        let mut backend = FakeBackend::default();
        assert_eq!(execute(&mut backend, &target), Ok(Outcome::Completed));
        assert_eq!(
            (backend.writes, backend.deletes, backend.prompts),
            (1, 1, 2)
        );
        assert!(backend.entry.is_none());
        assert!(backend.targets.iter().all(|value| value == &target.0));
    }

    #[test]
    fn any_preexisting_entry_blocks_all_mutation_and_prompts() {
        for existing in [true, false] {
            let mut backend = FakeBackend {
                entry: Some(existing),
                ..Default::default()
            };
            assert_eq!(
                execute(&mut backend, &target()),
                Err(FixtureError::UnexpectedEntry)
            );
            assert_eq!(
                (backend.writes, backend.deletes, backend.prompts),
                (0, 0, 0)
            );
            assert_eq!(backend.entry, Some(existing));
        }
    }

    #[test]
    fn raw_presence_error_is_not_absence() {
        let mut backend = FakeBackend {
            presence_error: true,
            ..Default::default()
        };
        assert_eq!(execute(&mut backend, &target()), Err(FixtureError::Native));
        assert_eq!((backend.writes, backend.deletes), (0, 0));
    }

    #[test]
    fn wrong_or_malformed_gui_value_is_left_untouched() {
        let mut backend = FakeBackend {
            wrong_gui: true,
            ..Default::default()
        };
        assert_eq!(
            execute(&mut backend, &target()),
            Err(FixtureError::ExpectedSynthetic)
        );
        assert_eq!((backend.writes, backend.deletes), (0, 0));
        assert_eq!(backend.entry, Some(false));
    }

    #[test]
    fn cancelled_instruction_never_automatically_cleans_up() {
        for step in [
            Step::AddInControlPanel,
            Step::RemoveInControlPanel,
            Step::CancelNativeEntry,
            Step::AcceptNativeEntry,
        ] {
            let mut backend = FakeBackend {
                denied_step: Some(step),
                ..Default::default()
            };
            assert_eq!(execute(&mut backend, &target()), Ok(Outcome::Denied));
            assert_eq!((backend.writes, backend.deletes), (0, 0));
            assert_eq!(
                backend.entry,
                if step == Step::RemoveInControlPanel {
                    Some(true)
                } else {
                    None
                }
            );
        }
    }

    #[test]
    fn unexpected_native_acceptance_during_cancel_stage_never_writes() {
        let mut backend = FakeBackend {
            accept_cancel_prompt: true,
            ..Default::default()
        };
        assert_eq!(
            execute(&mut backend, &target()),
            Err(FixtureError::ExpectedCancellation)
        );
        assert_eq!((backend.writes, backend.deletes), (0, 0));
    }

    #[test]
    fn cancelled_native_entry_never_writes() {
        let mut backend = FakeBackend {
            cancel_accept_prompt: true,
            ..Default::default()
        };
        assert_eq!(execute(&mut backend, &target()), Ok(Outcome::Denied));
        assert_eq!((backend.writes, backend.deletes), (0, 0));
    }

    #[test]
    fn native_error_or_rejected_input_never_writes() {
        let mut backend = FakeBackend {
            prompt_error: true,
            ..Default::default()
        };
        assert_eq!(execute(&mut backend, &target()), Err(FixtureError::Native));
        assert_eq!((backend.writes, backend.deletes), (0, 0));
    }

    #[test]
    fn unexpected_affirmative_during_denial_stage_preserves_fixture() {
        let mut backend = FakeBackend {
            decline_affirmative: true,
            ..Default::default()
        };
        assert_eq!(
            execute(&mut backend, &target()),
            Err(FixtureError::ExpectedCancellation)
        );
        assert_eq!((backend.writes, backend.deletes), (1, 0));
        assert_eq!(backend.entry, Some(true));
    }

    #[test]
    fn final_delete_denial_preserves_fixture() {
        let mut backend = FakeBackend {
            decline_delete_accept: true,
            ..Default::default()
        };
        assert_eq!(execute(&mut backend, &target()), Ok(Outcome::Denied));
        assert_eq!((backend.writes, backend.deletes), (1, 0));
        assert_eq!(backend.entry, Some(true));
    }

    #[test]
    fn changed_foreign_entry_blocks_deletion_without_cleanup() {
        let mut backend = FakeBackend {
            foreign_before_delete: true,
            ..Default::default()
        };
        assert_eq!(
            execute(&mut backend, &target()),
            Err(FixtureError::ExpectedSynthetic)
        );
        assert_eq!(backend.deletes, 0);
        assert_eq!(backend.entry, Some(false));
    }

    #[test]
    fn same_value_rewritten_during_consent_blocks_deletion() {
        let mut backend = FakeBackend {
            changed_before_delete: true,
            ..Default::default()
        };
        assert_eq!(execute(&mut backend, &target()), Err(FixtureError::Changed));
        assert_eq!(backend.deletes, 0);
        assert_eq!(backend.entry, Some(true));
    }

    #[test]
    fn interruption_has_no_drop_cleanup() {
        let mut backend = FakeBackend {
            panic_before_delete: true,
            ..Default::default()
        };
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| execute(
                &mut backend,
                &target()
            )))
            .is_err()
        );
        assert_eq!(backend.deletes, 0);
        assert_eq!(backend.entry, Some(true));
    }
}
