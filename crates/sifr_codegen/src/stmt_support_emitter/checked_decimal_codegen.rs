use crate::{CodegenError, RustEmitter, RustExpr, RustLiteral, RustParam, RustStmt};
use sifr_type_system::Type;

use super::result_type_helpers::result_error_member;

fn bigdecimal_is_zero(value: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "bigdecimal".to_string(),
            "Zero".to_string(),
            "is_zero".to_string(),
        ])),
        args: vec![RustExpr::Ref {
            mutable: false,
            expr: Box::new(value),
        }],
    }
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

pub(crate) fn decimal_to_bigdecimal_expr(value: RustExpr) -> RustExpr {
    RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__sifr_decimal".to_string(),
            ty: None,
            value,
        }],
        expr: Some(Box::new(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "BigDecimal".to_string(),
                "new".to_string(),
            ])),
            args: vec![
                RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "bigdecimal".to_string(),
                        "num_bigint".to_string(),
                        "BigInt".to_string(),
                        "from".to_string(),
                    ])),
                    args: vec![RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__sifr_decimal".to_string())),
                        method: "mantissa".to_string(),
                        args: vec![],
                    }],
                },
                RustExpr::Cast {
                    expr: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__sifr_decimal".to_string())),
                        method: "scale".to_string(),
                        args: vec![],
                    }),
                    ty: crate::RustType::I64,
                },
            ],
        })),
    }
}

fn bigdecimal_context_expr() -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "bigdecimal".to_string(),
            "Context".to_string(),
            "new".to_string(),
        ])),
        args: vec![
            RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "num".to_string(),
                    "NonZeroU64".to_string(),
                    "MIN".to_string(),
                ])),
                method: "saturating_add".to_string(),
                args: vec![RustExpr::Literal(RustLiteral::Int(27))],
            },
            RustExpr::Path(vec![
                "bigdecimal".to_string(),
                "RoundingMode".to_string(),
                "HalfEven".to_string(),
            ]),
        ],
    }
}

fn round_bigdecimal(value: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(bigdecimal_context_expr()),
        method: "round_decimal_ref".to_string(),
        args: vec![RustExpr::Ref {
            mutable: false,
            expr: Box::new(RustExpr::Paren(Box::new(value))),
        }],
    }
}

fn bigdecimal_operand(
    emitter: &RustEmitter,
    value: RustExpr,
    ty: &Type,
) -> Result<RustExpr, CodegenError> {
    match ty.resolve_alias() {
        Type::Int | Type::LiteralInt(_) | Type::FixedInt(_) => Ok(exact_int_to_bigdecimal_expr(
            emitter,
            emitter.coerce_typed_expr_to_sifr_int_value(value, ty),
        )),
        Type::Decimal => Ok(decimal_to_bigdecimal_expr(value)),
        Type::BigDecimal => Ok(RustExpr::Clone(Box::new(value))),
        _ => Err(CodegenError::new(
            "checked bigdecimal arithmetic received an unsupported operand type",
        )),
    }
}

pub(crate) fn lower_checked_bigdecimal_arithmetic(
    emitter: &RustEmitter,
    left: RustExpr,
    left_ty: &Type,
    op: &str,
    right: RustExpr,
    right_ty: &Type,
    result_ty: &Type,
) -> Result<Option<RustExpr>, CodegenError> {
    let Type::Result(ok_ty, _) = result_ty.resolve_alias() else {
        return Ok(None);
    };
    if !matches!(ok_ty.resolve_alias(), Type::BigDecimal) {
        return Ok(None);
    }

    let lowered_left = bigdecimal_operand(emitter, left, left_ty)?;
    if op == "**" {
        let division_error = wrap_result_error(
            result_ty,
            "DivisionError",
            named_error_value("DivisionError", "zero cannot be raised to a negative power"),
        )?;
        let exponent_result = map_decimal_conversion_to_result_error(
            result_ty,
            RustExpr::MethodCall {
                receiver: Box::new(emitter.coerce_typed_expr_to_sifr_int_value(right, right_ty)),
                method: "try_to_i64".to_string(),
                args: vec![],
            },
        )?;
        let success = RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
            args: vec![round_bigdecimal(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__sifr_bigdecimal_left".to_string())),
                method: "powi_with_context".to_string(),
                args: vec![
                    RustExpr::Ident("__sifr_bigdecimal_exponent".to_string()),
                    RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(bigdecimal_context_expr()),
                    },
                ],
            })],
        };
        let checked_power = RustExpr::If {
            cond: Box::new(RustExpr::BinOp {
                left: Box::new(bigdecimal_is_zero(RustExpr::Ident(
                    "__sifr_bigdecimal_left".to_string(),
                ))),
                op: "&&".to_string(),
                right: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("__sifr_bigdecimal_exponent".to_string())),
                    op: "<".to_string(),
                    right: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                }),
            }),
            then_expr: Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
                args: vec![division_error],
            }),
            else_expr: Some(Box::new(success)),
        };
        return Ok(Some(RustExpr::Block {
            stmts: vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__sifr_bigdecimal_left".to_string(),
                    ty: None,
                    value: lowered_left,
                },
                RustStmt::Let {
                    mutable: false,
                    name: "__sifr_bigdecimal_exponent_result".to_string(),
                    ty: None,
                    value: exponent_result,
                },
            ],
            expr: Some(Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(
                    "__sifr_bigdecimal_exponent_result".to_string(),
                )),
                method: "and_then".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "__sifr_bigdecimal_exponent".to_string(),
                        ty: crate::RustType::Named("_".to_string()),
                    }],
                    body: Box::new(checked_power),
                    is_move: true,
                }],
            })),
        }));
    }

    if !matches!(op, "/" | "//" | "%") {
        return Ok(None);
    }
    let lowered_right = bigdecimal_operand(emitter, right, right_ty)?;
    let division_error = wrap_result_error(
        result_ty,
        "DivisionError",
        named_error_value("DivisionError", "division by zero"),
    )?;
    let quotient = RustExpr::BinOp {
        left: Box::new(RustExpr::Ref {
            mutable: false,
            expr: Box::new(RustExpr::Ident("__sifr_bigdecimal_left".to_string())),
        }),
        op: "/".to_string(),
        right: Box::new(RustExpr::Ref {
            mutable: false,
            expr: Box::new(RustExpr::Ident("__sifr_bigdecimal_right".to_string())),
        }),
    };
    let success_value = match op {
        "/" => round_bigdecimal(quotient),
        "//" => round_bigdecimal(RustExpr::MethodCall {
            receiver: Box::new(quotient),
            method: "with_scale_round".to_string(),
            args: vec![
                RustExpr::Literal(RustLiteral::Int(0)),
                RustExpr::Path(vec![
                    "bigdecimal".to_string(),
                    "RoundingMode".to_string(),
                    "Floor".to_string(),
                ]),
            ],
        }),
        "%" => {
            let floored = RustExpr::MethodCall {
                receiver: Box::new(quotient),
                method: "with_scale_round".to_string(),
                args: vec![
                    RustExpr::Literal(RustLiteral::Int(0)),
                    RustExpr::Path(vec![
                        "bigdecimal".to_string(),
                        "RoundingMode".to_string(),
                        "Floor".to_string(),
                    ]),
                ],
            };
            round_bigdecimal(RustExpr::BinOp {
                left: Box::new(RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(RustExpr::Ident("__sifr_bigdecimal_left".to_string())),
                }),
                op: "-".to_string(),
                right: Box::new(RustExpr::BinOp {
                    left: Box::new(floored),
                    op: "*".to_string(),
                    right: Box::new(RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Ident("__sifr_bigdecimal_right".to_string())),
                    }),
                }),
            })
        }
        _ => return Ok(None),
    };
    Ok(Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__sifr_bigdecimal_left".to_string(),
                ty: None,
                value: lowered_left,
            },
            RustStmt::Let {
                mutable: false,
                name: "__sifr_bigdecimal_right".to_string(),
                ty: None,
                value: lowered_right,
            },
        ],
        expr: Some(Box::new(RustExpr::If {
            cond: Box::new(bigdecimal_is_zero(RustExpr::Ident(
                "__sifr_bigdecimal_right".to_string(),
            ))),
            then_expr: Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
                args: vec![division_error],
            }),
            else_expr: Some(Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                args: vec![success_value],
            })),
        })),
    }))
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

fn named_error_value(name: &str, message: &str) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![name.to_string(), "new".to_string()])),
        args: vec![RustExpr::Literal(RustLiteral::Str(message.to_string()))],
    }
}

fn wrap_result_error(
    result_ty: &Type,
    name: &str,
    value: RustExpr,
) -> Result<RustExpr, CodegenError> {
    let Some((error_ty, member_ty)) = result_error_member(result_ty, name) else {
        return Err(CodegenError::new(format!(
            "checked numeric result omitted required {name} member"
        )));
    };
    if crate::resolve_alias_type_for_plain_call(error_ty)
        == crate::resolve_alias_type_for_plain_call(member_ty)
    {
        return Ok(value);
    }
    crate::helpers::wrap_union_member_expr(error_ty, member_ty, value).ok_or_else(|| {
        CodegenError::new(format!(
            "checked numeric result could not wrap required {name} member"
        ))
    })
}

fn map_decimal_conversion_to_result_error(
    result_ty: &Type,
    receiver: RustExpr,
) -> Result<RustExpr, CodegenError> {
    let converted = wrap_result_error(
        result_ty,
        "DecimalConversionError",
        decimal_conversion_error(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__sifr_decimal_error".to_string())),
            method: "to_string".to_string(),
            args: vec![],
        }),
    )?;
    Ok(RustExpr::MethodCall {
        receiver: Box::new(receiver),
        method: "map_err".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "__sifr_decimal_error".to_string(),
                ty: crate::RustType::Named("_".to_string()),
            }],
            body: Box::new(converted),
            is_move: false,
        }],
    })
}

fn map_typed_decimal_conversion_to_result_error(
    result_ty: &Type,
    receiver: RustExpr,
) -> Result<RustExpr, CodegenError> {
    let converted = wrap_result_error(
        result_ty,
        "DecimalConversionError",
        RustExpr::Ident("__sifr_decimal_error".to_string()),
    )?;
    Ok(RustExpr::MethodCall {
        receiver: Box::new(receiver),
        method: "map_err".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "__sifr_decimal_error".to_string(),
                ty: crate::RustType::Named("_".to_string()),
            }],
            body: Box::new(converted),
            is_move: false,
        }],
    })
}

fn decimal_operand_result(
    emitter: &RustEmitter,
    value: RustExpr,
    ty: &Type,
    result_ty: &Type,
) -> Result<RustExpr, CodegenError> {
    let converted = match ty.resolve_alias() {
        Type::Int | Type::LiteralInt(_) => map_typed_decimal_conversion_to_result_error(
            result_ty,
            exact_int_to_decimal_result_expr(emitter, value),
        )?,
        Type::FixedInt(_) => RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
            args: vec![RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "Decimal".to_string(),
                    "from".to_string(),
                ])),
                args: vec![value],
            }],
        },
        Type::Decimal => RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
            args: vec![value],
        },
        _ => {
            return Err(CodegenError::new(
                "checked decimal arithmetic received an unsupported operand type",
            ));
        }
    };
    Ok(converted)
}

fn decimal_checked_operation(op: &str, result_ty: &Type) -> Result<Option<RustExpr>, CodegenError> {
    let left = RustExpr::Ident("__sifr_decimal_left".to_string());
    let right = RustExpr::Ident("__sifr_decimal_right".to_string());
    let right_for_zero_check = right.clone();
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
        "**" => RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "<Decimal as rust_decimal::MathematicalOps>".to_string(),
                "checked_powi".to_string(),
            ])),
            args: vec![
                RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(left),
                },
                RustExpr::Ident("__sifr_decimal_exponent".to_string()),
            ],
        },
        _ => return Ok(None),
    };
    let conversion_error = wrap_result_error(
        result_ty,
        "DecimalConversionError",
        named_error_value(
            "DecimalConversionError",
            &format!("decimal {op} operation overflowed its exact representation"),
        ),
    )?;
    let failure = if matches!(op, "/" | "//" | "%") {
        let division_error = wrap_result_error(
            result_ty,
            "DivisionError",
            named_error_value("DivisionError", "division by zero"),
        )?;
        RustExpr::If {
            cond: Box::new(RustExpr::MethodCall {
                receiver: Box::new(right_for_zero_check),
                method: "is_zero".to_string(),
                args: vec![],
            }),
            then_expr: Box::new(division_error),
            else_expr: Some(Box::new(conversion_error)),
        }
    } else {
        conversion_error
    };
    Ok(Some(RustExpr::MethodCall {
        receiver: Box::new(checked),
        method: "map_or_else".to_string(),
        args: vec![
            RustExpr::Closure {
                params: vec![],
                body: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
                    args: vec![failure],
                }),
                is_move: false,
            },
            RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "__sifr_decimal_value".to_string(),
                    ty: crate::RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                    args: vec![RustExpr::Ident("__sifr_decimal_value".to_string())],
                }),
                is_move: false,
            },
        ],
    }))
}

pub(crate) fn lower_checked_decimal_arithmetic(
    emitter: &RustEmitter,
    left: RustExpr,
    left_ty: &Type,
    op: &str,
    right: RustExpr,
    right_ty: &Type,
    result_ty: &Type,
) -> Result<Option<RustExpr>, CodegenError> {
    let Type::Result(ok_ty, error_ty) = result_ty.resolve_alias() else {
        return Ok(None);
    };
    if !matches!(ok_ty.resolve_alias(), Type::Decimal) {
        return Ok(None);
    }
    if result_error_member(result_ty, "DecimalConversionError").is_none() {
        return Err(CodegenError::new(format!(
            "checked decimal arithmetic has unsupported error type {}",
            error_ty.display_name()
        )));
    }

    if op == "**" {
        if !matches!(left_ty.resolve_alias(), Type::Decimal) {
            return Ok(None);
        }
        let left_result = decimal_operand_result(emitter, left, left_ty, result_ty)?;
        let exponent_result = map_decimal_conversion_to_result_error(
            result_ty,
            RustExpr::MethodCall {
                receiver: Box::new(emitter.coerce_typed_expr_to_sifr_int_value(right, right_ty)),
                method: "try_to_i64".to_string(),
                args: vec![],
            },
        )?;
        let Some(operation) = decimal_checked_operation(op, result_ty)? else {
            return Ok(None);
        };
        return Ok(Some(RustExpr::Block {
            stmts: vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__sifr_decimal_left_result".to_string(),
                    ty: None,
                    value: left_result,
                },
                RustStmt::Let {
                    mutable: false,
                    name: "__sifr_decimal_exponent_result".to_string(),
                    ty: None,
                    value: exponent_result,
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
                        receiver: Box::new(RustExpr::Ident(
                            "__sifr_decimal_exponent_result".to_string(),
                        )),
                        method: "and_then".to_string(),
                        args: vec![RustExpr::Closure {
                            params: vec![RustParam::Named {
                                name: "__sifr_decimal_exponent".to_string(),
                                ty: crate::RustType::Named("_".to_string()),
                            }],
                            body: Box::new(operation),
                            is_move: true,
                        }],
                    }),
                    is_move: true,
                }],
            })),
        }));
    }

    let Some(operation) = decimal_checked_operation(op, result_ty)? else {
        return Ok(None);
    };
    let left_result = decimal_operand_result(emitter, left, left_ty, result_ty)?;
    let right_result = decimal_operand_result(emitter, right, right_ty, result_ty)?;
    Ok(Some(RustExpr::Block {
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
    }))
}
