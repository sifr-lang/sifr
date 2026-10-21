use crate::diagnostics::diagnostic_with_code;
use sifr_diagnostics::{DiagnosticArg, RenderedDiagnostic};
use sifr_package::{PackageDiagnostic, PackageDiagnosticOrigin};

/// Converts package diagnostics into the canonical rendered diagnostic envelope.
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
