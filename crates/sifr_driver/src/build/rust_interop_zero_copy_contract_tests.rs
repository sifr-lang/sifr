use super::project_codegen::GeneratedBinaryProject;
use super::rust_interop::{
    PackageRustInteropContext, RustInteropModuleSource, apply_package_rust_interop_metadata,
};
use super::rust_interop_contract_tests::{
    interop_errors, package_context, param_contract, result_contract, set_bridge_roots,
    signature_contract,
};
use sifr_codegen::{
    RustBridgeParamConvention, RustBridgeSignatureContract, RustBridgeTypeContract,
    RustBridgeTypeKind, StdlibCode, generate_rust_multi_with_metadata,
};
use sifr_package::TrustPolicy;
use std::collections::BTreeMap;
use std::path::PathBuf;

const ZERO_COPY_BYTES_FIXTURE: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/zero_copy_bytes/positive/borrowed_bytes_view.sifr"
);
const ZERO_COPY_BYTES_NEGATIVE_FIXTURE: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/zero_copy_bytes/negative/copy_fallback_rejected.sifr"
);
const ZERO_COPY_VIEW_MATRIX_FIXTURE: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/zero_copy_view_matrix/positive/owner_lifetime_views.sifr"
);
const ZERO_COPY_VIEW_MATRIX_NEGATIVE_FIXTURE: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/zero_copy_view_matrix/negative/mutable_alias_rejected.sifr"
);
const VALID_ZERO_COPY_SOURCE: &str = r#"
class RustError(Error):
    message: str

@rust.opaque(type=bridge.bytes.BytesView, send=False, sync=False, clone=none, close=drop)
class BytesView:
    ptr: int

@rust.zero_copy(owner=input, view=bridge.bytes.BytesView)
@rust.view(owner=input, lifetime=owner, mutability=immutable, send=False, sync=False)
@rust(bridge.bytes.view, panic=map_error(bridge.bytes.map_panic))
def hash(input: bytes) -> Result[BytesView, RustError | RustPanicError]: ...
"#;

#[test]
fn package_rust_interop_zero_copy_accepts_borrowed_bytes_view_contract() {
    let mut generated = generated_from_fixture_source(ZERO_COPY_BYTES_FIXTURE);
    let mut context = context_with_source(ZERO_COPY_BYTES_FIXTURE);
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
    assert!(view_probe.requires_send);
    assert!(view_probe.requires_sync);
    let zero_copy_probe = generated
        .interop
        .rust
        .probe_plan
        .probes
        .iter()
        .find(|probe| probe.kind == sifr_codegen::RustBridgeProbeKind::ZeroCopy)
        .expect("zero-copy probe");
    assert!(zero_copy_probe.requires_send);
    assert!(zero_copy_probe.requires_sync);
}

#[test]
fn package_rust_interop_view_send_sync_metadata_reaches_probe_plan() {
    let mut generated = generated_from_fixture_source(ZERO_COPY_VIEW_MATRIX_FIXTURE);
    let mut context = context_with_source(ZERO_COPY_VIEW_MATRIX_FIXTURE);
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
    let zero_copy_probe = generated
        .interop
        .rust
        .probe_plan
        .probes
        .iter()
        .find(|probe| probe.kind == sifr_codegen::RustBridgeProbeKind::ZeroCopy)
        .expect("zero-copy probe");
    assert!(zero_copy_probe.requires_send);
    assert!(zero_copy_probe.requires_sync);
}

#[test]
fn package_rust_interop_accepts_async_static_lifetime_view() {
    let source = VALID_ZERO_COPY_SOURCE
        .replace("lifetime=owner", "lifetime=static")
        .replace(", panic=map_error(bridge.bytes.map_panic)", "")
        .replace("def hash", "async def hash");
    let mut generated = generated_from_source(&source);
    let mut context = context_with_source(&source);
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    generated = apply_package_rust_interop_metadata(generated, Some(context))
        .expect("async static-lifetime view contract should pass");

    assert!(
        generated
            .interop
            .rust
            .probe_plan
            .probes
            .iter()
            .any(|probe| probe.kind == sifr_codegen::RustBridgeProbeKind::View)
    );
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
fn package_rust_interop_rejects_view_type_return_mismatch() {
    let source = VALID_ZERO_COPY_SOURCE
        .replace("view=bridge.bytes.BytesView", "view=bridge.bytes.OtherView");
    let generated = generated_from_source(&source);
    let mut context = context_with_source(&source);
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    let diagnostics = interop_errors(generated, Some(context), "view return mismatch must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ZC-0001");
    assert!(diagnostics[0].message.contains("function return value"));
}

#[test]
fn package_rust_interop_rejects_view_type_prefix_alias() {
    let source =
        VALID_ZERO_COPY_SOURCE.replace("view=bridge.bytes.BytesView", "view=bridge.bytes.Bytes");
    let generated = generated_from_source(&source);
    let mut context = context_with_source(&source);
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    let diagnostics = interop_errors(generated, Some(context), "view prefix alias must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ZC-0001");
    assert!(diagnostics[0].message.contains("function return value"));
}

#[test]
fn package_rust_interop_rejects_view_nested_in_list_return() {
    let mut generated = generated_from_source(VALID_ZERO_COPY_SOURCE);
    generated.interop.rust.bridge_contracts.signatures[0].return_type =
        result_contract(list_view_type_contract(), error_type_contract());
    let mut context = context_with_source(VALID_ZERO_COPY_SOURCE);
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    let diagnostics = interop_errors(generated, Some(context), "nested view return must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ZC-0001");
    assert!(diagnostics[0].message.contains("function return value"));
}

#[test]
fn package_rust_interop_preserves_generated_record_view_contract() {
    let source = generated_record_view_source();
    let mut generated = generated_from_source(&source);
    let generated_record_path = generated_record_view_path(&generated);
    generated.interop.rust.bridge_contracts.signatures[0].return_type = result_contract(
        generated_record_view_type_contract(&generated_record_path),
        error_type_contract(),
    );
    let mut context = context_with_source(&source);
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    apply_package_rust_interop_metadata(generated, Some(context))
        .expect("contract-only generated record view should remain supported");
}

#[test]
fn package_rust_interop_preserves_unsupported_return_diagnostic() {
    let mut generated = generated_from_source(VALID_ZERO_COPY_SOURCE);
    let return_type = &mut generated.interop.rust.bridge_contracts.signatures[0].return_type;
    return_type.rust_return_type = None;
    return_type.unsupported_reason = Some("error type is not bridge-compatible".to_string());
    let mut context = context_with_source(VALID_ZERO_COPY_SOURCE);
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    let diagnostics = interop_errors(generated, Some(context), "unsupported return must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-TYPE-0001");
    assert!(
        diagnostics[0]
            .message
            .contains("unsupported Rust bridge type")
    );
    assert!(
        !diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "SIFR-RUST-ZC-0001")
    );
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
    assert!(
        diagnostics[0]
            .message
            .contains("unsupported `@rust.view(...)` key `mutable`")
    );
    assert_eq!(diagnostics.len(), 1);
}

#[test]
fn package_rust_interop_rejects_mutable_view_from_shared_borrow_owner() {
    let generated = generated_from_fixture_source(ZERO_COPY_VIEW_MATRIX_NEGATIVE_FIXTURE);
    let mut context = context_with_source(ZERO_COPY_VIEW_MATRIX_NEGATIVE_FIXTURE);
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    let diagnostics = interop_errors(generated, Some(context), "shared owner must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ZC-0001");
    assert!(diagnostics[0].message.contains("exclusive owner"));
}

#[test]
fn package_rust_interop_rejects_zero_copy_copy_fallback() {
    let generated = generated_from_fixture_source(ZERO_COPY_BYTES_NEGATIVE_FIXTURE);
    let mut context = context_with_source(ZERO_COPY_BYTES_NEGATIVE_FIXTURE);
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    let diagnostics = interop_errors(generated, Some(context), "copy fallback must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ZC-0001");
    assert!(
        diagnostics[0]
            .message
            .contains("unsupported `@rust.zero_copy(...)` key `copy_fallback`")
    );
}

#[test]
fn package_rust_interop_rejects_async_owner_lifetime_view() {
    let source = VALID_ZERO_COPY_SOURCE.replace("def hash", "async def hash");
    let generated = generated_from_source(&source);
    let mut context = context_with_source(&source);
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    let diagnostics = interop_errors(generated, Some(context), "async owner view must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ZC-0001");
    assert!(diagnostics[0].message.contains("async Rust interop views"));
}

fn generated_from_source(source: &str) -> GeneratedBinaryProject {
    let mut generated = generated_from_fixture_source(source);
    generated.interop.rust.bridge_contracts.signatures = vec![hash_signature_contract()];
    generated
}

fn generated_from_fixture_source(source: &str) -> GeneratedBinaryProject {
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

fn generated_record_view_source() -> String {
    VALID_ZERO_COPY_SOURCE.replace(
        "@rust.opaque(type=bridge.bytes.BytesView, send=False, sync=False, clone=none, close=drop)\n",
        "",
    )
}

fn generated_record_view_path(generated: &GeneratedBinaryProject) -> String {
    generated
        .interop
        .rust
        .bridge_contracts
        .generated_types
        .iter()
        .find(|generated_type| {
            generated_type.kind == sifr_codegen::RustGeneratedBridgeTypeKind::Record
        })
        .map(|generated_type| generated_type.rust_type_path.clone())
        .expect("generated record source should produce a record bridge")
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
    let mut signature = signature_contract(
        vec![param_contract(
            "input",
            RustBridgeParamConvention::Borrow,
            bytes_contract(),
        )],
        result_contract(view_type_contract(), error_type_contract()),
    );
    signature.panic_error = sifr_codegen::RustBridgePanicErrorContract::OrdinaryAndWrapper;
    signature
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
    let handle = sifr_codegen::rust_opaque_handle_type("bridge.bytes.BytesView");
    RustBridgeTypeContract {
        sifr_type: "BytesView".to_string(),
        rust_borrowed_type: Some(format!("&{handle}")),
        rust_owned_type: Some(handle.clone()),
        rust_return_type: Some(handle),
        kind: RustBridgeTypeKind::OpaqueHandle,
        unsupported_reason: None,
    }
}

fn list_view_type_contract() -> RustBridgeTypeContract {
    let handle = sifr_codegen::rust_opaque_handle_type("bridge.bytes.BytesView");
    RustBridgeTypeContract {
        sifr_type: "list[BytesView]".to_string(),
        rust_borrowed_type: Some(format!("&[{handle}]")),
        rust_owned_type: Some(format!("Vec<{handle}>")),
        rust_return_type: Some(format!("Vec<{handle}>")),
        kind: RustBridgeTypeKind::List,
        unsupported_reason: None,
    }
}

fn generated_record_view_type_contract(rust_type_path: &str) -> RustBridgeTypeContract {
    RustBridgeTypeContract {
        sifr_type: "BytesView".to_string(),
        rust_borrowed_type: Some(rust_type_path.to_string()),
        rust_owned_type: Some(rust_type_path.to_string()),
        rust_return_type: Some(rust_type_path.to_string()),
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
