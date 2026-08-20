use super::cargo_invocation_trace::record_cargo_invocation;
use super::cargo_resolution::{prepare_cargo_resolution, CargoResolutionPolicy};
use super::rust_interop_digest::fnv1a64_hex;
use super::rust_interop_panic_probe::panic_mapper_probe;
use super::rust_interop_probe_cache::{
    mark_probe_cache_hit, probe_cache_file, probe_cache_key, ProbeCacheKeyCache,
};
use super::rust_interop_probe_diagnostics::{
    classify_probe_failure, probe_cargo_resolution_failure, probe_resolution_diagnostics,
};
use super::rust_interop_probe_features::dependency_features;
use super::rust_interop_probe_manifest::{probe_cargo_toml, probe_cargo_vendor_args};
use super::rust_interop_probe_nonce::unique_probe_nonce;
use super::rust_interop_probe_paths::probe_cargo_target_dir;
use super::rust_interop_sqlx_offline::{
    configure_hermetic_build_environment, validate_probe_sqlx_offline_metadata,
};
use sifr_codegen::{
    RustBridgeParamConvention, RustBridgeSignatureContract, RustInteropPlanDeclaration,
};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{RustInteropDecoratorKind, RustInteropValue, RustTargetPath};
use sifr_package::BackendCrateMetadata;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum AsyncThreadAffinity {
    #[default]
    None,
    TokioCurrentThread,
}

#[derive(Clone)]
pub(super) struct PendingRustBridgeProbe {
    pub(super) declaration: RustInteropPlanDeclaration,
    pub(super) path: RustTargetPath,
    pub(super) backend: BackendCrateMetadata,
    pub(super) source_prefix: Option<String>,
    pub(super) signature: Option<RustBridgeSignatureContract>,
    pub(super) async_thread_affinity: AsyncThreadAffinity,
    pub(super) zero_copy_obligations: (bool, bool),
    pub(super) trusted_sysroot: bool,
    pub(super) sysroot_runtime_crate: PathBuf,
    pub(super) sysroot_vendor_dir: Option<PathBuf>,
    pub(super) cargo_resolution: CargoResolutionPolicy,
}

pub(super) struct ProbeExecutionFailure {
    pub(super) code: DiagnosticCode,
    pub(super) message_template: &'static str,
    pub(super) args: Vec<(&'static str, String)>,
    pub(super) notes: Vec<String>,
}

pub(super) fn execute_direct_cargo_probe(
    probe: &PendingRustBridgeProbe,
    cache: &mut ProbeCacheKeyCache,
) -> Result<(), ProbeExecutionFailure> {
    if !probe.backend.cargo_manifest_path.is_file() {
        if probe.cargo_resolution.lock_mode != sifr_package::CargoLockMode::Normal {
            return Err(probe_cargo_resolution_failure(format!(
                "Rust probe Cargo manifest '{}' is missing in {} mode",
                probe.backend.cargo_manifest_path.display(),
                probe.cargo_resolution.lock_mode.as_str()
            )));
        }
        return Ok(());
    }
    let Some(backend_root) = probe.backend.cargo_manifest_path.parent() else {
        return Ok(());
    };
    let dependency_features =
        dependency_features(&probe.backend.dependency_name, backend_root, &probe.path);
    let requires_structural_runtime = probe
        .signature
        .as_ref()
        .is_some_and(|signature| !signature.structural_type_params.is_empty());
    let probe_manifest = probe_cargo_toml(
        &probe.backend.dependency_name,
        &probe.backend.cargo_package_name,
        backend_root,
        &probe.sysroot_runtime_crate,
        &dependency_features,
        requires_structural_runtime,
    );
    let probe_source = probe_source(probe);
    let invocation_cwd = env::current_dir()
        .map_err(|error| probe_io_failure(format!("failed to resolve Rust probe cwd: {error}")))?;
    let cache_key = probe_cache_key(probe, backend_root, &probe_manifest, &probe_source, cache);
    let cache_file = probe_cache_file(&cache_key, &invocation_cwd);
    if cache_file.is_file() {
        return Ok(());
    }
    validate_probe_sqlx_offline_metadata(probe, backend_root)?;
    let probe_root = std::env::temp_dir().join(format!(
        "sifr_rust_probe_{}_{}_{}",
        std::process::id(),
        unique_probe_nonce(),
        fnv1a64_hex(
            format!(
                "{}:{}",
                probe.backend.cargo_package_id.0,
                probe.path.dotted()
            )
            .as_bytes()
        )
    ));
    if probe_root.exists() {
        let _ = fs::remove_dir_all(&probe_root);
    }
    fs::create_dir_all(probe_root.join("src")).map_err(|error| {
        probe_io_failure(format!("failed to create Rust probe project: {error}"))
    })?;
    fs::write(probe_root.join("Cargo.toml"), probe_manifest).map_err(|error| {
        probe_io_failure(format!("failed to write Rust probe manifest: {error}"))
    })?;
    fs::write(probe_root.join("src/lib.rs"), probe_source)
        .map_err(|error| probe_io_failure(format!("failed to write Rust probe source: {error}")))?;

    let vendor_dir = probe
        .cargo_resolution
        .uses_sysroot_vendor()
        .then(|| {
            probe.sysroot_vendor_dir.as_deref().or_else(|| {
                probe
                    .cargo_resolution
                    .trusted_vendor_dirs
                    .first()
                    .map(PathBuf::as_path)
            })
        })
        .flatten();
    let cargo_prefix_args = probe_cargo_vendor_args(vendor_dir);
    let prepared_resolution =
        prepare_cargo_resolution(&probe_root, &probe.cargo_resolution, &cargo_prefix_args)
            .map_err(|diagnostics| probe_resolution_diagnostics(&diagnostics))?;
    let mut command = Command::new("cargo");
    command
        .args(&cargo_prefix_args)
        .args(["check", "--quiet"])
        .current_dir(&probe_root);
    if let Some(argument) = probe.cargo_resolution.lock_mode.cargo_arg() {
        command.arg(argument);
    }
    if !matches!(
        probe.cargo_resolution.lock_mode,
        sifr_package::CargoLockMode::Normal | sifr_package::CargoLockMode::Frozen
    ) {
        command.arg("--frozen");
    }
    command.env("CARGO_TARGET_DIR", probe_cargo_target_dir(&invocation_cwd));
    configure_hermetic_build_environment(&mut command);
    record_cargo_invocation("rust-probe", probe.cargo_resolution.lock_mode, &command);
    let output = command
        .output()
        .map_err(|error| probe_io_failure(format!("failed to run Rust probe: {error}")))?;
    let unchanged = prepared_resolution
        .assert_unchanged()
        .map_err(|diagnostics| probe_resolution_diagnostics(&diagnostics));
    let _ = fs::remove_dir_all(&probe_root);
    unchanged?;
    if output.status.success() {
        mark_probe_cache_hit(&cache_file);
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(classify_probe_failure(probe, &stderr))
}

fn probe_source(probe: &PendingRustBridgeProbe) -> String {
    let rust_path = probe.path.segments.join("::");
    let body = match probe.declaration.declaration.kind {
        RustInteropDecoratorKind::Opaque => opaque_probe_source(probe, &rust_path),
        RustInteropDecoratorKind::Callback | RustInteropDecoratorKind::Structural => {
            unreachable!("targetless Rust metadata never enters probe planning")
        }
        RustInteropDecoratorKind::ZeroCopy => {
            zero_copy_type_probe_source(probe.zero_copy_obligations, &rust_path)
        }
        RustInteropDecoratorKind::Function
        | RustInteropDecoratorKind::Async
        | RustInteropDecoratorKind::View => {
            if let Some(signature) = &probe.signature {
                signature_probe_source(probe, signature, &rust_path)
            } else if let Some(mapper) = panic_mapper_probe(&probe.declaration, &probe.path) {
                panic_mapper_probe_source(&mapper)
            } else {
                format!("#![allow(dead_code)]\nfn __sifr_probe() {{ let _ = {rust_path}; }}\n")
            }
        }
    };
    let Some(prefix) = &probe.source_prefix else {
        return body;
    };
    prefixed_probe_source(prefix, &body)
}

pub(super) fn zero_copy_type_probe_source(obligations: (bool, bool), rust_path: &str) -> String {
    let (requires_send, requires_sync) = obligations;
    let mut out = format!("#![allow(dead_code)]\ntype __SifrView = {rust_path};\n");
    if requires_send {
        out.push_str("fn __sifr_assert_send<T: Send>() {}\n");
    }
    if requires_sync {
        out.push_str("fn __sifr_assert_sync<T: Sync>() {}\n");
    }
    out.push_str("fn __sifr_probe() {\n");
    if requires_send {
        out.push_str("    __sifr_assert_send::<__SifrView>();\n");
    }
    if requires_sync {
        out.push_str("    __sifr_assert_sync::<__SifrView>();\n");
    }
    out.push_str("}\n");
    out
}

fn panic_mapper_probe_source(mapper: &super::rust_interop_panic_probe::PanicMapperProbe) -> String {
    format!(
        "#![allow(dead_code)]\n{}\nfn __sifr_probe() {{\n    {}\n}}\n",
        mapper.assertion, mapper.invocation
    )
}

fn prefixed_probe_source(prefix: &str, body: &str) -> String {
    let body = body.strip_prefix("#![allow(dead_code)]\n").unwrap_or(body);
    format!("#![allow(dead_code)]\n{prefix}\n{body}")
}

fn opaque_probe_source(probe: &PendingRustBridgeProbe, rust_path: &str) -> String {
    opaque_type_probe_source(
        rust_path,
        opaque_target_argument(probe, "structural").as_deref(),
        opaque_bool_argument(probe, "send"),
        opaque_bool_argument(probe, "sync"),
        opaque_symbol_argument(probe, "clone") == Some("copy"),
    )
}

pub(super) fn opaque_type_probe_source(
    rust_path: &str,
    structural_mapping: Option<&str>,
    requires_send: bool,
    requires_sync: bool,
    requires_copy: bool,
) -> String {
    let mut out = format!("#![allow(dead_code)]\ntype __SifrProbe = {rust_path};\n");
    if let Some(mapping) = structural_mapping {
        let _ = writeln!(out, "type __SifrMapping = {mapping};");
        out.push_str(
            "fn __sifr_assert_structural_mapping<M: sifr_runtime::interop::structural::StructuralMapping<__SifrProbe>>() {}\n",
        );
    }
    if requires_send {
        out.push_str("fn __sifr_assert_send<T: Send>() {}\n");
    }
    if requires_sync {
        out.push_str("fn __sifr_assert_sync<T: Sync>() {}\n");
    }
    if requires_copy {
        out.push_str("fn __sifr_assert_copy<T: Copy>() {}\n");
    }
    out.push_str("fn __sifr_probe() {\n");
    if structural_mapping.is_some() {
        out.push_str("    __sifr_assert_structural_mapping::<__SifrMapping>();\n");
    }
    if requires_send {
        out.push_str("    __sifr_assert_send::<__SifrProbe>();\n");
    }
    if requires_sync {
        out.push_str("    __sifr_assert_sync::<__SifrProbe>();\n");
    }
    if requires_copy {
        out.push_str("    __sifr_assert_copy::<__SifrProbe>();\n");
    }
    out.push_str("}\n");
    out
}

fn opaque_target_argument(probe: &PendingRustBridgeProbe, name: &str) -> Option<String> {
    probe
        .declaration
        .declaration
        .arguments
        .iter()
        .find(|argument| argument.name.as_deref() == Some(name))
        .and_then(|argument| match &argument.value {
            sifr_ir::RustInteropValue::TargetPath(path) => Some(path.segments.join("::")),
            _ => None,
        })
}

fn signature_probe_source(
    probe: &PendingRustBridgeProbe,
    signature: &RustBridgeSignatureContract,
    rust_path: &str,
) -> String {
    if is_python_raw_callback_probe(rust_path) {
        return python_raw_callback_probe_source(signature, rust_path);
    }
    if !signature.structural_type_params.is_empty() {
        return structural_signature_probe_source(signature, rust_path);
    }
    let params = signature
        .params
        .iter()
        .map(|param| {
            rust_param_type(param.convention, &param.ty)
                .unwrap_or_else(|| "__SifrUnsupportedBridgeType".to_string())
        })
        .collect::<Vec<_>>()
        .join(", ");
    let return_type = signature_return_probe_type(&signature.return_type);
    let mut out = String::new();
    out.push_str("#![allow(dead_code)]\n");
    out.push_str(&generated_bridge_type_stubs(signature));
    if is_async_probe(probe) {
        out.push_str("fn __sifr_assert_async_future<Fut");
        if return_type.display_error_generic {
            out.push_str(", __SifrBridgeError");
        }
        out.push_str(">(_future: Fut)\nwhere\n    Fut: std::future::Future<Output = ");
        out.push_str(&return_type.ty);
        out.push('>');
        if async_future_requires_send(probe) {
            out.push_str(" + Send");
        }
        if return_type.display_error_generic {
            out.push_str(",\n    __SifrBridgeError: std::fmt::Display");
        }
        out.push_str(",\n{}\n");
        let has_borrowed_param = signature.params.iter().any(|param| {
            matches!(
                param.convention,
                RustBridgeParamConvention::Borrow | RustBridgeParamConvention::MutableBorrow
            )
        });
        out.push_str("fn __sifr_probe");
        if has_borrowed_param {
            out.push_str("<'__sifr_call>");
        }
        out.push('(');
        let probe_params = signature
            .params
            .iter()
            .enumerate()
            .map(|(index, param)| {
                let ty = rust_param_type(param.convention, &param.ty)
                    .unwrap_or_else(|| "__SifrUnsupportedBridgeType".to_string());
                let ty = if has_borrowed_param {
                    bind_probe_reference_lifetime(&ty)
                } else {
                    ty
                };
                format!("__sifr_arg_{index}: {ty}")
            })
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&probe_params);
        out.push_str(") {\n");
        let arguments = signature
            .params
            .iter()
            .enumerate()
            .map(|(index, _)| format!("__sifr_arg_{index}"))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str("    __sifr_assert_async_future(");
        out.push_str(rust_path);
        out.push('(');
        out.push_str(&arguments);
        out.push_str("));\n");
    } else {
        out.push_str("fn __sifr_assert_signature");
        if return_type.display_error_generic {
            out.push_str("<__SifrBridgeError: std::fmt::Display>");
        }
        out.push_str("(_f: fn(");
        out.push_str(&params);
        out.push_str(") -> ");
        out.push_str(&return_type.ty);
        out.push_str(") {}\nfn __sifr_probe() {\n    __sifr_assert_signature(");
        out.push_str(rust_path);
        out.push_str(");\n");
    }
    out.push_str("}\n");
    out
}

fn structural_signature_probe_source(
    signature: &RustBridgeSignatureContract,
    rust_path: &str,
) -> String {
    let mut out = String::from("#![allow(dead_code)]\n");
    out.push_str(&generated_bridge_type_stubs(signature));
    out.push_str("fn __sifr_probe<");
    out.push_str(&signature.structural_type_params.join(", "));
    out.push_str(">(");
    out.push_str(
        &signature
            .params
            .iter()
            .enumerate()
            .map(|(index, param)| {
                let ty = rust_param_type(param.convention, &param.ty)
                    .unwrap_or_else(|| "__SifrUnsupportedBridgeType".to_string());
                format!("__sifr_arg_{index}: {ty}")
            })
            .collect::<Vec<_>>()
            .join(", "),
    );
    out.push_str(")\nwhere\n");
    for type_param in &signature.structural_type_params {
        out.push_str("    ");
        out.push_str(type_param);
        let is_context = signature
            .method_slot_contract
            .as_ref()
            .is_some_and(|contract| contract.context_type_param == *type_param);
        if is_context {
            out.push_str(": ::sifr_runtime::interop::structural::StructuralType");
        } else {
            out.push_str(": ::sifr_runtime::interop::structural::StructuralConstruct + ::sifr_runtime::interop::structural::StructuralProject");
        }
        if signature.static_program_type_params.contains(type_param) {
            out.push_str(" + ::sifr_runtime::interop::structural::StaticProgramType");
        }
        if let Some(contract) = signature
            .method_slot_contract
            .as_ref()
            .filter(|contract| contract.owner_type_param == *type_param)
        {
            if contract.context_mutable {
                out.push_str(" + ::sifr_runtime::interop::structural::MethodSlotTable<");
                out.push_str(&contract.context_type_param);
                out.push('>');
            } else {
                out.push_str(" + for<'__sifr_context> ::sifr_runtime::interop::structural::MethodSlotTable<::sifr_runtime::interop::structural::SharedContext<'__sifr_context, ");
                out.push_str(&contract.context_type_param);
                out.push_str(">>");
            }
        }
        out.push_str(",\n");
    }
    out.push_str("{\n    let _: ");
    let return_type = signature_return_probe_type(&signature.return_type);
    let normalized_return_type = if return_type.display_error_generic {
        return_type.ty.replace("__SifrBridgeError", "String")
    } else {
        return_type.ty
    };
    out.push_str(&normalized_return_type);
    out.push_str(" = ");
    if return_type.display_error_generic {
        out.push('(');
    }
    out.push_str(rust_path);
    out.push_str("::<");
    out.push_str(&signature.structural_type_params.join(", "));
    out.push_str(">(");
    out.push_str(
        &(0..signature.params.len())
            .map(|index| format!("__sifr_arg_{index}"))
            .collect::<Vec<_>>()
            .join(", "),
    );
    out.push(')');
    if return_type.display_error_generic {
        out.push_str(").map_err(|error| error.to_string())");
    }
    out.push_str(";\n}\n");
    out
}

fn is_python_raw_callback_probe(rust_path: &str) -> bool {
    // Trusted sysroot Python callback constructors are generic over an actual
    // raw-object closure; the normal callback probe uses the package interop
    // marker type and cannot satisfy that bound.
    matches!(
        rust_path,
        "sifr_stdlib::python::py_local_callback"
            | "sifr_stdlib::python::py_threadsafe_callback"
            | "::sifr_stdlib::python::py_local_callback"
            | "::sifr_stdlib::python::py_threadsafe_callback"
    )
}

fn python_raw_callback_probe_source(
    signature: &RustBridgeSignatureContract,
    rust_path: &str,
) -> String {
    let mut out = String::new();
    out.push_str("#![allow(dead_code)]\n");
    out.push_str(&generated_bridge_type_stubs(signature));
    out.push_str(
        "fn __sifr_sample_python_callback(\n    _arg: ::sifr_runtime::interop::Handle<::sifr_runtime::python::ForeignObject>,\n) -> Result<::sifr_runtime::interop::Handle<::sifr_runtime::python::ForeignObject>, ::sifr_stdlib::python::PythonError> {\n    unreachable!()\n}\n",
    );
    out.push_str("fn __sifr_probe() {\n    let _: ");
    out.push_str(
        &signature_return_probe_type(&signature.return_type)
            .ty
            .replace("__SifrBridgeError", "::sifr_stdlib::python::PythonError"),
    );
    out.push_str(" = ");
    out.push_str(rust_path);
    out.push_str("(__sifr_sample_python_callback);\n}\n");
    out
}

struct SignatureReturnProbeType {
    ty: String,
    display_error_generic: bool,
}

fn signature_return_probe_type(
    return_type: &sifr_codegen::RustBridgeTypeContract,
) -> SignatureReturnProbeType {
    let ty = return_type
        .rust_return_type
        .as_deref()
        .unwrap_or("__SifrUnsupportedBridgeType");
    if let Some(mapped) = display_error_result_probe_type(ty) {
        return SignatureReturnProbeType {
            ty: mapped,
            display_error_generic: true,
        };
    }
    SignatureReturnProbeType {
        ty: ty.to_string(),
        display_error_generic: false,
    }
}

fn display_error_result_probe_type(return_type: &str) -> Option<String> {
    let inner = return_type
        .strip_prefix("Result<")
        .and_then(|value| value.strip_suffix('>'))?;
    let (ok_type, err_type) = inner.rsplit_once(", ")?;
    if !err_type.contains("__sifr_bridge") || !err_type.ends_with("Bridge") {
        return None;
    }
    Some(format!("Result<{ok_type}, __SifrBridgeError>"))
}

fn rust_param_type(
    convention: RustBridgeParamConvention,
    ty: &sifr_codegen::RustBridgeTypeContract,
) -> Option<String> {
    match convention {
        RustBridgeParamConvention::Borrow => ty.rust_borrowed_type.clone(),
        RustBridgeParamConvention::MutableBorrow => {
            ty.rust_borrowed_type.as_deref().map(mutable_borrow_type)
        }
        RustBridgeParamConvention::Own | RustBridgeParamConvention::OwnMutable => {
            ty.rust_owned_type.clone()
        }
    }
}

fn mutable_borrow_type(rust_type: &str) -> String {
    rust_type.strip_prefix('&').map_or_else(
        || rust_type.to_string(),
        |inner| format!("&mut {}", inner.trim_start()),
    )
}

fn bind_probe_reference_lifetime(rust_type: &str) -> String {
    rust_type.strip_prefix('&').map_or_else(
        || rust_type.to_string(),
        |inner| format!("&'__sifr_call {}", inner.trim_start()),
    )
}

fn is_async_probe(probe: &PendingRustBridgeProbe) -> bool {
    probe.declaration.declaration.kind == RustInteropDecoratorKind::Async
        || probe
            .declaration
            .declaration
            .abi_requirements
            .async_boundary
}

pub(super) fn async_future_requires_send(probe: &PendingRustBridgeProbe) -> bool {
    if !is_async_probe(probe) {
        return false;
    }
    probe.async_thread_affinity != AsyncThreadAffinity::TokioCurrentThread
}

fn opaque_bool_argument(probe: &PendingRustBridgeProbe, name: &str) -> bool {
    probe
        .declaration
        .declaration
        .arguments
        .iter()
        .find(|argument| argument.name.as_deref() == Some(name))
        .is_some_and(|argument| matches!(argument.value, RustInteropValue::Boolean(true)))
}

fn opaque_symbol_argument<'a>(probe: &'a PendingRustBridgeProbe, name: &str) -> Option<&'a str> {
    probe
        .declaration
        .declaration
        .arguments
        .iter()
        .find(|argument| argument.name.as_deref() == Some(name))
        .and_then(|argument| match &argument.value {
            RustInteropValue::Symbol(symbol) => Some(symbol.as_str()),
            _ => None,
        })
}

fn generated_bridge_type_stubs(signature: &RustBridgeSignatureContract) -> String {
    let mut root = BridgeStubModule::default();
    for param in &signature.params {
        collect_generated_bridge_paths(&param.ty, &mut root);
    }
    collect_generated_bridge_paths(&signature.return_type, &mut root);
    if root.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    out.push_str("mod __sifr_bridge {\n");
    render_bridge_stub_module(&mut out, &root);
    out.push_str("}\n");
    out
}

#[derive(Default)]
struct BridgeStubModule {
    children: BTreeMap<String, BridgeStubModule>,
    structs: BTreeSet<String>,
}

impl BridgeStubModule {
    fn is_empty(&self) -> bool {
        self.children.is_empty() && self.structs.is_empty()
    }

    fn insert_path(&mut self, path: &[String]) {
        let Some((bridge_name, modules)) = path.split_last() else {
            return;
        };
        let mut module = self;
        for segment in modules {
            module = module.children.entry(segment.clone()).or_default();
        }
        module.structs.insert(bridge_name.clone());
    }
}

fn render_bridge_stub_module(out: &mut String, module: &BridgeStubModule) {
    for (name, child) in &module.children {
        out.push_str("pub mod ");
        out.push_str(name);
        out.push_str(" {\n");
        render_bridge_stub_module(out, child);
        out.push_str("}\n");
    }
    for name in &module.structs {
        out.push_str("#[derive(Clone, Debug, PartialEq, Eq)]\npub struct ");
        out.push_str(name);
        out.push_str(";\n");
    }
}

fn collect_generated_bridge_paths(
    ty: &sifr_codegen::RustBridgeTypeContract,
    root: &mut BridgeStubModule,
) {
    for candidate in [
        ty.rust_borrowed_type.as_deref(),
        ty.rust_owned_type.as_deref(),
        ty.rust_return_type.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        let segments = candidate
            .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
            .filter(|segment| !segment.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();
        for (index, segment) in segments.iter().enumerate() {
            if segment != "__sifr_bridge" {
                continue;
            }
            let mut path = Vec::new();
            for bridge_segment in &segments[index + 1..] {
                path.push(bridge_segment.clone());
                if bridge_segment.ends_with("Bridge") && !bridge_segment.starts_with("__") {
                    root.insert_path(&path);
                    break;
                }
            }
        }
    }
}

fn probe_io_failure(message: String) -> ProbeExecutionFailure {
    ProbeExecutionFailure {
        code: DiagnosticCode::RUST_CARGO_METADATA,
        message_template: "{message}",
        args: vec![("message", message)],
        notes: Vec::new(),
    }
}

pub(super) fn canonical_sifr_target_path(declaration: &RustInteropPlanDeclaration) -> String {
    let mut path = declaration
        .module_name
        .clone()
        .unwrap_or_else(|| "main".to_string());
    match &declaration.owner {
        sifr_codegen::RustInteropOwner::Function { name } => {
            path.push('.');
            path.push_str(name);
        }
        sifr_codegen::RustInteropOwner::Class { name } => {
            path.push('.');
            path.push_str(name);
        }
        sifr_codegen::RustInteropOwner::Method { class_name, name } => {
            path.push('.');
            path.push_str(class_name);
            path.push('.');
            path.push_str(name);
        }
    }
    path
}

#[cfg(test)]
#[path = "rust_interop_probe_tests.rs"]
mod tests;
