use super::*;

#[test]
fn test_mut_on_mutating_method_call() {
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
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let rust_code = generate_rust(&module);
    assert!(rust_code.contains("let mut items"));
}

#[test]
fn test_mut_on_local_nested_function_mutborrow_call_argument() {
    let rust_code = generate_rust_from_source(
        r#"def main():
    vals: list[str] = ["x"]
    def dfs(vals: list[str]) -> None:
        vals.pop(0)
    dfs(vals)
"#,
    );

    assert!(
        rust_code.contains("let mut vals: Vec<String>"),
        "local nested mut-borrow call should mark argument binding mutable"
    );
    assert!(rust_code.contains("dfs(&mut vals);"));
}

#[test]
fn test_fieldless_class_gets_default_constructor() {
    let rust_code = generate_rust_from_source(
        r#"class Codec:
    pass

def main():
    codec = Codec()
"#,
    );

    assert!(rust_code.contains("impl Codec {"));
    assert!(rust_code.contains("fn new() -> Self {"));
    assert!(rust_code.contains("let codec: Codec = Codec::new();"));
}

#[test]
fn callable_field_default_constructor_boxes_the_stored_trait_object() {
    let rust_code = generate_rust_from_source(
        r#"class CallbackHolder:
    callback: Callable[[int], int]

def identity(value: int) -> int:
    return value

def main():
    holder = CallbackHolder(identity)
"#,
    );

    assert!(rust_code.contains("callback: Box<dyn Fn(i64) -> i64>"));
    assert!(rust_code.contains("fn new(callback: impl Fn(i64) -> i64 + 'static)"));
    assert!(rust_code.contains("Self { callback: Box::new(callback) }"));
}

#[test]
fn process_child_resource_derives_are_module_scoped() {
    let module = HirModule {
        functions: vec![],
        classes: vec![HirClass {
            name: "Child".to_string(),
            fields: vec![
                ("_handle".to_string(), Type::Int),
                ("_waited".to_string(), Type::Bool),
            ],
            methods: vec![],
            is_error_type: false,
            is_hashable: true,
            operator_impls: vec![],
            type_params: vec![],
            newtype_inner: None,
            implements_protocols: vec![],
            enum_variants: vec![],
            kind: HirClassKind::Regular,
            parent_class: None,
            rust_interop: Vec::new(),
        }],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let user_rust = generate_rust(&module);
    assert!(user_rust.contains("#[derive(Debug, Clone, PartialEq, Eq, Hash)]\nstruct Child"));

    let process_rust =
        generate_rust_with_stdlib_for_module(&module, &StdlibCode::default(), Some("sifr.process"))
            .rust_source;
    assert!(process_rust.contains("#[derive(Debug)]\nstruct Child"));
    assert!(process_rust.contains("impl Drop for Child"));
    assert!(process_rust.contains("::sifr_stdlib::process::process_child_close"));
    assert!(!process_rust.contains("__SIFR_PROCESS_CHILDREN"));
    assert!(!process_rust.contains("#[derive(Debug, Clone"));
}

#[test]
fn test_class_to_string_method_does_not_emit_generated_allow() {
    let module = HirModule {
        functions: vec![],
        classes: vec![HirClass {
            name: "LocaleId".to_string(),
            fields: vec![("value".to_string(), Type::Str)],
            methods: vec![HirFunction {
                name: "to_string".to_string(),
                params: vec![],
                return_type: Type::Str,
                body: vec![HirStmt::Return {
                    value: Some(HirExpr::StringLiteral("en-US".to_string())),
                }],
                is_async: false,
                method_kind: MethodKind::Regular,
                decorators: vec![],
                rust_interop: Vec::new(),
                python_interop: Vec::new(),
                compiler_intrinsic: None,
                type_params: vec![],
            }],
            is_hashable: false,
            is_error_type: false,
            kind: HirClassKind::Regular,
            operator_impls: vec![],
            newtype_inner: None,
            implements_protocols: vec![],
            parent_class: None,
            type_params: vec![],
            enum_variants: vec![],
            rust_interop: Vec::new(),
        }],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let rust_code = generate_rust(&module);
    assert!(!rust_code.contains("#[allow(clippy::inherent_to_string_shadow_display)]"));
    assert!(rust_code.contains("impl LocaleId"));
    assert!(rust_code.contains("impl ::std::fmt::Display for LocaleId"));
}

#[test]
fn test_non_option_local_widened_to_option_when_reassigned_none() {
    let rust_code = generate_rust_from_source(
        r#"class Payload:
    val: int

    def __init__(self, val: int = 0):
        self.val = val

def main():
    item = Payload(1)
    item = None
"#,
    );

    assert!(rust_code.contains("let mut item: Option<Payload> = Some("));
    assert!(rust_code.contains("item = None;"));
}

#[test]
fn test_guarded_non_option_compare_does_not_emit_some_wrapping() {
    let rust_code = generate_rust_from_source(
        r#"def parseIntToken(token: str) -> int:
    if len(token) > 0:
        first = token[0]
        if first == "-":
            return -1
    return 0
"#,
    );

    assert!(!rust_code.contains("first == Some("));
    assert!(!rust_code.contains("first == \"-\".to_string()"));
    assert!(rust_code.contains("first == \"-\""));
}

#[test]
fn test_empty_print() {
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
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
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
fn test_empty_string_print_emits_empty_println_macro() {
    let rust_code = generate_rust_from_source("def main():\n    print(\"\")\n");

    assert!(
        rust_code.contains("println!()"),
        "should emit println!() for print(\"\")"
    );
    assert!(
        !rust_code.contains("println!(\"\")"),
        "should not emit println with an empty string literal"
    );
}

fn render_strict_lowered_expr(emitter: &mut RustEmitter, expr: &HirExpr) -> String {
    let Some(lowered_expr) = emitter.try_lower_registry_expr_strict(expr) else {
        panic!("strict IR rendering path missing for expression: {expr:?}");
    };
    crate::render_expr(&lowered_expr)
}

#[test]
fn test_expr_to_string_leaf_rendering() {
    let mut emitter = RustEmitter::new();
    let int_code = render_strict_lowered_expr(&mut emitter, &HirExpr::IntLiteral(7));
    assert_eq!(int_code, "7_i64");

    let bool_op = HirExpr::BoolOp {
        op: "and".to_string(),
        values: vec![HirExpr::BoolLiteral(true), HirExpr::BoolLiteral(false)],
        ty: Type::Bool,
    };
    let bool_code = render_strict_lowered_expr(&mut emitter, &bool_op);
    assert_eq!(bool_code, "true && false");
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

    let code = render_strict_lowered_expr(&mut emitter, &expr);
    assert!(code.contains("LIMIT +"));
}

#[test]
fn test_render_expr_lowering_uses_module_constant_for_stdlib_named_constant() {
    let mut emitter = RustEmitter::new();
    emitter.intrinsic_functions.insert("pi".to_string());
    emitter
        .module_constants
        .insert("pi".to_string(), (Type::Float, "PI".to_string()));
    let expr = HirExpr::BinOp {
        left: Box::new(HirExpr::Name {
            name: "pi".to_string(),
            ty: Type::Float,
        }),
        op: "+".to_string(),
        right: Box::new(HirExpr::FloatLiteral(1.0)),
        ty: Type::Float,
    };

    let code = render_strict_lowered_expr(&mut emitter, &expr);
    assert!(code.contains("PI +"));
    assert!(!code.contains("std::f64::consts::PI"));
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

    let code = render_strict_lowered_expr(&mut emitter, &expr);
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
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![("limit".to_string(), Type::Int, HirExpr::IntLiteral(7))],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let result = generate_rust_with_metadata(&module);
    assert!(result.rust_source.contains("const LIMIT: i64 = 7_i64;"));
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
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
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

    let mut stdlib_code = StdlibCode::default();
    stdlib_code.module_constants.insert(
        "sifr.math".to_string(),
        std::collections::HashMap::from([("pi".to_string(), (Type::Float, "PI".to_string()))]),
    );

    let result = generate_rust_with_stdlib_for_module(&module, &stdlib_code, None);
    assert!(result.rust_source.contains("let x: f64 = PI;"));
    assert!(!result.rust_source.contains("let x: f64 = pi;"));
    assert!(!result.rust_source.contains("std::f64::consts::PI"));
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
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
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
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
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
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
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
            rust_interop: Vec::new(),
        }],
        imports: vec![],
        constants: vec![("ANSWER".to_string(), Type::Int, HirExpr::IntLiteral(7))],
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
    assert!(utils_rs.contains("pub const ANSWER: i64 = 7_i64;"));
    assert!(utils_rs.contains("pub value: i64"));
    assert!(utils_rs.contains("pub fn new(value: i64) -> Self"));
}

#[test]
fn test_generate_rust_multi_publicizes_non_main_reexports() {
    let root_module = HirModule {
        functions: vec![],
        classes: vec![],
        imports: vec![HirImport {
            module: "root.leaf".to_string(),
            names: vec!["leaf_value".to_string()],
            aliases: vec![],
        }],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };
    let leaf_module = HirModule {
        functions: vec![HirFunction {
            name: "leaf_value".to_string(),
            params: vec![],
            return_type: Type::Int,
            body: vec![HirStmt::Return {
                value: Some(HirExpr::IntLiteral(7)),
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let files = generate_rust_multi(&[("root", &root_module), ("root.leaf", &leaf_module)]);
    let root_rs = files.get("root").expect("root module should be generated");

    assert!(root_rs.contains("pub use crate::root::leaf::leaf_value;"));
}

#[test]
fn test_generate_rust_multi_skips_stdlib_use_paths_in_non_main_modules() {
    let main_module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Expr {
                expr: HirExpr::Call {
                    func: "helper".to_string(),
                    args: vec![],
                    ty: Type::Float,
                },
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![HirImport {
            module: "utils".to_string(),
            names: vec!["helper".to_string()],
            aliases: vec![],
        }],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let utils_module = HirModule {
        functions: vec![HirFunction {
            name: "helper".to_string(),
            params: vec![],
            return_type: Type::Float,
            body: vec![HirStmt::Return {
                value: Some(HirExpr::Call {
                    func: "sqrt".to_string(),
                    args: vec![HirExpr::FloatLiteral(9.0)],
                    ty: Type::Float,
                }),
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
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

    let files = generate_rust_multi(&[("main", &main_module), ("utils", &utils_module)]);
    let utils_rs = files
        .get("utils")
        .expect("utils module should be generated");
    assert!(
        !utils_rs.contains("use crate::sifr"),
        "stdlib imports must not render crate::sifr.* use paths in multi-module output"
    );
}

#[test]
fn test_generate_rust_multi_with_metadata_aggregates_reachable_dependency_closure() {
    let main_module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Expr {
                expr: HirExpr::Call {
                    func: "helper".to_string(),
                    args: vec![],
                    ty: Type::BigInt,
                },
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![HirImport {
            module: "helper".to_string(),
            names: vec!["helper".to_string()],
            aliases: vec![],
        }],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let helper_module = HirModule {
        functions: vec![HirFunction {
            name: "helper".to_string(),
            params: vec![],
            return_type: Type::BigInt,
            body: vec![HirStmt::Return {
                value: Some(HirExpr::Call {
                    func: "bigint".to_string(),
                    args: vec![HirExpr::IntLiteral(1)],
                    ty: Type::BigInt,
                }),
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![HirImport {
            module: "sifr.statistics".to_string(),
            names: vec!["mean".to_string()],
            aliases: vec![],
        }],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let mut stdlib_code = StdlibCode::default();
    stdlib_code.transitive_deps.insert(
        "sifr.statistics".to_string(),
        HashSet::from(["sifr.math".to_string()]),
    );

    let result = generate_rust_multi_with_metadata(
        &[("main", &main_module), ("helper", &helper_module)],
        &stdlib_code,
    );

    assert!(result.rust_files.contains_key("main"));
    assert!(result.rust_files.contains_key("helper"));
    assert!(result.used_stdlib_modules.contains("sifr.statistics"));
    assert!(result.used_stdlib_modules.contains("sifr.math"));
    assert!(result
        .required_features
        .contains(&sifr_stdlib_manifest::StdlibFeature::NumBigint));
    assert!(result
        .required_features
        .contains(&sifr_stdlib_manifest::StdlibFeature::NumTraits));
}

#[test]
fn test_generate_rust_multi_with_metadata_preserves_trait_impl_visibility() {
    let main_module = HirModule {
        functions: vec![HirFunction {
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
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![HirImport {
            module: "helper".to_string(),
            names: vec!["helper".to_string()],
            aliases: vec![],
        }],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let helper_module = HirModule {
        functions: vec![HirFunction {
            name: "helper".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Expr {
                expr: HirExpr::Call {
                    func: "loads".to_string(),
                    args: vec![HirExpr::StringLiteral(
                        "name = \"fixture-five\"\nvalue = 5".to_string(),
                    )],
                    ty: Type::Result(Box::new(Type::Str), Box::new(Type::Any)),
                },
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![HirImport {
            module: "sifr.tomllib".to_string(),
            names: vec!["loads".to_string()],
            aliases: vec![],
        }],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let stdlib_code = trait_impl_fixture_stdlib_code();

    let result = generate_rust_multi_with_metadata(
        &[("main", &main_module), ("helper", &helper_module)],
        &stdlib_code,
    );

    let helper_rs = result
        .rust_files
        .get("helper")
        .expect("helper module should be generated");
    assert!(
        helper_rs.contains("pub fn helper()"),
        "support-module functions should be exported"
    );
    assert!(
        helper_rs.contains("impl ::std::fmt::Display for TOMLDecodeError"),
        "stdlib trait impls should be preserved in publicized helper modules"
    );
    assert!(
        !helper_rs.contains("pub fn fmt("),
        "trait impl methods must not receive pub visibility during support-module publicization"
    );
}
