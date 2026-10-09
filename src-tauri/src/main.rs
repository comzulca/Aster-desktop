#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Fixed process-local opaque colour, before Tauri creates runtime threads or
    // any WebView2 controller. Never persist a user/machine environment change.
    #[cfg(windows)]
    std::env::set_var("WEBVIEW2_DEFAULT_BACKGROUND_COLOR", "FF1B1F20");
    aster_desktop_lib::run();
}
