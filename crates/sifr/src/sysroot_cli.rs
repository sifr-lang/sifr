use super::cli_model_and_entrypoint::{
    DiagnosticFormat, EXIT_SUCCESS, EXIT_USAGE_OR_CONFIG, diagnostic_with_code,
};
use super::diagnostic_rendering_and_run::render_diagnostics;
use clap::ValueEnum;
use sifr_diagnostics::DiagnosticCode;
use std::io::{self, Write as _};

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub(crate) enum PrintKind {
    Sysroot,
    CompilerIdentity,
    NativeContext,
}

pub(super) fn cmd_print(print: PrintKind, json: bool, diagnostic_format: DiagnosticFormat) -> i32 {
    match print {
        PrintKind::Sysroot => print_sysroot(json, diagnostic_format),
        PrintKind::CompilerIdentity => {
            let identity = crate::compiler_identity();
            if json {
                let _ = writeln!(
                    io::stdout(),
                    "{}",
                    serde_json::json!({"compiler_build_id": identity.as_str(), "identity_kind": "product"})
                );
            } else {
                let _ = writeln!(io::stdout(), "{}", identity.as_str());
            }
            EXIT_SUCCESS
        }
        PrintKind::NativeContext => print_native_context(json, diagnostic_format),
    }
}

pub(super) fn cmd_doctor(
    json: bool,
    verify_integrity: bool,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let native = std::env::current_dir()
        .map_err(|_| "cannot resolve invocation directory".to_owned())
        .and_then(|cwd| sifr_sysroot::NativeToolchain::resolve_at(&cwd));
    let native = match native {
        Ok(native) => native,
        Err(error) => {
            render_diagnostics(
                &[diagnostic_with_code(
                    format!(
                        "{error}; install the selected Rust toolchain with rustup toolchain install, or correct SIFR_CARGO/SIFR_RUSTC"
                    ),
                    DiagnosticCode::BUILD_RUSTC_OR_CARGO_FAILURE,
                )],
                diagnostic_format,
            );
            return EXIT_USAGE_OR_CONFIG;
        }
    };
    match sifr_sysroot::resolve_sysroot(None) {
        Ok(sysroot) => {
            let context = sifr_driver::CompilerContext::with_sysroot(
                crate::compiler_identity(),
                sysroot.clone(),
            );
            let metadata = context
                .inspect_metadata(verify_integrity)
                .and_then(|metadata| {
                    if verify_integrity {
                        sysroot.verify_integrity()?;
                    }
                    Ok(metadata)
                });
            let metadata = match metadata {
                Ok(metadata) => metadata,
                Err(error) => {
                    if json {
                        let _ = writeln!(
                            io::stdout(),
                            "{}",
                            serde_json::json!({
                                "schema_version":1,"status":"error","error_kind":"metadata", "message":error,
                                "compiler_build_id":crate::compiler_identity().as_str(),"root":sysroot.root
                            })
                        );
                    }
                    render_diagnostics(
                        &[diagnostic_with_code(
                            error,
                            DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE,
                        )],
                        diagnostic_format,
                    );
                    return EXIT_USAGE_OR_CONFIG;
                }
            };
            if json {
                let value = serde_json::json!({
                    "schema_version": 1,
                    "compiler_build_id": crate::compiler_identity().as_str(),
                    "cache_root": sifr_driver::cache_storage::root(),
                    "native_toolchain_id": native.identity(),
                    "rustc_version": native.rustc_version(),
                    "status": "ok",
                    "metadata":metadata,
                    "package_integrity_verified":verify_integrity,
                    "root": sysroot.root,
                    "toolchain_id": sysroot.toolchain_id(),
                    "sifr_version": sysroot.manifest.sifr_version,
                    "target_triple": sysroot.manifest.target_triple,
                    "checks": [
                        {"name": "manifest", "status": "ok", "path": sysroot.paths.manifest},
                        {"name": "stdlib_public_sources", "status": "ok", "path": sysroot.paths.stdlib_public_sources},
                        {"name": "stdlib_private_sources", "status": "ok", "path": sysroot.paths.stdlib_private_sources},
                        {"name": "runtime_crate", "status": "ok", "path": sysroot.paths.runtime_crate},
                        {"name": "stdlib_crate", "status": "ok", "path": sysroot.paths.stdlib_crate},
                        {"name": "cargo_lock", "status": "ok", "path": sysroot.paths.cargo_lock},
                        {"name": "vendor", "status": "ok", "path": sysroot.paths.vendor},
                    ],
                });
                let _ = writeln!(
                    io::stdout(),
                    "{}",
                    serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".to_string())
                );
            } else {
                let _ = writeln!(
                    io::stdout(),
                    "Sifr doctor: ok\nmetadata: {}",
                    metadata["metadata_id"]
                );
                let _ = writeln!(
                    io::stdout(),
                    "cache: {}",
                    sifr_driver::cache_storage::root().display()
                );
                let _ = writeln!(io::stdout(), "sysroot: {}", sysroot.root.display());
                let _ = writeln!(io::stdout(), "toolchain: {}", sysroot.toolchain_id());
                let _ = writeln!(io::stdout(), "target: {}", sysroot.manifest.target_triple);
                let _ = writeln!(
                    io::stdout(),
                    "runtime crate: {}",
                    sysroot.paths.runtime_crate.display()
                );
                let _ = writeln!(
                    io::stdout(),
                    "stdlib crate: {}",
                    sysroot.paths.stdlib_crate.display()
                );
                let _ = writeln!(io::stdout(), "vendor: {}", sysroot.paths.vendor.display());
            }
            EXIT_SUCCESS
        }
        Err(error) => {
            if json {
                let value = serde_json::json!({
                    "schema_version": 1,
                    "status": "error",
                    "error_kind": format!("{:?}", error.kind),
                    "message": error.message,
                    "binary_path": error.binary_path,
                    "attempted_sysroot": error.attempted_sysroot,
                    "asset_path": error.asset_path,
                    "help": "Select or reinstall a matching Sifr toolchain; source-tree contributors must prepare the sysroot with the documented sysroot build command.",
                });
                let _ = writeln!(
                    io::stdout(),
                    "{}",
                    serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".to_string())
                );
            }
            let diagnostic = diagnostic_with_code(
                format!(
                    "{}; select or reinstall a matching Sifr toolchain; source-tree contributors: run cargo build -p sifr in the complete source checkout",
                    error.boundary_message()
                ),
                DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
            );
            render_diagnostics(&[diagnostic], diagnostic_format);
            EXIT_USAGE_OR_CONFIG
        }
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

fn print_native_context(json: bool, diagnostic_format: DiagnosticFormat) -> i32 {
    let tools = std::env::current_dir()
        .map_err(|_| "cannot resolve invocation directory".to_owned())
        .and_then(|cwd| sifr_sysroot::NativeToolchain::resolve_at(&cwd));
    match tools {
        Ok(tools) => {
            if json {
                let _ = writeln!(
                    io::stdout(),
                    "{}",
                    serde_json::json!({
                        "compiler_build_id": crate::compiler_identity().as_str(),
                    "cache_root": sifr_driver::cache_storage::root(),
                        "native_toolchain_id": tools.identity(),
                    "cargo_path": tools.cargo_path(),
                    "rustc_path": tools.rustc_path(),
                        "cargo_version": tools.cargo_version(),
                        "rustc_version": tools.rustc_version(),
                        "host": tools.host(),
                        "target": tools.target()
                    })
                );
            } else {
                let _ = writeln!(
                    io::stdout(),
                    "native toolchain: {}\nhost: {}\n{}",
                    tools.identity(),
                    tools.host(),
                    tools.rustc_version()
                );
            }
            EXIT_SUCCESS
        }
        Err(error) => {
            render_diagnostics(
                &[diagnostic_with_code(
                    error,
                    DiagnosticCode::BUILD_RUSTC_OR_CARGO_FAILURE,
                )],
                diagnostic_format,
            );
            EXIT_USAGE_OR_CONFIG
        }
    }
}
