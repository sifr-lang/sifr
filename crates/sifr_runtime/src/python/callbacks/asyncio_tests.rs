use super::{
    abandon_callback_owner_after_error_async, asyncio_callback_scoped_with_owner,
    asyncio_callback_with_owner, finalize_retained_callbacks,
    finish_retained_callback_finalization, retained_callback_finalization_scope,
    AsyncioCallbackConcurrency, CallbackExecutionError, CallbackOwnerSlot, CallbackOwnerState,
    CallbackOwnerStatus, RetainedCallbackGroup,
};
use crate::cancellation::{CancellationBind, CancellationCarrier, CancellationRequest};
use crate::python::{
    async_from_int, async_from_object, async_to_int, initialize_runtime,
    reset_runtime_state_for_tests, submit_async_declaration, test_config, test_guard,
    PythonAsyncRequest, PythonAsyncType,
};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyModule};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const MODULE: &str = "__sifr_asyncio_callback_tests__";

#[tokio::test(flavor = "current_thread")]
async fn asyncio_callback_round_trips_on_owned_loop_and_drains_asynchronously() {
    let _guard = test_guard();
    initialize_callback_runtime("asyncio-callback-roundtrip");
    let owner = CallbackOwnerState::new_call_scoped().expect("owner should create");
    let callback = asyncio_callback_scoped_with_owner(
        owner,
        1,
        1,
        AsyncioCallbackConcurrency::Serial,
        |args| crate::python::to_int(&args[0]),
        |_, value, _| async move { Ok(value + 1) },
        crate::python::from_int,
    )
    .expect("asyncio callback should create");
    let request = function_request(
        "invoke",
        vec![
            async_from_object(callback.object()).expect("callback transport"),
            async_from_int(41).expect("argument transport"),
        ],
    );

    let value = async_to_int(
        submit_async_declaration(request, None)
            .await
            .expect("callback request should complete"),
    )
    .expect("callback result should convert");
    assert_eq!(value, 42);
    callback
        .close_call_scope()
        .await
        .expect("async callback owner should drain");
    reset_runtime_state_for_tests();
}

#[tokio::test(flavor = "current_thread")]
async fn python_future_cancellation_reaches_the_exact_sifr_handler() {
    let _guard = test_guard();
    initialize_callback_runtime("asyncio-callback-cancel");
    let cancellation_seen = Arc::new(tokio::sync::Notify::new());
    let cancellation_seen_by_handler = Arc::clone(&cancellation_seen);
    let owner = CallbackOwnerState::new_call_scoped().expect("owner should create");
    let callback = asyncio_callback_scoped_with_owner(
        owner,
        2,
        1,
        AsyncioCallbackConcurrency::Parallel,
        |args| crate::python::to_int(&args[0]),
        move |_, value, cancellation: CancellationCarrier| {
            let cancellation_seen = Arc::clone(&cancellation_seen_by_handler);
            async move {
                let signal = Arc::clone(&cancellation_seen);
                let _claim = cancellation
                    .claim(Arc::new(move || signal.notify_one()))
                    .map_err(|_| {
                        CallbackExecutionError::Infrastructure(
                            crate::python::PythonError::without_replay(
                                "callback",
                                "RuntimeError",
                                "callback cancellation claim failed",
                                String::new(),
                                "asyncio callback cancellation",
                            ),
                        )
                    })?;
                cancellation_seen.notified().await;
                Ok(value)
            }
        },
        crate::python::from_int,
    )
    .expect("asyncio callback should create");
    let request = function_request(
        "cancel",
        vec![async_from_object(callback.object()).expect("callback transport")],
    );

    let value = async_to_int(
        submit_async_declaration(request, None)
            .await
            .expect("Python cancellation should be observed and handled"),
    )
    .expect("cancellation result should convert");
    assert_eq!(value, 1);
    callback
        .close_call_scope()
        .await
        .expect("cancelled callback should drain");
    reset_runtime_state_for_tests();
}

#[tokio::test(flavor = "current_thread")]
async fn serial_reentrancy_is_rejected_before_waiting_for_the_fifo() {
    let _guard = test_guard();
    initialize_callback_runtime("asyncio-callback-reentrancy");
    let owner = CallbackOwnerState::new_call_scoped().expect("owner should create");
    let callback = asyncio_callback_scoped_with_owner(
        owner,
        3,
        1,
        AsyncioCallbackConcurrency::Serial,
        |args| crate::python::to_int(&args[0]),
        |_, _value, _| async move {
            let nested = function_request("reenter", Vec::new());
            match submit_async_declaration(nested, None).await {
                Ok(_) => Err(CallbackExecutionError::Infrastructure(
                    crate::python::PythonError::without_replay(
                        "callback",
                        "RuntimeError",
                        "serial callback unexpectedly reentered",
                        String::new(),
                        "asyncio callback reentrancy",
                    ),
                )),
                Err(error) => Err(CallbackExecutionError::Infrastructure(error)),
            }
        },
        crate::python::from_int,
    )
    .expect("asyncio callback should create");
    let request = function_request(
        "install_and_invoke",
        vec![async_from_object(callback.object()).expect("callback transport")],
    );

    let error = submit_async_declaration(request, None)
        .await
        .expect_err("serial recursion should fail instead of deadlocking");
    assert_eq!(error.exception_type, "SifrCallbackReentrancyError");
    callback
        .close_call_scope()
        .await
        .expect("reentrant callback should drain");
    reset_runtime_state_for_tests();
}

#[tokio::test(flavor = "current_thread")]
async fn call_scoped_close_cancels_and_joins_an_active_asyncio_callback() {
    let _guard = test_guard();
    initialize_callback_runtime("asyncio-callback-call-close");
    let started = Arc::new(tokio::sync::Notify::new());
    let release = Arc::new(tokio::sync::Notify::new());
    let started_by_handler = Arc::clone(&started);
    let release_by_handler = Arc::clone(&release);
    let owner = CallbackOwnerState::new_call_scoped().expect("owner should create");
    let callback = asyncio_callback_scoped_with_owner(
        owner.clone(),
        4,
        1,
        AsyncioCallbackConcurrency::Parallel,
        |args| crate::python::to_int(&args[0]),
        move |_, value, _| {
            let started = Arc::clone(&started_by_handler);
            let release = Arc::clone(&release_by_handler);
            async move {
                started.notify_one();
                release.notified().await;
                Ok(value)
            }
        },
        crate::python::from_int,
    )
    .expect("asyncio callback should create");
    let request = function_request(
        "invoke_catching_cancel",
        vec![
            async_from_object(callback.object()).expect("callback transport"),
            async_from_int(41).expect("argument transport"),
        ],
    );

    let controller = async {
        started.notified().await;
        callback.close_call_scope().await
    };
    let (invocation, close) = tokio::join!(submit_async_declaration(request, None), controller);
    close.expect("call-scoped close should cancel and join the invocation");
    let value = async_to_int(invocation.expect("Python should observe callback cancellation"))
        .expect("cancellation result should convert");
    assert_eq!(value, -1);
    assert_eq!(owner.status(), CallbackOwnerStatus::Closed);
    reset_runtime_state_for_tests();
}

#[tokio::test(flavor = "current_thread")]
async fn retained_asyncio_rollback_cancels_and_joins_without_blocking_the_executor() {
    let _guard = test_guard();
    initialize_callback_runtime("asyncio-callback-retained-rollback");
    let started = Arc::new(tokio::sync::Notify::new());
    let release = Arc::new(tokio::sync::Notify::new());
    let started_by_handler = Arc::clone(&started);
    let release_by_handler = Arc::clone(&release);
    let mut group = RetainedCallbackGroup::new().expect("retained group should create");
    let callback = asyncio_callback_with_owner(
        group.owner().clone(),
        5,
        1,
        AsyncioCallbackConcurrency::Parallel,
        |args| crate::python::to_int(&args[0]),
        move |_, value, _| {
            let started = Arc::clone(&started_by_handler);
            let release = Arc::clone(&release_by_handler);
            async move {
                started.notify_one();
                release.notified().await;
                Ok(value + 1)
            }
        },
        crate::python::from_int,
    )
    .expect("retained asyncio callback should create");
    callback
        .retain_in_owner()
        .expect("retained callback should transfer into its owner");
    let request = function_request(
        "invoke_catching_cancel",
        vec![
            async_from_object(callback.object()).expect("callback transport"),
            async_from_int(41).expect("argument transport"),
        ],
    );

    let controller = async {
        started.notified().await;
        group.rollback_async().await
    };
    let (invocation, rollback) = tokio::join!(submit_async_declaration(request, None), controller);
    rollback.expect("retained rollback should cancel and join the invocation");
    let value = async_to_int(invocation.expect("Python should observe callback cancellation"))
        .expect("cancellation result should convert");
    assert_eq!(value, -1);
    reset_runtime_state_for_tests();
}

#[tokio::test(flavor = "current_thread")]
async fn dropped_retained_group_drains_without_blocking_the_executor() {
    let _guard = test_guard();
    initialize_callback_runtime("asyncio-callback-retained-drop");
    let started = Arc::new(tokio::sync::Notify::new());
    let release = Arc::new(tokio::sync::Notify::new());
    let started_by_handler = Arc::clone(&started);
    let release_by_handler = Arc::clone(&release);
    let group = RetainedCallbackGroup::new().expect("retained group should create");
    let owner = group.owner().clone();
    let callback = asyncio_callback_with_owner(
        owner.clone(),
        6,
        1,
        AsyncioCallbackConcurrency::Parallel,
        |args| crate::python::to_int(&args[0]),
        move |_, value, _| {
            let started = Arc::clone(&started_by_handler);
            let release = Arc::clone(&release_by_handler);
            async move {
                started.notify_one();
                release.notified().await;
                Ok(value + 1)
            }
        },
        crate::python::from_int,
    )
    .expect("retained asyncio callback should create");
    callback
        .retain_in_owner()
        .expect("retained callback should transfer into its owner");
    let request = function_request(
        "invoke_catching_cancel",
        vec![
            async_from_object(callback.object()).expect("callback transport"),
            async_from_int(41).expect("argument transport"),
        ],
    );

    let controller = async {
        started.notified().await;
        drop(group);
    };
    let (invocation, ()) = tokio::join!(submit_async_declaration(request, None), controller);
    let value = async_to_int(invocation.expect("Python should observe callback cancellation"))
        .expect("cancellation result should convert");
    assert_eq!(value, -1);
    assert_eq!(owner.status(), CallbackOwnerStatus::Closed);
    reset_runtime_state_for_tests();
}

#[tokio::test(flavor = "current_thread")]
async fn retained_finalization_resumes_native_cancellation_only_after_rollback() {
    let _guard = test_guard();
    initialize_callback_runtime("asyncio-callback-retained-cancellation-mask");
    let started = Arc::new(tokio::sync::Notify::new());
    let started_by_handler = Arc::clone(&started);
    let mut group = RetainedCallbackGroup::new().expect("retained group should create");
    let owner = group.owner().clone();
    let callback = asyncio_callback_with_owner(
        owner.clone(),
        7,
        1,
        AsyncioCallbackConcurrency::Parallel,
        |args| crate::python::to_int(&args[0]),
        move |_, value, cancellation: CancellationCarrier| {
            let started = Arc::clone(&started_by_handler);
            async move {
                started.notify_one();
                let notification = Arc::new(tokio::sync::Notify::new());
                let signal = Arc::clone(&notification);
                let _claim = cancellation
                    .claim(Arc::new(move || signal.notify_one()))
                    .map_err(|_| {
                        CallbackExecutionError::Infrastructure(
                            crate::python::PythonError::without_replay(
                                "callback",
                                "RuntimeError",
                                "callback cancellation claim failed",
                                String::new(),
                                "retained callback cancellation",
                            ),
                        )
                    })?;
                notification.notified().await;
                Ok(value)
            }
        },
        crate::python::from_int,
    )
    .expect("retained asyncio callback should create");
    callback
        .retain_in_owner()
        .expect("retained callback should transfer into its owner");

    let parent = CancellationCarrier::new();
    let resumed_after_close = Arc::new(AtomicBool::new(false));
    let resumed = Arc::clone(&resumed_after_close);
    let owner_at_resume = owner.clone();
    assert_eq!(
        parent.bind_fallback(Arc::new(move || {
            resumed.store(
                owner_at_resume.status() == CallbackOwnerStatus::Closed,
                Ordering::SeqCst,
            );
        })),
        CancellationBind::Bound
    );
    let scope = retained_callback_finalization_scope(Some(&parent))
        .expect("cancellation scope should create")
        .expect("parent carrier should produce a scope");
    let child = scope.child().clone();
    let request = function_request(
        "invoke",
        vec![
            async_from_object(callback.object()).expect("callback transport"),
            async_from_int(41).expect("argument transport"),
        ],
    );

    let finalization = async {
        let outcome = submit_async_declaration(request, Some(&child)).await;
        let outcome = finalize_retained_callbacks(outcome, &mut group).await;
        finish_retained_callback_finalization(outcome, Some(scope)).await
    };
    let cancel = async {
        started.notified().await;
        assert_eq!(parent.request_cancel(), CancellationRequest::Claimed);
    };
    let (outcome, ()) = tokio::join!(finalization, cancel);
    outcome.expect_err("native cancellation should terminate the retained operation");
    assert_eq!(owner.status(), CallbackOwnerStatus::Closed);
    assert!(resumed_after_close.load(Ordering::SeqCst));
    reset_runtime_state_for_tests();
}

#[tokio::test(flavor = "current_thread")]
async fn retained_finalization_releases_claim_before_a_late_cancellation_request() {
    let parent = CancellationCarrier::new();
    let resumed = Arc::new(AtomicBool::new(false));
    let resumed_by_fallback = Arc::clone(&resumed);
    assert_eq!(
        parent.bind_fallback(Arc::new(move || {
            resumed_by_fallback.store(true, Ordering::SeqCst);
        })),
        CancellationBind::Bound
    );
    let scope = retained_callback_finalization_scope(Some(&parent))
        .expect("scope should create")
        .expect("scope should be present");

    finish_retained_callback_finalization((), Some(scope)).await;
    assert_eq!(parent.request_cancel(), CancellationRequest::Fallback);
    assert!(resumed.load(Ordering::SeqCst));
}

#[tokio::test(flavor = "current_thread")]
async fn failed_async_context_enter_drains_active_owner_without_running_exit() {
    let unregisters = Arc::new(AtomicBool::new(false));
    let unregisters_by_owner = Arc::clone(&unregisters);
    let owner = CallbackOwnerState::new_retained(move || {
        unregisters_by_owner.store(true, Ordering::SeqCst);
        Ok(())
    })
    .expect("owner should create");
    let active = owner.accept(1, false).expect("callback should enter");
    owner.record_failure(
        active.entry_sequence(),
        "HandlerError",
        "async enter failure",
    );
    let release = tokio::spawn(async move {
        tokio::task::yield_now().await;
        drop(active);
    });
    let primary = crate::python::PythonError::without_replay(
        "fixture",
        "EnterError",
        "async context entry failed",
        String::new(),
        "async context enter",
    );
    let error = abandon_callback_owner_after_error_async(
        primary,
        &CallbackOwnerSlot::from_owner(owner.clone()),
    )
    .await;
    release.await.expect("callback release should join");

    assert_eq!(owner.status(), CallbackOwnerStatus::Closed);
    assert!(!unregisters.load(Ordering::SeqCst));
    assert!(error.context.contains("HandlerError"), "{error:?}");
}

#[tokio::test(flavor = "current_thread")]
async fn provisional_receiver_rollback_cancels_entries_and_releases_target() {
    let _guard = test_guard();
    initialize_callback_runtime("asyncio-callback-provisional-receiver");
    let owner = CallbackOwnerState::new_retained(|| Ok(())).expect("owner should create");
    let capture = Arc::new(());
    let handler_capture = Arc::clone(&capture);
    let started = Arc::new(tokio::sync::Notify::new());
    let started_by_handler = Arc::clone(&started);
    let callback = asyncio_callback_with_owner(
        owner.clone(),
        8,
        1,
        AsyncioCallbackConcurrency::Parallel,
        |args| crate::python::to_int(&args[0]),
        move |_, value, _| {
            let _capture = Arc::clone(&handler_capture);
            let started = Arc::clone(&started_by_handler);
            async move {
                started.notify_one();
                std::future::pending::<()>().await;
                Ok(value)
            }
        },
        crate::python::from_int,
    )
    .expect("receiver callback should create");
    let request = function_request(
        "start_and_fail",
        vec![async_from_object(callback.object()).expect("callback transport")],
    );

    let error = submit_async_declaration(request, None)
        .await
        .expect_err("registration should fail after starting the callback");
    assert_eq!(error.exception_type, "RuntimeError");
    started.notified().await;
    callback
        .rollback_provisional()
        .await
        .expect("provisional callback should cancel and join");

    assert_eq!(owner.active_calls(), 0);
    assert_eq!(Arc::strong_count(&capture), 1);
    let unregister = owner
        .begin_owner_unregister()
        .expect("unregister should begin");
    drop(unregister);
    owner
        .close_after_owner_unregister_async()
        .await
        .expect("owner should close");
    reset_runtime_state_for_tests();
}

fn initialize_callback_runtime(label: &str) {
    reset_runtime_state_for_tests();
    let mut config = test_config(label);
    config.start_async_loop = true;
    config.required_import_roots.push(MODULE.to_string());
    config.trusted_import_roots.push(MODULE.to_string());
    initialize_runtime(config).expect("runtime should initialize");
    crate::python::attach(|py| install_module(py)).expect("runtime should attach");
}

fn install_module(py: Python<'_>) {
    let module = PyModule::from_code(
        py,
        c"import asyncio\n\nstored = None\n\nasync def invoke(callback, value):\n    return await callback(value)\n\nasync def invoke_catching_cancel(callback, value):\n    try:\n        return await callback(value)\n    except asyncio.CancelledError:\n        return -1\n\nasync def cancel(callback):\n    task = asyncio.ensure_future(callback(1))\n    await asyncio.sleep(0)\n    task.cancel()\n    try:\n        await task\n    except asyncio.CancelledError:\n        return 1\n    return 0\n\nasync def install_and_invoke(callback):\n    global stored\n    stored = callback\n    return await callback(1)\n\nasync def start_and_fail(callback):\n    asyncio.ensure_future(callback(1))\n    await asyncio.sleep(0)\n    raise RuntimeError('registration failed')\n\nasync def reenter():\n    return await stored(2)\n",
        c"__sifr_asyncio_callback_tests__.py",
        c"__sifr_asyncio_callback_tests__",
    )
    .expect("callback test module should compile");
    py.import("sys")
        .and_then(|sys| sys.getattr("modules"))
        .and_then(|modules| modules.cast_into::<PyDict>().map_err(Into::into))
        .and_then(|modules| modules.set_item(MODULE, module))
        .expect("callback test module should register");
}

fn function_request(
    member: &str,
    args: Vec<crate::python::PythonAsyncValue>,
) -> PythonAsyncRequest {
    PythonAsyncRequest::function(
        vec![MODULE.to_string(), member.to_string()],
        args,
        Vec::new(),
        PythonAsyncType::Int,
    )
}
