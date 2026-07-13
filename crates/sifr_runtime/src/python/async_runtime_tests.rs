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
fn invalid_awaitable_setup_resolves_without_leaking_submission_counts() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    let mut config = test_config("invalid-awaitable");
    config.start_async_loop = true;
    initialize_runtime(config).expect("init should start the owned loop");

    let terminal = super::super::attach(|py| {
        let not_a_coroutine = 42_i64
            .into_pyobject(py)
            .expect("integer should convert")
            .into_any()
            .unbind();
        submit_coroutine(py, &not_a_coroutine, None).expect("queueing should succeed")
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
}

#[test]
fn submission_queue_failure_releases_pending_reservation() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    let mut config = test_config("queue-failure");
    config.start_async_loop = true;
    initialize_runtime(config).expect("init should start the owned loop");
    FORCE_SUBMISSION_QUEUE_FAILURE.store(true, Ordering::SeqCst);

    super::super::attach(|py| {
        let asyncio = py.import("asyncio").expect("asyncio should import");
        let coroutine = asyncio
            .call_method1("sleep", (0.0,))
            .expect("sleep coroutine should be created")
            .unbind();
        let Err(error) = submit_coroutine(py, &coroutine, None) else {
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
        c"import asyncio\nmarker = []\n\nasync def cancellable():\n    try:\n        await asyncio.sleep(60)\n    finally:\n        marker.append('done')\n\nasync def suppresses():\n    try:\n        await asyncio.sleep(60)\n    except asyncio.CancelledError:\n        return 73\n",
        c"<sifr-cancellation-test>",
        c"__sifr_cancellation_test__",
    )
    .expect("cancellation test module should compile")
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
        super::super::detach(py, || terminal.wait())
            .expect("terminal should contain a value")
            .bind(py)
            .extract::<i64>()
            .expect("terminal value should be an integer")
    })
    .expect("runtime should attach")
}
