use super::rust_interop_digest::fnv1a64_hex;
use sifr_codegen::{
    RustBridgeParamConvention, RustBridgeSignatureContract, RustInteropPlanDeclaration,
};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{RustInteropDecoratorKind, RustTargetPath};
use sifr_package::BackendCrateMetadata;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Clone)]
pub(super) struct PendingRustBridgeProbe {
    pub(super) declaration: RustInteropPlanDeclaration,
    pub(super) path: RustTargetPath,
    pub(super) backend: BackendCrateMetadata,
    pub(super) signature: Option<RustBridgeSignatureContract>,
}

pub(super) struct ProbeExecutionFailure {
    pub(super) code: DiagnosticCode,
    pub(super) message_template: &'static str,
    pub(super) args: Vec<(&'static str, String)>,
    pub(super) notes: Vec<String>,
}

pub(super) fn execute_direct_cargo_probe(
    probe: &PendingRustBridgeProbe,
) -> Result<(), ProbeExecutionFailure> {
    if !probe.backend.cargo_manifest_path.is_file() {
        return Ok(());
    }
    let Some(backend_root) = probe.backend.cargo_manifest_path.parent() else {
        return Ok(());
    };
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
    fs::write(
        probe_root.join("Cargo.toml"),
        probe_cargo_toml(&probe.backend.dependency_name, backend_root),
    )
    .map_err(|error| probe_io_failure(format!("failed to write Rust probe manifest: {error}")))?;
    fs::write(probe_root.join("src/lib.rs"), probe_source(probe))
        .map_err(|error| probe_io_failure(format!("failed to write Rust probe source: {error}")))?;

    let output = Command::new("cargo")
        .args(["check", "--quiet"])
        .current_dir(&probe_root)
        .output()
        .map_err(|error| probe_io_failure(format!("failed to run Rust probe: {error}")))?;
    let _ = fs::remove_dir_all(&probe_root);
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let code = if stderr.contains("cannot find")
        || stderr.contains("failed to resolve")
        || stderr.contains("unresolved")
        || stderr.contains("not found")
    {
        DiagnosticCode::RUST_RESOLVE_TARGET_ROOT
    } else {
        DiagnosticCode::RUST_TYPE_PROBE_FAILURE
    };
    Err(ProbeExecutionFailure {
        code,
        message_template: "Rust bridge probe failed for `{target}`",
        args: vec![("target", canonical_sifr_target_path(&probe.declaration))],
        notes: vec![format!("rustc stderr: {}", stderr.trim())],
    })
}

fn probe_cargo_toml(dependency_name: &str, backend_root: &Path) -> String {
    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|crates_dir| crates_dir.join("sifr_runtime"))
        .unwrap_or_else(|| Path::new("crates/sifr_runtime").to_path_buf());
    format!(
        "[package]\nname = \"sifr-rust-probe\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[dependencies]\n{} = {{ path = {:?} }}\nsifr_runtime = {{ path = {:?} }}\n",
        dependency_name, backend_root, runtime_root
    )
}

fn unique_probe_nonce() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos())
}

fn probe_source(probe: &PendingRustBridgeProbe) -> String {
    let rust_path = probe.path.segments.join("::");
    match probe.declaration.declaration.kind {
        RustInteropDecoratorKind::Opaque => {
            format!("#![allow(dead_code)]\ntype __SifrProbe = {rust_path};\n")
        }
        RustInteropDecoratorKind::Function
        | RustInteropDecoratorKind::Async
        | RustInteropDecoratorKind::ZeroCopy
        | RustInteropDecoratorKind::View => {
            if let Some(signature) = &probe.signature {
                signature_probe_source(probe, signature, &rust_path)
            } else {
                format!("#![allow(dead_code)]\nfn __sifr_probe() {{ let _ = {rust_path}; }}\n")
            }
        }
    }
}

fn signature_probe_source(
    probe: &PendingRustBridgeProbe,
    signature: &RustBridgeSignatureContract,
    rust_path: &str,
) -> String {
    let params = signature
        .params
        .iter()
        .map(|param| {
            rust_param_type(param.convention, &param.ty)
                .unwrap_or_else(|| "__SifrUnsupportedBridgeType".to_string())
        })
        .collect::<Vec<_>>()
        .join(", ");
    let return_type = signature
        .return_type
        .rust_return_type
        .as_deref()
        .unwrap_or("__SifrUnsupportedBridgeType");
    let mut out = String::new();
    out.push_str("#![allow(dead_code)]\n");
    out.push_str(&generated_bridge_type_stubs(signature));
    if is_async_probe(probe) {
        out.push_str("fn __sifr_assert_async_signature<F, Fut>(_f: F)\nwhere\n    F: Fn(");
        out.push_str(&params);
        out.push_str(") -> Fut,\n    Fut: std::future::Future<Output = ");
        out.push_str(return_type);
        out.push_str(">,\n{}\nfn __sifr_probe() { __sifr_assert_async_signature(");
        out.push_str(rust_path);
        out.push_str("); }\n");
    } else {
        out.push_str("fn __sifr_assert_signature(_f: fn(");
        out.push_str(&params);
        out.push_str(") -> ");
        out.push_str(return_type);
        out.push_str(") {}\nfn __sifr_probe() { __sifr_assert_signature(");
        out.push_str(rust_path);
        out.push_str("); }\n");
    }
    out
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
        RustBridgeParamConvention::Own => ty.rust_owned_type.clone(),
    }
}

fn mutable_borrow_type(rust_type: &str) -> String {
    rust_type.strip_prefix('&').map_or_else(
        || rust_type.to_string(),
        |inner| format!("&mut {}", inner.trim_start()),
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

fn generated_bridge_type_stubs(signature: &RustBridgeSignatureContract) -> String {
    let module_name = signature
        .module_name
        .as_deref()
        .unwrap_or("__sifr_binary_entry");
    let mut names = Vec::new();
    for param in &signature.params {
        collect_generated_bridge_names(&param.ty, &mut names);
    }
    collect_generated_bridge_names(&signature.return_type, &mut names);
    names.sort();
    names.dedup();
    if names.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    out.push_str("mod __sifr_bridge {\n");
    out.push_str("pub mod ");
    out.push_str(module_name);
    out.push_str(" {\n");
    for name in names {
        out.push_str("#[derive(Clone, Debug, PartialEq, Eq)]\npub struct ");
        out.push_str(&name);
        out.push_str(";\n");
    }
    out.push_str("}\n}\n");
    out
}

fn collect_generated_bridge_names(
    ty: &sifr_codegen::RustBridgeTypeContract,
    names: &mut Vec<String>,
) {
    for candidate in [
        ty.rust_borrowed_type.as_deref(),
        ty.rust_owned_type.as_deref(),
        ty.rust_return_type.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        for segment in candidate.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_')) {
            if segment.ends_with("Bridge") && !segment.starts_with("__") {
                names.push(segment.to_string());
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

fn canonical_sifr_target_path(declaration: &RustInteropPlanDeclaration) -> String {
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
