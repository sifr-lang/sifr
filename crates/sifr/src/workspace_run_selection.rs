use super::check_and_package_commands::load_package_graph_context_from_root;
use super::cli_model_and_entrypoint::{
    diagnostic_with_code, package_diagnostic, DiagnosticFormat, PackageGraphContext,
    EXIT_USER_DIAGNOSTIC,
};
use super::diagnostic_rendering_and_run::{render_diagnostics, render_package_diagnostics};
use sifr_diagnostics::DiagnosticCode;
use std::path::Path;

pub(super) fn resolve_run_session(
    session: sifr_package::PackageSession,
    target: Option<&str>,
    packages: &[String],
    lock_mode: sifr_package::CargoLockMode,
    diagnostic_format: DiagnosticFormat,
) -> Result<sifr_package::PackageSession, i32> {
    if packages.is_empty() && (!session.manifest_less_mode || run_target_is_explicit_path(target)) {
        return Ok(session);
    }
    let context = load_package_graph_context_from_root(
        &session.workspace_root,
        lock_mode,
        diagnostic_format,
    )?;
    if !packages.is_empty() {
        return selected_run_session(&context, packages, lock_mode, diagnostic_format);
    }
    default_workspace_run_session(&context, lock_mode, diagnostic_format)
}

fn selected_run_session(
    context: &PackageGraphContext,
    packages: &[String],
    lock_mode: sifr_package::CargoLockMode,
    diagnostic_format: DiagnosticFormat,
) -> Result<sifr_package::PackageSession, i32> {
    if packages.len() != 1 {
        let diagnostic = diagnostic_with_code(
            "sifr run accepts exactly one -p/--package selector",
            DiagnosticCode::PACKAGE_SELECTOR_AMBIGUOUS,
        );
        render_diagnostics(&[diagnostic], diagnostic_format);
        return Err(EXIT_USER_DIAGNOSTIC);
    }
    let selection =
        sifr_package::explicit_package_selection(&context.metadata, &context.graph, packages)
            .map_err(|diagnostics| render_package_diagnostics(diagnostics, diagnostic_format))?;
    let Some(package_id) = selection.selected_sifr_packages.iter().next() else {
        let diagnostic = sifr_package::PackageDiagnostic::selector_ambiguous(&packages[0], &[]);
        render_diagnostics(&[package_diagnostic(diagnostic)], diagnostic_format);
        return Err(EXIT_USER_DIAGNOSTIC);
    };
    let Some(package) = context.graph.packages.get(package_id) else {
        let diagnostic = sifr_package::PackageDiagnostic::selector_ambiguous(&packages[0], &[]);
        render_diagnostics(&[package_diagnostic(diagnostic)], diagnostic_format);
        return Err(EXIT_USER_DIAGNOSTIC);
    };
    Ok(sifr_package::PackageSession::from_package_metadata(
        context.metadata.workspace_root.clone(),
        package,
        lock_mode,
    ))
}

fn default_workspace_run_session(
    context: &PackageGraphContext,
    lock_mode: sifr_package::CargoLockMode,
    diagnostic_format: DiagnosticFormat,
) -> Result<sifr_package::PackageSession, i32> {
    let default_members = if context.metadata.workspace_default_members.is_empty() {
        &context.metadata.workspace_members
    } else {
        &context.metadata.workspace_default_members
    };
    let mut candidates = Vec::new();
    for cargo_package_id in default_members {
        let Some(classification) = context.graph.classifications.get(cargo_package_id) else {
            continue;
        };
        let package_id = match classification {
            sifr_package::PackageClassification::SifrSource(package_id)
            | sifr_package::PackageClassification::RustBackedSifr(package_id) => package_id,
            sifr_package::PackageClassification::BackendRust => continue,
        };
        let Some(package) = context.graph.packages.get(package_id) else {
            continue;
        };
        let package_session = sifr_package::PackageSession::from_package_metadata(
            context.metadata.workspace_root.clone(),
            package,
            lock_mode,
        );
        match package_session.has_default_runnable_app() {
            Ok(true) => candidates.push((package.cargo_package_name.clone(), package_session)),
            Ok(false) => {}
            Err(error) => {
                render_diagnostics(&[package_diagnostic(error)], diagnostic_format);
                return Err(EXIT_USER_DIAGNOSTIC);
            }
        }
    }
    if candidates.len() == 1 {
        return Ok(candidates.remove(0).1);
    }
    let names = candidates
        .into_iter()
        .map(|(name, _)| name)
        .collect::<Vec<_>>();
    let diagnostic = sifr_package::PackageDiagnostic::workspace_run_ambiguous(&names);
    render_diagnostics(&[package_diagnostic(diagnostic)], diagnostic_format);
    Err(EXIT_USER_DIAGNOSTIC)
}

fn run_target_is_explicit_path(target: Option<&str>) -> bool {
    target.is_some_and(|value| {
        Path::new(value)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("sifr"))
            || value.contains('/')
            || value.contains(std::path::MAIN_SEPARATOR)
    })
}
