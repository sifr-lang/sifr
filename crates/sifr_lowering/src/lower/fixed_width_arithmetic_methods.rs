use crate::hir_nodes::HirExpr;
use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;
use sifr_type_system::{FixedIntType, Type};

use super::{expression_diagnostics, LowerCtx};

pub(in crate::lower) fn resolve_fixed_width_method_type(
    fixed: FixedIntType,
    method: &str,
    args: &[HirExpr],
    arg_ranges: &[TextRange],
    method_range: TextRange,
    ctx: &mut LowerCtx,
) -> Option<Type> {
    let fixed_ty = Type::FixedInt(fixed);
    match method {
        "checked_add" | "wrapping_add" | "saturating_add" | "overflowing_add" | "checked_sub"
        | "wrapping_sub" | "saturating_sub" | "overflowing_sub" | "checked_mul"
        | "wrapping_mul" | "saturating_mul" | "overflowing_mul" => {
            if args.len() != 1 {
                reject_exact_arg_count(
                    ctx,
                    &format!("{}.{}", fixed.source_name(), method),
                    1,
                    args.len(),
                    arg_ranges,
                    method_range,
                );
                return None;
            }
            let arg_ty = args[0].ty();
            if arg_ty.resolve_alias() != fixed_ty.resolve_alias() {
                expression_diagnostics::type_mismatch(
                    ctx,
                    format!(
                        "{}.{}() argument must be '{}', got '{}'",
                        fixed.source_name(),
                        method,
                        fixed.source_name(),
                        arg_ty.display_name()
                    ),
                    arg_ranges.first().copied().unwrap_or(method_range),
                );
                return None;
            }
            match method {
                "checked_add" | "checked_sub" | "checked_mul" => Some(Type::Result(
                    Box::new(fixed_ty),
                    Box::new(overflow_error_type(ctx)),
                )),
                "overflowing_add" | "overflowing_sub" | "overflowing_mul" => {
                    Some(Type::Tuple(vec![fixed_ty, Type::Bool]))
                }
                _ => Some(fixed_ty),
            }
        }
        _ => {
            ctx.error_with_code_at(
                DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE,
                format!("type '{}' has no method '{method}'", fixed.source_name()),
                method_range,
            );
            None
        }
    }
}

fn overflow_error_type(ctx: &LowerCtx) -> Type {
    ctx.class_types
        .get("OverflowError")
        .cloned()
        .unwrap_or(Type::Class {
            name: "OverflowError".to_string(),
            fields: vec![("message".to_string(), Type::Str)],
            methods: vec![],
            parent_class: Some("Error".to_string()),
        })
}

fn reject_exact_arg_count(
    ctx: &mut LowerCtx,
    method: &str,
    expected: usize,
    actual: usize,
    arg_ranges: &[TextRange],
    method_range: TextRange,
) {
    let suffix = if expected == 1 { "" } else { "s" };
    let range = if actual > expected {
        arg_ranges.get(expected).copied().unwrap_or(method_range)
    } else {
        method_range
    };
    ctx.error_with_code_at(
        DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT,
        format!("{method}() takes exactly {expected} argument{suffix}, got {actual}"),
        range,
    );
}
