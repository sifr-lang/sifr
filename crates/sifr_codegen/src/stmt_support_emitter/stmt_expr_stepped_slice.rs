macro_rules! stmt_expr_stepped_slice {
    ($emitter:ident, $object:ident, $start:ident, $stop:ident, $step_expr:ident, $lowered_object:ident) => {{
        let Some(lowered_step) = $emitter.lower_stmt_expr_for_ir($step_expr)? else {
            return Ok(None);
        };

        let const_step_value = match $step_expr.as_ref() {
            HirExpr::IntLiteral(value) => Some(*value),
            HirExpr::UnaryOp { op, operand, .. } if op == "-" => {
                if let HirExpr::IntLiteral(value) = operand.as_ref() {
                    Some(-*value)
                } else {
                    None
                }
            }
            HirExpr::UnaryOp { op, operand, .. } if op == "+" => {
                if let HirExpr::IntLiteral(value) = operand.as_ref() {
                    Some(*value)
                } else {
                    None
                }
            }
            _ => None,
        };

        if $start.is_none() && $stop.is_none() {
            if let (Type::Str, Some(step_value)) = (
                crate::resolve_alias_type_for_plain_call($object.ty()),
                const_step_value,
            ) {
                if let Some(magnitude) = step_value.checked_abs() {
                    if magnitude > 0 {
                        let mut iter_expr = crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Ident("_s".to_string())),
                            method: "chars".to_string(),
                            args: vec![],
                        };
                        if step_value < 0 {
                            iter_expr = crate::RustExpr::MethodCall {
                                receiver: Box::new(iter_expr),
                                method: "rev".to_string(),
                                args: vec![],
                            };
                        }
                        if magnitude > 1 {
                            iter_expr = crate::RustExpr::MethodCall {
                                receiver: Box::new(iter_expr),
                                method: "step_by".to_string(),
                                args: vec![crate::RustExpr::Cast {
                                    expr: Box::new(crate::RustExpr::Literal(
                                        crate::RustLiteral::Int(magnitude),
                                    )),
                                    ty: crate::RustType::Named("usize".to_string()),
                                }],
                            };
                        }
                        return Ok(Some(crate::RustExpr::Block {
                            stmts: vec![crate::RustStmt::Let {
                                mutable: false,
                                name: "_s".to_string(),
                                ty: None,
                                value: crate::RustExpr::Ref {
                                    mutable: false,
                                    expr: Box::new(crate::RustExpr::Paren(Box::new(
                                        $lowered_object,
                                    ))),
                                },
                            }],
                            expr: Some(Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(iter_expr),
                                method: "collect::<String>".to_string(),
                                args: vec![],
                            })),
                        }));
                    }
                }
            }
        }

        let lowered_start = if let Some(start_expr) = $start {
            let Some(value) = $emitter.lower_stmt_expr_for_ir(start_expr)? else {
                return Ok(None);
            };
            crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                args: vec![$emitter.coerce_expr_to_sifr_int_comparison_operand(value)],
            }
        } else {
            crate::RustExpr::Literal(crate::RustLiteral::None)
        };
        let lowered_stop = if let Some(stop_expr) = $stop {
            let Some(value) = $emitter.lower_stmt_expr_for_ir(stop_expr)? else {
                return Ok(None);
            };
            crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                args: vec![$emitter.coerce_expr_to_sifr_int_comparison_operand(value)],
            }
        } else {
            crate::RustExpr::Literal(crate::RustLiteral::None)
        };

        let indices_expr = |len_name: &str| crate::RustExpr::FnCall {
            func: Box::new(crate::RustExpr::Path(vec![
                "sifr_runtime".to_string(),
                "SifrSliceIndices".to_string(),
                "new_known_nonzero".to_string(),
            ])),
            args: vec![
                crate::RustExpr::Ident(len_name.to_string()),
                lowered_start.clone(),
                lowered_stop.clone(),
                $emitter.coerce_expr_to_sifr_int_comparison_operand(lowered_step.clone()),
            ],
        };

        return match crate::resolve_alias_type_for_plain_call($object.ty()) {
            Type::List(_) | Type::Bytes => {
                let copy_elements = match crate::resolve_alias_type_for_plain_call($object.ty()) {
                    Type::Bytes => true,
                    Type::List(element_ty) => {
                        crate::helpers::is_copy_type_for_codegen(element_ty.as_ref())
                    }
                    _ => false,
                };
                let indexed = crate::RustExpr::Index {
                    expr: Box::new(crate::RustExpr::Ident("_v".to_string())),
                    index: Box::new(crate::RustExpr::Ident("_i".to_string())),
                };
                let yielded = if copy_elements {
                    indexed
                } else {
                    crate::RustExpr::Clone(Box::new(indexed))
                };
                Ok(Some(crate::RustExpr::Block {
                    stmts: vec![
                        crate::RustStmt::Let {
                            mutable: false,
                            name: "_v".to_string(),
                            ty: None,
                            value: crate::RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(crate::RustExpr::Paren(Box::new($lowered_object))),
                            },
                        },
                        crate::RustStmt::Let {
                            mutable: false,
                            name: "_len".to_string(),
                            ty: None,
                            value: crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Ident("_v".to_string())),
                                method: "len".to_string(),
                                args: vec![],
                            },
                        },
                    ],
                    expr: Some(Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(indices_expr("_len")),
                            method: "map".to_string(),
                            args: vec![crate::RustExpr::Closure {
                                params: vec![crate::RustParam::Named {
                                    name: "_i".to_string(),
                                    ty: crate::RustType::Named("_".to_string()),
                                }],
                                body: Box::new(yielded),
                                is_move: false,
                            }],
                        }),
                        method: "collect::<Vec<_>>".to_string(),
                        args: vec![],
                    })),
                }))
            }
            Type::Str => Ok(Some(crate::RustExpr::Block {
                stmts: vec![
                    crate::RustStmt::Let {
                        mutable: false,
                        name: "_chars".to_string(),
                        ty: None,
                        value: crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                    $lowered_object,
                                ))),
                                method: "chars".to_string(),
                                args: vec![],
                            }),
                            method: "collect::<Vec<_>>".to_string(),
                            args: vec![],
                        },
                    },
                    crate::RustStmt::Let {
                        mutable: false,
                        name: "_len".to_string(),
                        ty: None,
                        value: crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Ident("_chars".to_string())),
                            method: "len".to_string(),
                            args: vec![],
                        },
                    },
                ],
                expr: Some(Box::new(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(indices_expr("_len")),
                        method: "map".to_string(),
                        args: vec![crate::RustExpr::Closure {
                            params: vec![crate::RustParam::Named {
                                name: "_i".to_string(),
                                ty: crate::RustType::Named("_".to_string()),
                            }],
                            body: Box::new(crate::RustExpr::Index {
                                expr: Box::new(crate::RustExpr::Ident("_chars".to_string())),
                                index: Box::new(crate::RustExpr::Ident("_i".to_string())),
                            }),
                            is_move: false,
                        }],
                    }),
                    method: "collect::<String>".to_string(),
                    args: vec![],
                })),
            })),
            _ => Ok(None),
        };
    }};
}
