use super::*;
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
            c"import asyncio\nimport threading\n\nasync def collect(a, *rest, label=None, **extra):\n    await asyncio.sleep(0)\n    values = [a, *rest, *extra.values()]\n    return {'total': sum(values), 'label': label, 'nested': values, 'identity': f'{id(asyncio.get_running_loop())}:{threading.get_ident()}'}\n\nasync def identity():\n    await asyncio.sleep(0)\n    return f'{id(asyncio.get_running_loop())}:{threading.get_ident()}'\n\ndef not_awaitable():\n    return 1\n\nasync def fails():\n    raise ValueError('typed boom')\n\nclass Client:\n    def __init__(self, value):\n        self.value = value\n\n    async def increment(self, amount):\n        await asyncio.sleep(0)\n        return self.value + amount\n\nasync def make_client(value):\n    await asyncio.sleep(0)\n    return Client(value)\n",
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
