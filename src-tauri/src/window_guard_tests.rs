use super::*;

#[test]
fn foreign_caption_caller_is_denied_for_every_action() {
    for action in [
        Action::Minimise,
        Action::ToggleMaximise,
        Action::ToggleFullscreen,
        Action::StartDragging,
        Action::Close,
    ] {
        assert_eq!(
            admit("other", action, false, false, 0, true),
            Err("caller_denied")
        );
    }
}

#[test]
fn native_review_and_closing_exclude_all_caption_changes() {
    for action in [
        Action::Minimise,
        Action::ToggleMaximise,
        Action::ToggleFullscreen,
        Action::StartDragging,
        Action::Close,
    ] {
        assert_eq!(
            admit("main", action, true, false, 0, true),
            Err("native_dialog_busy")
        );
        assert_eq!(
            admit("main", action, false, true, 0, true),
            Err("window_closing")
        );
    }
}

#[test]
fn close_cannot_abandon_active_request_or_persistence_work() {
    assert_eq!(
        admit("main", Action::Close, false, false, 1, true),
        Err("busy")
    );
    assert_eq!(
        admit("main", Action::Close, false, false, 0, false),
        Err("busy")
    );
    assert_eq!(admit("main", Action::Close, false, false, 0, true), Ok(()));
    assert_eq!(
        admit("main", Action::Minimise, false, false, 1, false),
        Ok(())
    );
}

#[test]
fn close_authorisation_is_one_use_and_failed_attempt_reopens_admission() {
    let state = CloseState::default();
    assert!(!state.is_closing());
    assert!(!state.consume_authorisation());
    assert_eq!(state.arm(), Ok(()));
    assert!(state.is_closing());
    assert_eq!(state.arm(), Err("window_closing"));
    assert!(state.consume_authorisation());
    assert!(!state.consume_authorisation());
    assert!(state.is_closing());
    state.abort();
    assert!(!state.is_closing());
    assert_eq!(state.arm(), Ok(()));
    state.abort();
    assert!(!state.consume_authorisation());
}

#[test]
fn preparation_cleanup_remains_exclusive_after_active_slot_is_removed() {
    // The native guard stays held until existing budget/history error cleanup
    // completes: a zero active count alone cannot authorise normal close.
    assert_eq!(
        admit("main", Action::Close, true, false, 0, true),
        Err("native_dialog_busy")
    );
    assert_eq!(admit("main", Action::Close, false, false, 0, true), Ok(()));
}

#[test]
fn os_close_rechecks_authorisation_native_state_and_quiescence() {
    assert_eq!(
        confirm_close(false, false, 0, true),
        Err("window_close_unconfirmed")
    );
    assert_eq!(
        confirm_close(true, true, 0, true),
        Err("native_dialog_busy")
    );
    assert_eq!(confirm_close(true, false, 1, true), Err("busy"));
    assert_eq!(confirm_close(true, false, 0, false), Err("busy"));
    assert_eq!(confirm_close(true, false, 0, true), Ok(()));
}
