use super::project_codegen::GeneratedBinaryProject;
use super::rust_interop::{
    apply_package_rust_interop_metadata, PackageRustInteropContext, RustInteropModuleSource,
};
use super::rust_interop_contract_tests::{interop_errors, package_context, set_bridge_roots};
use sifr_codegen::{generate_rust_multi_with_metadata, StdlibCode};
use sifr_package::TrustPolicy;
use std::collections::BTreeMap;
use std::path::PathBuf;

const CALLBACK_SOURCE: &str = r#"
class CallbackError(Error):
    message: str

class Subscription:
    id: int

@rust.callback(backpressure=bounded(1024), overflow=error, shutdown=drain)
@rust(bridge.events.subscribe, panic=map_error(bridge.events.map_panic))
def subscribe(callback: Callable[[int], None]) -> Result[Subscription, CallbackError | RustPanicError]:
    return Subscription(id=0)
"#;

#[test]
fn package_rust_interop_accepts_callback_policy_contract() {
    let generated = generated_from_source(CALLBACK_SOURCE);
    let context = context_with_source(CALLBACK_SOURCE);

    apply_package_rust_interop_metadata(generated, Some(context))
        .expect("valid callback contract should pass");
}

#[test]
fn package_rust_interop_accepts_direct_callback_backpressure() {
    let source = CALLBACK_SOURCE.replace(
        "backpressure=bounded(1024), overflow=error, shutdown=drain",
        "backpressure=direct, overflow=drop_newest, shutdown=cancel",
    );
    let generated = generated_from_source(&source);
    let context = context_with_source(&source);

    apply_package_rust_interop_metadata(generated, Some(context))
        .expect("direct callback policy should pass");
}

#[test]
fn package_rust_interop_rejects_callback_policy_without_rust_target() {
    let source = CALLBACK_SOURCE.replace(
        "@rust(bridge.events.subscribe, panic=map_error(bridge.events.map_panic))\n",
        "",
    );
    let generated = generated_from_source(&source);
    let context = context_with_source(&source);

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "callback metadata without target should fail",
    );
    assert_eq!(diagnostics[0].code, "SIFR-RUST-CB-0001");
    assert!(diagnostics[0]
        .message
        .contains("must accompany a `@rust(...)` target declaration"));
}

#[test]
fn package_rust_interop_rejects_callable_parameter_without_callback_policy() {
    let source = CALLBACK_SOURCE.replace(
        "@rust.callback(backpressure=bounded(1024), overflow=error, shutdown=drain)\n",
        "",
    );
    let generated = generated_from_source(&source);
    let context = context_with_source(&source);

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "callable parameter without callback contract should fail",
    );
    assert_eq!(diagnostics[0].code, "SIFR-RUST-TYPE-0001");
    assert!(format!("{:?}", diagnostics[0].args)
        .contains("callbacks require explicit callback contract support"));
}

#[test]
fn package_rust_interop_rejects_callback_missing_backpressure() {
    let source = CALLBACK_SOURCE.replace(
        "backpressure=bounded(1024), overflow=error, shutdown=drain",
        "overflow=error, shutdown=drain",
    );
    let generated = generated_from_source(&source);
    let context = context_with_source(&source);

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "missing callback backpressure should fail",
    );
    assert_eq!(diagnostics[0].code, "SIFR-RUST-CB-0001");
    assert!(diagnostics[0]
        .message
        .contains("missing required `backpressure=` policy"));
}

#[test]
fn package_rust_interop_rejects_callback_missing_overflow() {
    let source = CALLBACK_SOURCE.replace(
        "backpressure=bounded(1024), overflow=error, shutdown=drain",
        "backpressure=bounded(8), shutdown=drain",
    );
    let generated = generated_from_source(&source);
    let context = context_with_source(&source);

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "missing callback overflow should fail",
    );
    assert_eq!(diagnostics[0].code, "SIFR-RUST-CB-0001");
    assert!(diagnostics[0]
        .message
        .contains("missing required `overflow=` policy"));
}

#[test]
fn package_rust_interop_rejects_callback_missing_shutdown() {
    let source = CALLBACK_SOURCE.replace(
        "backpressure=bounded(1024), overflow=error, shutdown=drain",
        "backpressure=bounded(8), overflow=error",
    );
    let generated = generated_from_source(&source);
    let context = context_with_source(&source);

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "missing callback shutdown should fail",
    );
    assert_eq!(diagnostics[0].code, "SIFR-RUST-CB-0001");
    assert!(diagnostics[0]
        .message
        .contains("missing required `shutdown=` policy"));
}

#[test]
fn package_rust_interop_rejects_invalid_callback_backpressure_bound() {
    let source = CALLBACK_SOURCE.replace("backpressure=bounded(1024)", "backpressure=bounded(0)");
    let generated = generated_from_source(&source);
    let context = context_with_source(&source);

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "invalid callback bound should fail",
    );
    assert_eq!(diagnostics[0].code, "SIFR-RUST-CB-0001");
    assert!(diagnostics[0].message.contains("requires a positive bound"));
}

#[test]
fn package_rust_interop_rejects_unknown_callback_overflow_policy() {
    let source = CALLBACK_SOURCE.replace("overflow=error", "overflow=block");
    let generated = generated_from_source(&source);
    let context = context_with_source(&source);

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "unknown callback overflow should fail",
    );
    assert_eq!(diagnostics[0].code, "SIFR-RUST-CB-0001");
    assert!(diagnostics[0]
        .message
        .contains("`overflow=` must be error, drop_oldest, or drop_newest"));
}

#[test]
fn package_rust_interop_rejects_unknown_callback_shutdown_policy() {
    let source = CALLBACK_SOURCE.replace("shutdown=drain", "shutdown=leak");
    let generated = generated_from_source(&source);
    let context = context_with_source(&source);

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "unknown callback shutdown should fail",
    );
    assert_eq!(diagnostics[0].code, "SIFR-RUST-CB-0001");
    assert!(diagnostics[0]
        .message
        .contains("`shutdown=` must be drain, cancel, or detach_forbidden"));
}

#[test]
fn package_rust_interop_rejects_duplicate_callback_contracts() {
    let source = CALLBACK_SOURCE.replace(
        "@rust.callback(backpressure=bounded(1024), overflow=error, shutdown=drain)",
        "@rust.callback(backpressure=bounded(1024), overflow=error, shutdown=drain)\n@rust.callback(backpressure=bounded(8), overflow=drop_oldest, shutdown=cancel)",
    );
    let generated = generated_from_source(&source);
    let context = context_with_source(&source);

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "duplicate callback contract should fail",
    );
    assert_eq!(diagnostics[0].code, "SIFR-RUST-CB-0001");
    assert!(diagnostics[0]
        .message
        .contains("only one `@rust.callback(...)` contract"));
}

fn generated_from_source(source: &str) -> GeneratedBinaryProject {
    let parsed = sifr_syntax::parse_module(source, Some("app")).expect("source should parse");
    let module = sifr_lowering::lower_module(parsed.suite())
        .map(|result| result.module)
        .expect("source should lower");
    let mut result = generate_rust_multi_with_metadata(&[("app", &module)], &StdlibCode::default());
    let main_rs = result.rust_files.remove("app").unwrap_or_default();
    GeneratedBinaryProject {
        main_rs,
        support_modules: BTreeMap::new(),
        used_stdlib_modules: result.used_stdlib_modules,
        required_features: result.required_features,
        interop: result.interop,
        cache_key_fragment: None,
        python_runtime: None,
    }
}

fn context_with_source(source: &str) -> PackageRustInteropContext {
    let mut context = package_context(TrustPolicy::default(), Vec::new());
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);
    context.module_sources.insert(
        "app".to_string(),
        RustInteropModuleSource {
            source: source.to_string(),
            display_path: "/ws/app/sifr/app.sifr".to_string(),
        },
    );
    context
}
