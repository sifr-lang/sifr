use super::{
    abandon_callback_owner_after_error, CallbackOwnerSlot, CallbackOwnerState, CallbackOwnerStatus,
};
use crate::python::{test_guard, PythonError};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[test]
fn failed_context_enter_drains_active_owner_without_calling_context_exit() {
    let _guard = test_guard();
    let unregisters = Arc::new(AtomicUsize::new(0));
    let unregisters_by_owner = Arc::clone(&unregisters);
    let releases = Arc::new(AtomicUsize::new(0));
    let releases_by_owner = Arc::clone(&releases);
    let owner = CallbackOwnerState::new_retained_with_release(
        move || {
            unregisters_by_owner.fetch_add(1, Ordering::SeqCst);
            Ok(())
        },
        move || {
            releases_by_owner.fetch_add(1, Ordering::SeqCst);
        },
    )
    .expect("owner should create");
    let active = owner.accept(1, false).expect("callback should enter");
    owner.record_failure(
        active.entry_sequence(),
        "HandlerError",
        "enter-time failure",
    );
    let release_active = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(20));
        drop(active);
    });

    let primary = PythonError::without_replay(
        "fixture",
        "EnterError",
        "context entry failed",
        String::new(),
        "context enter",
    );
    let error =
        abandon_callback_owner_after_error(primary, &CallbackOwnerSlot::from_owner(owner.clone()));
    release_active.join().expect("callback release should join");

    assert_eq!(owner.status(), CallbackOwnerStatus::Closed);
    assert_eq!(unregisters.load(Ordering::SeqCst), 0);
    assert_eq!(releases.load(Ordering::SeqCst), 1);
    assert!(error.context.contains("HandlerError"), "{error:?}");
}
