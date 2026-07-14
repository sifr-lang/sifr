use super::{
    asyncio_callback_scoped_with_owner, asyncio_callback_with_owner, AsyncioCallbackConcurrency,
    CallbackExecutionError, CallbackOwnerState, RetainedCallbackGroup,
};
use crate::cancellation::CancellationCarrier;
use crate::python::{
    async_from_int, async_from_object, async_to_int, initialize_runtime,
    reset_runtime_state_for_tests, submit_async_declaration, test_config, test_guard,
    PythonAsyncRequest, PythonAsyncType,
};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyModule};
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
async fn retained_asyncio_rollback_drains_without_blocking_the_executor() {
    let _guard = test_guard();
    initialize_callback_runtime("asyncio-callback-retained-rollback");
    let started = Arc::new(tokio::sync::Notify::new());
    let release = Arc::new(tokio::sync::Notify::new());
    let started_by_handler = Arc::clone(&started);
    let release_by_handler = Arc::clone(&release);
    let mut group = RetainedCallbackGroup::new().expect("retained group should create");
    let callback = asyncio_callback_with_owner(
        group.owner().clone(),
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
        "invoke",
        vec![
            async_from_object(callback.object()).expect("callback transport"),
            async_from_int(41).expect("argument transport"),
        ],
    );

    let controller = async {
        started.notified().await;
        let release_task = tokio::spawn(async move {
            tokio::task::yield_now().await;
            release.notify_one();
        });
        let outcome = group.rollback_async().await;
        release_task.await.expect("release task should complete");
        outcome
    };
    let (invocation, rollback) = tokio::join!(submit_async_declaration(request, None), controller);
    rollback.expect("retained rollback should drain accepted invocation");
    let value = async_to_int(invocation.expect("accepted callback should complete"))
        .expect("callback result should convert");
    assert_eq!(value, 42);
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
        c"import asyncio\n\nstored = None\n\nasync def invoke(callback, value):\n    return await callback(value)\n\nasync def cancel(callback):\n    task = asyncio.ensure_future(callback(1))\n    await asyncio.sleep(0)\n    task.cancel()\n    try:\n        await task\n    except asyncio.CancelledError:\n        return 1\n    return 0\n\nasync def install_and_invoke(callback):\n    global stored\n    stored = callback\n    return await callback(1)\n\nasync def reenter():\n    return await stored(2)\n",
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
