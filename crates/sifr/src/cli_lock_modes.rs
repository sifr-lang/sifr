pub(super) const fn lock_mode_from_flags(
    locked: bool,
    offline: bool,
    frozen: bool,
) -> sifr_package::CargoLockMode {
    if frozen || (locked && offline) {
        sifr_package::CargoLockMode::Frozen
    } else if offline {
        sifr_package::CargoLockMode::Offline
    } else if locked {
        sifr_package::CargoLockMode::Locked
    } else {
        sifr_package::CargoLockMode::Normal
    }
}

pub(super) fn lock_mode_requires_package(
    command: &str,
    lock_mode: sifr_package::CargoLockMode,
) -> sifr_diagnostics::RenderedDiagnostic {
    crate::cli_model_and_entrypoint::diagnostic_with_code(
        format!(
            "sifr {command} --{} requires a package Cargo.lock",
            lock_mode.as_str()
        ),
        sifr_diagnostics::DiagnosticCode::RUST_CARGO_METADATA,
    )
}

pub(super) fn rust_interop_cargo_failure_diagnostic(
    workspace_root: &std::path::Path,
    plan: &sifr_package::CargoCommandPlan,
    lock_mode: sifr_package::CargoLockMode,
    exit_status: Option<i32>,
    excerpt: &str,
) -> sifr_diagnostics::RenderedDiagnostic {
    use sifr_diagnostics::{DiagnosticArg, DiagnosticCode};

    let mut diagnostic =
        crate::cargo_diagnostics::cargo_failure_diagnostic(plan, lock_mode, exit_status, excerpt);
    let cargo_reason = sifr_package::cargo::lock_modes::cargo_lock_failure_reason(excerpt);
    let reason = if cargo_reason == Some("stale lockfile or feature/source drift") {
        sifr_package::package_lock_drift_reason(workspace_root).or(cargo_reason)
    } else {
        cargo_reason
    }
    .unwrap_or("Cargo metadata is unavailable or inconsistent");
    diagnostic.code = DiagnosticCode::RUST_CARGO_METADATA.code().to_string();
    diagnostic.url = format!(
        "https://docs.sifr.sh/errors/{}",
        DiagnosticCode::RUST_CARGO_METADATA.code()
    );
    diagnostic.message = format!("Rust interop Cargo resolution failed: {reason}");
    diagnostic.message_template = "Rust interop Cargo resolution failed: {reason}".to_string();
    diagnostic.args.insert(
        "reason".to_string(),
        DiagnosticArg::String(reason.to_string()),
    );
    diagnostic
}
