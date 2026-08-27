use super::{Renderer, RustExpr, RustFile, RustItem, RustStmt, RustType};
use crate::generated_source_validate::assert_generated_source_is_safe;

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

pub fn render_items(items: &[RustItem]) -> String {
    let file = RustFile {
        items: items.to_vec(),
    };
    Renderer::new().render_file(&file)
}

pub fn render_stmts(stmts: &[RustStmt]) -> String {
    let mut renderer = Renderer::new();
    for stmt in stmts {
        renderer.render_stmt(stmt);
    }
    let output = renderer.output;
    assert_generated_source_is_safe(
        &format!("async fn __sifr_rendered_statements() {{ {output} }}"),
        "structured Rust statement render",
    );
    output
}

pub fn render_expr(expr: &RustExpr) -> String {
    let mut renderer = Renderer::new();
    renderer.render_expr(expr);
    let output = renderer.output;
    assert_generated_source_is_safe(
        &format!("fn __sifr_rendered_expression() {{ let _ = {output}; }}"),
        "structured Rust expression render",
    );
    output
}

pub fn render_type(ty: &RustType) -> String {
    let mut renderer = Renderer::new();
    renderer.render_type(ty);
    let output = renderer.output;
    assert_generated_source_is_safe(
        &format!("type __SifrRenderedType = {output};"),
        "structured Rust type render",
    );
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RustEnumVariant, RustLiteral, RustMatchArm, RustParam, RustTypeParam, Visibility};
    use insta::assert_snapshot;

    #[test]
    fn renders_struct_enum_trait_and_impl() {
        let items = vec![
            RustItem::Struct {
                name: "Point".to_string(),
                visibility: Visibility::Pub,
                derives: vec!["Debug".to_string(), "Clone".to_string()],
                fields: vec![
                    ("x".to_string(), RustType::I64),
                    ("y".to_string(), RustType::I64),
                ],
            },
            RustItem::Enum {
                name: "Token".to_string(),
                visibility: Visibility::Private,
                derives: vec!["Debug".to_string()],
                repr: None,
                variants: vec![
                    RustEnumVariant {
                        name: "Int".to_string(),
                        tuple_fields: vec![],
                        fields: vec![("value".to_string(), RustType::I64)],
                        value: None,
                    },
                    RustEnumVariant {
                        name: "Eof".to_string(),
                        tuple_fields: vec![],
                        fields: vec![],
                        value: None,
                    },
                    RustEnumVariant {
                        name: "Bytes".to_string(),
                        tuple_fields: vec![RustType::Vec(Box::new(RustType::I64))],
                        fields: vec![],
                        value: None,
                    },
                ],
            },
            RustItem::Trait {
                name: "Renderable".to_string(),
                visibility: Visibility::Pub,
                supertraits: vec![],
                methods: vec![RustItem::Fn {
                    name: "render".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![RustParam::SelfParam { mutable: false }],
                    ret: Some(RustType::String_),
                    body: vec![RustStmt::Return(Some(RustExpr::Literal(RustLiteral::Str(
                        "ok".to_string(),
                    ))))],
                    is_async: false,
                }],
            },
            RustItem::Impl {
                target: "Point".to_string(),
                type_params: vec![RustTypeParam {
                    name: "T".to_string(),
                    bounds: vec!["Clone".to_string()],
                }],
                trait_: Some("Renderable".to_string()),
                items: vec![RustItem::Fn {
                    name: "render".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![RustParam::SelfParam { mutable: false }],
                    ret: Some(RustType::String_),
                    body: vec![RustStmt::Return(Some(RustExpr::Literal(RustLiteral::Str(
                        "Point".to_string(),
                    ))))],
                    is_async: false,
                }],
            },
        ];

        let rendered = render_items(&items);
        assert_snapshot!(rendered, @r###"
        #[derive(Debug, Clone)]
        pub struct Point {
            x: i64,
            y: i64,
        }

        #[derive(Debug)]
        enum Token {
            Int { value: i64 },
            Eof,
            Bytes(Vec<i64>),
        }

        pub trait Renderable {
            fn render(&self) -> String {
                "ok".to_string()
            }
        }

        impl<T: Clone> Renderable for Point {
            fn render(&self) -> String {
                "Point".to_string()
            }
        }
        "###);
    }

    #[test]
    fn renders_use_alias_item() {
        let rendered = render_items(&[RustItem::UseAlias {
            path: vec![
                "crate".to_string(),
                "utils".to_string(),
                "helper".to_string(),
            ],
            alias: "h".to_string(),
        }]);
        assert_eq!(rendered, "use crate::utils::helper as h;\n");
    }

    #[test]
    fn renders_function_with_control_flow_statements() {
        let item = RustItem::Fn {
            name: "control".to_string(),
            visibility: Visibility::Pub,
            type_params: vec![],
            params: vec![RustParam::Named {
                name: "items".to_string(),
                ty: RustType::Vec(Box::new(RustType::I64)),
            }],
            ret: Some(RustType::Unit),
            body: vec![
                RustStmt::Let {
                    mutable: true,
                    name: "acc".to_string(),
                    ty: Some(RustType::I64),
                    value: RustExpr::Literal(RustLiteral::Int(0)),
                },
                RustStmt::If {
                    cond: RustExpr::BinOp {
                        left: Box::new(RustExpr::Ident("acc".to_string())),
                        op: "==".to_string(),
                        right: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                    },
                    then_body: vec![RustStmt::Expr(RustExpr::MacroCall {
                        name: "println".to_string(),
                        args: vec![RustExpr::Literal(RustLiteral::Str("empty".to_string()))],
                    })],
                    else_body: None,
                },
                RustStmt::For {
                    var: "value".to_string(),
                    iter: RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("items".to_string())),
                        method: "iter".to_string(),
                        args: vec![],
                    },
                    body: vec![RustStmt::AugAssign {
                        target: RustExpr::Ident("acc".to_string()),
                        op: "+".to_string(),
                        value: RustExpr::Deref(Box::new(RustExpr::Ident("value".to_string()))),
                    }],
                },
                RustStmt::While {
                    cond: RustExpr::BinOp {
                        left: Box::new(RustExpr::Ident("acc".to_string())),
                        op: "<".to_string(),
                        right: Box::new(RustExpr::Literal(RustLiteral::Int(10))),
                    },
                    body: vec![RustStmt::Break],
                },
                RustStmt::Loop {
                    body: vec![RustStmt::Continue],
                },
                RustStmt::Match {
                    expr: RustExpr::Ident("acc".to_string()),
                    arms: vec![
                        RustMatchArm {
                            pattern: "0".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: vec![RustStmt::Return(None)],
                        },
                        RustMatchArm {
                            pattern: "_".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: vec![RustStmt::Expr(RustExpr::MacroCall {
                                name: "println".to_string(),
                                args: vec![RustExpr::Literal(RustLiteral::Str(
                                    "non-zero".to_string(),
                                ))],
                            })],
                        },
                    ],
                },
            ],
            is_async: false,
        };

        let rendered = render_items(&[item]);
        assert_snapshot!(rendered, @r###"
        pub fn control(items: Vec<i64>) -> () {
            let mut acc: i64 = 0;
            if acc == 0 {
                println!("empty".to_string());
            }
            for value in items.iter() {
                acc += *value;
            }
            while acc < 10 {
                break;
            }
            loop {
                continue;
            }
            match acc {
                0 => {
                    return;
                },
                _ => {
                    println!("non-zero".to_string());
                },
            }
        }
        "###);
    }

    #[test]
    fn renders_augassign_with_normalized_and_raw_ops_without_double_equals() {
        let rendered = render_items(&[RustItem::Fn {
            name: "aug_ops".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![],
            ret: Some(RustType::Unit),
            body: vec![
                RustStmt::Let {
                    mutable: true,
                    name: "x".to_string(),
                    ty: Some(RustType::I64),
                    value: RustExpr::Literal(RustLiteral::Int(0)),
                },
                RustStmt::AugAssign {
                    target: RustExpr::Ident("x".to_string()),
                    op: "+".to_string(),
                    value: RustExpr::Literal(RustLiteral::Int(1)),
                },
                RustStmt::AugAssign {
                    target: RustExpr::Ident("x".to_string()),
                    op: "+=".to_string(),
                    value: RustExpr::Literal(RustLiteral::Int(2)),
                },
            ],
            is_async: false,
        }]);

        assert!(rendered.contains("x += 1;"));
        assert!(rendered.contains("x += 2;"));
        assert!(!rendered.contains("+=="));
    }

    #[test]
    fn renders_function_type_param_bounds() {
        let rendered = render_items(&[RustItem::Fn {
            name: "identity".to_string(),
            visibility: Visibility::Pub,
            type_params: vec![RustTypeParam {
                name: "T".to_string(),
                bounds: vec!["Clone".to_string(), "std::fmt::Display".to_string()],
            }],
            params: vec![RustParam::Named {
                name: "value".to_string(),
                ty: RustType::Named("T".to_string()),
            }],
            ret: Some(RustType::Named("T".to_string())),
            body: vec![RustStmt::Return(Some(RustExpr::Ident("value".to_string())))],
            is_async: false,
        }]);

        assert_snapshot!(rendered, @r###"
        pub fn identity<T: Clone + ::std::fmt::Display>(value: T) -> T {
            value
        }
        "###);
    }

    #[test]
    fn compiler_paths_are_absolute_without_rewriting_source_values() {
        assert_eq!(
            render_expr(&RustExpr::Ident(
                "std::sync::Arc::new(tokio::sync::Mutex::new(value))".to_string()
            )),
            "::std::sync::Arc::new(::tokio::sync::Mutex::new(value))"
        );
        assert_eq!(
            render_expr(&RustExpr::Path(vec![
                "std".to_string(),
                "convert".to_string(),
                "Into".to_string(),
            ])),
            "::std::convert::Into"
        );
        assert_eq!(render_expr(&RustExpr::Ident("std".to_string())), "std");
        assert_eq!(
            render_expr(&RustExpr::Literal(RustLiteral::Str(
                "std::path remains data".to_string()
            ))),
            "\"std::path remains data\".to_string()"
        );
        assert_eq!(
            render_expr(&RustExpr::Ident(
                r##"format!("std::data"); let raw = r#"tokio::data"#; std::path::Path::new()"##
                    .to_string()
            )),
            r##"format!("std::data"); let raw = r#"tokio::data"#; ::std::path::Path::new()"##
        );
        assert_eq!(
            Renderer::render_compiler_path_string(
                "/* std::comment */ tokio::spawn // rayon::comment\nrayon::join"
            ),
            "/* std::comment */ ::tokio::spawn // rayon::comment\n::rayon::join"
        );
        assert_eq!(
            Renderer::render_compiler_path_string(
                "not_std::value std::value ::std::value xstd::value"
            ),
            "not_std::value ::std::value ::std::value xstd::value"
        );
        assert_eq!(
            Renderer::render_compiler_path_string(
                r####"r###"std::raw"### b"tokio::bytes" 's' /* core::nested /* serde::nested */ */ core::value"####
            ),
            r####"r###"std::raw"### b"tokio::bytes" 's' /* core::nested /* serde::nested */ */ ::core::value"####
        );
    }

    #[test]
    fn renders_expression_variants() {
        let expr = RustExpr::FormatMacro {
            name: "format".to_string(),
            format_str: "{}-{}".to_string(),
            args: vec![
                RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("left".to_string())),
                    method: "trim".to_string(),
                    args: vec![],
                },
                RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "x".to_string(),
                        ty: RustType::I64,
                    }],
                    body: Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::Ident("x".to_string())),
                        op: "+".to_string(),
                        right: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
                    }),
                    is_move: true,
                },
            ],
        };

        let rendered = render_expr(&expr);
        assert_snapshot!(rendered, @r###"format!("{}-{}", left.trim(), move |x| x + 1)"###);
    }

    #[test]
    fn renders_method_call_on_range_receiver_with_parentheses() {
        let expr = RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Range {
                start: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
                end: Box::new(RustExpr::Literal(RustLiteral::Int(10))),
            }),
            method: "step_by".to_string(),
            args: vec![RustExpr::Cast {
                expr: Box::new(RustExpr::Literal(RustLiteral::Int(2))),
                ty: RustType::Named("usize".to_string()),
            }],
        };

        let rendered = render_expr(&expr);
        assert_eq!(rendered, "(1..10).step_by(2_usize)");
    }

    #[test]
    fn renders_special_float_literals_with_rust_constants() {
        assert_eq!(
            render_expr(&RustExpr::Literal(RustLiteral::Float(f64::INFINITY))),
            "f64::INFINITY"
        );
        assert_eq!(
            render_expr(&RustExpr::Literal(RustLiteral::Float(f64::NEG_INFINITY))),
            "f64::NEG_INFINITY"
        );
        assert_eq!(
            render_expr(&RustExpr::Literal(RustLiteral::Float(f64::NAN))),
            "f64::NAN"
        );
    }

    #[test]
    fn renders_parenthesized_expression_node() {
        let expr = RustExpr::Paren(Box::new(RustExpr::BinOp {
            left: Box::new(RustExpr::Ident("a".to_string())),
            op: "+".to_string(),
            right: Box::new(RustExpr::Ident("b".to_string())),
        }));

        let rendered = render_expr(&expr);
        assert_eq!(rendered, "(a + b)");
    }

    #[test]
    fn renders_slice_expression() {
        let expr = RustExpr::Slice {
            expr: Box::new(RustExpr::Ident("values".to_string())),
            start: Some(Box::new(RustExpr::Literal(RustLiteral::Int(1)))),
            stop: Some(Box::new(RustExpr::Literal(RustLiteral::Int(3)))),
        };

        let rendered = render_expr(&expr);
        assert_eq!(rendered, "values[1..3]");
    }

    #[test]
    fn renders_write_macro_format_arg_as_literal() {
        let expr = RustExpr::MacroCall {
            name: "write".to_string(),
            args: vec![
                RustExpr::Ident("f".to_string()),
                RustExpr::Literal(RustLiteral::Str("{}".to_string())),
                RustExpr::Ident("v".to_string()),
            ],
        };

        let rendered = render_expr(&expr);
        assert_eq!(rendered, "write!(f, \"{}\", v)");
    }

    #[test]
    fn renders_empty_println_format_macro_without_empty_string_literal() {
        let expr = RustExpr::FormatMacro {
            name: "println".to_string(),
            format_str: String::new(),
            args: vec![],
        };

        let rendered = render_expr(&expr);
        assert_eq!(rendered, "println!()");
    }

    #[test]
    fn render_stmts_helper_renders_block() {
        let stmts = vec![
            RustStmt::Let {
                mutable: false,
                name: "x".to_string(),
                ty: Some(RustType::I64),
                value: RustExpr::Literal(RustLiteral::Int(1)),
            },
            RustStmt::LetDecl {
                mutable: true,
                name: "pending".to_string(),
                ty: RustType::String_,
            },
            RustStmt::LetPattern {
                pattern: "(a, b)".to_string(),
                value: RustExpr::Tuple(vec![
                    RustExpr::Literal(RustLiteral::Int(2)),
                    RustExpr::Literal(RustLiteral::Bool(true)),
                ]),
            },
            RustStmt::Expr(RustExpr::Try(Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Ident("may_fail".to_string())),
                args: vec![],
            }))),
        ];
        let rendered = render_stmts(&stmts);
        assert_snapshot!(rendered, @r###"
        let x: i64 = 1;
        let mut pending: String;
        let (a, b) = (2, true);
        may_fail()?;
        "###);
    }

    #[test]
    fn render_stmts_renders_assert_variants() {
        let stmts = vec![
            RustStmt::Assert {
                cond: RustExpr::Literal(RustLiteral::Bool(true)),
                msg: None,
            },
            RustStmt::Assert {
                cond: RustExpr::Literal(RustLiteral::Bool(false)),
                msg: Some(RustExpr::Literal(RustLiteral::Str("boom".to_string()))),
            },
        ];
        let rendered = render_stmts(&stmts);
        assert_snapshot!(rendered, @r###"
        assert!(true);
        assert!(false, "{}", "boom".to_string());
        "###);
    }

    #[test]
    fn render_stmts_escapes_keyword_identifiers_and_patterns() {
        let stmts = vec![
            RustStmt::Let {
                mutable: true,
                name: "mod".to_string(),
                ty: Some(RustType::I64),
                value: RustExpr::Literal(RustLiteral::Int(7)),
            },
            RustStmt::LetPattern {
                pattern: "(mut res, mod, gen)".to_string(),
                value: RustExpr::Tuple(vec![
                    RustExpr::Literal(RustLiteral::Int(1)),
                    RustExpr::Literal(RustLiteral::Int(2)),
                    RustExpr::Literal(RustLiteral::Int(3)),
                ]),
            },
            RustStmt::Expr(RustExpr::BinOp {
                left: Box::new(RustExpr::Ident("res".to_string())),
                op: "%".to_string(),
                right: Box::new(RustExpr::Ident("mod".to_string())),
            }),
        ];

        let rendered = render_stmts(&stmts);
        assert_snapshot!(rendered, @r###"
        let mut r#mod: i64 = 7;
        let (mut res, r#mod, r#gen) = (1, 2, 3);
        res % r#mod;
        "###);
    }
}
