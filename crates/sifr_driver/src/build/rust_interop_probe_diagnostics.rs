use super::rust_interop_callback_probe::{
    signature_has_call_scoped_callback, stderr_reports_callback_escape,
};
use super::rust_interop_panic_probe::stderr_reports_invalid_panic_mapper;
use super::rust_interop_probe::{
    async_future_requires_send, canonical_sifr_target_path, PendingRustBridgeProbe,
    ProbeExecutionFailure,
};
use crate::diagnostics::RenderedDiagnostic;
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::RustInteropDecoratorKind;
use sifr_package::cargo::lock_modes::cargo_lock_failure_reason;

pub(super) fn classify_probe_failure(
    probe: &PendingRustBridgeProbe,
    stderr: &str,
) -> ProbeExecutionFailure {
    let (code, message_template, args, include_stderr) = if let Some(reason) =
        cargo_lock_failure_reason(stderr)
    {
        (
            DiagnosticCode::RUST_CARGO_METADATA,
            "Rust bridge Cargo resolution failed for `{reason}`",
            vec![("reason", reason.to_string())],
            true,
        )
    } else if stderr_reports_resolution_failure(stderr) {
        (
            DiagnosticCode::RUST_RESOLVE_TARGET_ROOT,
            "Rust bridge probe failed for `{target}`",
            vec![("target", canonical_sifr_target_path(&probe.declaration))],
            true,
        )
    } else if stderr_reports_invalid_panic_mapper(stderr) {
        (
                DiagnosticCode::RUST_PANIC_CONTRACT,
                "invalid Rust panic contract: {reason}",
                vec![(
                    "reason",
                    "panic error mapper must accept one `RustPanicErrorBridge` and return a Display error"
                        .to_string(),
                )],
                true,
            )
    } else if async_future_requires_send(probe) && stderr_reports_non_send_future(stderr) {
        (
                DiagnosticCode::RUST_ASYNC_CONTRACT,
                "invalid Rust async contract: {reason}",
                vec![(
                    "reason",
                    format!(
                        "future returned by `{}` must be Send or declare thread_affinity=tokio_current_thread",
                        canonical_sifr_target_path(&probe.declaration)
                    ),
                )],
                true,
            )
    } else if signature_has_call_scoped_callback(probe) && stderr_reports_callback_escape(stderr) {
        (
                DiagnosticCode::RUST_CALLBACK_CONTRACT,
                "invalid Rust callback contract for `{target}`: {reason}",
                vec![
                    ("target", canonical_sifr_target_path(&probe.declaration)),
                    (
                        "reason",
                        "call-scoped callbacks must match the generated bridge signature and cannot be stored, returned, or moved to another thread"
                            .to_string(),
                    ),
                ],
                true,
            )
    } else if probe.declaration.declaration.kind == RustInteropDecoratorKind::ZeroCopy
        && (probe.zero_copy_obligations.0 || probe.zero_copy_obligations.1)
        && stderr_reports_unsatisfied_view_obligation(stderr)
    {
        (
            DiagnosticCode::RUST_ZERO_COPY_CONTRACT,
            "invalid Rust zero-copy/view contract: {reason}",
            vec![(
                "reason",
                "the declared Rust view type does not satisfy its Send/Sync obligations"
                    .to_string(),
            )],
            false,
        )
    } else {
        (
            DiagnosticCode::RUST_TYPE_PROBE_FAILURE,
            "Rust bridge probe failed for `{target}`",
            vec![("target", canonical_sifr_target_path(&probe.declaration))],
            true,
        )
    };
    ProbeExecutionFailure {
        code,
        message_template,
        args,
        notes: if include_stderr {
            vec![format!("rustc stderr: {}", stderr.trim())]
        } else {
            Vec::new()
        },
    }
}

pub(super) fn probe_resolution_diagnostics(
    diagnostics: &[RenderedDiagnostic],
) -> ProbeExecutionFailure {
    let message = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>()
        .join("; ");
    probe_cargo_resolution_failure(message)
}

pub(super) fn probe_cargo_resolution_failure(message: String) -> ProbeExecutionFailure {
    ProbeExecutionFailure {
        code: DiagnosticCode::RUST_CARGO_METADATA,
        message_template: "Rust bridge Cargo resolution failed for `{reason}`",
        args: vec![("reason", message)],
        notes: vec![
            "prepare the package lockfile and dependency cache before retrying locked, offline, or frozen compilation"
                .to_string(),
        ],
    }
}

fn stderr_reports_unsatisfied_view_obligation(stderr: &str) -> bool {
    stderr.contains("cannot be sent between threads safely")
        || stderr.contains("cannot be shared between threads safely")
        || stderr.contains("the trait `Send` is not implemented")
        || stderr.contains("the trait `Sync` is not implemented")
}

pub(super) fn stderr_reports_non_send_future(stderr: &str) -> bool {
    stderr.contains("future cannot be sent")
        || (stderr.contains("future") && stderr.contains("cannot be sent between threads safely"))
        || stderr.contains("future is not `Send`")
}

pub(super) fn stderr_reports_resolution_failure(stderr: &str) -> bool {
    stderr.contains("cannot find")
        || stderr.contains("failed to resolve")
        || stderr.contains("unresolved")
        || stderr.contains("not found")
}
