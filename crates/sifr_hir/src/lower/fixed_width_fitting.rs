use super::LowerCtx;
use crate::HirExpr;
use num_bigint::BigInt;
use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;
use sifr_type_system::{FixedIntType, Type};

pub(super) fn validate_fixed_width_initializer(
    ctx: &mut LowerCtx,
    target: &Type,
    value: &HirExpr,
    range: TextRange,
) -> Option<bool> {
    let Type::FixedInt(fixed) = target.resolve_alias() else {
        return None;
    };
    let value = const_integer_value(value)?;
    let (min, max) = fixed_range(*fixed);
    if value >= min && value <= max {
        return Some(true);
    }

    ctx.error_with_code_at(
        DiagnosticCode::INT_FIXED_WIDTH_OUT_OF_RANGE,
        format!(
            "integer value {} does not fit target type {}; valid range is {}..={}",
            value,
            fixed.source_name(),
            min,
            max
        ),
        range,
    );
    Some(false)
}

pub(super) fn validate_annotated_constant_initializer(
    ctx: &mut LowerCtx,
    declared_type: &Type,
    value: &HirExpr,
    range: TextRange,
) {
    let fixed_width_fit = validate_fixed_width_initializer(ctx, declared_type, value, range);
    if fixed_width_fit.is_some() {
        return;
    }

    let value_ty = value.ty();
    let is_int_to_bigint = value_ty == &Type::Int && declared_type == &Type::BigInt;
    if !is_int_to_bigint && !value_ty.is_assignable_to(declared_type) {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!(
                "type mismatch: expected '{}', got '{}'",
                declared_type.display_name(),
                value_ty.display_name()
            ),
            range,
        );
    }
}

fn const_integer_value(value: &HirExpr) -> Option<BigInt> {
    match value {
        HirExpr::IntLiteral(value) => Some(BigInt::from(*value)),
        HirExpr::LargeIntLiteral(value) => value.parse().ok(),
        HirExpr::UnaryOp { op, operand, .. } if op == "+" => const_integer_value(operand),
        HirExpr::UnaryOp { op, operand, .. } if op == "-" => {
            const_integer_value(operand).map(|value| -value)
        }
        _ => None,
    }
}

fn fixed_range(fixed: FixedIntType) -> (BigInt, BigInt) {
    match fixed {
        FixedIntType::I8 => (BigInt::from(i8::MIN), BigInt::from(i8::MAX)),
        FixedIntType::I16 => (BigInt::from(i16::MIN), BigInt::from(i16::MAX)),
        FixedIntType::I32 => (BigInt::from(i32::MIN), BigInt::from(i32::MAX)),
        FixedIntType::I64 => (BigInt::from(i64::MIN), BigInt::from(i64::MAX)),
        FixedIntType::U8 => (BigInt::from(u8::MIN), BigInt::from(u8::MAX)),
        FixedIntType::U16 => (BigInt::from(u16::MIN), BigInt::from(u16::MAX)),
        FixedIntType::U32 => (BigInt::from(u32::MIN), BigInt::from(u32::MAX)),
        FixedIntType::U64 => (BigInt::from(u64::MIN), BigInt::from(u64::MAX)),
        FixedIntType::ISize => (BigInt::from(isize::MIN), BigInt::from(isize::MAX)),
        FixedIntType::USize => (BigInt::from(usize::MIN), BigInt::from(usize::MAX)),
    }
}
