use super::errors::registered_exception_names;
use super::registry::{retained_owner_count, shutdown_registered_callback_owners};
use super::{
    attach_callback_failure_evidence, current_callback, current_callback_with_owner,
    foreign_callback, foreign_callback_with_owner, CallbackExecutionError, CallbackFailureSlot,
    CallbackHandlerFailure, CallbackOwnerSlot, CallbackOwnerState, CallbackOwnerStatus,
    ForeignCallbackConcurrency, RetainedCallbackCleanup, RetainedCallbackGroup,
};
use crate::python::{
    call_object_owned, close_object, context_exit_normal_with_callbacks, enter_context, from_int,
    from_str, initialize_runtime, reset_runtime_state_for_tests, resolve_target,
    semantic_close_with_callbacks, test_config, test_guard, to_int, ObjectHandle, PythonError,
};
use pyo3::types::PyAnyMethods;
use std::cell::Cell;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, Mutex};
use std::time::Duration;

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
            "SifrCallbackThreadError",
        ]
    );
}

#[test]
fn current_callback_keeps_non_send_capture_on_creator_thread_and_checks_shape() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("current-callback")).expect("runtime should initialize");
    let capture = Rc::new(Cell::new(0_i64));
    let handler_capture = Rc::clone(&capture);
    let callback = current_callback(
        1,
        1,
        |args| {
            assert_eq!(unsafe { pyo3::ffi::PyGILState_Check() }, 1);
            to_int(&args[0])
        },
        move |_, value| {
            handler_capture.set(handler_capture.get() + value);
            Ok(value + 1)
        },
        |value| {
            assert_eq!(unsafe { pyo3::ffi::PyGILState_Check() }, 1);
            from_int(value)
        },
    )
    .expect("callback should create");

    let arg = from_int(41).expect("argument should convert");
    let result = call_object_owned(callback.object(), &[arg], &[])
        .and_then(|value| to_int(&value))
        .expect("callback should execute");
    assert_eq!(result, 42);
    assert_eq!(capture.get(), 41);

    let shape_error = call_object_owned(callback.object(), &[], &[])
        .expect_err("wrong arity should fail before the handler");
    assert_eq!(shape_error.exception_type, "TypeError");
    let wrong_type = from_str("not-an-int").expect("argument should convert");
    let conversion_error = call_object_owned(callback.object(), &[wrong_type], &[])
        .expect_err("wrong argument type should fail before the handler");
    assert_eq!(conversion_error.exception_type, "TypeError");

    let foreign_handle = callback.object().clone();
    let thread_error = std::thread::spawn(move || {
        let arg = from_int(1).expect("argument should convert");
        call_object_owned(&foreign_handle, &[arg], &[])
            .expect_err("current callback must reject a foreign thread")
    })
    .join()
    .expect("foreign caller should join");
    assert_eq!(thread_error.exception_type, "SifrCallbackThreadError");
    callback.close().expect("call scope should close");
}

#[test]
fn callback_result_conversion_failure_crosses_python_as_type_error() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("callback-result-conversion"))
        .expect("runtime should initialize");
    let callback = current_callback(
        22,
        1,
        |args| to_int(&args[0]),
        |_, value| Ok(value),
        |_value| {
            Err(PythonError::without_replay(
                "conversion",
                "TypeError",
                "callback result has the wrong type",
                "",
                "callback result",
            ))
        },
    )
    .expect("callback should create");
    let arg = from_int(1).expect("argument should convert");
    let error = call_object_owned(callback.object(), &[arg], &[])
        .expect_err("result conversion should fail");
    assert_eq!(error.exception_type, "TypeError");
    callback.close().expect("callback should close");
}

#[test]
fn call_scoped_callbacks_accept_borrowed_handler_state() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("scoped-callback-borrows")).expect("runtime should initialize");

    let current_total = Cell::new(0_i64);
    {
        let callback = current_callback(
            20,
            1,
            |args| to_int(&args[0]),
            |_, value| {
                current_total.set(current_total.get() + value);
                Ok(value)
            },
            from_int,
        )
        .expect("borrowed current callback should create");
        let arg = from_int(3).expect("argument should convert");
        call_object_owned(callback.object(), &[arg], &[]).expect("callback should execute");
        callback.close().expect("callback should close");
    }
    assert_eq!(current_total.get(), 3);

    let foreign_total = AtomicUsize::new(0);
    {
        let callback = foreign_callback(
            21,
            1,
            ForeignCallbackConcurrency::Parallel,
            |args| to_int(&args[0]),
            |_, value| {
                foreign_total.fetch_add(usize::try_from(value).unwrap_or(0), Ordering::SeqCst);
                Ok(value)
            },
            from_int,
        )
        .expect("borrowed foreign callback should create");
        invoke_from_threads(callback.object(), 4);
        callback.close_call_scope().expect("callback should close");
    }
    assert_eq!(foreign_total.load(Ordering::SeqCst), 6);
}

#[test]
fn handler_failure_raises_registered_error_and_retains_first_entry_evidence() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("callback-handler-error")).expect("runtime should initialize");
    let callback = current_callback(
        2,
        1,
        |args| to_int(&args[0]),
        |_, _| -> Result<i64, CallbackExecutionError> {
            Err(CallbackExecutionError::Handler(
                CallbackHandlerFailure::new("DomainError", "handler rejected value"),
            ))
        },
        from_int,
    )
    .expect("callback should create");
    let arg = from_int(7).expect("argument should convert");
    let error = call_object_owned(callback.object(), &[arg], &[])
        .expect_err("handler failure should cross Python as a registered error");
    assert_eq!(error.exception_type, "SifrCallbackError");
    let evidence = callback
        .owner()
        .first_failure()
        .expect("typed adapter should retain redacted evidence");
    assert_eq!(evidence.entry_sequence, 1);
    assert_eq!(evidence.exception_type, "DomainError");
    assert_eq!(evidence.message, "handler rejected value");
    callback.close().expect("call scope should close");
}

#[test]
fn swallowed_python_callback_error_remains_in_typed_failure_slot() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("swallowed-callback-error")).expect("runtime should initialize");
    let failures = CallbackFailureSlot::new();
    let failures_for_handler = failures.clone();
    let callback = current_callback(
        23,
        1,
        |args| to_int(&args[0]),
        move |sequence, _| -> Result<i64, CallbackExecutionError> {
            failures_for_handler.record(sequence, "typed handler failure".to_string());
            Err(CallbackExecutionError::Handler(
                CallbackHandlerFailure::new("HandlerError", "handler failed"),
            ))
        },
        from_int,
    )
    .expect("callback should create");
    let catcher = crate::python::attach(|py| {
        let module = pyo3::types::PyModule::from_code(
            py,
            c"def invoke(callback):\n    try:\n        return callback(1)\n    except Exception:\n        return 99\n",
            c"callback_catcher.py",
            c"callback_catcher",
        )
        .expect("catcher module should compile");
        let invoke = module.getattr("invoke").expect("invoke should exist");
        super::super::object_ops::store_object(invoke.unbind())
            .expect("catcher should enter object store")
    })
    .expect("runtime should attach");
    let callback_arg = super::super::object_ops::temporary_argument_handle(callback.object())
        .expect("callback argument should clone");
    let result = call_object_owned(&catcher, &[callback_arg], &[])
        .and_then(|value| to_int(&value))
        .expect("Python target should swallow the callback exception");
    assert_eq!(result, 99);
    assert_eq!(failures.take().as_deref(), Some("typed handler failure"));
    callback.close().expect("callback should close");
    close_object(catcher).expect("catcher should close");
}

#[test]
fn python_error_remains_primary_with_callback_failure_as_secondary_evidence() {
    let owner = CallbackOwnerState::new_call_scoped().expect("owner should create");
    owner.record_failure(4, "HandlerError", "handler failed");
    let primary =
        PythonError::without_replay("call", "ValueError", "target failed", "", "python target");
    let error = attach_callback_failure_evidence::<()>(Err(primary), &[&owner, &owner])
        .expect_err("Python failure should remain primary");
    assert_eq!(error.exception_type, "ValueError");
    assert!(error.context.contains("python target"));
    assert!(error.context.contains("secondary Sifr callback failure"));
    assert!(error.context.contains("HandlerError at entry 4"));
    assert_eq!(
        error
            .context
            .matches("secondary Sifr callback failure")
            .count(),
        1
    );
    owner.close_call_scope().expect("owner should close");
}

#[test]
fn foreign_serial_callback_is_fifo_and_runs_without_the_gil() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("foreign-serial-callback")).expect("runtime should initialize");
    let execution_order = Arc::new(Mutex::new(Vec::new()));
    let active = Arc::new(AtomicUsize::new(0));
    let max_active = Arc::new(AtomicUsize::new(0));
    let order_for_handler = Arc::clone(&execution_order);
    let active_for_handler = Arc::clone(&active);
    let max_for_handler = Arc::clone(&max_active);
    let callback = foreign_callback(
        3,
        1,
        ForeignCallbackConcurrency::Serial,
        |args| {
            assert_eq!(unsafe { pyo3::ffi::PyGILState_Check() }, 1);
            to_int(&args[0])
        },
        move |entry_sequence, value| {
            assert_eq!(unsafe { pyo3::ffi::PyGILState_Check() }, 0);
            let now = active_for_handler.fetch_add(1, Ordering::SeqCst) + 1;
            max_for_handler.fetch_max(now, Ordering::SeqCst);
            order_for_handler
                .lock()
                .expect("order lock")
                .push(entry_sequence);
            std::thread::sleep(Duration::from_millis(5));
            active_for_handler.fetch_sub(1, Ordering::SeqCst);
            Ok(value)
        },
        |value| {
            assert_eq!(unsafe { pyo3::ffi::PyGILState_Check() }, 1);
            from_int(value)
        },
    )
    .expect("callback should create");
    invoke_from_threads(callback.object(), 8);
    assert_eq!(max_active.load(Ordering::SeqCst), 1);
    assert_eq!(
        *execution_order.lock().expect("order lock"),
        (1_u64..=8).collect::<Vec<_>>()
    );
    callback
        .close_call_scope()
        .expect("foreign call scope should close");
}

#[test]
fn foreign_parallel_overlaps_and_serial_reentrancy_fails_before_handler_lock() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("foreign-parallel-callback"))
        .expect("runtime should initialize");
    let entered = Arc::new(Barrier::new(4));
    let active = Arc::new(AtomicUsize::new(0));
    let max_active = Arc::new(AtomicUsize::new(0));
    let entered_for_handler = Arc::clone(&entered);
    let active_for_handler = Arc::clone(&active);
    let max_for_handler = Arc::clone(&max_active);
    let parallel = foreign_callback(
        4,
        1,
        ForeignCallbackConcurrency::Parallel,
        |args| to_int(&args[0]),
        move |_, value| {
            let now = active_for_handler.fetch_add(1, Ordering::SeqCst) + 1;
            max_for_handler.fetch_max(now, Ordering::SeqCst);
            entered_for_handler.wait();
            active_for_handler.fetch_sub(1, Ordering::SeqCst);
            Ok(value)
        },
        from_int,
    )
    .expect("parallel callback should create");
    invoke_from_threads(parallel.object(), 4);
    assert_eq!(max_active.load(Ordering::SeqCst), 4);
    parallel
        .close_call_scope()
        .expect("parallel scope should close");

    let callback_slot = Arc::new(Mutex::new(None::<ObjectHandle>));
    let slot_for_handler = Arc::clone(&callback_slot);
    let owner = CallbackOwnerState::new_call_scoped().expect("owner should create");
    let reentrant = foreign_callback_with_owner(
        owner.clone(),
        5,
        1,
        ForeignCallbackConcurrency::Serial,
        |args| to_int(&args[0]),
        move |_, value| {
            let callback = slot_for_handler
                .lock()
                .expect("callback slot")
                .clone()
                .expect("callback should be installed");
            let arg = from_int(value)?;
            call_object_owned(&callback, &[arg], &[])
                .map(|_| value)
                .map_err(CallbackExecutionError::from)
        },
        from_int,
    )
    .expect("serial callback should create");
    *callback_slot.lock().expect("callback slot") = Some(reentrant.object().clone());
    let arg = from_int(1).expect("argument should convert");
    let error = call_object_owned(reentrant.object(), &[arg], &[])
        .expect_err("recursive serial entry should fail");
    assert_eq!(error.exception_type, "SifrCallbackReentrancyError");
    assert_eq!(owner.active_calls(), 0);
    reentrant
        .close_call_scope()
        .expect("serial scope should close");
}

fn invoke_from_threads(callback: &ObjectHandle, count: usize) {
    let start = Arc::new(Barrier::new(count));
    let mut threads = Vec::new();
    for value in 0..count {
        let callback = callback.clone();
        let start = Arc::clone(&start);
        threads.push(std::thread::spawn(move || {
            start.wait();
            let value = i64::try_from(value).expect("test value should fit");
            let arg = from_int(value).expect("argument should convert");
            let result = call_object_owned(&callback, &[arg], &[])
                .and_then(|value| to_int(&value))
                .expect("callback should execute");
            assert_eq!(result, value);
        }));
    }
    for thread in threads {
        thread.join().expect("callback thread should join");
    }
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
    let call_scope_error = owner
        .close_call_scope()
        .expect_err("retained owner must not use call-scoped close");
    assert!(call_scope_error.message.contains("unregister authority"));
    owner
        .shutdown_from_runtime()
        .expect("runtime should still close owner");
}

#[test]
fn retained_owner_shutdown_surfaces_unobserved_handler_failure() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    let owner = CallbackOwnerState::new_retained(|| Ok(())).expect("owner should create");
    owner.record_failure(3, "HandlerError", "retained handler failed");
    let error = owner
        .shutdown_from_runtime()
        .expect_err("unobserved retained failure should surface at shutdown");
    assert_eq!(error.exception_type, "SifrCallbackError");
    assert!(error.message.contains("HandlerError at entry 3"));
    assert_eq!(owner.status(), CallbackOwnerStatus::Closed);
    assert_eq!(retained_owner_count(), 0);
}

#[test]
fn semantic_close_leaves_retained_failure_for_typed_owner_observer() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("retained-typed-observer")).expect("runtime should initialize");
    let target = resolve_target(&["contextlib".to_string(), "ExitStack".to_string()])
        .expect("ExitStack should resolve");
    let object = call_object_owned(&target, &[], &[]).expect("ExitStack should construct");
    let failures = CallbackFailureSlot::new();
    failures.record(1, "typed retained failure".to_string());

    let mut group = RetainedCallbackGroup::new().expect("group should create");
    group
        .owner()
        .record_failure(1, "HandlerError", "retained handler failed");
    let owner = group
        .commit_for_object(&object, RetainedCallbackCleanup::Close)
        .expect("owner should commit");
    semantic_close_with_callbacks(
        object,
        "close",
        CallbackOwnerSlot::from_owner(owner.clone()),
    )
    .expect("semantic cleanup should leave typed handler failure to generated code");

    assert_eq!(
        failures.take_if_owner_first(&owner).as_deref(),
        Some("typed retained failure")
    );
    assert_eq!(owner.status(), CallbackOwnerStatus::Closed);
    close_object(target).expect("target should close");
}

#[test]
fn shared_call_scope_selects_first_failure_across_callback_parameters() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("shared-call-callback-owner"))
        .expect("runtime should initialize");
    let owner = CallbackOwnerState::new_call_scoped().expect("owner should create");
    let first_slot = CallbackFailureSlot::new();
    let first_handler_slot = first_slot.clone();
    let first = current_callback_with_owner(
        owner.clone(),
        1,
        1,
        |args| to_int(&args[0]),
        move |sequence, _| -> Result<i64, CallbackExecutionError> {
            first_handler_slot.record(sequence, "first parameter".to_string());
            Err(CallbackExecutionError::Handler(
                CallbackHandlerFailure::new("FirstError", "first parameter failed"),
            ))
        },
        from_int,
    )
    .expect("first callback should create");
    let second_slot = CallbackFailureSlot::new();
    let second_handler_slot = second_slot.clone();
    let second = current_callback_with_owner(
        owner.clone(),
        2,
        1,
        |args| to_int(&args[0]),
        move |sequence, _| -> Result<i64, CallbackExecutionError> {
            second_handler_slot.record(sequence, "second parameter".to_string());
            Err(CallbackExecutionError::Handler(
                CallbackHandlerFailure::new("SecondError", "second parameter failed"),
            ))
        },
        from_int,
    )
    .expect("second callback should create");

    for callback in [&second, &first] {
        let arg = from_int(1).expect("argument should convert");
        call_object_owned(callback.object(), &[arg], &[])
            .expect_err("handler should fail through Python");
    }
    assert_eq!(first_slot.take_if_owner_first(&owner), None);
    assert_eq!(
        second_slot.take_if_owner_first(&owner).as_deref(),
        Some("second parameter")
    );
    first.close().expect("shared scope should close");
    second
        .close()
        .expect("second callable should close idempotently");
}

#[test]
fn retained_group_commits_to_opaque_slot_and_semantic_close_releases_captures() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("retained-callback-slot")).expect("runtime should initialize");
    let target = resolve_target(&["contextlib".to_string(), "ExitStack".to_string()])
        .expect("ExitStack should resolve");
    let object = call_object_owned(&target, &[], &[]).expect("ExitStack should construct");

    let released = Arc::new(AtomicUsize::new(0));
    let released_by_owner = Arc::clone(&released);
    let mut group = RetainedCallbackGroup::new().expect("group should create");
    group
        .owner()
        .retain_capture(move || {
            released_by_owner.fetch_add(1, Ordering::SeqCst);
        })
        .expect("capture should attach");
    let owner = group
        .commit_for_object(&object, RetainedCallbackCleanup::Close)
        .expect("owner should commit");
    let slot = CallbackOwnerSlot::from_owner(owner.clone());

    semantic_close_with_callbacks(object, "close", slot)
        .expect("semantic close should unregister and drain callbacks");
    assert_eq!(owner.status(), CallbackOwnerStatus::Closed);
    assert_eq!(released.load(Ordering::SeqCst), 1);
    assert_eq!(retained_owner_count(), 0);
    close_object(target).expect("target should close");
}

#[test]
fn retained_context_owner_exits_before_callback_capture_release() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("retained-context-callback-slot"))
        .expect("runtime should initialize");
    let target = resolve_target(&["contextlib".to_string(), "ExitStack".to_string()])
        .expect("ExitStack should resolve");
    let object = call_object_owned(&target, &[], &[]).expect("ExitStack should construct");
    let entered = enter_context(&object).expect("context should enter");
    close_object(entered).expect("entered alias should close");

    let released = Arc::new(AtomicUsize::new(0));
    let released_by_owner = Arc::clone(&released);
    let mut group = RetainedCallbackGroup::new().expect("group should create");
    group
        .owner()
        .retain_capture(move || {
            released_by_owner.fetch_add(1, Ordering::SeqCst);
        })
        .expect("capture should attach");
    let owner = group
        .commit_for_object(&object, RetainedCallbackCleanup::Context)
        .expect("owner should commit");
    let slot = CallbackOwnerSlot::from_owner(owner.clone());

    context_exit_normal_with_callbacks(object, slot).expect("context exit should drain callbacks");
    assert_eq!(owner.status(), CallbackOwnerStatus::Closed);
    assert_eq!(released.load(Ordering::SeqCst), 1);
    close_object(target).expect("target should close");
}

#[test]
fn uncommitted_retained_group_releases_provisional_captures() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    let released = Arc::new(AtomicUsize::new(0));
    let released_by_owner = Arc::clone(&released);
    let owner = {
        let group = RetainedCallbackGroup::new().expect("group should create");
        group
            .owner()
            .retain_capture(move || {
                released_by_owner.fetch_add(1, Ordering::SeqCst);
            })
            .expect("capture should attach");
        group.owner().clone()
    };
    assert_eq!(owner.status(), CallbackOwnerStatus::Closed);
    assert_eq!(released.load(Ordering::SeqCst), 1);
    assert_eq!(retained_owner_count(), 0);
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
