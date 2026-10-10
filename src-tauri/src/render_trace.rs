//! Bounded process-local observation, not a rendering fix or an authority grant.
//! Tauri page/parent events are native observations; renderer markers are claims.
//! No URL, path, identity, prose, credential or native error is recorded.
use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tauri::{State, WebviewWindow};

const MAX_EVENTS: usize = 96;
const MAX_MS: u64 = 60_000;
const RESERVED_CRITICAL_EVENTS: usize = 9;
// Gate, first shell admission, reveal decisions, UI starts, returned outcomes
// (including dispatch denial), and the first native visibility respectively.
const CRITICAL_CAPACITIES: [u8; 6] = [1, 1, 2, 2, 2, 1];

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum NativePhase {
    HostCreated,
    ParentCreated,
    PageStarted,
    PageFinished,
    ParentVisible,
    VisibilityDenied,
    Resized,
    ScaleChanged,
    Focused,
    Unfocused,
    Destroyed,
    ObserverAttached,
    ObserverUnavailable,
    ObserverDetached,
    NavigationStarted,
    ContentLoading,
    DomContentLoaded,
    NavigationCompleted,
    NavigationFailed,
    ProcessFailed,
    ControllerVisible,
    ControllerHidden,
    ControllerUnavailable,
    BackgroundDark,
    BackgroundOther,
    BackgroundUnavailable,
    ZoomDefault,
    ZoomChanged,
    ZoomUnavailable,
    StartupGateCreated,
    StartupShellAccepted,
    StartupRevealReady,
    StartupRevealDeadline,
    StartupShowStarted,
    StartupShowVisible,
    StartupShowDenied,
    StartupShowVisibleCleanupFailed,
    StartupShowDeniedCleanupFailed,
    StartupDispatchDenied,
}
impl NativePhase {
    fn reservation(self) -> Option<usize> {
        match self {
            Self::StartupGateCreated => Some(0),
            Self::StartupShellAccepted => Some(1),
            Self::StartupRevealReady | Self::StartupRevealDeadline => Some(2),
            Self::StartupShowStarted => Some(3),
            Self::StartupShowVisible
            | Self::StartupShowDenied
            | Self::StartupShowVisibleCleanupFailed
            | Self::StartupShowDeniedCleanupFailed
            | Self::StartupDispatchDenied => Some(4),
            Self::ParentVisible => Some(5),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RendererPhase {
    Mounted,
    AssetsReady,
    HydrationStarted,
    HydrationFinished,
    Visible,
    Hidden,
}
#[derive(Clone, Serialize)]
#[serde(tag = "source", content = "phase", rename_all = "lowercase")]
enum Phase {
    Native(NativePhase),
    Renderer(RendererPhase),
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Event {
    elapsed_ms: u64,
    #[serde(flatten)]
    phase: Phase,
}
struct Trace {
    events: Vec<Event>,
    dropped: u32,
    visible_at_ms: Option<u64>,
    detail_available: bool,
    critical_used: [u8; 6],
}
struct Inner {
    started: Instant,
    trace: Mutex<Trace>,
}
#[derive(Clone)]
pub(crate) struct RenderTrace {
    inner: Arc<Inner>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Snapshot {
    schema_version: u8,
    classification: &'static str,
    visible_at_ms: Option<u64>,
    elapsed_ms: u64,
    dropped: u32,
    events: Vec<Event>,
    webview2_detail_events_available: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct RendererIntent {
    protocol_version: u8,
    phase: RendererPhase,
}
impl RenderTrace {
    pub(crate) fn new() -> Self {
        let result = Self {
            inner: Arc::new(Inner {
                started: Instant::now(),
                trace: Mutex::new(Trace {
                    events: Vec::new(),
                    dropped: 0,
                    visible_at_ms: None,
                    detail_available: false,
                    critical_used: [0; 6],
                }),
            }),
        };
        result.native(NativePhase::HostCreated);
        result
    }
    fn elapsed(&self) -> u64 {
        self.inner
            .started
            .elapsed()
            .as_millis()
            .min(u64::MAX as u128) as u64
    }
    fn record(&self, phase: Phase) {
        if let Ok(mut trace) = self.inner.trace.lock() {
            let tick = self.elapsed();
            let reservation = match phase {
                Phase::Native(native) => native.reservation(),
                Phase::Renderer(_) => None,
            };
            let first_native_visible = matches!(phase, Phase::Native(NativePhase::ParentVisible))
                && trace.visible_at_ms.is_none();
            // Retain every unused reservation, even after visibility. Ordinary
            // chatter never consumes it; no retained observation is evicted.
            let remaining_reserved = RESERVED_CRITICAL_EVENTS
                - trace
                    .critical_used
                    .iter()
                    .map(|used| usize::from(*used))
                    .sum::<usize>();
            let limit = if reservation.is_some() {
                MAX_EVENTS
            } else {
                MAX_EVENTS - remaining_reserved
            };
            let surplus_critical = reservation
                .is_some_and(|kind| trace.critical_used[kind] >= CRITICAL_CAPACITIES[kind]);
            if tick > MAX_MS || trace.events.len() >= limit || surplus_critical {
                trace.dropped = trace.dropped.saturating_add(1);
                return;
            }
            if let Some(kind) = reservation {
                trace.critical_used[kind] += 1;
            }
            if first_native_visible {
                trace.visible_at_ms = Some(tick);
            }
            trace.events.push(Event {
                elapsed_ms: tick,
                phase,
            });
        }
    }
    pub(crate) fn native(&self, phase: NativePhase) {
        self.record(Phase::Native(phase));
    }
    pub(crate) fn remaining(&self) -> Duration {
        Duration::from_millis(MAX_MS.saturating_sub(self.elapsed()))
    }
    pub(crate) fn detail_attached(&self) {
        if let Ok(mut trace) = self.inner.trace.lock() {
            if self.elapsed() <= MAX_MS {
                trace.detail_available = true;
            }
        }
        self.native(NativePhase::ObserverAttached);
    }
    fn snapshot(&self) -> Result<Snapshot, String> {
        let trace = self
            .inner
            .trace
            .lock()
            .map_err(|_| "render_trace_unavailable")?;
        Ok(Snapshot {
            schema_version: 1,
            classification: "bounded-observation-not-a-rendering-cure",
            visible_at_ms: trace.visible_at_ms,
            elapsed_ms: self.elapsed().min(MAX_MS),
            dropped: trace.dropped,
            events: trace.events.clone(),
            webview2_detail_events_available: trace.detail_available,
        })
    }
    #[cfg(test)]
    pub(crate) fn snapshot_for_fixture(&self) -> Snapshot {
        self.snapshot().unwrap()
    }
}

#[tauri::command]
pub(crate) fn record_render_event(
    window: WebviewWindow,
    trace: State<'_, RenderTrace>,
    input: RendererIntent,
) -> Result<bool, String> {
    crate::trusted(&window)?;
    if input.protocol_version != 1 {
        return Err("render_trace_protocol_denied".into());
    }
    trace.record(Phase::Renderer(input.phase));
    Ok(true)
}
#[tauri::command]
pub(crate) fn get_render_trace(
    window: WebviewWindow,
    trace: State<'_, RenderTrace>,
) -> Result<Snapshot, String> {
    crate::trusted(&window)?;
    trace.snapshot()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_fixed_content_free_marker_objects_are_admitted() {
        for value in [
            r#"{"protocolVersion":1,"phase":"mounted","url":"private"}"#,
            r#"{"protocolVersion":1,"phase":"private text"}"#,
            r#"{"protocolVersion":1,"protocolVersion":1,"phase":"mounted"}"#,
            "[]",
        ] {
            assert!(serde_json::from_str::<RendererIntent>(value).is_err());
        }
    }
    #[test]
    fn renderer_markers_cannot_manufacture_native_visibility_or_unbounded_trace() {
        let trace = RenderTrace::new();
        for _ in 0..200 {
            trace.record(Phase::Renderer(RendererPhase::Visible));
        }
        let snapshot = trace.snapshot().unwrap();
        assert_eq!(snapshot.events.len(), MAX_EVENTS - RESERVED_CRITICAL_EVENTS);
        assert_eq!(snapshot.visible_at_ms, None);
        assert!(snapshot.dropped > 0);
        let value = serde_json::to_value(snapshot).unwrap();
        assert!(!value["webview2DetailEventsAvailable"].as_bool().unwrap());
        assert!(value.to_string().len() < 16_384);
    }
    #[test]
    fn first_native_visibility_retains_its_reserved_event_after_renderer_saturation() {
        let trace = RenderTrace::new();
        for _ in 0..200 {
            trace.record(Phase::Renderer(RendererPhase::Visible));
        }
        let before = trace.snapshot().unwrap();
        assert_eq!(before.events.len(), MAX_EVENTS - RESERVED_CRITICAL_EVENTS);
        assert_eq!(before.visible_at_ms, None);
        assert_eq!(before.dropped, 114);

        // Consume the eight separately reserved startup observations before
        // exercising the original last-slot visibility assertions below.
        for phase in [
            NativePhase::StartupGateCreated,
            NativePhase::StartupShellAccepted,
            NativePhase::StartupRevealReady,
            NativePhase::StartupShowStarted,
            NativePhase::StartupShowDenied,
            NativePhase::StartupRevealDeadline,
            NativePhase::StartupShowStarted,
            NativePhase::StartupShowVisible,
        ] {
            trace.native(phase);
        }

        trace.native(NativePhase::ParentVisible);
        let visible = trace.snapshot().unwrap();
        assert_eq!(visible.events.len(), MAX_EVENTS);
        assert_eq!(visible.dropped, before.dropped);
        let retained = visible.events.last().unwrap();
        assert!(matches!(
            retained.phase,
            Phase::Native(NativePhase::ParentVisible)
        ));
        assert_eq!(visible.visible_at_ms, Some(retained.elapsed_ms));
        assert!(retained.elapsed_ms <= visible.elapsed_ms);

        trace.native(NativePhase::ParentVisible);
        trace.record(Phase::Renderer(RendererPhase::Visible));
        let full = trace.snapshot().unwrap();
        assert_eq!(full.events.len(), MAX_EVENTS);
        assert_eq!(full.dropped, before.dropped + 2);
        assert_eq!(full.visible_at_ms, visible.visible_at_ms);
    }
    #[test]
    fn cloned_native_owner_records_in_one_shared_trace_without_renderer_availability() {
        let trace = RenderTrace::new();
        let observer = trace.clone();
        trace.record(Phase::Renderer(RendererPhase::Visible));
        assert!(!trace.snapshot().unwrap().webview2_detail_events_available);
        observer.detail_attached();
        observer.native(NativePhase::NavigationFailed);
        let snapshot = trace.snapshot().unwrap();
        assert!(snapshot.webview2_detail_events_available);
        assert_eq!(snapshot.visible_at_ms, None);
        assert_eq!(snapshot.events.len(), 4);
        for native in ["observer_attached", "navigation_started", "process_failed"] {
            let forged = serde_json::json!({"protocolVersion":1,"phase":native});
            assert!(serde_json::from_value::<RendererIntent>(forged).is_err());
        }
    }
    #[test]
    fn timestamps_are_monotonic_and_recording_stops_after_deadline_without_sleep() {
        let trace = RenderTrace::new();
        trace.native(NativePhase::ParentVisible);
        trace.record(Phase::Renderer(RendererPhase::Visible));
        let snapshot = trace.snapshot().unwrap();
        assert!(snapshot
            .events
            .windows(2)
            .all(|pair| pair[0].elapsed_ms <= pair[1].elapsed_ms));
        assert!(snapshot.events.last().unwrap().elapsed_ms <= snapshot.elapsed_ms);
        assert!(snapshot.visible_at_ms.is_some());

        let expired = RenderTrace {
            inner: Arc::new(Inner {
                started: Instant::now()
                    .checked_sub(Duration::from_millis(MAX_MS + 1))
                    .unwrap(),
                trace: Mutex::new(Trace {
                    events: Vec::new(),
                    dropped: 0,
                    visible_at_ms: None,
                    detail_available: false,
                    critical_used: [0; 6],
                }),
            }),
        };
        expired.native(NativePhase::ParentVisible);
        expired.record(Phase::Renderer(RendererPhase::Visible));
        expired.detail_attached();
        let snapshot = expired.snapshot().unwrap();
        assert!(snapshot.events.is_empty());
        assert_eq!(snapshot.dropped, 3);
        assert_eq!(snapshot.visible_at_ms, None);
        assert!(!snapshot.webview2_detail_events_available);
        assert_eq!(snapshot.elapsed_ms, MAX_MS);
        assert!(expired.remaining().is_zero());
    }

    #[test]
    fn startup_vocabulary_is_fixed_native_only_and_renderer_cannot_forge_it() {
        for (phase, expected) in [
            (NativePhase::StartupGateCreated, "startup_gate_created"),
            (NativePhase::StartupShellAccepted, "startup_shell_accepted"),
            (NativePhase::StartupRevealReady, "startup_reveal_ready"),
            (
                NativePhase::StartupRevealDeadline,
                "startup_reveal_deadline",
            ),
            (NativePhase::StartupShowStarted, "startup_show_started"),
            (NativePhase::StartupShowVisible, "startup_show_visible"),
            (NativePhase::StartupShowDenied, "startup_show_denied"),
            (
                NativePhase::StartupShowVisibleCleanupFailed,
                "startup_show_visible_cleanup_failed",
            ),
            (
                NativePhase::StartupShowDeniedCleanupFailed,
                "startup_show_denied_cleanup_failed",
            ),
            (
                NativePhase::StartupDispatchDenied,
                "startup_dispatch_denied",
            ),
        ] {
            assert_eq!(serde_json::to_value(phase).unwrap(), expected);
            for forged in [
                serde_json::json!({"protocolVersion":1,"phase":expected}),
                serde_json::json!({"protocolVersion":1,"source":"native","phase":expected}),
            ] {
                assert!(serde_json::from_value::<RendererIntent>(forged).is_err());
            }
        }
        assert_eq!(
            CRITICAL_CAPACITIES
                .iter()
                .map(|count| usize::from(*count))
                .sum::<usize>(),
            RESERVED_CRITICAL_EVENTS
        );
    }

    #[test]
    fn all_critical_slots_survive_chatter_without_evicting_retained_events() {
        let trace = RenderTrace::new();
        for _ in 0..100 {
            trace.native(NativePhase::ControllerVisible);
            trace.record(Phase::Renderer(RendererPhase::Visible));
        }
        let before = trace.snapshot().unwrap();
        assert_eq!(before.events.len(), 87);
        let retained_before = serde_json::to_value(&before.events).unwrap();
        for phase in [
            NativePhase::StartupGateCreated,
            NativePhase::StartupShellAccepted,
            NativePhase::StartupRevealReady,
            NativePhase::StartupShowStarted,
            NativePhase::StartupShowDeniedCleanupFailed,
            NativePhase::StartupRevealDeadline,
            NativePhase::StartupShowStarted,
            NativePhase::StartupShowVisibleCleanupFailed,
            NativePhase::ParentVisible,
        ] {
            trace.native(phase);
        }
        let after = trace.snapshot().unwrap();
        assert_eq!(after.events.len(), MAX_EVENTS);
        assert_eq!(after.dropped, before.dropped);
        assert_eq!(
            serde_json::to_value(&after.events[..before.events.len()]).unwrap(),
            retained_before
        );
        assert_eq!(
            trace.inner.trace.lock().unwrap().critical_used,
            CRITICAL_CAPACITIES
        );
        assert_eq!(
            after.visible_at_ms,
            Some(after.events.last().unwrap().elapsed_ms)
        );
        assert!(after
            .events
            .windows(2)
            .all(|pair| pair[0].elapsed_ms <= pair[1].elapsed_ms));
    }

    #[test]
    fn unused_startup_slots_remain_reserved_after_first_native_visibility() {
        let trace = RenderTrace::new();
        trace.native(NativePhase::ParentVisible);
        let origin = trace.snapshot().unwrap().visible_at_ms;
        for _ in 0..200 {
            trace.record(Phase::Renderer(RendererPhase::Visible));
        }
        let full_ordinary = trace.snapshot().unwrap();
        assert_eq!(full_ordinary.events.len(), 88);
        assert_eq!(full_ordinary.dropped, 114);
        assert_eq!(
            trace.inner.trace.lock().unwrap().critical_used,
            [0, 0, 0, 0, 0, 1]
        );
        trace.native(NativePhase::ParentVisible);
        trace.native(NativePhase::StartupGateCreated);
        trace.record(Phase::Renderer(RendererPhase::Visible));
        let after = trace.snapshot().unwrap();
        assert_eq!(after.events.len(), 89);
        assert_eq!(after.dropped, full_ordinary.dropped + 2);
        assert_eq!(after.visible_at_ms, origin);
    }

    #[test]
    fn surplus_critical_kinds_drop_even_with_free_ordinary_capacity() {
        let trace = RenderTrace::new();
        for phase in [
            NativePhase::StartupGateCreated,
            NativePhase::StartupGateCreated,
            NativePhase::StartupShellAccepted,
            NativePhase::StartupShellAccepted,
            NativePhase::StartupRevealReady,
            NativePhase::StartupRevealDeadline,
            NativePhase::StartupRevealReady,
            NativePhase::StartupShowStarted,
            NativePhase::StartupShowStarted,
            NativePhase::StartupShowStarted,
            NativePhase::StartupShowDenied,
            NativePhase::StartupDispatchDenied,
            NativePhase::StartupShowVisible,
            NativePhase::StartupShowVisibleCleanupFailed,
            NativePhase::StartupShowDeniedCleanupFailed,
            NativePhase::ParentVisible,
            NativePhase::ParentVisible,
        ] {
            trace.native(phase);
        }
        let snapshot = trace.snapshot().unwrap();
        assert_eq!(snapshot.events.len(), 1 + RESERVED_CRITICAL_EVENTS);
        assert_eq!(snapshot.dropped, 8);
        assert_eq!(
            trace.inner.trace.lock().unwrap().critical_used,
            CRITICAL_CAPACITIES
        );
        assert_eq!(
            snapshot.visible_at_ms,
            Some(snapshot.events.last().unwrap().elapsed_ms)
        );
    }

    #[test]
    fn expired_critical_calls_consume_no_reservation_or_visibility_origin() {
        let trace = RenderTrace {
            inner: Arc::new(Inner {
                started: Instant::now()
                    .checked_sub(Duration::from_millis(MAX_MS + 1))
                    .unwrap(),
                trace: Mutex::new(Trace {
                    events: Vec::new(),
                    dropped: 0,
                    visible_at_ms: None,
                    detail_available: false,
                    critical_used: [0; 6],
                }),
            }),
        };
        for phase in [
            NativePhase::StartupGateCreated,
            NativePhase::StartupShellAccepted,
            NativePhase::StartupRevealReady,
            NativePhase::StartupRevealDeadline,
            NativePhase::StartupShowStarted,
            NativePhase::StartupShowVisible,
            NativePhase::StartupShowDenied,
            NativePhase::StartupShowVisibleCleanupFailed,
            NativePhase::StartupShowDeniedCleanupFailed,
            NativePhase::StartupDispatchDenied,
            NativePhase::ParentVisible,
        ] {
            trace.native(phase);
        }
        let snapshot = trace.snapshot().unwrap();
        assert!(snapshot.events.is_empty());
        assert_eq!(snapshot.dropped, 11);
        assert_eq!(snapshot.visible_at_ms, None);
        assert_eq!(trace.inner.trace.lock().unwrap().critical_used, [0; 6]);
        assert_eq!(snapshot.elapsed_ms, MAX_MS);
    }

    #[test]
    fn dropped_accounting_saturates_without_releasing_or_evicting_reservations() {
        let trace = RenderTrace::new();
        trace.native(NativePhase::StartupGateCreated);
        trace.inner.trace.lock().unwrap().dropped = u32::MAX;
        trace.native(NativePhase::StartupGateCreated);
        let snapshot = trace.snapshot().unwrap();
        assert_eq!(snapshot.events.len(), 2);
        assert_eq!(snapshot.dropped, u32::MAX);
        assert_eq!(
            trace.inner.trace.lock().unwrap().critical_used,
            [1, 0, 0, 0, 0, 0]
        );
    }
}
