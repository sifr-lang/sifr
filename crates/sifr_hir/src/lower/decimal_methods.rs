use super::*;

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
    if args.len() != 1 {
        ctx.error(format!(
            "{receiver_name}.{method}() takes exactly 1 argument, got {}",
            args.len()
        ));
        return None;
    }
    if !matches!(args[0].ty(), Type::Int | Type::LiteralInt(_)) {
        ctx.error(format!(
            "{receiver_name}.{method}() scale argument must be 'int', got '{}'",
            args[0].ty().display_name()
        ));
        return None;
    }
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
                        "decimal.round() takes at most 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                if args.len() == 1 && !matches!(args[0].ty(), Type::Int | Type::LiteralInt(_)) {
                    ctx.error(format!(
                        "decimal.round() scale argument must be 'int', got '{}'",
                        args[0].ty().display_name()
                    ));
                    return None;
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
                        "bigdecimal.round() takes at most 1 argument, got {}",
                        args.len()
                    ));
                    return None;
                }
                if args.len() == 1 && !matches!(args[0].ty(), Type::Int | Type::LiteralInt(_)) {
                    ctx.error(format!(
                        "bigdecimal.round() scale argument must be 'int', got '{}'",
                        args[0].ty().display_name()
                    ));
                    return None;
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
