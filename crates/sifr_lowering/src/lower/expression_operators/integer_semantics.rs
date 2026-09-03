use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use bigdecimal::num_traits::{Signed, ToPrimitive, Zero};
use num_bigint::{BigInt, Sign};
use sifr_type_system::{Type, make_union};

const DEFAULT_MAX_INTEGER_OUTPUT_BITS: u64 = 1_000_000;

pub(in crate::lower) fn checked_decimal_arithmetic_result_type(
    left: &HirExpr,
    op: &str,
    right: &HirExpr,
    ctx: &LowerCtx,
) -> Option<Type> {
    if !matches!(op, "+" | "-" | "*" | "/" | "//" | "%" | "**") {
        return None;
    }
    let left_ty = left.ty().resolve_alias();
    let right_ty = right.ty().resolve_alias();
    let integral = |ty: &Type| matches!(ty, Type::Int | Type::LiteralInt(_) | Type::FixedInt(_));
    let decimal_operands = (matches!(left_ty, Type::Decimal)
        && (matches!(right_ty, Type::Decimal) || integral(right_ty)))
        || (op != "**" && matches!(right_ty, Type::Decimal) && integral(left_ty));
    if decimal_operands {
        let conversion = super::super::decimal_methods::decimal_conversion_error_type(ctx);
        let error = if matches!(op, "/" | "//" | "%") {
            make_union(vec![division_error_type(ctx), conversion])
        } else {
            conversion
        };
        return Some(Type::Result(Box::new(Type::Decimal), Box::new(error)));
    }

    let bigdecimal_division = matches!(left_ty, Type::BigDecimal)
        && (matches!(right_ty, Type::BigDecimal) || integral(right_ty))
        && matches!(op, "/" | "//" | "%");
    if bigdecimal_division {
        return Some(Type::Result(
            Box::new(Type::BigDecimal),
            Box::new(division_error_type(ctx)),
        ));
    }

    let bigdecimal_power = matches!(left_ty, Type::BigDecimal) && integral(right_ty) && op == "**";
    bigdecimal_power.then(|| {
        Type::Result(
            Box::new(Type::BigDecimal),
            Box::new(make_union(vec![
                division_error_type(ctx),
                super::super::decimal_methods::decimal_conversion_error_type(ctx),
            ])),
        )
    })
}

pub(in crate::lower) fn exact_int_floor_result_type(
    left: &HirExpr,
    op: &str,
    right: &HirExpr,
    ctx: &LowerCtx,
) -> Option<Type> {
    if !matches!(op, "//" | "%") {
        return None;
    }
    if !is_exact_or_fixed_int_like(left.ty()) || !is_exact_or_fixed_int_like(right.ty()) {
        return None;
    }
    if is_proven_nonzero_integer_expr(right, ctx) {
        return None;
    }
    Some(Type::Result(
        Box::new(Type::Int),
        Box::new(division_error_type(ctx)),
    ))
}

pub(in crate::lower) fn bounded_integer_arithmetic_result_type(
    left: &HirExpr,
    op: &str,
    right: &HirExpr,
    ctx: &LowerCtx,
) -> Option<Type> {
    if !matches!(op, "**" | "<<" | ">>") {
        return None;
    }
    if !is_exact_or_fixed_int_like(left.ty()) || !is_exact_or_fixed_int_like(right.ty()) {
        return None;
    }
    if statically_safe_bounded_integer_arithmetic(left, op, right, ctx) {
        return Some(Type::Int);
    }
    Some(Type::Result(
        Box::new(Type::Int),
        Box::new(make_union(vec![
            builtin_error_type(ctx, "ValueError", "Error", vec![]),
            builtin_error_type(
                ctx,
                "ArithmeticLimitError",
                "OverflowError",
                vec![("limit".to_string(), Type::Int)],
            ),
        ])),
    ))
}

fn statically_safe_bounded_integer_arithmetic(
    left: &HirExpr,
    op: &str,
    right: &HirExpr,
    ctx: &LowerCtx,
) -> bool {
    let Some(exponent) = literal_exact_integer_value(right) else {
        return false;
    };
    if exponent.sign() == Sign::Minus {
        return false;
    }
    if op == ">>" {
        return exponent
            .to_u64()
            .is_some_and(|shift| shift <= DEFAULT_MAX_INTEGER_OUTPUT_BITS);
    }
    let Some(value) = proven_exact_integer_value(left, ctx) else {
        return exponent == BigInt::ZERO;
    };
    if op == "**" {
        let Some(exponent) = exponent.to_u32().map(u64::from) else {
            return false;
        };
        if exponent == 0
            || value == BigInt::ZERO
            || value == BigInt::from(1_i8)
            || value == BigInt::from(-1_i8)
        {
            return true;
        }
        return value
            .bits()
            .checked_mul(exponent)
            .is_some_and(|bits| bits <= DEFAULT_MAX_INTEGER_OUTPUT_BITS);
    }
    let Some(shift) = exponent.to_u64() else {
        return false;
    };
    value == BigInt::ZERO
        || value
            .bits()
            .checked_add(shift)
            .is_some_and(|bits| bits <= DEFAULT_MAX_INTEGER_OUTPUT_BITS)
}

pub(in crate::lower) fn statically_safe_bounded_integer_augassign(
    op: &str,
    right: &HirExpr,
) -> bool {
    let Some(operand) = literal_exact_integer_value(right) else {
        return false;
    };
    if operand.sign() == Sign::Minus {
        return false;
    }
    match op {
        "**" | "<<" => operand == BigInt::ZERO,
        ">>" => true,
        _ => false,
    }
}

fn literal_exact_integer_value(expr: &HirExpr) -> Option<BigInt> {
    match expr {
        HirExpr::IntLiteral(value) => Some(BigInt::from(*value)),
        HirExpr::LargeIntLiteral(value) => value.parse::<BigInt>().ok(),
        HirExpr::UnaryOp { op, operand, .. } if op == "+" => literal_exact_integer_value(operand),
        HirExpr::UnaryOp { op, operand, .. } if op == "-" => {
            literal_exact_integer_value(operand).map(|value| -value)
        }
        _ => None,
    }
}

pub(in crate::lower) fn builtin_error_type(
    ctx: &LowerCtx,
    name: &str,
    parent: &str,
    extra_fields: Vec<(String, Type)>,
) -> Type {
    ctx.class_types.get(name).cloned().unwrap_or_else(|| {
        let mut fields = vec![("message".to_string(), Type::Str)];
        fields.extend(extra_fields);
        Type::Class {
            identity: None,
            type_args: Vec::new(),
            name: name.to_string(),
            fields,
            methods: vec![],
            parent_class: Some(parent.to_string()),
        }
    })
}

fn division_error_type(ctx: &LowerCtx) -> Type {
    ctx.class_types
        .get("DivisionError")
        .cloned()
        .unwrap_or(Type::Class {
            identity: None,
            type_args: Vec::new(),
            name: "DivisionError".to_string(),
            fields: vec![("message".to_string(), Type::Str)],
            methods: vec![],
            parent_class: Some("Error".to_string()),
        })
}

pub(in crate::lower) fn exact_int_true_division_result_type(
    left: &HirExpr,
    op: &str,
    right: &HirExpr,
    ctx: &LowerCtx,
) -> Option<Type> {
    if op != "/" {
        return None;
    }
    if !is_exact_or_fixed_int_like(left.ty()) || !is_exact_or_fixed_int_like(right.ty()) {
        return None;
    }
    if exact_integer_ratio_is_proven_float_representable(left, right, ctx) {
        return Some(Type::Float);
    }
    Some(Type::Result(
        Box::new(Type::Float),
        Box::new(make_union(vec![
            builtin_error_type(ctx, "DivisionError", "Error", vec![]),
            builtin_error_type(ctx, "FloatOverflowError", "OverflowError", vec![]),
            builtin_error_type(ctx, "FloatPrecisionLossError", "OverflowError", vec![]),
        ])),
    ))
}

fn exact_integer_ratio_is_proven_float_representable(
    numerator: &HirExpr,
    denominator: &HirExpr,
    ctx: &LowerCtx,
) -> bool {
    if !proven_exact_integer_value(numerator, ctx)
        .as_ref()
        .is_some_and(is_exactly_representable_as_float)
        || !proven_exact_integer_value(denominator, ctx)
            .as_ref()
            .is_some_and(is_exactly_representable_as_float)
    {
        return false;
    }
    let (Some(numerator), Some(denominator)) = (
        proven_exact_integer_value(numerator, ctx),
        proven_exact_integer_value(denominator, ctx),
    ) else {
        return false;
    };
    if denominator.is_zero() {
        return false;
    }
    if numerator.is_zero() {
        return true;
    }

    let mut numerator = numerator.abs();
    let mut denominator = denominator.abs();
    let divisor = bigint_gcd(numerator.clone(), denominator.clone());
    numerator /= &divisor;
    denominator /= divisor;

    let one = BigInt::from(1_u8);
    if (&denominator & (&denominator - &one)) != BigInt::ZERO {
        return false;
    }
    let denominator_power = denominator.bits().saturating_sub(1);
    let mut numerator_twos = 0_u64;
    while (&numerator & &one) == BigInt::ZERO {
        numerator >>= 1_usize;
        numerator_twos += 1;
    }
    let significand_bits = numerator.bits();
    if significand_bits > 53 {
        return false;
    }
    let exponent = i128::from(numerator_twos) - i128::from(denominator_power);
    let highest_exponent = exponent + i128::from(significand_bits) - 1;
    highest_exponent <= 1023 && exponent >= -1074
}

pub(in crate::lower) fn proven_exact_integer_literal(
    expr: &HirExpr,
    ctx: &LowerCtx,
) -> Option<HirExpr> {
    let value = proven_exact_integer_value(expr, ctx)?;
    Some(value.to_i64().map_or_else(
        || HirExpr::LargeIntLiteral(value.to_string()),
        HirExpr::IntLiteral,
    ))
}

fn bigint_gcd(mut left: BigInt, mut right: BigInt) -> BigInt {
    while !right.is_zero() {
        let remainder = left % &right;
        left = right;
        right = remainder;
    }
    left
}

pub(in crate::lower) fn is_exact_or_fixed_int_like(ty: &Type) -> bool {
    matches!(
        ty.resolve_alias(),
        Type::Int | Type::LiteralInt(_) | Type::FixedInt(_)
    )
}

pub(in crate::lower) fn proven_exact_integer_value(
    expr: &HirExpr,
    ctx: &LowerCtx,
) -> Option<BigInt> {
    match expr {
        HirExpr::IntLiteral(value) => Some(BigInt::from(*value)),
        HirExpr::LargeIntLiteral(value) => value.parse::<BigInt>().ok(),
        HirExpr::UnaryOp { op, operand, .. } if op == "-" => {
            proven_exact_integer_value(operand, ctx).map(|value| -value)
        }
        HirExpr::Name {
            name, binding_id, ..
        } => ctx.scope.const_integer_value(name).cloned().or_else(|| {
            binding_id.and_then(|binding_id| ctx.const_integer_values.get(binding_id).cloned())
        }),
        _ => None,
    }
}

pub(in crate::lower) fn exact_integer_expr_is_proven_float_representable(
    expr: &HirExpr,
    ctx: &LowerCtx,
) -> bool {
    proven_exact_integer_value(expr, ctx)
        .as_ref()
        .is_some_and(is_exactly_representable_as_float)
}

pub(in crate::lower) fn is_exactly_representable_as_float(value: &BigInt) -> bool {
    if value.is_zero() {
        return true;
    }
    let mut significand = value.abs();
    let one = BigInt::from(1_u8);
    let mut exponent = 0_u64;
    while (&significand & &one).is_zero() {
        significand >>= 1_usize;
        exponent += 1;
    }
    significand.bits() <= 53
        && exponent
            .saturating_add(significand.bits())
            .saturating_sub(1)
            <= 1023
}

fn is_proven_nonzero_integer_expr(expr: &HirExpr, ctx: &LowerCtx) -> bool {
    if let Some(value) = proven_exact_integer_value(expr, ctx) {
        return !value.is_zero();
    }
    match expr {
        HirExpr::UnaryOp { op, operand, .. } if op == "-" => {
            is_proven_nonzero_integer_expr(operand, ctx)
        }
        HirExpr::Name { name, .. } => ctx.is_proven_nonzero_integer_binding(name),
        HirExpr::Call { func, args, .. } if func == "len" && args.len() == 1 => {
            super::super::sequence_guards::hir_sequence_guard_target_name(&args[0])
                .is_some_and(|name| ctx.min_length_guard(&name) > 0)
        }
        _ => false,
    }
}
