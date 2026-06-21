use super::rust_interop_digest::fnv1a64_hex;
use sifr_codegen::RustInteropPlanDeclaration;
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
        "sifr_rust_probe_{}_{}",
        std::process::id(),
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
    format!(
        "[package]\nname = \"sifr-rust-probe\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[dependencies]\n{} = {{ path = {:?} }}\n",
        dependency_name, backend_root
    )
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
            format!("#![allow(dead_code)]\nfn __sifr_probe() {{ let _ = {rust_path}; }}\n")
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
