use super::*;
use sifr_hir::*;
use sifr_type_system::{Type, ParamConvention};

#[test]
fn test_simple_function_codegen() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Expr {
                expr: HirExpr::Call {
                    func: "print".to_string(),
                    args: vec![HirExpr::StringLiteral("Hello, World!".to_string())],
                    ty: Type::None,
                },
            }],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let rust_code = generate_rust(&module);
    assert!(rust_code.contains("fn main()"));
    assert!(rust_code.contains("println!"));
    assert!(rust_code.contains("Hello, World!"));
}

#[test]
fn test_arithmetic_codegen() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "add".to_string(),
            params: vec![
                HirParam { name: "a".to_string(), ty: Type::Int, default: None, keyword_only: false, convention: ParamConvention::Own },
                HirParam { name: "b".to_string(), ty: Type::Int, default: None, keyword_only: false, convention: ParamConvention::Own },
            ],
            return_type: Type::Int,
            body: vec![HirStmt::Return {
                value: Some(HirExpr::BinOp {
                    left: Box::new(HirExpr::Name { name: "a".to_string(), ty: Type::Int }),
                    op: "+".to_string(),
                    right: Box::new(HirExpr::Name { name: "b".to_string(), ty: Type::Int }),
                    ty: Type::Int,
                }),
            }],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let rust_code = generate_rust(&module);
    assert!(rust_code.contains("fn add(a: i64, b: i64) -> i64"));
    assert!(rust_code.contains("return a + b;"));
}

// --- Codegen Quality Tests ---

#[test]
fn test_no_unnecessary_mut() {
    // Variable that is never reassigned should NOT have `mut`
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![
                HirStmt::Let {
                    name: "x".to_string(),
                    ty: Type::Int,
                    value: HirExpr::IntLiteral(42),
                    is_mutable: true, // HIR says mutable, but codegen should ignore
                },
                HirStmt::Expr {
                    expr: HirExpr::Call {
                        func: "print".to_string(),
                        args: vec![HirExpr::Name { name: "x".to_string(), ty: Type::Int }],
                        ty: Type::None,
                    },
                },
            ],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let rust_code = generate_rust(&module);
    assert!(rust_code.contains("let x: i64"), "should emit `let x` without mut");
    assert!(!rust_code.contains("let mut x"), "should NOT emit `let mut x`");
}

#[test]
fn test_mut_on_reassigned_variable() {
    // Variable that IS reassigned should have `mut`
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![
                HirStmt::Let {
                    name: "x".to_string(),
                    ty: Type::Int,
                    value: HirExpr::IntLiteral(0),
                    is_mutable: true,
                },
                HirStmt::Assign {
                    name: "x".to_string(),
                    value: HirExpr::IntLiteral(1),
                },
            ],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let rust_code = generate_rust(&module);
    assert!(rust_code.contains("let mut x: i64"), "should emit `let mut x` for reassigned var");
}

#[test]
fn test_println_fstring_inlined() {
    // print(f"hello {name}") should emit println!("hello {}", name) not println!("{}", format!(...))
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![
                HirStmt::Let {
                    name: "name".to_string(),
                    ty: Type::Str,
                    value: HirExpr::StringLiteral("World".to_string()),
                    is_mutable: false,
                },
                HirStmt::Expr {
                    expr: HirExpr::Call {
                        func: "print".to_string(),
                        args: vec![HirExpr::FString {
                            parts: vec![
                                HirFStringPart::Literal("Hello, ".to_string()),
                                HirFStringPart::Expr(HirExpr::Name { name: "name".to_string(), ty: Type::Str }),
                                HirFStringPart::Literal("!".to_string()),
                            ],
                            ty: Type::Str,
                        }],
                        ty: Type::None,
                    },
                },
            ],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let rust_code = generate_rust(&module);
    assert!(rust_code.contains("println!(\"Hello, {}!\", name)"), "should inline f-string into println!");
    assert!(!rust_code.contains("format!(\"Hello, {}!\""), "should NOT have standalone format! inside println!");
}

#[test]
fn test_no_tostring_in_println() {
    // print("hello") should emit println!("{}", "hello") not println!("{}", "hello".to_string())
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Expr {
                expr: HirExpr::Call {
                    func: "print".to_string(),
                    args: vec![HirExpr::StringLiteral("hello".to_string())],
                    ty: Type::None,
                },
            }],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let rust_code = generate_rust(&module);
    assert!(rust_code.contains("println!(\"hello\")"), "should inline string literal directly into println!");
    assert!(!rust_code.contains("\"hello\".to_string()"), "should NOT have .to_string() in println context");
}

#[test]
fn test_hashmap_short_name() {
    // Dict literal should use HashMap::from not std::collections::HashMap::from
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Let {
                name: "d".to_string(),
                ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
                value: HirExpr::DictLiteral {
                    keys: vec![HirExpr::StringLiteral("a".to_string())],
                    values: vec![HirExpr::IntLiteral(1)],
                    ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
                },
                is_mutable: false,
            }],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let rust_code = generate_rust(&module);
    assert!(rust_code.contains("use std::collections::HashMap;"), "should have HashMap import");
    assert!(rust_code.contains("HashMap::from("), "should use short HashMap::from");
    assert!(!rust_code.contains("std::collections::HashMap::from("), "should NOT use fully qualified HashMap::from");
    assert!(rust_code.contains("HashMap<String, i64>"), "type annotation should use short HashMap");
}

#[test]
fn test_dict_get_string_literal_key() {
    // d["key"] should emit d.get("key") not d.get(&"key".to_string())
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![
                HirStmt::Let {
                    name: "d".to_string(),
                    ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
                    value: HirExpr::DictLiteral {
                        keys: vec![HirExpr::StringLiteral("key".to_string())],
                        values: vec![HirExpr::IntLiteral(1)],
                        ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
                    },
                    is_mutable: false,
                },
                HirStmt::Let {
                    name: "v".to_string(),
                    ty: Type::Union(vec![Type::Int, Type::None]),
                    value: HirExpr::Index {
                        object: Box::new(HirExpr::Name {
                            name: "d".to_string(),
                            ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
                        }),
                        index: Box::new(HirExpr::StringLiteral("key".to_string())),
                        ty: Type::Union(vec![Type::Int, Type::None]),
                    },
                    is_mutable: false,
                },
            ],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let rust_code = generate_rust(&module);
    assert!(rust_code.contains(".get(\"key\")"), "should emit .get(\"key\") for string literal key");
    assert!(!rust_code.contains("&\"key\".to_string()"), "should NOT have &\"key\".to_string()");
}

#[test]
fn test_string_concat_flattened() {
    // "a" + "b" + "c" should emit format!("{}{}{}", ...) not nested format!
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Let {
                name: "s".to_string(),
                ty: Type::Str,
                value: HirExpr::BinOp {
                    left: Box::new(HirExpr::BinOp {
                        left: Box::new(HirExpr::StringLiteral("a".to_string())),
                        op: "+".to_string(),
                        right: Box::new(HirExpr::StringLiteral("b".to_string())),
                        ty: Type::Str,
                    }),
                    op: "+".to_string(),
                    right: Box::new(HirExpr::StringLiteral("c".to_string())),
                    ty: Type::Str,
                },
                is_mutable: false,
            }],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let rust_code = generate_rust(&module);
    // All parts are string literals, so they should be folded into a single string
    assert!(rust_code.contains("\"abc\".to_string()"), "should fold all string literals into a single string");
    assert!(!rust_code.contains("format!"), "should NOT use format! when all parts are literals");
}

#[test]
fn test_mut_on_mutating_method_call() {
    // Variable with .push() call should have `mut`
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![
                HirStmt::Let {
                    name: "items".to_string(),
                    ty: Type::List(Box::new(Type::Int)),
                    value: HirExpr::ListLiteral {
                        elements: vec![HirExpr::IntLiteral(1)],
                        ty: Type::List(Box::new(Type::Int)),
                    },
                    is_mutable: true,
                },
                HirStmt::Expr {
                    expr: HirExpr::MethodCall {
                        object: Box::new(HirExpr::Name {
                            name: "items".to_string(),
                            ty: Type::List(Box::new(Type::Int)),
                        }),
                        method: "append".to_string(),
                        args: vec![HirExpr::IntLiteral(2)],
                        ty: Type::None,
                    },
                },
            ],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let rust_code = generate_rust(&module);
    assert!(rust_code.contains("let mut items"), "should emit `let mut items` for variable with .push()");
}

#[test]
fn test_empty_print() {
    // print() should emit println!() not println!("{}", "")
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Expr {
                expr: HirExpr::Call {
                    func: "print".to_string(),
                    args: vec![],
                    ty: Type::None,
                },
            }],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let rust_code = generate_rust(&module);
    assert!(rust_code.contains("println!()"), "should emit println!() for empty print");
    assert!(!rust_code.contains(r#"println!("{}", "")"#), "should NOT emit println with empty string arg");
}

#[test]
fn test_expr_to_string_leaf_rendering() {
    let mut emitter = RustEmitter::new();
    let int_code = emitter.expr_to_string(&HirExpr::IntLiteral(7));
    assert_eq!(int_code, "7 as i64");

    let bool_op = HirExpr::BoolOp {
        op: "and".to_string(),
        values: vec![HirExpr::BoolLiteral(true), HirExpr::BoolLiteral(false)],
        ty: Type::Bool,
    };
    let bool_code = emitter.expr_to_string(&bool_op);
    assert_eq!(bool_code, "true && false");
}

#[test]
fn test_match_int_literal_pattern_avoids_cast_expression() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![
                HirStmt::Let {
                    name: "x".to_string(),
                    ty: Type::Int,
                    value: HirExpr::IntLiteral(1),
                    is_mutable: false,
                },
                HirStmt::Match {
                    subject: HirExpr::Name {
                        name: "x".to_string(),
                        ty: Type::Int,
                    },
                    subject_ty: Type::Int,
                    arms: vec![
                        HirMatchArm {
                            pattern: HirPattern::Literal {
                                value: HirExpr::IntLiteral(1),
                            },
                            guard: None,
                            body: vec![HirStmt::Expr {
                                expr: HirExpr::Call {
                                    func: "print".to_string(),
                                    args: vec![HirExpr::StringLiteral("one".to_string())],
                                    ty: Type::None,
                                },
                            }],
                        },
                        HirMatchArm {
                            pattern: HirPattern::Wildcard,
                            guard: None,
                            body: vec![HirStmt::Expr {
                                expr: HirExpr::Call {
                                    func: "print".to_string(),
                                    args: vec![HirExpr::StringLiteral("other".to_string())],
                                    ty: Type::None,
                                },
                            }],
                        },
                    ],
                },
            ],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let rust_code = generate_rust(&module);
    assert!(rust_code.contains("1 => {"));
    assert!(!rust_code.contains("1 as i64 => {"));
}

#[test]
fn test_generate_rust_multi_exports_non_main_items() {
    let main_module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Pass],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let utils_module = HirModule {
        functions: vec![HirFunction {
            name: "helper".to_string(),
            params: vec![],
            return_type: Type::Int,
            body: vec![HirStmt::Return {
                value: Some(HirExpr::IntLiteral(7)),
            }],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![HirClass {
            name: "Thing".to_string(),
            fields: vec![("value".to_string(), Type::Int)],
            methods: vec![],
            is_hashable: false,
            is_error_type: false,
            is_protocol: false,
            operator_impls: vec![],
            newtype_inner: None,
            implements_protocols: vec![],
            parent_class: None,
            type_params: vec![],
            is_enum: false,
            enum_variants: vec![],
        }],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let files = generate_rust_multi(&[("main", &main_module), ("utils", &utils_module)]);
    let main_rs = files.get("main").expect("main module should be generated");
    let utils_rs = files.get("utils").expect("utils module should be generated");

    assert!(main_rs.contains("fn main()"));
    assert!(!main_rs.contains("pub fn main("));
    assert!(utils_rs.contains("pub fn helper() -> i64"));
    assert!(utils_rs.contains("pub struct Thing"));
    assert!(utils_rs.contains("pub value: i64"));
    assert!(utils_rs.contains("pub fn new(value: i64) -> Self"));
}

#[test]
fn test_nested_break_without_inner_else_does_not_set_outer_broke_flag() {
    let int_list_ty = Type::List(Box::new(Type::Int));
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::For {
                target: "i".to_string(),
                target_ty: Type::Int,
                iter: HirExpr::ListLiteral {
                    elements: vec![HirExpr::IntLiteral(1)],
                    ty: int_list_ty.clone(),
                },
                body: vec![HirStmt::For {
                    target: "j".to_string(),
                    target_ty: Type::Int,
                    iter: HirExpr::ListLiteral {
                        elements: vec![HirExpr::IntLiteral(1)],
                        ty: int_list_ty,
                    },
                    body: vec![HirStmt::Break],
                    else_body: None,
                }],
                else_body: Some(vec![HirStmt::Expr {
                    expr: HirExpr::Call {
                        func: "print".to_string(),
                        args: vec![HirExpr::StringLiteral("outer else".to_string())],
                        ty: Type::None,
                    },
                }]),
            }],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let rust_code = generate_rust(&module);
    assert_eq!(rust_code.matches("let mut _broke = false;").count(), 1);
    assert!(rust_code.contains("if !_broke {"));
    assert!(!rust_code.contains("_broke = true;"));
}

#[test]
fn test_generate_rust_test_uses_explicit_test_mode_context() {
    let module = HirModule {
        functions: vec![
            HirFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![HirStmt::Pass],
                method_kind: MethodKind::Regular,
                decorators: vec![],
                type_params: vec![],
            },
            HirFunction {
                name: "test_sample".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![HirStmt::Pass],
                method_kind: MethodKind::Regular,
                decorators: vec![],
                type_params: vec![],
            },
            HirFunction {
                name: "helper".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![HirStmt::Pass],
                method_kind: MethodKind::Regular,
                decorators: vec![],
                type_params: vec![],
            },
        ],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let rust_code = generate_rust_test(&module).rust_source;
    assert!(!rust_code.contains("fn main("));
    assert!(rust_code.contains("#[test]\nfn test_sample()"));
    assert!(rust_code.contains("fn helper()"));
    assert!(!rust_code.contains("#[test]\nfn helper()"));
}

#[test]
fn test_self_field_clone_suppression_is_scoped_and_non_sticky() {
    let items_ty = Type::List(Box::new(Type::Int));
    let table_ty = Type::Dict(Box::new(Type::Str), Box::new(Type::Int));
    let label_ty = Type::Str;
    let class_ty = Type::Class {
        name: "Bucket".to_string(),
        fields: vec![
            ("items".to_string(), items_ty.clone()),
            ("table".to_string(), table_ty.clone()),
            ("label".to_string(), label_ty.clone()),
        ],
        methods: vec![],
        parent_class: None,
    };

    let module = HirModule {
        functions: vec![],
        classes: vec![HirClass {
            name: "Bucket".to_string(),
            fields: vec![
                ("items".to_string(), items_ty.clone()),
                ("table".to_string(), table_ty.clone()),
                ("label".to_string(), label_ty.clone()),
            ],
            methods: vec![
                HirFunction {
                    name: "append_item".to_string(),
                    params: vec![HirParam {
                        name: "x".to_string(),
                        ty: Type::Int,
                        default: None,
                        keyword_only: false,
                        convention: ParamConvention::Own,
                    }],
                    return_type: Type::None,
                    body: vec![HirStmt::Expr {
                        expr: HirExpr::MethodCall {
                            object: Box::new(HirExpr::FieldAccess {
                                object: Box::new(HirExpr::Name {
                                    name: "self".to_string(),
                                    ty: class_ty.clone(),
                                }),
                                field: "items".to_string(),
                                ty: items_ty.clone(),
                            }),
                            method: "append".to_string(),
                            args: vec![HirExpr::Name {
                                name: "x".to_string(),
                                ty: Type::Int,
                            }],
                            ty: Type::None,
                        },
                    }],
                    method_kind: MethodKind::Regular,
                    decorators: vec![],
                    type_params: vec![],
                },
                HirFunction {
                    name: "read_table".to_string(),
                    params: vec![],
                    return_type: Type::Union(vec![Type::Int, Type::None]),
                    body: vec![HirStmt::Return {
                        value: Some(HirExpr::Index {
                            object: Box::new(HirExpr::FieldAccess {
                                object: Box::new(HirExpr::Name {
                                    name: "self".to_string(),
                                    ty: class_ty.clone(),
                                }),
                                field: "table".to_string(),
                                ty: table_ty.clone(),
                            }),
                            index: Box::new(HirExpr::StringLiteral("k".to_string())),
                            ty: Type::Union(vec![Type::Int, Type::None]),
                        }),
                    }],
                    method_kind: MethodKind::Regular,
                    decorators: vec![],
                    type_params: vec![],
                },
                HirFunction {
                    name: "leak_guard".to_string(),
                    params: vec![],
                    return_type: Type::Str,
                    body: vec![
                        HirStmt::Let {
                            name: "d".to_string(),
                            ty: table_ty.clone(),
                            value: HirExpr::DictLiteral {
                                keys: vec![HirExpr::StringLiteral("k".to_string())],
                                values: vec![HirExpr::IntLiteral(1)],
                                ty: table_ty.clone(),
                            },
                            is_mutable: false,
                        },
                        HirStmt::Expr {
                            expr: HirExpr::Index {
                                object: Box::new(HirExpr::Name {
                                    name: "d".to_string(),
                                    ty: table_ty.clone(),
                                }),
                                index: Box::new(HirExpr::StringLiteral("k".to_string())),
                                ty: Type::Union(vec![Type::Int, Type::None]),
                            },
                        },
                        HirStmt::Return {
                            value: Some(HirExpr::FieldAccess {
                                object: Box::new(HirExpr::Name {
                                    name: "self".to_string(),
                                    ty: class_ty,
                                }),
                                field: "label".to_string(),
                                ty: label_ty,
                            }),
                        },
                    ],
                    method_kind: MethodKind::Regular,
                    decorators: vec![],
                    type_params: vec![],
                },
            ],
            is_hashable: false,
            is_error_type: false,
            is_protocol: false,
            operator_impls: vec![],
            newtype_inner: None,
            implements_protocols: vec![],
            parent_class: None,
            type_params: vec![],
            is_enum: false,
            enum_variants: vec![],
        }],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let rust_code = generate_rust(&module);
    assert!(rust_code.contains("self.items.push(x);"));
    assert!(!rust_code.contains("self.items.clone().push(x)"));
    assert!(rust_code.contains("return self.table.get(\"k\").cloned();"));
    assert!(!rust_code.contains("self.table.clone().get(\"k\")"));
    assert!(rust_code.contains("return self.label.clone();"));
}
