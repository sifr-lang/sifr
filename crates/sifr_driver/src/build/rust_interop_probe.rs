use super::rust_interop_digest::fnv1a64_hex;
use super::rust_interop_probe_cache::{mark_probe_cache_hit, probe_cache_file, probe_cache_key};
use super::workspace::artifact_cache_root;
use sifr_codegen::{
    RustBridgeParamConvention, RustBridgeSignatureContract, RustInteropPlanDeclaration,
};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{RustInteropDecoratorKind, RustInteropValue, RustTargetPath};
use sifr_package::BackendCrateMetadata;
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::{env, fs};

static PROBE_NONCE_COUNTER: AtomicU64 = AtomicU64::new(0);
const RUST_BRIDGE_PROBE_TARGET_DIR: &str = "rust_bridge_probe_target";

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
    pub(super) sysroot_runtime_crate: PathBuf,
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
    let dependency_features =
        dependency_features(&probe.backend.dependency_name, backend_root, &probe.path);
    let probe_manifest = probe_cargo_toml(
        &probe.backend.dependency_name,
        backend_root,
        &probe.sysroot_runtime_crate,
        &dependency_features,
    );
    let probe_source = probe_source(probe);
    let invocation_cwd = env::current_dir()
        .map_err(|error| probe_io_failure(format!("failed to resolve Rust probe cwd: {error}")))?;
    let cache_key = probe_cache_key(probe, backend_root, &probe_manifest, &probe_source);
    let cache_file = probe_cache_file(&cache_key, &invocation_cwd);
    if cache_file.is_file() {
        return Ok(());
    }
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

    let mut command = Command::new("cargo");
    command
        .args(cargo_vendor_args(probe.sysroot_vendor_dir.as_deref()))
        .args(["check", "--quiet"])
        .current_dir(&probe_root);
    command.env("CARGO_TARGET_DIR", probe_cargo_target_dir(&invocation_cwd));
    let output = command
        .output()
        .map_err(|error| probe_io_failure(format!("failed to run Rust probe: {error}")))?;
    let _ = fs::remove_dir_all(&probe_root);
    if output.status.success() {
        mark_probe_cache_hit(&cache_file);
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
    sysroot_runtime_crate: &Path,
    dependency_features: &[String],
) -> String {
    let mut cargo_toml =
        "[package]\nname = \"sifr-rust-probe\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[dependencies]\n"
            .to_string();
    cargo_toml.push_str(&dependency_line(
        dependency_name,
        backend_root,
        dependency_features,
    ));
    if dependency_name != "sifr_runtime" {
        cargo_toml.push_str(&format!(
            "sifr_runtime = {{ path = {sysroot_runtime_crate:?} }}\n"
        ));
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

fn probe_cargo_target_dir(invocation_cwd: &Path) -> PathBuf {
    probe_cargo_target_dir_with_env(env::var_os("CARGO_TARGET_DIR"), invocation_cwd)
}

fn probe_cargo_target_dir_with_env(configured: Option<OsString>, invocation_cwd: &Path) -> PathBuf {
    configured.map_or_else(
        || artifact_cache_root().join(RUST_BRIDGE_PROBE_TARGET_DIR),
        |target_dir| normalize_cargo_target_dir(invocation_cwd, PathBuf::from(target_dir)),
    )
}

pub(super) fn normalize_cargo_target_dir(invocation_cwd: &Path, target_dir: PathBuf) -> PathBuf {
    if target_dir.is_absolute() {
        target_dir
    } else {
        invocation_cwd.join(target_dir)
    }
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
    let return_type = signature_return_probe_type(&signature.return_type);
    let mut out = String::new();
    out.push_str("#![allow(dead_code)]\n");
    out.push_str(&generated_bridge_type_stubs(signature));
    if is_async_probe(probe) {
        out.push_str("fn __sifr_assert_async_signature<F, Fut");
        if return_type.display_error_generic {
            out.push_str(", __SifrBridgeError");
        }
        out.push_str(">(_f: F)\nwhere\n    F: Fn(");
        out.push_str(&params);
        out.push_str(") -> Fut,\n    Fut: std::future::Future<Output = ");
        out.push_str(&return_type.ty);
        out.push('>');
        if async_future_requires_send(probe) {
            out.push_str(" + Send");
        }
        if return_type.display_error_generic {
            out.push_str(",\n    __SifrBridgeError: std::fmt::Display");
        }
        out.push_str(",\n{}\nfn __sifr_probe() { __sifr_assert_async_signature(");
        out.push_str(rust_path);
        out.push_str("); }\n");
    } else {
        out.push_str("fn __sifr_assert_signature");
        if return_type.display_error_generic {
            out.push_str("<__SifrBridgeError: std::fmt::Display>");
        }
        out.push_str("(_f: fn(");
        out.push_str(&params);
        out.push_str(") -> ");
        out.push_str(&return_type.ty);
        out.push_str(") {}\nfn __sifr_probe() { __sifr_assert_signature(");
        out.push_str(rust_path);
        out.push_str("); }\n");
    }
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
        artifact_cache_root, cargo_vendor_args, dependency_features, generated_bridge_type_stubs,
        normalize_cargo_target_dir, probe_cargo_target_dir_with_env, probe_cargo_toml,
        signature_return_probe_type, RUST_BRIDGE_PROBE_TARGET_DIR,
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
            Path::new("/opt/sifr/crates/sifr_runtime"),
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
            Path::new("/opt/sifr/crates/sifr_runtime"),
            &[],
        );

        assert_eq!(manifest.matches("sifr_runtime =").count(), 1);
    }

    #[test]
    fn sysroot_probe_manifest_enables_declared_stdlib_features() {
        let manifest = probe_cargo_toml(
            "sifr_stdlib",
            Path::new("/opt/sifr/crates/sifr_stdlib"),
            Path::new("/opt/sifr/crates/sifr_runtime"),
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
    fn generated_bridge_type_stubs_follow_sanitized_bridge_type_paths() {
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
                rust_return_type: Some("__sifr_bridge::_sifr_calendar::CalendarBridge".to_string()),
                kind: RustBridgeTypeKind::GeneratedRecord,
                unsupported_reason: None,
            },
            span: TextRange::default(),
        };
        let source = generated_bridge_type_stubs(&signature);

        assert!(source.contains("pub mod _sifr_calendar {\n"));
        assert!(source.contains("pub struct CalendarBridge;"));
        assert!(!source.contains("pub mod _sifr.calendar"));
    }

    #[test]
    fn result_error_bridge_return_probes_display_error_generic() {
        let return_type = RustBridgeTypeContract {
            sifr_type: "Result[str, ParseError]".to_string(),
            rust_borrowed_type: None,
            rust_owned_type: None,
            rust_return_type: Some(
                "Result<String, crate::__sifr_bridge::_sifr_crypto::ParseErrorBridge>".to_string(),
            ),
            kind: RustBridgeTypeKind::Result,
            unsupported_reason: None,
        };

        let probe_type = signature_return_probe_type(&return_type);

        assert_eq!(
            probe_type.ty,
            "Result<String, __SifrBridgeError>".to_string()
        );
        assert!(probe_type.display_error_generic);
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

    #[test]
    fn relative_probe_target_dir_is_anchored_to_invocation_cwd() {
        assert_eq!(
            normalize_cargo_target_dir(
                Path::new("/workspace/sifr"),
                Path::new("target/create-pr").to_path_buf()
            ),
            Path::new("/workspace/sifr/target/create-pr")
        );
    }

    #[test]
    fn absolute_probe_target_dir_is_preserved() {
        assert_eq!(
            normalize_cargo_target_dir(
                Path::new("/workspace/sifr"),
                Path::new("/tmp/sifr-target").to_path_buf()
            ),
            Path::new("/tmp/sifr-target")
        );
    }

    #[test]
    fn probe_target_dir_defaults_to_stable_artifact_cache_subdir() {
        assert_eq!(
            probe_cargo_target_dir_with_env(None, Path::new("/workspace/sifr")),
            artifact_cache_root().join(RUST_BRIDGE_PROBE_TARGET_DIR)
        );
    }

    #[test]
    fn probe_target_dir_honors_relative_env_override() {
        assert_eq!(
            probe_cargo_target_dir_with_env(
                Some(std::ffi::OsString::from("target/create-pr")),
                Path::new("/workspace/sifr")
            ),
            Path::new("/workspace/sifr/target/create-pr")
        );
    }
}
