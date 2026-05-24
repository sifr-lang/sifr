//! TOML intrinsic lowerers for registry lowering.

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

fn ok_expr(value: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
        args: vec![value],
    }
}

fn box_expr(value: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Box".to_string(), "new".to_string()])),
        args: vec![value],
    }
}

fn toml_decode_error(message: RustExpr) -> RustExpr {
    RustExpr::StructInit {
        name: "TOMLDecodeError".to_string(),
        fields: vec![
            ("message".to_string(), message),
            ("line".to_string(), RustExpr::Literal(RustLiteral::Int(0))),
            ("column".to_string(), RustExpr::Literal(RustLiteral::Int(0))),
        ],
    }
}

struct TomlStructFields {
    bool_value: RustExpr,
    int_value: RustExpr,
    float_value: RustExpr,
    str_value: RustExpr,
    datetime_value: RustExpr,
    array_items: RustExpr,
    table_items: RustExpr,
}

impl TomlStructFields {
    fn empty() -> Self {
        Self {
            bool_value: RustExpr::Literal(RustLiteral::None),
            int_value: RustExpr::Literal(RustLiteral::None),
            float_value: RustExpr::Literal(RustLiteral::None),
            str_value: RustExpr::Literal(RustLiteral::None),
            datetime_value: RustExpr::Literal(RustLiteral::None),
            array_items: box_expr(RustExpr::Vec(vec![])),
            table_items: box_expr(RustExpr::Vec(vec![])),
        }
    }
}

fn toml_struct(kind: &str, fields: TomlStructFields) -> RustExpr {
    RustExpr::StructInit {
        name: "TomlValue".to_string(),
        fields: vec![
            ("kind".to_string(), string_expr(kind)),
            ("bool_value".to_string(), fields.bool_value),
            ("int_value".to_string(), fields.int_value),
            ("float_value".to_string(), fields.float_value),
            ("str_value".to_string(), fields.str_value),
            ("datetime_value".to_string(), fields.datetime_value),
            ("array_items".to_string(), fields.array_items),
            ("table_items".to_string(), fields.table_items),
        ],
    }
}

fn toml_bool_expr(value: RustExpr) -> RustExpr {
    toml_struct(
        "bool",
        TomlStructFields {
            bool_value: some_expr(value),
            ..TomlStructFields::empty()
        },
    )
}

fn toml_int_expr(value: RustExpr) -> RustExpr {
    toml_struct(
        "int",
        TomlStructFields {
            int_value: some_expr(value),
            ..TomlStructFields::empty()
        },
    )
}

fn toml_float_expr(value: RustExpr) -> RustExpr {
    toml_struct(
        "float",
        TomlStructFields {
            float_value: some_expr(value),
            ..TomlStructFields::empty()
        },
    )
}

fn toml_str_expr(value: RustExpr) -> RustExpr {
    toml_struct(
        "str",
        TomlStructFields {
            str_value: some_expr(value),
            ..TomlStructFields::empty()
        },
    )
}

fn toml_datetime_expr(value: RustExpr) -> RustExpr {
    toml_struct(
        "datetime",
        TomlStructFields {
            datetime_value: some_expr(value),
            ..TomlStructFields::empty()
        },
    )
}

fn toml_array_expr(value: RustExpr) -> RustExpr {
    toml_struct(
        "array",
        TomlStructFields {
            array_items: box_expr(value),
            ..TomlStructFields::empty()
        },
    )
}

fn toml_table_expr(value: RustExpr) -> RustExpr {
    toml_struct(
        "table",
        TomlStructFields {
            table_items: box_expr(value),
            ..TomlStructFields::empty()
        },
    )
}

pub(crate) fn lower_toml_parse(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__toml_input".to_string(),
                ty: None,
                value: RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(args[0].clone()),
                },
            },
            RustStmt::LocalFn {
                name: "__sifr_toml_value_from_parsed".to_string(),
                params: vec![RustParam::Named {
                    name: "value".to_string(),
                    ty: RustType::Named("toml::Value".to_string()),
                }],
                ret: Some(RustType::Named(
                    "Result<TomlValue, TOMLDecodeError>".to_string(),
                )),
                body: vec![RustStmt::Match {
                    expr: RustExpr::Ident("value".to_string()),
                    arms: vec![
                        RustMatchArm {
                            pattern: "toml::Value::Boolean(v)".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: vec![RustStmt::Return(Some(ok_expr(toml_bool_expr(
                                RustExpr::Ident("v".to_string()),
                            ))))],
                        },
                        RustMatchArm {
                            pattern: "toml::Value::Integer(v)".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: vec![RustStmt::Return(Some(ok_expr(toml_int_expr(
                                RustExpr::Ident("v".to_string()),
                            ))))],
                        },
                        RustMatchArm {
                            pattern: "toml::Value::Float(v)".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: vec![RustStmt::Return(Some(ok_expr(toml_float_expr(
                                RustExpr::Ident("v".to_string()),
                            ))))],
                        },
                        RustMatchArm {
                            pattern: "toml::Value::String(v)".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: vec![RustStmt::Return(Some(ok_expr(toml_str_expr(
                                RustExpr::Ident("v".to_string()),
                            ))))],
                        },
                        RustMatchArm {
                            pattern: "toml::Value::Datetime(v)".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: vec![RustStmt::Return(Some(ok_expr(toml_datetime_expr(
                                RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("v".to_string())),
                                    method: "to_string".to_string(),
                                    args: vec![],
                                },
                            ))))],
                        },
                        RustMatchArm {
                            pattern: "toml::Value::Array(items)".to_string(),
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
                                                "__sifr_toml_value_from_parsed".to_string(),
                                            )),
                                            args: vec![RustExpr::Ident("item".to_string())],
                                        }))],
                                    })],
                                },
                                RustStmt::Return(Some(ok_expr(toml_array_expr(RustExpr::Ident(
                                    "converted".to_string(),
                                ))))),
                            ],
                        },
                        RustMatchArm {
                            pattern: "toml::Value::Table(items)".to_string(),
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
                                    iter: RustExpr::Ident("items".to_string()),
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
                                                    "__sifr_toml_value_from_parsed".to_string(),
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
                                RustStmt::Return(Some(ok_expr(toml_table_expr(RustExpr::Ident(
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
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__toml_input".to_string())),
                        method: "parse::<toml::Table>".to_string(),
                        args: vec![],
                    }),
                    method: "map".to_string(),
                    args: vec![RustExpr::Ident("toml::Value::Table".to_string())],
                }),
                method: "map_err".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "e".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(toml_decode_error(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("e".to_string())),
                        method: "to_string".to_string(),
                        args: vec![],
                    })),
                    is_move: false,
                }],
            }),
            method: "and_then".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "parsed".to_string(),
                    ty: RustType::Named("toml::Value".to_string()),
                }],
                body: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Ident("__sifr_toml_value_from_parsed".to_string())),
                    args: vec![RustExpr::Ident("parsed".to_string())],
                }),
                is_move: false,
            }],
        })),
    })
}
