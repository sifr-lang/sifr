use super::*;
use pyo3::types::{PyDict, PyModule};
use std::future::Future;
use std::pin::pin;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};

const MODULE: &str = "__sifr_async_context__";

#[test]
fn async_context_enter_borrows_manager_and_exit_consumes_it_exactly_once() {
    let _guard = test_guard();
    initialize_context_runtime("async-context-normal");
    let manager = manager("normal");
    let alias = manager.clone();

    let entered = block_on(submit_async_context_enter(
        &manager,
        PythonAsyncType::Int,
        None,
    ))
    .expect("aenter should complete");
    assert_eq!(async_to_int(entered).expect("entered int"), 41);
    async_from_object(&alias).expect("borrowed enter must retain manager ownership");

    let decision = block_on(submit_async_context_exit(
        manager,
        PythonAsyncExitCause::Normal,
        None,
    ))
    .expect("aexit should complete");
    assert_eq!(decision, PythonExitDecision::Propagate);
    assert_eq!(events(), vec!["normal:enter", "normal:exit:None"]);
    let closed = async_from_object(&alias).expect_err("terminal exit must close retained aliases");
    assert!(closed.message.contains("closed"), "{closed:?}");
    assert!(
        block_on(submit_async_context_exit(
            alias,
            PythonAsyncExitCause::Normal,
            None,
        ))
        .is_err()
    );

    shutdown_context_runtime();
}

#[test]
fn async_context_exit_replays_python_exception_and_normalizes_truthiness() {
    let _guard = test_guard();
    initialize_context_runtime("async-context-replay");
    let manager = manager("suppress");
    let error = captured_python_error();

    let decision = block_on(submit_async_context_exit(
        manager,
        PythonAsyncExitCause::Python(error.clone()),
        None,
    ))
    .expect("aexit should receive replay triple");
    assert_eq!(decision, PythonExitDecision::Suppress);
    assert_eq!(events(), vec!["suppress:exit:ValueError:originating boom"]);
    assert_eq!(error.exception_type, "ValueError");

    shutdown_context_runtime();
}

#[test]
fn async_context_exit_materializes_redacted_sifr_boundary() {
    let _guard = test_guard();
    initialize_context_runtime("async-context-boundary");
    let manager = manager("truthy");
    let cause = SifrExitCause {
        kind: SifrExitCauseKind::Cancellation,
        sifr_type: "Cancelled".to_string(),
        message: "task cancelled".to_string(),
    };

    let decision = block_on(submit_async_context_exit(
        manager,
        PythonAsyncExitCause::Sifr(cause),
        None,
    ))
    .expect("aexit should receive SifrBoundaryError");
    assert_eq!(decision, PythonExitDecision::Suppress);
    assert_eq!(
        events(),
        vec!["truthy:exit:SifrBoundaryError:task cancelled"]
    );

    shutdown_context_runtime();
}

#[test]
fn async_context_exit_failure_poisons_manager_and_enter_conversion_leaves_it_exit_capable() {
    let _guard = test_guard();
    initialize_context_runtime("async-context-poison");
    let conversion_manager = manager("conversion");
    let conversion_alias = conversion_manager.clone();
    let conversion = block_on(submit_async_context_enter(
        &conversion_manager,
        PythonAsyncType::Str,
        None,
    ))
    .expect_err("entered value should fail conversion");
    assert_eq!(conversion.kind, "conversion");
    block_on(submit_async_context_exit(
        conversion_manager,
        PythonAsyncExitCause::Sifr(SifrExitCause {
            kind: SifrExitCauseKind::OrdinaryError,
            sifr_type: "ConversionError".to_string(),
            message: conversion.message,
        }),
        None,
    ))
    .expect("conversion failure must leave manager available for mandatory exit");
    assert!(async_from_object(&conversion_alias).is_err());

    let failing = manager("fail");
    let failing_alias = failing.clone();
    let error = block_on(submit_async_context_exit(
        failing,
        PythonAsyncExitCause::Normal,
        None,
    ))
    .expect_err("aexit failure should propagate");
    assert_eq!(error.exception_type, "RuntimeError");
    let poisoned = async_from_object(&failing_alias).expect_err("failed aexit must poison manager");
    assert!(poisoned.message.contains("poisoned"), "{poisoned:?}");

    shutdown_context_runtime();
}

#[test]
fn child_carrier_waits_for_python_finally_before_cancellation_race_wins() {
    let _guard = test_guard();
    initialize_context_runtime("async-context-cancellation");
    let manager = manager("wait-enter");
    let parent = crate::cancellation::CancellationCarrier::new();
    assert_eq!(
        parent.bind_fallback(Arc::new(|| {})),
        crate::cancellation::CancellationBind::Bound
    );
    let scope = crate::cancellation::CancellationScopeLease::claim(&parent)
        .expect("scope should claim parent");
    let child = scope.child().clone();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Tokio runtime should build");
    let cancelled = runtime.block_on(async {
        let requester_parent = parent.clone();
        let requester = tokio::spawn(async move {
            for _ in 0..4_000 {
                if async_runtime_diagnostics()
                    .is_ok_and(|diagnostics| diagnostics.active_submissions == 1)
                {
                    assert_eq!(
                        requester_parent.request_cancel(),
                        crate::cancellation::CancellationRequest::Claimed
                    );
                    return;
                }
                tokio::task::yield_now().await;
            }
            panic!("async enter submission did not register");
        });
        let mut notification = Box::pin(scope.notification());
        let mut enter = Box::pin(submit_async_context_enter(
            &manager,
            PythonAsyncType::Int,
            Some(&child),
        ));
        let cancelled = tokio::select! {
            biased;
            _ = &mut notification => true,
            _ = &mut enter => false,
        };
        requester.await.expect("requester should not panic");
        cancelled
    });
    assert!(
        cancelled,
        "sticky cancellation arm must win after Python terminal"
    );
    assert_eq!(events(), vec!["wait-enter:enter-finally"]);
    assert_eq!(
        scope.release_and_resume_parent(),
        crate::cancellation::CancellationResume::Invoked
    );
    poison_object(manager);
    shutdown_context_runtime();
}

fn initialize_context_runtime(label: &str) {
    reset_runtime_state_for_tests();
    let mut config = test_config(label);
    config.start_async_loop = true;
    config.required_import_roots.push(MODULE.to_string());
    config.trusted_import_roots.push(MODULE.to_string());
    initialize_runtime(config).expect("async context runtime should initialize");
    attach(|py| {
        let module = PyModule::from_code(
            py,
            c"import asyncio\n\nevents = []\n\nclass Manager:\n    def __init__(self, mode):\n        self.mode = mode\n\n    async def __aenter__(self):\n        if self.mode == 'wait-enter':\n            try:\n                await asyncio.Event().wait()\n            finally:\n                events.append(f'{self.mode}:enter-finally')\n        events.append(f'{self.mode}:enter')\n        await asyncio.sleep(0)\n        return 41\n\n    async def __aexit__(self, error_type, error_value, traceback):\n        await asyncio.sleep(0)\n        label = 'None' if error_type is None else f'{error_type.__name__}:{error_value}'\n        events.append(f'{self.mode}:exit:{label}')\n        if self.mode == 'fail':\n            raise RuntimeError('exit boom')\n        return self.mode in ('suppress', 'truthy')\n\ndef originating_error():\n    raise ValueError('originating boom')\n",
            c"<sifr-async-context>",
            c"__sifr_async_context__",
        )?;
        let modules = py.import("sys")?.getattr("modules")?.cast_into::<PyDict>()?;
        modules.set_item(MODULE, module)
    })
    .expect("runtime should attach")
    .expect("test module should install");
}

fn manager(mode: &str) -> ObjectHandle {
    attach(|py| {
        let object = py.import(MODULE)?.getattr("Manager")?.call1((mode,))?;
        super::object_ops::store_object(object.unbind().into())
            .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(error.to_string()))
    })
    .expect("runtime should attach")
    .expect("manager should store")
}

fn captured_python_error() -> PythonError {
    attach(|py| {
        let error = py
            .import(MODULE)
            .and_then(|module| module.call_method0("originating_error"))
            .expect_err("originating_error should raise");
        PythonError::from_pyerr(py, error, "call", "originating_error")
    })
    .expect("runtime should attach")
}

fn events() -> Vec<String> {
    attach(|py| py.import(MODULE)?.getattr("events")?.extract())
        .expect("runtime should attach")
        .expect("events should extract")
}

fn shutdown_context_runtime() {
    super::async_runtime::shutdown().expect("owned loop should stop");
}

fn block_on<T>(future: impl Future<Output = T>) -> T {
    struct ThreadWake(std::thread::Thread);

    impl Wake for ThreadWake {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
    }

    let waker = Waker::from(Arc::new(ThreadWake(std::thread::current())));
    let mut context = Context::from_waker(&waker);
    let mut future = pin!(future);
    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => return value,
            Poll::Pending => std::thread::park(),
        }
    }
}
