use super::*;

const DECIMAL_MAX_SCALE: i64 = 28;

fn decimal_diag_code(receiver_name: &str) -> &'static str {
    match receiver_name {
        "decimal" => "E2507",
        "bigdecimal" => "E2508",
        _ => "E2508",
    }
}

fn literal_scale_value(arg: &HirExpr) -> Option<i64> {
    match arg {
        HirExpr::IntLiteral(v) => Some(*v),
        HirExpr::UnaryOp { op, operand, .. } if op == "-" => match operand.as_ref() {
            HirExpr::IntLiteral(v) => v.checked_neg(),
            _ => None,
        },
        _ => match arg.ty() {
            Type::LiteralInt(v) => Some(*v),
            _ => None,
        },
    }
}

fn validate_decimal_context_scale(
    receiver_name: &str,
    method: &str,
    arg: &HirExpr,
    ctx: &mut LowerCtx,
) -> Option<()> {
    let Some(scale) = literal_scale_value(arg) else {
        return Some(());
    };
    if receiver_name == "decimal" && !(0..=DECIMAL_MAX_SCALE).contains(&scale) {
        ctx.error(format!(
            "[E2507] decimal.{method}() scale must be between 0 and {DECIMAL_MAX_SCALE}, got {scale}"
        ));
        return None;
    }
    if receiver_name == "bigdecimal" && scale < 0 {
        ctx.error(format!(
            "[E2508] bigdecimal.{method}() scale must be >= 0 for default context, got {scale}"
        ));
        return None;
    }
    Some(())
}

pub(super) fn decimal_conversion_error_type(ctx: &LowerCtx) -> Type {
    ctx.class_types
        .get("DecimalConversionError")
        .cloned()
        .unwrap_or(Type::Class {
            name: "DecimalConversionError".to_string(),
            fields: vec![("message".to_string(), Type::Str)],
            methods: vec![],
            parent_class: None,
        })
}

pub(super) fn validate_decimal_scale_argument(
    receiver_name: &str,
    method: &str,
    args: &[HirExpr],
    ctx: &mut LowerCtx,
) -> Option<()> {
    let diag_code = decimal_diag_code(receiver_name);
    if args.len() != 1 {
        ctx.error(format!(
            "[{diag_code}] {receiver_name}.{method}() takes exactly 1 argument, got {}",
            args.len()
        ));
        return None;
    }
    if !matches!(args[0].ty(), Type::Int | Type::LiteralInt(_)) {
        ctx.error(format!(
            "[{diag_code}] {receiver_name}.{method}() scale argument must be 'int', got '{}'",
            args[0].ty().display_name()
        ));
        return None;
    }
    validate_decimal_context_scale(receiver_name, method, &args[0], ctx)?;
    Some(())
}

pub(super) fn resolve_decimal_method_type(
    object_ty: &Type,
    method: &str,
    args: &[HirExpr],
    ctx: &mut LowerCtx,
) -> Option<Type> {
    match object_ty {
        Type::Decimal => match method {
            "quantize" => {
                validate_decimal_scale_argument("decimal", method, args, ctx)?;
                Some(Type::Decimal)
            }
            "sqrt" => {
                if !args.is_empty() {
                    ctx.error("decimal.sqrt() takes no arguments".to_string());
                    return None;
                }
                Some(Type::Result(
                    Box::new(Type::Decimal),
                    Box::new(decimal_conversion_error_type(ctx)),
                ))
            }
            "round" => {
                if args.len() > 1 {
                    ctx.error(format!(
                        "[E2507] decimal.round() takes at most 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                if args.len() == 1 && !matches!(args[0].ty(), Type::Int | Type::LiteralInt(_)) {
                    ctx.error(format!(
                        "[E2507] decimal.round() scale argument must be 'int', got '{}'",
                        args[0].ty().display_name()
                    ));
                    return None;
                }
                if args.len() == 1 {
                    validate_decimal_context_scale("decimal", "round", &args[0], ctx)?;
                }
                Some(Type::Decimal)
            }
            "abs" => {
                if !args.is_empty() {
                    ctx.error("decimal.abs() takes no arguments".to_string());
                    return None;
                }
                Some(Type::Decimal)
            }
            "is_zero" | "is_finite" => {
                if !args.is_empty() {
                    ctx.error(format!("decimal.{method}() takes no arguments"));
                    return None;
                }
                Some(Type::Bool)
            }
            _ => {
                ctx.error(format!("type 'decimal' has no method '{method}'"));
                None
            }
        },
        Type::BigDecimal => match method {
            "quantize" => {
                validate_decimal_scale_argument("bigdecimal", method, args, ctx)?;
                Some(Type::BigDecimal)
            }
            "sqrt" => {
                if !args.is_empty() {
                    ctx.error("bigdecimal.sqrt() takes no arguments".to_string());
                    return None;
                }
                Some(Type::Result(
                    Box::new(Type::BigDecimal),
                    Box::new(decimal_conversion_error_type(ctx)),
                ))
            }
            "round" => {
                if args.len() > 1 {
                    ctx.error(format!(
                        "[E2508] bigdecimal.round() takes at most 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                if args.len() == 1 && !matches!(args[0].ty(), Type::Int | Type::LiteralInt(_)) {
                    ctx.error(format!(
                        "[E2508] bigdecimal.round() scale argument must be 'int', got '{}'",
                        args[0].ty().display_name()
                    ));
                    return None;
                }
                if args.len() == 1 {
                    validate_decimal_context_scale("bigdecimal", "round", &args[0], ctx)?;
                }
                Some(Type::BigDecimal)
            }
            "abs" => {
                if !args.is_empty() {
                    ctx.error("bigdecimal.abs() takes no arguments".to_string());
                    return None;
                }
                Some(Type::BigDecimal)
            }
            "is_zero" | "is_finite" => {
                if !args.is_empty() {
                    ctx.error(format!("bigdecimal.{method}() takes no arguments"));
                    return None;
                }
                Some(Type::Bool)
            }
            _ => {
                ctx.error(format!("type 'bigdecimal' has no method '{method}'"));
                None
            }
        },
        _ => None,
    }
}
