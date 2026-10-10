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
            let first_native_visible = matches!(phase, Phase::Native(NativePhase::ParentVisible))
                && trace.visible_at_ms.is_none();
            // Keep the first native origin and its event together even when
            // pre-visibility chatter fills the ordinary observation budget.
            let limit = if trace.visible_at_ms.is_none() && !first_native_visible {
                MAX_EVENTS - 1
            } else {
                MAX_EVENTS
            };
            if tick > MAX_MS || trace.events.len() >= limit {
                trace.dropped = trace.dropped.saturating_add(1);
                return;
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
        assert_eq!(snapshot.events.len(), MAX_EVENTS - 1);
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
        assert_eq!(before.events.len(), MAX_EVENTS - 1);
        assert_eq!(before.visible_at_ms, None);
        assert_eq!(before.dropped, 106);

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
}
