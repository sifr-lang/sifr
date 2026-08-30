use crate::{CodegenError, RustEmitter, RustExpr, RustLiteral, RustMatchArm, RustParam, RustStmt};
use sifr_type_system::Type;

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
    result_ty: &Type,
) -> Result<Option<RustExpr>, CodegenError> {
    if let Some(lowered) = lower_decimal_integer_arithmetic(
        emitter,
        left.clone(),
        left_ty,
        op,
        right.clone(),
        right_ty,
        result_ty,
    ) {
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
        let method = match op {
            "**" => "pow_known_valid",
            "<<" => "shl_known_valid",
            ">>" => "shr_known_valid",
            _ => unreachable!(),
        };
        return Ok(Some(RustExpr::MethodCall {
            receiver: Box::new(emitter.coerce_expr_to_sifr_int_value(left)),
            method: method.to_string(),
            args: vec![emitter.coerce_expr_to_sifr_int_comparison_operand(right)],
        }));
    }

    lower_mixed_float_integer_arithmetic(emitter, left, right, left_ty, right_ty, op, result_ty)
}

pub(crate) fn exact_int_to_bigdecimal_expr(emitter: &RustEmitter, value: RustExpr) -> RustExpr {
    let signed_bytes = RustExpr::MethodCall {
        receiver: Box::new(emitter.coerce_expr_to_sifr_int_method_receiver(value)),
        method: "to_signed_bytes_be".to_string(),
        args: vec![],
    };
    let bigint = RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "bigdecimal".to_string(),
            "num_bigint".to_string(),
            "BigInt".to_string(),
            "from_signed_bytes_be".to_string(),
        ])),
        args: vec![RustExpr::Ref {
            mutable: false,
            expr: Box::new(signed_bytes),
        }],
    };
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "BigDecimal".to_string(),
            "new".to_string(),
        ])),
        args: vec![bigint, RustExpr::Literal(RustLiteral::Int(0))],
    }
}

fn decimal_conversion_error(message: RustExpr) -> RustExpr {
    RustExpr::StructInit {
        name: "DecimalConversionError".to_string(),
        fields: vec![("message".to_string(), message)],
    }
}

fn map_decimal_conversion_error(receiver: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(receiver),
        method: "map_err".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "__sifr_decimal_error".to_string(),
                ty: crate::RustType::Named("_".to_string()),
            }],
            body: Box::new(decimal_conversion_error(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__sifr_decimal_error".to_string())),
                method: "to_string".to_string(),
                args: vec![],
            })),
            is_move: false,
        }],
    }
}

pub(crate) fn exact_int_to_decimal_result_expr(emitter: &RustEmitter, value: RustExpr) -> RustExpr {
    let narrowed = map_decimal_conversion_error(RustExpr::MethodCall {
        receiver: Box::new(emitter.coerce_expr_to_sifr_int_method_receiver(value)),
        method: "try_to_i128".to_string(),
        args: vec![],
    });
    RustExpr::MethodCall {
        receiver: Box::new(narrowed),
        method: "and_then".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "__sifr_decimal_mantissa".to_string(),
                ty: crate::RustType::Named("_".to_string()),
            }],
            body: Box::new(map_decimal_conversion_error(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "Decimal".to_string(),
                    "try_from_i128_with_scale".to_string(),
                ])),
                args: vec![
                    RustExpr::Ident("__sifr_decimal_mantissa".to_string()),
                    RustExpr::Literal(RustLiteral::Int(0)),
                ],
            })),
            is_move: false,
        }],
    }
}

fn decimal_operand_result(emitter: &RustEmitter, value: RustExpr, ty: &Type) -> RustExpr {
    if matches!(ty.resolve_alias(), Type::Int | Type::LiteralInt(_)) {
        exact_int_to_decimal_result_expr(emitter, value)
    } else {
        RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
            args: vec![value],
        }
    }
}

fn decimal_checked_operation(op: &str) -> Option<RustExpr> {
    let left = RustExpr::Ident("__sifr_decimal_left".to_string());
    let right = RustExpr::Ident("__sifr_decimal_right".to_string());
    let checked = match op {
        "+" => RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "Decimal".to_string(),
                "checked_add".to_string(),
            ])),
            args: vec![left, right],
        },
        "-" => RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "Decimal".to_string(),
                "checked_sub".to_string(),
            ])),
            args: vec![left, right],
        },
        "*" => RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "Decimal".to_string(),
                "checked_mul".to_string(),
            ])),
            args: vec![left, right],
        },
        "/" => RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "Decimal".to_string(),
                "checked_div".to_string(),
            ])),
            args: vec![left, right],
        },
        "//" => RustExpr::MethodCall {
            receiver: Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "Decimal".to_string(),
                    "checked_div".to_string(),
                ])),
                args: vec![left, right],
            }),
            method: "map".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "__sifr_decimal_quotient".to_string(),
                    ty: crate::RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__sifr_decimal_quotient".to_string())),
                    method: "floor".to_string(),
                    args: vec![],
                }),
                is_move: false,
            }],
        },
        "%" => {
            let quotient = RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "Decimal".to_string(),
                    "checked_div".to_string(),
                ])),
                args: vec![left.clone(), right.clone()],
            };
            RustExpr::MethodCall {
                receiver: Box::new(quotient),
                method: "and_then".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "__sifr_decimal_quotient".to_string(),
                        ty: crate::RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec![
                                "Decimal".to_string(),
                                "checked_mul".to_string(),
                            ])),
                            args: vec![
                                RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident(
                                        "__sifr_decimal_quotient".to_string(),
                                    )),
                                    method: "floor".to_string(),
                                    args: vec![],
                                },
                                right,
                            ],
                        }),
                        method: "and_then".to_string(),
                        args: vec![RustExpr::Closure {
                            params: vec![RustParam::Named {
                                name: "__sifr_decimal_product".to_string(),
                                ty: crate::RustType::Named("_".to_string()),
                            }],
                            body: Box::new(RustExpr::FnCall {
                                func: Box::new(RustExpr::Path(vec![
                                    "Decimal".to_string(),
                                    "checked_sub".to_string(),
                                ])),
                                args: vec![
                                    left,
                                    RustExpr::Ident("__sifr_decimal_product".to_string()),
                                ],
                            }),
                            is_move: false,
                        }],
                    }),
                    is_move: false,
                }],
            }
        }
        _ => return None,
    };
    Some(RustExpr::MethodCall {
        receiver: Box::new(checked),
        method: "ok_or_else".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![],
            body: Box::new(decimal_conversion_error(RustExpr::Literal(
                RustLiteral::Str(format!(
                    "decimal {op} operation failed (division by zero or overflow)"
                )),
            ))),
            is_move: false,
        }],
    })
}

pub(crate) fn lower_decimal_integer_arithmetic(
    emitter: &RustEmitter,
    left: RustExpr,
    left_ty: &Type,
    op: &str,
    right: RustExpr,
    right_ty: &Type,
    result_ty: &Type,
) -> Option<RustExpr> {
    let Type::Result(ok_ty, error_ty) = result_ty.resolve_alias() else {
        return None;
    };
    if !matches!(ok_ty.resolve_alias(), Type::Decimal)
        || !matches!(error_ty.resolve_alias(), Type::Class { name, .. } if name == "DecimalConversionError")
    {
        return None;
    }
    let operation = decimal_checked_operation(op)?;
    let left_result = decimal_operand_result(emitter, left, left_ty);
    let right_result = decimal_operand_result(emitter, right, right_ty);
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__sifr_decimal_left_result".to_string(),
                ty: None,
                value: left_result,
            },
            RustStmt::Let {
                mutable: false,
                name: "__sifr_decimal_right_result".to_string(),
                ty: None,
                value: right_result,
            },
        ],
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__sifr_decimal_left_result".to_string())),
            method: "and_then".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "__sifr_decimal_left".to_string(),
                    ty: crate::RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__sifr_decimal_right_result".to_string())),
                    method: "and_then".to_string(),
                    args: vec![RustExpr::Closure {
                        params: vec![RustParam::Named {
                            name: "__sifr_decimal_right".to_string(),
                            ty: crate::RustType::Named("_".to_string()),
                        }],
                        body: Box::new(operation),
                        is_move: true,
                    }],
                }),
                is_move: true,
            }],
        })),
    })
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
