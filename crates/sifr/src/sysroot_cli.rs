use super::cli_model_and_entrypoint::{
    diagnostic_with_code, DiagnosticFormat, EXIT_SUCCESS, EXIT_USAGE_OR_CONFIG,
};
use super::diagnostic_rendering_and_run::render_diagnostics;
use clap::ValueEnum;
use sifr_diagnostics::DiagnosticCode;
use std::io::{self, Write as _};

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub(crate) enum PrintKind {
    Sysroot,
}

pub(super) fn cmd_print(print: PrintKind, json: bool, diagnostic_format: DiagnosticFormat) -> i32 {
    match print {
        PrintKind::Sysroot => print_sysroot(json, diagnostic_format),
    }
}

fn print_sysroot(json: bool, diagnostic_format: DiagnosticFormat) -> i32 {
    match sifr_sysroot::resolve_sysroot(None) {
        Ok(sysroot) => {
            if json {
                let value = serde_json::json!({
                    "schema_version": 1,
                    "root": sysroot.root,
                    "toolchain_id": sysroot.toolchain_id(),
                    "sifr_version": sysroot.manifest.sifr_version,
                    "target_triple": sysroot.manifest.target_triple,
                    "built_by_compiler_commit": sysroot.manifest.built_by_compiler_commit,
                    "sysroot_content_sha256": sysroot.manifest.sysroot_content_sha256,
                    "cargo_lock_sha256": sysroot.manifest.cargo_lock_sha256,
                    "paths": {
                        "manifest": sysroot.paths.manifest,
                        "stdlib_root": sysroot.paths.stdlib_root,
                        "stdlib_public_sources": sysroot.paths.stdlib_public_sources,
                        "stdlib_private_sources": sysroot.paths.stdlib_private_sources,
                        "runtime_crate": sysroot.paths.runtime_crate,
                        "runtime_crate_manifest": sysroot.paths.runtime_crate_manifest,
                        "stdlib_crate": sysroot.paths.stdlib_crate,
                        "stdlib_crate_manifest": sysroot.paths.stdlib_crate_manifest,
                        "cargo_manifest": sysroot.paths.cargo_manifest,
                        "cargo_lock": sysroot.paths.cargo_lock,
                        "cargo_config": sysroot.paths.cargo_config,
                        "vendor": sysroot.paths.vendor,
                    },
                });
                let _ = writeln!(
                    io::stdout(),
                    "{}",
                    serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".to_string())
                );
            } else {
                let _ = writeln!(io::stdout(), "{}", sysroot.root.display());
            }
            EXIT_SUCCESS
        }
        Err(error) => {
            let diagnostic = diagnostic_with_code(
                error.boundary_message(),
                DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
            );
            render_diagnostics(&[diagnostic], diagnostic_format);
            EXIT_USAGE_OR_CONFIG
        }
    }
}
