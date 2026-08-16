use ruff_text_size::TextRange;
use sifr_diagnostics::{DiagnosticArg, DiagnosticCode};
use sifr_lowering::{HirDiagnostic, StaticProgramValue};
use std::collections::BTreeMap;

pub(crate) fn static_program_value(value: &crate::ConstValue) -> StaticProgramValue {
    match value {
        crate::ConstValue::None => StaticProgramValue::None,
        crate::ConstValue::Bool(value) => StaticProgramValue::Bool(*value),
        crate::ConstValue::Integer(value) => StaticProgramValue::Integer(value.to_string()),
        crate::ConstValue::FloatBits(value) => StaticProgramValue::FloatBits(*value),
        crate::ConstValue::String(value) => StaticProgramValue::String(value.clone()),
        crate::ConstValue::Bytes(value) => StaticProgramValue::Bytes(value.clone()),
        crate::ConstValue::Tuple(values) => {
            StaticProgramValue::Tuple(values.iter().map(static_program_value).collect())
        }
        crate::ConstValue::List(values) => {
            StaticProgramValue::List(values.iter().map(static_program_value).collect())
        }
        crate::ConstValue::Record(values) => StaticProgramValue::Record(
            values
                .iter()
                .map(|(name, value)| (name.clone(), static_program_value(value)))
                .collect(),
        ),
    }
}

pub(crate) fn malformed(
    package: &str,
    reason_code: &str,
    problem: impl Into<String>,
    range: TextRange,
) -> HirDiagnostic {
    let problem = problem.into();
    HirDiagnostic {
        code: Some(DiagnosticCode::META_MALFORMED_DECLARATION),
        message: format!(
            "package {package} declared malformed specialization issue {reason_code}: {problem}"
        ),
        args: BTreeMap::from([
            (
                "package".to_string(),
                DiagnosticArg::String(package.to_string()),
            ),
            (
                "reason_code".to_string(),
                DiagnosticArg::String(reason_code.to_string()),
            ),
            (
                "declaration_problem".to_string(),
                DiagnosticArg::String(problem),
            ),
        ]),
        help: None,
        primary_range: Some(range),
        line: None,
        col: None,
    }
}

pub(crate) fn method_slot_diagnostic(
    error: crate::slot_table::MethodSlotError,
    range: TextRange,
) -> HirDiagnostic {
    let code = match error.kind() {
        crate::slot_table::MethodSlotErrorKind::List => DiagnosticCode::RUST_SLOT_LIST,
        crate::slot_table::MethodSlotErrorKind::Method => DiagnosticCode::RUST_SLOT_METHOD,
        crate::slot_table::MethodSlotErrorKind::Signature => DiagnosticCode::RUST_SLOT_SIGNATURE,
        crate::slot_table::MethodSlotErrorKind::Context => DiagnosticCode::RUST_SLOT_CONTEXT,
    };
    let reason = error.into_reason();
    HirDiagnostic {
        code: Some(code),
        message: match code.code() {
            "SIFR-RUST-SLOT-0001" => format!("invalid reserved method-slot list: {reason}"),
            "SIFR-RUST-SLOT-0002" => format!("invalid method-slot target: {reason}"),
            "SIFR-RUST-SLOT-0005" => format!("invalid method-slot context: {reason}"),
            _ => format!("invalid method-slot signature: {reason}"),
        },
        args: BTreeMap::from([("reason".to_string(), DiagnosticArg::String(reason))]),
        help: None,
        primary_range: Some(range),
        line: None,
        col: None,
    }
}
