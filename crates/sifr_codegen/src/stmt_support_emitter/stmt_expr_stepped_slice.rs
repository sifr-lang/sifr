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
            let Some(start_lowered) = $emitter.lower_stmt_expr_for_ir(start_expr)? else {
                return Ok(None);
            };
            crate::RustExpr::Block {
                stmts: vec![crate::RustStmt::Let {
                    mutable: false,
                    name: "_sv".to_string(),
                    ty: None,
                    value: start_lowered,
                }],
                expr: Some(Box::new(crate::RustExpr::If {
                    cond: Box::new(crate::RustExpr::BinOp {
                        left: Box::new(crate::RustExpr::Ident("_sv".to_string())),
                        op: "<".to_string(),
                        right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(0))),
                    }),
                    then_expr: Box::new(crate::RustExpr::Cast {
                        expr: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                crate::RustExpr::BinOp {
                                    left: Box::new(crate::RustExpr::Ident("_len".to_string())),
                                    op: "+".to_string(),
                                    right: Box::new(crate::RustExpr::Ident("_sv".to_string())),
                                },
                            ))),
                            method: "max".to_string(),
                            args: vec![crate::RustExpr::Literal(crate::RustLiteral::Int(0))],
                        }),
                        ty: crate::RustType::Named("usize".to_string()),
                    }),
                    else_expr: Some(Box::new(crate::RustExpr::Cast {
                        expr: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Ident("_sv".to_string())),
                            method: "min".to_string(),
                            args: vec![crate::RustExpr::Ident("_len".to_string())],
                        }),
                        ty: crate::RustType::Named("usize".to_string()),
                    })),
                })),
            }
        } else {
            crate::RustExpr::If {
                cond: Box::new(crate::RustExpr::BinOp {
                    left: Box::new(crate::RustExpr::Ident("_step".to_string())),
                    op: ">".to_string(),
                    right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(0))),
                }),
                then_expr: Box::new(crate::RustExpr::Cast {
                    expr: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(0))),
                    ty: crate::RustType::Named("usize".to_string()),
                }),
                else_expr: Some(Box::new(crate::RustExpr::Cast {
                    expr: Box::new(crate::RustExpr::Paren(Box::new(crate::RustExpr::BinOp {
                        left: Box::new(crate::RustExpr::Ident("_len".to_string())),
                        op: "-".to_string(),
                        right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(1))),
                    }))),
                    ty: crate::RustType::Named("usize".to_string()),
                })),
            }
        };

        let lowered_stop = if let Some(stop_expr) = $stop {
            let Some(stop_lowered) = $emitter.lower_stmt_expr_for_ir(stop_expr)? else {
                return Ok(None);
            };
            crate::RustExpr::Block {
                stmts: vec![crate::RustStmt::Let {
                    mutable: false,
                    name: "_ev".to_string(),
                    ty: None,
                    value: stop_lowered,
                }],
                expr: Some(Box::new(crate::RustExpr::If {
                    cond: Box::new(crate::RustExpr::BinOp {
                        left: Box::new(crate::RustExpr::Ident("_ev".to_string())),
                        op: "<".to_string(),
                        right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(0))),
                    }),
                    then_expr: Box::new(crate::RustExpr::Cast {
                        expr: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                crate::RustExpr::BinOp {
                                    left: Box::new(crate::RustExpr::Ident("_len".to_string())),
                                    op: "+".to_string(),
                                    right: Box::new(crate::RustExpr::Ident("_ev".to_string())),
                                },
                            ))),
                            method: "max".to_string(),
                            args: vec![crate::RustExpr::Literal(crate::RustLiteral::Int(0))],
                        }),
                        ty: crate::RustType::Named("usize".to_string()),
                    }),
                    else_expr: Some(Box::new(crate::RustExpr::Cast {
                        expr: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Ident("_ev".to_string())),
                            method: "min".to_string(),
                            args: vec![crate::RustExpr::Ident("_len".to_string())],
                        }),
                        ty: crate::RustType::Named("usize".to_string()),
                    })),
                })),
            }
        } else {
            crate::RustExpr::If {
                cond: Box::new(crate::RustExpr::BinOp {
                    left: Box::new(crate::RustExpr::Ident("_step".to_string())),
                    op: ">".to_string(),
                    right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(0))),
                }),
                then_expr: Box::new(crate::RustExpr::Cast {
                    expr: Box::new(crate::RustExpr::Ident("_len".to_string())),
                    ty: crate::RustType::Named("usize".to_string()),
                }),
                else_expr: Some(Box::new(crate::RustExpr::Path(vec![
                    "usize".to_string(),
                    "MAX".to_string(),
                ]))),
            }
        };

        return match crate::resolve_alias_type_for_plain_call($object.ty()) {
            Type::List(_) | Type::Bytes => {
                let copy_slice_elements =
                    match crate::resolve_alias_type_for_plain_call($object.ty()) {
                        Type::Bytes => true,
                        Type::List(element_ty) => {
                            crate::helpers::is_copy_type_for_codegen(element_ty.as_ref())
                        }
                        _ => false,
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
                            value: crate::RustExpr::Cast {
                                expr: Box::new(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Ident("_v".to_string())),
                                    method: "len".to_string(),
                                    args: vec![],
                                }),
                                ty: crate::RustType::I64,
                            },
                        },
                        crate::RustStmt::Let {
                            mutable: false,
                            name: "_step".to_string(),
                            ty: None,
                            value: lowered_step,
                        },
                        crate::RustStmt::Let {
                            mutable: false,
                            name: "_start".to_string(),
                            ty: None,
                            value: lowered_start,
                        },
                        crate::RustStmt::Let {
                            mutable: false,
                            name: "_stop".to_string(),
                            ty: None,
                            value: lowered_stop,
                        },
                        crate::RustStmt::Let {
                            mutable: true,
                            name: "_result".to_string(),
                            ty: None,
                            value: crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec![
                                    "Vec".to_string(),
                                    "new".to_string(),
                                ])),
                                args: vec![],
                            },
                        },
                        crate::RustStmt::If {
                            cond: crate::RustExpr::BinOp {
                                left: Box::new(crate::RustExpr::Ident("_step".to_string())),
                                op: ">".to_string(),
                                right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(
                                    0,
                                ))),
                            },
                            then_body: vec![
                                crate::RustStmt::Let {
                                    mutable: true,
                                    name: "_i".to_string(),
                                    ty: None,
                                    value: crate::RustExpr::Ident("_start".to_string()),
                                },
                                crate::RustStmt::While {
                                    cond: crate::RustExpr::BinOp {
                                        left: Box::new(crate::RustExpr::Ident("_i".to_string())),
                                        op: "<".to_string(),
                                        right: Box::new(crate::RustExpr::Ident(
                                            "_stop".to_string(),
                                        )),
                                    },
                                    body: vec![
                                        crate::RustStmt::IfLet {
                                            pattern: "Some(_el)".to_string(),
                                            expr: crate::RustExpr::MethodCall {
                                                receiver: Box::new(crate::RustExpr::Ident(
                                                    "_v".to_string(),
                                                )),
                                                method: "get".to_string(),
                                                args: vec![crate::RustExpr::Ident(
                                                    "_i".to_string(),
                                                )],
                                            },
                                            then_body: vec![crate::RustStmt::Expr(
                                                crate::RustExpr::MethodCall {
                                                    receiver: Box::new(crate::RustExpr::Ident(
                                                        "_result".to_string(),
                                                    )),
                                                    method: "push".to_string(),
                                                    args: vec![if copy_slice_elements {
                                                        crate::RustExpr::Deref(Box::new(
                                                            crate::RustExpr::Ident(
                                                                "_el".to_string(),
                                                            ),
                                                        ))
                                                    } else {
                                                        crate::RustExpr::Clone(Box::new(
                                                            crate::RustExpr::Ident(
                                                                "_el".to_string(),
                                                            ),
                                                        ))
                                                    }],
                                                },
                                            )],
                                            else_body: None,
                                        },
                                        crate::RustStmt::AugAssign {
                                            target: crate::RustExpr::Ident("_i".to_string()),
                                            op: "+".to_string(),
                                            value: crate::RustExpr::Cast {
                                                expr: Box::new(crate::RustExpr::Ident(
                                                    "_step".to_string(),
                                                )),
                                                ty: crate::RustType::Named("usize".to_string()),
                                            },
                                        },
                                    ],
                                },
                            ],
                            else_body: Some(vec![
                                crate::RustStmt::Let {
                                    mutable: true,
                                    name: "_i".to_string(),
                                    ty: None,
                                    value: crate::RustExpr::Cast {
                                        expr: Box::new(crate::RustExpr::Ident(
                                            "_start".to_string(),
                                        )),
                                        ty: crate::RustType::I64,
                                    },
                                },
                                crate::RustStmt::Let {
                                    mutable: false,
                                    name: "_stop_i".to_string(),
                                    ty: None,
                                    value: crate::RustExpr::Cast {
                                        expr: Box::new(crate::RustExpr::Ident("_stop".to_string())),
                                        ty: crate::RustType::I64,
                                    },
                                },
                                crate::RustStmt::While {
                                    cond: crate::RustExpr::BinOp {
                                        left: Box::new(crate::RustExpr::Ident("_i".to_string())),
                                        op: ">".to_string(),
                                        right: Box::new(crate::RustExpr::Ident(
                                            "_stop_i".to_string(),
                                        )),
                                    },
                                    body: vec![
                                        crate::RustStmt::If {
                                            cond: crate::RustExpr::BinOp {
                                                left: Box::new(crate::RustExpr::Ident(
                                                    "_i".to_string(),
                                                )),
                                                op: ">=".to_string(),
                                                right: Box::new(crate::RustExpr::Literal(
                                                    crate::RustLiteral::Int(0),
                                                )),
                                            },
                                            then_body: vec![crate::RustStmt::IfLet {
                                                pattern: "Some(_el)".to_string(),
                                                expr: crate::RustExpr::MethodCall {
                                                    receiver: Box::new(crate::RustExpr::Ident(
                                                        "_v".to_string(),
                                                    )),
                                                    method: "get".to_string(),
                                                    args: vec![crate::RustExpr::Cast {
                                                        expr: Box::new(crate::RustExpr::Ident(
                                                            "_i".to_string(),
                                                        )),
                                                        ty: crate::RustType::Named(
                                                            "usize".to_string(),
                                                        ),
                                                    }],
                                                },
                                                then_body: vec![crate::RustStmt::Expr(
                                                    crate::RustExpr::MethodCall {
                                                        receiver: Box::new(crate::RustExpr::Ident(
                                                            "_result".to_string(),
                                                        )),
                                                        method: "push".to_string(),
                                                        args: vec![if copy_slice_elements {
                                                            crate::RustExpr::Deref(Box::new(
                                                                crate::RustExpr::Ident(
                                                                    "_el".to_string(),
                                                                ),
                                                            ))
                                                        } else {
                                                            crate::RustExpr::Clone(Box::new(
                                                                crate::RustExpr::Ident(
                                                                    "_el".to_string(),
                                                                ),
                                                            ))
                                                        }],
                                                    },
                                                )],
                                                else_body: None,
                                            }],
                                            else_body: None,
                                        },
                                        crate::RustStmt::AugAssign {
                                            target: crate::RustExpr::Ident("_i".to_string()),
                                            op: "+".to_string(),
                                            value: crate::RustExpr::Ident("_step".to_string()),
                                        },
                                    ],
                                },
                            ]),
                        },
                    ],
                    expr: Some(Box::new(crate::RustExpr::Ident("_result".to_string()))),
                }))
            }
            Type::Str => Ok(Some(crate::RustExpr::Block {
                stmts: vec![
                    crate::RustStmt::Let {
                        mutable: false,
                        name: "_s".to_string(),
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
                        value: crate::RustExpr::Cast {
                            expr: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Ident("_s".to_string())),
                                    method: "chars".to_string(),
                                    args: vec![],
                                }),
                                method: "count".to_string(),
                                args: vec![],
                            }),
                            ty: crate::RustType::I64,
                        },
                    },
                    crate::RustStmt::Let {
                        mutable: false,
                        name: "_step".to_string(),
                        ty: None,
                        value: lowered_step,
                    },
                    crate::RustStmt::Let {
                        mutable: false,
                        name: "_start".to_string(),
                        ty: None,
                        value: lowered_start,
                    },
                    crate::RustStmt::Let {
                        mutable: false,
                        name: "_stop".to_string(),
                        ty: None,
                        value: lowered_stop,
                    },
                    crate::RustStmt::Let {
                        mutable: true,
                        name: "_result".to_string(),
                        ty: None,
                        value: crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec![
                                "String".to_string(),
                                "new".to_string(),
                            ])),
                            args: vec![],
                        },
                    },
                    crate::RustStmt::If {
                        cond: crate::RustExpr::BinOp {
                            left: Box::new(crate::RustExpr::Ident("_step".to_string())),
                            op: ">".to_string(),
                            right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(0))),
                        },
                        then_body: vec![
                            crate::RustStmt::Let {
                                mutable: true,
                                name: "_i".to_string(),
                                ty: None,
                                value: crate::RustExpr::Ident("_start".to_string()),
                            },
                            crate::RustStmt::While {
                                cond: crate::RustExpr::BinOp {
                                    left: Box::new(crate::RustExpr::Ident("_i".to_string())),
                                    op: "<".to_string(),
                                    right: Box::new(crate::RustExpr::Ident("_stop".to_string())),
                                },
                                body: vec![
                                    crate::RustStmt::IfLet {
                                        pattern: "Some(_ch)".to_string(),
                                        expr: crate::RustExpr::MethodCall {
                                            receiver: Box::new(crate::RustExpr::MethodCall {
                                                receiver: Box::new(crate::RustExpr::Ident(
                                                    "_s".to_string(),
                                                )),
                                                method: "chars".to_string(),
                                                args: vec![],
                                            }),
                                            method: "nth".to_string(),
                                            args: vec![crate::RustExpr::Ident("_i".to_string())],
                                        },
                                        then_body: vec![crate::RustStmt::Expr(
                                            crate::RustExpr::MethodCall {
                                                receiver: Box::new(crate::RustExpr::Ident(
                                                    "_result".to_string(),
                                                )),
                                                method: "push".to_string(),
                                                args: vec![crate::RustExpr::Ident(
                                                    "_ch".to_string(),
                                                )],
                                            },
                                        )],
                                        else_body: None,
                                    },
                                    crate::RustStmt::AugAssign {
                                        target: crate::RustExpr::Ident("_i".to_string()),
                                        op: "+".to_string(),
                                        value: crate::RustExpr::Cast {
                                            expr: Box::new(crate::RustExpr::Ident(
                                                "_step".to_string(),
                                            )),
                                            ty: crate::RustType::Named("usize".to_string()),
                                        },
                                    },
                                ],
                            },
                        ],
                        else_body: Some(vec![
                            crate::RustStmt::Let {
                                mutable: true,
                                name: "_i".to_string(),
                                ty: None,
                                value: crate::RustExpr::Cast {
                                    expr: Box::new(crate::RustExpr::Ident("_start".to_string())),
                                    ty: crate::RustType::I64,
                                },
                            },
                            crate::RustStmt::Let {
                                mutable: false,
                                name: "_stop_i".to_string(),
                                ty: None,
                                value: crate::RustExpr::Cast {
                                    expr: Box::new(crate::RustExpr::Ident("_stop".to_string())),
                                    ty: crate::RustType::I64,
                                },
                            },
                            crate::RustStmt::While {
                                cond: crate::RustExpr::BinOp {
                                    left: Box::new(crate::RustExpr::Ident("_i".to_string())),
                                    op: ">".to_string(),
                                    right: Box::new(crate::RustExpr::Ident("_stop_i".to_string())),
                                },
                                body: vec![
                                    crate::RustStmt::If {
                                        cond: crate::RustExpr::BinOp {
                                            left: Box::new(crate::RustExpr::Ident(
                                                "_i".to_string(),
                                            )),
                                            op: ">=".to_string(),
                                            right: Box::new(crate::RustExpr::Literal(
                                                crate::RustLiteral::Int(0),
                                            )),
                                        },
                                        then_body: vec![crate::RustStmt::IfLet {
                                            pattern: "Some(_ch)".to_string(),
                                            expr: crate::RustExpr::MethodCall {
                                                receiver: Box::new(crate::RustExpr::MethodCall {
                                                    receiver: Box::new(crate::RustExpr::Ident(
                                                        "_s".to_string(),
                                                    )),
                                                    method: "chars".to_string(),
                                                    args: vec![],
                                                }),
                                                method: "nth".to_string(),
                                                args: vec![crate::RustExpr::Cast {
                                                    expr: Box::new(crate::RustExpr::Ident(
                                                        "_i".to_string(),
                                                    )),
                                                    ty: crate::RustType::Named("usize".to_string()),
                                                }],
                                            },
                                            then_body: vec![crate::RustStmt::Expr(
                                                crate::RustExpr::MethodCall {
                                                    receiver: Box::new(crate::RustExpr::Ident(
                                                        "_result".to_string(),
                                                    )),
                                                    method: "push".to_string(),
                                                    args: vec![crate::RustExpr::Ident(
                                                        "_ch".to_string(),
                                                    )],
                                                },
                                            )],
                                            else_body: None,
                                        }],
                                        else_body: None,
                                    },
                                    crate::RustStmt::AugAssign {
                                        target: crate::RustExpr::Ident("_i".to_string()),
                                        op: "+".to_string(),
                                        value: crate::RustExpr::Ident("_step".to_string()),
                                    },
                                ],
                            },
                        ]),
                    },
                ],
                expr: Some(Box::new(crate::RustExpr::Ident("_result".to_string()))),
            })),
            _ => Ok(None),
        };
    }};
}
