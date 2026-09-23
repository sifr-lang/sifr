use super::cli_model_and_entrypoint::{
    DiagnosticFormat, EXIT_SUCCESS, EXIT_USAGE_OR_CONFIG, EXIT_USER_DIAGNOSTIC,
    diagnostic_with_code, package_diagnostic, run_with_panic_boundary,
};
use super::diagnostic_rendering_and_run::render_diagnostics;
use super::package_graph_context::load_package_graph_context_for_entrypoint;
use super::package_session_cli::package_session_for_cwd;
use sifr_diagnostics::DiagnosticCode;
use sifr_driver::{PackageEntrypoint, run_tests_with_package};
use sifr_frontend::DiskSourceProvider;
use std::fs;
use std::path::Path;

pub(super) fn cmd_test(
    dir: &Path,
    lock_mode: sifr_package::CargoLockMode,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let mut provider = DiskSourceProvider::new();
    let session = match package_session_for_cwd(lock_mode, &mut provider) {
        Ok(session) => session,
        Err(error) => return render_diagnostics(&[package_diagnostic(error)], diagnostic_format),
    };
    if session.manifest_less_mode && lock_mode != sifr_package::CargoLockMode::Normal {
        return render_diagnostics(
            &[super::cli_lock_modes::lock_mode_requires_package(
                "test", lock_mode,
            )],
            diagnostic_format,
        );
    }
    let package = if session.manifest_less_mode {
        None
    } else {
        let context = match load_package_graph_context_for_entrypoint(
            &session,
            lock_mode,
            diagnostic_format,
            &mut provider,
        ) {
            Ok(Some(context)) => context,
            Ok(None) => return EXIT_USAGE_OR_CONFIG,
            Err(exit) => return exit,
        };
        let canonical_dir = match fs::canonicalize(dir) {
            Ok(path) => path,
            Err(error) => {
                return render_diagnostics(
                    &[diagnostic_with_code(
                        format!("cannot resolve test directory '{}': {error}", dir.display()),
                        DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
                    )],
                    diagnostic_format,
                );
            }
        };
        let package_id = context
            .graph
            .packages
            .iter()
            .filter_map(|(id, package)| {
                let root = fs::canonicalize(&package.package_root).ok()?;
                canonical_dir.starts_with(&root).then_some((id, root))
            })
            .max_by_key(|(_, root)| root.components().count())
            .map(|(id, _)| id.clone());
        let Some(package_id) = package_id else {
            return render_diagnostics(
                &[diagnostic_with_code(
                    "sifr test directory must be inside one Sifr package",
                    DiagnosticCode::RUST_CARGO_METADATA,
                )],
                diagnostic_format,
            );
        };
        Some(PackageEntrypoint {
            main_file: canonical_dir,
            package_id,
            graph: context.graph,
            source_map: context.source_map,
            python_runtime: None,
            lock_mode,
        })
    };
    let run_result = match run_with_panic_boundary(
        "internal compiler panic during test command execution",
        || {
            run_tests_with_package(
                &crate::compiler_context(),
                dir,
                package.as_ref(),
                &mut provider,
            )
        },
    ) {
        Ok(result) => result,
        Err(internal) => return render_diagnostics(&[*internal], diagnostic_format),
    };
    match run_result {
        Ok(success) => {
            if success {
                EXIT_SUCCESS
            } else {
                EXIT_USER_DIAGNOSTIC
            }
        }
        Err(errors) => render_diagnostics(&errors, diagnostic_format),
    }
}
