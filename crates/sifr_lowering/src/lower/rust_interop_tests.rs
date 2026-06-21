use crate::{lower_module, HirDiagnostic, HirModule};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{RustInteropDecoratorKind, RustInteropEffect, RustInteropValue};
use sifr_python_parser::parse_module;

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
    assert!(errors
        .iter()
        .any(|error| error.code == Some(DiagnosticCode::RUST_CONFIG_MALFORMED_DECORATOR)));
}

#[test]
fn rust_interop_lowers_function_decorators_into_hir() {
    let module = lower_ok(
        r"
@rust(bridge.hash.digest, panic=map_error(bridge.hash.map_panic))
@rust.zero_copy(owner=input, view=bridge.hash.DigestView)
@rust.view(lifetime=borrowed, mutable=False)
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
fn rust_interop_rejects_async_decorator_on_sync_function() {
    let errors = lower_errors(
        r"
@rust.async(thread_affinity=tokio_current_thread)
def digest(input: bytes) -> int:
    return 1
",
    );

    assert!(errors
        .iter()
        .any(|error| error.code == Some(DiagnosticCode::RUST_ASYNC_CONTRACT)));
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
    assert!(function
        .rust_interop
        .iter()
        .all(|declaration| { declaration.effect == RustInteropEffect::Async }));
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

    assert!(errors
        .iter()
        .any(|error| error.code == Some(DiagnosticCode::RUST_ASYNC_CONTRACT)));
}

#[test]
fn rust_interop_lowers_opaque_class_and_self_method_targets() {
    let module = lower_ok(
        r"
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
    def poll(self) -> int:
        return 1
",
    );

    let class = &module.classes[0];
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
fn rust_interop_allows_self_targets_in_method_keyword_values() {
    let module = lower_ok(
        r"
@rust.opaque(type=bridge.kafka.Consumer)
class Consumer:
    @rust(Self.poll, view=Self.PollView)
    def poll(self) -> int:
        return 1
",
    );

    let method = &module.classes[0].methods[0];
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

    assert!(module.functions[0].rust_interop[0]
        .arguments
        .iter()
        .any(|arg| arg.name.as_deref() == Some("retry")
            && arg.value == RustInteropValue::Integer(-1)));
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

#[test]
fn rust_interop_rejects_prior_draft_keyword_syntax() {
    let errors = lower_errors(
        r"
@rust(crate=crc32fast, path=hash)
def digest(input: bytes) -> bytes:
    return input
",
    );

    assert_malformed(&errors);
}

#[test]
fn rust_interop_rejects_self_target_outside_methods() {
    let errors = lower_errors(
        r"
@rust(Self.poll)
def poll() -> int:
    return 1
",
    );

    assert_malformed(&errors);
}

#[test]
fn rust_interop_rejects_opaque_on_functions() {
    let errors = lower_errors(
        r"
@rust.opaque(type=bridge.kafka.Consumer)
def digest(input: bytes) -> bytes:
    return input
",
    );

    assert_malformed(&errors);
}

#[test]
fn rust_interop_rejects_function_decorators_on_classes() {
    let errors = lower_errors(
        r"
@rust.async()
class Consumer:
    pass
",
    );

    assert_malformed(&errors);
}

#[test]
fn rust_interop_rejects_unknown_decorator_names() {
    let errors = lower_errors(
        r"
@rust.unknown()
def digest(input: bytes) -> bytes:
    return input
",
    );

    assert_malformed(&errors);
}

#[test]
fn rust_interop_rejects_double_star_keyword_splat() {
    let errors = lower_errors(
        r"
@rust(bridge.hash.digest, **options)
def digest(input: bytes) -> bytes:
    return input
",
    );

    assert_malformed(&errors);
}

#[test]
fn rust_interop_rejects_bare_rust_decorators() {
    let errors = lower_errors(
        r"
@rust
def digest(input: bytes) -> bytes:
    return input
",
    );

    assert_malformed(&errors);
}
