use super::render_diagnostics;
use crate::check_and_package_commands::{
    materialize_entrypoint_report, materialize_package_entrypoint_report,
};
use crate::cli_model_and_entrypoint::{
    DiagnosticFormat, EXIT_SUCCESS, EXIT_USAGE_OR_CONFIG, package_diagnostic,
    run_with_panic_boundary,
};
use crate::package_session_cli::package_session_for_cwd;
use sifr_driver::MaterializedRustProjectReport;
use sifr_frontend::DiskSourceProvider;
use std::io::{self, Write as _};
use std::path::Path;

pub(super) fn cmd_materialize_rust_project(
    file: &Path,
    output: &Path,
    lock_mode: sifr_package::CargoLockMode,
    quiet: bool,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let mut provider = DiskSourceProvider::new();
    match package_session_for_cwd(lock_mode, &mut provider) {
        Ok(session) if !session.manifest_less_mode => {
            match materialize_package_entrypoint_report(
                file,
                output,
                &session,
                lock_mode,
                diagnostic_format,
                &mut provider,
            ) {
                Ok(Some(report)) => {
                    return emit_materialized_project_result(&report, quiet, diagnostic_format);
                }
                Ok(None) => {}
                Err(exit_code) => return exit_code,
            }
        }
        Ok(_) => {}
        Err(error) => {
            render_diagnostics(&[package_diagnostic(error)], diagnostic_format);
            return EXIT_USAGE_OR_CONFIG;
        }
    }

    if lock_mode != sifr_package::CargoLockMode::Normal {
        return render_diagnostics(
            &[crate::cli_lock_modes::lock_mode_requires_package(
                "build", lock_mode,
            )],
            diagnostic_format,
        );
    }

    let result = match run_with_panic_boundary(
        "internal compiler panic during Rust project materialization",
        || materialize_entrypoint_report(file, output, &mut provider),
    ) {
        Ok(result) => result,
        Err(internal) => return render_diagnostics(&[*internal], diagnostic_format),
    };
    match result {
        Ok(report) => emit_materialized_project_result(&report, quiet, diagnostic_format),
        Err(errors) => render_diagnostics(&errors, diagnostic_format),
    }
}

fn emit_materialized_project_result(
    report: &MaterializedRustProjectReport,
    quiet: bool,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    if !report.frontend_diagnostics().is_empty() {
        let exit = render_diagnostics(report.frontend_diagnostics(), diagnostic_format);
        if exit != EXIT_SUCCESS {
            return exit;
        }
    }
    if !quiet && diagnostic_format == DiagnosticFormat::Human {
        let _ = writeln!(
            io::stderr(),
            "Materialized Rust project: {}",
            report.project_path().display()
        );
    }
    EXIT_SUCCESS
}
