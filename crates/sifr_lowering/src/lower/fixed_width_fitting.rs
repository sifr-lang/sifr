use super::LowerCtx;
use crate::HirExpr;
use num_bigint::BigInt;
use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;
use sifr_type_system::{FixedIntType, Type};

const MAX_EXACT_SHIFT_OR_EXPONENT: u32 = 13_610;

pub(in crate::lower) enum FixedWidthInitializerFit {
    NotConst,
    Fits(HirExpr),
    Rejected,
}

pub(in crate::lower) fn validate_fixed_width_initializer(
    ctx: &mut LowerCtx,
    target: &Type,
    value: &HirExpr,
    range: TextRange,
) -> FixedWidthInitializerFit {
    let Type::FixedInt(fixed) = target.resolve_alias() else {
        return FixedWidthInitializerFit::NotConst;
    };
    let value = match const_integer_value(ctx, value, range) {
        ConstIntegerValue::Value(value) => value,
        ConstIntegerValue::Unsupported => return FixedWidthInitializerFit::NotConst,
        ConstIntegerValue::Rejected => return FixedWidthInitializerFit::Rejected,
    };
    let (min, max) = fixed_range(*fixed);
    if value >= min && value <= max {
        return FixedWidthInitializerFit::Fits(bigint_to_hir_integer_literal(&value));
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
    FixedWidthInitializerFit::Rejected
}

pub(in crate::lower) fn validate_annotated_constant_initializer(
    ctx: &mut LowerCtx,
    declared_type: &Type,
    value: &HirExpr,
    range: TextRange,
) -> Option<HirExpr> {
    let fixed_width_fit = validate_fixed_width_initializer(ctx, declared_type, value, range);
    match fixed_width_fit {
        FixedWidthInitializerFit::Fits(value) => return Some(value),
        FixedWidthInitializerFit::Rejected => return None,
        FixedWidthInitializerFit::NotConst => {}
    }

    let value_ty = value.ty();
    if !value_ty.is_assignable_to(declared_type) {
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
    None
}

pub(in crate::lower) fn remember_module_const_integer(
    ctx: &mut LowerCtx,
    name: &str,
    value: &HirExpr,
    range: TextRange,
) -> Option<BigInt> {
    if let ConstIntegerValue::Value(value) = const_integer_value(ctx, value, range) {
        ctx.const_integer_values
            .insert(name.to_string(), value.clone());
        return Some(value);
    }
    None
}

enum ConstIntegerValue {
    Value(BigInt),
    Unsupported,
    Rejected,
}

fn const_integer_value(ctx: &mut LowerCtx, value: &HirExpr, range: TextRange) -> ConstIntegerValue {
    let evaluated = match value {
        HirExpr::IntLiteral(value) => BigInt::from(*value),
        HirExpr::LargeIntLiteral(value) => {
            if value.trim_start_matches('-').len()
                > super::integer_literal_diagnostics::INTEGER_EVAL_DECIMAL_DIGIT_BUDGET
            {
                return ConstIntegerValue::Rejected;
            }
            match value.parse() {
                Ok(value) => value,
                Err(_) => return ConstIntegerValue::Unsupported,
            }
        }
        HirExpr::Name { name, .. } => {
            if is_shadowed_by_inner_scope(ctx, name) {
                return ConstIntegerValue::Unsupported;
            }
            return ctx
                .const_integer_values
                .get(name)
                .cloned()
                .map_or(ConstIntegerValue::Unsupported, ConstIntegerValue::Value);
        }
        HirExpr::UnaryOp { op, operand, .. } if op == "+" => {
            return const_integer_value(ctx, operand, range);
        }
        HirExpr::UnaryOp { op, operand, .. } if op == "-" => {
            match const_integer_value(ctx, operand, range) {
                ConstIntegerValue::Value(value) => -value,
                other => return other,
            }
        }
        HirExpr::BinOp {
            left, op, right, ..
        } => {
            let left = match const_integer_value(ctx, left, range) {
                ConstIntegerValue::Value(value) => value,
                other => return other,
            };
            let right = match const_integer_value(ctx, right, range) {
                ConstIntegerValue::Value(value) => value,
                other => return other,
            };
            match evaluate_integer_binop(ctx, &left, op, &right, range) {
                ConstIntegerValue::Value(value) => value,
                other => return other,
            }
        }
        _ => return ConstIntegerValue::Unsupported,
    };
    reject_if_over_budget(ctx, evaluated, range)
}

fn evaluate_integer_binop(
    ctx: &mut LowerCtx,
    left: &BigInt,
    op: &str,
    right: &BigInt,
    range: TextRange,
) -> ConstIntegerValue {
    let zero = BigInt::ZERO;
    match op {
        "+" => ConstIntegerValue::Value(left + right),
        "-" => ConstIntegerValue::Value(left - right),
        "*" => ConstIntegerValue::Value(left * right),
        "//" => {
            if right == &zero {
                return ConstIntegerValue::Unsupported;
            }
            ConstIntegerValue::Value(python_floor_div(left, right))
        }
        "%" => {
            if right == &zero {
                return ConstIntegerValue::Unsupported;
            }
            ConstIntegerValue::Value(python_mod(left, right))
        }
        "<<" => evaluate_left_shift(ctx, left, right, range),
        ">>" => evaluate_right_shift(left, right),
        "**" => evaluate_pow(ctx, left, right, range),
        _ => ConstIntegerValue::Unsupported,
    }
}

fn python_floor_div(left: &BigInt, right: &BigInt) -> BigInt {
    let zero = BigInt::ZERO;
    let quotient = left / right;
    let remainder = left % right;
    if remainder != zero && ((left < &zero) != (right < &zero)) {
        quotient - 1
    } else {
        quotient
    }
}

fn python_mod(left: &BigInt, right: &BigInt) -> BigInt {
    left - (python_floor_div(left, right) * right)
}

fn evaluate_left_shift(
    ctx: &mut LowerCtx,
    left: &BigInt,
    right: &BigInt,
    range: TextRange,
) -> ConstIntegerValue {
    let Some(shift) = non_negative_u32(right) else {
        return ConstIntegerValue::Unsupported;
    };
    if shift > MAX_EXACT_SHIFT_OR_EXPONENT && left != &BigInt::ZERO {
        emit_budget_exceeded(ctx, approximate_left_shift_digits(left, shift), range);
        return ConstIntegerValue::Rejected;
    }
    ConstIntegerValue::Value(left << shift as usize)
}

fn evaluate_right_shift(left: &BigInt, right: &BigInt) -> ConstIntegerValue {
    let Some(shift) = non_negative_u32(right) else {
        return ConstIntegerValue::Unsupported;
    };
    ConstIntegerValue::Value(left >> shift as usize)
}

fn evaluate_pow(
    ctx: &mut LowerCtx,
    left: &BigInt,
    right: &BigInt,
    range: TextRange,
) -> ConstIntegerValue {
    let Some(exponent) = non_negative_u32(right) else {
        return ConstIntegerValue::Unsupported;
    };
    let abs_left = if left < &BigInt::ZERO {
        -left
    } else {
        left.clone()
    };
    if exponent > MAX_EXACT_SHIFT_OR_EXPONENT && abs_left > BigInt::ONE {
        emit_budget_exceeded(ctx, approximate_pow_digits(&abs_left, exponent), range);
        return ConstIntegerValue::Rejected;
    }
    ConstIntegerValue::Value(left.pow(exponent))
}

fn is_shadowed_by_inner_scope(ctx: &LowerCtx, name: &str) -> bool {
    let frame_count = ctx.scope.frame_count();
    frame_count > 1
        && ctx
            .scope
            .lookup_in_frame_range(name, 1, frame_count - 1)
            .is_some()
}

fn non_negative_u32(value: &BigInt) -> Option<u32> {
    if value < &BigInt::ZERO {
        return None;
    }
    u32::try_from(value.clone()).ok()
}

fn reject_if_over_budget(ctx: &mut LowerCtx, value: BigInt, range: TextRange) -> ConstIntegerValue {
    let digits = decimal_digit_count(&value);
    if digits > super::integer_literal_diagnostics::INTEGER_EVAL_DECIMAL_DIGIT_BUDGET {
        emit_budget_exceeded(ctx, digits, range);
        return ConstIntegerValue::Rejected;
    }
    ConstIntegerValue::Value(value)
}

fn emit_budget_exceeded(ctx: &mut LowerCtx, digits: usize, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::INT_EVAL_BUDGET_EXCEEDED,
        format!(
            "integer literal exceeds compile-time evaluation budget: {digits} decimal digits (max {})",
            super::integer_literal_diagnostics::INTEGER_EVAL_DECIMAL_DIGIT_BUDGET
        ),
        range,
    );
}

fn decimal_digit_count(value: &BigInt) -> usize {
    value.to_str_radix(10).trim_start_matches('-').len()
}

fn approximate_left_shift_digits(left: &BigInt, shift: u32) -> usize {
    let bit_digits = u64::from(shift).saturating_mul(30_103) / 100_000 + 1;
    let bit_digits = usize::try_from(bit_digits).unwrap_or(usize::MAX);
    decimal_digit_count(left).saturating_add(bit_digits)
}

fn approximate_pow_digits(abs_left: &BigInt, exponent: u32) -> usize {
    decimal_digit_count(abs_left).saturating_mul(usize::try_from(exponent).unwrap_or(usize::MAX))
}

fn bigint_to_hir_integer_literal(value: &BigInt) -> HirExpr {
    if let Ok(value) = i64::try_from(value.clone()) {
        HirExpr::IntLiteral(value)
    } else {
        HirExpr::LargeIntLiteral(value.to_str_radix(10))
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
