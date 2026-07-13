use super::errors::registered_exception_names;
use super::registry::{retained_owner_count, shutdown_registered_callback_owners};
use super::{CallbackOwnerState, CallbackOwnerStatus};
use crate::python::{
    initialize_runtime, reset_runtime_state_for_tests, test_config, test_guard, PythonError,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, Mutex};

#[test]
fn runtime_initialization_registers_stable_callback_exception_types() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("callback-exceptions")).expect("runtime should initialize");

    let names = crate::python::attach(|py| registered_exception_names(py))
        .expect("runtime should attach")
        .expect("callback exceptions should be registered");
    assert_eq!(
        names,
        [
            "SifrCallbackError",
            "SifrCallbackClosedError",
            "SifrCallbackReentrancyError",
            "SifrCallbackCloseReentrancyError",
        ]
    );
}

#[test]
fn close_rejects_new_entries_and_drains_accepted_invocations() {
    let _guard = test_guard();
    let unregister_order = Arc::new(Mutex::new(Vec::new()));
    let order_for_unregister = Arc::clone(&unregister_order);
    let owner = CallbackOwnerState::new_retained(move || {
        order_for_unregister
            .lock()
            .expect("order lock should work")
            .push("unregister");
        Ok(())
    })
    .expect("owner should create");
    let invocation = owner.accept(7, false).expect("entry should be accepted");
    let started = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let started_in_thread = Arc::clone(&started);
    let release_in_thread = Arc::clone(&release);
    let invocation_thread = std::thread::spawn(move || {
        let _active = invocation.enter().expect("invocation should enter");
        started_in_thread.wait();
        release_in_thread.wait();
    });
    started.wait();

    let close_owner = owner.clone();
    let close_thread = std::thread::spawn(move || {
        close_owner
            .shutdown_from_runtime()
            .expect("runtime close should drain");
    });
    while owner.status() == CallbackOwnerStatus::Open {
        std::thread::yield_now();
    }
    let rejected = owner.accept(7, false).err().unwrap_or_else(|| {
        panic!("closing owner must reject new entry");
    });
    assert_eq!(rejected.exception_type, "SifrCallbackClosedError");
    assert_eq!(owner.active_calls(), 1);
    release.wait();
    invocation_thread.join().expect("invocation should join");
    close_thread.join().expect("close should join");

    assert_eq!(owner.status(), CallbackOwnerStatus::Closed);
    assert!(owner.captures_released());
    assert_eq!(
        *unregister_order.lock().expect("order lock should work"),
        vec!["unregister"]
    );
    assert_eq!(retained_owner_count(), 0);
}

#[test]
fn invocation_guard_keeps_temporary_admission_live_until_execution_finishes() {
    let _guard = test_guard();
    let owner = CallbackOwnerState::new_call_scoped().expect("owner should create");
    let execution = owner
        .accept(11, false)
        .expect("entry should be accepted")
        .enter()
        .expect("invocation should enter");
    assert_eq!(owner.active_calls(), 1);

    let close_owner = owner.clone();
    let close_finished = Arc::new(AtomicUsize::new(0));
    let close_finished_in_thread = Arc::clone(&close_finished);
    let close_thread = std::thread::spawn(move || {
        close_owner.close_call_scope().expect("close should drain");
        close_finished_in_thread.store(1, Ordering::SeqCst);
    });
    while owner.status() == CallbackOwnerStatus::Open {
        std::thread::yield_now();
    }
    assert_eq!(close_finished.load(Ordering::SeqCst), 0);

    drop(execution);
    close_thread.join().expect("close should finish");
    assert_eq!(close_finished.load(Ordering::SeqCst), 1);
    assert_eq!(owner.status(), CallbackOwnerStatus::Closed);
}

#[test]
fn serial_reentrancy_and_close_from_invocation_fail_before_waiting() {
    let _guard = test_guard();
    let owner = CallbackOwnerState::new_call_scoped().expect("owner should create");
    let invocation = owner.accept(3, true).expect("entry should be accepted");
    let _active = invocation.enter().expect("invocation should enter");

    let reentrant = owner.accept(3, true).err().unwrap_or_else(|| {
        panic!("serial reentrancy should fail");
    });
    assert_eq!(reentrant.exception_type, "SifrCallbackReentrancyError");
    let close = owner
        .close_call_scope()
        .expect_err("close from invocation should fail");
    assert_eq!(close.exception_type, "SifrCallbackCloseReentrancyError");
}

#[test]
fn parallel_reentrancy_is_admitted_and_tracks_the_nested_invocation() {
    let _guard = test_guard();
    let owner = CallbackOwnerState::new_call_scoped().expect("owner should create");
    let first = owner.accept(3, false).expect("first should be accepted");
    let _first_active = first.enter().expect("first should enter");
    let nested = owner
        .accept(3, false)
        .expect("parallel nesting should enter");
    let _nested_active = nested.enter().expect("nested invocation should enter");
    assert_eq!(owner.active_calls(), 2);
}

#[test]
fn concurrent_close_joiners_release_captures_exactly_once() {
    let _guard = test_guard();
    let releases = Arc::new(AtomicUsize::new(0));
    let releases_for_owner = Arc::clone(&releases);
    let owner = CallbackOwnerState::new_call_scoped_with_release(move || {
        releases_for_owner.fetch_add(1, Ordering::SeqCst);
    })
    .expect("owner should create");
    let barrier = Arc::new(Barrier::new(3));
    let mut joiners = Vec::new();
    for _ in 0..2 {
        let owner = owner.clone();
        let barrier = Arc::clone(&barrier);
        joiners.push(std::thread::spawn(move || {
            barrier.wait();
            owner.close_call_scope().expect("close should join");
        }));
    }
    barrier.wait();
    for joiner in joiners {
        joiner.join().expect("close joiner should finish");
    }

    assert_eq!(releases.load(Ordering::SeqCst), 1);
    assert!(owner.captures_released());
}

#[test]
fn semantic_unregister_claim_excludes_runtime_unregister_and_delays_release() {
    let _guard = test_guard();
    let runtime_unregisters = Arc::new(AtomicUsize::new(0));
    let runtime_unregisters_for_owner = Arc::clone(&runtime_unregisters);
    let releases = Arc::new(AtomicUsize::new(0));
    let releases_for_owner = Arc::clone(&releases);
    let owner = CallbackOwnerState::new_retained_with_release(
        move || {
            runtime_unregisters_for_owner.fetch_add(1, Ordering::SeqCst);
            Ok(())
        },
        move || {
            releases_for_owner.fetch_add(1, Ordering::SeqCst);
        },
    )
    .expect("owner should create");
    let semantic_unregister = owner
        .begin_owner_unregister()
        .expect("claim should succeed")
        .expect("semantic cleanup should claim unregister authority");

    let shutdown_owner = owner.clone();
    let shutdown_finished = Arc::new(AtomicUsize::new(0));
    let shutdown_finished_in_thread = Arc::clone(&shutdown_finished);
    let shutdown_started = Arc::new(Barrier::new(2));
    let shutdown_started_in_thread = Arc::clone(&shutdown_started);
    let shutdown_thread = std::thread::spawn(move || {
        shutdown_started_in_thread.wait();
        shutdown_owner
            .shutdown_from_runtime()
            .expect("runtime shutdown should join semantic unregister");
        shutdown_finished_in_thread.store(1, Ordering::SeqCst);
    });
    shutdown_started.wait();
    for _ in 0..100 {
        assert!(!shutdown_thread.is_finished());
        std::thread::yield_now();
    }
    assert_eq!(runtime_unregisters.load(Ordering::SeqCst), 0);
    assert_eq!(releases.load(Ordering::SeqCst), 0);
    assert_eq!(shutdown_finished.load(Ordering::SeqCst), 0);

    drop(semantic_unregister);
    shutdown_thread
        .join()
        .expect("runtime shutdown should finish");
    assert_eq!(runtime_unregisters.load(Ordering::SeqCst), 0);
    assert_eq!(releases.load(Ordering::SeqCst), 1);
    assert_eq!(shutdown_finished.load(Ordering::SeqCst), 1);
    assert_eq!(owner.status(), CallbackOwnerStatus::Closed);
}

#[test]
fn retained_owner_close_requires_unregister_authority_claim() {
    let _guard = test_guard();
    let owner = CallbackOwnerState::new_retained(|| Ok(())).expect("owner should create");
    let error = owner
        .close_after_owner_unregister()
        .expect_err("retained close must not bypass unregister authority");
    assert!(error.message.contains("unregister authority"));
    owner
        .shutdown_from_runtime()
        .expect("runtime should still close owner");
}

#[test]
fn failure_evidence_is_selected_by_entry_sequence_not_completion() {
    let _guard = test_guard();
    let owner = CallbackOwnerState::new_call_scoped().expect("owner should create");
    let first = owner.accept(1, false).expect("first should enter");
    let second = owner.accept(1, false).expect("second should enter");
    owner.record_failure(second.entry_sequence(), "SecondError", "completed first");
    owner.record_failure(first.entry_sequence(), "FirstError", "entered first");

    let evidence = owner.first_failure().expect("failure should be retained");
    assert_eq!(evidence.entry_sequence, first.entry_sequence());
    assert_eq!(evidence.exception_type, "FirstError");
    drop(second);
    drop(first);
    owner.close_call_scope().expect("call scope should close");
}

#[test]
fn runtime_shutdown_uses_stable_owner_order_and_surfaces_unregister_failure() {
    let _guard = test_guard();
    let order = Arc::new(Mutex::new(Vec::new()));
    let first_order = Arc::clone(&order);
    let first = CallbackOwnerState::new_retained(move || {
        first_order.lock().expect("order lock").push("first");
        Ok(())
    })
    .expect("first owner should create");
    let second_order = Arc::clone(&order);
    let second = CallbackOwnerState::new_retained(move || {
        second_order.lock().expect("order lock").push("second");
        Err(PythonError {
            kind: "callback".to_string(),
            exception_type: "UnregisterError".to_string(),
            message: "unregister failed".to_string(),
            traceback: String::new(),
            context: "test".to_string(),
            replay: None,
        })
    })
    .expect("second owner should create");

    let error = shutdown_registered_callback_owners()
        .expect_err("shutdown should surface first unregister failure");
    assert!(error.to_string().contains("unregister failed"));
    assert_eq!(*order.lock().expect("order lock"), vec!["first", "second"]);
    assert_eq!(first.status(), CallbackOwnerStatus::Closed);
    assert_eq!(second.status(), CallbackOwnerStatus::Closed);
    assert!(second.captures_released());
    assert_eq!(retained_owner_count(), 0);
}
