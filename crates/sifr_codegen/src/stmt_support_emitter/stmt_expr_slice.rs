macro_rules! stmt_expr_slice {
    ($emitter:ident, $expr:ident) => {{
        if let HirExpr::Slice {
            object,
            start,
            stop,
            step,
            ..
        } = $expr
        {
            let Some(lowered_object) = $emitter.lower_stmt_expr_for_ir(object)? else {
                return Ok(None);
            };
            if let Some(step_expr) = step {
                let Some(lowered_step) = $emitter.lower_stmt_expr_for_ir(step_expr)? else {
                    return Ok(None);
                };

                let lowered_start = if let Some(start_expr) = start {
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
                                right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(
                                    0,
                                ))),
                            }),
                            then_expr: Box::new(crate::RustExpr::Cast {
                                expr: Box::new(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                        crate::RustExpr::BinOp {
                                            left: Box::new(crate::RustExpr::Ident(
                                                "_len".to_string(),
                                            )),
                                            op: "+".to_string(),
                                            right: Box::new(crate::RustExpr::Ident(
                                                "_sv".to_string(),
                                            )),
                                        },
                                    ))),
                                    method: "max".to_string(),
                                    args: vec![crate::RustExpr::Literal(crate::RustLiteral::Int(
                                        0,
                                    ))],
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
                            expr: Box::new(crate::RustExpr::Paren(Box::new(
                                crate::RustExpr::BinOp {
                                    left: Box::new(crate::RustExpr::Ident("_len".to_string())),
                                    op: "-".to_string(),
                                    right: Box::new(crate::RustExpr::Literal(
                                        crate::RustLiteral::Int(1),
                                    )),
                                },
                            ))),
                            ty: crate::RustType::Named("usize".to_string()),
                        })),
                    }
                };

                let lowered_stop = if let Some(stop_expr) = stop {
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
                                right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(
                                    0,
                                ))),
                            }),
                            then_expr: Box::new(crate::RustExpr::Cast {
                                expr: Box::new(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                        crate::RustExpr::BinOp {
                                            left: Box::new(crate::RustExpr::Ident(
                                                "_len".to_string(),
                                            )),
                                            op: "+".to_string(),
                                            right: Box::new(crate::RustExpr::Ident(
                                                "_ev".to_string(),
                                            )),
                                        },
                                    ))),
                                    method: "max".to_string(),
                                    args: vec![crate::RustExpr::Literal(crate::RustLiteral::Int(
                                        0,
                                    ))],
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

                return match crate::resolve_alias_type_for_plain_call(object.ty()) {
                    Type::List(_) | Type::Bytes => {
                        let copy_slice_elements =
                            match crate::resolve_alias_type_for_plain_call(object.ty()) {
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
                                        expr: Box::new(crate::RustExpr::Paren(Box::new(
                                            lowered_object,
                                        ))),
                                    },
                                },
                                crate::RustStmt::Let {
                                    mutable: false,
                                    name: "_len".to_string(),
                                    ty: None,
                                    value: crate::RustExpr::Cast {
                                        expr: Box::new(crate::RustExpr::MethodCall {
                                            receiver: Box::new(crate::RustExpr::Ident(
                                                "_v".to_string(),
                                            )),
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
                                        right: Box::new(crate::RustExpr::Literal(
                                            crate::RustLiteral::Int(0),
                                        )),
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
                                                left: Box::new(crate::RustExpr::Ident(
                                                    "_i".to_string(),
                                                )),
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
                                                            receiver: Box::new(
                                                                crate::RustExpr::Ident(
                                                                    "_result".to_string(),
                                                                ),
                                                            ),
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
                                                    target: crate::RustExpr::Ident(
                                                        "_i".to_string(),
                                                    ),
                                                    op: "+".to_string(),
                                                    value: crate::RustExpr::Cast {
                                                        expr: Box::new(crate::RustExpr::Ident(
                                                            "_step".to_string(),
                                                        )),
                                                        ty: crate::RustType::Named(
                                                            "usize".to_string(),
                                                        ),
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
                                                expr: Box::new(crate::RustExpr::Ident(
                                                    "_stop".to_string(),
                                                )),
                                                ty: crate::RustType::I64,
                                            },
                                        },
                                        crate::RustStmt::While {
                                            cond: crate::RustExpr::BinOp {
                                                left: Box::new(crate::RustExpr::Ident(
                                                    "_i".to_string(),
                                                )),
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
                                                            receiver: Box::new(
                                                                crate::RustExpr::Ident(
                                                                    "_v".to_string(),
                                                                ),
                                                            ),
                                                            method: "get".to_string(),
                                                            args: vec![crate::RustExpr::Cast {
                                                                expr: Box::new(
                                                                    crate::RustExpr::Ident(
                                                                        "_i".to_string(),
                                                                    ),
                                                                ),
                                                                ty: crate::RustType::Named(
                                                                    "usize".to_string(),
                                                                ),
                                                            }],
                                                        },
                                                        then_body: vec![crate::RustStmt::Expr(
                                                            crate::RustExpr::MethodCall {
                                                                receiver: Box::new(
                                                                    crate::RustExpr::Ident(
                                                                        "_result".to_string(),
                                                                    ),
                                                                ),
                                                                method: "push".to_string(),
                                                                args: vec![
                                                                    if copy_slice_elements {
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
                                                                    },
                                                                ],
                                                            },
                                                        )],
                                                        else_body: None,
                                                    }],
                                                    else_body: None,
                                                },
                                                crate::RustStmt::AugAssign {
                                                    target: crate::RustExpr::Ident(
                                                        "_i".to_string(),
                                                    ),
                                                    op: "+".to_string(),
                                                    value: crate::RustExpr::Ident(
                                                        "_step".to_string(),
                                                    ),
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
                                    expr: Box::new(crate::RustExpr::Paren(Box::new(
                                        lowered_object,
                                    ))),
                                },
                            },
                            crate::RustStmt::Let {
                                mutable: false,
                                name: "_len".to_string(),
                                ty: None,
                                value: crate::RustExpr::Cast {
                                    expr: Box::new(crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::MethodCall {
                                            receiver: Box::new(crate::RustExpr::Ident(
                                                "_s".to_string(),
                                            )),
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
                                    right: Box::new(crate::RustExpr::Literal(
                                        crate::RustLiteral::Int(0),
                                    )),
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
                                            left: Box::new(crate::RustExpr::Ident(
                                                "_i".to_string(),
                                            )),
                                            op: "<".to_string(),
                                            right: Box::new(crate::RustExpr::Ident(
                                                "_stop".to_string(),
                                            )),
                                        },
                                        body: vec![
                                            crate::RustStmt::IfLet {
                                                pattern: "Some(_ch)".to_string(),
                                                expr: crate::RustExpr::MethodCall {
                                                    receiver: Box::new(
                                                        crate::RustExpr::MethodCall {
                                                            receiver: Box::new(
                                                                crate::RustExpr::Ident(
                                                                    "_s".to_string(),
                                                                ),
                                                            ),
                                                            method: "chars".to_string(),
                                                            args: vec![],
                                                        },
                                                    ),
                                                    method: "nth".to_string(),
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
                                            expr: Box::new(crate::RustExpr::Ident(
                                                "_stop".to_string(),
                                            )),
                                            ty: crate::RustType::I64,
                                        },
                                    },
                                    crate::RustStmt::While {
                                        cond: crate::RustExpr::BinOp {
                                            left: Box::new(crate::RustExpr::Ident(
                                                "_i".to_string(),
                                            )),
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
                                                    pattern: "Some(_ch)".to_string(),
                                                    expr: crate::RustExpr::MethodCall {
                                                        receiver: Box::new(
                                                            crate::RustExpr::MethodCall {
                                                                receiver: Box::new(
                                                                    crate::RustExpr::Ident(
                                                                        "_s".to_string(),
                                                                    ),
                                                                ),
                                                                method: "chars".to_string(),
                                                                args: vec![],
                                                            },
                                                        ),
                                                        method: "nth".to_string(),
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
                                                            receiver: Box::new(
                                                                crate::RustExpr::Ident(
                                                                    "_result".to_string(),
                                                                ),
                                                            ),
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
            }
            let lowered_start_raw = if let Some(start_expr) = start {
                let Some(start_lowered) = $emitter.lower_stmt_expr_for_ir(start_expr)? else {
                    return Ok(None);
                };
                Some(start_lowered)
            } else {
                None
            };
            let lowered_stop_raw = if let Some(stop_expr) = stop {
                let Some(stop_lowered) = $emitter.lower_stmt_expr_for_ir(stop_expr)? else {
                    return Ok(None);
                };
                Some(stop_lowered)
            } else {
                None
            };
            let normalize_bound_i64 =
                |raw_opt: Option<crate::RustExpr>, default_value: crate::RustExpr| {
                    let Some(raw) = raw_opt else {
                        return default_value;
                    };
                    crate::RustExpr::If {
                        cond: Box::new(crate::RustExpr::BinOp {
                            left: Box::new(raw.clone()),
                            op: "<".to_string(),
                            right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(0))),
                        }),
                        then_expr: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                crate::RustExpr::BinOp {
                                    left: Box::new(crate::RustExpr::Ident(
                                        "_slice_len_i64".to_string(),
                                    )),
                                    op: "+".to_string(),
                                    right: Box::new(raw.clone()),
                                },
                            ))),
                            method: "max".to_string(),
                            args: vec![crate::RustExpr::Literal(crate::RustLiteral::Int(0))],
                        }),
                        else_expr: Some(Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(raw),
                            method: "min".to_string(),
                            args: vec![crate::RustExpr::Ident("_slice_len_i64".to_string())],
                        })),
                    }
                };

            match crate::resolve_alias_type_for_plain_call(object.ty()) {
                Type::Str => {
                    let start_i64 = normalize_bound_i64(
                        lowered_start_raw,
                        crate::RustExpr::Literal(crate::RustLiteral::Int(0)),
                    );
                    let stop_i64 = normalize_bound_i64(
                        lowered_stop_raw,
                        crate::RustExpr::Ident("_slice_len_i64".to_string()),
                    );
                    let start_usize = crate::RustExpr::Cast {
                        expr: Box::new(crate::RustExpr::Ident("_slice_start_i64".to_string())),
                        ty: crate::RustType::Named("usize".to_string()),
                    };
                    let take_count = crate::RustExpr::Cast {
                        expr: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                crate::RustExpr::BinOp {
                                    left: Box::new(crate::RustExpr::Ident(
                                        "_slice_stop_i64".to_string(),
                                    )),
                                    op: "-".to_string(),
                                    right: Box::new(crate::RustExpr::Ident(
                                        "_slice_start_i64".to_string(),
                                    )),
                                },
                            ))),
                            method: "max".to_string(),
                            args: vec![crate::RustExpr::Literal(crate::RustLiteral::Int(0))],
                        }),
                        ty: crate::RustType::Named("usize".to_string()),
                    };
                    let iter = crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Ident(
                                    "_slice_src".to_string(),
                                )),
                                method: "chars".to_string(),
                                args: vec![],
                            }),
                            method: "skip".to_string(),
                            args: vec![start_usize],
                        }),
                        method: "take".to_string(),
                        args: vec![take_count],
                    };
                    let slice_expr = crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "String".to_string(),
                            "from_iter".to_string(),
                        ])),
                        args: vec![iter],
                    };
                    return Ok(Some(crate::RustExpr::Block {
                        stmts: vec![
                            crate::RustStmt::Let {
                                mutable: false,
                                name: "_slice_src".to_string(),
                                ty: None,
                                value: lowered_object,
                            },
                            crate::RustStmt::Let {
                                mutable: false,
                                name: "_slice_len_i64".to_string(),
                                ty: None,
                                value: crate::RustExpr::Cast {
                                    expr: Box::new(crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::MethodCall {
                                            receiver: Box::new(crate::RustExpr::Ident(
                                                "_slice_src".to_string(),
                                            )),
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
                                name: "_slice_start_i64".to_string(),
                                ty: None,
                                value: start_i64,
                            },
                            crate::RustStmt::Let {
                                mutable: false,
                                name: "_slice_stop_i64".to_string(),
                                ty: None,
                                value: stop_i64,
                            },
                        ],
                        expr: Some(Box::new(slice_expr)),
                    }));
                }
                Type::List(_) | Type::Bytes => {
                    let start_i64 = normalize_bound_i64(
                        lowered_start_raw,
                        crate::RustExpr::Literal(crate::RustLiteral::Int(0)),
                    );
                    let stop_i64 = normalize_bound_i64(
                        lowered_stop_raw,
                        crate::RustExpr::Ident("_slice_len_i64".to_string()),
                    );
                    let start_usize = crate::RustExpr::Cast {
                        expr: Box::new(crate::RustExpr::Ident("_slice_start_i64".to_string())),
                        ty: crate::RustType::Named("usize".to_string()),
                    };
                    let take_count = crate::RustExpr::Cast {
                        expr: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                crate::RustExpr::BinOp {
                                    left: Box::new(crate::RustExpr::Ident(
                                        "_slice_stop_i64".to_string(),
                                    )),
                                    op: "-".to_string(),
                                    right: Box::new(crate::RustExpr::Ident(
                                        "_slice_start_i64".to_string(),
                                    )),
                                },
                            ))),
                            method: "max".to_string(),
                            args: vec![crate::RustExpr::Literal(crate::RustLiteral::Int(0))],
                        }),
                        ty: crate::RustType::Named("usize".to_string()),
                    };
                    let iter = crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Ident(
                                        "_slice_src".to_string(),
                                    )),
                                    method: "iter".to_string(),
                                    args: vec![],
                                }),
                                method: "skip".to_string(),
                                args: vec![start_usize],
                            }),
                            method: "take".to_string(),
                            args: vec![take_count],
                        }),
                        method: "cloned".to_string(),
                        args: vec![],
                    };
                    let slice_expr = crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "Vec".to_string(),
                            "from_iter".to_string(),
                        ])),
                        args: vec![iter],
                    };
                    return Ok(Some(crate::RustExpr::Block {
                        stmts: vec![
                            crate::RustStmt::Let {
                                mutable: false,
                                name: "_slice_src".to_string(),
                                ty: None,
                                value: lowered_object,
                            },
                            crate::RustStmt::Let {
                                mutable: false,
                                name: "_slice_len_i64".to_string(),
                                ty: None,
                                value: crate::RustExpr::Cast {
                                    expr: Box::new(crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::Ident(
                                            "_slice_src".to_string(),
                                        )),
                                        method: "len".to_string(),
                                        args: vec![],
                                    }),
                                    ty: crate::RustType::I64,
                                },
                            },
                            crate::RustStmt::Let {
                                mutable: false,
                                name: "_slice_start_i64".to_string(),
                                ty: None,
                                value: start_i64,
                            },
                            crate::RustStmt::Let {
                                mutable: false,
                                name: "_slice_stop_i64".to_string(),
                                ty: None,
                                value: stop_i64,
                            },
                        ],
                        expr: Some(Box::new(slice_expr)),
                    }));
                }
                _ => return Ok(None),
            }
        }
    }};
}
