macro_rules! stmt_expr_binop {
    ($emitter:ident, $expr:ident) => {{
        if let HirExpr::BinOp {
            left,
            op,
            right,
            ty,
        } = $expr
        {
            if let Some(lowered) = $emitter.try_lower_structured_class_binop_expr(left, op, right)? {
                return Ok(Some(lowered));
            }
            if let Some(lowered) = $emitter.try_lower_stmt_string_concat_expr_for_ir($expr)? {
                return Ok(Some(lowered));
            }
            let lowered_left = match $emitter.lower_stmt_expr_for_ir(left)? {
                Some(lowered) => lowered,
                None => {
                    let Some(lowered) = $emitter.try_lower_registry_expr_strict(left) else {
                        return Ok(None);
                    };
                    lowered
                }
            };
            let lowered_right = match $emitter.lower_stmt_expr_for_ir(right)? {
                Some(lowered) => lowered,
                None => {
                    let Some(lowered) = $emitter.try_lower_registry_expr_strict(right) else {
                        return Ok(None);
                    };
                    lowered
                }
            };
            let resolved_result_ty = crate::resolve_alias_type_for_plain_call(ty);
            let resolved_left_ty = crate::resolve_alias_type_for_plain_call(left.ty());
            let resolved_right_ty = crate::resolve_alias_type_for_plain_call(right.ty());

            if matches!(op.as_str(), "//" | "%")
                && matches!(resolved_left_ty, Type::Int | Type::LiteralInt(_))
                && matches!(resolved_right_ty, Type::Int | Type::LiteralInt(_))
                && is_result_int_division_error_type(resolved_result_ty)
            {
                let method = if op == "//" {
                    "checked_floor_div"
                } else {
                    "checked_floor_mod"
                };
                return Ok(Some(crate::RustExpr::Block {
                    stmts: vec![
                        crate::RustStmt::Let {
                            mutable: false,
                            name: "__sifr_floor_left".to_string(),
                            ty: Some(crate::RustType::Named("SifrInt".to_string())),
                            value: $emitter.coerce_expr_to_sifr_int_value(lowered_left),
                        },
                        crate::RustStmt::Let {
                            mutable: false,
                            name: "__sifr_floor_right".to_string(),
                            ty: Some(crate::RustType::Named("SifrInt".to_string())),
                            value: $emitter.coerce_expr_to_sifr_int_value(lowered_right),
                        },
                    ],
                    expr: Some(Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Ident(
                                "__sifr_floor_left".to_string(),
                            )),
                            method: method.to_string(),
                            args: vec![crate::RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(crate::RustExpr::Ident(
                                    "__sifr_floor_right".to_string(),
                                )),
                            }],
                        }),
                        method: "ok_or_else".to_string(),
                        args: vec![crate::RustExpr::Closure {
                            params: vec![],
                            body: Box::new(crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec![
                                    "DivisionError".to_string(),
                                    "new".to_string(),
                                ])),
                                args: vec![crate::RustExpr::Literal(crate::RustLiteral::Str(
                                    "division by zero".to_string(),
                                ))],
                            }),
                            is_move: false,
                        }],
                    })),
                }));
            }

            if op == "*" && matches!(resolved_result_ty, Type::Str) {
                let (string_expr, count_expr) = match (
                    matches!(resolved_left_ty, Type::Str),
                    matches!(resolved_right_ty, Type::Str),
                ) {
                    (true, false) => (lowered_left.clone(), lowered_right.clone()),
                    (false, true) => (lowered_right.clone(), lowered_left.clone()),
                    _ => return Ok(None),
                };
                return Ok(Some(crate::RustExpr::Block {
                    stmts: vec![crate::RustStmt::Let {
                        mutable: false,
                        name: "__n".to_string(),
                        ty: None,
                        value: count_expr,
                    }],
                    expr: Some(Box::new(crate::RustExpr::If {
                        cond: Box::new(crate::RustExpr::BinOp {
                            left: Box::new(crate::RustExpr::Ident("__n".to_string())),
                            op: "<=".to_string(),
                            right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(0))),
                        }),
                        then_expr: Box::new(crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec![
                                "String".to_string(),
                                "new".to_string(),
                            ])),
                            args: vec![],
                        }),
                        else_expr: Some(Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(string_expr))),
                            method: "repeat".to_string(),
                            args: vec![crate::RustExpr::Cast {
                                expr: Box::new(crate::RustExpr::Ident("__n".to_string())),
                                ty: crate::RustType::Named("usize".to_string()),
                            }],
                        })),
                    })),
                }));
            }

            if op == "*"
                && (matches!(resolved_result_ty, Type::List(_))
                    || matches!(resolved_result_ty, Type::Bytes))
            {
                let is_collection_like = |candidate: &Type| {
                    matches!(candidate, Type::List(_)) || matches!(candidate, Type::Bytes)
                };
                let is_count_like =
                    |candidate: &Type| matches!(candidate, Type::Int | Type::LiteralInt(_));
                let (collection_expr, count_expr) = match (
                    (
                        is_collection_like(resolved_left_ty),
                        is_count_like(resolved_right_ty),
                    ),
                    (
                        is_collection_like(resolved_right_ty),
                        is_count_like(resolved_left_ty),
                    ),
                ) {
                    ((true, true), _) => (lowered_left.clone(), lowered_right.clone()),
                    (_, (true, true)) => (lowered_right.clone(), lowered_left.clone()),
                    _ => return Ok(None),
                };
                if let crate::RustExpr::Vec(items) = &collection_expr {
                    if let [item] = items.as_slice() {
                        return Ok(Some(crate::RustExpr::Block {
                            stmts: vec![crate::RustStmt::Let {
                                mutable: false,
                                name: "__sifr_repeat_n".to_string(),
                                ty: None,
                                value: count_expr,
                            }],
                            expr: Some(Box::new(crate::RustExpr::If {
                                cond: Box::new(crate::RustExpr::BinOp {
                                    left: Box::new(crate::RustExpr::Ident(
                                        "__sifr_repeat_n".to_string(),
                                    )),
                                    op: "<=".to_string(),
                                    right: Box::new(crate::RustExpr::Literal(
                                        crate::RustLiteral::Int(0),
                                    )),
                                }),
                                then_expr: Box::new(crate::RustExpr::Vec(vec![])),
                                else_expr: Some(Box::new(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::FnCall {
                                            func: Box::new(crate::RustExpr::Path(vec![
                                                "std".to_string(),
                                                "iter".to_string(),
                                                "repeat".to_string(),
                                            ])),
                                            args: vec![item.clone()],
                                        }),
                                        method: "take".to_string(),
                                        args: vec![crate::RustExpr::Cast {
                                            expr: Box::new(crate::RustExpr::Ident(
                                                "__sifr_repeat_n".to_string(),
                                            )),
                                            ty: crate::RustType::Named("usize".to_string()),
                                        }],
                                    }),
                                    method: "collect::<Vec<_>>".to_string(),
                                    args: vec![],
                                })),
                            })),
                        }));
                    }
                }
                return Ok(Some(crate::RustExpr::Block {
                    stmts: vec![
                        crate::RustStmt::Let {
                            mutable: false,
                            name: "__sifr_repeat_src".to_string(),
                            ty: None,
                            value: crate::RustExpr::Clone(Box::new(crate::RustExpr::Paren(
                                Box::new(collection_expr),
                            ))),
                        },
                        crate::RustStmt::Let {
                            mutable: false,
                            name: "__sifr_repeat_n".to_string(),
                            ty: None,
                            value: count_expr,
                        },
                    ],
                    expr: Some(Box::new(crate::RustExpr::If {
                        cond: Box::new(crate::RustExpr::BinOp {
                            left: Box::new(crate::RustExpr::Ident("__sifr_repeat_n".to_string())),
                            op: "<=".to_string(),
                            right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(0))),
                        }),
                        then_expr: Box::new(crate::RustExpr::Vec(vec![])),
                        else_expr: Some(Box::new(crate::RustExpr::Block {
                            stmts: vec![
                                crate::RustStmt::Let {
                                    mutable: true,
                                    name: "__sifr_repeat_out".to_string(),
                                    ty: None,
                                    value: crate::RustExpr::Vec(vec![]),
                                },
                                crate::RustStmt::For {
                                    var: "_".to_string(),
                                    iter: crate::RustExpr::Range {
                                        start: Box::new(crate::RustExpr::Literal(
                                            crate::RustLiteral::Int(0),
                                        )),
                                        end: Box::new(crate::RustExpr::Ident(
                                            "__sifr_repeat_n".to_string(),
                                        )),
                                    },
                                    body: vec![crate::RustStmt::Expr(
                                        crate::RustExpr::MethodCall {
                                            receiver: Box::new(crate::RustExpr::Ident(
                                                "__sifr_repeat_out".to_string(),
                                            )),
                                            method: "extend".to_string(),
                                            args: vec![crate::RustExpr::MethodCall {
                                                receiver: Box::new(crate::RustExpr::MethodCall {
                                                    receiver: Box::new(crate::RustExpr::Paren(
                                                        Box::new(crate::RustExpr::Ident(
                                                            "__sifr_repeat_src".to_string(),
                                                        )),
                                                    )),
                                                    method: "iter".to_string(),
                                                    args: vec![],
                                                }),
                                                method: "cloned".to_string(),
                                                args: vec![],
                                            }],
                                        },
                                    )],
                                },
                            ],
                            expr: Some(Box::new(crate::RustExpr::Ident(
                                "__sifr_repeat_out".to_string(),
                            ))),
                        })),
                    })),
                }));
            }

            if op == "+"
                && (matches!(resolved_result_ty, Type::List(_))
                    || matches!(resolved_result_ty, Type::Bytes))
                && (matches!(resolved_left_ty, Type::List(_))
                    || matches!(resolved_left_ty, Type::Bytes))
                && (matches!(resolved_right_ty, Type::List(_))
                    || matches!(resolved_right_ty, Type::Bytes))
            {
                return Ok(Some(crate::RustExpr::Block {
                    stmts: vec![
                        crate::RustStmt::Let {
                            mutable: true,
                            name: "__v".to_string(),
                            ty: None,
                            value: crate::RustExpr::Clone(Box::new(crate::RustExpr::Paren(
                                Box::new(lowered_left.clone()),
                            ))),
                        },
                        crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Ident("__v".to_string())),
                            method: "extend".to_string(),
                            args: vec![crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                        lowered_right.clone(),
                                    ))),
                                    method: "iter".to_string(),
                                    args: vec![],
                                }),
                                method: "cloned".to_string(),
                                args: vec![],
                            }],
                        }),
                    ],
                    expr: Some(Box::new(crate::RustExpr::Ident("__v".to_string()))),
                }));
            }

            let runtime_exit_block = |message: &str| crate::RustExpr::Block {
                stmts: vec![crate::RustStmt::Expr(crate::RustExpr::FormatMacro {
                    name: "eprintln".to_string(),
                    format_str: message.to_string(),
                    args: vec![],
                })],
                expr: Some(Box::new(crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec![
                        "std".to_string(),
                        "process".to_string(),
                        "exit".to_string(),
                    ])),
                    args: vec![crate::RustExpr::Literal(crate::RustLiteral::Int(1))],
                })),
            };
            let bigdecimal_default_context_expr = || {
                let base_ctx = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "bigdecimal".to_string(),
                            "Context".to_string(),
                            "default".to_string(),
                        ])),
                        args: vec![],
                    }),
                    method: "with_rounding_mode".to_string(),
                    args: vec![crate::RustExpr::Path(vec![
                        "bigdecimal".to_string(),
                        "RoundingMode".to_string(),
                        "HalfEven".to_string(),
                    ])],
                };
                crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(base_ctx.clone()),
                        method: "with_prec".to_string(),
                        args: vec![crate::RustExpr::Literal(crate::RustLiteral::Int(28))],
                    }),
                    method: "unwrap_or_else".to_string(),
                    args: vec![crate::RustExpr::Closure {
                        params: vec![],
                        body: Box::new(base_ctx),
                        is_move: false,
                    }],
                }
            };
            let round_bigdecimal_with_default_context =
                |value: crate::RustExpr| crate::RustExpr::MethodCall {
                    receiver: Box::new(bigdecimal_default_context_expr()),
                    method: "round_decimal_ref".to_string(),
                    args: vec![crate::RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(crate::RustExpr::Paren(Box::new(value))),
                    }],
                };

            let mut lowered_left = lowered_left;
            let mut lowered_right = lowered_right;
            let is_move_arith_op = matches!(op.as_str(), "+" | "-" | "*" | "/" | "//" | "%" | "**");
            if is_move_arith_op {
                if matches!(resolved_left_ty, Type::BigInt) {
                    lowered_left = crate::RustExpr::Clone(Box::new(lowered_left));
                }
                if matches!(resolved_right_ty, Type::BigInt) {
                    lowered_right = crate::RustExpr::Clone(Box::new(lowered_right));
                }
                if matches!(resolved_left_ty, Type::BigDecimal) {
                    lowered_left = crate::RustExpr::Clone(Box::new(lowered_left));
                }
                if matches!(resolved_right_ty, Type::BigDecimal) {
                    lowered_right = crate::RustExpr::Clone(Box::new(lowered_right));
                }
            }
            if matches!(
                resolved_result_ty,
                Type::Int | Type::Float | Type::LiteralInt(_) | Type::TypeVar(_) | Type::BigInt
            ) {
                if Self::option_inner_type_for_ir(ty).is_none() {
                    if Self::option_inner_type_for_ir(left.ty()).is_some()
                        || Self::option_inner_type_for_ir(right.ty()).is_some()
                    {
                        return Err(crate::CodegenError::new(
                            "internal codegen invariant violated: numeric expression kept optional operand in non-optional context",
                        ));
                    }
                }
                if matches!(resolved_result_ty, Type::Float) {
                    if matches!(resolved_left_ty, Type::Int | Type::LiteralInt(_)) {
                        lowered_left = crate::RustExpr::Cast {
                            expr: Box::new(crate::RustExpr::Paren(Box::new(lowered_left))),
                            ty: crate::RustType::F64,
                        };
                    }
                    if matches!(resolved_right_ty, Type::Int | Type::LiteralInt(_)) {
                        lowered_right = crate::RustExpr::Cast {
                            expr: Box::new(crate::RustExpr::Paren(Box::new(lowered_right))),
                            ty: crate::RustType::F64,
                        };
                    }
                }
            }

            if matches!(resolved_result_ty, Type::Decimal) {
                let lower_bigint_to_decimal =
                    |value: crate::RustExpr| crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec![
                                "Decimal".to_string(),
                                "from_str_exact".to_string(),
                            ])),
                            args: vec![crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Paren(Box::new(value))),
                                    method: "to_string".to_string(),
                                    args: vec![],
                                }),
                                method: "as_str".to_string(),
                                args: vec![],
                            }],
                        }),
                        method: "unwrap_or_else".to_string(),
                        args: vec![crate::RustExpr::Closure {
                            params: vec![crate::RustParam::Named {
                                name: "__e".to_string(),
                                ty: crate::RustType::Named("_".to_string()),
                            }],
                            body: Box::new(crate::RustExpr::MacroCall {
                                name: "unreachable".to_string(),
                                args: vec![],
                            }),
                            is_move: false,
                        }],
                    };
                if matches!(resolved_left_ty, Type::Int | Type::LiteralInt(_)) {
                    lowered_left = crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "Decimal".to_string(),
                            "from".to_string(),
                        ])),
                        args: vec![lowered_left],
                    };
                } else if matches!(resolved_left_ty, Type::BigInt) {
                    lowered_left = lower_bigint_to_decimal(lowered_left);
                }
                if op != "**" && matches!(resolved_right_ty, Type::Int | Type::LiteralInt(_)) {
                    lowered_right = crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "Decimal".to_string(),
                            "from".to_string(),
                        ])),
                        args: vec![lowered_right],
                    };
                } else if op != "**" && matches!(resolved_right_ty, Type::BigInt) {
                    lowered_right = lower_bigint_to_decimal(lowered_right);
                }
            }

            if matches!(resolved_result_ty, Type::BigDecimal) {
                let lower_decimal_to_bigdecimal =
                    |value: crate::RustExpr| crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Paren(Box::new(value))),
                                method: "to_string".to_string(),
                                args: vec![],
                            }),
                            method: "parse::<BigDecimal>".to_string(),
                            args: vec![],
                        }),
                        method: "unwrap_or_else".to_string(),
                        args: vec![crate::RustExpr::Closure {
                            params: vec![crate::RustParam::Named {
                                name: "__e".to_string(),
                                ty: crate::RustType::Named("_".to_string()),
                            }],
                            body: Box::new(crate::RustExpr::MacroCall {
                                name: "unreachable".to_string(),
                                args: vec![],
                            }),
                            is_move: false,
                        }],
                    };
                if matches!(
                    resolved_left_ty,
                    Type::Int | Type::LiteralInt(_) | Type::BigInt
                ) {
                    lowered_left = crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "BigDecimal".to_string(),
                            "from".to_string(),
                        ])),
                        args: vec![lowered_left],
                    };
                } else if matches!(resolved_left_ty, Type::Decimal) {
                    lowered_left = lower_decimal_to_bigdecimal(lowered_left);
                }
                if op != "**"
                    && matches!(
                        resolved_right_ty,
                        Type::Int | Type::LiteralInt(_) | Type::BigInt
                    )
                {
                    lowered_right = crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "BigDecimal".to_string(),
                            "from".to_string(),
                        ])),
                        args: vec![lowered_right],
                    };
                } else if op != "**" && matches!(resolved_right_ty, Type::Decimal) {
                    lowered_right = lower_decimal_to_bigdecimal(lowered_right);
                }
            }

            if matches!(resolved_result_ty, Type::Decimal)
                && matches!(op.as_str(), "/" | "//" | "%")
            {
                let invalid_message = match op.as_str() {
                    "/" => "runtime error: decimal division failed (division by zero or overflow)",
                    "//" => {
                        "runtime error: decimal floor-division failed (division by zero or overflow)"
                    }
                    _ => "runtime error: decimal modulo failed (division by zero or overflow)",
                };
                let success_expr = match op.as_str() {
                    "/" => crate::RustExpr::Ident("__q".to_string()),
                    "//" => crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Ident("__q".to_string())),
                        method: "floor".to_string(),
                        args: vec![],
                    },
                    "%" => crate::RustExpr::BinOp {
                        left: Box::new(crate::RustExpr::Ident("__l".to_string())),
                        op: "-".to_string(),
                        right: Box::new(crate::RustExpr::BinOp {
                            left: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Ident("__q".to_string())),
                                method: "floor".to_string(),
                                args: vec![],
                            }),
                            op: "*".to_string(),
                            right: Box::new(crate::RustExpr::Ident("__r".to_string())),
                        }),
                    },
                    _ => return Ok(None),
                };
                return Ok(Some(crate::RustExpr::Block {
                    stmts: vec![
                        crate::RustStmt::Let {
                            mutable: false,
                            name: "__l".to_string(),
                            ty: None,
                            value: lowered_left,
                        },
                        crate::RustStmt::Let {
                            mutable: false,
                            name: "__r".to_string(),
                            ty: None,
                            value: lowered_right,
                        },
                    ],
                    expr: Some(Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec![
                                "Decimal".to_string(),
                                "checked_div".to_string(),
                            ])),
                            args: vec![
                                crate::RustExpr::Ident("__l".to_string()),
                                crate::RustExpr::Ident("__r".to_string()),
                            ],
                        }),
                        method: "map_or_else".to_string(),
                        args: vec![
                            crate::RustExpr::Closure {
                                params: vec![],
                                body: Box::new(runtime_exit_block(invalid_message)),
                                is_move: false,
                            },
                            crate::RustExpr::Closure {
                                params: vec![crate::RustParam::Named {
                                    name: "__q".to_string(),
                                    ty: crate::RustType::Named("_".to_string()),
                                }],
                                body: Box::new(success_expr),
                                is_move: false,
                            },
                        ],
                    })),
                }));
            }

            if matches!(resolved_result_ty, Type::BigDecimal)
                && matches!(op.as_str(), "+" | "-" | "*")
            {
                return Ok(Some(round_bigdecimal_with_default_context(
                    crate::RustExpr::BinOp {
                        left: Box::new(lowered_left),
                        op: op.clone(),
                        right: Box::new(lowered_right),
                    },
                )));
            }

            if matches!(resolved_result_ty, Type::BigDecimal)
                && matches!(op.as_str(), "/" | "//" | "%")
            {
                let invalid_message = match op.as_str() {
                    "/" => "runtime error: bigdecimal division by zero",
                    "//" => "runtime error: bigdecimal floor-division by zero",
                    _ => "runtime error: bigdecimal modulo by zero",
                };
                let zero_check = crate::RustExpr::BinOp {
                    left: Box::new(crate::RustExpr::Ident("__r".to_string())),
                    op: "==".to_string(),
                    right: Box::new(crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "BigDecimal".to_string(),
                            "from".to_string(),
                        ])),
                        args: vec![crate::RustExpr::Literal(crate::RustLiteral::Int(0))],
                    }),
                };
                let success_expr = match op.as_str() {
                    "/" => round_bigdecimal_with_default_context(crate::RustExpr::BinOp {
                        left: Box::new(crate::RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(crate::RustExpr::Ident("__l".to_string())),
                        }),
                        op: "/".to_string(),
                        right: Box::new(crate::RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(crate::RustExpr::Ident("__r".to_string())),
                        }),
                    }),
                    "//" => round_bigdecimal_with_default_context(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::BinOp {
                            left: Box::new(crate::RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(crate::RustExpr::Ident("__l".to_string())),
                            }),
                            op: "/".to_string(),
                            right: Box::new(crate::RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(crate::RustExpr::Ident("__r".to_string())),
                            }),
                        }),
                        method: "with_scale_round".to_string(),
                        args: vec![
                            crate::RustExpr::Literal(crate::RustLiteral::Int(0)),
                            crate::RustExpr::Path(vec![
                                "bigdecimal".to_string(),
                                "RoundingMode".to_string(),
                                "Floor".to_string(),
                            ]),
                        ],
                    }),
                    "%" => {
                        let floored_q = crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::BinOp {
                                left: Box::new(crate::RustExpr::Ref {
                                    mutable: false,
                                    expr: Box::new(crate::RustExpr::Ident("__l".to_string())),
                                }),
                                op: "/".to_string(),
                                right: Box::new(crate::RustExpr::Ref {
                                    mutable: false,
                                    expr: Box::new(crate::RustExpr::Ident("__r".to_string())),
                                }),
                            }),
                            method: "with_scale_round".to_string(),
                            args: vec![
                                crate::RustExpr::Literal(crate::RustLiteral::Int(0)),
                                crate::RustExpr::Path(vec![
                                    "bigdecimal".to_string(),
                                    "RoundingMode".to_string(),
                                    "Floor".to_string(),
                                ]),
                            ],
                        };
                        round_bigdecimal_with_default_context(crate::RustExpr::BinOp {
                            left: Box::new(crate::RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(crate::RustExpr::Ident("__l".to_string())),
                            }),
                            op: "-".to_string(),
                            right: Box::new(crate::RustExpr::BinOp {
                                left: Box::new(floored_q),
                                op: "*".to_string(),
                                right: Box::new(crate::RustExpr::Ref {
                                    mutable: false,
                                    expr: Box::new(crate::RustExpr::Ident("__r".to_string())),
                                }),
                            }),
                        })
                    }
                    _ => return Ok(None),
                };
                return Ok(Some(crate::RustExpr::Block {
                    stmts: vec![
                        crate::RustStmt::Let {
                            mutable: false,
                            name: "__l".to_string(),
                            ty: None,
                            value: lowered_left,
                        },
                        crate::RustStmt::Let {
                            mutable: false,
                            name: "__r".to_string(),
                            ty: None,
                            value: lowered_right,
                        },
                    ],
                    expr: Some(Box::new(crate::RustExpr::If {
                        cond: Box::new(zero_check),
                        then_expr: Box::new(runtime_exit_block(invalid_message)),
                        else_expr: Some(Box::new(success_expr)),
                    })),
                }));
            }

            if op == "**" {
                if matches!(resolved_left_ty, Type::Float)
                    || matches!(resolved_right_ty, Type::Float)
                    || matches!(resolved_result_ty, Type::Float)
                {
                    return Ok(Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_left))),
                        method: "powf".to_string(),
                        args: vec![crate::RustExpr::Cast {
                            expr: Box::new(lowered_right),
                            ty: crate::RustType::F64,
                        }],
                    }));
                }
                if matches!(resolved_result_ty, Type::Decimal) {
                    let exponent_i64 = if matches!(resolved_right_ty, Type::BigInt) {
                        crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec![
                                    "i64".to_string(),
                                    "try_from".to_string(),
                                ])),
                                args: vec![crate::RustExpr::Ref {
                                    mutable: false,
                                    expr: Box::new(crate::RustExpr::Paren(Box::new(
                                        lowered_right.clone(),
                                    ))),
                                }],
                            }),
                            method: "map_or_else".to_string(),
                            args: vec![
                                crate::RustExpr::Closure {
                                    params: vec![crate::RustParam::Named {
                                        name: "__e".to_string(),
                                        ty: crate::RustType::Named("_".to_string()),
                                    }],
                                    body: Box::new(runtime_exit_block(
                                        "runtime error: decimal exponent is out of i64 range",
                                    )),
                                    is_move: false,
                                },
                                crate::RustExpr::Closure {
                                    params: vec![crate::RustParam::Named {
                                        name: "__v".to_string(),
                                        ty: crate::RustType::Named("_".to_string()),
                                    }],
                                    body: Box::new(crate::RustExpr::Ident("__v".to_string())),
                                    is_move: false,
                                },
                            ],
                        }
                    } else {
                        lowered_right.clone()
                    };
                    return Ok(Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec![
                                "<Decimal as rust_decimal::MathematicalOps>".to_string(),
                                "checked_powi".to_string(),
                            ])),
                            args: vec![
                                crate::RustExpr::Ref {
                                    mutable: false,
                                    expr: Box::new(crate::RustExpr::Paren(Box::new(lowered_left))),
                                },
                                exponent_i64,
                            ],
                        }),
                        method: "map_or_else".to_string(),
                        args: vec![
                            crate::RustExpr::Closure {
                                params: vec![],
                                body: Box::new(runtime_exit_block(
                                    "runtime error: decimal exponentiation failed",
                                )),
                                is_move: false,
                            },
                            crate::RustExpr::Closure {
                                params: vec![crate::RustParam::Named {
                                    name: "__v".to_string(),
                                    ty: crate::RustType::Named("_".to_string()),
                                }],
                                body: Box::new(crate::RustExpr::Ident("__v".to_string())),
                                is_move: false,
                            },
                        ],
                    }));
                }
                if matches!(resolved_result_ty, Type::BigDecimal) {
                    let exponent_i64 = if matches!(resolved_right_ty, Type::BigInt) {
                        crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec![
                                    "i64".to_string(),
                                    "try_from".to_string(),
                                ])),
                                args: vec![crate::RustExpr::Ref {
                                    mutable: false,
                                    expr: Box::new(crate::RustExpr::Paren(Box::new(
                                        lowered_right.clone(),
                                    ))),
                                }],
                            }),
                            method: "map_or_else".to_string(),
                            args: vec![
                                crate::RustExpr::Closure {
                                    params: vec![crate::RustParam::Named {
                                        name: "__e".to_string(),
                                        ty: crate::RustType::Named("_".to_string()),
                                    }],
                                    body: Box::new(runtime_exit_block(
                                        "runtime error: bigdecimal exponent is out of i64 range",
                                    )),
                                    is_move: false,
                                },
                                crate::RustExpr::Closure {
                                    params: vec![crate::RustParam::Named {
                                        name: "__v".to_string(),
                                        ty: crate::RustType::Named("_".to_string()),
                                    }],
                                    body: Box::new(crate::RustExpr::Ident("__v".to_string())),
                                    is_move: false,
                                },
                            ],
                        }
                    } else {
                        lowered_right.clone()
                    };
                    return Ok(Some(round_bigdecimal_with_default_context(
                        crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_left))),
                            method: "powi_with_context".to_string(),
                            args: vec![
                                exponent_i64,
                                crate::RustExpr::Ref {
                                    mutable: false,
                                    expr: Box::new(bigdecimal_default_context_expr()),
                                },
                            ],
                        },
                    )));
                }
                return Ok(Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_left))),
                    method: "pow".to_string(),
                    args: vec![crate::RustExpr::Cast {
                        expr: Box::new(lowered_right),
                        ty: crate::RustType::Named("u32".to_string()),
                    }],
                }));
            }
            return Ok(Some(crate::stmt_support_emitter::binop_with_optional_operands(
                lowered_left,
                lowered_right,
                op,
                &resolved_left_ty,
                &resolved_right_ty,
                resolved_result_ty,
            )));
        }
    }};
}
