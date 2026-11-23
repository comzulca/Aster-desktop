//! Local draft dictation. Only a session identifier crosses inbound IPC.
use crate::{admission::opaque_id, native_review::ReviewDocument, Host};
use std::{
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
#[cfg(windows)]
use tauri::{Emitter, Manager};
use tauri::{State, WebviewWindow};
use tokio::sync::watch;

#[cfg(windows)]
mod windows;

const ACTIVE: u8 = 0;
const FINISH: u8 = 1;
const CANCEL: u8 = 2;

struct Control {
    id: String,
    action: AtomicU8,
    cancel_review: watch::Sender<bool>,
}

impl Control {
    fn signal(&self, action: u8) {
        // Cancellation always wins, including while the consent window is open.
        self.action.fetch_max(action, Ordering::AcqRel);
        self.cancel_review.send_replace(true);
    }
    fn action(&self) -> u8 {
        self.action.load(Ordering::Acquire)
    }
}

#[derive(Default, Clone)]
pub(crate) struct Speech(Arc<Mutex<Option<Arc<Control>>>>);

impl Speech {
    fn reserve(&self, id: String) -> Result<Session, &'static str> {
        if !opaque_id(&id) {
            return Err("speech_invalid_session");
        }
        let mut slot = self.0.lock().map_err(|_| "speech_failed")?;
        if slot.is_some() {
            return Err("busy");
        }
        let control = Arc::new(Control {
            id,
            action: AtomicU8::new(ACTIVE),
            cancel_review: watch::channel(false).0,
        });
        *slot = Some(control.clone());
        Ok(Session {
            state: self.clone(),
            control,
        })
    }
    fn signal(&self, id: &str, action: u8) -> Result<(), &'static str> {
        let slot = self.0.lock().map_err(|_| "speech_failed")?;
        let control = slot
            .as_ref()
            .filter(|c| c.id == id)
            .ok_or("speech_invalid_session")?;
        control.signal(action);
        Ok(())
    }
    pub(crate) fn cancel_active(&self) {
        if let Ok(slot) = self.0.lock() {
            if let Some(control) = slot.as_ref() {
                control.signal(CANCEL);
            }
        }
    }
}

struct Session {
    state: Speech,
    control: Arc<Control>,
}

#[cfg(windows)]
impl Session {
    fn dictate(
        self,
        window: WebviewWindow,
        directory: &std::path::Path,
    ) -> Result<String, &'static str> {
        let notify = |phase: &str| {
            window
                .emit_to(
                    crate::MAIN_WINDOW,
                    "aster://speech-phase",
                    serde_json::json!({
                        "sessionId": self.control.id, "phase": phase
                    }),
                )
                .map_err(|_| "speech_cancelled")
        };
        let assets = windows::VerifiedAssets::open(directory)?;
        let recording = windows::capture(&self.control, || notify("recording"))?;
        notify("transcribing")?;
        windows::transcribe(&assets, recording, &self.control)
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.control.signal(CANCEL);
        if let Ok(mut slot) = self.state.0.lock() {
            if slot.as_ref().is_some_and(|c| Arc::ptr_eq(c, &self.control)) {
                *slot = None;
            }
        }
    }
}

struct CancelOnDrop(Arc<Control>);
impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.signal(CANCEL);
    }
}

#[tauri::command]
pub(crate) async fn start_speech_capture(
    window: WebviewWindow,
    host: State<'_, Host>,
    state: State<'_, Speech>,
    session_id: String,
) -> Result<String, String> {
    crate::trusted(&window)?;
    let consent_guard = host.enter_native_dialog()?;
    host.idle()?;
    let session = state.reserve(session_id)?;
    let cancellation = CancelOnDrop(session.control.clone());
    let consent = ReviewDocument::proposal(
        r#"{"action":"Record the default microphone for up to 30 seconds","destination":"Local whisper.cpp, on this computer","result":"Insert recognized words into the editable chat draft. You choose whether to send."}"#
    ).map_err(|_| "speech_failed")?
        .with_request_cancellation(session.control.cancel_review.subscribe())
        .with_source_deadline(Instant::now() + Duration::from_secs(60));
    if !crate::review(&window, consent, &consent_guard)
        .await
        .map_err(|e| {
            if e == "cancelled" {
                "speech_cancelled"
            } else {
                "speech_unavailable"
            }
        })?
        || session.control.action() != ACTIVE
    {
        return Err("speech_cancelled".into());
    }
    drop(consent_guard);
    #[cfg(windows)]
    {
        let directory = window
            .path()
            .resource_dir()
            .map_err(|_| "speech_unavailable")?
            .join("speech");
        let result =
            tauri::async_runtime::spawn_blocking(move || session.dictate(window, &directory))
                .await
                .map_err(|_| "speech_failed")?;
        drop(cancellation);
        result.map_err(str::to_owned)
    }
    #[cfg(not(windows))]
    {
        drop(cancellation);
        Err("speech_unavailable".into())
    }
}

#[tauri::command]
pub(crate) fn finish_speech_capture(
    window: WebviewWindow,
    state: State<'_, Speech>,
    session_id: String,
) -> Result<(), String> {
    crate::trusted(&window)?;
    state.signal(&session_id, FINISH).map_err(str::to_owned)
}

#[tauri::command]
pub(crate) fn cancel_speech_capture(
    window: WebviewWindow,
    state: State<'_, Speech>,
    session_id: String,
) -> Result<(), String> {
    crate::trusted(&window)?;
    state.signal(&session_id, CANCEL).map_err(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_the_bound_session_can_stop_and_cancel_is_terminal() {
        let speech = Speech::default();
        let id = "ab".repeat(16);
        assert!(speech.reserve("../mic".into()).is_err());
        let session = speech.reserve(id.clone()).unwrap();
        assert!(speech.reserve("cd".repeat(16)).is_err());
        assert!(speech.signal(&"cd".repeat(16), FINISH).is_err());
        assert_eq!(session.control.action(), ACTIVE);
        speech.signal(&id, FINISH).unwrap();
        assert!(*session.control.cancel_review.borrow());
        speech.cancel_active();
        speech.signal(&id, FINISH).unwrap();
        assert_eq!(session.control.action(), CANCEL);
        drop(session);
        assert!(speech.signal(&id, CANCEL).is_err());
        assert!(speech.reserve("cd".repeat(16)).is_ok());
    }
}
