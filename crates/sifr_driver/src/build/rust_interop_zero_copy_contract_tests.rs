use super::project_codegen::GeneratedBinaryProject;
use super::rust_interop::{
    apply_package_rust_interop_metadata, PackageRustInteropContext, RustInteropModuleSource,
};
use super::rust_interop_contract_tests::{
    interop_errors, package_context, param_contract, result_contract, set_bridge_roots,
    signature_contract,
};
use sifr_codegen::{
    generate_rust_multi_with_metadata, RustBridgeParamConvention, RustBridgeSignatureContract,
    RustBridgeTypeContract, RustBridgeTypeKind, StdlibCode,
};
use sifr_package::TrustPolicy;
use std::collections::BTreeMap;
use std::path::PathBuf;

const VALID_ZERO_COPY_SOURCE: &str = r#"
class RustError(Error):
    message: str

class BytesView:
    ptr: int

@rust.zero_copy(owner=input, view=bridge.bytes.BytesView)
@rust.view(owner=input, lifetime=owner, mutability=immutable, send=False, sync=False)
@rust(bridge.bytes.view, panic=map_error(bridge.bytes.map_panic))
def hash(input: bytes) -> Result[BytesView, RustError]:
    return BytesView(ptr=0)
"#;

#[test]
fn package_rust_interop_zero_copy_accepts_borrowed_bytes_view_contract() {
    let mut generated = generated_from_source(VALID_ZERO_COPY_SOURCE);
    let mut context = context_with_source(VALID_ZERO_COPY_SOURCE);
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    generated = apply_package_rust_interop_metadata(generated, Some(context))
        .expect("valid zero-copy view contract should pass");

    let view_probe = generated
        .interop
        .rust
        .probe_plan
        .probes
        .iter()
        .find(|probe| probe.kind == sifr_codegen::RustBridgeProbeKind::View)
        .expect("view probe");
    assert!(!view_probe.requires_send);
    assert!(!view_probe.requires_sync);
}

#[test]
fn package_rust_interop_view_send_sync_metadata_reaches_probe_plan() {
    let source = VALID_ZERO_COPY_SOURCE.replace("send=False, sync=False", "send=True, sync=True");
    let mut generated = generated_from_source(&source);
    let mut context = context_with_source(&source);
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    generated = apply_package_rust_interop_metadata(generated, Some(context))
        .expect("valid send/sync view contract should pass");

    let view_probe = generated
        .interop
        .rust
        .probe_plan
        .probes
        .iter()
        .find(|probe| probe.kind == sifr_codegen::RustBridgeProbeKind::View)
        .expect("view probe");
    assert!(view_probe.requires_send);
    assert!(view_probe.requires_sync);
}

#[test]
fn package_rust_interop_accepts_async_static_lifetime_view() {
    let source = VALID_ZERO_COPY_SOURCE
        .replace("lifetime=owner", "lifetime=static")
        .replace("def hash", "async def hash")
        .replace(
            "return BytesView(ptr=0)",
            "await task.sleep(0.0)\n    return BytesView(ptr=0)",
        );
    let mut generated = generated_from_source(&source);
    let mut context = context_with_source(&source);
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    generated = apply_package_rust_interop_metadata(generated, Some(context))
        .expect("async static-lifetime view contract should pass");

    assert!(generated
        .interop
        .rust
        .probe_plan
        .probes
        .iter()
        .any(|probe| probe.kind == sifr_codegen::RustBridgeProbeKind::View));
}

#[test]
fn package_rust_interop_zero_copy_requires_view_contract() {
    let source = VALID_ZERO_COPY_SOURCE.replace(
        "@rust.view(owner=input, lifetime=owner, mutability=immutable, send=False, sync=False)\n",
        "",
    );
    let generated = generated_from_source(&source);
    let mut context = context_with_source(&source);
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    let diagnostics = interop_errors(generated, Some(context), "missing view must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ZC-0001");
    assert!(diagnostics[0].message.contains("requires a paired"));
}

#[test]
fn package_rust_interop_rejects_call_lifetime_returned_view() {
    let source = VALID_ZERO_COPY_SOURCE.replace("lifetime=owner", "lifetime=call");
    let generated = generated_from_source(&source);
    let mut context = context_with_source(&source);
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    let diagnostics = interop_errors(generated, Some(context), "call lifetime must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ZC-0001");
    assert!(diagnostics[0].message.contains("lifetime=call"));
}

#[test]
fn package_rust_interop_rejects_zero_copy_and_view_owner_mismatch() {
    let source = VALID_ZERO_COPY_SOURCE.replace(
        "@rust.zero_copy(owner=input, view=bridge.bytes.BytesView)",
        "@rust.zero_copy(owner=other, view=bridge.bytes.BytesView)",
    );
    let generated = generated_from_source(&source);
    let mut context = context_with_source(&source);
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    let diagnostics = interop_errors(generated, Some(context), "owner mismatch must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ZC-0001");
    assert!(diagnostics[0].message.contains("same owner"));
}

#[test]
fn package_rust_interop_rejects_unknown_view_owner() {
    let source = VALID_ZERO_COPY_SOURCE
        .replace(
            "@rust.zero_copy(owner=input, view=bridge.bytes.BytesView)",
            "@rust.zero_copy(owner=missing, view=bridge.bytes.BytesView)",
        )
        .replace(
            "@rust.view(owner=input, lifetime=owner, mutability=immutable, send=False, sync=False)",
            "@rust.view(owner=missing, lifetime=owner, mutability=immutable, send=False, sync=False)",
        );
    let generated = generated_from_source(&source);
    let mut context = context_with_source(&source);
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    let diagnostics = interop_errors(generated, Some(context), "unknown owner must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ZC-0001");
    assert!(diagnostics[0].message.contains("owner must name"));
}

#[test]
fn package_rust_interop_rejects_legacy_mutable_bool_key() {
    let source = VALID_ZERO_COPY_SOURCE.replace("mutability=immutable", "mutable=False");
    let generated = generated_from_source(&source);
    let mut context = context_with_source(&source);
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    let diagnostics = interop_errors(generated, Some(context), "legacy mutable key must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ZC-0001");
    assert!(diagnostics[0].message.contains("unsupported `@rust.view"));
    assert_eq!(diagnostics.len(), 1);
}

#[test]
fn package_rust_interop_rejects_mutable_view_from_shared_borrow_owner() {
    let source = VALID_ZERO_COPY_SOURCE.replace("mutability=immutable", "mutability=mutable");
    let generated = generated_from_source(&source);
    let mut context = context_with_source(&source);
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    let diagnostics = interop_errors(generated, Some(context), "shared owner must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ZC-0001");
    assert!(diagnostics[0].message.contains("exclusive owner"));
}

#[test]
fn package_rust_interop_rejects_zero_copy_copy_fallback() {
    let source = VALID_ZERO_COPY_SOURCE.replace(
        "@rust.zero_copy(owner=input, view=bridge.bytes.BytesView)",
        "@rust.zero_copy(owner=input, view=bridge.bytes.BytesView, copy_fallback=True)",
    );
    let generated = generated_from_source(&source);
    let mut context = context_with_source(&source);
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    let diagnostics = interop_errors(generated, Some(context), "copy fallback must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ZC-0001");
    assert!(diagnostics[0]
        .message
        .contains("unsupported `@rust.zero_copy"));
}

#[test]
fn package_rust_interop_rejects_async_owner_lifetime_view() {
    let source = VALID_ZERO_COPY_SOURCE
        .replace("def hash", "async def hash")
        .replace(
            "return BytesView(ptr=0)",
            "await task.sleep(0.0)\n    return BytesView(ptr=0)",
        );
    let generated = generated_from_source(&source);
    let mut context = context_with_source(&source);
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    let diagnostics = interop_errors(generated, Some(context), "async owner view must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ZC-0001");
    assert!(diagnostics[0].message.contains("async Rust interop views"));
}

fn generated_from_source(source: &str) -> GeneratedBinaryProject {
    let parsed = sifr_syntax::parse_module(source, Some("app")).expect("source should parse");
    let module = sifr_lowering::lower_module(parsed.suite())
        .map(|result| result.module)
        .expect("source should lower");
    let mut result = generate_rust_multi_with_metadata(&[("app", &module)], &StdlibCode::default());
    result.interop.rust.bridge_contracts.signatures = vec![hash_signature_contract()];
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
    context.module_sources.insert(
        "app".to_string(),
        RustInteropModuleSource {
            source: source.to_string(),
            display_path: "/ws/app/sifr/app.sifr".to_string(),
        },
    );
    context
}

fn hash_signature_contract() -> RustBridgeSignatureContract {
    signature_contract(
        vec![param_contract(
            "input",
            RustBridgeParamConvention::Borrow,
            bytes_contract(),
        )],
        result_contract(view_type_contract(), error_type_contract()),
    )
}

fn bytes_contract() -> RustBridgeTypeContract {
    RustBridgeTypeContract {
        sifr_type: "bytes".to_string(),
        rust_borrowed_type: Some("&[u8]".to_string()),
        rust_owned_type: Some("Vec<u8>".to_string()),
        rust_return_type: Some("Vec<u8>".to_string()),
        kind: RustBridgeTypeKind::Bytes,
        unsupported_reason: None,
    }
}

fn view_type_contract() -> RustBridgeTypeContract {
    RustBridgeTypeContract {
        sifr_type: "BytesView".to_string(),
        rust_borrowed_type: Some("crate::__sifr_bridge::BytesViewBridge".to_string()),
        rust_owned_type: Some("crate::__sifr_bridge::BytesViewBridge".to_string()),
        rust_return_type: Some("crate::__sifr_bridge::BytesViewBridge".to_string()),
        kind: RustBridgeTypeKind::GeneratedRecord,
        unsupported_reason: None,
    }
}

fn error_type_contract() -> RustBridgeTypeContract {
    RustBridgeTypeContract {
        sifr_type: "RustError".to_string(),
        rust_borrowed_type: Some("crate::__sifr_bridge::RustErrorBridge".to_string()),
        rust_owned_type: Some("crate::__sifr_bridge::RustErrorBridge".to_string()),
        rust_return_type: Some("crate::__sifr_bridge::RustErrorBridge".to_string()),
        kind: RustBridgeTypeKind::GeneratedError,
        unsupported_reason: None,
    }
}
