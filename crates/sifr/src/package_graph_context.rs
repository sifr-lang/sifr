use crate::cargo_diagnostics::{bounded_excerpt, cargo_failure_diagnostic};
use crate::cli_model_and_entrypoint::{
    package_diagnostic, DiagnosticFormat, PackageGraphContext, EXIT_USAGE_OR_CONFIG,
    EXIT_USER_DIAGNOSTIC,
};
use crate::diagnostic_rendering_and_run::render_diagnostics;
use std::path::Path;

pub(super) fn load_package_graph_context(
    session: &sifr_package::PackageSession,
    lock_mode: sifr_package::CargoLockMode,
    diagnostic_format: DiagnosticFormat,
) -> Result<Option<PackageGraphContext>, i32> {
    if session.manifest_less_mode {
        return Ok(None);
    }
    load_package_graph_context_from_root_with_rust_hint(
        &session.workspace_root,
        lock_mode,
        diagnostic_format,
        false,
    )
    .map(Some)
}

pub(super) fn load_package_graph_context_for_entrypoint(
    session: &sifr_package::PackageSession,
    lock_mode: sifr_package::CargoLockMode,
    diagnostic_format: DiagnosticFormat,
) -> Result<Option<PackageGraphContext>, i32> {
    if session.manifest_less_mode {
        return Ok(None);
    }
    load_package_graph_context_from_root_with_rust_hint(
        &session.workspace_root,
        lock_mode,
        diagnostic_format,
        // Cargo metadata can fail before the source graph exists, so classify
        // from the already-parsed package manifest rather than scanning only
        // the selected entry file for decorators.
        session
            .manifest
            .as_ref()
            .is_some_and(sifr_package::SifrManifest::declares_rust_backend),
    )
    .map(Some)
}

pub(super) fn load_package_graph_context_from_root(
    workspace_root: &Path,
    lock_mode: sifr_package::CargoLockMode,
    diagnostic_format: DiagnosticFormat,
) -> Result<PackageGraphContext, i32> {
    load_package_graph_context_from_root_with_rust_hint(
        workspace_root,
        lock_mode,
        diagnostic_format,
        false,
    )
}

fn load_package_graph_context_from_root_with_rust_hint(
    workspace_root: &Path,
    lock_mode: sifr_package::CargoLockMode,
    diagnostic_format: DiagnosticFormat,
    rust_cargo_boundary: bool,
) -> Result<PackageGraphContext, i32> {
    let snapshot = match sifr_package::load_package_graph_snapshot(workspace_root, lock_mode) {
        Ok(snapshot) => snapshot,
        Err(failure) => {
            let exit = match &failure.kind {
                sifr_package::PackageGraphLoadFailureKind::Spawn { message } => {
                    let diagnostic =
                        cargo_failure_diagnostic(&failure.plan, lock_mode, None, message);
                    render_diagnostics(&[diagnostic], diagnostic_format);
                    EXIT_USAGE_OR_CONFIG
                }
                sifr_package::PackageGraphLoadFailureKind::Command {
                    exit_status,
                    output,
                } => {
                    let excerpt = bounded_excerpt(output);
                    let diagnostic = if rust_cargo_boundary
                        && sifr_package::cargo::lock_modes::cargo_lock_failure_reason(&excerpt)
                            .is_some()
                    {
                        crate::cli_lock_modes::rust_interop_cargo_failure_diagnostic(
                            workspace_root,
                            &failure.plan,
                            lock_mode,
                            *exit_status,
                            &excerpt,
                        )
                    } else {
                        cargo_failure_diagnostic(&failure.plan, lock_mode, *exit_status, &excerpt)
                    };
                    render_diagnostics(&[diagnostic], diagnostic_format);
                    EXIT_USER_DIAGNOSTIC
                }
                sifr_package::PackageGraphLoadFailureKind::Package {
                    diagnostics,
                    usage_error,
                } => {
                    let diagnostics = diagnostics
                        .iter()
                        .cloned()
                        .map(package_diagnostic)
                        .collect::<Vec<_>>();
                    render_diagnostics(&diagnostics, diagnostic_format);
                    if *usage_error {
                        EXIT_USAGE_OR_CONFIG
                    } else {
                        EXIT_USER_DIAGNOSTIC
                    }
                }
            };
            return Err(exit);
        }
    };
    Ok(PackageGraphContext {
        metadata: snapshot.metadata,
        graph: snapshot.graph,
        source_map: snapshot.source_map,
    })
}
