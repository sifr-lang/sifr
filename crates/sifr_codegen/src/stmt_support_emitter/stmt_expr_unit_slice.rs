macro_rules! stmt_expr_unit_slice {
    ($emitter:ident, $object:ident, $start:ident, $stop:ident, $lowered_object:ident) => {{
        let lowered_start_raw = if let Some(start_expr) = $start {
            let Some(start_lowered) = $emitter.lower_stmt_expr_for_ir(start_expr)? else {
                return Ok(None);
            };
            Some(start_lowered)
        } else {
            None
        };
        let lowered_stop_raw = if let Some(stop_expr) = $stop {
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

        match crate::resolve_alias_type_for_plain_call($object.ty()) {
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
                            receiver: Box::new(crate::RustExpr::Ident("_slice_src".to_string())),
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
                            value: $lowered_object,
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
                            value: $lowered_object,
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
    }};
}
