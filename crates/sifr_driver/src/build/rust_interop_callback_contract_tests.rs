use super::project_codegen::GeneratedBinaryProject;
use super::rust_interop::{
    PackageRustInteropContext, RustInteropModuleSource, apply_package_rust_interop_metadata,
};
use super::rust_interop_contract_tests::{
    interop_errors, package_context, set_bridge_roots, temp_package_root,
};
use sifr_codegen::{StdlibCode, generate_rust_multi_with_metadata};
use sifr_package::TrustPolicy;
use std::collections::BTreeMap;
use std::path::PathBuf;

const CALLBACK_SOURCE: &str = r#"
class CallbackError(Error):
    message: str

@rust.opaque(type=bridge.events.Subscription, send=True, sync=False, clone=none, close=close)
class Subscription:
    @rust(Self.close)
    def close(own self) -> Result[None, CallbackError | RustPanicError]: ...

@rust.callback(backpressure=bounded(1024), overflow=error, shutdown=drain)
@rust(bridge.events.subscribe, panic=map_error(bridge.events.map_panic))
def subscribe(own callback: Callable[[int], None]) -> Result[Subscription, CallbackError | RustPanicError]: ...
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
fn package_rust_interop_rejects_mutable_threadsafe_callback() {
    let source = CALLBACK_SOURCE.replace("own callback:", "own mut callback:");
    let generated = generated_from_source(&source);
    let context = context_with_source(&source);

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "retained callback cannot expose mutable callable state",
    );
    assert_eq!(diagnostics[0].code, "SIFR-RUST-CB-0001");
    assert!(diagnostics[0].message.contains("cannot be declared `mut`"));
}

#[test]
fn package_rust_interop_rejects_threadsafe_callback_without_subscription_handle() {
    let source = CALLBACK_SOURCE.replace(
        "Result[Subscription, CallbackError | RustPanicError]",
        "Result[int, CallbackError | RustPanicError]",
    );
    let generated = generated_from_source(&source);
    let context = context_with_source(&source);

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "retained callback must return a cleanup handle",
    );
    assert_eq!(diagnostics[0].code, "SIFR-RUST-CB-0001");
    assert!(diagnostics[0].message.contains("explicit cleanup handle"));
}

#[test]
fn package_rust_interop_rejects_callback_policy_without_rust_target() {
    let source = CALLBACK_SOURCE
        .replace(
            "@rust(bridge.events.subscribe, panic=map_error(bridge.events.map_panic))\n",
            "",
        )
        .replace(
            "-> Result[Subscription, CallbackError | RustPanicError]: ...",
            "-> None:\n    return None",
        );
    let generated = generated_from_source(&source);
    let context = context_with_source(&source);

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "callback metadata without target should fail",
    );
    assert_eq!(diagnostics[0].code, "SIFR-RUST-CB-0001");
    assert!(
        diagnostics[0]
            .message
            .contains("must accompany a `@rust(...)` target declaration")
    );
}

#[test]
fn package_rust_interop_accepts_callable_parameter_as_call_scoped() {
    let source = CALLBACK_SOURCE.replace(
        "@rust.callback(backpressure=bounded(1024), overflow=error, shutdown=drain)\n",
        "",
    );
    let generated = generated_from_source(&source);
    let context = context_with_source(&source);

    apply_package_rust_interop_metadata(generated, Some(context))
        .expect("plain Callable parameters should use the call-scoped bridge contract");
}

#[test]
fn package_rust_interop_rejects_call_scoped_callback_across_async_boundary() {
    let source = CALLBACK_SOURCE
        .replace(
            "@rust.callback(backpressure=bounded(1024), overflow=error, shutdown=drain)\n",
            "",
        )
        .replace("def subscribe", "async def subscribe");
    let generated = generated_from_source(&source);
    let context = context_with_source(&source);

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "call-scoped callback across async boundary should fail",
    );
    assert_eq!(diagnostics[0].code, "SIFR-RUST-CB-0001");
    assert!(
        diagnostics[0]
            .message
            .contains("cannot cross an async boundary")
    );
}

#[test]
fn package_rust_interop_rejects_call_scoped_callback_with_async_policy() {
    let source = CALLBACK_SOURCE
        .replace(
            "@rust.callback(backpressure=bounded(1024), overflow=error, shutdown=drain)\n",
            "@rust.async(thread_affinity=tokio_current_thread)\n",
        )
        .replace("def subscribe", "async def subscribe");
    let generated = generated_from_source(&source);
    let context = context_with_source(&source);

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "explicit async callback policy should fail",
    );
    assert_eq!(diagnostics[0].code, "SIFR-RUST-CB-0001");
    assert!(
        diagnostics[0]
            .message
            .contains("cannot cross an async boundary")
    );
}

#[test]
fn package_rust_interop_rejects_call_scoped_callback_without_panic_boundary() {
    let source = CALLBACK_SOURCE
        .replace(
            "@rust.callback(backpressure=bounded(1024), overflow=error, shutdown=drain)\n",
            "",
        )
        .replace(
            "@rust(bridge.events.subscribe, panic=map_error(bridge.events.map_panic))",
            "@rust(bridge.events.subscribe, panic=trusted_no_panic)",
        )
        .replace(
            "-> Result[Subscription, CallbackError | RustPanicError]",
            "-> None",
        );
    let generated = generated_from_source(&source);
    let context = context_with_source(&source);

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "callback without recoverable panic boundary should fail",
    );
    assert_eq!(diagnostics[0].code, "SIFR-RUST-CB-0001");
    assert!(
        diagnostics[0]
            .message
            .contains("recoverable outer panic boundary")
    );
}

#[test]
fn package_rust_interop_rejects_call_scoped_callback_with_abort_policy() {
    let source = CALLBACK_SOURCE
        .replace(
            "@rust.callback(backpressure=bounded(1024), overflow=error, shutdown=drain)\n",
            "",
        )
        .replace("panic=map_error(bridge.events.map_panic)", "panic=abort");
    let generated = generated_from_source(&source);
    let context = context_with_source(&source);

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "abort policy cannot contain callback panics",
    );
    assert_eq!(diagnostics[0].code, "SIFR-RUST-CB-0001");
    assert!(diagnostics[0].message.contains("`panic=abort`"));
}

#[test]
fn package_rust_interop_rejects_threadsafe_callback_with_abort_policy() {
    let source = CALLBACK_SOURCE.replace("panic=map_error(bridge.events.map_panic)", "panic=abort");
    let generated = generated_from_source(&source);
    let context = context_with_source(&source);

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "retained callback panics require unwind",
    );
    assert_eq!(diagnostics[0].code, "SIFR-RUST-CB-0001");
    assert!(diagnostics[0].message.contains("`panic=abort`"));
}

#[test]
fn package_rust_interop_rejects_threadsafe_abort_without_signature_contract() {
    let source = CALLBACK_SOURCE.replace("panic=map_error(bridge.events.map_panic)", "panic=abort");
    let mut generated = generated_from_source(&source);
    generated.interop.rust.bridge_contracts.signatures.clear();
    let context = context_with_source(&source);

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "retained callback abort policy is independent of signature lookup",
    );
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "SIFR-RUST-CB-0001" && diagnostic.message.contains("`panic=abort`")
    }));
}

#[test]
fn package_rust_interop_rejects_abort_policy_after_sibling_decorator() {
    let source = CALLBACK_SOURCE
        .replace(
            "@rust.callback(backpressure=bounded(1024), overflow=error, shutdown=drain)\n",
            "",
        )
        .replace(
            "@rust(bridge.events.subscribe, panic=map_error(bridge.events.map_panic))",
            "@rust.view(owner=callback, lifetime=owner, mutability=immutable, send=False, sync=False)\n@rust(bridge.events.subscribe, panic=abort)",
        );
    let generated = generated_from_source(&source);
    let context = context_with_source(&source);

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "abort policy must aggregate across sibling decorators",
    );
    assert_eq!(diagnostics[0].code, "SIFR-RUST-CB-0001");
    assert!(diagnostics[0].message.contains("`panic=abort`"));
}

#[test]
fn package_rust_interop_rejects_call_scoped_callback_in_abort_profile() {
    let source = CALLBACK_SOURCE.replace(
        "@rust.callback(backpressure=bounded(1024), overflow=error, shutdown=drain)\n",
        "",
    );
    let generated = generated_from_source(&source);
    let mut context = context_with_source(&source);
    let package_root = temp_package_root("rust_callback_abort_profile");
    std::fs::create_dir_all(&package_root).expect("abort-profile package root");
    std::fs::write(
        package_root.join("Cargo.toml"),
        "[package]\nname = \"callback-abort-profile\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[profile.release]\npanic = \"abort\"\n",
    )
    .expect("abort-profile Cargo.toml");
    context
        .graph
        .packages
        .get_mut(&context.package_id)
        .expect("package metadata")
        .package_root = package_root.clone();

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "abort Cargo profile cannot contain callback panic",
    );
    assert_eq!(diagnostics[0].code, "SIFR-RUST-CB-0001");
    assert!(diagnostics[0].message.contains("abort profiles"));
    let _ = std::fs::remove_dir_all(package_root);
}

#[test]
fn package_rust_interop_rejects_threadsafe_callback_in_abort_profile() {
    let generated = generated_from_source(CALLBACK_SOURCE);
    let mut context = context_with_source(CALLBACK_SOURCE);
    let package_root = temp_package_root("rust_threadsafe_callback_abort_profile");
    std::fs::create_dir_all(&package_root).expect("abort-profile package root");
    std::fs::write(
        package_root.join("Cargo.toml"),
        "[package]\nname = \"threadsafe-callback-abort-profile\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[profile.release]\npanic = \"abort\"\n",
    )
    .expect("abort-profile Cargo.toml");
    context
        .graph
        .packages
        .get_mut(&context.package_id)
        .expect("package metadata")
        .package_root = package_root.clone();

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "abort Cargo profile cannot contain retained callback panic",
    );
    assert_eq!(diagnostics[0].code, "SIFR-RUST-CB-0001");
    assert!(diagnostics[0].message.contains("abort profiles"));
    let _ = std::fs::remove_dir_all(package_root);
}

#[test]
fn package_rust_interop_rejects_mutable_call_scoped_callback_parameter() {
    let source = CALLBACK_SOURCE
        .replace(
            "@rust.callback(backpressure=bounded(1024), overflow=error, shutdown=drain)\n",
            "",
        )
        .replace(
            "def subscribe(own callback:",
            "def subscribe(own mut callback:",
        );
    let generated = generated_from_source(&source);
    let context = context_with_source(&source);

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "mutable callback parameter should fail",
    );
    assert_eq!(diagnostics[0].code, "SIFR-RUST-CB-0001");
    assert!(
        diagnostics[0]
            .message
            .contains("cannot be declared `mut`; remove `mut`")
    );
}

#[test]
fn package_rust_interop_rejects_callback_missing_backpressure() {
    let source = CALLBACK_SOURCE.replace(
        "backpressure=bounded(1024), overflow=error, shutdown=drain",
        "overflow=error, shutdown=drain",
    );
    let (code, message) = lowering_callback_error(&source);
    assert_eq!(code, "SIFR-RUST-CB-0001");
    assert!(message.contains("missing required `backpressure=` policy"));
}

#[test]
fn package_rust_interop_rejects_callback_missing_overflow() {
    let source = CALLBACK_SOURCE.replace(
        "backpressure=bounded(1024), overflow=error, shutdown=drain",
        "backpressure=bounded(8), shutdown=drain",
    );
    let (code, message) = lowering_callback_error(&source);
    assert_eq!(code, "SIFR-RUST-CB-0001");
    assert!(message.contains("missing required `overflow=` policy"));
}

#[test]
fn package_rust_interop_rejects_callback_missing_shutdown() {
    let source = CALLBACK_SOURCE.replace(
        "backpressure=bounded(1024), overflow=error, shutdown=drain",
        "backpressure=bounded(8), overflow=error",
    );
    let (code, message) = lowering_callback_error(&source);
    assert_eq!(code, "SIFR-RUST-CB-0001");
    assert!(message.contains("missing required `shutdown=` policy"));
}

#[test]
fn package_rust_interop_rejects_invalid_callback_backpressure_bound() {
    let source = CALLBACK_SOURCE.replace("backpressure=bounded(1024)", "backpressure=bounded(0)");
    let (code, message) = lowering_callback_error(&source);
    assert_eq!(code, "SIFR-RUST-CB-0001");
    assert!(message.contains("requires a positive bound"));
}

#[test]
fn package_rust_interop_rejects_unknown_callback_overflow_policy() {
    let source = CALLBACK_SOURCE.replace("overflow=error", "overflow=block");
    let (code, message) = lowering_callback_error(&source);
    assert_eq!(code, "SIFR-RUST-CB-0001");
    assert!(message.contains("`overflow=` must be error, drop_oldest, or drop_newest"));
}

#[test]
fn package_rust_interop_rejects_unknown_callback_shutdown_policy() {
    let source = CALLBACK_SOURCE.replace("shutdown=drain", "shutdown=leak");
    let (code, message) = lowering_callback_error(&source);
    assert_eq!(code, "SIFR-RUST-CB-0001");
    assert!(message.contains("`shutdown=` must be drain, cancel, or detach_forbidden"));
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
    assert!(
        diagnostics[0]
            .message
            .contains("only one `@rust.callback(...)` contract")
    );
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
        bridge_modules: Default::default(),
        python_runtime: None,
    }
}

fn lowering_callback_error(source: &str) -> (String, String) {
    let parsed = sifr_syntax::parse_module(source, Some("app")).expect("source should parse");
    let errors = match sifr_lowering::lower_module(parsed.suite()) {
        Ok(_) => panic!("invalid callback policy should fail lowering"),
        Err(errors) => errors,
    };
    let callback_errors = errors
        .into_iter()
        .filter(|error| {
            error
                .code
                .is_some_and(|code| code.code() == "SIFR-RUST-CB-0001")
        })
        .collect::<Vec<_>>();
    assert_eq!(callback_errors.len(), 1, "{callback_errors:?}");
    (
        callback_errors[0]
            .code
            .map_or_else(String::new, |code| code.code().to_string()),
        callback_errors[0].message.clone(),
    )
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
