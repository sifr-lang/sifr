use crate::{HirDiagnostic, HirModule, HirStmt, lower_module};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{RustInteropDecoratorKind, RustInteropEffect, RustInteropValue};
use sifr_python_parser::parse_module;
use sifr_type_system::Type;

fn lower_ok(source: &str) -> HirModule {
    let parsed = parse_module(source).expect("source should parse");
    lower_module(parsed.suite())
        .map(|result| result.module)
        .expect("source should lower")
}

fn lower_errors(source: &str) -> Vec<HirDiagnostic> {
    let parsed = parse_module(source).expect("source should parse");
    match lower_module(parsed.suite()) {
        Ok(_) => panic!("source should fail lowering"),
        Err(errors) => errors,
    }
}

fn assert_malformed(errors: &[HirDiagnostic]) {
    assert!(
        errors
            .iter()
            .any(|error| error.code == Some(DiagnosticCode::RUST_CONFIG_MALFORMED_DECORATOR))
    );
}

#[test]
fn rust_interop_accepts_ellipsis_only_function_stub() {
    let module = lower_ok(
        r"
@rust(bridge.hash.digest, panic=trusted_no_panic)
def digest(input: bytes) -> int:
    ...
",
    );

    let function = &module.functions[0];
    assert!(function.body.is_empty());
    assert_eq!(function.return_type, Type::Int);
    assert_eq!(
        function.rust_interop[0]
            .target
            .as_ref()
            .expect("target")
            .dotted(),
        "bridge.hash.digest"
    );
}

#[test]
fn rust_interop_accepts_ellipsis_only_async_function_stub() {
    let module = lower_ok(
        r"
@rust(bridge.http.fetch)
async def fetch() -> str:
    ...
",
    );

    let function = &module.functions[0];
    assert!(function.is_async);
    assert!(function.body.is_empty());
    assert_eq!(function.return_type, Type::Str);
    assert_eq!(
        function.rust_interop[0]
            .target
            .as_ref()
            .expect("target")
            .dotted(),
        "bridge.http.fetch"
    );
}

#[test]
fn rust_interop_accepts_ellipsis_only_nested_function_stub() {
    let module = lower_ok(
        r"
def outer() -> int:
    @rust(bridge.hash.digest, panic=trusted_no_panic)
    def digest(input: bytes) -> int:
        ...
    return 1
",
    );

    let HirStmt::NestedFunction { func, .. } = &module.functions[0].body[0] else {
        panic!("expected nested function");
    };
    assert!(func.body.is_empty());
    assert_eq!(func.return_type, Type::Int);
    assert_eq!(
        func.rust_interop[0]
            .target
            .as_ref()
            .expect("target")
            .dotted(),
        "bridge.hash.digest"
    );
}

#[test]
fn rust_interop_accepts_ellipsis_only_method_stub() {
    let module = lower_ok(
        r"
class PollError(Error):
    message: str

@rust.opaque(type=bridge.kafka.Consumer)
class Consumer:
    @rust(Self.poll)
    def poll(self) -> Result[int, PollError]:
        ...
",
    );

    let method = &module.classes[1].methods[0];
    assert!(method.body.is_empty());
    assert!(matches!(method.return_type, Type::Result(_, _)));
    assert_eq!(
        method.rust_interop[0]
            .target
            .as_ref()
            .expect("target")
            .dotted(),
        "Self.poll"
    );
}

#[test]
fn rust_interop_rejects_non_interop_ellipsis_function_body() {
    let errors = lower_errors(
        r"
def placeholder() -> int:
    ...
",
    );

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM)
            && error
                .message
                .contains("complete body of a Rust interop declaration")
    }));
}

#[test]
fn rust_interop_does_not_report_non_interop_ellipsis_for_malformed_rust_decorator() {
    let errors = lower_errors(
        r#"
@rust("bridge.hash.digest")
def digest(input: bytes) -> int:
    ...
"#,
    );

    assert_malformed(&errors);
    assert!(errors.iter().all(|error| {
        !error
            .message
            .contains("complete body of a Rust interop declaration")
    }));
}

#[test]
fn rust_interop_rejects_mixed_ellipsis_stub_body() {
    let errors = lower_errors(
        r"
@rust(bridge.hash.digest, panic=trusted_no_panic)
def digest(input: bytes) -> int:
    ...
    return 1
",
    );

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::RUST_CONFIG_MALFORMED_DECORATOR)
            && error.message.contains("exactly one ellipsis statement")
    }));
}

#[test]
fn rust_interop_rejects_ellipsis_expression_outside_stub_path() {
    let errors = lower_errors(
        r"
def placeholder() -> int:
    value = ...
    return 1
",
    );

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM)
            && error
                .message
                .contains("complete body of a Rust interop declaration")
    }));
}

#[test]
fn rust_interop_lowers_function_decorators_into_hir() {
    let module = lower_ok(
        r"
@rust(bridge.hash.digest, panic=map_error(bridge.hash.map_panic))
@rust.zero_copy(owner=input, view=bridge.hash.DigestView)
@rust.view(owner=input, lifetime=owner, mutability=immutable, send=False, sync=False)
def digest(input: bytes) -> int:
    return 1
",
    );

    let function = &module.functions[0];
    assert_eq!(function.rust_interop.len(), 3);
    assert_eq!(
        function.rust_interop[0].kind,
        RustInteropDecoratorKind::Function
    );
    assert_eq!(
        function.rust_interop[0]
            .target
            .as_ref()
            .expect("target")
            .dotted(),
        "bridge.hash.digest"
    );
    assert_eq!(function.rust_interop[0].effect, RustInteropEffect::Sync);
    assert!(function.rust_interop.iter().any(|declaration| {
        declaration.kind == RustInteropDecoratorKind::ZeroCopy
            && declaration.abi_requirements.zero_copy
    }));
    assert!(function.rust_interop.iter().any(|declaration| {
        declaration.kind == RustInteropDecoratorKind::View && declaration.abi_requirements.view
    }));
}

#[test]
fn rust_interop_accepts_builtin_rust_panic_error_result_surface() {
    let module = lower_ok(
        r#"
@rust(bridge.hash.digest)
def digest() -> Result[bytes, RustPanicError]:
    return b"ok"
"#,
    );

    assert!(matches!(
        &module.functions[0].return_type,
        Type::Result(_, err) if matches!(err.as_ref(), Type::Class { name, .. } if name == "RustPanicError")
    ));
}

#[test]
fn rust_interop_rejects_async_decorator_on_sync_function() {
    let errors = lower_errors(
        r"
@rust.async(thread_affinity=tokio_current_thread)
def digest(input: bytes) -> int:
    return 1
",
    );

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::RUST_ASYNC_CONTRACT)
            && error.message
                == "invalid Rust async contract: `@rust.async(...)` requires `async def`"
    }));
}

#[test]
fn rust_interop_lowers_blocking_io_effect_for_sync_rust_function() {
    let module = lower_ok(
        r"
@blocking_io
@rust(bridge.db.query)
def query() -> int:
    return 1
",
    );

    assert_eq!(
        module.functions[0].rust_interop[0].effect,
        RustInteropEffect::BlockingIo
    );
}

#[test]
fn rust_interop_lowers_async_decorator_on_async_function() {
    let module = lower_ok(
        r#"
@rust(bridge.http.fetch)
@rust.async(thread_affinity=tokio_current_thread)
async def fetch(url: str) -> str:
    await task.sleep(0.0)
    return "ok"
"#,
    );

    let function = &module.functions[0];
    assert_eq!(function.rust_interop.len(), 2);
    assert!(
        function
            .rust_interop
            .iter()
            .all(|declaration| { declaration.effect == RustInteropEffect::Async })
    );
    assert!(function.rust_interop.iter().any(|declaration| {
        declaration.kind == RustInteropDecoratorKind::Function
            && declaration.abi_requirements.async_boundary
            && declaration
                .target
                .as_ref()
                .is_some_and(|target| target.dotted() == "bridge.http.fetch")
    }));
    assert!(function.rust_interop.iter().any(|declaration| {
        declaration.kind == RustInteropDecoratorKind::Async
            && declaration.abi_requirements.async_boundary
            && declaration.arguments.iter().any(|argument| {
                argument.name.as_deref() == Some("thread_affinity")
                    && matches!(&argument.value, RustInteropValue::Symbol(value) if value == "tokio_current_thread")
            })
    }));
}

#[test]
fn rust_interop_rejects_blocking_classification_on_async_function() {
    let errors = lower_errors(
        r"
@blocking_io
@rust(bridge.db.query)
async def query() -> int:
    await task.sleep(0.0)
    return 1
",
    );

    assert!(
        errors
            .iter()
            .any(|error| error.code == Some(DiagnosticCode::RUST_ASYNC_CONTRACT))
    );
}

#[test]
fn rust_interop_hidden_blocking_async_resource_evidence_is_rejected() {
    let errors = lower_errors(
        r"
@blocking_io
@rust(sifr_stdlib.async_core.hidden_blocking_wait, panic=trusted_no_panic)
async def hidden_blocking_wait() -> None:
    return None
",
    );

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::RUST_ASYNC_CONTRACT)
            && error.message
                == "invalid Rust async contract: Rust async interop cannot be combined with blocking or CPU-heavy classification"
    }));
}

#[test]
fn rust_interop_lowers_opaque_class_and_self_method_targets() {
    let module = lower_ok(
        r"
class ConsumerError(Error):
    message: str

@rust.opaque(
    type=bridge.kafka.Consumer,
    send=False,
    sync=False,
    clone=custom(bridge.kafka.clone_consumer),
    close=async_close,
    borrow=exclusive,
    thread_affinity=tokio_current_thread,
)
class Consumer:
    @rust(Self.poll)
    def poll(self) -> Result[int, ConsumerError]:
        ...

    @rust(bridge.kafka.aclose)
    async def aclose(own self) -> Result[None, RustPanicError]:
        ...
",
    );

    let class = &module.classes[1];
    assert_eq!(class.rust_interop.len(), 1);
    assert_eq!(class.rust_interop[0].kind, RustInteropDecoratorKind::Opaque);
    assert!(class.rust_interop[0].abi_requirements.opaque_handle);
    assert!(class.rust_interop[0].arguments.iter().any(|arg| {
        arg.name.as_deref() == Some("type")
            && matches!(&arg.value, RustInteropValue::TargetPath(path) if path.dotted() == "bridge.kafka.Consumer")
    }));

    let method = &class.methods[0];
    assert_eq!(
        method.rust_interop[0]
            .target
            .as_ref()
            .expect("target")
            .dotted(),
        "Self.poll"
    );
}

#[test]
fn rust_opaque_async_close_marks_selected_receiver_consuming() {
    let module = lower_ok(
        r"
class ResourceError(Error):
    message: str

@rust.opaque(type=bridge.resources.Resource, close=async_close)
class Resource:
    @rust(bridge.resources.aclose)
    async def aclose(own self) -> Result[None, ResourceError]:
        ...
",
    );

    assert!(module.classes[1].methods[0].rust_interop[0].consumes_receiver);
}

#[test]
fn rust_opaque_async_close_rejects_mismatched_consuming_member() {
    let errors = lower_errors(
        r"
class ResourceError(Error):
    message: str

@rust.opaque(type=bridge.resources.Resource, close=async_close)
class Resource:
    @rust(bridge.resources.close)
    def close(own self) -> Result[None, ResourceError]:
        ...
",
    );

    assert_malformed(&errors);
    assert!(errors.iter().any(|error| {
        error
            .message
            .contains("reserved for the member selected by the class close policy")
    }));
}

#[test]
fn rust_opaque_async_close_rejects_double_close_during_lowering() {
    let errors = lower_errors(
        r"
class ResourceError(Error):
    message: str

@rust.opaque(type=bridge.resources.Resource, close=async_close)
class Resource:
    @rust(bridge.resources.aclose)
    async def aclose(own self) -> Result[None, ResourceError]:
        ...

@rust(bridge.resources.open)
async def open_resource() -> Result[Resource, ResourceError]:
    ...

async def close_twice() -> Result[None, ResourceError]:
    try:
        resource: Resource = await open_resource()
        _first: None = await resource.aclose()
        _second: None = await resource.aclose()
        return None
    except ResourceError as error:
        raise error
",
    );

    assert!(
        errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::OWN_USE_AFTER_MOVE)
                && error.message.contains("use of moved value: 'resource'")
        }),
        "{errors:#?}"
    );
}

#[test]
fn rust_opaque_async_close_rejects_borrowed_call_receiver() {
    let errors = lower_errors(
        r"
class ResourceError(Error):
    message: str

@rust.opaque(type=bridge.resources.Resource, close=async_close)
class Resource:
    @rust(bridge.resources.aclose)
    async def aclose(own self) -> Result[None, ResourceError]:
        ...

async def close_borrowed(resource: Resource) -> Result[None, ResourceError]:
    try:
        return await resource.aclose()
    except ResourceError as error:
        raise error
",
    );

    assert!(
        errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::OWN_BORROWED_PARAMETER_ESCAPES)
                && error
                    .message
                    .contains("cannot consume borrowed parameter 'resource'")
        }),
        "{errors:#?}"
    );
}

#[test]
fn rust_opaque_close_rejects_field_receiver_without_owned_local() {
    let errors = lower_errors(
        r"
class ResourceError(Error):
    message: str

@rust.opaque(type=bridge.resources.Resource, close=close)
class Resource:
    @rust(bridge.resources.close)
    def close(own self) -> Result[None, ResourceError]:
        ...

class Wrapper:
    resource: Resource

    def close_resource(self) -> Result[None, ResourceError]:
        return self.resource.close()
",
    );

    assert!(
        errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::OWN_BORROWED_PARAMETER_ESCAPES)
                && error
                    .message
                    .contains("must consume an owned local binding")
        }),
        "{errors:#?}"
    );
}

#[test]
fn rust_self_target_requires_opaque_class_before_codegen() {
    let errors = lower_errors(
        r"
class Plain:
    @rust(Self.encode, panic=trusted_no_panic)
    def encode(self, text: str) -> str:
        ...
",
    );

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::RUST_CONFIG_MALFORMED_DECORATOR)
            && error.message.contains("require the owning class")
    }));
}

#[test]
fn rust_opaque_self_target_requires_representable_state_error() {
    let errors = lower_errors(
        r"
class ResourceError(Error):
    detail: int

@rust.opaque(type=bridge.resources.Resource)
class Resource:
    @rust(Self.ping, panic=trusted_no_panic)
    def ping(self) -> Result[str, ResourceError]:
        ...
",
    );

    assert!(
        errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::RUST_CONFIG_MALFORMED_DECORATOR)
                && error.message.contains("message-shaped Error result")
        }),
        "{errors:#?}"
    );
}

#[test]
fn rust_opaque_self_target_rejects_python_error_state_mapping() {
    let errors = lower_errors(
        r"
class PythonError(Error):
    message: str
    kind: str
    exception_type: str
    traceback: str
    context: str

@rust.opaque(type=bridge.resources.Resource)
class Resource:
    @rust(Self.ping, panic=trusted_no_panic)
    def ping(self) -> Result[str, PythonError]:
        ...
",
    );

    assert!(
        errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::RUST_CONFIG_MALFORMED_DECORATOR)
                && error.message.contains("message-shaped Error result")
        }),
        "{errors:#?}"
    );
}

#[test]
fn rust_opaque_self_target_requires_regular_instance_method() {
    let errors = lower_errors(
        r"
class ResourceError(Error):
    message: str

@rust.opaque(type=bridge.resources.Resource)
class Resource:
    @staticmethod
    @rust(Self.ping, panic=trusted_no_panic)
    def ping() -> Result[str, ResourceError]:
        ...
",
    );

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::RUST_CONFIG_MALFORMED_DECORATOR)
            && error.message.contains("require regular instance methods")
    }));
}

#[test]
fn passing_opaque_resource_fixtures_use_owned_cleanup_boundaries() {
    let fixtures = [
        (
            "async runtime",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../verification/areas/rust_interop/fixtures/async_runtime_core/positive/stdlib_async_resource_lifecycle.sifr"
            )),
        ),
        (
            "close after use",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../verification/areas/rust_interop/fixtures/close_after_use/positive/closed_handle_error_surface.sifr"
            )),
        ),
        (
            "opaque resource core",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../verification/areas/rust_interop/fixtures/opaque_resource_core/positive/stdlib_handle_close_poison_lifecycle.sifr"
            )),
        ),
        (
            "opaque resource matrix",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../verification/areas/rust_interop/fixtures/opaque_resource_matrix/positive/resource_close_aclose_matrix.sifr"
            )),
        ),
        (
            "opaque tokenizer",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../verification/areas/rust_interop/fixtures/opaque_handle_tokenizer/positive/declared_send_sync_copy_handle.sifr"
            )),
        ),
    ];

    for (name, source) in fixtures {
        let parsed = parse_module(source).unwrap_or_else(|error| panic!("{name}: {error}"));
        lower_module(parsed.suite()).unwrap_or_else(|errors| panic!("{name}: {errors:#?}"));
    }

    let callback_source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../verification/areas/rust_interop/fixtures/callback_subscription_core/positive/signal_subscription_cancel_shutdown.sifr"
    ));
    let parsed = parse_module(callback_source).expect("callback subscription fixture should parse");
    let ownership_errors = match lower_module(parsed.suite()) {
        Ok(_) => Vec::new(),
        Err(errors) => errors
            .into_iter()
            .filter(|error| error.code == Some(DiagnosticCode::OWN_BORROWED_PARAMETER_ESCAPES))
            .collect::<Vec<_>>(),
    };
    assert!(ownership_errors.is_empty(), "{ownership_errors:#?}");
}

#[test]
fn rust_interop_allows_self_targets_in_method_keyword_values() {
    let module = lower_ok(
        r"
class ConsumerError(Error):
    message: str

@rust.opaque(type=bridge.kafka.Consumer)
class Consumer:
    @rust(Self.poll, view=Self.PollView)
    def poll(self) -> Result[int, ConsumerError]:
        ...
",
    );

    let method = &module.classes[1].methods[0];
    assert!(method.rust_interop[0].arguments.iter().any(|arg| {
        arg.name.as_deref() == Some("view")
            && matches!(&arg.value, RustInteropValue::TargetPath(path) if path.dotted() == "Self.PollView")
    }));
}

#[test]
fn rust_interop_lowers_negative_integer_values() {
    let module = lower_ok(
        r"
@rust(bridge.hash.digest, retry=-1)
def digest(input: bytes) -> int:
    return 1
",
    );

    assert!(
        module.functions[0].rust_interop[0]
            .arguments
            .iter()
            .any(|arg| arg.name.as_deref() == Some("retry")
                && arg.value == RustInteropValue::Integer(-1))
    );
}

#[test]
fn rust_interop_lowers_integer_list_values() {
    let module = lower_ok(
        r"
@rust.view(owner=input, lifetime=owner, mutability=immutable, send=True, sync=True, shape=[2, 3], strides=[3, 1])
def tensor(input: bytes) -> int:
    return 1
",
    );

    assert!(
        module.functions[0].rust_interop[0]
            .arguments
            .iter()
            .any(|arg| arg.name.as_deref() == Some("shape")
                && arg.value == RustInteropValue::IntegerList(vec![2, 3]))
    );
    assert!(
        module.functions[0].rust_interop[0]
            .arguments
            .iter()
            .any(|arg| arg.name.as_deref() == Some("strides")
                && arg.value == RustInteropValue::IntegerList(vec![3, 1]))
    );
}

#[test]
fn rust_interop_lowers_callback_policy_contract() {
    let source = r"
@rust.callback(backpressure=bounded(1024), overflow=error, shutdown=drain)
@rust(bridge.events.subscribe)
def subscribe(own callback: Callable[[int], None]) -> None:
    pass
";
    let module = lower_ok(source);
    let func = module
        .functions
        .iter()
        .find(|func| func.name == "subscribe")
        .expect("subscribe should lower");
    assert!(func.rust_interop.iter().any(|declaration| {
        declaration.kind == RustInteropDecoratorKind::Callback
            && declaration.arguments.iter().any(|arg| {
                arg.name.as_deref() == Some("backpressure")
                    && matches!(
                        &arg.value,
                        RustInteropValue::PolicyCall { name, argument, .. }
                            if name == "bounded"
                                && matches!(argument.as_ref(), RustInteropValue::Integer(1024))
                    )
            })
            && declaration.arguments.iter().any(|arg| {
                arg.name.as_deref() == Some("overflow")
                    && matches!(&arg.value, RustInteropValue::Symbol(value) if value == "error")
            })
            && declaration.arguments.iter().any(|arg| {
                arg.name.as_deref() == Some("shutdown")
                    && matches!(&arg.value, RustInteropValue::Symbol(value) if value == "drain")
            })
    }));
}

#[test]
fn rust_interop_rejects_string_target() {
    let errors = lower_errors(
        r#"
@rust("bridge.hash.digest")
def digest(input: bytes) -> bytes:
    return input
"#,
    );

    assert_malformed(&errors);
}

#[path = "rust_interop_tests/decorator_validation_tests.rs"]
mod decorator_validation_tests;
