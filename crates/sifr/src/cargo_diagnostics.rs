use crate::cli_model_and_entrypoint::package_diagnostic;
use sifr_diagnostics::{DiagnosticArg, RenderedDiagnostic};

pub(super) fn bounded_excerpt(text: &str) -> String {
    const MAX_LINES: usize = 12;
    const MAX_BYTES: usize = 4096;
    let mut excerpt = text.lines().take(MAX_LINES).collect::<Vec<_>>().join("\n");
    if excerpt.len() > MAX_BYTES {
        excerpt.truncate(MAX_BYTES);
    }
    excerpt
}

pub(super) fn cargo_failure_diagnostic(
    plan: &sifr_package::CargoCommandPlan,
    lock_mode: sifr_package::CargoLockMode,
    exit_status: Option<i32>,
    excerpt: &str,
) -> RenderedDiagnostic {
    let stderr_redacted = sifr_package::cargo::errors::redact_cargo_stderr(excerpt);
    let package = sifr_package::map_cargo_failure(plan.action, &stderr_redacted);
    let mut diagnostic = package_diagnostic(package);
    diagnostic.args.insert(
        "action".to_string(),
        DiagnosticArg::String(plan.action.as_str().to_string()),
    );
    diagnostic.args.insert(
        "current_dir".to_string(),
        DiagnosticArg::String(plan.current_dir.display().to_string()),
    );
    diagnostic.args.insert(
        "args_redacted".to_string(),
        DiagnosticArg::String(redacted_args(&plan.args).join(" ")),
    );
    diagnostic.args.insert(
        "lock_mode".to_string(),
        DiagnosticArg::String(lock_mode.as_str().to_string()),
    );
    diagnostic.args.insert(
        "network_mode".to_string(),
        DiagnosticArg::String(if lock_mode.is_network_disallowed() {
            "offline".to_string()
        } else {
            "online".to_string()
        }),
    );
    diagnostic.args.insert(
        "stderr_redacted".to_string(),
        DiagnosticArg::String(stderr_redacted),
    );
    if let Some(status) = exit_status {
        diagnostic.args.insert(
            "exit_status".to_string(),
            DiagnosticArg::String(status.to_string()),
        );
    }
    diagnostic
}

fn redacted_args(args: &[String]) -> Vec<String> {
    args.iter()
        .map(|arg| sifr_package::cargo::errors::redact_cargo_stderr(arg))
        .collect()
}
