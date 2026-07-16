use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::Expr;
use sifr_type_system::{type_check_comparison, union_contains_none, Type};

pub(in crate::lower) fn validate_two_arg_min_max_operands(
    func_name: &str,
    left: &HirExpr,
    left_range: TextRange,
    right: &HirExpr,
    right_range: TextRange,
    ctx: &mut LowerCtx,
) -> bool {
    for (operand, range) in [(left, left_range), (right, right_range)] {
        if operand.ty().contains_affine_resource() {
            ctx.error_with_code_at(
                DiagnosticCode::PYZC_INVALID_DECLARATION,
                format!(
                    "{func_name}() cannot order affine Python buffer values because comparison would consume a non-orderable resource"
                ),
                range,
            );
            return false;
        }
        if matches!(operand.ty().resolve_alias(), Type::Any | Type::Unknown) {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISMATCH,
                format!(
                    "{func_name}() requires a statically known ordering capability, got '{}'",
                    operand.ty().display_name()
                ),
                range,
            );
            return false;
        }
        if matches!(operand.ty().resolve_alias(), Type::TypeVar(_)) {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISMATCH,
                format!(
                    "{func_name}() requires a concrete total-order capability; unconstrained generic operands are unsupported"
                ),
                range,
            );
            return false;
        }
    }
    if union_contains_none(left.ty()) || union_contains_none(right.ty()) {
        let range = if union_contains_none(left.ty()) {
            left_range
        } else {
            right_range
        };
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!(
                "{func_name}() with 2 arguments does not accept optional operands; got '{}' and '{}' (guard or unwrap first)",
                left.ty().display_name(),
                right.ty().display_name()
            ),
            range,
        );
        return false;
    }
    if type_check_comparison(left.ty(), "<", right.ty()).is_err() {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!(
                "{func_name}() arguments must be comparable and type-compatible; got '{}' and '{}'",
                left.ty().display_name(),
                right.ty().display_name()
            ),
            right_range,
        );
        return false;
    }
    true
}

pub(in crate::lower) fn validate_variadic_min_max_operands(
    func_name: &str,
    operands: &[HirExpr],
    operand_ranges: &[Expr],
    ctx: &mut LowerCtx,
) -> bool {
    if operands.len() < 2 {
        return true;
    }

    for index in 1..operands.len() {
        if !validate_two_arg_min_max_operands(
            func_name,
            &operands[index - 1],
            operand_ranges[index - 1].range(),
            &operands[index],
            operand_ranges[index].range(),
            ctx,
        ) {
            return false;
        }
    }

    true
}
