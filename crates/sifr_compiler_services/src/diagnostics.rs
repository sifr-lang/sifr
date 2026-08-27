use sifr_diagnostics::{DiagnosticArg, DiagnosticCode, RenderedDiagnostic};
use sifr_frontend::SourceOrigin;
use sifr_package::{PackageDiagnostic, PackageDiagnosticOrigin};
use std::any::Any;
use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedSourceMapFile {
    pub path: String,
    pub origin: SourceOrigin,
    pub source: String,
}

#[must_use]
pub(crate) fn diagnostic_with_code(
    message: impl Into<String>,
    code: DiagnosticCode,
) -> RenderedDiagnostic {
    let message = message.into();
    let mut args = BTreeMap::new();
    args.insert(
        "message".to_string(),
        DiagnosticArg::String(message.clone()),
    );
    RenderedDiagnostic {
        code: code.code().to_string(),
        severity: code.declared_severity(),
        message,
        message_template: "{message}".to_string(),
        args,
        url: code.docs_url(),
        spans: Vec::new(),
        children: Vec::new(),
        help: None,
        suggestions: Vec::new(),
    }
}

#[must_use]
pub fn render_package_diagnostic(diagnostic: PackageDiagnostic) -> RenderedDiagnostic {
    let PackageDiagnostic {
        code,
        message,
        origin,
        help,
    } = diagnostic;
    let mut rendered = diagnostic_with_code(message, code);
    rendered.help = help;
    add_package_origin_args(&mut rendered, &origin);
    rendered
}

fn add_package_origin_args(rendered: &mut RenderedDiagnostic, origin: &PackageDiagnosticOrigin) {
    match origin {
        PackageDiagnosticOrigin::CargoMetadata { cargo_package_id } => {
            insert_arg(rendered, "origin_kind", "cargo_metadata");
            if let Some(cargo_package_id) = cargo_package_id {
                insert_arg(rendered, "cargo_package_id", &cargo_package_id.0);
            }
        }
        PackageDiagnosticOrigin::CargoManifest {
            cargo_package_id,
            path,
            key,
        } => {
            insert_arg(rendered, "origin_kind", "cargo_manifest");
            insert_arg(rendered, "cargo_package_id", &cargo_package_id.0);
            insert_arg(rendered, "manifest_path", path.display().to_string());
            if let Some(key) = key {
                insert_arg(rendered, "manifest_key", key);
            }
        }
        PackageDiagnosticOrigin::SifrManifest {
            cargo_package_id,
            path,
            key,
        } => {
            insert_arg(rendered, "origin_kind", "sifr_manifest");
            insert_arg(rendered, "cargo_package_id", &cargo_package_id.0);
            insert_arg(rendered, "manifest_path", path.display().to_string());
            if let Some(key) = key {
                insert_arg(rendered, "manifest_key", key);
            }
        }
        PackageDiagnosticOrigin::RustMarker {
            cargo_package_id,
            path,
        } => {
            insert_arg(rendered, "origin_kind", "rust_marker");
            insert_arg(rendered, "cargo_package_id", &cargo_package_id.0);
            insert_arg(rendered, "marker_path", path.display().to_string());
        }
        PackageDiagnosticOrigin::PythonBridgeSource {
            cargo_package_id,
            path,
        } => {
            insert_arg(rendered, "origin_kind", "python_bridge_source");
            insert_arg(rendered, "cargo_package_id", &cargo_package_id.0);
            insert_arg(rendered, "bridge_path", path.display().to_string());
        }
        PackageDiagnosticOrigin::PackageGraph { cargo_package_id } => {
            insert_arg(rendered, "origin_kind", "package_graph");
            insert_arg(rendered, "cargo_package_id", &cargo_package_id.0);
        }
        PackageDiagnosticOrigin::CargoCommand { action } => {
            insert_arg(rendered, "origin_kind", "cargo_command");
            insert_arg(rendered, "cargo_action", action);
        }
    }
}

fn insert_arg(
    rendered: &mut RenderedDiagnostic,
    name: impl Into<String>,
    value: impl Into<String>,
) {
    rendered
        .args
        .insert(name.into(), DiagnosticArg::String(value.into()));
}

fn panic_payload_message(payload: &(dyn Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        return (*message).to_string();
    }
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    "non-string panic payload".to_string()
}

pub(crate) fn run_codegen_with_boundary<T>(
    context: impl Into<String>,
    operation: impl FnOnce() -> T,
) -> Result<T, Box<RenderedDiagnostic>> {
    let context = context.into();
    match catch_unwind(AssertUnwindSafe(operation)) {
        Ok(value) => Ok(value),
        Err(payload) => Err(Box::new(diagnostic_with_code(
            format!("{context}: {}", panic_payload_message(payload.as_ref())),
            DiagnosticCode::INTERNAL_COMPILER_PANIC,
        ))),
    }
}

pub(crate) fn render_codegen_error(error: &sifr_codegen::CodegenError) -> RenderedDiagnostic {
    diagnostic_with_code(
        format!("code generation failed: {error}"),
        DiagnosticCode::INTERNAL_CODEGEN_FAILURE,
    )
}
