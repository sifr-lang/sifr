use super::rust_interop_digest::fnv1a64_hex;
use sifr_codegen::{
    RustBridgeParamConvention, RustBridgeSignatureContract, RustInteropPlanDeclaration,
};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{RustInteropDecoratorKind, RustInteropValue, RustTargetPath};
use sifr_package::BackendCrateMetadata;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static PROBE_NONCE_COUNTER: AtomicU64 = AtomicU64::new(0);

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
    pub(super) signature: Option<RustBridgeSignatureContract>,
    pub(super) async_thread_affinity: AsyncThreadAffinity,
    pub(super) sysroot_runtime_crate_manifest: Option<PathBuf>,
    pub(super) sysroot_vendor_dir: Option<PathBuf>,
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
        probe_cargo_toml(
            &probe.backend.dependency_name,
            backend_root,
            probe.sysroot_runtime_crate_manifest.as_deref(),
            &dependency_features(&probe.backend.dependency_name, backend_root, &probe.path),
        ),
    )
    .map_err(|error| probe_io_failure(format!("failed to write Rust probe manifest: {error}")))?;
    fs::write(probe_root.join("src/lib.rs"), probe_source(probe))
        .map_err(|error| probe_io_failure(format!("failed to write Rust probe source: {error}")))?;

    let output = Command::new("cargo")
        .args(cargo_vendor_args(probe.sysroot_vendor_dir.as_deref()))
        .args(["check", "--quiet"])
        .current_dir(&probe_root)
        .output()
        .map_err(|error| probe_io_failure(format!("failed to run Rust probe: {error}")))?;
    let _ = fs::remove_dir_all(&probe_root);
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let (code, message_template, args) = if stderr_reports_resolution_failure(&stderr) {
        (
            DiagnosticCode::RUST_RESOLVE_TARGET_ROOT,
            "Rust bridge probe failed for `{target}`",
            vec![("target", canonical_sifr_target_path(&probe.declaration))],
        )
    } else if async_future_requires_send(probe) && stderr_reports_non_send_future(&stderr) {
        (
            DiagnosticCode::RUST_ASYNC_CONTRACT,
            "invalid Rust async contract: {reason}",
            vec![(
                "reason",
                format!(
                    "future returned by `{}` must be Send or declare thread_affinity=tokio_current_thread",
                    canonical_sifr_target_path(&probe.declaration)
                ),
            )],
        )
    } else {
        (
            DiagnosticCode::RUST_TYPE_PROBE_FAILURE,
            "Rust bridge probe failed for `{target}`",
            vec![("target", canonical_sifr_target_path(&probe.declaration))],
        )
    };
    Err(ProbeExecutionFailure {
        code,
        message_template,
        args,
        notes: vec![format!("rustc stderr: {}", stderr.trim())],
    })
}

fn probe_cargo_toml(
    dependency_name: &str,
    backend_root: &Path,
    sysroot_runtime_crate_manifest: Option<&Path>,
    dependency_features: &[String],
) -> String {
    let runtime_root = sysroot_runtime_crate_manifest
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .map(|crates_dir| crates_dir.join("sifr_runtime"))
                .unwrap_or_else(|| Path::new("crates/sifr_runtime").to_path_buf())
        });
    let mut cargo_toml =
        "[package]\nname = \"sifr-rust-probe\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[dependencies]\n"
            .to_string();
    cargo_toml.push_str(&dependency_line(
        dependency_name,
        backend_root,
        dependency_features,
    ));
    if dependency_name != "sifr_runtime" {
        cargo_toml.push_str(&format!("sifr_runtime = {{ path = {runtime_root:?} }}\n"));
    }
    cargo_toml
}

fn dependency_line(dependency_name: &str, backend_root: &Path, features: &[String]) -> String {
    let default_features = if dependency_name == "sifr_stdlib" {
        ", default-features = false"
    } else {
        ""
    };
    if features.is_empty() {
        return format!("{dependency_name} = {{ path = {backend_root:?}{default_features} }}\n");
    }
    let features = features
        .iter()
        .map(|feature| toml_quote_string(feature))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "{dependency_name} = {{ path = {backend_root:?}{default_features}, features = [{features}] }}\n"
    )
}

fn dependency_features(
    dependency_name: &str,
    backend_root: &Path,
    path: &RustTargetPath,
) -> Vec<String> {
    if dependency_name != "sifr_stdlib" {
        return Vec::new();
    }
    let Some(feature) = path.segments.get(1) else {
        return Vec::new();
    };
    if !crate_feature_exists(backend_root, feature) {
        return Vec::new();
    }
    vec![feature.clone()]
}

/// Return whether `feature` is declared by the probed crate. This deliberately
/// treats undeclared path segments as no feature so sysroot-interop tests can use
/// minimal temp crates and future flat targets can still probe without a feature.
fn crate_feature_exists(backend_root: &Path, feature: &str) -> bool {
    let Ok(manifest) = fs::read_to_string(backend_root.join("Cargo.toml")) else {
        return false;
    };
    let Ok(value) = manifest.parse::<toml::Table>() else {
        return false;
    };
    value
        .get("features")
        .and_then(toml::Value::as_table)
        .is_some_and(|features| features.contains_key(feature))
}

fn cargo_vendor_args(vendor_dir: Option<&Path>) -> Vec<String> {
    let Some(vendor_dir) = vendor_dir else {
        return Vec::new();
    };
    vec![
        "--config".to_string(),
        "source.crates-io.replace-with=\"sifr-vendor\"".to_string(),
        "--config".to_string(),
        format!(
            "source.sifr-vendor.directory={}",
            toml_quote_string(&vendor_dir.display().to_string())
        ),
    ]
}

fn toml_quote_string(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');
    for ch in value.chars() {
        match ch {
            '\\' => quoted.push_str("\\\\"),
            '"' => quoted.push_str("\\\""),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            ch => quoted.push(ch),
        }
    }
    quoted.push('"');
    quoted
}

fn unique_probe_nonce() -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let counter = PROBE_NONCE_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{timestamp}_{counter}")
}

fn probe_source(probe: &PendingRustBridgeProbe) -> String {
    let rust_path = probe.path.segments.join("::");
    match probe.declaration.declaration.kind {
        RustInteropDecoratorKind::Opaque => opaque_probe_source(probe, &rust_path),
        RustInteropDecoratorKind::Callback => {
            unreachable!("callback metadata is targetless and never enters probe planning")
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

fn opaque_probe_source(probe: &PendingRustBridgeProbe, rust_path: &str) -> String {
    let mut out = format!("#![allow(dead_code)]\ntype __SifrProbe = {rust_path};\n");
    if opaque_bool_argument(probe, "send") {
        out.push_str("fn __sifr_assert_send<T: Send>() {}\n");
    }
    if opaque_bool_argument(probe, "sync") {
        out.push_str("fn __sifr_assert_sync<T: Sync>() {}\n");
    }
    if opaque_symbol_argument(probe, "clone") == Some("copy") {
        out.push_str("fn __sifr_assert_copy<T: Copy>() {}\n");
    }
    out.push_str("fn __sifr_probe() {\n");
    if opaque_bool_argument(probe, "send") {
        out.push_str("    __sifr_assert_send::<__SifrProbe>();\n");
    }
    if opaque_bool_argument(probe, "sync") {
        out.push_str("    __sifr_assert_sync::<__SifrProbe>();\n");
    }
    if opaque_symbol_argument(probe, "clone") == Some("copy") {
        out.push_str("    __sifr_assert_copy::<__SifrProbe>();\n");
    }
    out.push_str("}\n");
    out
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
        out.push('>');
        if async_future_requires_send(probe) {
            out.push_str(" + Send");
        }
        out.push_str(",\n{}\nfn __sifr_probe() { __sifr_assert_async_signature(");
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

fn async_future_requires_send(probe: &PendingRustBridgeProbe) -> bool {
    if !is_async_probe(probe) {
        return false;
    }
    probe.async_thread_affinity != AsyncThreadAffinity::TokioCurrentThread
}

fn stderr_reports_non_send_future(stderr: &str) -> bool {
    stderr.contains("future cannot be sent")
        || (stderr.contains("future") && stderr.contains("cannot be sent between threads safely"))
        || stderr.contains("future is not `Send`")
}

fn stderr_reports_resolution_failure(stderr: &str) -> bool {
    stderr.contains("cannot find")
        || stderr.contains("failed to resolve")
        || stderr.contains("unresolved")
        || stderr.contains("not found")
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
    for segment in module_name.split('.') {
        out.push_str("pub mod ");
        out.push_str(segment);
        out.push_str(" {\n");
    }
    for name in names {
        out.push_str("#[derive(Clone, Debug, PartialEq, Eq)]\npub struct ");
        out.push_str(&name);
        out.push_str(";\n");
    }
    for _ in module_name.split('.') {
        out.push_str("}\n");
    }
    out.push_str("}\n");
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

#[cfg(test)]
mod tests {
    use super::{
        cargo_vendor_args, dependency_features, generated_bridge_type_stubs, probe_cargo_toml,
    };
    use ruff_text_size::TextRange;
    use sifr_codegen::{
        RustBridgeSignatureContract, RustBridgeTypeContract, RustBridgeTypeKind, RustInteropOwner,
    };
    use sifr_ir::RustTargetPath;
    use std::path::Path;

    #[test]
    fn sysroot_probe_manifest_uses_sysroot_runtime_crate() {
        let manifest = probe_cargo_toml(
            "sifr_stdlib",
            Path::new("/opt/sifr/crates/sifr_stdlib"),
            Some(Path::new("/opt/sifr/crates/sifr_runtime/Cargo.toml")),
            &[],
        );

        assert!(manifest.contains(
            "sifr_stdlib = { path = \"/opt/sifr/crates/sifr_stdlib\", default-features = false }"
        ));
        assert!(manifest.contains("sifr_runtime = { path = \"/opt/sifr/crates/sifr_runtime\" }"));
    }

    #[test]
    fn sysroot_runtime_probe_manifest_does_not_duplicate_runtime_dependency() {
        let manifest = probe_cargo_toml(
            "sifr_runtime",
            Path::new("/opt/sifr/crates/sifr_runtime"),
            Some(Path::new("/opt/sifr/crates/sifr_runtime/Cargo.toml")),
            &[],
        );

        assert_eq!(manifest.matches("sifr_runtime =").count(), 1);
    }

    #[test]
    fn sysroot_probe_manifest_enables_declared_stdlib_features() {
        let manifest = probe_cargo_toml(
            "sifr_stdlib",
            Path::new("/opt/sifr/crates/sifr_stdlib"),
            Some(Path::new("/opt/sifr/crates/sifr_runtime/Cargo.toml")),
            &["platform".to_string()],
        );

        assert!(manifest.contains(
            "sifr_stdlib = { path = \"/opt/sifr/crates/sifr_stdlib\", default-features = false, features = [\"platform\"] }"
        ));
    }

    #[test]
    fn sysroot_stdlib_probe_features_follow_target_module_segment() {
        let stdlib_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("driver crate should have crates parent")
            .join("sifr_stdlib");
        let path = RustTargetPath {
            segments: ["sifr_stdlib", "platform", "platform_system"]
                .into_iter()
                .map(str::to_string)
                .collect(),
            span: TextRange::default(),
        };

        assert_eq!(
            dependency_features("sifr_stdlib", &stdlib_root, &path),
            vec!["platform".to_string()]
        );
    }

    #[test]
    fn sysroot_stdlib_probe_features_ignore_undeclared_target_segment() {
        let stdlib_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("driver crate should have crates parent")
            .join("sifr_stdlib");
        let path = RustTargetPath {
            segments: ["sifr_stdlib", "not_a_feature", "leaf"]
                .into_iter()
                .map(str::to_string)
                .collect(),
            span: TextRange::default(),
        };

        assert!(dependency_features("sifr_stdlib", &stdlib_root, &path).is_empty());
    }

    #[test]
    fn generated_bridge_type_stubs_split_dotted_module_names() {
        let signature = RustBridgeSignatureContract {
            canonical_target_path: "_sifr.calendar.calendar_monthrange".to_string(),
            module_name: Some("_sifr.calendar".to_string()),
            owner: RustInteropOwner::Function {
                name: "calendar_monthrange".to_string(),
            },
            params: Vec::new(),
            return_type: RustBridgeTypeContract {
                sifr_type: "list[int]".to_string(),
                rust_borrowed_type: None,
                rust_owned_type: None,
                rust_return_type: Some(
                    "__sifr_bridge::_sifr::calendar::CalendarBridge".to_string(),
                ),
                kind: RustBridgeTypeKind::GeneratedRecord,
                unsupported_reason: None,
            },
            span: TextRange::default(),
        };
        let source = generated_bridge_type_stubs(&signature);

        assert!(source.contains("pub mod _sifr {\npub mod calendar {\n"));
        assert!(source.contains("pub struct CalendarBridge;"));
        assert!(!source.contains("pub mod _sifr.calendar"));
    }

    #[test]
    fn sysroot_probe_vendor_args_use_invocation_scoped_config() {
        let args = cargo_vendor_args(Some(Path::new("/opt/sifr sysroot/vendor")));

        assert_eq!(
            args,
            vec![
                "--config",
                "source.crates-io.replace-with=\"sifr-vendor\"",
                "--config",
                "source.sifr-vendor.directory=\"/opt/sifr sysroot/vendor\"",
            ]
        );
    }
}
