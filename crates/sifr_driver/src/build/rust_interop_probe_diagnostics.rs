use super::rust_interop_callback_probe::{
    signature_has_call_scoped_callback, stderr_reports_callback_escape,
};
use super::rust_interop_panic_probe::stderr_reports_invalid_panic_mapper;
use super::rust_interop_probe::{
    async_future_requires_send, canonical_sifr_target_path, PendingRustBridgeProbe,
    ProbeExecutionFailure,
};
use sifr_diagnostics::DiagnosticCode;

pub(super) fn classify_probe_failure(
    probe: &PendingRustBridgeProbe,
    stderr: &str,
) -> ProbeExecutionFailure {
    let (code, message_template, args) = if stderr_reports_resolution_failure(stderr) {
        (
            DiagnosticCode::RUST_RESOLVE_TARGET_ROOT,
            "Rust bridge probe failed for `{target}`",
            vec![("target", canonical_sifr_target_path(&probe.declaration))],
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
        )
    } else {
        (
            DiagnosticCode::RUST_TYPE_PROBE_FAILURE,
            "Rust bridge probe failed for `{target}`",
            vec![("target", canonical_sifr_target_path(&probe.declaration))],
        )
    };
    ProbeExecutionFailure {
        code,
        message_template,
        args,
        notes: vec![format!("rustc stderr: {}", stderr.trim())],
    }
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
