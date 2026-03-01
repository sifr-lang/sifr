use super::*;
use sifr_hir::*;
use sifr_type_system::{ParamConvention, Type};

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
                HirParam {
                    name: "a".to_string(),
                    ty: Type::Int,
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::Own,
                },
                HirParam {
                    name: "b".to_string(),
                    ty: Type::Int,
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::Own,
                },
            ],
            return_type: Type::Int,
            body: vec![HirStmt::Return {
                value: Some(HirExpr::BinOp {
                    left: Box::new(HirExpr::Name {
                        name: "a".to_string(),
                        ty: Type::Int,
                    }),
                    op: "+".to_string(),
                    right: Box::new(HirExpr::Name {
                        name: "b".to_string(),
                        ty: Type::Int,
                    }),
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
                        args: vec![HirExpr::Name {
                            name: "x".to_string(),
                            ty: Type::Int,
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
    assert!(
        rust_code.contains("let x: i64"),
        "should emit `let x` without mut"
    );
    assert!(
        !rust_code.contains("let mut x"),
        "should NOT emit `let mut x`"
    );
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
    assert!(
        rust_code.contains("let mut x: i64"),
        "should emit `let mut x` for reassigned var"
    );
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
                                HirFStringPart::Expr(HirExpr::Name {
                                    name: "name".to_string(),
                                    ty: Type::Str,
                                }),
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
    assert!(
        rust_code.contains("println!(\"Hello, {}!\", name)"),
        "should inline f-string into println!"
    );
    assert!(
        !rust_code.contains("format!(\"Hello, {}!\""),
        "should NOT have standalone format! inside println!"
    );
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
    assert!(
        rust_code.contains("println!(\"hello\")"),
        "should inline string literal directly into println!"
    );
    assert!(
        !rust_code.contains("\"hello\".to_string()"),
        "should NOT have .to_string() in println context"
    );
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
    assert!(
        rust_code.contains("use std::collections::HashMap;"),
        "should have HashMap import"
    );
    assert!(
        rust_code.contains("HashMap::from("),
        "should use short HashMap::from"
    );
    assert!(
        !rust_code.contains("std::collections::HashMap::from("),
        "should NOT use fully qualified HashMap::from"
    );
    assert!(
        rust_code.contains("HashMap<String, i64>"),
        "type annotation should use short HashMap"
    );
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
    assert!(
        rust_code.contains(".get(\"key\")"),
        "should emit .get(\"key\") for string literal key"
    );
    assert!(
        !rust_code.contains("&\"key\".to_string()"),
        "should NOT have &\"key\".to_string()"
    );
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
    assert!(
        rust_code.contains("\"abc\".to_string()"),
        "should fold all string literals into a single string"
    );
    assert!(
        !rust_code.contains("format!"),
        "should NOT use format! when all parts are literals"
    );
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
    assert!(
        rust_code.contains("let mut items"),
        "should emit `let mut items` for variable with .push()"
    );
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
    assert!(
        rust_code.contains("println!()"),
        "should emit println!() for empty print"
    );
    assert!(
        !rust_code.contains(r#"println!("{}", "")"#),
        "should NOT emit println with empty string arg"
    );
}

#[test]
fn test_expr_to_string_leaf_rendering() {
    let mut emitter = RustEmitter::new();
    let int_code = emitter.render_expr_with_lowered_path(&HirExpr::IntLiteral(7));
    assert_eq!(int_code, "7 as i64");

    let bool_op = HirExpr::BoolOp {
        op: "and".to_string(),
        values: vec![HirExpr::BoolLiteral(true), HirExpr::BoolLiteral(false)],
        ty: Type::Bool,
    };
    let bool_code = emitter.render_expr_with_lowered_path(&bool_op);
    assert_eq!(bool_code, "true && false");
}

#[test]
fn test_render_expr_lowering_rewrites_stdlib_constant_idents() {
    let mut emitter = RustEmitter::new();
    emitter.intrinsic_functions.insert("pi".to_string());
    let expr = HirExpr::BinOp {
        left: Box::new(HirExpr::Name {
            name: "pi".to_string(),
            ty: Type::Float,
        }),
        op: "+".to_string(),
        right: Box::new(HirExpr::FloatLiteral(1.0)),
        ty: Type::Float,
    };

    let code = emitter.render_expr_with_lowered_path(&expr);
    assert!(code.contains("std::f64::consts::PI"));
    assert!(!code.contains("pi +"));
}

#[test]
fn test_render_expr_lowering_rewrites_module_constant_ident() {
    let mut emitter = RustEmitter::new();
    emitter
        .module_constants
        .insert("limit".to_string(), (Type::Int, "LIMIT".to_string()));
    let expr = HirExpr::BinOp {
        left: Box::new(HirExpr::Name {
            name: "limit".to_string(),
            ty: Type::Int,
        }),
        op: "+".to_string(),
        right: Box::new(HirExpr::IntLiteral(1)),
        ty: Type::Int,
    };

    let code = emitter.render_expr_with_lowered_path(&expr);
    assert!(code.contains("LIMIT +"));
}

#[test]
fn test_render_expr_lowering_rewrites_module_constant_helper_call() {
    let mut emitter = RustEmitter::new();
    emitter.module_constants.insert(
        "greeting".to_string(),
        (Type::Str, "__const_greeting()".to_string()),
    );
    let expr = HirExpr::Name {
        name: "greeting".to_string(),
        ty: Type::Str,
    };

    let code = emitter.render_expr_with_lowered_path(&expr);
    assert_eq!(code, "__const_greeting()");
}

#[test]
fn test_structured_stmt_path_rewrites_module_constant_name() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Let {
                name: "x".to_string(),
                ty: Type::Int,
                value: HirExpr::Name {
                    name: "limit".to_string(),
                    ty: Type::Int,
                },
                is_mutable: false,
            }],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![("limit".to_string(), Type::Int, HirExpr::IntLiteral(7))],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let result = generate_rust_with_metadata(&module);
    assert!(result.rust_source.contains("const LIMIT: i64 = 7 as i64;"));
    assert!(result.rust_source.contains("let x: i64 = LIMIT;"));
    assert!(result.lowering_stats.stmt_structured >= 1);
}

#[test]
fn test_structured_stmt_path_rewrites_stdlib_constant_name() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Let {
                name: "x".to_string(),
                ty: Type::Float,
                value: HirExpr::Name {
                    name: "pi".to_string(),
                    ty: Type::Float,
                },
                is_mutable: false,
            }],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![HirImport {
            module: "sifr.math".to_string(),
            names: vec!["pi".to_string()],
            aliases: vec![],
        }],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let result = generate_rust_with_metadata(&module);
    assert!(result
        .rust_source
        .contains("let x: f64 = std::f64::consts::PI;"));
    assert!(!result.rust_source.contains("let x: f64 = pi;"));
    assert!(result.lowering_stats.stmt_structured >= 1);
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
            kind: HirClassKind::Regular,
            operator_impls: vec![],
            newtype_inner: None,
            implements_protocols: vec![],
            parent_class: None,
            type_params: vec![],
            enum_variants: vec![],
        }],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let files = generate_rust_multi(&[("main", &main_module), ("utils", &utils_module)]);
    let main_rs = files.get("main").expect("main module should be generated");
    let utils_rs = files
        .get("utils")
        .expect("utils module should be generated");

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
fn test_generate_rust_test_collects_imports_from_emitted_code() {
    let module = HirModule {
        functions: vec![
            HirFunction {
                name: "test_collections".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![
                    HirStmt::Expr {
                        expr: HirExpr::DictLiteral {
                            keys: vec![HirExpr::StringLiteral("k".to_string())],
                            values: vec![HirExpr::IntLiteral(1)],
                            ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
                        },
                    },
                    HirStmt::Expr {
                        expr: HirExpr::SetLiteral {
                            elements: vec![HirExpr::IntLiteral(1)],
                            ty: Type::Set(Box::new(Type::Int)),
                        },
                    },
                ],
                method_kind: MethodKind::Regular,
                decorators: vec![],
                type_params: vec![],
            },
            HirFunction {
                name: "helper_bigint".to_string(),
                params: vec![HirParam {
                    name: "x".to_string(),
                    ty: Type::BigInt,
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::Own,
                }],
                return_type: Type::BigInt,
                body: vec![HirStmt::Return {
                    value: Some(HirExpr::Name {
                        name: "x".to_string(),
                        ty: Type::BigInt,
                    }),
                }],
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

    let result = generate_rust_test(&module);
    assert!(result
        .rust_source
        .contains("use std::collections::HashMap;"));
    assert!(result
        .rust_source
        .contains("use std::collections::HashSet;"));
    assert!(result.rust_source.contains("use num_bigint::BigInt;"));
    assert!(result.required_crates.contains("num-bigint"));
    assert!(result.required_crates.contains("num-traits"));
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
            kind: HirClassKind::Regular,
            operator_impls: vec![],
            newtype_inner: None,
            implements_protocols: vec![],
            parent_class: None,
            type_params: vec![],
            enum_variants: vec![],
        }],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let rust_code = generate_rust(&module);
    assert!(
        rust_code.contains("self.items.push(x);")
            || rust_code.contains("(self.items).push(x);")
            || rust_code.contains("self.items.extend(")
            || rust_code.contains("(self.items).extend("),
        "{rust_code}"
    );
    assert!(!rust_code.contains("self.items.clone().push(x)"));
    assert!(rust_code.contains("return self.table.get(\"k\").cloned();"));
    assert!(!rust_code.contains("self.table.clone().get(\"k\")"));
    assert!(
        rust_code.contains("return self.label.clone();"),
        "{rust_code}"
    );
}

#[test]
fn test_codegen_structured_lowering_applies_to_simple_stmt() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Let {
                name: "x".to_string(),
                ty: Type::Int,
                value: HirExpr::IntLiteral(1),
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

    let generated = generate_rust_with_metadata(&module);

    assert!(generated.rust_source.contains("1 as i64"));
    assert!(generated.lowering_stats.stmt_structured > 0);
}

#[test]
fn test_structured_aug_assign_uses_string_and_list_methods() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![
                HirStmt::Let {
                    name: "s".to_string(),
                    ty: Type::Str,
                    value: HirExpr::StringLiteral("Hello".to_string()),
                    is_mutable: true,
                },
                HirStmt::AugAssign {
                    name: "s".to_string(),
                    op: "+=".to_string(),
                    value: HirExpr::StringLiteral("World".to_string()),
                },
                HirStmt::Let {
                    name: "items".to_string(),
                    ty: Type::List(Box::new(Type::Int)),
                    value: HirExpr::ListLiteral {
                        elements: vec![HirExpr::IntLiteral(1)],
                        ty: Type::List(Box::new(Type::Int)),
                    },
                    is_mutable: true,
                },
                HirStmt::AugAssign {
                    name: "items".to_string(),
                    op: "+=".to_string(),
                    value: HirExpr::ListLiteral {
                        elements: vec![HirExpr::IntLiteral(2)],
                        ty: Type::List(Box::new(Type::Int)),
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

    let generated = generate_rust_with_metadata(&module);
    assert!(generated.rust_source.contains("s.push_str("));
    assert!(generated
        .rust_source
        .contains("items.extend(vec![2 as i64])"));
    assert!(!generated.rust_source.contains("s += "));
    assert!(!generated.rust_source.contains("items += "));
}

#[test]
fn test_stmt_path_handles_nested_function() {
    let nested = HirFunction {
        name: "inner".to_string(),
        params: vec![],
        return_type: Type::Int,
        body: vec![HirStmt::Return {
            value: Some(HirExpr::IntLiteral(1)),
        }],
        method_kind: MethodKind::Regular,
        decorators: vec![],
        type_params: vec![],
    };

    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![
                HirStmt::NestedFunction { func: nested },
                HirStmt::Expr {
                    expr: HirExpr::Call {
                        func: "inner".to_string(),
                        args: vec![],
                        ty: Type::Int,
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

    let generated = generate_rust_with_metadata(&module);

    assert!(generated.rust_source.contains("let inner = || {"));
    assert!(generated.rust_source.contains("inner()"));
}

#[test]
fn test_expr_path_handles_call_expression() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Expr {
                expr: HirExpr::Call {
                    func: "print".to_string(),
                    args: vec![HirExpr::StringLiteral("marker".to_string())],
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

    let generated = generate_rust_with_metadata(&module);

    assert!(generated.rust_source.contains("println!"));
    assert!(generated.rust_source.contains("marker"));
}

#[test]
fn test_structured_expr_path_handles_intrinsic_call_expression() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Expr {
                expr: HirExpr::Call {
                    func: "sqrt".to_string(),
                    args: vec![HirExpr::FloatLiteral(9.0)],
                    ty: Type::Float,
                },
            }],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![HirImport {
            module: "sifr.math".to_string(),
            names: vec!["sqrt".to_string()],
            aliases: vec![],
        }],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module);
    assert!(generated.rust_source.contains("(9.0 as f64).sqrt()"));
    assert!(
        generated.lowering_stats.expr_structured > 0,
        "intrinsic call should be emitted through structured expr path"
    );
    assert!(
        generated.lowering_stats.stmt_structured > 0,
        "expression statement should be emitted through structured stmt path"
    );
}

#[test]
fn test_structured_expr_path_handles_nested_intrinsic_call_argument() {
    let list_ty = Type::List(Box::new(Type::Int));
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Expr {
                expr: HirExpr::Call {
                    func: "set_len".to_string(),
                    args: vec![HirExpr::Call {
                        func: "set_from_list".to_string(),
                        args: vec![HirExpr::ListLiteral {
                            elements: vec![HirExpr::IntLiteral(1), HirExpr::IntLiteral(2)],
                            ty: list_ty.clone(),
                        }],
                        ty: list_ty,
                    }],
                    ty: Type::Int,
                },
            }],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![HirImport {
            module: "sifr.collections".to_string(),
            names: vec!["set_len".to_string(), "set_from_list".to_string()],
            aliases: vec![],
        }],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module);
    assert!(
        generated.rust_source.contains(".len() as i64"),
        "nested intrinsic call argument should lower through registry"
    );
    assert!(
        !generated.rust_source.contains("set_len("),
        "set_len should not be emitted as unresolved function call"
    );
}

#[test]
fn test_structured_expr_path_handles_intrinsic_arg_with_typed_method_call() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Expr {
                expr: HirExpr::Call {
                    func: "isnan".to_string(),
                    args: vec![HirExpr::MethodCall {
                        object: Box::new(HirExpr::FloatLiteral(1.0)),
                        method: "max".to_string(),
                        args: vec![HirExpr::FloatLiteral(2.0)],
                        ty: Type::Float,
                    }],
                    ty: Type::Bool,
                },
            }],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![HirImport {
            module: "sifr.math".to_string(),
            names: vec!["isnan".to_string()],
            aliases: vec![],
        }],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module);
    assert!(
        generated.rust_source.contains(".is_nan()"),
        "intrinsic arg with typed method call should lower through registry"
    );
    assert!(
        !generated.rust_source.contains("isnan("),
        "isnan should not be emitted as unresolved function call"
    );
}

#[test]
fn test_structured_expr_path_handles_plain_signature_call_expression() {
    let module = HirModule {
        functions: vec![
            HirFunction {
                name: "helper".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![HirStmt::Expr {
                    expr: HirExpr::Call {
                        func: "print".to_string(),
                        args: vec![HirExpr::StringLiteral("inner".to_string())],
                        ty: Type::None,
                    },
                }],
                method_kind: MethodKind::Regular,
                decorators: vec![],
                type_params: vec![],
            },
            HirFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![HirStmt::Expr {
                    expr: HirExpr::Call {
                        func: "helper".to_string(),
                        args: vec![],
                        ty: Type::None,
                    },
                }],
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

    let generated = generate_rust_with_metadata(&module);
    assert!(generated.rust_source.contains("helper();"));
    assert!(
        generated.lowering_stats.expr_structured > 0,
        "plain calls with by-value signatures should use structured expr path"
    );
}

#[test]
fn test_structured_expr_path_handles_registry_method_call_expression() {
    let list_ty = Type::List(Box::new(Type::Int));
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![
                HirStmt::Let {
                    name: "items".to_string(),
                    ty: list_ty.clone(),
                    value: HirExpr::ListLiteral {
                        elements: vec![HirExpr::IntLiteral(1)],
                        ty: list_ty.clone(),
                    },
                    is_mutable: true,
                },
                HirStmt::Expr {
                    expr: HirExpr::MethodCall {
                        object: Box::new(HirExpr::Name {
                            name: "items".to_string(),
                            ty: list_ty,
                        }),
                        method: "clear".to_string(),
                        args: vec![],
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

    let generated = generate_rust_with_metadata(&module);
    assert!(generated.rust_source.contains("items.clear();"));
    assert!(
        generated.lowering_stats.expr_structured > 0,
        "registry-backed method call should be emitted through structured expr path"
    );
}

#[test]
fn test_registry_dict_update_with_typed_literal_arg_lowers_to_extend() {
    let dict_ty = Type::Dict(Box::new(Type::Str), Box::new(Type::Int));
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![
                HirStmt::Let {
                    name: "d2".to_string(),
                    ty: dict_ty.clone(),
                    value: HirExpr::DictLiteral {
                        keys: vec![HirExpr::StringLiteral("a".to_string())],
                        values: vec![HirExpr::IntLiteral(1)],
                        ty: dict_ty.clone(),
                    },
                    is_mutable: true,
                },
                HirStmt::Expr {
                    expr: HirExpr::MethodCall {
                        object: Box::new(HirExpr::Name {
                            name: "d2".to_string(),
                            ty: dict_ty.clone(),
                        }),
                        method: "update".to_string(),
                        args: vec![HirExpr::DictLiteral {
                            keys: vec![HirExpr::StringLiteral("c".to_string())],
                            values: vec![HirExpr::IntLiteral(3)],
                            ty: dict_ty.clone(),
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

    let generated = generate_rust_with_metadata(&module);
    assert!(
        generated.rust_source.contains("d2.extend("),
        "dict update should lower via registry to HashMap::extend"
    );
    assert!(
        !generated.rust_source.contains("d2.update("),
        "dict update method call should not be emitted"
    );
}

#[test]
fn test_structured_stmt_path_handles_copy_typed_assign_expr() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![
                HirStmt::Let {
                    name: "x".to_string(),
                    ty: Type::Float,
                    value: HirExpr::FloatLiteral(0.0),
                    is_mutable: true,
                },
                HirStmt::Assign {
                    name: "x".to_string(),
                    value: HirExpr::Call {
                        func: "sqrt".to_string(),
                        args: vec![HirExpr::FloatLiteral(9.0)],
                        ty: Type::Float,
                    },
                },
            ],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![HirImport {
            module: "sifr.math".to_string(),
            names: vec!["sqrt".to_string()],
            aliases: vec![],
        }],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module);
    assert!(generated.rust_source.contains("x = (9.0 as f64).sqrt();"));
    assert!(
        generated.lowering_stats.stmt_structured >= 2,
        "let + assign should be emitted through structured stmt path"
    );
}

#[test]
fn test_structured_stmt_path_handles_copy_typed_let_expr() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Let {
                name: "x".to_string(),
                ty: Type::Float,
                value: HirExpr::Call {
                    func: "sqrt".to_string(),
                    args: vec![HirExpr::FloatLiteral(9.0)],
                    ty: Type::Float,
                },
                is_mutable: false,
            }],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![HirImport {
            module: "sifr.math".to_string(),
            names: vec!["sqrt".to_string()],
            aliases: vec![],
        }],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module);
    assert!(generated
        .rust_source
        .contains("let x: f64 = (9.0 as f64).sqrt();"));
    assert!(
        generated.lowering_stats.stmt_structured >= 1,
        "copy-typed let should be emitted through structured stmt path"
    );
}

#[test]
fn test_structured_stmt_path_handles_copy_typed_return_expr() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "value".to_string(),
            params: vec![],
            return_type: Type::Float,
            body: vec![HirStmt::Return {
                value: Some(HirExpr::Call {
                    func: "sqrt".to_string(),
                    args: vec![HirExpr::FloatLiteral(9.0)],
                    ty: Type::Float,
                }),
            }],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![HirImport {
            module: "sifr.math".to_string(),
            names: vec!["sqrt".to_string()],
            aliases: vec![],
        }],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module);
    assert!(generated
        .rust_source
        .contains("return (9.0 as f64).sqrt();"));
    assert!(
        generated.lowering_stats.stmt_structured >= 1,
        "copy-typed return should be emitted through structured stmt path"
    );
}

#[test]
fn test_emit_expr_prefers_structured_name_path() {
    let mut emitter = RustEmitter::new();
    emitter.intrinsic_functions.insert("clock".to_string());
    let expr = HirExpr::Name {
        name: "clock".to_string(),
        ty: Type::Int,
    };

    emitter.emit_expr(&expr);

    assert_eq!(emitter.output, "clock");
    assert_eq!(emitter.lowering_stats.expr_structured, 1);
    assert_eq!(emitter.lowering_stats.expr_lowering_errors, 0);
}

#[test]
fn test_emit_expr_borrowed_compare_is_structured() {
    let mut emitter = RustEmitter::new();
    emitter.borrowed_params.insert("lhs".to_string());
    let expr = HirExpr::Compare {
        left: Box::new(HirExpr::Name {
            name: "lhs".to_string(),
            ty: Type::Str,
        }),
        ops: vec!["==".to_string()],
        comparators: vec![HirExpr::StringLiteral("ok".to_string())],
        ty: Type::Bool,
    };

    emitter.emit_expr(&expr);

    assert!(emitter.output.contains("lhs"));
    assert!(emitter.output.contains(".as_str() =="));
    assert_eq!(emitter.lowering_stats.expr_structured, 1);
    assert_eq!(emitter.lowering_stats.expr_lowering_errors, 0);
}

#[test]
fn test_lib_decomposition_guards_keep_stmt_expr_logic_out_of_lib_rs() {
    let lib_src = include_str!("lib.rs");
    let stmt_src = include_str!("stmt_emitter.rs");
    let expr_src = include_str!("expr_emitter.rs");

    assert!(lib_src.contains("mod stmt_emitter;"));
    assert!(lib_src.contains("mod expr_emitter;"));
    assert!(!lib_src.contains("CodegenLoweringMode"));
    assert!(!lib_src.contains("StructuredPreferred"));
    assert!(!lib_src.contains("should_force_stmt_string_path"));
    assert!(!lib_src.contains("should_force_expr_string_path"));

    let emit_stmt_start = lib_src
        .find("fn emit_stmt(&mut self, stmt: &HirStmt) {")
        .expect("emit_stmt wrapper should exist");
    let emit_expr_start = lib_src
        .find("fn emit_expr(&mut self, expr: &HirExpr) {")
        .expect("emit_expr wrapper should exist");
    let emit_stmt_wrapper = &lib_src[emit_stmt_start..emit_expr_start];
    assert!(!emit_stmt_wrapper.contains("self.emit_stmt_string_backend(stmt);"));
    assert!(emit_stmt_wrapper.contains("structured statement emission missing for production path"));
    assert!(!emit_stmt_wrapper.contains("self.try_emit_stmt_string_"));
    assert!(
        !emit_stmt_wrapper.contains("match stmt"),
        "emit_stmt should stay orchestration-only"
    );

    let body_contains_yield_start = lib_src
        .find("pub fn body_contains_yield(stmts: &[HirStmt]) -> bool {")
        .expect("body_contains_yield should exist");
    let emit_expr_wrapper = &lib_src[emit_expr_start..body_contains_yield_start];
    assert!(!emit_expr_wrapper.contains("self.emit_expr_string_backend(expr);"));
    assert!(
        emit_expr_wrapper.contains("structured expression emission missing for production path")
    );
    assert!(!emit_expr_wrapper.contains("self.try_emit_expr_string_"));
    assert!(
        !emit_expr_wrapper.contains("match expr"),
        "emit_expr should stay orchestration-only"
    );

    let lib_lines = lib_src.lines().count();
    assert!(
        lib_lines <= 1450,
        "lib.rs should stay decomposed (current lines: {lib_lines})"
    );

    let lib_direct_write_calls = lib_src.match_indices("self.write(").count();
    assert!(
        lib_direct_write_calls <= 30,
        "lib.rs should not regain write-heavy emission logic (self.write count: {lib_direct_write_calls})"
    );

    assert!(stmt_src.contains("emit_stmt_string_backend"));
    assert!(expr_src.contains("emit_expr_string_backend"));
    assert!(stmt_src.contains("unreachable in production structured codegen path"));
    assert!(expr_src.contains("unreachable in production structured codegen path"));
}

#[test]
fn test_production_lowering_contract_uses_result_helpers_only() {
    let lib_src = include_str!("lib.rs");
    let module_constants_src = include_str!("module_constants.rs");
    let expr_render_helpers_src = include_str!("expr_render_helpers.rs");

    assert!(lib_src.contains("try_lower_simple_stmt_with_scope_result("));
    assert!(lib_src.contains("try_lower_leaf_expr_result("));
    assert!(module_constants_src.contains("try_lower_simple_module_constant_item_result("));
    assert!(expr_render_helpers_src.contains("try_lower_registry_expr_result("));

    assert!(!lib_src.contains("try_lower_simple_stmt_with_scope("));
    assert!(!lib_src.contains("try_lower_leaf_expr("));
    assert!(!module_constants_src.contains("try_lower_simple_module_constant_item("));
}

#[test]
fn test_union_display_impl_uses_structured_ir() {
    let union_src = include_str!("union_type_helpers.rs");
    assert!(!union_src.contains("RustType::RawCode(\"&mut std::fmt::Formatter<'_>\""));
    assert!(!union_src.contains("RustType::RawCode(\"std::fmt::Result\""));
    assert!(!union_src.contains("RustStmt::RawCode(match_lines)"));
    assert!(!union_src.contains("RustExpr::RawCode(format!(\"\\\"{fmt_spec}\\\"\"))"));
    assert!(union_src.contains("RustType::Ref {"));
    assert!(union_src.contains("RustStmt::Match {"));
    assert!(union_src.contains("RustExpr::Literal(RustLiteral::Str(fmt_spec.to_string()))"));
}

#[test]
fn test_union_enum_definitions_emit_structured_items() {
    let union_src = include_str!("union_type_helpers.rs");
    let lib_src = include_str!("lib.rs");

    assert!(union_src.contains("self.enum_items.push(RustItem::Enum {"));
    assert!(!union_src.contains("enum_defs"));
    assert!(!lib_src.contains("enum_defs"));
}

#[test]
fn test_generate_rust_with_stdlib_assembles_single_rust_file() {
    let lib_src = include_str!("lib.rs");
    let start = lib_src
        .find("pub fn generate_rust_with_stdlib")
        .expect("generate_rust_with_stdlib should exist");
    let end = lib_src
        .find("/// Generate Rust source code for a multi-module project.")
        .expect("generate_rust_multi docs should exist");
    let generate_block = &lib_src[start..end];

    assert!(generate_block.contains("let file_issues = validate_items(&file_items);"));
    assert!(generate_block.contains("let rust_file = RustFile { items: file_items };"));
    assert!(generate_block.contains("Renderer::new().render_file(&rust_file)"));
    assert!(generate_block
        .contains("assert_output_drained(&emitter.output, \"generate_rust_with_stdlib\")"));
    assert!(!generate_block.contains("if !emitter.output.is_empty() {"));
    assert!(!generate_block.contains("result.push_str(&emitter.output)"));
}

#[test]
fn test_generate_rust_multi_assembles_single_rust_file() {
    let lib_src = include_str!("lib.rs");
    let start = lib_src
        .find("pub fn generate_rust_multi")
        .expect("generate_rust_multi should exist");
    let end = lib_src
        .find("/// Generate a complete Rust project (Cargo.toml + main.rs content).")
        .expect("generate_project docs should exist");
    let generate_block = &lib_src[start..end];

    assert!(generate_block.contains("RustItem::UseAlias"));
    assert!(generate_block.contains("let file_issues = validate_items(&file_items);"));
    assert!(generate_block.contains("let rust_file = RustFile { items: file_items };"));
    assert!(generate_block.contains("Renderer::new().render_file(&rust_file)"));
    assert!(
        generate_block.contains("assert_output_drained(&emitter.output, \"generate_rust_multi\")")
    );
    assert!(!generate_block.contains("if !emitter.output.is_empty() {"));
    assert!(!generate_block.contains("module_import_prelude"));
    assert!(!generate_block.contains("result.push_str(&emitter.output)"));
}

#[test]
fn test_module_constants_flow_through_assembled_body_items() {
    let module_constants_src = include_str!("module_constants.rs");
    let entrypoints_src = include_str!("entrypoints.rs");
    let lib_src = include_str!("lib.rs");

    assert!(module_constants_src.contains("self.body_items.push(item);"));
    assert!(module_constants_src
        .contains("structured module constant emission missing for production path"));
    assert!(!module_constants_src.contains("push_syn_items_from_source"));
    assert!(!module_constants_src.contains("RustItem::RawCode"));
    assert!(!module_constants_src.contains("self.output.push_str(&render_items(&[item]))"));

    assert!(entrypoints_src.contains("if !emitter.body_items.is_empty() {"));
    assert!(lib_src.contains("if !emitter.body_items.is_empty() {"));
    assert!(
        entrypoints_src.contains("assert_output_drained(&emitter.output, \"generate_rust_test\")")
    );
    assert!(!entrypoints_src.contains("if !emitter.output.is_empty() {"));
    assert!(!lib_src.contains("if !emitter.output.is_empty() {"));
    assert!(!lib_src.contains("RustItem::RawCode(stdlib_preamble.clone())"));
}

#[test]
fn test_module_body_flows_through_assembled_body_items() {
    let module_body_src = include_str!("module_body.rs");
    let lib_src = include_str!("lib.rs");

    assert!(module_body_src.contains("self.drain_emitted_output_items(output_len"));
    assert!(module_body_src.contains("self.push_syn_items_from_source(&emitted"));
    assert!(module_body_src.contains("self.body_items.push(RustItem::SynItem(rendered));"));
    assert!(!module_body_src.contains("self.output.push('\\n');"));
    assert!(lib_src.contains("if !emitter.body_items.is_empty() {"));
}

#[test]
fn test_generator_init_emission_is_structured_only() {
    let stmt_support_src = include_str!("stmt_support_emitter.rs");
    assert!(stmt_support_src.contains("match self.try_emit_structured_expr(value)"));
    assert!(stmt_support_src.contains("match self.try_emit_structured_stmt(stmt)"));
    assert!(!stmt_support_src.contains("self.try_emit_expr_string_"));
    assert!(!stmt_support_src.contains("self.try_emit_stmt_string_"));
    assert!(!stmt_support_src.contains("self.emit_expr(value);"));
    assert!(!stmt_support_src.contains("self.emit_stmt(stmt);"));
}

#[test]
fn test_round_parenthesizes_cast_receiver() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Expr {
                expr: HirExpr::Call {
                    func: "print".to_string(),
                    args: vec![HirExpr::Call {
                        func: "round".to_string(),
                        args: vec![HirExpr::Call {
                            func: "float".to_string(),
                            args: vec![HirExpr::IntLiteral(3)],
                            ty: Type::Float,
                        }],
                        ty: Type::Int,
                    }],
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
    assert!(
        rust_code.contains("((3 as i64) as f64).round() as i64"),
        "expected round receiver to be parenthesized; got: {rust_code}"
    );
    assert!(
        !rust_code.contains("as f64.round()"),
        "invalid Rust precedence should not be emitted"
    );
}

#[test]
fn test_float_min_max_parenthesize_cast_receivers() {
    let float_one = HirExpr::Call {
        func: "float".to_string(),
        args: vec![HirExpr::IntLiteral(1)],
        ty: Type::Float,
    };
    let float_two = HirExpr::Call {
        func: "float".to_string(),
        args: vec![HirExpr::IntLiteral(2)],
        ty: Type::Float,
    };

    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![
                HirStmt::Expr {
                    expr: HirExpr::Call {
                        func: "print".to_string(),
                        args: vec![HirExpr::Call {
                            func: "min".to_string(),
                            args: vec![float_one.clone(), float_two.clone()],
                            ty: Type::Float,
                        }],
                        ty: Type::None,
                    },
                },
                HirStmt::Expr {
                    expr: HirExpr::Call {
                        func: "print".to_string(),
                        args: vec![HirExpr::Call {
                            func: "max".to_string(),
                            args: vec![float_one, float_two],
                            ty: Type::Float,
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
    assert!(
        rust_code.contains("((1 as i64) as f64).min((2 as i64) as f64)"),
        "expected min receiver to be parenthesized; got: {rust_code}"
    );
    assert!(
        rust_code.contains("((1 as i64) as f64).max((2 as i64) as f64)"),
        "expected max receiver to be parenthesized; got: {rust_code}"
    );
    assert!(
        !rust_code.contains("as f64.min(") && !rust_code.contains("as f64.max("),
        "invalid Rust precedence should not be emitted"
    );
}
