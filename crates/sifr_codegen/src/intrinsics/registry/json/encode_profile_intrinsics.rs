use super::{
    boxed_field_clone_iter, child_array_path_expr, child_object_path_expr, json_null_value_expr,
    json_profile_error_struct_from_runtime, json_profile_int_match_arms, json_value_from_scalar,
    json_value_from_string, ok_expr, string_expr, JsonIntegerProfileLowering, RustExpr,
    RustMatchArm, RustParam, RustStmt, RustType,
};
pub(crate) fn lower_json_dumps_value(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__json_value".to_string(),
                ty: None,
                value: args[0].clone(),
            },
            RustStmt::LocalFn {
                name: "__sifr_json_value_to_serde".to_string(),
                params: vec![RustParam::Named {
                    name: "value".to_string(),
                    ty: RustType::Ref {
                        mutable: false,
                        inner: Box::new(RustType::Named("JsonValue".to_string())),
                    },
                }],
                ret: Some(RustType::Named("serde_json::Value".to_string())),
                body: vec![RustStmt::Match {
                    expr: RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Field {
                            expr: Box::new(RustExpr::Ident("value".to_string())),
                            field: "kind".to_string(),
                        }),
                        method: "as_str".to_string(),
                        args: vec![],
                    },
                    arms: vec![
                        RustMatchArm {
                            pattern: "\"null\"".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: vec![RustStmt::Return(Some(json_null_value_expr()))],
                        },
                        RustMatchArm {
                            pattern: "\"bool\"".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: vec![
                                RustStmt::IfLet {
                                    pattern: "Some(v)".to_string(),
                                    expr: RustExpr::Field {
                                        expr: Box::new(RustExpr::Ident("value".to_string())),
                                        field: "bool_value".to_string(),
                                    },
                                    then_body: vec![RustStmt::Return(Some(
                                        json_value_from_scalar(RustExpr::Ident("v".to_string())),
                                    ))],
                                    else_body: None,
                                },
                                RustStmt::Return(Some(json_null_value_expr())),
                            ],
                        },
                        RustMatchArm {
                            pattern: "\"int\"".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: vec![
                                RustStmt::IfLet {
                                    pattern: "Some(v)".to_string(),
                                    expr: RustExpr::Field {
                                        expr: Box::new(RustExpr::Ident("value".to_string())),
                                        field: "int_value".to_string(),
                                    },
                                    then_body: vec![RustStmt::Return(Some(
                                        json_value_from_scalar(RustExpr::Ident("v".to_string())),
                                    ))],
                                    else_body: None,
                                },
                                RustStmt::Return(Some(json_null_value_expr())),
                            ],
                        },
                        RustMatchArm {
                            pattern: "\"float\"".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: vec![
                                RustStmt::IfLet {
                                    pattern: "Some(v)".to_string(),
                                    expr: RustExpr::Field {
                                        expr: Box::new(RustExpr::Ident("value".to_string())),
                                        field: "float_value".to_string(),
                                    },
                                    then_body: vec![RustStmt::Return(Some(
                                        json_value_from_scalar(RustExpr::Ident("v".to_string())),
                                    ))],
                                    else_body: None,
                                },
                                RustStmt::Return(Some(json_null_value_expr())),
                            ],
                        },
                        RustMatchArm {
                            pattern: "\"str\"".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: vec![
                                RustStmt::IfLet {
                                    pattern: "Some(v)".to_string(),
                                    expr: RustExpr::Clone(Box::new(RustExpr::Field {
                                        expr: Box::new(RustExpr::Ident("value".to_string())),
                                        field: "str_value".to_string(),
                                    })),
                                    then_body: vec![RustStmt::Return(Some(
                                        json_value_from_string(RustExpr::Ident("v".to_string())),
                                    ))],
                                    else_body: None,
                                },
                                RustStmt::Return(Some(json_null_value_expr())),
                            ],
                        },
                        RustMatchArm {
                            pattern: "\"array\"".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: vec![
                                RustStmt::Let {
                                    mutable: true,
                                    name: "converted".to_string(),
                                    ty: None,
                                    value: RustExpr::Vec(vec![]),
                                },
                                RustStmt::For {
                                    var: "item".to_string(),
                                    iter: boxed_field_clone_iter("value", "array_items"),
                                    body: vec![RustStmt::Expr(RustExpr::MethodCall {
                                        receiver: Box::new(RustExpr::Ident(
                                            "converted".to_string(),
                                        )),
                                        method: "push".to_string(),
                                        args: vec![RustExpr::FnCall {
                                            func: Box::new(RustExpr::Ident(
                                                "__sifr_json_value_to_serde".to_string(),
                                            )),
                                            args: vec![RustExpr::Ref {
                                                mutable: false,
                                                expr: Box::new(RustExpr::Ident("item".to_string())),
                                            }],
                                        }],
                                    })],
                                },
                                RustStmt::Return(Some(RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "serde_json".to_string(),
                                        "Value".to_string(),
                                        "Array".to_string(),
                                    ])),
                                    args: vec![RustExpr::Ident("converted".to_string())],
                                })),
                            ],
                        },
                        RustMatchArm {
                            pattern: "\"object\"".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: vec![
                                RustStmt::Let {
                                    mutable: true,
                                    name: "converted".to_string(),
                                    ty: None,
                                    value: RustExpr::FnCall {
                                        func: Box::new(RustExpr::Path(vec![
                                            "serde_json".to_string(),
                                            "Map".to_string(),
                                            "new".to_string(),
                                        ])),
                                        args: vec![],
                                    },
                                },
                                RustStmt::For {
                                    var: "entry".to_string(),
                                    iter: boxed_field_clone_iter("value", "object_items"),
                                    body: vec![
                                        RustStmt::Let {
                                            mutable: false,
                                            name: "entry_key".to_string(),
                                            ty: None,
                                            value: RustExpr::Field {
                                                expr: Box::new(RustExpr::Ident(
                                                    "entry".to_string(),
                                                )),
                                                field: "0".to_string(),
                                            },
                                        },
                                        RustStmt::Let {
                                            mutable: false,
                                            name: "entry_value".to_string(),
                                            ty: None,
                                            value: RustExpr::Field {
                                                expr: Box::new(RustExpr::Ident(
                                                    "entry".to_string(),
                                                )),
                                                field: "1".to_string(),
                                            },
                                        },
                                        RustStmt::Expr(RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::Ident(
                                                "converted".to_string(),
                                            )),
                                            method: "insert".to_string(),
                                            args: vec![
                                                RustExpr::Ident("entry_key".to_string()),
                                                RustExpr::FnCall {
                                                    func: Box::new(RustExpr::Ident(
                                                        "__sifr_json_value_to_serde".to_string(),
                                                    )),
                                                    args: vec![RustExpr::Ref {
                                                        mutable: false,
                                                        expr: Box::new(RustExpr::Ident(
                                                            "entry_value".to_string(),
                                                        )),
                                                    }],
                                                },
                                            ],
                                        }),
                                    ],
                                },
                                RustStmt::Return(Some(RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "serde_json".to_string(),
                                        "Value".to_string(),
                                        "Object".to_string(),
                                    ])),
                                    args: vec![RustExpr::Ident("converted".to_string())],
                                })),
                            ],
                        },
                        RustMatchArm {
                            pattern: "_".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: vec![RustStmt::Return(Some(json_null_value_expr()))],
                        },
                    ],
                }],
            },
        ],
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "serde_json".to_string(),
                    "to_string".to_string(),
                ])),
                args: vec![RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(RustExpr::FnCall {
                        func: Box::new(RustExpr::Ident("__sifr_json_value_to_serde".to_string())),
                        args: vec![RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(RustExpr::Ident("__json_value".to_string())),
                        }],
                    }),
                }],
            }),
            method: "unwrap_or_else".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "_err".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(string_expr("null")),
                is_move: false,
            }],
        })),
    })
}

pub(crate) fn lower_json_dumps_value_exact(args: &[RustExpr]) -> Option<RustExpr> {
    lower_json_dumps_value_profile(args, JsonIntegerProfileLowering::Exact)
}

pub(crate) fn lower_json_dumps_value_web(args: &[RustExpr]) -> Option<RustExpr> {
    lower_json_dumps_value_profile(args, JsonIntegerProfileLowering::Web)
}

pub(crate) fn lower_json_dumps_value_string_ints(args: &[RustExpr]) -> Option<RustExpr> {
    lower_json_dumps_value_profile(args, JsonIntegerProfileLowering::StringInts)
}

pub(crate) fn lower_json_dumps_value_profile(
    args: &[RustExpr],
    profile: JsonIntegerProfileLowering,
) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let serde_result = RustExpr::MethodCall {
        receiver: Box::new(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "serde_json".to_string(),
                "to_string".to_string(),
            ])),
            args: vec![RustExpr::Ref {
                mutable: false,
                expr: Box::new(RustExpr::Try(Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Ident(
                        "__sifr_json_value_to_serde_profile".to_string(),
                    )),
                    args: vec![
                        RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(RustExpr::Ident("__json_value".to_string())),
                        },
                        string_expr("$"),
                    ],
                }))),
            }],
        }),
        method: "unwrap_or_else".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "_err".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(string_expr("null")),
            is_move: false,
        }],
    };
    let final_expr = if profile.is_fallible() {
        ok_expr(serde_result)
    } else {
        RustExpr::MethodCall {
            receiver: Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Closure {
                    params: vec![],
                    body: Box::new(RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "Ok::<String, JsonIntegerRangeError>".to_string()
                        ])),
                        args: vec![serde_result],
                    }),
                    is_move: false,
                }),
                args: vec![],
            }),
            method: "unwrap_or_else".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "_err".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(string_expr("null")),
                is_move: false,
            }],
        }
    };

    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__json_value".to_string(),
                ty: None,
                value: args[0].clone(),
            },
            RustStmt::LocalFn {
                name: "__sifr_json_integer_range_error_from_runtime".to_string(),
                params: vec![RustParam::Named {
                    name: "err".to_string(),
                    ty: RustType::Named("sifr_runtime::json::JsonIntegerRangeError".to_string()),
                }],
                ret: Some(RustType::Named("JsonIntegerRangeError".to_string())),
                body: vec![RustStmt::Return(Some(
                    json_profile_error_struct_from_runtime("err"),
                ))],
            },
            RustStmt::LocalFn {
                name: "__sifr_json_value_to_serde_profile".to_string(),
                params: vec![
                    RustParam::Named {
                        name: "value".to_string(),
                        ty: RustType::Ref {
                            mutable: false,
                            inner: Box::new(RustType::Named("JsonValue".to_string())),
                        },
                    },
                    RustParam::Named {
                        name: "path".to_string(),
                        ty: RustType::String_,
                    },
                ],
                ret: Some(RustType::Named(
                    "Result<serde_json::Value, JsonIntegerRangeError>".to_string(),
                )),
                body: vec![RustStmt::Match {
                    expr: RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Field {
                            expr: Box::new(RustExpr::Ident("value".to_string())),
                            field: "kind".to_string(),
                        }),
                        method: "as_str".to_string(),
                        args: vec![],
                    },
                    arms: vec![
                        RustMatchArm {
                            pattern: "\"null\"".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: vec![RustStmt::Return(Some(ok_expr(json_null_value_expr())))],
                        },
                        RustMatchArm {
                            pattern: "\"bool\"".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: vec![
                                RustStmt::IfLet {
                                    pattern: "Some(v)".to_string(),
                                    expr: RustExpr::Field {
                                        expr: Box::new(RustExpr::Ident("value".to_string())),
                                        field: "bool_value".to_string(),
                                    },
                                    then_body: vec![RustStmt::Return(Some(ok_expr(
                                        json_value_from_scalar(RustExpr::Ident("v".to_string())),
                                    )))],
                                    else_body: None,
                                },
                                RustStmt::Return(Some(ok_expr(json_null_value_expr()))),
                            ],
                        },
                        RustMatchArm {
                            pattern: "\"int\"".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: json_profile_int_match_arms(profile),
                        },
                        RustMatchArm {
                            pattern: "\"float\"".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: vec![
                                RustStmt::IfLet {
                                    pattern: "Some(v)".to_string(),
                                    expr: RustExpr::Field {
                                        expr: Box::new(RustExpr::Ident("value".to_string())),
                                        field: "float_value".to_string(),
                                    },
                                    then_body: vec![RustStmt::Return(Some(ok_expr(
                                        json_value_from_scalar(RustExpr::Ident("v".to_string())),
                                    )))],
                                    else_body: None,
                                },
                                RustStmt::Return(Some(ok_expr(json_null_value_expr()))),
                            ],
                        },
                        RustMatchArm {
                            pattern: "\"str\"".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: vec![
                                RustStmt::IfLet {
                                    pattern: "Some(v)".to_string(),
                                    expr: RustExpr::Clone(Box::new(RustExpr::Field {
                                        expr: Box::new(RustExpr::Ident("value".to_string())),
                                        field: "str_value".to_string(),
                                    })),
                                    then_body: vec![RustStmt::Return(Some(ok_expr(
                                        json_value_from_string(RustExpr::Ident("v".to_string())),
                                    )))],
                                    else_body: None,
                                },
                                RustStmt::Return(Some(ok_expr(json_null_value_expr()))),
                            ],
                        },
                        RustMatchArm {
                            pattern: "\"array\"".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: vec![
                                RustStmt::Let {
                                    mutable: true,
                                    name: "converted".to_string(),
                                    ty: None,
                                    value: RustExpr::Vec(vec![]),
                                },
                                RustStmt::For {
                                    var: "(idx, item)".to_string(),
                                    iter: RustExpr::MethodCall {
                                        receiver: Box::new(boxed_field_clone_iter(
                                            "value",
                                            "array_items",
                                        )),
                                        method: "enumerate".to_string(),
                                        args: vec![],
                                    },
                                    body: vec![RustStmt::Expr(RustExpr::MethodCall {
                                        receiver: Box::new(RustExpr::Ident(
                                            "converted".to_string(),
                                        )),
                                        method: "push".to_string(),
                                        args: vec![RustExpr::Try(Box::new(RustExpr::FnCall {
                                            func: Box::new(RustExpr::Ident(
                                                "__sifr_json_value_to_serde_profile".to_string(),
                                            )),
                                            args: vec![
                                                RustExpr::Ref {
                                                    mutable: false,
                                                    expr: Box::new(RustExpr::Ident(
                                                        "item".to_string(),
                                                    )),
                                                },
                                                child_array_path_expr(),
                                            ],
                                        }))],
                                    })],
                                },
                                RustStmt::Return(Some(ok_expr(RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "serde_json".to_string(),
                                        "Value".to_string(),
                                        "Array".to_string(),
                                    ])),
                                    args: vec![RustExpr::Ident("converted".to_string())],
                                }))),
                            ],
                        },
                        RustMatchArm {
                            pattern: "\"object\"".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: vec![
                                RustStmt::Let {
                                    mutable: true,
                                    name: "converted".to_string(),
                                    ty: None,
                                    value: RustExpr::FnCall {
                                        func: Box::new(RustExpr::Path(vec![
                                            "serde_json".to_string(),
                                            "Map".to_string(),
                                            "new".to_string(),
                                        ])),
                                        args: vec![],
                                    },
                                },
                                RustStmt::For {
                                    var: "entry".to_string(),
                                    iter: boxed_field_clone_iter("value", "object_items"),
                                    body: vec![
                                        RustStmt::Let {
                                            mutable: false,
                                            name: "entry_key".to_string(),
                                            ty: None,
                                            value: RustExpr::Field {
                                                expr: Box::new(RustExpr::Ident(
                                                    "entry".to_string(),
                                                )),
                                                field: "0".to_string(),
                                            },
                                        },
                                        RustStmt::Let {
                                            mutable: false,
                                            name: "entry_value".to_string(),
                                            ty: None,
                                            value: RustExpr::Field {
                                                expr: Box::new(RustExpr::Ident(
                                                    "entry".to_string(),
                                                )),
                                                field: "1".to_string(),
                                            },
                                        },
                                        RustStmt::Expr(RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::Ident(
                                                "converted".to_string(),
                                            )),
                                            method: "insert".to_string(),
                                            args: vec![
                                                RustExpr::Clone(Box::new(RustExpr::Ident(
                                                    "entry_key".to_string(),
                                                ))),
                                                RustExpr::Try(Box::new(RustExpr::FnCall {
                                                    func: Box::new(RustExpr::Ident(
                                                        "__sifr_json_value_to_serde_profile"
                                                            .to_string(),
                                                    )),
                                                    args: vec![
                                                        RustExpr::Ref {
                                                            mutable: false,
                                                            expr: Box::new(RustExpr::Ident(
                                                                "entry_value".to_string(),
                                                            )),
                                                        },
                                                        child_object_path_expr(),
                                                    ],
                                                })),
                                            ],
                                        }),
                                    ],
                                },
                                RustStmt::Return(Some(ok_expr(RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "serde_json".to_string(),
                                        "Value".to_string(),
                                        "Object".to_string(),
                                    ])),
                                    args: vec![RustExpr::Ident("converted".to_string())],
                                }))),
                            ],
                        },
                        RustMatchArm {
                            pattern: "_".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: vec![RustStmt::Return(Some(ok_expr(json_null_value_expr())))],
                        },
                    ],
                }],
            },
        ],
        expr: Some(Box::new(final_expr)),
    })
}
