use crate::hir_nodes::HirExpr;
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{Expr, ExprCall};
use sifr_type_system::Type;
use std::str::FromStr;

use super::expressions::lower_expr;
use super::LowerCtx;

const DECIMAL_MAX_SCALE: i64 = 28;

fn decimal_scale_diagnostic_code(receiver_name: &str) -> DiagnosticCode {
    match receiver_name {
        "decimal" => DiagnosticCode::DECIMAL_SCALE_INVALID,
        "bigdecimal" => DiagnosticCode::DECIMAL_BIGDECIMAL_SCALE_OR_CONTEXT_INVALID,
        _ => DiagnosticCode::DECIMAL_BIGDECIMAL_SCALE_OR_CONTEXT_INVALID,
    }
}

fn decimal_method_arity_range(arg_ranges: &[TextRange], method_range: TextRange) -> TextRange {
    arg_ranges.last().copied().unwrap_or(method_range)
}

fn reject_decimal_method_wrong_count(
    ctx: &mut LowerCtx,
    message: String,
    arg_ranges: &[TextRange],
    method_range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT,
        message,
        decimal_method_arity_range(arg_ranges, method_range),
    );
}

fn reject_decimal_method_unsupported(ctx: &mut LowerCtx, message: String, method_range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE,
        message,
        method_range,
    );
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
    range: TextRange,
    ctx: &mut LowerCtx,
) -> Option<()> {
    let Some(scale) = literal_scale_value(arg) else {
        return Some(());
    };
    if receiver_name == "decimal" && !(0..=DECIMAL_MAX_SCALE).contains(&scale) {
        ctx.error_with_code_at(
            DiagnosticCode::DECIMAL_SCALE_INVALID,
            format!(
                "decimal.{method}() scale must be between 0 and {DECIMAL_MAX_SCALE}, got {scale}"
            ),
            range,
        );
        return None;
    }
    if receiver_name == "bigdecimal" && scale < 0 {
        ctx.error_with_code_at(
            DiagnosticCode::DECIMAL_BIGDECIMAL_SCALE_OR_CONTEXT_INVALID,
            format!("bigdecimal.{method}() scale must be >= 0 for default context, got {scale}"),
            range,
        );
        return None;
    }
    Some(())
}

pub(in crate::lower) fn decimal_conversion_error_type(ctx: &LowerCtx) -> Type {
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

pub(in crate::lower) fn validate_decimal_string_literal(
    value: &str,
    range: TextRange,
    ctx: &mut LowerCtx,
) -> Option<()> {
    if rust_decimal::Decimal::from_str_exact(value).is_err() {
        ctx.error_with_code_at(
            DiagnosticCode::DECIMAL_INVALID_LITERAL,
            format!("Decimal() received invalid exact literal '{value}'"),
            range,
        );
        return None;
    }
    Some(())
}

pub(in crate::lower) fn validate_bigdecimal_string_literal(
    value: &str,
    range: TextRange,
    ctx: &mut LowerCtx,
) -> Option<()> {
    if bigdecimal::BigDecimal::from_str(value).is_err() {
        ctx.error_with_code_at(
            DiagnosticCode::DECIMAL_BIGDECIMAL_INVALID_LITERAL,
            format!("BigDecimal() received invalid decimal literal '{value}'"),
            range,
        );
        return None;
    }
    Some(())
}

pub(in crate::lower) fn lower_decimal_constructor_call(
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    if call.arguments.args.len() != 1 {
        let range = if call.arguments.args.len() > 1 {
            call.arguments.args[1].range()
        } else {
            call.func.range()
        };
        ctx.error_with_code_at(
            DiagnosticCode::DECIMAL_FLOAT_CONSTRUCTION_FORBIDDEN,
            format!(
                "Decimal() takes exactly 1 argument, got {}",
                call.arguments.args.len()
            ),
            range,
        );
        return None;
    }
    let arg = lower_expr(&call.arguments.args[0], ctx)?;
    let arg_ty = arg.ty().clone();
    let result_ty = match arg_ty {
        Type::Str => {
            if let Expr::StringLiteral(lit) = &call.arguments.args[0] {
                validate_decimal_string_literal(
                    lit.value.to_str(),
                    call.arguments.args[0].range(),
                    ctx,
                )?;
            } else {
                ctx.error_with_code_at(
                    DiagnosticCode::DECIMAL_INVALID_LITERAL,
                    "Decimal() string construction requires a string literal".to_string(),
                    call.arguments.args[0].range(),
                );
                return None;
            }
            Type::Decimal
        }
        Type::Int | Type::LiteralInt(_) | Type::Decimal => Type::Decimal,
        Type::BigInt | Type::BigDecimal => Type::Result(
            Box::new(Type::Decimal),
            Box::new(decimal_conversion_error_type(ctx)),
        ),
        Type::Float => {
            ctx.error_with_code_at(
                DiagnosticCode::DECIMAL_FLOAT_CONSTRUCTION_FORBIDDEN,
                "Decimal(float_value) is not allowed; use Decimal(\"...\") for exact construction"
                    .to_string(),
                call.arguments.args[0].range(),
            );
            return None;
        }
        _ => {
            ctx.error_with_code_at(
                DiagnosticCode::DECIMAL_FLOAT_CONSTRUCTION_FORBIDDEN,
                format!(
                    "Decimal() requires str, int, bigint, decimal, or bigdecimal argument, got '{}'",
                    arg_ty.display_name()
                ),
                call.arguments.args[0].range(),
            );
            return None;
        }
    };
    Some(HirExpr::Call {
        func: "Decimal".to_string(),
        args: vec![arg],
        ty: result_ty,
    })
}

pub(in crate::lower) fn lower_bigdecimal_constructor_call(
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    if call.arguments.args.len() != 1 {
        let range = if call.arguments.args.len() > 1 {
            call.arguments.args[1].range()
        } else {
            call.func.range()
        };
        ctx.error_with_code_at(
            DiagnosticCode::DECIMAL_BIGDECIMAL_FLOAT_CONSTRUCTION_FORBIDDEN,
            format!(
                "BigDecimal() takes exactly 1 argument, got {}",
                call.arguments.args.len()
            ),
            range,
        );
        return None;
    }
    let arg = lower_expr(&call.arguments.args[0], ctx)?;
    let arg_ty = arg.ty().clone();
    match arg_ty {
        Type::Str => {
            if let Expr::StringLiteral(lit) = &call.arguments.args[0] {
                validate_bigdecimal_string_literal(
                    lit.value.to_str(),
                    call.arguments.args[0].range(),
                    ctx,
                )?;
            } else {
                ctx.error_with_code_at(
                    DiagnosticCode::DECIMAL_BIGDECIMAL_INVALID_LITERAL,
                    "BigDecimal() string construction requires a string literal".to_string(),
                    call.arguments.args[0].range(),
                );
                return None;
            }
        }
        Type::Int | Type::LiteralInt(_) | Type::BigInt | Type::Decimal | Type::BigDecimal => {}
        Type::Float => {
            ctx.error_with_code_at(
                DiagnosticCode::DECIMAL_BIGDECIMAL_FLOAT_CONSTRUCTION_FORBIDDEN,
                "BigDecimal(float_value) is not allowed; use BigDecimal(\"...\") for exact construction"
                    .to_string(),
                call.arguments.args[0].range(),
            );
            return None;
        }
        _ => {
            ctx.error_with_code_at(
                DiagnosticCode::DECIMAL_BIGDECIMAL_FLOAT_CONSTRUCTION_FORBIDDEN,
                format!(
                    "BigDecimal() requires str, int, bigint, decimal, or bigdecimal argument, got '{}'",
                    arg_ty.display_name()
                ),
                call.arguments.args[0].range(),
            );
            return None;
        }
    }
    Some(HirExpr::Call {
        func: "BigDecimal".to_string(),
        args: vec![arg],
        ty: Type::BigDecimal,
    })
}

pub(in crate::lower) fn validate_decimal_scale_argument(
    receiver_name: &str,
    method: &str,
    args: &[HirExpr],
    arg_ranges: &[TextRange],
    method_range: TextRange,
    ctx: &mut LowerCtx,
) -> Option<()> {
    let code = decimal_scale_diagnostic_code(receiver_name);
    if args.len() != 1 {
        let range = if args.len() > 1 {
            arg_ranges[1]
        } else {
            method_range
        };
        ctx.error_with_code_at(
            code,
            format!(
                "{receiver_name}.{method}() takes exactly 1 argument, got {}",
                args.len()
            ),
            range,
        );
        return None;
    }
    if !matches!(args[0].ty(), Type::Int | Type::LiteralInt(_)) {
        ctx.error_with_code_at(
            code,
            format!(
                "{receiver_name}.{method}() scale argument must be 'int', got '{}'",
                args[0].ty().display_name()
            ),
            arg_ranges[0],
        );
        return None;
    }
    validate_decimal_context_scale(receiver_name, method, &args[0], arg_ranges[0], ctx)?;
    Some(())
}

pub(in crate::lower) fn resolve_decimal_method_type(
    object_ty: &Type,
    method: &str,
    args: &[HirExpr],
    arg_ranges: &[TextRange],
    method_range: TextRange,
    ctx: &mut LowerCtx,
) -> Option<Type> {
    match object_ty {
        Type::Decimal => match method {
            "quantize" => {
                validate_decimal_scale_argument(
                    "decimal",
                    method,
                    args,
                    arg_ranges,
                    method_range,
                    ctx,
                )?;
                Some(Type::Decimal)
            }
            "sqrt" => {
                if !args.is_empty() {
                    reject_decimal_method_wrong_count(
                        ctx,
                        "decimal.sqrt() takes no arguments".to_string(),
                        arg_ranges,
                        method_range,
                    );
                    return None;
                }
                Some(Type::Result(
                    Box::new(Type::Decimal),
                    Box::new(decimal_conversion_error_type(ctx)),
                ))
            }
            "round" => {
                if args.len() > 1 {
                    ctx.error_with_code_at(
                        DiagnosticCode::DECIMAL_SCALE_INVALID,
                        format!(
                            "decimal.round() takes at most 1 argument, got {}",
                            args.len()
                        ),
                        arg_ranges[1],
                    );
                    return None;
                }
                if args.len() == 1 && !matches!(args[0].ty(), Type::Int | Type::LiteralInt(_)) {
                    ctx.error_with_code_at(
                        DiagnosticCode::DECIMAL_SCALE_INVALID,
                        format!(
                            "decimal.round() scale argument must be 'int', got '{}'",
                            args[0].ty().display_name()
                        ),
                        arg_ranges[0],
                    );
                    return None;
                }
                if args.len() == 1 {
                    validate_decimal_context_scale(
                        "decimal",
                        "round",
                        &args[0],
                        arg_ranges[0],
                        ctx,
                    )?;
                }
                Some(Type::Decimal)
            }
            "abs" => {
                if !args.is_empty() {
                    reject_decimal_method_wrong_count(
                        ctx,
                        "decimal.abs() takes no arguments".to_string(),
                        arg_ranges,
                        method_range,
                    );
                    return None;
                }
                Some(Type::Decimal)
            }
            "is_zero" | "is_finite" => {
                if !args.is_empty() {
                    reject_decimal_method_wrong_count(
                        ctx,
                        format!("decimal.{method}() takes no arguments"),
                        arg_ranges,
                        method_range,
                    );
                    return None;
                }
                Some(Type::Bool)
            }
            _ => {
                reject_decimal_method_unsupported(
                    ctx,
                    format!("type 'decimal' has no method '{method}'"),
                    method_range,
                );
                None
            }
        },
        Type::BigDecimal => match method {
            "quantize" => {
                validate_decimal_scale_argument(
                    "bigdecimal",
                    method,
                    args,
                    arg_ranges,
                    method_range,
                    ctx,
                )?;
                Some(Type::BigDecimal)
            }
            "sqrt" => {
                if !args.is_empty() {
                    reject_decimal_method_wrong_count(
                        ctx,
                        "bigdecimal.sqrt() takes no arguments".to_string(),
                        arg_ranges,
                        method_range,
                    );
                    return None;
                }
                Some(Type::Result(
                    Box::new(Type::BigDecimal),
                    Box::new(decimal_conversion_error_type(ctx)),
                ))
            }
            "round" => {
                if args.len() > 1 {
                    ctx.error_with_code_at(
                        DiagnosticCode::DECIMAL_BIGDECIMAL_SCALE_OR_CONTEXT_INVALID,
                        format!(
                            "bigdecimal.round() takes at most 1 argument, got {}",
                            args.len()
                        ),
                        arg_ranges[1],
                    );
                    return None;
                }
                if args.len() == 1 && !matches!(args[0].ty(), Type::Int | Type::LiteralInt(_)) {
                    ctx.error_with_code_at(
                        DiagnosticCode::DECIMAL_BIGDECIMAL_SCALE_OR_CONTEXT_INVALID,
                        format!(
                            "bigdecimal.round() scale argument must be 'int', got '{}'",
                            args[0].ty().display_name()
                        ),
                        arg_ranges[0],
                    );
                    return None;
                }
                if args.len() == 1 {
                    validate_decimal_context_scale(
                        "bigdecimal",
                        "round",
                        &args[0],
                        arg_ranges[0],
                        ctx,
                    )?;
                }
                Some(Type::BigDecimal)
            }
            "abs" => {
                if !args.is_empty() {
                    reject_decimal_method_wrong_count(
                        ctx,
                        "bigdecimal.abs() takes no arguments".to_string(),
                        arg_ranges,
                        method_range,
                    );
                    return None;
                }
                Some(Type::BigDecimal)
            }
            "is_zero" | "is_finite" => {
                if !args.is_empty() {
                    reject_decimal_method_wrong_count(
                        ctx,
                        format!("bigdecimal.{method}() takes no arguments"),
                        arg_ranges,
                        method_range,
                    );
                    return None;
                }
                Some(Type::Bool)
            }
            _ => {
                reject_decimal_method_unsupported(
                    ctx,
                    format!("type 'bigdecimal' has no method '{method}'"),
                    method_range,
                );
                None
            }
        },
        _ => None,
    }
}
