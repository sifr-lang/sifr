use crate::{CodegenError, RustEmitter, RustExpr, RustLiteral, RustMatchArm, RustParam, RustStmt};
use bigdecimal::num_bigint::BigInt;
use bigdecimal::num_traits::{FromPrimitive, ToPrimitive};
use sifr_type_system::Type;
use std::str::FromStr as _;

pub(crate) use super::checked_decimal_codegen::{
    decimal_to_bigdecimal_expr, exact_int_to_bigdecimal_expr, exact_int_to_decimal_result_expr,
};
use super::checked_decimal_codegen::{
    lower_checked_bigdecimal_arithmetic, lower_checked_decimal_arithmetic,
};
use super::result_type_helpers::{
    integer_arithmetic_error_union, integer_division_error_union,
    integer_float_conversion_error_union, is_result_int_division_error_type,
};

pub(crate) fn lower_numeric_binop_with_exact_integer_semantics(
    emitter: &mut RustEmitter,
    left: RustExpr,
    left_ty: &Type,
    op: &str,
    right: RustExpr,
    right_ty: &Type,
    right_source: &sifr_ir::HirExpr,
    result_ty: &Type,
) -> Result<Option<RustExpr>, CodegenError> {
    if let Some(lowered) = lower_checked_decimal_arithmetic(
        emitter,
        left.clone(),
        left_ty,
        op,
        right.clone(),
        right_ty,
        result_ty,
    )? {
        return Ok(Some(lowered));
    }
    if let Some(lowered) = lower_checked_bigdecimal_arithmetic(
        emitter,
        left.clone(),
        left_ty,
        op,
        right.clone(),
        right_ty,
        result_ty,
    )? {
        return Ok(Some(lowered));
    }

    let exact_or_fixed = |ty: &Type| {
        matches!(
            ty.resolve_alias(),
            Type::Int | Type::LiteralInt(_) | Type::FixedInt(_)
        )
    };
    let exact_operands = exact_or_fixed(left_ty) && exact_or_fixed(right_ty);
    if matches!(op, "/" | "//" | "%")
        && exact_operands
        && matches!(result_ty.resolve_alias(), Type::Int | Type::LiteralInt(_))
    {
        let runtime_op = if op == "%" { "%" } else { "/" };
        return Ok(Some(
            emitter.sifr_int_known_nonzero_floor_expr(runtime_op, left, right),
        ));
    }

    if matches!(op, "/" | "//" | "%")
        && exact_operands
        && is_result_int_division_error_type(result_ty)
    {
        let method = if op == "%" {
            "checked_floor_mod"
        } else {
            "checked_floor_div"
        };
        return Ok(Some(RustExpr::Block {
            stmts: vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__sifr_floor_left".to_string(),
                    ty: Some(crate::RustType::Named("SifrInt".to_string())),
                    value: emitter.coerce_expr_to_sifr_int_value(left),
                },
                RustStmt::Let {
                    mutable: false,
                    name: "__sifr_floor_right".to_string(),
                    ty: Some(crate::RustType::Named("SifrInt".to_string())),
                    value: emitter.coerce_expr_to_sifr_int_value(right),
                },
            ],
            expr: Some(Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__sifr_floor_left".to_string())),
                    method: method.to_string(),
                    args: vec![RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident("__sifr_floor_right".to_string())),
                    }],
                }),
                method: "ok_or_else".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![],
                    body: Box::new(RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "DivisionError".to_string(),
                            "new".to_string(),
                        ])),
                        args: vec![RustExpr::Literal(RustLiteral::Str(
                            "division by zero".to_string(),
                        ))],
                    }),
                    is_move: false,
                }],
            })),
        }));
    }

    if op == "/"
        && let Some(lowered) =
            lower_integer_true_division(emitter, left.clone(), right.clone(), result_ty)?
    {
        return Ok(Some(lowered));
    }

    if matches!(op, "**" | "<<" | ">>")
        && let Some(lowered) =
            lower_bounded_integer_arithmetic(emitter, left.clone(), right.clone(), op, result_ty)?
    {
        return Ok(Some(lowered));
    }

    if matches!(op, "**" | "<<" | ">>")
        && exact_operands
        && matches!(result_ty.resolve_alias(), Type::Int | Type::LiteralInt(_))
    {
        let primitive_ty = if op == "**" { "u32" } else { "usize" };
        let exponent = crate::integer_literal_decimal(right_source)
            .and_then(|value| value.parse::<u64>().ok())
            .and_then(|value| i64::try_from(value).ok())
            .ok_or_else(|| {
                CodegenError::new(
                    "bounded exact-integer operation reached codegen without a literal exponent",
                )
            })?;
        let method = if op == "**" {
            "pow_known_valid"
        } else if op == "<<" {
            "shl_known_valid"
        } else {
            "shr_known_valid"
        };
        return Ok(Some(RustExpr::MethodCall {
            receiver: Box::new(emitter.coerce_expr_to_sifr_int_value(left)),
            method: method.to_string(),
            args: vec![RustExpr::Cast {
                expr: Box::new(RustExpr::Literal(RustLiteral::Int(exponent))),
                ty: crate::RustType::Named(primitive_ty.to_string()),
            }],
        }));
    }

    lower_mixed_float_integer_arithmetic(emitter, left, right, left_ty, right_ty, op, result_ty)
}

pub(crate) fn exact_integer_float_literal(
    expr: &sifr_ir::HirExpr,
) -> Result<RustExpr, CodegenError> {
    let source = crate::integer_literal_decimal(expr).ok_or_else(|| {
        CodegenError::new(
            "exact integer reached infallible float rendering without literal proof metadata",
        )
    })?;
    let integer = BigInt::from_str(&source).map_err(|_| {
        CodegenError::new("exact integer literal could not be reconstructed for float rendering")
    })?;
    let value = integer
        .to_f64()
        .filter(|value| value.is_finite())
        .ok_or_else(|| {
            CodegenError::new("proven exact integer literal is outside the finite float range")
        })?;
    if BigInt::from_f64(value).as_ref() != Some(&integer) {
        return Err(CodegenError::new(
            "proven exact integer literal cannot be represented without float precision loss",
        ));
    }
    Ok(RustExpr::Literal(RustLiteral::Float(value)))
}

pub(crate) fn lower_integer_true_division(
    emitter: &RustEmitter,
    left: RustExpr,
    right: RustExpr,
    result_ty: &Type,
) -> Result<Option<RustExpr>, CodegenError> {
    let Some((error_union, division_ty, overflow_ty, precision_ty)) =
        integer_division_error_union(result_ty)
    else {
        return Ok(None);
    };
    let error_value = |name: &str, message: &str| RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![name.to_string(), "new".to_string()])),
        args: vec![RustExpr::Literal(RustLiteral::Str(message.to_string()))],
    };
    let wrap = |member_ty: &Type, value: RustExpr| {
        crate::helpers::wrap_union_member_expr(error_union, member_ty, value)
    };
    let Some(division_error) = wrap(
        division_ty,
        error_value("DivisionError", "division by zero"),
    ) else {
        return Err(CodegenError::new(
            "integer true-division error union omitted DivisionError",
        ));
    };
    let Some(overflow_error) = wrap(
        overflow_ty,
        error_value(
            "FloatOverflowError",
            "exact integer quotient is outside the finite float range",
        ),
    ) else {
        return Err(CodegenError::new(
            "integer true-division error union omitted FloatOverflowError",
        ));
    };
    let Some(precision_error) = wrap(
        precision_ty,
        error_value(
            "FloatPrecisionLossError",
            "exact integer quotient cannot be represented without float precision loss",
        ),
    ) else {
        return Err(CodegenError::new(
            "integer true-division error union omitted FloatPrecisionLossError",
        ));
    };
    Ok(Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(emitter.coerce_expr_to_sifr_int_value(left)),
            method: "checked_true_div".to_string(),
            args: vec![RustExpr::Ref {
                mutable: false,
                expr: Box::new(emitter.coerce_expr_to_sifr_int_value(right)),
            }],
        }),
        method: "map_err".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "__sifr_division_error".to_string(),
                ty: crate::RustType::Named("_".to_string()),
            }],
            body: Box::new(RustExpr::Match {
                expr: Box::new(RustExpr::Ident("__sifr_division_error".to_string())),
                arms: vec![
                    RustMatchArm {
                        pattern: "::sifr_runtime::IntegerDivisionError::DivisionByZero".to_string(),
                        bindings: vec![],
                        guard: None,
                        body: vec![RustStmt::TailExpr(division_error)],
                    },
                    RustMatchArm {
                        pattern: "::sifr_runtime::IntegerDivisionError::FloatOverflow".to_string(),
                        bindings: vec![],
                        guard: None,
                        body: vec![RustStmt::TailExpr(overflow_error)],
                    },
                    RustMatchArm {
                        pattern: "::sifr_runtime::IntegerDivisionError::FloatPrecisionLoss"
                            .to_string(),
                        bindings: vec![],
                        guard: None,
                        body: vec![RustStmt::TailExpr(precision_error)],
                    },
                ],
            }),
            is_move: false,
        }],
    }))
}

pub(crate) fn lower_bounded_integer_arithmetic(
    emitter: &RustEmitter,
    left: RustExpr,
    right: RustExpr,
    op: &str,
    result_ty: &Type,
) -> Result<Option<RustExpr>, CodegenError> {
    let Some((error_union, value_error_ty, limit_error_ty)) =
        integer_arithmetic_error_union(result_ty)
    else {
        return Ok(None);
    };
    let method = match op {
        "**" => "checked_pow",
        "<<" => "checked_shl",
        ">>" => "checked_shr",
        _ => return Ok(None),
    };
    let value_error = RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "ValueError".to_string(),
            "new".to_string(),
        ])),
        args: vec![RustExpr::Literal(RustLiteral::Str(
            "integer exponent or shift must be non-negative".to_string(),
        ))],
    };
    let limit_error = RustExpr::StructInit {
        name: "ArithmeticLimitError".to_string(),
        fields: vec![
            (
                "message".to_string(),
                RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Literal(RustLiteral::Str(
                        "integer output exceeds configured bit limit".to_string(),
                    ))),
                    method: "to_string".to_string(),
                    args: vec![],
                },
            ),
            (
                "limit".to_string(),
                RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "SifrInt".to_string(),
                        "from".to_string(),
                    ])),
                    args: vec![RustExpr::Ident("limit".to_string())],
                },
            ),
        ],
    };
    let Some(value_error) =
        crate::helpers::wrap_union_member_expr(error_union, value_error_ty, value_error)
    else {
        return Err(CodegenError::new(
            "integer arithmetic ValueError union member was not registered",
        ));
    };
    let Some(limit_error) =
        crate::helpers::wrap_union_member_expr(error_union, limit_error_ty, limit_error)
    else {
        return Err(CodegenError::new(
            "integer arithmetic limit-error union member was not registered",
        ));
    };
    Ok(Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(emitter.coerce_expr_to_sifr_int_value(left)),
            method: method.to_string(),
            args: vec![RustExpr::Ref {
                mutable: false,
                expr: Box::new(emitter.coerce_expr_to_sifr_int_value(right)),
            }],
        }),
        method: "map_err".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "__sifr_integer_error".to_string(),
                ty: crate::RustType::Named("_".to_string()),
            }],
            body: Box::new(RustExpr::Match {
                expr: Box::new(RustExpr::Ident("__sifr_integer_error".to_string())),
                arms: vec![
                    RustMatchArm {
                        pattern: "::sifr_runtime::IntegerArithmeticError::NegativeOperand"
                            .to_string(),
                        bindings: vec![],
                        guard: None,
                        body: vec![RustStmt::TailExpr(value_error)],
                    },
                    RustMatchArm {
                        pattern: "::sifr_runtime::IntegerArithmeticError::LimitExceeded { limit }"
                            .to_string(),
                        bindings: vec!["limit".to_string()],
                        guard: None,
                        body: vec![RustStmt::TailExpr(limit_error)],
                    },
                ],
            }),
            is_move: false,
        }],
    }))
}

pub(crate) fn lower_mixed_float_integer_arithmetic(
    emitter: &mut RustEmitter,
    left: RustExpr,
    right: RustExpr,
    left_ty: &Type,
    right_ty: &Type,
    op: &str,
    result_ty: &Type,
) -> Result<Option<RustExpr>, CodegenError> {
    let Some((error_union, overflow_ty, precision_ty)) =
        integer_float_conversion_error_union(result_ty)
    else {
        return Ok(None);
    };
    emitter.register_union_type(error_union);
    let left_is_integer = matches!(
        left_ty.resolve_alias(),
        Type::Int | Type::LiteralInt(_) | Type::FixedInt(_)
    );
    let right_is_integer = matches!(
        right_ty.resolve_alias(),
        Type::Int | Type::LiteralInt(_) | Type::FixedInt(_)
    );
    if left_is_integer == right_is_integer {
        return Ok(None);
    }
    let integer_ty = if left_is_integer { left_ty } else { right_ty };
    let integer_expr = if left_is_integer {
        left.clone()
    } else {
        right.clone()
    };
    let integer_expr = if matches!(integer_ty.resolve_alias(), Type::FixedInt(_)) {
        RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "SifrInt".to_string(),
                "from".to_string(),
            ])),
            args: vec![integer_expr],
        }
    } else {
        emitter.coerce_expr_to_sifr_int_method_receiver(integer_expr)
    };
    let converted = RustExpr::Ident("__sifr_integer_float".to_string());
    let operation = if op == "**" {
        RustExpr::MethodCall {
            receiver: Box::new(if left_is_integer {
                converted.clone()
            } else {
                left
            }),
            method: "powf".to_string(),
            args: vec![if right_is_integer {
                converted.clone()
            } else {
                right
            }],
        }
    } else {
        RustExpr::BinOp {
            left: Box::new(if left_is_integer {
                converted.clone()
            } else {
                left
            }),
            op: if op == "//" { "/" } else { op }.to_string(),
            right: Box::new(if right_is_integer {
                converted.clone()
            } else {
                right
            }),
        }
    };
    let error = |name: &str, message: &str| RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![name.to_string(), "new".to_string()])),
        args: vec![RustExpr::Literal(RustLiteral::Str(message.to_string()))],
    };
    let Some(overflow) = crate::helpers::wrap_union_member_expr(
        error_union,
        overflow_ty,
        error(
            "FloatOverflowError",
            "exact integer is outside the finite float range",
        ),
    ) else {
        return Err(CodegenError::new(
            "mixed numeric error union omitted FloatOverflowError",
        ));
    };
    let Some(precision) = crate::helpers::wrap_union_member_expr(
        error_union,
        precision_ty,
        error(
            "FloatPrecisionLossError",
            "exact integer cannot be represented without float precision loss",
        ),
    ) else {
        return Err(CodegenError::new(
            "mixed numeric error union omitted FloatPrecisionLossError",
        ));
    };
    let converted_result = RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(integer_expr),
            method: "checked_to_f64".to_string(),
            args: vec![],
        }),
        method: "map_err".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "__sifr_float_error".to_string(),
                ty: crate::RustType::Named("_".to_string()),
            }],
            body: Box::new(RustExpr::Match {
                expr: Box::new(RustExpr::Ident("__sifr_float_error".to_string())),
                arms: vec![
                    RustMatchArm {
                        pattern: "::sifr_runtime::IntegerFloatConversionError::Overflow"
                            .to_string(),
                        bindings: vec![],
                        guard: None,
                        body: vec![RustStmt::TailExpr(overflow)],
                    },
                    RustMatchArm {
                        pattern: "::sifr_runtime::IntegerFloatConversionError::PrecisionLoss"
                            .to_string(),
                        bindings: vec![],
                        guard: None,
                        body: vec![RustStmt::TailExpr(precision)],
                    },
                ],
            }),
            is_move: false,
        }],
    };
    Ok(Some(RustExpr::MethodCall {
        receiver: Box::new(converted_result),
        method: "map".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "__sifr_integer_float".to_string(),
                ty: crate::RustType::F64,
            }],
            body: Box::new(operation),
            is_move: false,
        }],
    }))
}
