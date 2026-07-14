use super::*;
use crate::cancellation::{CancellationBind, CancellationCarrier, CancellationRequest};
use pyo3::types::{PyDict, PyModule};
use std::future::Future;
use std::pin::pin;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};

const MODULE: &str = "__sifr_typed_async__";

#[test]
fn typed_function_converts_complete_frames_and_recursive_results_on_owned_loop() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_typed_runtime("typed-frame");

    let request = PythonAsyncRequest::function(
        vec![MODULE.to_string(), "collect".to_string()],
        vec![
            async_from_int(3).expect("int transport"),
            async_from_int(4).expect("variadic transport"),
            async_from_int(5).expect("variadic transport"),
        ],
        vec![
            (
                "label".to_string(),
                async_from_str("ready").expect("label transport"),
            ),
            (
                "bonus".to_string(),
                async_from_int(6).expect("keyword variadic transport"),
            ),
        ],
        PythonAsyncType::Record(vec![
            ("total".to_string(), PythonAsyncType::Int),
            ("label".to_string(), PythonAsyncType::Str),
            (
                "nested".to_string(),
                PythonAsyncType::List(Box::new(PythonAsyncType::Int)),
            ),
            ("identity".to_string(), PythonAsyncType::Str),
        ]),
    );
    let mut result = block_on(submit_async_declaration(request, None))
        .expect("typed declaration should complete");

    assert_eq!(
        async_to_int(async_record_field(&mut result, "total").expect("total field"))
            .expect("total int"),
        18
    );
    assert_eq!(
        async_to_str(async_record_field(&mut result, "label").expect("label field"))
            .expect("label str"),
        "ready"
    );
    let nested = async_list_items(async_record_field(&mut result, "nested").expect("nested field"))
        .expect("nested list")
        .into_iter()
        .map(async_to_int)
        .collect::<Result<Vec<_>, _>>()
        .expect("nested ints");
    assert_eq!(nested, vec![3, 4, 5, 6]);
    let identity =
        async_to_str(async_record_field(&mut result, "identity").expect("identity field"))
            .expect("identity str");
    assert!(identity.contains(':'));

    shutdown_typed_runtime();
}

#[test]
fn typed_factory_and_borrowed_method_preserve_sealed_identity_across_await() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_typed_runtime("typed-method");

    let factory = PythonAsyncRequest::function(
        vec![MODULE.to_string(), "make_client".to_string()],
        vec![async_from_int(41).expect("factory input")],
        Vec::new(),
        PythonAsyncType::Opaque(vec![MODULE.to_string(), "Client".to_string()]),
    );
    let client = async_to_object(
        block_on(submit_async_declaration(factory, None)).expect("factory should complete"),
    )
    .expect("factory should produce an owned identity");
    let request = PythonAsyncRequest::borrowed_method(
        &client,
        "increment".to_string(),
        vec![async_from_int(1).expect("method input")],
        Vec::new(),
        PythonAsyncType::Int,
    )
    .expect("borrowed receiver should freeze");

    // Runtime cleanup may mark the public handle closed, but the compiler-private
    // lease pins the exact identity until the in-flight request reaches terminal.
    close_object(client).expect("public identity should close");
    let value =
        block_on(submit_async_declaration(request, None)).expect("leased method should complete");
    assert_eq!(async_to_int(value).expect("method result"), 42);

    shutdown_typed_runtime();
}

#[test]
fn typed_failures_and_concurrent_calls_use_one_terminal_registry_and_loop() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_typed_runtime("typed-failures");

    let identity_request = || {
        PythonAsyncRequest::function(
            vec![MODULE.to_string(), "identity".to_string()],
            Vec::new(),
            Vec::new(),
            PythonAsyncType::Str,
        )
    };
    let first =
        std::thread::spawn(move || block_on(submit_async_declaration(identity_request(), None)));
    let second =
        std::thread::spawn(move || block_on(submit_async_declaration(identity_request(), None)));
    let first = first.join().expect("first worker should not panic");
    let second = second.join().expect("second worker should not panic");
    assert_eq!(
        async_to_str(first.expect("first identity")).expect("first str"),
        async_to_str(second.expect("second identity")).expect("second str")
    );
    assert_eq!(
        async_runtime_diagnostics().expect("async diagnostics"),
        PythonAsyncRuntimeDiagnostics {
            running: true,
            stopping: false,
            loop_threads: 1,
            active_submissions: 0,
            pending_submissions: 0,
        }
    );

    let non_awaitable = PythonAsyncRequest::function(
        vec![MODULE.to_string(), "not_awaitable".to_string()],
        Vec::new(),
        Vec::new(),
        PythonAsyncType::Int,
    );
    let error = block_on(submit_async_declaration(non_awaitable, None))
        .expect_err("non-awaitable should fail");
    assert!(error.message.contains("non-awaitable"));

    let python_failure = PythonAsyncRequest::function(
        vec![MODULE.to_string(), "fails".to_string()],
        Vec::new(),
        Vec::new(),
        PythonAsyncType::Int,
    );
    let error = block_on(submit_async_declaration(python_failure, None))
        .expect_err("Python exception should fail");
    assert_eq!(error.exception_type, "ValueError");

    let independent_cancellation = PythonAsyncRequest::function(
        vec![MODULE.to_string(), "raises_cancelled".to_string()],
        Vec::new(),
        Vec::new(),
        PythonAsyncType::None,
    );
    let error = block_on(submit_async_declaration(independent_cancellation, None))
        .expect_err("independently raised CancelledError should remain a Python failure");
    assert!(error.exception_type.contains("CancelledError"), "{error:?}");

    let conversion_failure = PythonAsyncRequest::function(
        vec![MODULE.to_string(), "identity".to_string()],
        Vec::new(),
        Vec::new(),
        PythonAsyncType::Int,
    );
    let error = block_on(submit_async_declaration(conversion_failure, None))
        .expect_err("conversion should fail");
    assert_eq!(error.kind, "conversion");

    shutdown_typed_runtime();
}

#[test]
fn typed_bridge_target_resolves_inside_owned_loop() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    let mut config = test_config("typed-bridge");
    config.start_async_loop = true;
    config.bridge_sources.extend([
        PythonBridgeSource {
            module: "__sifr_bridge__".to_string(),
            source: String::new(),
            filename: "<typed-bridge-root>".to_string(),
            is_package: true,
            package_prefix: "__sifr_bridge__".to_string(),
        },
        PythonBridgeSource {
            module: "__sifr_bridge__.p_typed".to_string(),
            source: String::new(),
            filename: "<typed-bridge-package>".to_string(),
            is_package: true,
            package_prefix: "__sifr_bridge__.p_typed".to_string(),
        },
        PythonBridgeSource {
            module: "__sifr_bridge__.p_typed.adapter".to_string(),
            source: "async def value():\n    return {'answer': 73}\n".to_string(),
            filename: "<typed-bridge>".to_string(),
            is_package: false,
            package_prefix: "__sifr_bridge__.p_typed".to_string(),
        },
    ]);
    initialize_runtime(config).expect("bridge runtime should initialize");

    let request = PythonAsyncRequest::function(
        vec![
            "__sifr_bridge__".to_string(),
            "p_typed".to_string(),
            "adapter".to_string(),
            "value".to_string(),
        ],
        Vec::new(),
        Vec::new(),
        PythonAsyncType::Dict(Box::new(PythonAsyncType::Int)),
    );
    let result = block_on(submit_async_declaration(request, None))
        .expect("bridge declaration should complete");
    let items = async_dict_items(result).expect("bridge dict");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].0, "answer");

    shutdown_typed_runtime();
}

#[test]
fn semantic_async_close_is_exact_once_and_seals_retained_alias() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_typed_runtime("semantic-close-success");
    let client = close_client("success");
    let alias = client.clone();
    let request = PythonAsyncRequest::semantic_close_method(client, "aclose".to_string())
        .expect("open identity should begin semantic close");
    let closing = async_from_object(&alias).expect_err("closing identity must reject new leases");
    assert!(closing.message.contains("closing"), "{closing:?}");
    assert!(
        PythonAsyncRequest::semantic_close_method(alias.clone(), "aclose".to_string()).is_err(),
        "closing identity must reject a second semantic close"
    );

    let result =
        block_on(submit_async_declaration(request, None)).expect("semantic close should complete");
    async_to_none(result).expect("close should return None");

    assert_eq!(close_events(), vec!["success:start", "success:finally"]);
    let error = async_from_object(&alias).expect_err("closed alias must reject reuse");
    assert!(error.message.contains("closed"), "{error:?}");
    assert!(
        PythonAsyncRequest::semantic_close_method(alias, "aclose".to_string()).is_err(),
        "duplicate close must be rejected"
    );
    shutdown_typed_runtime();
}

#[test]
fn semantic_async_close_failures_poison_identity() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_typed_runtime("semantic-close-failures");

    let abandoned = close_client("success");
    let abandoned_alias = abandoned.clone();
    let abandoned_request =
        PythonAsyncRequest::semantic_close_method(abandoned, "aclose".to_string())
            .expect("open identity should begin semantic close");
    drop(abandoned_request);
    let poisoned =
        async_from_object(&abandoned_alias).expect_err("dropped close request must poison");
    assert!(poisoned.message.contains("poisoned"), "{poisoned:?}");

    for (mode, member, expected) in [
        ("fail", "aclose", "close boom"),
        ("non_none", "aclose", "expected Python None"),
        ("non_awaitable", "not_awaitable", "non-awaitable"),
    ] {
        let client = close_client(mode);
        let alias = client.clone();
        let request = PythonAsyncRequest::semantic_close_method(client, member.to_string())
            .expect("open identity should begin semantic close");
        let error = block_on(submit_async_declaration(request, None))
            .expect_err("failed semantic close should return an error");
        assert!(error.message.contains(expected), "{mode}: {error:?}");
        let poisoned = async_from_object(&alias).expect_err("failed close must poison identity");
        assert!(poisoned.message.contains("poisoned"), "{poisoned:?}");
        assert!(
            PythonAsyncRequest::semantic_close_method(alias, member.to_string()).is_err(),
            "poisoned identity must reject duplicate close"
        );
    }
    shutdown_typed_runtime();
}

#[test]
fn semantic_async_close_uses_python_terminal_outcome_after_cancellation() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_typed_runtime("semantic-close-cancellation");

    let suppressed = close_client("suppress");
    let suppressed_alias = suppressed.clone();
    let suppressed_carrier = CancellationCarrier::new();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Tokio test runtime should build");
    runtime.block_on(async {
        let carrier = suppressed_carrier.clone();
        let suppressed_worker = tokio::spawn(async move {
            let request =
                PythonAsyncRequest::semantic_close_method(suppressed, "aclose".to_string())
                    .expect("suppressed close request");
            submit_async_declaration(request, Some(&carrier)).await
        });
        bind_abort_fallback(&suppressed_carrier, &suppressed_worker);
        tokio::task::yield_now().await;
        wait_for_typed_submission();
        assert_eq!(
            suppressed_carrier.request_cancel(),
            CancellationRequest::Claimed
        );
        async_to_none(
            suppressed_worker
                .await
                .expect("suppressed worker should not be cancelled")
                .expect("suppressed cancellation should return normally"),
        )
        .expect("suppressed close should return None");
    });
    let closed = async_from_object(&suppressed_alias).expect_err("successful close is sealed");
    assert!(closed.message.contains("closed"), "{closed:?}");

    let observed = close_client("wait");
    let observed_alias = observed.clone();
    let observed_carrier = CancellationCarrier::new();
    runtime.block_on(async {
        let carrier = observed_carrier.clone();
        let observed_worker = tokio::spawn(async move {
            let request = PythonAsyncRequest::semantic_close_method(observed, "aclose".to_string())
                .expect("observed close request");
            submit_async_declaration(request, Some(&carrier)).await
        });
        bind_abort_fallback(&observed_carrier, &observed_worker);
        tokio::task::yield_now().await;
        wait_for_typed_submission();
        assert_eq!(
            observed_carrier.request_cancel(),
            CancellationRequest::Claimed
        );
        let cancelled = observed_worker
            .await
            .expect_err("observed Python cancellation should abort the native task");
        assert!(cancelled.is_cancelled(), "{cancelled:?}");
    });
    let poisoned = async_from_object(&observed_alias).expect_err("cancelled close is poisoned");
    assert!(poisoned.message.contains("poisoned"), "{poisoned:?}");
    assert!(close_events().iter().any(|event| event == "wait:finally"));

    let suppressed_failure = close_client("suppress_fail");
    let suppressed_failure_alias = suppressed_failure.clone();
    let suppressed_failure_carrier = CancellationCarrier::new();
    runtime.block_on(async {
        let carrier = suppressed_failure_carrier.clone();
        let worker = tokio::spawn(async move {
            let request =
                PythonAsyncRequest::semantic_close_method(suppressed_failure, "aclose".to_string())
                    .expect("suppression failure close request");
            submit_async_declaration(request, Some(&carrier)).await
        });
        bind_abort_fallback(&suppressed_failure_carrier, &worker);
        tokio::task::yield_now().await;
        wait_for_typed_submission();
        assert_eq!(
            suppressed_failure_carrier.request_cancel(),
            CancellationRequest::Claimed
        );
        let error = worker
            .await
            .expect("suppressed cancellation exception should not cancel native task")
            .expect_err("later Python exception should win");
        assert_eq!(error.exception_type, "ValueError");
    });
    let poisoned = async_from_object(&suppressed_failure_alias)
        .expect_err("suppression followed by failure poisons close identity");
    assert!(poisoned.message.contains("poisoned"), "{poisoned:?}");
    assert!(close_events()
        .iter()
        .any(|event| event == "suppress_fail:finally"));
    shutdown_typed_runtime();
}

#[test]
fn retained_owner_close_drains_before_native_cancellation_resumes() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_typed_runtime("retained-owner-close-cancellation-mask");

    let captures_released = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let captures_released_by_owner = Arc::clone(&captures_released);
    let owner = CallbackOwnerState::new_retained_with_release(
        || Ok(()),
        move || {
            captures_released_by_owner.store(true, std::sync::atomic::Ordering::SeqCst);
        },
    )
    .expect("retained owner should create");
    let active = owner
        .accept(1, false)
        .expect("active retained callback should enter");

    let parent = CancellationCarrier::new();
    let resumed_after_close = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let resumed_after_close_by_fallback = Arc::clone(&resumed_after_close);
    let owner_at_resume = owner.clone();
    let captures_at_resume = Arc::clone(&captures_released);
    assert_eq!(
        parent.bind_fallback(Arc::new(move || {
            resumed_after_close_by_fallback.store(
                owner_at_resume.status() == CallbackOwnerStatus::Closed
                    && captures_at_resume.load(std::sync::atomic::Ordering::SeqCst),
                std::sync::atomic::Ordering::SeqCst,
            );
        })),
        CancellationBind::Bound
    );
    let scope = retained_callback_finalization_scope(Some(&parent))
        .expect("owner-close cancellation scope should create")
        .expect("parent carrier should produce an owner-close scope");
    let child = scope.child().clone();
    let request = PythonAsyncRequest::function(
        vec![MODULE.to_string(), "identity".to_string()],
        Vec::new(),
        Vec::new(),
        PythonAsyncType::Str,
    );

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Tokio test runtime should build");
    runtime.block_on(async {
        let finalization = async {
            let outcome = submit_async_declaration_with_callbacks(
                request,
                Some(&child),
                CallbackOwnerSlot::from_owner(owner.clone()),
            )
            .await;
            finish_retained_callback_finalization(outcome, Some(scope)).await
        };
        let cancel = async {
            for _ in 0..4_000 {
                if owner.status() == CallbackOwnerStatus::Closing {
                    assert_eq!(parent.request_cancel(), CancellationRequest::Claimed);
                    assert!(!resumed_after_close.load(std::sync::atomic::Ordering::SeqCst));
                    drop(active);
                    return;
                }
                tokio::task::yield_now().await;
            }
            panic!("retained owner close did not begin");
        };
        let (outcome, ()) = tokio::join!(finalization, cancel);
        async_to_str(outcome.expect("primary result should survive masked owner close"))
            .expect("identity result should remain typed");
    });

    assert_eq!(owner.status(), CallbackOwnerStatus::Closed);
    assert!(captures_released.load(std::sync::atomic::Ordering::SeqCst));
    assert!(resumed_after_close.load(std::sync::atomic::Ordering::SeqCst));
    shutdown_typed_runtime();
}

#[test]
fn active_cancellation_propagation_failures_are_explicit_and_bounded() {
    let pre_cancelled = CancellationCarrier::new();
    assert_eq!(
        pre_cancelled.request_cancel(),
        CancellationRequest::FallbackPending
    );
    assert!(matches!(
        super::async_runtime::terminal_for_submission(Some(&pre_cancelled)),
        Err(super::async_terminal::PythonTerminalError::ActiveCancellation)
    ));

    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_typed_runtime("active-cancellation-propagation-errors");

    for (mode, bind_noop, expected) in [
        ("wait", false, "FallbackUnavailable"),
        ("wait", true, "fallback returned without terminating"),
    ] {
        let client = close_client(mode);
        let carrier = CancellationCarrier::new();
        if bind_noop {
            assert_eq!(
                carrier.bind_fallback(Arc::new(|| {})),
                CancellationBind::Bound
            );
        }
        let worker_carrier = carrier.clone();
        let worker = std::thread::spawn(move || {
            let request = PythonAsyncRequest::semantic_close_method(client, "aclose".to_string())
                .expect("active cancellation request");
            block_on(submit_async_declaration(request, Some(&worker_carrier)))
        });
        wait_for_typed_submission();
        assert_eq!(carrier.request_cancel(), CancellationRequest::Claimed);
        let error = worker
            .join()
            .expect("propagation worker should not panic")
            .expect_err("malformed fallback must return an explicit runtime error");
        assert!(error.message.contains(expected), "{error:?}");
    }
    shutdown_typed_runtime();
}

fn bind_abort_fallback<T>(carrier: &CancellationCarrier, task: &tokio::task::JoinHandle<T>) {
    let abort = task.abort_handle();
    assert_eq!(
        carrier.bind_fallback(Arc::new(move || abort.abort())),
        CancellationBind::Bound
    );
}

#[test]
fn semantic_async_close_shutdown_and_submission_rejection_poison_safely() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_typed_runtime("semantic-close-shutdown");

    let client = close_client("wait");
    let alias = client.clone();
    let worker = std::thread::spawn(move || {
        let request = PythonAsyncRequest::semantic_close_method(client, "aclose".to_string())
            .expect("shutdown close request");
        block_on(submit_async_declaration(request, None))
    });
    wait_for_typed_submission();
    super::async_runtime::shutdown().expect("shutdown should terminally drain close");
    let error = worker
        .join()
        .expect("shutdown worker should not panic")
        .expect_err("shutdown cancellation should fail close");
    assert!(error.exception_type.contains("CancelledError"), "{error:?}");
    let poisoned = async_from_object(&alias).expect_err("shutdown close must poison identity");
    assert!(poisoned.message.contains("poisoned"), "{poisoned:?}");
    assert!(close_events().iter().any(|event| event == "wait:finally"));
    assert_eq!(
        async_runtime_diagnostics().expect("async diagnostics"),
        PythonAsyncRuntimeDiagnostics::default()
    );

    super::async_runtime::ensure_started().expect("owned loop should restart for rejection race");
    let (held_terminal, pending_id) = attach(|py| {
        let (terminal, _cancellation) = super::async_runtime::terminal_for_submission(None)
            .map_err(|error| {
                super::async_terminal::terminal_error_to_python(
                    py,
                    error,
                    "typed shutdown test reservation",
                )
            })?;
        let (submission_id, _loop_object) = super::async_runtime::reserve_submission(py, &terminal)
            .map_err(PythonError::runtime)?;
        Ok::<_, PythonError>((terminal, submission_id))
    })
    .expect("runtime should attach")
    .expect("pending reservation should succeed");
    let rejected = close_client("success");
    let rejected_alias = rejected.clone();
    let request = PythonAsyncRequest::semantic_close_method(rejected, "aclose".to_string())
        .expect("identity can enter closing before submission rejection");
    let shutdown = std::thread::spawn(super::async_runtime::shutdown);
    wait_for_runtime_stopping();
    block_on(submit_async_declaration(request, None))
        .expect_err("stopping runtime must reject semantic close");
    let poisoned =
        async_from_object(&rejected_alias).expect_err("rejected submission must poison identity");
    assert!(poisoned.message.contains("poisoned"), "{poisoned:?}");
    super::async_runtime::release_pending_submission(pending_id);
    shutdown
        .join()
        .expect("shutdown thread should not panic")
        .expect("shutdown should finish after pending reservation releases");
    drop(held_terminal);
}

fn initialize_typed_runtime(label: &str) {
    let mut config = test_config(label);
    config.start_async_loop = true;
    config.required_import_roots.push(MODULE.to_string());
    config.trusted_import_roots.push(MODULE.to_string());
    initialize_runtime(config).expect("typed runtime should initialize");
    install_module();
}

fn install_module() {
    attach(|py| {
        let module = PyModule::from_code(
            py,
            c"import asyncio\nimport threading\n\nclose_log = []\n\nasync def collect(a, *rest, label=None, **extra):\n    await asyncio.sleep(0)\n    values = [a, *rest, *extra.values()]\n    return {'total': sum(values), 'label': label, 'nested': values, 'identity': f'{id(asyncio.get_running_loop())}:{threading.get_ident()}'}\n\nasync def identity():\n    await asyncio.sleep(0)\n    return f'{id(asyncio.get_running_loop())}:{threading.get_ident()}'\n\ndef not_awaitable():\n    return 1\n\nasync def fails():\n    raise ValueError('typed boom')\n\nasync def raises_cancelled():\n    raise asyncio.CancelledError()\n\nclass Client:\n    def __init__(self, value):\n        self.value = value\n\n    async def increment(self, amount):\n        await asyncio.sleep(0)\n        return self.value + amount\n\nclass CloseClient:\n    def __init__(self, mode):\n        self.mode = mode\n\n    def not_awaitable(self):\n        close_log.append(f'{self.mode}:start')\n        return None\n\n    async def aclose(self):\n        close_log.append(f'{self.mode}:start')\n        try:\n            if self.mode == 'fail':\n                raise ValueError('close boom')\n            if self.mode == 'non_none':\n                return 1\n            if self.mode in ('suppress', 'suppress_fail'):\n                try:\n                    await asyncio.Event().wait()\n                except asyncio.CancelledError:\n                    if self.mode == 'suppress_fail':\n                        raise ValueError('suppressed cancellation failure')\n                    return None\n            if self.mode == 'wait':\n                await asyncio.Event().wait()\n            await asyncio.sleep(0)\n            return None\n        finally:\n            close_log.append(f'{self.mode}:finally')\n\nasync def make_client(value):\n    await asyncio.sleep(0)\n    return Client(value)\n",
            c"<sifr-typed-async>",
            c"__sifr_typed_async__",
        )
        .expect("typed module should compile");
        let modules = py
            .import("sys")
            .expect("sys should import")
            .getattr("modules")
            .expect("sys.modules should resolve")
            .cast_into::<PyDict>()
            .expect("sys.modules should be a dict");
        modules
            .set_item(MODULE, module)
            .expect("typed module should install");
    })
    .expect("runtime should attach");
}

fn close_client(mode: &str) -> ObjectHandle {
    attach(|py| {
        let module = py.import(MODULE).expect("typed module should import");
        let object = module
            .getattr("CloseClient")
            .expect("CloseClient should resolve")
            .call1((mode,))
            .expect("CloseClient should construct");
        super::object_ops::store_object(object.unbind().into())
    })
    .expect("runtime should attach")
    .expect("close client should store")
}

fn close_events() -> Vec<String> {
    attach(|py| {
        py.import(MODULE)?
            .getattr("close_log")?
            .extract::<Vec<String>>()
    })
    .expect("runtime should attach")
    .expect("close log should extract")
}

fn wait_for_typed_submission() {
    for _ in 0..2_000 {
        if async_runtime_diagnostics().is_ok_and(|diagnostics| diagnostics.active_submissions == 1)
        {
            return;
        }
        std::thread::yield_now();
    }
    panic!("typed submission did not register");
}

fn wait_for_runtime_stopping() {
    for _ in 0..2_000 {
        if async_runtime_diagnostics().is_ok_and(|diagnostics| diagnostics.stopping) {
            return;
        }
        std::thread::yield_now();
    }
    panic!("owned asyncio runtime did not enter stopping state");
}

fn shutdown_typed_runtime() {
    super::async_runtime::shutdown().expect("owned loop should stop");
    assert_eq!(
        async_runtime_diagnostics().expect("async diagnostics"),
        PythonAsyncRuntimeDiagnostics::default()
    );
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
