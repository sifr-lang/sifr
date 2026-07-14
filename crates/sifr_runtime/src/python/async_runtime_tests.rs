use super::*;
use crate::cancellation::{CancellationBind, CancellationRequest};
use crate::python::{initialize_runtime, reset_runtime_state_for_tests, test_config, test_guard};
use pyo3::types::PyModule;
use std::sync::atomic::AtomicUsize;

#[test]
fn loop_setup_failure_is_joined_and_leaves_no_live_thread() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("async-loop-failure"))
        .expect("CPython should initialize without the loop");
    FORCE_LOOP_SETUP_FAILURE.store(true, Ordering::SeqCst);

    let error = start().expect_err("forced loop setup should fail");

    assert!(matches!(error, PythonRuntimeError::AsyncRuntimeFailed(_)));
    assert_eq!(
        async_runtime_diagnostics().expect("diagnostics should remain available"),
        PythonAsyncRuntimeDiagnostics::default()
    );
    assert!(matches!(
        shutdown(),
        Err(PythonRuntimeError::AsyncRuntimeFailed(_))
    ));
    assert_eq!(
        async_runtime_diagnostics().expect("failed runtime should normalize to stopped"),
        PythonAsyncRuntimeDiagnostics::default()
    );
}

#[test]
fn cancellation_before_claim_does_not_submit_python_work() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    let mut config = test_config("cancel-before-claim");
    config.start_async_loop = true;
    initialize_runtime(config).expect("init should start the owned loop");
    let carrier = CancellationCarrier::new();
    assert_eq!(
        carrier.request_cancel(),
        CancellationRequest::FallbackPending
    );

    super::super::attach(|py| {
        let asyncio = py.import("asyncio").expect("asyncio should import");
        let coroutine = asyncio
            .call_method1("sleep", (0.0,))
            .expect("sleep coroutine should be created")
            .unbind();
        let Err(error) = submit_coroutine(py, &coroutine, Some(&carrier)) else {
            panic!("pre-cancelled carrier should reject submission");
        };
        assert_eq!(error.kind, "runtime");
        assert!(error.message.contains("cancelled before start"));
        coroutine
            .bind(py)
            .call_method0("close")
            .expect("rejected coroutine should close");
    })
    .expect("runtime should attach");
    assert_eq!(
        async_runtime_diagnostics().expect("diagnostics should be available"),
        PythonAsyncRuntimeDiagnostics {
            running: true,
            stopping: false,
            loop_threads: 1,
            active_submissions: 0,
            pending_submissions: 0,
        }
    );
}

#[test]
fn exact_task_cancellation_runs_finally_before_terminal_completion() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    let mut config = test_config("cancel-finally");
    config.start_async_loop = true;
    initialize_runtime(config).expect("init should start the owned loop");
    let carrier = CancellationCarrier::new();
    let fallback_calls = Arc::new(AtomicUsize::new(0));
    let fallback_counter = Arc::clone(&fallback_calls);
    assert_eq!(
        carrier.bind_fallback(Arc::new(move || {
            fallback_counter.fetch_add(1, Ordering::SeqCst);
        })),
        CancellationBind::Bound
    );

    let (terminal, marker) = super::super::attach(|py| {
        let module = cancellation_test_module(py);
        let marker = module
            .getattr("marker")
            .expect("marker should resolve")
            .unbind();
        let coroutine = module
            .getattr("cancellable")
            .expect("cancellable should resolve")
            .call0()
            .expect("cancellable coroutine should be created")
            .unbind();
        let terminal =
            submit_coroutine(py, &coroutine, Some(&carrier)).expect("submit should succeed");
        (terminal, marker)
    })
    .expect("runtime should attach");
    wait_for_active_submission();

    assert_eq!(carrier.request_cancel(), CancellationRequest::Claimed);
    let outcome = super::super::attach(|py| {
        let outcome = super::super::detach(py, || terminal.wait());
        assert_eq!(marker.bind(py).len().expect("marker should have length"), 1);
        outcome
    })
    .expect("runtime should attach");
    assert!(matches!(outcome, Err(PythonTerminalError::Python(_))));
    assert_eq!(fallback_calls.load(Ordering::SeqCst), 0);
    assert_eq!(
        async_runtime_diagnostics()
            .expect("terminal callback should remove the submission")
            .active_submissions,
        0
    );
}

#[test]
fn cancellation_suppression_result_wins_after_terminal_wait() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    let mut config = test_config("cancel-suppression");
    config.start_async_loop = true;
    initialize_runtime(config).expect("init should start the owned loop");
    let carrier = CancellationCarrier::new();

    let terminal = super::super::attach(|py| {
        let module = cancellation_test_module(py);
        let coroutine = module
            .getattr("suppresses")
            .expect("suppresses should resolve")
            .call0()
            .expect("suppression coroutine should be created")
            .unbind();
        submit_coroutine(py, &coroutine, Some(&carrier)).expect("submit should succeed")
    })
    .expect("runtime should attach");
    wait_for_active_submission();

    assert_eq!(carrier.request_cancel(), CancellationRequest::Claimed);
    super::super::attach(|py| {
        let value = super::super::detach(py, || terminal.wait())
            .expect("suppressed cancellation should return normally");
        let PythonTerminalValue::Raw(value) = value else {
            panic!("raw submission should return a raw terminal value");
        };
        assert_eq!(
            value
                .bind(py)
                .extract::<i64>()
                .expect("result should be an integer"),
            73
        );
    })
    .expect("runtime should attach");
}

#[test]
fn completed_submission_releases_carrier_for_sequential_python_await() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    let mut config = test_config("sequential-cancellation-claim");
    config.start_async_loop = true;
    initialize_runtime(config).expect("init should start the owned loop");
    let carrier = CancellationCarrier::new();

    for expected in [11_i64, 29_i64] {
        let terminal = super::super::attach(|py| {
            let module = cancellation_test_module(py);
            let coroutine = module
                .getattr("immediate")
                .expect("immediate should resolve")
                .call1((expected,))
                .expect("immediate coroutine should be created")
                .unbind();
            submit_coroutine(py, &coroutine, Some(&carrier)).expect("submit should succeed")
        })
        .expect("runtime should attach");
        assert_eq!(wait_terminal_int(terminal), expected);
    }
}

#[test]
fn independent_exact_tasks_cancel_without_cross_talk() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    let mut config = test_config("independent-cancellation");
    config.start_async_loop = true;
    initialize_runtime(config).expect("init should start the owned loop");
    let first_carrier = CancellationCarrier::new();
    let second_carrier = CancellationCarrier::new();

    let (first_terminal, second_terminal) = super::super::attach(|py| {
        let module = cancellation_test_module(py);
        let make_coroutine = || {
            module
                .getattr("suppresses")
                .expect("suppresses should resolve")
                .call0()
                .expect("suppression coroutine should be created")
                .unbind()
        };
        (
            submit_coroutine(py, &make_coroutine(), Some(&first_carrier))
                .expect("first submit should succeed"),
            submit_coroutine(py, &make_coroutine(), Some(&second_carrier))
                .expect("second submit should succeed"),
        )
    })
    .expect("runtime should attach");
    wait_for_submission_count(2);

    assert_eq!(first_carrier.request_cancel(), CancellationRequest::Claimed);
    assert_eq!(wait_terminal_int(first_terminal), 73);
    assert_eq!(
        async_runtime_diagnostics()
            .expect("second task should remain registered")
            .active_submissions,
        1
    );
    assert_eq!(
        second_carrier.request_cancel(),
        CancellationRequest::Claimed
    );
    assert_eq!(wait_terminal_int(second_terminal), 73);
}

#[test]
fn shutdown_terminally_drains_claimed_task_and_finally() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    let mut config = test_config("claimed-shutdown");
    config.start_async_loop = true;
    initialize_runtime(config).expect("init should start the owned loop");
    let carrier = CancellationCarrier::new();

    let (terminal, marker) = super::super::attach(|py| {
        let module = cancellation_test_module(py);
        let marker = module
            .getattr("marker")
            .expect("marker should resolve")
            .unbind();
        let coroutine = module
            .getattr("cancellable")
            .expect("cancellable should resolve")
            .call0()
            .expect("cancellable coroutine should be created")
            .unbind();
        (
            submit_coroutine(py, &coroutine, Some(&carrier)).expect("submit should succeed"),
            marker,
        )
    })
    .expect("runtime should attach");
    wait_for_active_submission();

    shutdown().expect("shutdown should cancel and drain the exact task");
    assert_eq!(
        *SHUTDOWN_PHASE_TRACE
            .lock()
            .expect("shutdown phase trace should lock"),
        vec![
            ShutdownPhase::AdmissionsStopped,
            ShutdownPhase::CallbackShutdown,
            ShutdownPhase::AsyncCleanup,
            ShutdownPhase::SubmissionCancellation,
            ShutdownPhase::LoopStop,
            ShutdownPhase::LoopJoin,
        ]
    );
    let outcome = super::super::attach(|py| {
        let outcome = super::super::detach(py, || terminal.wait());
        assert_eq!(marker.bind(py).len().expect("marker should have length"), 1);
        outcome
    })
    .expect("runtime should attach");
    assert!(matches!(outcome, Err(PythonTerminalError::Python(_))));
    assert_eq!(
        async_runtime_diagnostics().expect("shutdown should normalize diagnostics"),
        PythonAsyncRuntimeDiagnostics::default()
    );
}

#[test]
fn shutdown_retains_owned_loop_authority_for_async_callback_unregister() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    let mut config = test_config("async-callback-unregister-shutdown");
    config.start_async_loop = true;
    initialize_runtime(config).expect("init should start the owned loop");

    let (object, marker) = super::super::attach(|py| {
        let module = PyModule::from_code(
            py,
            c"import asyncio\nmarker = []\n\nclass Resource:\n    async def aclose(self):\n        await asyncio.sleep(0)\n        marker.append('closed')\n",
            c"<sifr-async-unregister-shutdown-test>",
            c"__sifr_async_unregister_shutdown_test__",
        )
        .expect("async unregister test module should compile");
        let object = module
            .getattr("Resource")
            .expect("resource class should resolve")
            .call0()
            .expect("resource should construct");
        let object = super::super::object_ops::store_object(object.unbind())?;
        let marker = module
            .getattr("marker")
            .expect("marker should resolve")
            .unbind();
        Ok::<_, super::super::PythonError>((object, marker))
    })
    .expect("runtime should attach")
    .expect("resource should store");
    let mut group =
        super::super::RetainedCallbackGroup::new().expect("retained callback group should create");
    let owner = group
        .commit_for_object(&object, super::super::RetainedCallbackCleanup::AsyncClose)
        .expect("async close unregister should commit");
    let capture_released = Arc::new(AtomicBool::new(false));
    let capture_at_shutdown = Arc::clone(&capture_released);
    owner
        .retain_capture(move || {
            capture_at_shutdown.store(true, Ordering::SeqCst);
        })
        .expect("shutdown capture should attach");
    drop(group);

    shutdown().expect("shutdown should run async callback unregister before stopping the loop");

    super::super::attach(|py| {
        assert_eq!(marker.bind(py).len().expect("marker should have length"), 1);
    })
    .expect("runtime should attach");
    assert!(capture_released.load(Ordering::SeqCst));
    assert_eq!(owner.status(), super::super::CallbackOwnerStatus::Closed);
}

#[test]
fn invalid_awaitable_setup_resolves_without_leaking_submission_counts() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    let mut config = test_config("invalid-awaitable");
    config.start_async_loop = true;
    initialize_runtime(config).expect("init should start the owned loop");

    let carrier = CancellationCarrier::new();
    let terminal = super::super::attach(|py| {
        let not_a_coroutine = 42_i64
            .into_pyobject(py)
            .expect("integer should convert")
            .into_any()
            .unbind();
        submit_coroutine(py, &not_a_coroutine, Some(&carrier)).expect("queueing should succeed")
    })
    .expect("runtime should attach");
    let outcome = super::super::attach(|py| super::super::detach(py, || terminal.wait()))
        .expect("runtime should attach");

    assert!(matches!(outcome, Err(PythonTerminalError::Python(_))));
    assert_eq!(
        async_runtime_diagnostics().expect("diagnostics should be available"),
        PythonAsyncRuntimeDiagnostics {
            running: true,
            stopping: false,
            loop_threads: 1,
            active_submissions: 0,
            pending_submissions: 0,
        }
    );

    let terminal = immediate_submission(&carrier, 31);
    assert_eq!(wait_terminal_int(terminal), 31);
}

#[test]
fn submission_queue_failure_releases_pending_reservation() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    let mut config = test_config("queue-failure");
    config.start_async_loop = true;
    initialize_runtime(config).expect("init should start the owned loop");
    FORCE_SUBMISSION_QUEUE_FAILURE.store(true, Ordering::SeqCst);
    let carrier = CancellationCarrier::new();

    super::super::attach(|py| {
        let asyncio = py.import("asyncio").expect("asyncio should import");
        let coroutine = asyncio
            .call_method1("sleep", (0.0,))
            .expect("sleep coroutine should be created")
            .unbind();
        let Err(error) = submit_coroutine(py, &coroutine, Some(&carrier)) else {
            panic!("forced queue failure should reject submission");
        };
        assert!(error.message.contains("forced owned asyncio submission"));
        coroutine
            .bind(py)
            .call_method0("close")
            .expect("rejected coroutine should close");
    })
    .expect("runtime should attach");
    assert_eq!(
        async_runtime_diagnostics()
            .expect("queue failure should release pending reservation")
            .pending_submissions,
        0
    );
    let terminal = immediate_submission(&carrier, 37);
    assert_eq!(wait_terminal_int(terminal), 37);
}

#[test]
fn pending_reservations_unwind_by_exact_submission_id() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    let mut config = test_config("pending-id-unwind");
    config.start_async_loop = true;
    initialize_runtime(config).expect("init should start the owned loop");

    let (first_id, second_id, first_terminal, second_terminal) = super::super::attach(|py| {
        let first = PythonTerminal::new();
        let second = PythonTerminal::new();
        let (first_id, _) =
            reserve_submission(py, &first).expect("first reservation should succeed");
        let (second_id, _) =
            reserve_submission(py, &second).expect("second reservation should succeed");
        (first_id, second_id, first, second)
    })
    .expect("runtime should attach");

    release_pending_submission(first_id);
    assert_eq!(
        async_runtime_diagnostics()
            .expect("diagnostics should remain available")
            .pending_submissions,
        1
    );
    release_pending_submission(second_id);
    assert_eq!(
        async_runtime_diagnostics()
            .expect("diagnostics should remain available")
            .pending_submissions,
        0
    );
    drop((first_terminal, second_terminal));
}

#[test]
fn shutdown_errors_do_not_skip_cancel_drain_stop_or_join() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    let mut config = test_config("ordered-shutdown-errors");
    config.start_async_loop = true;
    initialize_runtime(config).expect("init should start the owned loop");
    let carrier = CancellationCarrier::new();
    let terminal = super::super::attach(|py| {
        let module = cancellation_test_module(py);
        let coroutine = module
            .getattr("cancellable")
            .expect("cancellable should resolve")
            .call0()
            .expect("cancellable coroutine should be created")
            .unbind();
        submit_coroutine(py, &coroutine, Some(&carrier)).expect("submit should succeed")
    })
    .expect("runtime should attach");
    wait_for_active_submission();
    super::super::shutdown_hooks::force_callback_shutdown_failure();
    super::super::shutdown_hooks::force_async_cleanup_failure();
    FORCE_SUBMISSION_CANCEL_FAILURE.store(true, Ordering::SeqCst);

    let error = shutdown().expect_err("the first shutdown phase error should be returned");
    assert!(
        matches!(error, PythonRuntimeError::AsyncRuntimeFailed(message) if message.contains("callback shutdown"))
    );
    assert!(matches!(
        terminal.wait(),
        Err(PythonTerminalError::Runtime(
            PythonRuntimeError::AsyncRuntimeFailed(_)
        ))
    ));
    assert_eq!(
        *SHUTDOWN_PHASE_TRACE
            .lock()
            .expect("shutdown phase trace should lock"),
        vec![
            ShutdownPhase::AdmissionsStopped,
            ShutdownPhase::CallbackShutdown,
            ShutdownPhase::AsyncCleanup,
            ShutdownPhase::SubmissionCancellation,
            ShutdownPhase::LoopStop,
            ShutdownPhase::LoopJoin,
        ]
    );
    assert_eq!(
        async_runtime_diagnostics().expect("shutdown should normalize diagnostics"),
        PythonAsyncRuntimeDiagnostics::default()
    );
}

#[test]
fn live_loop_failure_terminally_drains_pending_reservations() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    let terminal = PythonTerminal::new();
    {
        let mut state = ASYNC_STATE.lock().expect("async state should lock");
        state.lifecycle = AsyncLifecycle::Running;
        state.pending_submissions.insert(41, terminal.clone());
    }

    fail_live_runtime("forced live loop failure");

    assert!(matches!(
        terminal.wait(),
        Err(PythonTerminalError::Runtime(
            PythonRuntimeError::AsyncRuntimeFailed(message)
        )) if message.contains("forced live loop failure")
    ));
    assert_eq!(
        async_runtime_diagnostics()
            .expect("failure diagnostics should remain available")
            .pending_submissions,
        0
    );
    assert!(matches!(
        shutdown(),
        Err(PythonRuntimeError::AsyncRuntimeFailed(message))
            if message.contains("forced live loop failure")
    ));
}

#[test]
fn terminal_callback_panic_is_contained_and_removes_registration() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    let mut config = test_config("terminal-panic");
    config.start_async_loop = true;
    initialize_runtime(config).expect("init should start the owned loop");
    FORCE_TERMINAL_CALLBACK_PANIC.store(true, Ordering::SeqCst);

    let terminal = super::super::attach(|py| {
        let asyncio = py.import("asyncio").expect("asyncio should import");
        let coroutine = asyncio
            .call_method1("sleep", (0.0,))
            .expect("sleep coroutine should be created")
            .unbind();
        submit_coroutine(py, &coroutine, None).expect("submission should queue")
    })
    .expect("runtime should attach");
    let outcome = super::super::attach(|py| super::super::detach(py, || terminal.wait()))
        .expect("runtime should attach");

    assert!(matches!(
        outcome,
        Err(PythonTerminalError::Runtime(
            PythonRuntimeError::AsyncRuntimeFailed(_)
        ))
    ));
    assert_eq!(
        async_runtime_diagnostics()
            .expect("terminal failure should remove the registration")
            .active_submissions,
        0
    );
}

fn cancellation_test_module(py: Python<'_>) -> Bound<'_, PyModule> {
    PyModule::from_code(
        py,
        c"import asyncio\nmarker = []\n\nasync def cancellable():\n    try:\n        await asyncio.sleep(60)\n    finally:\n        marker.append('done')\n\nasync def suppresses():\n    try:\n        await asyncio.sleep(60)\n    except asyncio.CancelledError:\n        return 73\n\nasync def immediate(value):\n    return value\n",
        c"<sifr-cancellation-test>",
        c"__sifr_cancellation_test__",
    )
    .expect("cancellation test module should compile")
}

fn immediate_submission(carrier: &CancellationCarrier, value: i64) -> PythonTerminal {
    super::super::attach(|py| {
        let module = cancellation_test_module(py);
        let coroutine = module
            .getattr("immediate")
            .expect("immediate should resolve")
            .call1((value,))
            .expect("immediate coroutine should be created")
            .unbind();
        submit_coroutine(py, &coroutine, Some(carrier)).expect("submit should succeed")
    })
    .expect("runtime should attach")
}

fn wait_for_active_submission() {
    wait_for_submission_count(1);
}

fn wait_for_submission_count(expected: usize) {
    let active = (0..5_000).any(|_| {
        let active = async_runtime_diagnostics()
            .expect("diagnostics should be available")
            .active_submissions
            == expected;
        if !active {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        active
    });
    assert!(active, "expected submission count should become active");
}

fn wait_terminal_int(terminal: PythonTerminal) -> i64 {
    super::super::attach(|py| {
        let value =
            super::super::detach(py, || terminal.wait()).expect("terminal should contain a value");
        let PythonTerminalValue::Raw(value) = value else {
            panic!("raw submission should return a raw terminal value");
        };
        value
            .bind(py)
            .extract::<i64>()
            .expect("terminal value should be an integer")
    })
    .expect("runtime should attach")
}
