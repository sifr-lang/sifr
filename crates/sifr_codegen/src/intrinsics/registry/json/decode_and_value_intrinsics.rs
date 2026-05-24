use crate::{RustExpr, RustLiteral, RustMatchArm, RustParam, RustStmt, RustType};

pub(crate) fn string_expr(value: &str) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Literal(RustLiteral::Str(value.to_string()))),
        method: "to_string".to_string(),
        args: vec![],
    }
}

pub(crate) fn some_expr(value: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
        args: vec![value],
    }
}

pub(crate) fn box_expr(value: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Box".to_string(), "new".to_string()])),
        args: vec![value],
    }
}

pub(crate) fn boxed_field_clone_iter(owner: &str, field: &str) -> RustExpr {
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

pub(crate) fn ok_expr(value: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
        args: vec![value],
    }
}

pub(crate) fn err_expr(value: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
        args: vec![value],
    }
}

pub(crate) fn json_decode_error(message: RustExpr, line: RustExpr, column: RustExpr) -> RustExpr {
    RustExpr::StructInit {
        name: "JSONDecodeError".to_string(),
        fields: vec![
            ("message".to_string(), message),
            ("line".to_string(), line),
            ("column".to_string(), column),
        ],
    }
}

pub(crate) fn json_struct(
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

pub(crate) fn json_null_expr() -> RustExpr {
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

pub(crate) fn json_bool_expr(value: RustExpr) -> RustExpr {
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

pub(crate) fn json_int_expr(value: RustExpr) -> RustExpr {
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

pub(crate) fn json_float_expr(value: RustExpr) -> RustExpr {
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

pub(crate) fn json_str_expr(value: RustExpr) -> RustExpr {
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

pub(crate) fn json_array_expr(value: RustExpr) -> RustExpr {
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

pub(crate) fn json_object_expr(value: RustExpr) -> RustExpr {
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

pub(crate) fn json_value_from_scalar(value: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "serde_json".to_string(),
            "Value".to_string(),
            "from".to_string(),
        ])),
        args: vec![value],
    }
}

pub(crate) fn json_value_from_string(value: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "serde_json".to_string(),
            "Value".to_string(),
            "String".to_string(),
        ])),
        args: vec![value],
    }
}

pub(crate) fn json_null_value_expr() -> RustExpr {
    RustExpr::Path(vec![
        "serde_json".to_string(),
        "Value".to_string(),
        "Null".to_string(),
    ])
}

#[derive(Clone, Copy)]
pub(crate) enum JsonIntegerProfileLowering {
    Exact,
    Web,
    StringInts,
}

impl JsonIntegerProfileLowering {
    const fn runtime_variant(self) -> &'static str {
        match self {
            Self::Exact => "Exact",
            Self::Web => "Web",
            Self::StringInts => "StringInts",
        }
    }

    pub(crate) const fn is_fallible(self) -> bool {
        matches!(self, Self::Web)
    }
}

pub(crate) fn runtime_json_profile(profile: JsonIntegerProfileLowering) -> RustExpr {
    RustExpr::Path(vec![
        "sifr_runtime".to_string(),
        "json".to_string(),
        "JsonIntegerProfile".to_string(),
        profile.runtime_variant().to_string(),
    ])
}

pub(crate) fn runtime_json_encoding_path(variant: &str) -> String {
    format!("sifr_runtime::json::JsonIntegerEncoding::{variant}")
}

pub(crate) fn sifr_int_from_i64(value: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "sifr_runtime".to_string(),
            "SifrInt".to_string(),
            "from_i64".to_string(),
        ])),
        args: vec![value],
    }
}

pub(crate) fn path_as_str() -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident("path".to_string())),
        method: "as_str".to_string(),
        args: vec![],
    }
}

pub(crate) fn json_profile_error_struct_from_runtime(err_name: &str) -> RustExpr {
    RustExpr::StructInit {
        name: "JsonIntegerRangeError".to_string(),
        fields: vec![
            (
                "message".to_string(),
                RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident(err_name.to_string())),
                        method: "message".to_string(),
                        args: vec![],
                    }),
                    method: "to_string".to_string(),
                    args: vec![],
                },
            ),
            (
                "path".to_string(),
                RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident(err_name.to_string())),
                        method: "path".to_string(),
                        args: vec![],
                    }),
                    method: "to_string".to_string(),
                    args: vec![],
                },
            ),
            (
                "profile".to_string(),
                RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident(err_name.to_string())),
                        method: "profile".to_string(),
                        args: vec![],
                    }),
                    method: "to_string".to_string(),
                    args: vec![],
                },
            ),
        ],
    }
}

pub(crate) fn json_limit_error_struct_from_runtime(err_name: &str) -> RustExpr {
    RustExpr::StructInit {
        name: "JsonLimitError".to_string(),
        fields: vec![
            (
                "message".to_string(),
                RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident(err_name.to_string())),
                        method: "message".to_string(),
                        args: vec![],
                    }),
                    method: "to_string".to_string(),
                    args: vec![],
                },
            ),
            (
                "limit".to_string(),
                RustExpr::Cast {
                    expr: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident(err_name.to_string())),
                        method: "limit".to_string(),
                        args: vec![],
                    }),
                    ty: RustType::I64,
                },
            ),
        ],
    }
}

pub(crate) fn json_limit_error_as_decode_error(err_name: &str) -> RustExpr {
    json_decode_error(
        RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(err_name.to_string())),
                method: "message".to_string(),
                args: vec![],
            }),
            method: "to_string".to_string(),
            args: vec![],
        },
        RustExpr::Literal(RustLiteral::Int(0)),
        RustExpr::Literal(RustLiteral::Int(0)),
    )
}

pub(crate) fn json_validate_integer_digit_limits_expr(input: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "sifr_runtime".to_string(),
            "json".to_string(),
            "validate_json_integer_digit_limits".to_string(),
        ])),
        args: vec![
            RustExpr::MethodCall {
                receiver: Box::new(input),
                method: "as_ref".to_string(),
                args: vec![],
            },
            RustExpr::Path(vec![
                "sifr_runtime".to_string(),
                "json".to_string(),
                "DEFAULT_JSON_INTEGER_DIGIT_LIMIT".to_string(),
            ]),
        ],
    }
}

pub(crate) fn json_integer_profile_encode_expr(profile: JsonIntegerProfileLowering) -> RustExpr {
    RustExpr::Try(Box::new(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "sifr_runtime".to_string(),
                "json".to_string(),
                "encode_integer_for_profile".to_string(),
            ])),
            args: vec![
                RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(sifr_int_from_i64(RustExpr::Ident("v".to_string()))),
                },
                runtime_json_profile(profile),
                path_as_str(),
            ],
        }),
        method: "map_err".to_string(),
        args: vec![RustExpr::Ident(
            "__sifr_json_integer_range_error_from_runtime".to_string(),
        )],
    }))
}

pub(crate) fn json_profile_int_match_arms(profile: JsonIntegerProfileLowering) -> Vec<RustStmt> {
    vec![
        RustStmt::IfLet {
            pattern: "Some(v)".to_string(),
            expr: RustExpr::Field {
                expr: Box::new(RustExpr::Ident("value".to_string())),
                field: "int_value".to_string(),
            },
            then_body: vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__encoded_int".to_string(),
                    ty: None,
                    value: json_integer_profile_encode_expr(profile),
                },
                RustStmt::Match {
                    expr: RustExpr::Ident("__encoded_int".to_string()),
                    arms: vec![
                        RustMatchArm {
                            pattern: runtime_json_encoding_path("Number(_)"),
                            bindings: vec![],
                            guard: None,
                            body: vec![RustStmt::Return(Some(ok_expr(json_value_from_scalar(
                                RustExpr::Ident("v".to_string()),
                            ))))],
                        },
                        RustMatchArm {
                            pattern: runtime_json_encoding_path("DecimalString(decimal)"),
                            bindings: vec![],
                            guard: None,
                            body: vec![RustStmt::Return(Some(ok_expr(json_value_from_string(
                                RustExpr::Ident("decimal".to_string()),
                            ))))],
                        },
                    ],
                },
            ],
            else_body: None,
        },
        RustStmt::Return(Some(ok_expr(json_null_value_expr()))),
    ]
}

pub(crate) fn child_array_path_expr() -> RustExpr {
    RustExpr::FormatMacro {
        name: "format".to_string(),
        format_str: "{}[{}]".to_string(),
        args: vec![
            RustExpr::Ident("path".to_string()),
            RustExpr::Ident("idx".to_string()),
        ],
    }
}

pub(crate) fn child_object_path_expr() -> RustExpr {
    RustExpr::FormatMacro {
        name: "format".to_string(),
        format_str: "{}.{}".to_string(),
        args: vec![
            RustExpr::Ident("path".to_string()),
            RustExpr::Ident("entry_key".to_string()),
        ],
    }
}

pub(crate) fn lower_json_loads(args: &[RustExpr]) -> Option<RustExpr> {
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
                name: "__sifr_json_limit_error_as_decode_error".to_string(),
                params: vec![RustParam::Named {
                    name: "err".to_string(),
                    ty: RustType::Named("sifr_runtime::json::JsonLimitError".to_string()),
                }],
                ret: Some(RustType::Named("JSONDecodeError".to_string())),
                body: vec![RustStmt::Return(Some(json_limit_error_as_decode_error(
                    "err",
                )))],
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
                receiver: Box::new(json_validate_integer_digit_limits_expr(RustExpr::Ident(
                    "__json_input".to_string(),
                ))),
                method: "map_err".to_string(),
                args: vec![RustExpr::Ident(
                    "__sifr_json_limit_error_as_decode_error".to_string(),
                )],
            }),
            method: "and_then".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "_".to_string(),
                    ty: RustType::Unit,
                }],
                body: Box::new(RustExpr::MethodCall {
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
                            func: Box::new(RustExpr::Ident(
                                "__sifr_json_value_from_serde".to_string(),
                            )),
                            args: vec![RustExpr::Ident("parsed".to_string())],
                        }),
                        is_move: false,
                    }],
                }),
                is_move: false,
            }],
        })),
    })
}

pub(crate) fn lower_json_validate_integer_digit_limits(args: &[RustExpr]) -> Option<RustExpr> {
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
                name: "__sifr_json_limit_error_from_runtime".to_string(),
                params: vec![RustParam::Named {
                    name: "err".to_string(),
                    ty: RustType::Named("sifr_runtime::json::JsonLimitError".to_string()),
                }],
                ret: Some(RustType::Named("JsonLimitError".to_string())),
                body: vec![RustStmt::Return(Some(
                    json_limit_error_struct_from_runtime("err"),
                ))],
            },
        ],
        expr: Some(Box::new(RustExpr::MethodCall {
            receiver: Box::new(json_validate_integer_digit_limits_expr(RustExpr::Ident(
                "__json_input".to_string(),
            ))),
            method: "map_err".to_string(),
            args: vec![RustExpr::Ident(
                "__sifr_json_limit_error_from_runtime".to_string(),
            )],
        })),
    })
}

pub(crate) fn lower_json_dumps(args: &[RustExpr]) -> Option<RustExpr> {
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
