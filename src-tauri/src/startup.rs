//! Bounded initial presentation only (NFR-008 / TM-030 / T-077).
//! Shell readiness is a scheduling signal, never proof of native painting or
//! permission to read, disclose, spend, restore a grant or mutate user data.
use crate::render_trace::{NativePhase, RenderTrace};
use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, Mutex, MutexGuard},
    time::{Duration, Instant},
};
use tauri::{State, WebviewWindow};

const MIN_REVEAL_MS: u64 = 6_000;
const HARD_DEADLINE_MS: u64 = 10_000;
const MAX_TRACE_EVENTS: usize = 16;
const MAX_TRACE_ELAPSED_MS: u64 = 60_000;
const SHELL_PROTOCOL: u8 = 1;

#[derive(Debug)]
pub(crate) struct ShellIntent {
    protocol_version: u8,
}

#[derive(Debug, Serialize)]
pub(crate) struct ShellReceipt {
    accepted: bool,
    // Native visibility state only, not proof of compositor or page painting.
    visible: bool,
}

// Object-only, duplicate/unknown-field-denying IPC. No window label, URL,
// action, time, delay, asset path or approval can be supplied by the renderer.
impl<'de> Deserialize<'de> for ShellIntent {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ObjectOnly;
        impl<'de> serde::de::Visitor<'de> for ObjectOnly {
            type Value = ShellIntent;
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a closed shell-readiness object")
            }
            fn visit_map<M: serde::de::MapAccess<'de>>(
                self,
                map: M,
            ) -> Result<Self::Value, M::Error> {
                #[derive(Deserialize)]
                #[serde(deny_unknown_fields, rename_all = "camelCase")]
                struct Fields {
                    protocol_version: u8,
                }
                let fields =
                    Fields::deserialize(serde::de::value::MapAccessDeserializer::new(map))?;
                if fields.protocol_version != SHELL_PROTOCOL {
                    return Err(serde::de::Error::custom("incompatible_startup_protocol"));
                }
                Ok(ShellIntent {
                    protocol_version: fields.protocol_version,
                })
            }
        }
        deserializer.deserialize_map(ObjectOnly)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RevealReason {
    ShellReady,
    Deadline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Presentation {
    Waiting,
    Showing(RevealReason),
    EarlyShowFailed,
    VisibleRequested,
    DeadlineFailed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Setup,
    ShellReady,
    RevealReady,
    RevealDeadline,
    RevealAccepted,
    RevealFailed,
    HookCleanupFailed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PhaseRecord {
    phase: Phase,
    elapsed_ms: u64,
}

struct Policy {
    ready: bool,
    presentation: Presentation,
    // Closed, process-memory-only diagnostic events. No strings, document
    // contents, names, paths, URLs, identities, errors or credential state.
    trace: Vec<PhaseRecord>,
}

impl Policy {
    fn new() -> Self {
        Self {
            ready: false,
            presentation: Presentation::Waiting,
            trace: vec![PhaseRecord {
                phase: Phase::Setup,
                elapsed_ms: 0,
            }],
        }
    }

    fn record(&mut self, phase: Phase, elapsed_ms: u64) {
        if self.trace.len() < MAX_TRACE_EVENTS {
            let previous = self.trace.last().map_or(0, |event| event.elapsed_ms);
            self.trace.push(PhaseRecord {
                phase,
                elapsed_ms: elapsed_ms.min(MAX_TRACE_ELAPSED_MS).max(previous),
            });
        }
    }

    fn admit_shell(
        &mut self,
        caller: &str,
        protocol_version: u8,
        elapsed_ms: u64,
    ) -> Result<(), &'static str> {
        if caller != crate::MAIN_WINDOW {
            return Err("caller_denied");
        }
        if protocol_version != SHELL_PROTOCOL {
            return Err("incompatible_startup_protocol");
        }
        // Replays acknowledge the same intent, not another window action.
        // A late renderer cannot undo a deadline reveal or restart the timer.
        if !self.ready && self.presentation == Presentation::Waiting {
            self.ready = true;
            self.record(Phase::ShellReady, elapsed_ms);
        }
        Ok(())
    }

    fn begin_reveal(&mut self, elapsed_ms: u64) -> Option<RevealReason> {
        let reason = match self.presentation {
            Presentation::Waiting if elapsed_ms >= HARD_DEADLINE_MS => RevealReason::Deadline,
            Presentation::Waiting if self.ready && elapsed_ms >= MIN_REVEAL_MS => {
                RevealReason::ShellReady
            }
            // An unsuccessful early OS show has exactly one deadline attempt.
            // An accepted show is never repeated and the app is never re-hidden.
            Presentation::EarlyShowFailed if elapsed_ms >= HARD_DEADLINE_MS => {
                RevealReason::Deadline
            }
            _ => return None,
        };
        self.presentation = Presentation::Showing(reason);
        self.record(
            match reason {
                RevealReason::ShellReady => Phase::RevealReady,
                RevealReason::Deadline => Phase::RevealDeadline,
            },
            elapsed_ms,
        );
        Some(reason)
    }

    fn complete_reveal(&mut self, reason: RevealReason, accepted: bool, elapsed_ms: u64) {
        if self.presentation != Presentation::Showing(reason) {
            return;
        }
        self.presentation = match (accepted, reason) {
            (true, _) => Presentation::VisibleRequested,
            (false, RevealReason::ShellReady) => Presentation::EarlyShowFailed,
            (false, RevealReason::Deadline) => Presentation::DeadlineFailed,
        };
        self.record(
            if accepted {
                Phase::RevealAccepted
            } else {
                Phase::RevealFailed
            },
            elapsed_ms,
        );
    }
}

#[derive(Clone)]
pub(crate) struct StartupGate {
    started: Instant,
    policy: Arc<Mutex<Policy>>,
    render_trace: RenderTrace,
}

impl StartupGate {
    pub(crate) fn new(render_trace: RenderTrace) -> Self {
        let gate = Self {
            started: Instant::now(),
            policy: Arc::new(Mutex::new(Policy::new())),
            render_trace,
        };
        gate.render_trace.native(NativePhase::StartupGateCreated);
        gate
    }

    fn policy(&self) -> MutexGuard<'_, Policy> {
        // This isolated presentation state has no grants, data or finance.
        // Recover poison rather than making a startup failure invisible. Every
        // OS operation runs outside this lock; events remain bounded and closed.
        self.policy
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }

    fn elapsed_ms(&self) -> u64 {
        self.started.elapsed().as_millis().min(u64::MAX as u128) as u64
    }

    fn admit_shell(
        &self,
        caller: &str,
        protocol_version: u8,
        clock: impl FnOnce() -> u64,
    ) -> Result<(), &'static str> {
        let mut policy = self.policy();
        // Preserve the original receiver/argument evaluation order: the gate
        // clock is sampled after Policy acquisition, never before contention.
        let elapsed_ms = clock();
        let was_ready = policy.ready;
        policy.admit_shell(caller, protocol_version, elapsed_ms)?;
        if !was_ready && policy.ready {
            // Only Policy -> Trace nesting: the milestone cannot overtake the
            // serialised reveal decision, and a replay is not a new admission.
            self.render_trace.native(NativePhase::StartupShellAccepted);
        }
        Ok(())
    }

    fn begin_reveal(&self, clock: impl FnOnce() -> u64) -> Option<RevealReason> {
        let mut policy = self.policy();
        let reason = policy.begin_reveal(clock())?;
        self.render_trace.native(match reason {
            RevealReason::ShellReady => NativePhase::StartupRevealReady,
            RevealReason::Deadline => NativePhase::StartupRevealDeadline,
        });
        Some(reason)
    }

    fn dispatch_denied(&self, reason: RevealReason, clock: impl FnOnce() -> u64) {
        let mut policy = self.policy();
        let elapsed_ms = clock();
        self.render_trace.native(NativePhase::StartupDispatchDenied);
        policy.complete_reveal(reason, false, elapsed_ms);
    }

    fn reveal_if_due(&self, window: &WebviewWindow) -> Result<(), String> {
        if window.label() != crate::MAIN_WINDOW {
            return Err("caller_denied".into());
        }
        let Some(reason) = self.begin_reveal(|| self.elapsed_ms()) else {
            return Ok(());
        };
        let gate = self.clone();
        let owned_window = window.clone();
        // Use the owning UI thread: locked Tauri/Tao execute show synchronously
        // there. Preserve Tao's internal VISIBLE state while vetoing activation
        // only for this owned window during that single show call.
        if window
            .run_on_main_thread(move || {
                gate.render_trace.native(NativePhase::StartupShowStarted);
                let outcome = show_without_activation(&owned_window);
                gate.render_trace.native(outcome.observed_phase());
                use tauri::Manager;
                owned_window
                    .state::<crate::render_trace::RenderTrace>()
                    .native(if outcome.visible {
                        crate::render_trace::NativePhase::ParentVisible
                    } else {
                        crate::render_trace::NativePhase::VisibilityDenied
                    });
                crate::webview_render_observer::sample_on_ui_thread();
                let mut policy = gate.policy();
                policy.complete_reveal(reason, outcome.visible, gate.elapsed_ms());
                if outcome.cleanup_failed {
                    policy.record(Phase::HookCleanupFailed, gate.elapsed_ms());
                }
            })
            .is_err()
        {
            self.dispatch_denied(reason, || self.elapsed_ms());
            return Err("startup_visibility_unavailable".into());
        }
        Ok(())
    }

    pub(crate) fn start_monitor(&self, window: WebviewWindow) {
        let gate = self.clone();
        // This native deadline is independent of renderer timers, asset loading
        // and slow or failed status/history hydration. Existing Tokio only.
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep_until(tokio::time::Instant::from_std(
                gate.started + Duration::from_millis(MIN_REVEAL_MS),
            ))
            .await;
            let _ = gate.reveal_if_due(&window);
            tokio::time::sleep_until(tokio::time::Instant::from_std(
                gate.started + Duration::from_millis(HARD_DEADLINE_MS),
            ))
            .await;
            let _ = gate.reveal_if_due(&window);
        });
    }
}

struct ShowOutcome {
    visible: bool,
    cleanup_failed: bool,
}
impl ShowOutcome {
    fn observed_phase(&self) -> NativePhase {
        match (self.visible, self.cleanup_failed) {
            (true, false) => NativePhase::StartupShowVisible,
            (false, false) => NativePhase::StartupShowDenied,
            (true, true) => NativePhase::StartupShowVisibleCleanupFailed,
            (false, true) => NativePhase::StartupShowDeniedCleanupFailed,
        }
    }
}

// A fixed native CBT hook, not an input hook. In particular, a negative code,
// any unrelated event, another HWND and an absent target always pass through.
#[cfg(any(windows, test))]
fn veto_activation(code: i32, candidate: usize, target: usize, child: bool) -> bool {
    matches!(code, 5 | 9) && target != 0 && candidate != 0 && (candidate == target || child)
}

#[cfg(windows)]
fn show_without_activation(window: &WebviewWindow) -> ShowOutcome {
    windows_visibility::show(window)
}

#[cfg(not(windows))]
fn show_without_activation(_window: &WebviewWindow) -> ShowOutcome {
    // This product's native presentation contract is Windows-only.
    ShowOutcome {
        visible: false,
        cleanup_failed: false,
    }
}

#[cfg(windows)]
mod windows_visibility {
    use super::{veto_activation, ShowOutcome};
    use std::{cell::Cell, ptr};
    use tauri::WebviewWindow;
    use windows_sys::Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        System::Threading::GetCurrentThreadId,
        UI::WindowsAndMessaging::{
            CallNextHookEx, GetWindowThreadProcessId, IsChild, IsWindow, IsWindowVisible,
            SetWindowsHookExW, UnhookWindowsHookEx, HCBT_ACTIVATE, HCBT_SETFOCUS, HHOOK, WH_CBT,
        },
    };

    thread_local! {
        // No callback event, key, pointer, input, title or content is retained.
        // The sole target exists only while the owning UI thread calls show.
        static TARGET: Cell<usize> = const { Cell::new(0) };
    }

    unsafe extern "system" fn suppress_activation(
        code: i32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        let target = TARGET.try_with(Cell::get).unwrap_or(0);
        let candidate = wparam as HWND;
        let relevant = code == HCBT_ACTIVATE as i32 || code == HCBT_SETFOCUS as i32;
        // SAFETY: IsChild is queried only for a callback HWND and the fixed
        // current-thread main window; no pointer data in lparam is dereferenced.
        let child = relevant && target != 0 && unsafe { IsChild(target as HWND, candidate) != 0 };
        if veto_activation(code, wparam, target, child) {
            return 1;
        }
        // SAFETY: Forward the original callback unchanged. This is a thread-
        // local CBT hook, never a global/DLL, keyboard or mouse hook.
        unsafe { CallNextHookEx(ptr::null_mut(), code, wparam, lparam) }
    }

    struct ActivationGuard {
        hook: HHOOK,
    }

    impl ActivationGuard {
        fn install(window: HWND) -> Option<Self> {
            let mut process = 0;
            // SAFETY: The handle comes only from the authenticated main Tauri
            // window on its UI thread, not from IPC or a document. Verify that
            // the live handle belongs to this process and this exact OS thread.
            let thread = unsafe { GetWindowThreadProcessId(window, &mut process) };
            if window.is_null()
                || unsafe { IsWindow(window) } == 0
                || process != std::process::id()
                || thread == 0
                || thread != unsafe { GetCurrentThreadId() }
                || TARGET.with(Cell::get) != 0
            {
                return None;
            }
            TARGET.with(|target| target.set(window as usize));
            // SAFETY: Own UI thread only, module=NULL and a static callback in
            // this process. No hook is installed in any other process/thread.
            let hook = unsafe {
                SetWindowsHookExW(WH_CBT, Some(suppress_activation), ptr::null_mut(), thread)
            };
            if hook.is_null() {
                TARGET.with(|target| target.set(0));
                return None;
            }
            Some(Self { hook })
        }

        fn remove(&mut self) -> bool {
            if self.hook.is_null() {
                return true;
            }
            let hook = std::mem::replace(&mut self.hook, ptr::null_mut());
            // SAFETY: This is only the handle returned by our scoped install.
            // Retry cleanup at most once; clearing TARGET makes even a failed
            // removal inert and pass-through until the UI thread exits.
            let removed = unsafe { UnhookWindowsHookEx(hook) } != 0
                || unsafe { UnhookWindowsHookEx(hook) } != 0;
            TARGET.with(|target| target.set(0));
            removed
        }
    }

    impl Drop for ActivationGuard {
        fn drop(&mut self) {
            let _ = self.remove();
        }
    }

    pub(super) fn show(window: &WebviewWindow) -> ShowOutcome {
        let failed = || ShowOutcome {
            visible: false,
            cleanup_failed: false,
        };
        if window.label() != crate::MAIN_WINDOW {
            return failed();
        }
        let Ok(handle) = window.hwnd() else {
            return failed();
        };
        let handle = handle.0 as HWND;
        let Some(mut guard) = ActivationGuard::install(handle) else {
            // Never fall back to an activating unguarded show.
            return failed();
        };
        let accepted = window.show().is_ok();
        // SAFETY: Same owned UI-thread handle, no thread hop or event wait.
        let visible = accepted && unsafe { IsWindowVisible(handle) } != 0;
        let cleanup_failed = !guard.remove();
        ShowOutcome {
            visible,
            cleanup_failed,
        }
    }
}

#[tauri::command]
pub(crate) async fn startup_shell_ready(
    window: WebviewWindow,
    gate: State<'_, StartupGate>,
    input: ShellIntent,
) -> Result<ShellReceipt, String> {
    crate::trusted(&window)?;
    gate.admit_shell(window.label(), input.protocol_version, || gate.elapsed_ms())
        .map_err(str::to_owned)?;
    gate.reveal_if_due(&window)?;
    Ok(ShellReceipt {
        accepted: true,
        visible: window
            .is_visible()
            .map_err(|_| "startup_visibility_unavailable")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn native_phases(trace: &RenderTrace) -> Vec<String> {
        let value = serde_json::to_value(trace.snapshot_for_fixture()).unwrap();
        value["events"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["source"] == "native")
            .map(|event| event["phase"].as_str().unwrap().to_owned())
            .collect()
    }

    #[test]
    fn activation_veto_is_only_owned_window_focus_or_activation_not_input() {
        for code in [-1, 0, 1, 2, 3, 4, 6, 7, 8, 10, i32::MAX] {
            assert!(!veto_activation(code, 42, 42, true));
        }
        for code in [5, 9] {
            assert!(veto_activation(code, 42, 42, false));
            assert!(veto_activation(code, 43, 42, true));
            assert!(!veto_activation(code, 43, 42, false));
            assert!(!veto_activation(code, 42, 0, true));
            assert!(!veto_activation(code, 0, 42, true));
        }
    }

    #[test]
    fn shell_intent_is_versioned_object_only_and_denies_authority_fields() {
        assert!(serde_json::from_str::<ShellIntent>(r#"{"protocolVersion":1}"#).is_ok());
        for value in [
            "null",
            "[]",
            "[1]",
            "{}",
            r#"{"protocolVersion":0}"#,
            r#"{"protocolVersion":2}"#,
            r#"{"protocolVersion":"1"}"#,
            r#"{"protocolVersion":true}"#,
            r#"{"protocolVersion":1.5}"#,
            r#"{"protocolVersion":1,"protocolVersion":1}"#,
        ] {
            assert!(serde_json::from_str::<ShellIntent>(value).is_err());
        }
        for field in [
            "window", "label", "url", "path", "delay", "time", "approved", "force", "action",
        ] {
            let mut value = serde_json::json!({"protocolVersion":1});
            value[field] = serde_json::json!("forged");
            assert!(serde_json::from_value::<ShellIntent>(value).is_err());
        }
    }

    #[test]
    fn receipt_has_only_the_two_bounded_presentation_booleans() {
        assert_eq!(
            serde_json::to_value(ShellReceipt {
                accepted: true,
                visible: false
            })
            .unwrap(),
            serde_json::json!({"accepted":true,"visible":false})
        );
    }

    #[test]
    fn forged_caller_or_protocol_cannot_change_readiness_or_trace() {
        let mut policy = Policy::new();
        for caller in ["", "other", "MAIN", "main-child"] {
            assert_eq!(policy.admit_shell(caller, 1, 6_000), Err("caller_denied"));
        }
        assert_eq!(
            policy.admit_shell(crate::MAIN_WINDOW, 2, 6_000),
            Err("incompatible_startup_protocol")
        );
        assert!(!policy.ready);
        assert_eq!(policy.trace.len(), 1);
        assert_eq!(policy.begin_reveal(9_999), None);
    }

    #[test]
    fn early_readiness_waits_six_seconds_and_duplicate_ready_is_one_shot() {
        let mut policy = Policy::new();
        policy.admit_shell(crate::MAIN_WINDOW, 1, 100).unwrap();
        for now in [100, 1_000, 5_000, 5_999] {
            assert_eq!(policy.begin_reveal(now), None);
            policy.admit_shell(crate::MAIN_WINDOW, 1, now).unwrap();
        }
        assert_eq!(policy.trace.len(), 2);
        assert_eq!(policy.begin_reveal(6_000), Some(RevealReason::ShellReady));
        assert_eq!(policy.begin_reveal(6_000), None);
        assert_eq!(policy.begin_reveal(10_000), None);
        policy.complete_reveal(RevealReason::ShellReady, true, 6_001);
        policy.admit_shell(crate::MAIN_WINDOW, 1, 12_000).unwrap();
        assert_eq!(policy.begin_reveal(12_000), None);
        assert_eq!(policy.presentation, Presentation::VisibleRequested);
        assert_eq!(policy.trace.len(), 4);
    }

    #[test]
    fn late_readiness_reveals_without_waiting_for_hydration_or_new_timer() {
        let mut policy = Policy::new();
        assert_eq!(policy.begin_reveal(6_000), None);
        policy.admit_shell(crate::MAIN_WINDOW, 1, 8_000).unwrap();
        assert_eq!(policy.begin_reveal(8_000), Some(RevealReason::ShellReady));
        policy.complete_reveal(RevealReason::ShellReady, true, 8_001);
        assert_eq!(policy.begin_reveal(10_000), None);
    }

    #[test]
    fn deadline_shows_with_no_renderer_and_late_signal_cannot_rehide_or_reshow() {
        let mut policy = Policy::new();
        assert_eq!(policy.begin_reveal(9_999), None);
        assert_eq!(policy.begin_reveal(10_000), Some(RevealReason::Deadline));
        assert_eq!(policy.begin_reveal(10_000), None);
        policy.complete_reveal(RevealReason::Deadline, true, 10_001);
        policy.admit_shell(crate::MAIN_WINDOW, 1, 10_002).unwrap();
        assert_eq!(policy.begin_reveal(20_000), None);
        assert_eq!(policy.trace.len(), 3);
        assert!(!policy.ready);
    }

    #[test]
    fn deadline_wins_simultaneous_readiness_and_stale_completion_is_inert() {
        let mut policy = Policy::new();
        policy.admit_shell(crate::MAIN_WINDOW, 1, 10_000).unwrap();
        assert_eq!(policy.begin_reveal(10_000), Some(RevealReason::Deadline));
        policy.complete_reveal(RevealReason::ShellReady, true, 10_000);
        assert_eq!(
            policy.presentation,
            Presentation::Showing(RevealReason::Deadline)
        );
        policy.complete_reveal(RevealReason::Deadline, true, 10_001);
        policy.complete_reveal(RevealReason::Deadline, false, 10_002);
        assert_eq!(policy.presentation, Presentation::VisibleRequested);
    }

    #[test]
    fn failed_early_show_gets_exactly_one_bounded_deadline_attempt() {
        let mut policy = Policy::new();
        policy.admit_shell(crate::MAIN_WINDOW, 1, 20).unwrap();
        assert_eq!(policy.begin_reveal(6_000), Some(RevealReason::ShellReady));
        policy.complete_reveal(RevealReason::ShellReady, false, 6_001);
        for now in [6_002, 9_999] {
            assert_eq!(policy.begin_reveal(now), None);
        }
        assert_eq!(policy.begin_reveal(10_000), Some(RevealReason::Deadline));
        policy.complete_reveal(RevealReason::Deadline, false, 10_001);
        for now in [10_002, 20_000, u64::MAX] {
            assert_eq!(policy.begin_reveal(now), None);
            policy.admit_shell(crate::MAIN_WINDOW, 1, now).unwrap();
        }
        assert_eq!(policy.presentation, Presentation::DeadlineFailed);
        assert_eq!(policy.trace.len(), 6);
    }

    #[test]
    fn diagnostics_are_closed_capped_and_monotonic_even_for_delayed_fixtures() {
        let mut policy = Policy::new();
        policy.record(Phase::ShellReady, u64::MAX);
        for _ in 0..100 {
            policy.record(Phase::RevealFailed, 0);
        }
        assert_eq!(policy.trace.len(), MAX_TRACE_EVENTS);
        assert!(policy
            .trace
            .windows(2)
            .all(|pair| pair[0].elapsed_ms <= pair[1].elapsed_ms));
        assert!(policy
            .trace
            .iter()
            .all(|event| event.elapsed_ms <= MAX_TRACE_ELAPSED_MS));
        assert_eq!(policy.trace[1].elapsed_ms, MAX_TRACE_ELAPSED_MS);
    }

    #[test]
    fn real_policy_mutex_serialises_ready_deadline_race_without_native_execution() {
        let policy = Arc::new(Mutex::new(Policy::new()));
        let barrier = Arc::new(std::sync::Barrier::new(3));
        let workers: Vec<_> = [false, true]
            .into_iter()
            .map(|signal| {
                let state = policy.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    let mut state = state.lock().unwrap();
                    if signal {
                        state.admit_shell(crate::MAIN_WINDOW, 1, 10_000).unwrap();
                    }
                    state.begin_reveal(10_000)
                })
            })
            .collect();
        barrier.wait();
        let intents: Vec<_> = workers
            .into_iter()
            .filter_map(|worker| worker.join().unwrap())
            .collect();
        assert_eq!(intents, vec![RevealReason::Deadline]);
        let mut state = policy.lock().unwrap();
        state.complete_reveal(RevealReason::Deadline, true, 10_001);
        assert_eq!(state.begin_reveal(10_002), None);
    }

    #[test]
    fn native_shell_marker_records_only_first_admission_not_forgery_replay_or_late_ack() {
        let trace = RenderTrace::new();
        let gate = StartupGate::new(trace.clone());
        let clone = gate.clone();
        assert_eq!(
            native_phases(&trace),
            ["host_created", "startup_gate_created"]
        );
        for caller in ["", "other", "MAIN", "main-child"] {
            assert_eq!(clone.admit_shell(caller, 1, || 6_000), Err("caller_denied"));
        }
        assert_eq!(
            clone.admit_shell(crate::MAIN_WINDOW, 2, || 6_000),
            Err("incompatible_startup_protocol")
        );
        assert_eq!(
            native_phases(&trace),
            ["host_created", "startup_gate_created"]
        );
        clone.admit_shell(crate::MAIN_WINDOW, 1, || 100).unwrap();
        for now in [100, 1_000, 5_000, 5_999] {
            assert_eq!(gate.begin_reveal(|| now), None);
            gate.admit_shell(crate::MAIN_WINDOW, 1, || now).unwrap();
        }
        assert_eq!(clone.begin_reveal(|| 6_000), Some(RevealReason::ShellReady));
        assert_eq!(gate.begin_reveal(|| 6_000), None);
        gate.policy()
            .complete_reveal(RevealReason::ShellReady, true, 6_001);
        gate.admit_shell(crate::MAIN_WINDOW, 1, || 12_000).unwrap();
        assert_eq!(gate.begin_reveal(|| 12_000), None);
        assert_eq!(
            native_phases(&trace),
            [
                "host_created",
                "startup_gate_created",
                "startup_shell_accepted",
                "startup_reveal_ready",
            ]
        );
        assert_eq!(gate.policy().presentation, Presentation::VisibleRequested);
        assert_eq!(gate.policy().trace.len(), 4);
    }

    #[test]
    fn native_deadline_marker_does_not_invent_late_readiness_or_accept_stale_completion() {
        let trace = RenderTrace::new();
        let gate = StartupGate::new(trace.clone());
        assert_eq!(gate.begin_reveal(|| 9_999), None);
        assert_eq!(gate.begin_reveal(|| 10_000), Some(RevealReason::Deadline));
        gate.admit_shell(crate::MAIN_WINDOW, 1, || 10_001).unwrap();
        gate.policy()
            .complete_reveal(RevealReason::ShellReady, true, 10_001);
        assert_eq!(
            gate.policy().presentation,
            Presentation::Showing(RevealReason::Deadline)
        );
        assert!(!gate.policy().ready);
        gate.policy()
            .complete_reveal(RevealReason::Deadline, true, 10_002);
        assert_eq!(gate.begin_reveal(|| 20_000), None);
        assert_eq!(
            native_phases(&trace),
            [
                "host_created",
                "startup_gate_created",
                "startup_reveal_deadline"
            ]
        );
    }

    #[test]
    fn native_show_model_keeps_decision_entry_outcome_order_and_fixed_cleanup_alternatives() {
        for (visible, cleanup_failed, outcome_phase) in [
            (true, false, "startup_show_visible"),
            (false, false, "startup_show_denied"),
            (true, true, "startup_show_visible_cleanup_failed"),
            (false, true, "startup_show_denied_cleanup_failed"),
        ] {
            let trace = RenderTrace::new();
            let gate = StartupGate::new(trace.clone());
            gate.admit_shell(crate::MAIN_WINDOW, 1, || 100).unwrap();
            let reason = gate.begin_reveal(|| 6_000).unwrap();
            // Pure boundary model only: no Tauri window, hook or OS show.
            gate.render_trace.native(NativePhase::StartupShowStarted);
            let outcome = ShowOutcome {
                visible,
                cleanup_failed,
            };
            gate.render_trace.native(outcome.observed_phase());
            trace.native(if outcome.visible {
                NativePhase::ParentVisible
            } else {
                NativePhase::VisibilityDenied
            });
            {
                let mut policy = gate.policy();
                policy.complete_reveal(reason, outcome.visible, 6_001);
                if outcome.cleanup_failed {
                    policy.record(Phase::HookCleanupFailed, 6_001);
                }
            }
            assert_eq!(
                native_phases(&trace),
                [
                    "host_created",
                    "startup_gate_created",
                    "startup_shell_accepted",
                    "startup_reveal_ready",
                    "startup_show_started",
                    outcome_phase,
                    if visible {
                        "parent_visible"
                    } else {
                        "visibility_denied"
                    },
                ]
            );
            let value = serde_json::to_value(trace.snapshot_for_fixture()).unwrap();
            let events = value["events"].as_array().unwrap();
            assert!(events
                .windows(2)
                .all(|pair| pair[0]["elapsedMs"].as_u64().unwrap()
                    <= pair[1]["elapsedMs"].as_u64().unwrap()));
            assert_eq!(value["visibleAtMs"].is_number(), visible);
        }
    }

    #[test]
    fn native_early_failure_records_only_one_subsequent_deadline_attempt() {
        let trace = RenderTrace::new();
        let gate = StartupGate::new(trace.clone());
        gate.admit_shell(crate::MAIN_WINDOW, 1, || 20).unwrap();
        assert_eq!(gate.begin_reveal(|| 6_000), Some(RevealReason::ShellReady));
        gate.render_trace.native(NativePhase::StartupShowStarted);
        gate.render_trace.native(
            ShowOutcome {
                visible: false,
                cleanup_failed: false,
            }
            .observed_phase(),
        );
        gate.policy()
            .complete_reveal(RevealReason::ShellReady, false, 6_001);
        assert_eq!(gate.begin_reveal(|| 9_999), None);
        assert_eq!(gate.begin_reveal(|| 10_000), Some(RevealReason::Deadline));
        gate.render_trace.native(NativePhase::StartupShowStarted);
        gate.render_trace.native(
            ShowOutcome {
                visible: false,
                cleanup_failed: true,
            }
            .observed_phase(),
        );
        gate.policy()
            .complete_reveal(RevealReason::Deadline, false, 10_001);
        for now in [10_002, 20_000, u64::MAX] {
            assert_eq!(gate.begin_reveal(|| now), None);
            gate.admit_shell(crate::MAIN_WINDOW, 1, || now).unwrap();
        }
        assert_eq!(gate.policy().presentation, Presentation::DeadlineFailed);
        assert_eq!(
            native_phases(&trace),
            [
                "host_created",
                "startup_gate_created",
                "startup_shell_accepted",
                "startup_reveal_ready",
                "startup_show_started",
                "startup_show_denied",
                "startup_reveal_deadline",
                "startup_show_started",
                "startup_show_denied_cleanup_failed",
            ]
        );
    }

    #[test]
    fn native_dispatch_denial_never_fabricates_ui_entry_or_returned_show() {
        let trace = RenderTrace::new();
        let gate = StartupGate::new(trace.clone());
        gate.admit_shell(crate::MAIN_WINDOW, 1, || 100).unwrap();
        let reason = gate.begin_reveal(|| 6_000).unwrap();
        gate.dispatch_denied(reason, || 6_001);
        assert_eq!(gate.begin_reveal(|| 9_999), None);
        let reason = gate.begin_reveal(|| 10_000).unwrap();
        gate.dispatch_denied(reason, || 10_001);
        assert_eq!(gate.begin_reveal(|| 20_000), None);
        assert_eq!(
            native_phases(&trace),
            [
                "host_created",
                "startup_gate_created",
                "startup_shell_accepted",
                "startup_reveal_ready",
                "startup_dispatch_denied",
                "startup_reveal_deadline",
                "startup_dispatch_denied",
            ]
        );
        assert_eq!(gate.policy().presentation, Presentation::DeadlineFailed);
        let value = serde_json::to_value(trace.snapshot_for_fixture()).unwrap();
        assert!(value["visibleAtMs"].is_null());
    }

    #[test]
    fn native_policy_race_serialises_admission_before_the_single_deadline_marker() {
        let trace = RenderTrace::new();
        let gate = StartupGate::new(trace.clone());
        let barrier = Arc::new(std::sync::Barrier::new(3));
        let workers: Vec<_> = [false, true]
            .into_iter()
            .map(|signal| {
                let gate = gate.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    if signal {
                        gate.admit_shell(crate::MAIN_WINDOW, 1, || 10_000).unwrap();
                    }
                    gate.begin_reveal(|| 10_000)
                })
            })
            .collect();
        barrier.wait();
        let reasons: Vec<_> = workers
            .into_iter()
            .filter_map(|worker| worker.join().unwrap())
            .collect();
        assert_eq!(reasons, [RevealReason::Deadline]);
        let phases = native_phases(&trace);
        assert!(
            phases
                == [
                    "host_created",
                    "startup_gate_created",
                    "startup_reveal_deadline"
                ]
                || phases
                    == [
                        "host_created",
                        "startup_gate_created",
                        "startup_shell_accepted",
                        "startup_reveal_deadline"
                    ]
        );
        assert_eq!(
            gate.policy().presentation,
            Presentation::Showing(RevealReason::Deadline)
        );
    }

    #[test]
    fn native_clock_suppliers_are_sampled_once_while_policy_transition_is_locked() {
        let trace = RenderTrace::new();
        let gate = StartupGate::new(trace.clone());
        let calls = std::cell::Cell::new(0);
        let sampled = |elapsed_ms| {
            assert!(matches!(
                gate.policy.try_lock(),
                Err(std::sync::TryLockError::WouldBlock)
            ));
            calls.set(calls.get() + 1);
            elapsed_ms
        };
        gate.admit_shell(crate::MAIN_WINDOW, 1, || sampled(100))
            .unwrap();
        gate.admit_shell(crate::MAIN_WINDOW, 1, || sampled(101))
            .unwrap();
        assert_eq!(gate.begin_reveal(|| sampled(5_999)), None);
        let ready = gate.begin_reveal(|| sampled(6_000)).unwrap();
        gate.dispatch_denied(ready, || sampled(6_001));
        assert_eq!(gate.begin_reveal(|| sampled(9_999)), None);
        let deadline = gate.begin_reveal(|| sampled(10_000)).unwrap();
        gate.dispatch_denied(deadline, || sampled(10_001));
        gate.admit_shell(crate::MAIN_WINDOW, 1, || sampled(12_000))
            .unwrap();
        assert_eq!(gate.begin_reveal(|| sampled(12_000)), None);
        assert_eq!(calls.get(), 10);
        assert_eq!(gate.policy().presentation, Presentation::DeadlineFailed);
        assert_eq!(
            native_phases(&trace),
            [
                "host_created",
                "startup_gate_created",
                "startup_shell_accepted",
                "startup_reveal_ready",
                "startup_dispatch_denied",
                "startup_reveal_deadline",
                "startup_dispatch_denied",
            ]
        );
    }
}
