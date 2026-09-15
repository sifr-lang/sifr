use super::project_codegen::GeneratedBinaryProject;
use super::rust_interop::{
    PackageRustInteropContext, RustInteropModuleSource, apply_package_rust_interop_metadata,
};
use super::rust_interop_contract_tests::{
    backend_with_manifest, base_project_with_contracts, interop_errors, none_contract,
    package_context, param_contract, result_contract, signature_contract, temp_package_root,
    trusted_no_panic_context, trusted_no_panic_declaration_entry,
};
use sifr_codegen::{
    RustBridgeParamConvention, RustBridgeSignatureContract, RustBridgeTypeContract,
    RustBridgeTypeKind, RustInteropOwner, StdlibCode, generate_rust_multi_with_metadata,
};
use sifr_package::TrustPolicy;
use std::collections::BTreeMap;
use std::path::PathBuf;

const ARROW_SOURCE: &str = r#"
class ArrowError(Error):
    message: str

class ArrowRecordBatch:
    ptr: int

@rust.zero_copy(owner=columns, view=sifr_arrow_bridge.record_batch.RecordBatchView)
@rust.view(owner=columns, lifetime=owner, mutability=immutable, send=True, sync=True, data=arrow_record_batch, schema=sifr_arrow_bridge.schema.RecordBatch, ownership=borrowed)
@rust(sifr_arrow_bridge.record_batch_from_columns, panic=map_error(sifr_arrow_bridge.panic.map))
def record_batch(columns: bytes) -> Result[ArrowRecordBatch, ArrowError | RustPanicError]:
    return ArrowRecordBatch(ptr=0)
"#;

const TENSOR_SOURCE: &str = r#"
class TensorError(Error):
    message: str

class TensorView:
    ptr: int

@rust.zero_copy(owner=input, view=sifr_tensor_bridge.tensor.TensorView)
@rust.view(owner=input, lifetime=owner, mutability=immutable, send=True, sync=True, data=tensor, dtype=f32, rank=2, shape=[2, 3], layout=strided, strides=[3, 1], device=cpu, ownership=borrowed)
@rust(sifr_tensor_bridge.tensor_from_bytes, panic=map_error(sifr_tensor_bridge.panic.map))
def tensor(input: bytes) -> Result[TensorView, TensorError | RustPanicError]:
    return TensorView(ptr=0)
"#;

const DLPACK_SOURCE: &str = r#"
class TensorError(Error):
    message: str

class TensorView:
    ptr: int

@rust.zero_copy(owner=input, view=sifr_tensor_bridge.dlpack.DlpackView)
@rust.view(owner=input, lifetime=owner, mutability=immutable, send=True, sync=True, data=dlpack, dtype=f32, rank=2, shape=[2, 3], layout=strided, strides=[3, 1], device=cpu, ownership=transfer, protocol=sifr_tensor_bridge.dlpack.Capsule)
@rust(sifr_tensor_bridge.dlpack_from_bytes, panic=map_error(sifr_tensor_bridge.panic.map))
def dlpack(input: bytes) -> Result[TensorView, TensorError | RustPanicError]:
    return TensorView(ptr=0)
"#;

#[test]
fn package_rust_interop_probe_keeps_same_basename_bridge_types_distinct() {
    let backend_root = temp_package_root("rust_interop_distinct_canonical_bridges");
    std::fs::create_dir_all(backend_root.join("src")).expect("create backend src");
    std::fs::write(
        backend_root.join("Cargo.toml"),
        "[package]\nname = \"native\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("write backend cargo toml");
    std::fs::write(
        backend_root.join("src/lib.rs"),
        "pub fn hash<A, B>(_csv: A, _config: B) {}\n",
    )
    .expect("write backend lib");

    let generated = base_project_with_contracts(
        vec![trusted_no_panic_declaration_entry(
            "native.hash",
            sifr_ir::RustInteropDecoratorKind::Function,
        )],
        vec![signature_contract(
            vec![
                param_contract(
                    "csv_error",
                    RustBridgeParamConvention::Borrow,
                    generated_error_contract(
                        "sifr.csv.Error",
                        "crate::__sifr_bridge::sifr_csv::ErrorBridge",
                    ),
                ),
                param_contract(
                    "config_error",
                    RustBridgeParamConvention::Borrow,
                    generated_error_contract(
                        "sifr.configparser.Error",
                        "crate::__sifr_bridge::sifr_configparser::ErrorBridge",
                    ),
                ),
            ],
            none_contract(),
        )],
    );
    let context = trusted_no_panic_context(vec![backend_with_manifest(
        "native",
        backend_root.join("Cargo.toml"),
    )]);

    apply_package_rust_interop_metadata(generated, Some(context))
        .expect("canonical bridge paths should produce distinct compiled probe stubs");
}

#[test]
fn package_rust_interop_accepts_arrow_record_batch_metadata_contract() {
    let generated = generated_from_source(ARROW_SOURCE, record_batch_signature_contract());
    let context = context_with_source(ARROW_SOURCE, "sifr_arrow_bridge");

    apply_package_rust_interop_metadata(generated, Some(context))
        .expect("arrow record batch metadata contract should pass");
}

#[test]
fn package_rust_interop_accepts_arrow_array_metadata_contract() {
    let source = ARROW_SOURCE.replace("data=arrow_record_batch", "data=arrow_array");
    let generated = generated_from_source(&source, record_batch_signature_contract());
    let context = context_with_source(&source, "sifr_arrow_bridge");

    apply_package_rust_interop_metadata(generated, Some(context))
        .expect("arrow array metadata contract should pass");
}

#[test]
fn package_rust_interop_accepts_dataframe_metadata_contract() {
    let source = ARROW_SOURCE.replace("data=arrow_record_batch", "data=dataframe");
    let generated = generated_from_source(&source, record_batch_signature_contract());
    let context = context_with_source(&source, "sifr_arrow_bridge");

    apply_package_rust_interop_metadata(generated, Some(context))
        .expect("dataframe metadata contract should pass");
}

#[test]
fn package_rust_interop_accepts_tensor_metadata_contract() {
    let generated = generated_from_source(TENSOR_SOURCE, tensor_signature_contract("tensor"));
    let context = context_with_source(TENSOR_SOURCE, "sifr_tensor_bridge");

    apply_package_rust_interop_metadata(generated, Some(context))
        .expect("tensor metadata contract should pass");
}

#[test]
fn package_rust_interop_accepts_explicit_dlpack_transfer_contract() {
    let generated = generated_from_source(
        DLPACK_SOURCE,
        tensor_signature_contract_with_convention("dlpack", RustBridgeParamConvention::Own),
    );
    let context = context_with_source(DLPACK_SOURCE, "sifr_tensor_bridge");

    apply_package_rust_interop_metadata(generated, Some(context))
        .expect("explicit DLPack transfer contract should pass");
}

#[test]
fn package_rust_interop_rejects_advanced_data_metadata_without_data_kind() {
    let source = ARROW_SOURCE.replace("data=arrow_record_batch, ", "");
    let generated = generated_from_source(&source, record_batch_signature_contract());
    let context = context_with_source(&source, "sifr_arrow_bridge");

    let diagnostics = interop_errors(generated, Some(context), "missing data kind must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ZC-0001");
    assert!(diagnostics[0].message.contains("requires `data=`"));
}

#[test]
fn package_rust_interop_rejects_advanced_data_metadata_without_ownership() {
    let source = ARROW_SOURCE.replace(", ownership=borrowed", "");
    let generated = generated_from_source(&source, record_batch_signature_contract());
    let context = context_with_source(&source, "sifr_arrow_bridge");

    let diagnostics = interop_errors(generated, Some(context), "missing ownership must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ZC-0001");
    assert!(diagnostics[0].message.contains("requires `ownership=`"));
}

#[test]
fn package_rust_interop_rejects_unknown_advanced_data_kind() {
    let source = ARROW_SOURCE.replace("data=arrow_record_batch", "data=parquet");
    let generated = generated_from_source(&source, record_batch_signature_contract());
    let context = context_with_source(&source, "sifr_arrow_bridge");

    let diagnostics = interop_errors(generated, Some(context), "unknown data kind must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ZC-0001");
    assert!(diagnostics[0].message.contains("arrow_array"));
}

#[test]
fn package_rust_interop_rejects_arrow_view_without_schema() {
    let source = ARROW_SOURCE.replace(", schema=sifr_arrow_bridge.schema.RecordBatch", "");
    let generated = generated_from_source(&source, record_batch_signature_contract());
    let context = context_with_source(&source, "sifr_arrow_bridge");

    let diagnostics = interop_errors(generated, Some(context), "missing schema must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ZC-0001");
    assert!(diagnostics[0].message.contains("require `schema=`"));
}

#[test]
fn package_rust_interop_rejects_arrow_view_with_tensor_metadata() {
    let source = ARROW_SOURCE.replace("ownership=borrowed", "ownership=borrowed, dtype=f32");
    let generated = generated_from_source(&source, record_batch_signature_contract());
    let context = context_with_source(&source, "sifr_arrow_bridge");

    let diagnostics = interop_errors(generated, Some(context), "tensor metadata must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ZC-0001");
    assert!(
        diagnostics[0]
            .message
            .contains("cannot declare tensor metadata")
    );
}

#[test]
fn package_rust_interop_rejects_arrow_transfer_ownership() {
    let source = ARROW_SOURCE.replace("ownership=borrowed", "ownership=transfer");
    let generated = generated_from_source(&source, record_batch_signature_contract());
    let context = context_with_source(&source, "sifr_arrow_bridge");

    let diagnostics = interop_errors(generated, Some(context), "arrow transfer must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ZC-0001");
    assert!(
        diagnostics[0]
            .message
            .contains("ownership=borrowed` or `ownership=owned")
    );
}

#[test]
fn package_rust_interop_rejects_schema_outside_arrow_bridge() {
    let source = ARROW_SOURCE.replace(
        "schema=sifr_arrow_bridge.schema.RecordBatch",
        "schema=sifr_tensor_bridge.schema.RecordBatch",
    );
    let generated = generated_from_source(&source, record_batch_signature_contract());
    let context = context_with_source(&source, "sifr_arrow_bridge");

    let diagnostics = interop_errors(generated, Some(context), "schema root must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ZC-0001");
    assert!(diagnostics[0].message.contains("sifr_arrow_bridge"));
}

#[test]
fn package_rust_interop_rejects_arrow_view_on_tensor_bridge_root() {
    let source = ARROW_SOURCE.replace(
        "@rust(sifr_arrow_bridge.record_batch_from_columns",
        "@rust(sifr_tensor_bridge.record_batch_from_columns",
    );
    let generated = generated_from_source(&source, record_batch_signature_contract());
    let context = context_with_source(&source, "sifr_tensor_bridge");

    let diagnostics = interop_errors(generated, Some(context), "wrong bridge root must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ZC-0001");
    assert!(
        diagnostics[0]
            .message
            .contains("matching shared bridge crate")
    );
}

#[test]
fn package_rust_interop_rejects_implicit_dlpack_ownership() {
    let source = DLPACK_SOURCE.replace("ownership=transfer", "ownership=borrowed");
    let generated = generated_from_source(
        &source,
        tensor_signature_contract_with_convention("dlpack", RustBridgeParamConvention::Own),
    );
    let context = context_with_source(&source, "sifr_tensor_bridge");

    let diagnostics = interop_errors(generated, Some(context), "DLPack ownership must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ZC-0001");
    assert!(diagnostics[0].message.contains("ownership=transfer"));
}

#[test]
fn package_rust_interop_rejects_dlpack_transfer_from_borrowed_owner() {
    let generated = generated_from_source(DLPACK_SOURCE, tensor_signature_contract("dlpack"));
    let context = context_with_source(DLPACK_SOURCE, "sifr_tensor_bridge");

    let diagnostics = interop_errors(generated, Some(context), "borrowed transfer must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ZC-0001");
    assert!(diagnostics[0].message.contains("owned owner parameter"));
}

#[test]
fn package_rust_interop_rejects_dlpack_without_protocol() {
    let source = DLPACK_SOURCE.replace(", protocol=sifr_tensor_bridge.dlpack.Capsule", "");
    let generated = generated_from_source(
        &source,
        tensor_signature_contract_with_convention("dlpack", RustBridgeParamConvention::Own),
    );
    let context = context_with_source(&source, "sifr_tensor_bridge");

    let diagnostics = interop_errors(generated, Some(context), "missing protocol must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ZC-0001");
    assert!(diagnostics[0].message.contains("requires `protocol=`"));
}

#[test]
fn package_rust_interop_rejects_tensor_schema_metadata() {
    let source = TENSOR_SOURCE.replace(
        "ownership=borrowed",
        "ownership=borrowed, schema=sifr_arrow_bridge.schema.Tensor",
    );
    let generated = generated_from_source(&source, tensor_signature_contract("tensor"));
    let context = context_with_source(&source, "sifr_tensor_bridge");

    let diagnostics = interop_errors(generated, Some(context), "tensor schema must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ZC-0001");
    assert!(diagnostics[0].message.contains("cannot declare `schema=`"));
}

#[test]
fn package_rust_interop_rejects_negative_tensor_rank() {
    let source = TENSOR_SOURCE.replace("rank=2", "rank=-1");
    let generated = generated_from_source(&source, tensor_signature_contract("tensor"));
    let context = context_with_source(&source, "sifr_tensor_bridge");

    let diagnostics = interop_errors(generated, Some(context), "invalid rank must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ZC-0001");
    assert!(diagnostics[0].message.contains("non-negative integer"));
}

#[test]
fn package_rust_interop_rejects_invalid_tensor_dtype() {
    let source = TENSOR_SOURCE.replace("dtype=f32", "dtype=bool32");
    let generated = generated_from_source(&source, tensor_signature_contract("tensor"));
    let context = context_with_source(&source, "sifr_tensor_bridge");

    let diagnostics = interop_errors(generated, Some(context), "invalid dtype must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ZC-0001");
    assert!(diagnostics[0].message.contains("supported tensor dtype"));
}

#[test]
fn package_rust_interop_rejects_invalid_tensor_layout() {
    let source = TENSOR_SOURCE.replace("layout=strided", "layout=ragged");
    let generated = generated_from_source(&source, tensor_signature_contract("tensor"));
    let context = context_with_source(&source, "sifr_tensor_bridge");

    let diagnostics = interop_errors(generated, Some(context), "invalid layout must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ZC-0001");
    assert!(diagnostics[0].message.contains("contiguous or strided"));
}

#[test]
fn package_rust_interop_rejects_invalid_tensor_device() {
    let source = TENSOR_SOURCE.replace("device=cpu", "device=cuda");
    let generated = generated_from_source(&source, tensor_signature_contract("tensor"));
    let context = context_with_source(&source, "sifr_tensor_bridge");

    let diagnostics = interop_errors(generated, Some(context), "invalid device must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ZC-0001");
    assert!(diagnostics[0].message.contains("must be cpu"));
}

#[test]
fn package_rust_interop_rejects_negative_tensor_shape_dimension() {
    let source = TENSOR_SOURCE.replace("shape=[2, 3]", "shape=[2, -3]");
    let generated = generated_from_source(&source, tensor_signature_contract("tensor"));
    let context = context_with_source(&source, "sifr_tensor_bridge");

    let diagnostics = interop_errors(generated, Some(context), "negative shape must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ZC-0001");
    assert!(diagnostics[0].message.contains("non-negative integers"));
}

#[test]
fn package_rust_interop_rejects_tensor_shape_stride_rank_mismatch() {
    let source = TENSOR_SOURCE.replace("strides=[3, 1]", "strides=[1]");
    let generated = generated_from_source(&source, tensor_signature_contract("tensor"));
    let context = context_with_source(&source, "sifr_tensor_bridge");

    let diagnostics = interop_errors(generated, Some(context), "stride length must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ZC-0001");
    assert!(diagnostics[0].message.contains("same rank"));
}

#[test]
fn package_rust_interop_rejects_tensor_rank_shape_length_mismatch() {
    let source = TENSOR_SOURCE.replace("rank=2", "rank=3");
    let generated = generated_from_source(&source, tensor_signature_contract("tensor"));
    let context = context_with_source(&source, "sifr_tensor_bridge");

    let diagnostics = interop_errors(generated, Some(context), "rank length must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ZC-0001");
    assert!(diagnostics[0].message.contains("must match"));
}

fn generated_from_source(
    source: &str,
    signature: RustBridgeSignatureContract,
) -> GeneratedBinaryProject {
    let parsed = sifr_syntax::parse_module(source, Some("app")).expect("source should parse");
    let module = sifr_lowering::lower_module(parsed.suite())
        .map(|result| result.module)
        .expect("source should lower");
    let mut result = generate_rust_multi_with_metadata(&[("app", &module)], &StdlibCode::default());
    result.interop.rust.bridge_contracts.signatures = vec![signature];
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

fn context_with_source(source: &str, bridge_crate: &str) -> PackageRustInteropContext {
    let mut context = package_context(
        TrustPolicy::default(),
        vec![backend_with_manifest(
            bridge_crate,
            PathBuf::from(format!("/ws/{bridge_crate}/Cargo.toml")),
        )],
    );
    context.module_sources.insert(
        "app".to_string(),
        RustInteropModuleSource {
            source: source.to_string(),
            display_path: "/ws/app/sifr/app.sifr".to_string(),
        },
    );
    context
}

fn record_batch_signature_contract() -> RustBridgeSignatureContract {
    signature_contract(
        vec![param_contract(
            "columns",
            RustBridgeParamConvention::Borrow,
            bytes_contract(),
        )],
        result_contract(
            record_type_contract("ArrowRecordBatch"),
            error_type_contract("ArrowError"),
        ),
    )
    .with_target("record_batch")
}

fn tensor_signature_contract(name: &str) -> RustBridgeSignatureContract {
    tensor_signature_contract_with_convention(name, RustBridgeParamConvention::Borrow)
}

fn tensor_signature_contract_with_convention(
    name: &str,
    convention: RustBridgeParamConvention,
) -> RustBridgeSignatureContract {
    signature_contract(
        vec![param_contract("input", convention, bytes_contract())],
        result_contract(
            record_type_contract("TensorView"),
            error_type_contract("TensorError"),
        ),
    )
    .with_target(name)
}

trait SignatureTarget {
    fn with_target(self, name: &str) -> Self;
}

impl SignatureTarget for RustBridgeSignatureContract {
    fn with_target(mut self, name: &str) -> Self {
        self.canonical_target_path = format!("app.{name}");
        self.owner = RustInteropOwner::Function {
            name: name.to_string(),
        };
        self.panic_error = sifr_codegen::RustBridgePanicErrorContract::OrdinaryAndWrapper;
        self
    }
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

fn record_type_contract(name: &str) -> RustBridgeTypeContract {
    RustBridgeTypeContract {
        sifr_type: name.to_string(),
        rust_borrowed_type: Some(format!("crate::__sifr_bridge::{name}Bridge")),
        rust_owned_type: Some(format!("crate::__sifr_bridge::{name}Bridge")),
        rust_return_type: Some(format!("crate::__sifr_bridge::{name}Bridge")),
        kind: RustBridgeTypeKind::GeneratedRecord,
        unsupported_reason: None,
    }
}

fn error_type_contract(name: &str) -> RustBridgeTypeContract {
    RustBridgeTypeContract {
        sifr_type: name.to_string(),
        rust_borrowed_type: Some(format!("crate::__sifr_bridge::{name}Bridge")),
        rust_owned_type: Some(format!("crate::__sifr_bridge::{name}Bridge")),
        rust_return_type: Some(format!("crate::__sifr_bridge::{name}Bridge")),
        kind: RustBridgeTypeKind::GeneratedError,
        unsupported_reason: None,
    }
}

fn generated_error_contract(sifr_type: &str, rust_type: &str) -> RustBridgeTypeContract {
    RustBridgeTypeContract {
        sifr_type: sifr_type.to_string(),
        rust_borrowed_type: Some(rust_type.to_string()),
        rust_owned_type: Some(rust_type.to_string()),
        rust_return_type: Some(rust_type.to_string()),
        kind: RustBridgeTypeKind::GeneratedError,
        unsupported_reason: None,
    }
}
