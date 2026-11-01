//! UI-thread-owned, read-only WebView2 instrumentation. No URI/content/error
//! strings, setters, navigation, scripts, native dialogue or credential access.
use crate::render_trace::{NativePhase, RenderTrace};
use tauri::{Manager, WebviewWindow};

#[cfg(windows)]
mod native {
    use super::*;
    use std::cell::RefCell;
    use webview2_com::{
        ContentLoadingEventHandler, DOMContentLoadedEventHandler,
        Microsoft::Web::WebView2::Win32::{
            ICoreWebView2, ICoreWebView2Controller, ICoreWebView2Controller2, ICoreWebView2_2,
            COREWEBVIEW2_COLOR,
        },
        NavigationCompletedEventHandler, NavigationStartingEventHandler, ProcessFailedEventHandler,
    };
    use windows::core::{Interface, BOOL};

    thread_local! {
        // COM interfaces never leave the Tauri owning UI thread. Handlers keep
        // only the trace, which holds no window/controller/AppHandle reference.
        static OBSERVER: RefCell<Option<Observer>> = const { RefCell::new(None) };
    }
    struct Observer {
        core: ICoreWebView2,
        core2: ICoreWebView2_2,
        controller: ICoreWebView2Controller,
        background: Option<ICoreWebView2Controller2>,
        tokens: [Option<i64>; 5],
        trace: RenderTrace,
    }
    impl Observer {
        fn new(
            controller: ICoreWebView2Controller,
            trace: RenderTrace,
        ) -> windows::core::Result<Self> {
            // Called only by with_webview on the owning UI thread. Typed COM
            // results retain their references; partial registration is RAII.
            let core = unsafe { controller.CoreWebView2()? };
            let core2 = core.cast::<ICoreWebView2_2>()?;
            let background = controller.cast::<ICoreWebView2Controller2>().ok();
            let mut owner = Self {
                core,
                core2,
                controller,
                background,
                tokens: [None; 5],
                trace,
            };
            macro_rules! register {
                ($index:expr, $method:ident, $handler:ident, $phase:expr) => {{
                    let trace = owner.trace.clone();
                    let callback = $handler::create(Box::new(move |_, _| {
                        trace.native($phase);
                        Ok(())
                    }));
                    let mut token = 0;
                    unsafe {
                        owner.core.$method(&callback, &mut token)?;
                    }
                    owner.tokens[$index] = Some(token);
                }};
            }
            register!(
                0,
                add_NavigationStarting,
                NavigationStartingEventHandler,
                NativePhase::NavigationStarted
            );
            register!(
                1,
                add_ContentLoading,
                ContentLoadingEventHandler,
                NativePhase::ContentLoading
            );
            register!(
                3,
                add_ProcessFailed,
                ProcessFailedEventHandler,
                NativePhase::ProcessFailed
            );
            let trace = owner.trace.clone();
            let completed = NavigationCompletedEventHandler::create(Box::new(move |_, args| {
                let mut success = BOOL::default();
                let phase = match args {
                    Some(args) if unsafe { args.IsSuccess(&mut success) }.is_ok() => {
                        if success.as_bool() {
                            NativePhase::NavigationCompleted
                        } else {
                            NativePhase::NavigationFailed
                        }
                    }
                    _ => NativePhase::ObserverUnavailable,
                };
                trace.native(phase);
                sample_current();
                Ok(())
            }));
            let mut token = 0;
            unsafe {
                owner.core.add_NavigationCompleted(&completed, &mut token)?;
            }
            owner.tokens[2] = Some(token);
            let trace = owner.trace.clone();
            let dom = DOMContentLoadedEventHandler::create(Box::new(move |_, _| {
                trace.native(NativePhase::DomContentLoaded);
                Ok(())
            }));
            let mut token = 0;
            unsafe {
                owner.core2.add_DOMContentLoaded(&dom, &mut token)?;
            }
            owner.tokens[4] = Some(token);
            Ok(owner)
        }
        fn sample(&self) {
            if self.trace.remaining().is_zero() {
                return;
            }
            let mut visible = BOOL::default();
            let visibility = if unsafe { self.controller.IsVisible(&mut visible) }.is_err() {
                NativePhase::ControllerUnavailable
            } else if visible.as_bool() {
                NativePhase::ControllerVisible
            } else {
                NativePhase::ControllerHidden
            };
            self.trace.native(visibility);
            let mut zoom = 0.0;
            let zoom = if unsafe { self.controller.ZoomFactor(&mut zoom) }.is_err()
                || !zoom.is_finite()
                || zoom <= 0.0
            {
                NativePhase::ZoomUnavailable
            } else if (zoom - 1.0).abs() <= f64::EPSILON {
                NativePhase::ZoomDefault
            } else {
                NativePhase::ZoomChanged
            };
            self.trace.native(zoom);
            let mut colour = COREWEBVIEW2_COLOR::default();
            let background = match &self.background {
                Some(controller)
                    if unsafe { controller.DefaultBackgroundColor(&mut colour) }.is_ok() =>
                {
                    if (colour.A, colour.R, colour.G, colour.B) == (255, 27, 31, 32) {
                        NativePhase::BackgroundDark
                    } else {
                        NativePhase::BackgroundOther
                    }
                }
                _ => NativePhase::BackgroundUnavailable,
            };
            self.trace.native(background);
        }
    }
    impl Drop for Observer {
        fn drop(&mut self) {
            // Every caller runs on the owning UI thread, including the TLS
            // destructor fallback. Removal precedes dropping core/controller.
            for (index, token) in self.tokens.iter_mut().enumerate() {
                if let Some(token) = token.take() {
                    let removed = unsafe {
                        match index {
                            0 => self.core.remove_NavigationStarting(token),
                            1 => self.core.remove_ContentLoading(token),
                            2 => self.core.remove_NavigationCompleted(token),
                            3 => self.core.remove_ProcessFailed(token),
                            4 => self.core2.remove_DOMContentLoaded(token),
                            _ => unreachable!(),
                        }
                    };
                    if removed.is_err() {
                        self.trace.native(NativePhase::ObserverUnavailable);
                    }
                }
            }
            self.trace.native(NativePhase::ObserverDetached);
        }
    }
    pub(super) fn sample_current() {
        OBSERVER.with(|slot| {
            if let Ok(owner) = slot.try_borrow() {
                if let Some(owner) = owner.as_ref() {
                    owner.sample();
                }
            }
        });
    }
    pub(super) fn detach_current() {
        let owner = OBSERVER.with(|slot| slot.borrow_mut().take());
        drop(owner); // Never remove callbacks while holding the RefCell borrow.
    }
    pub(super) fn attach(window: &WebviewWindow, trace: RenderTrace) {
        let failure = trace.clone();
        let remaining = trace.remaining();
        if remaining.is_zero() {
            trace.native(NativePhase::ObserverUnavailable);
            return;
        }
        if window
            .with_webview(move |webview| {
                detach_current();
                match Observer::new(webview.controller(), trace.clone()) {
                    Ok(owner) => {
                        OBSERVER.with(|slot| *slot.borrow_mut() = Some(owner));
                        trace.detail_attached();
                        sample_current();
                    }
                    Err(_) => trace.native(NativePhase::ObserverUnavailable),
                }
            })
            .is_err()
        {
            failure.native(NativePhase::ObserverUnavailable);
        }
        let app = window.app_handle().clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(remaining).await;
            // A bounded cleanup task, not an event-handler reference cycle.
            let _ = app.run_on_main_thread(detach_current);
        });
    }
}

pub(crate) fn attach(window: &WebviewWindow) {
    if window.label() != crate::MAIN_WINDOW {
        return;
    }
    let trace = window.state::<RenderTrace>().inner().clone();
    #[cfg(windows)]
    native::attach(window, trace);
    #[cfg(not(windows))]
    trace.native(NativePhase::ObserverUnavailable);
}
pub(crate) fn sample_on_ui_thread() {
    #[cfg(windows)]
    native::sample_current();
}
pub(crate) fn detach_on_ui_thread() {
    #[cfg(windows)]
    native::detach_current();
}
