use super::{
    registry_option_inner_type, registry_uses_debug_display_format, HirExpr, RustEmitter, RustExpr,
    Type,
};
impl RustEmitter {
    pub(crate) fn try_lower_registry_numeric_builtin_call_expr(
        &mut self,
        func: &str,
        args: &[HirExpr],
        _result_ty: Option<&Type>,
    ) -> Option<crate::RustExpr> {
        match func {
            "abs" if args.len() == 1 => {
                let lowered = self.try_lower_registry_expr_strict(&args[0])?;
                let lowered = if matches!(
                    crate::resolve_alias_type_for_plain_call(args[0].ty()),
                    Type::FixedInt(fixed) if fixed.supports_current_int_builtin_widening()
                ) {
                    crate::RustExpr::Cast {
                        expr: Box::new(lowered),
                        ty: crate::RustType::I64,
                    }
                } else {
                    lowered
                };
                Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered))),
                    method: "abs".to_string(),
                    args: vec![],
                })
            }
            "ord" if args.len() == 1 => {
                let lowered = self.try_lower_registry_expr_strict(&args[0])?;
                Some(RustExpr::Block {
                    stmts: vec![crate::RustStmt::Let {
                        mutable: false,
                        name: "__sifr_ord_chars".to_string(),
                        ty: None,
                        value: RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Paren(Box::new(lowered))),
                                method: "chars".to_string(),
                                args: vec![],
                            }),
                            method: "collect::<Vec<char>>".to_string(),
                            args: vec![],
                        },
                    }],
                    expr: Some(Box::new(RustExpr::If {
                        cond: Box::new(RustExpr::BinOp {
                            left: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("__sifr_ord_chars".to_string())),
                                method: "len".to_string(),
                                args: vec![],
                            }),
                            op: "==".to_string(),
                            right: Box::new(RustExpr::Literal(crate::RustLiteral::Int(1))),
                        }),
                        then_expr: Box::new(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                            args: vec![RustExpr::Cast {
                                expr: Box::new(RustExpr::Index {
                                    expr: Box::new(RustExpr::Ident("__sifr_ord_chars".to_string())),
                                    index: Box::new(RustExpr::Literal(crate::RustLiteral::Int(0))),
                                }),
                                ty: crate::RustType::I64,
                            }],
                        }),
                        else_expr: Some(Box::new(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
                            args: vec![RustExpr::StructInit {
                                name: "ValueError".to_string(),
                                fields: vec![(
                                    "message".to_string(),
                                    RustExpr::Literal(crate::RustLiteral::Str(
                                        "ord() expected a string of length 1".to_string(),
                                    )),
                                )],
                            }],
                        })),
                    })),
                })
            }
            "chr" if args.len() == 1 => {
                let lowered = self.try_lower_registry_expr_strict(&args[0])?;
                Some(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec![
                                "std".to_string(),
                                "char".to_string(),
                                "from_u32".to_string(),
                            ])),
                            args: vec![RustExpr::Cast {
                                expr: Box::new(RustExpr::Paren(Box::new(lowered))),
                                ty: crate::RustType::Named("u32".to_string()),
                            }],
                        }),
                        method: "map".to_string(),
                        args: vec![RustExpr::Closure {
                            params: vec![crate::RustParam::Named {
                                name: "__sifr_chr".to_string(),
                                ty: crate::RustType::Named("_".to_string()),
                            }],
                            body: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("__sifr_chr".to_string())),
                                method: "to_string".to_string(),
                                args: vec![],
                            }),
                            is_move: false,
                        }],
                    }),
                    method: "ok_or_else".to_string(),
                    args: vec![RustExpr::Closure {
                        params: vec![],
                        body: Box::new(RustExpr::StructInit {
                            name: "ValueError".to_string(),
                            fields: vec![(
                                "message".to_string(),
                                RustExpr::Literal(crate::RustLiteral::Str(
                                    "chr() arg not in range(0x110000)".to_string(),
                                )),
                            )],
                        }),
                        is_move: false,
                    }],
                })
            }
            "round" if args.len() == 1 => {
                let lowered = self.try_lower_registry_expr_strict(&args[0])?;
                Some(crate::RustExpr::Cast {
                    expr: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered))),
                        method: "round".to_string(),
                        args: vec![],
                    }),
                    ty: crate::RustType::I64,
                })
            }
            "repr" if args.len() == 1 => {
                let lowered = self.try_lower_registry_expr_strict(&args[0])?;
                if registry_option_inner_type(args[0].ty()).is_some() {
                    Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered))),
                        method: "map_or".to_string(),
                        args: vec![
                            crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Literal(
                                    crate::RustLiteral::Str("None".to_string()),
                                )),
                                method: "to_string".to_string(),
                                args: vec![],
                            },
                            crate::RustExpr::Closure {
                                params: vec![crate::RustParam::Named {
                                    name: "__v".to_string(),
                                    ty: crate::RustType::Named("_".to_string()),
                                }],
                                body: Box::new(crate::RustExpr::FormatMacro {
                                    name: "format".to_string(),
                                    format_str: "{:?}".to_string(),
                                    args: vec![crate::RustExpr::Ident("__v".to_string())],
                                }),
                                is_move: false,
                            },
                        ],
                    })
                } else {
                    Some(crate::RustExpr::FormatMacro {
                        name: "format".to_string(),
                        format_str: "{:?}".to_string(),
                        args: vec![lowered],
                    })
                }
            }
            "max" | "min" if args.len() >= 2 => {
                let mut lowered_args = Vec::with_capacity(args.len());
                for arg in args {
                    let mut lowered = self
                        .try_lower_registry_expr_strict(arg)
                        .or_else(|| self.lower_stmt_expr_for_ir(arg).ok().flatten())?;
                    if matches!(arg.ty().resolve_alias(), Type::Str)
                        && matches!(arg, HirExpr::Name { .. })
                    {
                        if matches!(arg, HirExpr::Name { name, .. } if self.borrowed_params.contains(name))
                        {
                            lowered = RustExpr::Paren(Box::new(RustExpr::Deref(Box::new(lowered))));
                        }
                        lowered = RustExpr::Clone(Box::new(lowered));
                    }
                    lowered_args.push(lowered);
                }

                let use_float_comparison = args.iter().any(|arg| {
                    matches!(
                        crate::resolve_alias_type_for_plain_call(arg.ty()),
                        Type::Float
                    )
                });
                let mut iter = lowered_args.into_iter();
                let mut reduced = iter.next()?;
                for next in iter {
                    reduced = if use_float_comparison {
                        crate::RustExpr::MethodCall {
                            receiver: Box::new(reduced),
                            method: func.to_string(),
                            args: vec![next],
                        }
                    } else {
                        crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec![
                                "std".to_string(),
                                "cmp".to_string(),
                                func.to_string(),
                            ])),
                            args: vec![reduced, next],
                        }
                    };
                }

                Some(reduced)
            }
            "pow" if args.len() == 2 => {
                let base = self.try_lower_registry_expr_strict(&args[0])?;
                let exp = self.try_lower_registry_expr_strict(&args[1])?;
                if matches!(
                    crate::resolve_alias_type_for_plain_call(args[0].ty()),
                    Type::Int | Type::LiteralInt(_)
                ) && matches!(
                    crate::resolve_alias_type_for_plain_call(args[1].ty()),
                    Type::Int | Type::LiteralInt(_)
                ) {
                    Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(base),
                        method: "pow".to_string(),
                        args: vec![crate::RustExpr::Cast {
                            expr: Box::new(exp),
                            ty: crate::RustType::Named("u32".to_string()),
                        }],
                    })
                } else {
                    Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Cast {
                            expr: Box::new(base),
                            ty: crate::RustType::F64,
                        }),
                        method: "powf".to_string(),
                        args: vec![crate::RustExpr::Cast {
                            expr: Box::new(exp),
                            ty: crate::RustType::F64,
                        }],
                    })
                }
            }
            "bool" if args.len() == 1 => {
                let lowered = self.try_lower_registry_expr_strict(&args[0])?;
                match crate::resolve_alias_type_for_plain_call(args[0].ty()) {
                    Type::Int | Type::LiteralInt(_) => Some(crate::RustExpr::BinOp {
                        left: Box::new(lowered),
                        op: "!=".to_string(),
                        right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(0))),
                    }),
                    Type::Float => Some(crate::RustExpr::BinOp {
                        left: Box::new(lowered),
                        op: "!=".to_string(),
                        right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Float(0.0))),
                    }),
                    Type::Str | Type::Bytes | Type::List(_) | Type::Dict(_, _) => {
                        Some(crate::RustExpr::UnaryOp {
                            op: "!".to_string(),
                            operand: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(lowered),
                                method: "is_empty".to_string(),
                                args: vec![],
                            }),
                        })
                    }
                    Type::Tuple(elems) => Some(crate::RustExpr::Literal(crate::RustLiteral::Bool(
                        !elems.is_empty(),
                    ))),
                    Type::Bool => Some(lowered),
                    Type::None => Some(crate::RustExpr::Literal(crate::RustLiteral::Bool(false))),
                    _ => Some(lowered),
                }
            }
            "float" if args.len() == 1 => {
                let lowered = self.try_lower_registry_expr_strict(&args[0])?;
                match crate::resolve_alias_type_for_plain_call(args[0].ty()) {
                    Type::Int | Type::LiteralInt(_) => Some(crate::RustExpr::Cast {
                        expr: Box::new(lowered),
                        ty: crate::RustType::F64,
                    }),
                    Type::Str => Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered))),
                            method: "parse::<f64>".to_string(),
                            args: vec![],
                        }),
                        method: "map_err".to_string(),
                        args: vec![crate::RustExpr::Closure {
                            params: vec![crate::RustParam::Named {
                                name: "e".to_string(),
                                ty: crate::RustType::Named("_".to_string()),
                            }],
                            body: Box::new(crate::RustExpr::StructInit {
                                name: "ParseError".to_string(),
                                fields: vec![(
                                    "message".to_string(),
                                    crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::Ident("e".to_string())),
                                        method: "to_string".to_string(),
                                        args: vec![],
                                    },
                                )],
                            }),
                            is_move: false,
                        }],
                    }),
                    Type::Bool => Some(crate::RustExpr::If {
                        cond: Box::new(lowered),
                        then_expr: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Float(
                            1.0,
                        ))),
                        else_expr: Some(Box::new(crate::RustExpr::Literal(
                            crate::RustLiteral::Float(0.0),
                        ))),
                    }),
                    _ => Some(lowered),
                }
            }
            "int" if args.len() == 1 => {
                let lowered = self.try_lower_registry_expr_strict(&args[0])?;
                match crate::resolve_alias_type_for_plain_call(args[0].ty()) {
                    Type::Float => Some(crate::RustExpr::Cast {
                        expr: Box::new(lowered),
                        ty: crate::RustType::I64,
                    }),
                    Type::Str => Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered))),
                            method: "parse::<i64>".to_string(),
                            args: vec![],
                        }),
                        method: "map_err".to_string(),
                        args: vec![crate::RustExpr::Closure {
                            params: vec![crate::RustParam::Named {
                                name: "e".to_string(),
                                ty: crate::RustType::Named("_".to_string()),
                            }],
                            body: Box::new(crate::RustExpr::StructInit {
                                name: "ParseError".to_string(),
                                fields: vec![(
                                    "message".to_string(),
                                    crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::Ident("e".to_string())),
                                        method: "to_string".to_string(),
                                        args: vec![],
                                    },
                                )],
                            }),
                            is_move: false,
                        }],
                    }),
                    Type::Bool => Some(crate::RustExpr::If {
                        cond: Box::new(lowered),
                        then_expr: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(1))),
                        else_expr: Some(Box::new(crate::RustExpr::Literal(
                            crate::RustLiteral::Int(0),
                        ))),
                    }),
                    Type::FixedInt(_) => Some(crate::RustExpr::Cast {
                        expr: Box::new(lowered),
                        ty: crate::RustType::I64,
                    }),
                    Type::BigInt => Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec![
                                "i64".to_string(),
                                "try_from".to_string(),
                            ])),
                            args: vec![crate::RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(crate::RustExpr::Paren(Box::new(lowered))),
                            }],
                        }),
                        method: "map_err".to_string(),
                        args: vec![crate::RustExpr::Closure {
                            params: vec![crate::RustParam::Named {
                                name: "__e_ignored".to_string(),
                                ty: crate::RustType::Named("_".to_string()),
                            }],
                            body: Box::new(crate::RustExpr::StructInit {
                                name: "OverflowError".to_string(),
                                fields: vec![(
                                    "message".to_string(),
                                    crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::Literal(
                                            crate::RustLiteral::Str(
                                                "bigint value out of range for int".to_string(),
                                            ),
                                        )),
                                        method: "to_string".to_string(),
                                        args: vec![],
                                    },
                                )],
                            }),
                            is_move: false,
                        }],
                    }),
                    Type::Decimal => Some(crate::RustExpr::Block {
                        stmts: vec![crate::RustStmt::Let {
                            mutable: false,
                            name: "__decimal_bigint".to_string(),
                            ty: None,
                            value: crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec![
                                    "BigInt".to_string(),
                                    "from".to_string(),
                                ])),
                                args: vec![crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                            lowered,
                                        ))),
                                        method: "trunc".to_string(),
                                        args: vec![],
                                    }),
                                    method: "mantissa".to_string(),
                                    args: vec![],
                                }],
                            },
                        }],
                        expr: Some(Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec![
                                    "i64".to_string(),
                                    "try_from".to_string(),
                                ])),
                                args: vec![crate::RustExpr::Ref {
                                    mutable: false,
                                    expr: Box::new(crate::RustExpr::Ident(
                                        "__decimal_bigint".to_string(),
                                    )),
                                }],
                            }),
                            method: "map_err".to_string(),
                            args: vec![crate::RustExpr::Closure {
                                params: vec![crate::RustParam::Named {
                                    name: "__e_ignored".to_string(),
                                    ty: crate::RustType::Named("_".to_string()),
                                }],
                                body: Box::new(crate::RustExpr::StructInit {
                                    name: "DecimalConversionError".to_string(),
                                    fields: vec![(
                                        "message".to_string(),
                                        crate::RustExpr::MethodCall {
                                            receiver: Box::new(crate::RustExpr::Literal(
                                                crate::RustLiteral::Str(
                                                    "decimal value out of range for int"
                                                        .to_string(),
                                                ),
                                            )),
                                            method: "to_string".to_string(),
                                            args: vec![],
                                        },
                                    )],
                                }),
                                is_move: false,
                            }],
                        })),
                    }),
                    Type::BigDecimal => Some(crate::RustExpr::Block {
                        stmts: vec![crate::RustStmt::Let {
                            mutable: false,
                            name: "__decimal_bigint".to_string(),
                            ty: None,
                            value: crate::RustExpr::Field {
                                expr: Box::new(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                            lowered,
                                        ))),
                                        method: "with_scale".to_string(),
                                        args: vec![crate::RustExpr::Literal(
                                            crate::RustLiteral::Int(0),
                                        )],
                                    }),
                                    method: "into_bigint_and_scale".to_string(),
                                    args: vec![],
                                }),
                                field: "0".to_string(),
                            },
                        }],
                        expr: Some(Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec![
                                    "i64".to_string(),
                                    "try_from".to_string(),
                                ])),
                                args: vec![crate::RustExpr::Ref {
                                    mutable: false,
                                    expr: Box::new(crate::RustExpr::Ident(
                                        "__decimal_bigint".to_string(),
                                    )),
                                }],
                            }),
                            method: "map_err".to_string(),
                            args: vec![crate::RustExpr::Closure {
                                params: vec![crate::RustParam::Named {
                                    name: "__e_ignored".to_string(),
                                    ty: crate::RustType::Named("_".to_string()),
                                }],
                                body: Box::new(crate::RustExpr::StructInit {
                                    name: "DecimalConversionError".to_string(),
                                    fields: vec![(
                                        "message".to_string(),
                                        crate::RustExpr::MethodCall {
                                            receiver: Box::new(crate::RustExpr::Literal(
                                                crate::RustLiteral::Str(
                                                    "bigdecimal value out of range for int"
                                                        .to_string(),
                                                ),
                                            )),
                                            method: "to_string".to_string(),
                                            args: vec![],
                                        },
                                    )],
                                }),
                                is_move: false,
                            }],
                        })),
                    }),
                    _ => Some(lowered),
                }
            }
            "bigint" if args.len() == 1 => {
                let lowered = self.try_lower_registry_expr_strict(&args[0])?;
                match crate::resolve_alias_type_for_plain_call(args[0].ty()) {
                    Type::Int | Type::LiteralInt(_) | Type::BigInt => {
                        Some(crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec![
                                "BigInt".to_string(),
                                "from".to_string(),
                            ])),
                            args: vec![lowered],
                        })
                    }
                    Type::Decimal => Some(crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "BigInt".to_string(),
                            "from".to_string(),
                        ])),
                        args: vec![crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered))),
                                method: "trunc".to_string(),
                                args: vec![],
                            }),
                            method: "mantissa".to_string(),
                            args: vec![],
                        }],
                    }),
                    Type::BigDecimal => Some(crate::RustExpr::Field {
                        expr: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered))),
                                method: "with_scale".to_string(),
                                args: vec![crate::RustExpr::Literal(crate::RustLiteral::Int(0))],
                            }),
                            method: "into_bigint_and_scale".to_string(),
                            args: vec![],
                        }),
                        field: "0".to_string(),
                    }),
                    _ => None,
                }
            }
            "Decimal" if args.len() == 1 => {
                let lowered = self.try_lower_registry_expr_strict(&args[0])?;
                match crate::resolve_alias_type_for_plain_call(args[0].ty()) {
                    Type::Int | Type::LiteralInt(_) => Some(crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "Decimal".to_string(),
                            "from".to_string(),
                        ])),
                        args: vec![lowered],
                    }),
                    Type::Decimal => Some(lowered),
                    Type::Str | Type::LiteralStr(_) => Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec![
                                "Decimal".to_string(),
                                "from_str_exact".to_string(),
                            ])),
                            args: vec![crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered))),
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
                    }),
                    Type::BigInt | Type::BigDecimal => Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec![
                                    "Decimal".to_string(),
                                    "from_str_exact".to_string(),
                                ])),
                                args: vec![crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                            lowered,
                                        ))),
                                        method: "to_string".to_string(),
                                        args: vec![],
                                    }),
                                    method: "as_str".to_string(),
                                    args: vec![],
                                }],
                            }),
                            method: "map_err".to_string(),
                            args: vec![crate::RustExpr::Closure {
                                params: vec![crate::RustParam::Named {
                                    name: "e".to_string(),
                                    ty: crate::RustType::Named("_".to_string()),
                                }],
                                body: Box::new(crate::RustExpr::StructInit {
                                    name: "DecimalConversionError".to_string(),
                                    fields: vec![(
                                        "message".to_string(),
                                        crate::RustExpr::MethodCall {
                                            receiver: Box::new(crate::RustExpr::Ident(
                                                "e".to_string(),
                                            )),
                                            method: "to_string".to_string(),
                                            args: vec![],
                                        },
                                    )],
                                }),
                                is_move: false,
                            }],
                        }),
                        method: "map".to_string(),
                        args: vec![crate::RustExpr::Closure {
                            params: vec![crate::RustParam::Named {
                                name: "__v".to_string(),
                                ty: crate::RustType::Named("_".to_string()),
                            }],
                            body: Box::new(crate::RustExpr::Ident("__v".to_string())),
                            is_move: false,
                        }],
                    }),
                    _ => None,
                }
            }
            "BigDecimal" if args.len() == 1 => {
                let lowered = self.try_lower_registry_expr_strict(&args[0])?;
                match crate::resolve_alias_type_for_plain_call(args[0].ty()) {
                    Type::Int | Type::LiteralInt(_) | Type::BigInt => {
                        Some(crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec![
                                "BigDecimal".to_string(),
                                "from".to_string(),
                            ])),
                            args: vec![lowered],
                        })
                    }
                    Type::Decimal | Type::Str | Type::LiteralStr(_) => {
                        let source = match crate::resolve_alias_type_for_plain_call(args[0].ty()) {
                            Type::Decimal => crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered))),
                                method: "to_string".to_string(),
                                args: vec![],
                            },
                            _ => lowered,
                        };
                        Some(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Paren(Box::new(source))),
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
                        })
                    }
                    Type::BigDecimal => Some(lowered),
                    _ => None,
                }
            }
            "str" if args.len() == 1 => {
                let lowered = self.try_lower_registry_expr_strict(&args[0])?;
                let call_return_ty = if let HirExpr::Call { func, .. } = &args[0] {
                    self.func_signatures.get(func).map(|(_, ret)| ret.clone())
                } else {
                    None
                };
                let str_arg_ty = call_return_ty.as_ref().unwrap_or_else(|| args[0].ty());
                if let Some(inner) = registry_option_inner_type(str_arg_ty) {
                    let format_str = if registry_uses_debug_display_format(inner) {
                        "{:?}".to_string()
                    } else {
                        "{}".to_string()
                    };
                    Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered))),
                        method: "map_or".to_string(),
                        args: vec![
                            crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Literal(
                                    crate::RustLiteral::Str("None".to_string()),
                                )),
                                method: "to_string".to_string(),
                                args: vec![],
                            },
                            crate::RustExpr::Closure {
                                params: vec![crate::RustParam::Named {
                                    name: "__v".to_string(),
                                    ty: crate::RustType::Named("_".to_string()),
                                }],
                                body: Box::new(crate::RustExpr::FormatMacro {
                                    name: "format".to_string(),
                                    format_str,
                                    args: vec![crate::RustExpr::Ident("__v".to_string())],
                                }),
                                is_move: false,
                            },
                        ],
                    })
                } else {
                    Some(crate::RustExpr::FormatMacro {
                        name: "format".to_string(),
                        format_str: if registry_uses_debug_display_format(str_arg_ty) {
                            "{:?}".to_string()
                        } else {
                            "{}".to_string()
                        },
                        args: vec![lowered],
                    })
                }
            }
            _ => None,
        }
    }
}
