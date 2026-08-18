use super::LowerCtx;
use ruff_text_size::TextRange;
use sifr_diagnostics::{DiagnosticArg, DiagnosticCode};
use std::collections::BTreeMap;

pub(super) fn fixed_receiver_violation(
    ctx: &mut LowerCtx,
    class_name: &str,
    method: &str,
    trait_name: &str,
    range: TextRange,
) {
    let mut args = BTreeMap::new();
    args.insert(
        "class_name".to_string(),
        DiagnosticArg::String(class_name.to_string()),
    );
    args.insert(
        "method".to_string(),
        DiagnosticArg::String(method.to_string()),
    );
    args.insert(
        "trait_name".to_string(),
        DiagnosticArg::String(trait_name.to_string()),
    );
    ctx.error_with_code_args_help_at(
        DiagnosticCode::PROTO_FIXED_RECEIVER_VIOLATION,
        format!(
            "class '{class_name}' method '{method}' violates the receiver convention fixed by Rust trait '{trait_name}'"
        ),
        args,
        None,
        range,
    );
}

pub(super) fn protocol_receiver_convention_mismatch(
    ctx: &mut LowerCtx,
    class_name: &str,
    method: &str,
    protocol: &str,
    range: TextRange,
) {
    let mut args = BTreeMap::new();
    args.insert(
        "class_name".to_string(),
        DiagnosticArg::String(class_name.to_string()),
    );
    args.insert(
        "method".to_string(),
        DiagnosticArg::String(method.to_string()),
    );
    args.insert(
        "protocol".to_string(),
        DiagnosticArg::String(protocol.to_string()),
    );
    ctx.error_with_code_args_help_at(
        DiagnosticCode::PROTO_RECEIVER_CONVENTION_MISMATCH,
        format!(
            "class '{class_name}' method '{method}' requires a mutable receiver but protocol '{protocol}' declares a shared receiver"
        ),
        args,
        None,
        range,
    );
}
