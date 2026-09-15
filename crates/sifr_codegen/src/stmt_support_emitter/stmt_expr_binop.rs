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

            if let Some(lowered) = $crate::stmt_support_emitter::checked_integer_codegen::lower_numeric_binop_with_exact_integer_semantics(
                $emitter,
                $crate::stmt_support_emitter::checked_integer_codegen::CheckedNumericBinop {
                    left: lowered_left.clone(),
                    left_ty: resolved_left_ty,
                    op,
                    right: lowered_right.clone(),
                    right_ty: resolved_right_ty,
                    right_source: right,
                    result_ty: resolved_result_ty,
                },
            )? {
                return Ok(Some(lowered));
            }
            if matches!(resolved_result_ty, Type::Decimal)
                || (matches!(resolved_result_ty, Type::BigDecimal)
                    && matches!(op.as_str(), "/" | "//" | "%" | "**"))
            {
                return Err(crate::CodegenError::new(
                    "internal codegen invariant violated: error-producing decimal arithmetic reached rendering without a typed Result",
                ));
            }

            if op == "*" && matches!(resolved_result_ty, Type::Str) {
                let (string_expr, count_source, count_expr) = match (
                    matches!(resolved_left_ty, Type::Str),
                    matches!(resolved_right_ty, Type::Str),
                ) {
                    (true, false) => (lowered_left.clone(), right.as_ref(), lowered_right.clone()),
                    (false, true) => (lowered_right.clone(), left.as_ref(), lowered_left.clone()),
                    _ => return Ok(None),
                };
                let mut operand_stmts = vec![
                        crate::RustStmt::Let {
                            mutable: false,
                            name: "__sifr_repeat_src".to_string(),
                            ty: Some(crate::RustType::Named("&str".to_string())),
                            value: crate::RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(crate::RustExpr::Paren(Box::new(string_expr))),
                            },
                        },
                        crate::RustStmt::Let {
                            mutable: false,
                            name: "__n".to_string(),
                            ty: None,
                            value: $emitter.materialize_reusable_value_for_ir(count_source, count_expr),
                        },
                    ];
                if !matches!(resolved_left_ty, Type::Str) {
                    operand_stmts.swap(0, 1);
                }
                return Ok(Some(crate::RustExpr::Block {
                    stmts: operand_stmts,
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
                        else_expr: Some(Box::new(crate::RustExpr::Block {
                            stmts: vec![
                                crate::RustStmt::Let {
                                    mutable: true,
                                    name: "__sifr_repeat_out".to_string(),
                                    ty: None,
                                    value: crate::RustExpr::FnCall {
                                        func: Box::new(crate::RustExpr::Path(vec![
                                            "String".to_string(),
                                            "new".to_string(),
                                        ])),
                                        args: vec![],
                                    },
                                },
                                crate::RustStmt::Let {
                                    mutable: true,
                                    name: "__sifr_repeat_i".to_string(),
                                    ty: None,
                                    value: crate::RustExpr::FnCall {
                                        func: Box::new(crate::RustExpr::Path(vec![
                                            "SifrInt".to_string(),
                                            "from_i64".to_string(),
                                        ])),
                                        args: vec![crate::RustExpr::Literal(
                                            crate::RustLiteral::Int(0),
                                        )],
                                    },
                                },
                                crate::RustStmt::While {
                                    cond: crate::RustExpr::BinOp {
                                        left: Box::new(crate::RustExpr::Ref {
                                            mutable: false,
                                            expr: Box::new(crate::RustExpr::Ident(
                                                "__sifr_repeat_i".to_string(),
                                            )),
                                        }),
                                        op: "<".to_string(),
                                        right: Box::new(crate::RustExpr::Ref {
                                            mutable: false,
                                            expr: Box::new(crate::RustExpr::Ident(
                                                "__n".to_string(),
                                            )),
                                        }),
                                    },
                                    body: vec![
                                        crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                                            receiver: Box::new(crate::RustExpr::Ident(
                                                "__sifr_repeat_out".to_string(),
                                            )),
                                            method: "push_str".to_string(),
                                            args: vec![crate::RustExpr::Ident(
                                                "__sifr_repeat_src".to_string(),
                                            )],
                                        }),
                                        crate::RustStmt::AugAssign {
                                            target: crate::RustExpr::Ident(
                                                "__sifr_repeat_i".to_string(),
                                            ),
                                            op: "+".to_string(),
                                            value: crate::RustExpr::FnCall {
                                                func: Box::new(crate::RustExpr::Path(vec![
                                                    "SifrInt".to_string(),
                                                    "from_i64".to_string(),
                                                ])),
                                                args: vec![crate::RustExpr::Literal(
                                                    crate::RustLiteral::Int(1),
                                                )],
                                            },
                                        },
                                    ],
                                },
                            ],
                            expr: Some(Box::new(crate::RustExpr::Ident(
                                "__sifr_repeat_out".to_string(),
                            ))),
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
                let (collection_expr, count_source, count_expr) = match (
                    (
                        is_collection_like(resolved_left_ty),
                        is_count_like(resolved_right_ty),
                    ),
                    (
                        is_collection_like(resolved_right_ty),
                        is_count_like(resolved_left_ty),
                    ),
                ) {
                    ((true, true), _) => (lowered_left.clone(), right.as_ref(), lowered_right.clone()),
                    (_, (true, true)) => (lowered_right.clone(), left.as_ref(), lowered_left.clone()),
                    _ => return Ok(None),
                };
                if let crate::RustExpr::Vec(elements) = &collection_expr {
                    if let [element] = elements.as_slice() {
                        return Ok(Some($emitter.lower_singleton_repeat_for_ir(
                            element.clone(),
                            count_source,
                            count_expr,
                            is_count_like(resolved_left_ty),
                        )));
                    }
                }
                let mut operand_stmts = vec![
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
                            value: $emitter.materialize_reusable_value_for_ir(count_source, count_expr),
                        },
                    ];
                if is_count_like(resolved_left_ty) {
                    operand_stmts.swap(0, 1);
                }
                return Ok(Some(crate::RustExpr::Block {
                    stmts: operand_stmts,
                    expr: Some(Box::new(crate::RustExpr::If {
                        cond: Box::new(crate::RustExpr::BinOp {
                            left: Box::new(crate::RustExpr::Ident("__sifr_repeat_n".to_string())),
                            op: "<=".to_string(),
                            right: Box::new(crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec![
                                    "SifrInt".to_string(),
                                    "from_i64".to_string(),
                                ])),
                                args: vec![crate::RustExpr::Literal(crate::RustLiteral::Int(0))],
                            }),
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
                                crate::RustStmt::Let {
                                    mutable: true,
                                    name: "__sifr_repeat_i".to_string(),
                                    ty: None,
                                    value: crate::RustExpr::FnCall {
                                        func: Box::new(crate::RustExpr::Path(vec![
                                            "SifrInt".to_string(),
                                            "from_i64".to_string(),
                                        ])),
                                        args: vec![crate::RustExpr::Literal(
                                            crate::RustLiteral::Int(0),
                                        )],
                                    },
                                },
                                crate::RustStmt::While {
                                    cond: crate::RustExpr::BinOp {
                                        left: Box::new(crate::RustExpr::Ref {
                                            mutable: false,
                                            expr: Box::new(crate::RustExpr::Ident(
                                                "__sifr_repeat_i".to_string(),
                                            )),
                                        }),
                                        op: "<".to_string(),
                                        right: Box::new(crate::RustExpr::Ref {
                                            mutable: false,
                                            expr: Box::new(crate::RustExpr::Ident(
                                                "__sifr_repeat_n".to_string(),
                                            )),
                                        }),
                                    },
                                    body: vec![
                                        crate::RustStmt::Expr(crate::RustExpr::MethodCall {
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
                                        }),
                                        crate::RustStmt::AugAssign {
                                            target: crate::RustExpr::Ident(
                                                "__sifr_repeat_i".to_string(),
                                            ),
                                            op: "+".to_string(),
                                            value: crate::RustExpr::FnCall {
                                                func: Box::new(crate::RustExpr::Path(vec![
                                                    "SifrInt".to_string(),
                                                    "from_i64".to_string(),
                                                ])),
                                                args: vec![crate::RustExpr::Literal(
                                                    crate::RustLiteral::Int(1),
                                                )],
                                            },
                                        },
                                    ],
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
                            value: crate::ownership_plan::materialize_owned_value(
                                &resolved_left_ty,
                                crate::RustExpr::Paren(Box::new(lowered_left.clone())),
                            ),
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

            let bigdecimal_default_context_expr = || {
                crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec![
                        "bigdecimal".to_string(),
                        "Context".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![
                        crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Path(vec![
                                "std".to_string(),
                                "num".to_string(),
                                "NonZeroU64".to_string(),
                                "MIN".to_string(),
                            ])),
                            method: "saturating_add".to_string(),
                            args: vec![crate::RustExpr::Literal(crate::RustLiteral::Int(27))],
                        },
                        crate::RustExpr::Path(vec![
                        "bigdecimal".to_string(),
                        "RoundingMode".to_string(),
                        "HalfEven".to_string(),
                        ]),
                    ],
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
                lowered_left = $emitter.clone_borrowed_generic_operand(left, resolved_left_ty, lowered_left);
                lowered_right = $emitter.clone_borrowed_generic_operand(right, resolved_right_ty, lowered_right);
                if matches!(resolved_left_ty, Type::BigDecimal) {
                    lowered_left = crate::RustExpr::Clone(Box::new(lowered_left));
                }
                if matches!(resolved_right_ty, Type::BigDecimal) {
                    lowered_right = crate::RustExpr::Clone(Box::new(lowered_right));
                }
            }
            let exact_integer_operand = |value: crate::RustExpr, operand_ty: &Type| {
                match operand_ty {
                    Type::FixedInt(_) => $emitter
                        .coerce_typed_expr_to_sifr_int_value(value, operand_ty),
                    Type::Int | Type::LiteralInt(_) => $emitter
                        .coerce_expr_to_sifr_int_comparison_operand(value),
                    _ => value,
                }
            };
            if matches!(resolved_result_ty, Type::Int | Type::LiteralInt(_))
                && matches!(op.as_str(), "+" | "-" | "*" | "&" | "|" | "^")
            {
                lowered_left = exact_integer_operand(lowered_left, resolved_left_ty);
                lowered_right = exact_integer_operand(lowered_right, resolved_right_ty);
            }
            if matches!(resolved_result_ty, Type::Int | Type::LiteralInt(_))
                && matches!(op.as_str(), "<<" | ">>")
            {
                let shift = crate::integer_literal_decimal(right)
                    .and_then(|value| value.parse::<usize>().ok())
                    .ok_or_else(|| crate::CodegenError::new(
                        "exact integer shift reached codegen without a proven target-width shift count",
                    ))?;
                let shift = i64::try_from(shift).map_err(|_| {
                    crate::CodegenError::new(
                        "exact integer shift count exceeds generated Rust literal width",
                    )
                })?;
                return Ok(Some(crate::RustExpr::BinOp {
                    left: Box::new(exact_integer_operand(lowered_left, resolved_left_ty)),
                    op: op.clone(),
                    right: Box::new(crate::RustExpr::Cast {
                        expr: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(shift))),
                        ty: crate::RustType::Named("usize".to_string()),
                    }),
                }));
            }
            if matches!(
                resolved_result_ty,
                Type::Int | Type::Float | Type::LiteralInt(_) | Type::TypeVar(_)
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
                        lowered_left = crate::stmt_support_emitter::checked_integer_codegen::exact_integer_float_literal(left)?;
                    } else if matches!(resolved_left_ty, Type::FixedInt(_)) {
                        lowered_left = crate::RustExpr::Cast {
                            expr: Box::new(lowered_left),
                            ty: crate::RustType::F64,
                        };
                    }
                    if matches!(resolved_right_ty, Type::Int | Type::LiteralInt(_)) {
                        lowered_right = crate::stmt_support_emitter::checked_integer_codegen::exact_integer_float_literal(right)?;
                    } else if matches!(resolved_right_ty, Type::FixedInt(_)) {
                        lowered_right = crate::RustExpr::Cast {
                            expr: Box::new(lowered_right),
                            ty: crate::RustType::F64,
                        };
                    }
                }
            }

            if matches!(resolved_result_ty, Type::BigDecimal) {
                let lower_decimal_to_bigdecimal = |value: crate::RustExpr| {
                    crate::stmt_support_emitter::checked_integer_codegen::decimal_to_bigdecimal_expr(
                        value,
                    )
                };
                if matches!(
                    resolved_left_ty,
                    Type::Int | Type::LiteralInt(_)
                ) {
                    lowered_left = crate::stmt_support_emitter::checked_integer_codegen::exact_int_to_bigdecimal_expr(
                        $emitter,
                        lowered_left,
                    );
                } else if matches!(resolved_left_ty, Type::Decimal) {
                    lowered_left = lower_decimal_to_bigdecimal(lowered_left);
                }
                if op != "**"
                    && matches!(
                        resolved_right_ty,
                        Type::Int | Type::LiteralInt(_)
                    )
                {
                    lowered_right = crate::stmt_support_emitter::checked_integer_codegen::exact_int_to_bigdecimal_expr(
                        $emitter,
                        lowered_right,
                    );
                } else if op != "**" && matches!(resolved_right_ty, Type::Decimal) {
                    lowered_right = lower_decimal_to_bigdecimal(lowered_right);
                }
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

            if op == "+" && matches!(resolved_result_ty, Type::TypeVar(_)) {
                return Ok(Some(crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec![
                        "__SifrAdd".to_string(),
                        "__sifr_add".to_string(),
                    ])),
                    args: vec![lowered_left, lowered_right],
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
                return Ok(Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_left))),
                    method: "pow".to_string(),
                    args: vec![crate::RustExpr::Cast {
                        expr: Box::new(lowered_right),
                        ty: crate::RustType::Named("u32".to_string()),
                    }],
                }));
            }
            return Ok(crate::stmt_support_emitter::binop_with_optional_operands(
                lowered_left,
                lowered_right,
                op,
                &resolved_left_ty,
                &resolved_right_ty,
                resolved_result_ty,
            ));
        }
    }};
}
