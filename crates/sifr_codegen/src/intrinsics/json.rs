//! JSON intrinsic lowerers for registry lowering.

use crate::{RustExpr, RustLiteral, RustMatchArm, RustParam, RustStmt, RustType};

fn string_expr(value: &str) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Literal(RustLiteral::Str(value.to_string()))),
        method: "to_string".to_string(),
        args: vec![],
    }
}

fn some_expr(value: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
        args: vec![value],
    }
}

fn box_expr(value: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Box".to_string(), "new".to_string()])),
        args: vec![value],
    }
}

fn boxed_field_clone_iter(owner: &str, field: &str) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Field {
                    expr: Box::new(RustExpr::Ident(owner.to_string())),
                    field: field.to_string(),
                }),
                method: "as_ref".to_string(),
                args: vec![],
            }),
            method: "iter".to_string(),
            args: vec![],
        }),
        method: "cloned".to_string(),
        args: vec![],
    }
}

fn ok_expr(value: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
        args: vec![value],
    }
}

fn err_expr(value: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
        args: vec![value],
    }
}

fn json_decode_error(message: RustExpr, line: RustExpr, column: RustExpr) -> RustExpr {
    RustExpr::StructInit {
        name: "JSONDecodeError".to_string(),
        fields: vec![
            ("message".to_string(), message),
            ("line".to_string(), line),
            ("column".to_string(), column),
        ],
    }
}

fn json_struct(
    kind: &str,
    bool_value: RustExpr,
    int_value: RustExpr,
    float_value: RustExpr,
    str_value: RustExpr,
    array_items: RustExpr,
    object_items: RustExpr,
) -> RustExpr {
    RustExpr::StructInit {
        name: "JsonValue".to_string(),
        fields: vec![
            ("kind".to_string(), string_expr(kind)),
            ("bool_value".to_string(), bool_value),
            ("int_value".to_string(), int_value),
            ("float_value".to_string(), float_value),
            ("str_value".to_string(), str_value),
            ("array_items".to_string(), array_items),
            ("object_items".to_string(), object_items),
        ],
    }
}

fn json_null_expr() -> RustExpr {
    json_struct(
        "null",
        RustExpr::Literal(RustLiteral::None),
        RustExpr::Literal(RustLiteral::None),
        RustExpr::Literal(RustLiteral::None),
        RustExpr::Literal(RustLiteral::None),
        box_expr(RustExpr::Vec(vec![])),
        box_expr(RustExpr::Vec(vec![])),
    )
}

fn json_bool_expr(value: RustExpr) -> RustExpr {
    json_struct(
        "bool",
        some_expr(value),
        RustExpr::Literal(RustLiteral::None),
        RustExpr::Literal(RustLiteral::None),
        RustExpr::Literal(RustLiteral::None),
        box_expr(RustExpr::Vec(vec![])),
        box_expr(RustExpr::Vec(vec![])),
    )
}

fn json_int_expr(value: RustExpr) -> RustExpr {
    json_struct(
        "int",
        RustExpr::Literal(RustLiteral::None),
        some_expr(value),
        RustExpr::Literal(RustLiteral::None),
        RustExpr::Literal(RustLiteral::None),
        box_expr(RustExpr::Vec(vec![])),
        box_expr(RustExpr::Vec(vec![])),
    )
}

fn json_float_expr(value: RustExpr) -> RustExpr {
    json_struct(
        "float",
        RustExpr::Literal(RustLiteral::None),
        RustExpr::Literal(RustLiteral::None),
        some_expr(value),
        RustExpr::Literal(RustLiteral::None),
        box_expr(RustExpr::Vec(vec![])),
        box_expr(RustExpr::Vec(vec![])),
    )
}

fn json_str_expr(value: RustExpr) -> RustExpr {
    json_struct(
        "str",
        RustExpr::Literal(RustLiteral::None),
        RustExpr::Literal(RustLiteral::None),
        RustExpr::Literal(RustLiteral::None),
        some_expr(value),
        box_expr(RustExpr::Vec(vec![])),
        box_expr(RustExpr::Vec(vec![])),
    )
}

fn json_array_expr(value: RustExpr) -> RustExpr {
    json_struct(
        "array",
        RustExpr::Literal(RustLiteral::None),
        RustExpr::Literal(RustLiteral::None),
        RustExpr::Literal(RustLiteral::None),
        RustExpr::Literal(RustLiteral::None),
        box_expr(value),
        box_expr(RustExpr::Vec(vec![])),
    )
}

fn json_object_expr(value: RustExpr) -> RustExpr {
    json_struct(
        "object",
        RustExpr::Literal(RustLiteral::None),
        RustExpr::Literal(RustLiteral::None),
        RustExpr::Literal(RustLiteral::None),
        RustExpr::Literal(RustLiteral::None),
        box_expr(RustExpr::Vec(vec![])),
        box_expr(value),
    )
}

fn json_value_from_scalar(value: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "serde_json".to_string(),
            "Value".to_string(),
            "from".to_string(),
        ])),
        args: vec![value],
    }
}

fn json_value_from_string(value: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "serde_json".to_string(),
            "Value".to_string(),
            "String".to_string(),
        ])),
        args: vec![value],
    }
}

fn json_null_value_expr() -> RustExpr {
    RustExpr::Path(vec![
        "serde_json".to_string(),
        "Value".to_string(),
        "Null".to_string(),
    ])
}

pub(super) fn lower_json_loads(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__json_input".to_string(),
                ty: None,
                value: args[0].clone(),
            },
            RustStmt::LocalFn {
                name: "__sifr_json_value_from_serde".to_string(),
                params: vec![RustParam::Named {
                    name: "value".to_string(),
                    ty: RustType::Named("serde_json::Value".to_string()),
                }],
                ret: Some(RustType::Named(
                    "Result<JsonValue, JSONDecodeError>".to_string(),
                )),
                body: vec![RustStmt::Match {
                    expr: RustExpr::Ident("value".to_string()),
                    arms: vec![
                        RustMatchArm {
                            pattern: "serde_json::Value::Null".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: vec![RustStmt::Return(Some(ok_expr(json_null_expr())))],
                        },
                        RustMatchArm {
                            pattern: "serde_json::Value::Bool(b)".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: vec![RustStmt::Return(Some(ok_expr(json_bool_expr(
                                RustExpr::Ident("b".to_string()),
                            ))))],
                        },
                        RustMatchArm {
                            pattern: "serde_json::Value::Number(n)".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: vec![
                                RustStmt::IfLet {
                                    pattern: "Some(i)".to_string(),
                                    expr: RustExpr::MethodCall {
                                        receiver: Box::new(RustExpr::Ident("n".to_string())),
                                        method: "as_i64".to_string(),
                                        args: vec![],
                                    },
                                    then_body: vec![RustStmt::Return(Some(ok_expr(
                                        json_int_expr(RustExpr::Ident("i".to_string())),
                                    )))],
                                    else_body: None,
                                },
                                RustStmt::If {
                                    cond: RustExpr::MethodCall {
                                        receiver: Box::new(RustExpr::Ident("n".to_string())),
                                        method: "is_u64".to_string(),
                                        args: vec![],
                                    },
                                    then_body: vec![RustStmt::Return(Some(err_expr(
                                        json_decode_error(
                                            string_expr("json integer out of range for sifr int"),
                                            RustExpr::Literal(RustLiteral::Int(0)),
                                            RustExpr::Literal(RustLiteral::Int(0)),
                                        ),
                                    )))],
                                    else_body: None,
                                },
                                RustStmt::IfLet {
                                    pattern: "Some(f)".to_string(),
                                    expr: RustExpr::MethodCall {
                                        receiver: Box::new(RustExpr::Ident("n".to_string())),
                                        method: "as_f64".to_string(),
                                        args: vec![],
                                    },
                                    then_body: vec![RustStmt::Return(Some(ok_expr(
                                        json_float_expr(RustExpr::Ident("f".to_string())),
                                    )))],
                                    else_body: None,
                                },
                                RustStmt::Return(Some(err_expr(json_decode_error(
                                    string_expr("unsupported json number representation"),
                                    RustExpr::Literal(RustLiteral::Int(0)),
                                    RustExpr::Literal(RustLiteral::Int(0)),
                                )))),
                            ],
                        },
                        RustMatchArm {
                            pattern: "serde_json::Value::String(s)".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: vec![RustStmt::Return(Some(ok_expr(json_str_expr(
                                RustExpr::Ident("s".to_string()),
                            ))))],
                        },
                        RustMatchArm {
                            pattern: "serde_json::Value::Array(items)".to_string(),
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
                                    iter: RustExpr::Ident("items".to_string()),
                                    body: vec![RustStmt::Expr(RustExpr::MethodCall {
                                        receiver: Box::new(RustExpr::Ident(
                                            "converted".to_string(),
                                        )),
                                        method: "push".to_string(),
                                        args: vec![RustExpr::Try(Box::new(RustExpr::FnCall {
                                            func: Box::new(RustExpr::Ident(
                                                "__sifr_json_value_from_serde".to_string(),
                                            )),
                                            args: vec![RustExpr::Ident("item".to_string())],
                                        }))],
                                    })],
                                },
                                RustStmt::Return(Some(ok_expr(json_array_expr(RustExpr::Ident(
                                    "converted".to_string(),
                                ))))),
                            ],
                        },
                        RustMatchArm {
                            pattern: "serde_json::Value::Object(entries)".to_string(),
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
                                    var: "entry".to_string(),
                                    iter: RustExpr::Ident("entries".to_string()),
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
                                        RustStmt::Let {
                                            mutable: false,
                                            name: "converted_value".to_string(),
                                            ty: None,
                                            value: RustExpr::Try(Box::new(RustExpr::FnCall {
                                                func: Box::new(RustExpr::Ident(
                                                    "__sifr_json_value_from_serde".to_string(),
                                                )),
                                                args: vec![RustExpr::Ident(
                                                    "entry_value".to_string(),
                                                )],
                                            })),
                                        },
                                        RustStmt::Expr(RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::Ident(
                                                "converted".to_string(),
                                            )),
                                            method: "push".to_string(),
                                            args: vec![RustExpr::Tuple(vec![
                                                RustExpr::Ident("entry_key".to_string()),
                                                RustExpr::Ident("converted_value".to_string()),
                                            ])],
                                        }),
                                    ],
                                },
                                RustStmt::Return(Some(ok_expr(json_object_expr(RustExpr::Ident(
                                    "converted".to_string(),
                                ))))),
                            ],
                        },
                    ],
                }],
            },
        ],
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "serde_json".to_string(),
                        "from_str::<serde_json::Value>".to_string(),
                    ])),
                    args: vec![RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__json_input".to_string())),
                        method: "as_ref".to_string(),
                        args: vec![],
                    }],
                }),
                method: "map_err".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "e".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(json_decode_error(
                        RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("e".to_string())),
                            method: "to_string".to_string(),
                            args: vec![],
                        },
                        RustExpr::Cast {
                            expr: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("e".to_string())),
                                method: "line".to_string(),
                                args: vec![],
                            }),
                            ty: RustType::I64,
                        },
                        RustExpr::Cast {
                            expr: Box::new(RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("e".to_string())),
                                method: "column".to_string(),
                                args: vec![],
                            }),
                            ty: RustType::I64,
                        },
                    )),
                    is_move: false,
                }],
            }),
            method: "and_then".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "parsed".to_string(),
                    ty: RustType::Named("serde_json::Value".to_string()),
                }],
                body: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Ident("__sifr_json_value_from_serde".to_string())),
                    args: vec![RustExpr::Ident("parsed".to_string())],
                }),
                is_move: false,
            }],
        })),
    })
}

pub(super) fn lower_json_dumps(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "serde_json".to_string(),
                "to_string".to_string(),
            ])),
            args: vec![RustExpr::Ref {
                mutable: false,
                expr: Box::new(args[0].clone()),
            }],
        }),
        method: "unwrap_or_default".to_string(),
        args: vec![],
    })
}

pub(super) fn lower_json_dumps_value(args: &[RustExpr]) -> Option<RustExpr> {
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
