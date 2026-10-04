//! One closed main-window route. No target label, process or data mutation.
use crate::{window_guard, Host, MAIN_WINDOW};
use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering;
use tauri::{Emitter, Manager, State, WebviewWindow, Window, WindowEvent};

pub(crate) const CLOSE_EVENT: &str = "aster://close-request";

#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WindowAction {
    Minimise,
    ToggleMaximise,
    ToggleFullscreen,
    StartDragging,
    Close,
}

impl From<WindowAction> for window_guard::Action {
    fn from(action: WindowAction) -> Self {
        match action {
            WindowAction::Minimise => Self::Minimise,
            WindowAction::ToggleMaximise => Self::ToggleMaximise,
            WindowAction::ToggleFullscreen => Self::ToggleFullscreen,
            WindowAction::StartDragging => Self::StartDragging,
            WindowAction::Close => Self::Close,
        }
    }
}

#[derive(Debug)]
pub(crate) struct WindowIntent {
    action: WindowAction,
}

// Derived struct deserialisation also admits positional sequences. Caption
// intent is an object-only IPC contract, including duplicate-field denial.
impl<'de> Deserialize<'de> for WindowIntent {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ObjectOnly;
        impl<'de> serde::de::Visitor<'de> for ObjectOnly {
            type Value = WindowIntent;
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a closed caption-intent object")
            }
            fn visit_map<M: serde::de::MapAccess<'de>>(
                self,
                map: M,
            ) -> Result<Self::Value, M::Error> {
                #[derive(Deserialize)]
                #[serde(deny_unknown_fields)]
                struct Fields {
                    action: WindowAction,
                }
                let fields =
                    Fields::deserialize(serde::de::value::MapAccessDeserializer::new(map))?;
                Ok(WindowIntent {
                    action: fields.action,
                })
            }
        }
        deserializer.deserialize_map(ObjectOnly)
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WindowStatus {
    maximised: bool,
    fullscreen: bool,
}

fn status(window: &WebviewWindow) -> Result<WindowStatus, String> {
    Ok(WindowStatus {
        maximised: window
            .is_maximized()
            .map_err(|_| "window_action_unavailable")?,
        fullscreen: window
            .is_fullscreen()
            .map_err(|_| "window_action_unavailable")?,
    })
}

#[tauri::command]
pub(crate) async fn window_action(
    window: WebviewWindow,
    host: State<'_, Host>,
    input: WindowIntent,
) -> Result<WindowStatus, String> {
    crate::trusted(&window)?;
    // Shared CAS excludes native secret/disclosure review during every caption
    // change. It also refuses any new operation after a close has been armed.
    let guard = host.enter_native_dialog()?;
    let action = window_guard::Action::from(input.action);
    window_guard::admit(
        window.label(),
        action,
        false,
        host.window_close.is_closing(),
        0,
        true,
    )
    .map_err(str::to_owned)?;
    let before = status(&window)?;
    match input.action {
        WindowAction::Minimise => window.minimize(),
        WindowAction::ToggleMaximise => {
            if before.maximised {
                window.unmaximize()
            } else {
                window.maximize()
            }
        }
        WindowAction::ToggleFullscreen => window.set_fullscreen(!before.fullscreen),
        WindowAction::StartDragging => window.start_dragging(),
        WindowAction::Close => {
            // Never block the OS event thread on these locks. The second check
            // happens again in CloseRequested after this command queues close.
            // An unavailable store can be closed without resetting/repairing it;
            // ownership of the lock proves no ongoing durable operation here.
            let persistence = host.persistent.try_lock().map_err(|_| "busy")?;
            let sessions = host.sessions.try_lock().map_err(|_| "busy")?;
            window_guard::admit(
                window.label(),
                action,
                false,
                host.window_close.is_closing(),
                sessions.active_count(),
                true,
            )
            .map_err(str::to_owned)?;
            host.window_close.arm().map_err(str::to_owned)?;
            drop(sessions);
            drop(persistence);
            // closing remains true, preventing new native/send/create admission.
            // Release this CAS before the callback; it must not mistake this
            // command's own guard for an open trusted dialogue.
            drop(guard);
            if window.close().is_err() {
                host.window_close.abort();
                return Err("window_action_unavailable".into());
            }
            // No query on a queued closing/destroyed window. This receipt is
            // the pre-close presentation state, not proof that exit completed.
            return Ok(before);
        }
    }
    .map_err(|_| "window_action_unavailable")?;
    status(&window)
}

pub(crate) fn on_window_event(window: &Window, event: &WindowEvent) {
    if matches!(event, WindowEvent::Destroyed) && window.label() == MAIN_WINDOW {
        if let Some(host) = window.try_state::<Host>() {
            host.expiry_shutdown.store(true, Ordering::Release);
            if let Ok(mut registry) = host.attachments.try_lock() {
                registry.invalidate();
            }
        }
        return;
    }
    let WindowEvent::CloseRequested { api, .. } = event else {
        return;
    };
    // This app currently creates exactly one window. A future/forged foreign
    // close is denied, never treated as the main renderer's authorisation.
    if window.label() != MAIN_WINDOW {
        api.prevent_close();
        return;
    }
    let Some(host) = window.try_state::<Host>() else {
        api.prevent_close();
        return;
    };
    if host.window_close.consume_authorisation() {
        let allowed = (|| {
            // Existing mutation order is persistent -> sessions. try_lock is
            // mandatory in this event-loop callback to avoid deadlock with IPC
            // workers waiting for window queries or native dialogue callbacks.
            let _persistence = host.persistent.try_lock().map_err(|_| "busy")?;
            let sessions = host.sessions.try_lock().map_err(|_| "busy")?;
            window_guard::confirm_close(
                host.window_close.is_closing(),
                host.native_dialog_open.load(Ordering::Acquire),
                sessions.active_count(),
                true,
            )?;
            host.attachments
                .try_lock()
                .map_err(|_| "busy")?
                .invalidate();
            Ok::<_, &'static str>(())
        })();
        if allowed.is_ok() {
            return;
        }
        // The attempt could not be completed: retain the window and every
        // persisted record, disarm the one-use token and require a fresh review.
        host.window_close.abort();
    }
    api.prevent_close();
    // This does not bypass busy/native admission: the renderer supplies only
    // draft review; an eventual bounded close command repeats the host checks.
    let _ = window.emit_to(MAIN_WINDOW, CLOSE_EVENT, ());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_window_intent_rejects_unknown_actions_and_all_authority_fields() {
        for action in [
            "minimise",
            "toggle_maximise",
            "toggle_fullscreen",
            "start_dragging",
            "close",
        ] {
            let value = serde_json::json!({ "action": action });
            assert!(serde_json::from_value::<WindowIntent>(value.clone()).is_ok());
            for field in [
                "window",
                "label",
                "path",
                "process",
                "approved",
                "ownerHwnd",
                "force",
                "reservation",
            ] {
                let mut extra = value.clone();
                extra[field] = serde_json::json!("forged");
                assert!(serde_json::from_value::<WindowIntent>(extra).is_err());
            }
        }
        for action in ["destroy", "exit", "shell", "minimize", "toggleMaximise", ""] {
            assert!(serde_json::from_value::<WindowIntent>(
                serde_json::json!({ "action": action })
            )
            .is_err());
        }
        for value in [
            serde_json::json!({}),
            serde_json::json!(null),
            serde_json::json!({ "action": 1 }),
            serde_json::json!(["close"]),
        ] {
            assert!(serde_json::from_value::<WindowIntent>(value).is_err());
        }
        assert!(
            serde_json::from_str::<WindowIntent>(r#"{"action":"close","action":"minimise"}"#)
                .is_err()
        );
    }

    #[test]
    fn status_has_only_bounded_camel_case_presentation_fields() {
        assert_eq!(
            serde_json::to_value(WindowStatus {
                maximised: true,
                fullscreen: false
            })
            .unwrap(),
            serde_json::json!({ "maximised": true, "fullscreen": false })
        );
    }
}
