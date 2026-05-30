use super::*;
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
            is_async: false,
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
fn test_mut_on_local_nested_function_mutborrow_call_argument() {
    let rust_code = generate_rust_from_source(
        "def main():\n\
    vals: list[str] = [\"x\"]\n\
    def dfs(vals: list[str]) -> None:\n\
        vals.pop(0)\n\
    dfs(vals)\n",
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
        "class Codec:\n\
    pass\n\
\n\
def main():\n\
    codec = Codec()\n",
    );

    assert!(rust_code.contains("impl Codec {"));
    assert!(rust_code.contains("fn new() -> Self {"));
    assert!(rust_code.contains("let codec: Codec = Codec::new();"));
}

#[test]
fn test_non_option_local_widened_to_option_when_reassigned_none() {
    let rust_code = generate_rust_from_source(
        "class TreeNode:\n\
    val: int\n\
\n\
    def __init__(self, val: int = 0):\n\
        self.val = val\n\
\n\
def main():\n\
    root = TreeNode(1)\n\
    root = None\n",
    );

    assert!(rust_code.contains("let mut root: Option<TreeNode> = Some("));
    assert!(rust_code.contains("root = None;"));
}

#[test]
fn test_guarded_non_option_compare_does_not_emit_some_wrapping() {
    let rust_code = generate_rust_from_source(
        "def parseIntToken(token: str) -> int:\n\
    first = token[0]\n\
    if first is not None and first == \"-\":\n\
        return -1\n\
    return 0\n",
    );

    assert!(!rust_code.contains("first == Some("));
    assert!(rust_code.contains("first == \"-\".to_string()"));
}

#[test]
fn test_owned_recursive_option_field_moves_without_tail_clone() {
    let rust_code = generate_rust_from_source(
        r#"class ListNode:
    val: int
    next: ListNode | None

    def __init__(self, val: int = 0, next: ListNode | None = None):
        self.val = val
        self.next = next

def reverseInto(own mut cur: ListNode | None, own prev: ListNode | None) -> ListNode | None:
    if cur is None:
        return prev
    next_node: ListNode | None = cur.next
    cur.next = prev
    return reverseInto(next_node, cur)
"#,
    );

    assert!(
        rust_code.contains("(cur.next).map(|__sifr_boxed_recursive_value|"),
        "owned recursive field read should move the boxed child instead of cloning it:\n{rust_code}"
    );
    assert!(
        !rust_code.contains("(cur.next).as_deref().cloned()"),
        "owned recursive field read should not clone the remaining list tail:\n{rust_code}"
    );
    assert!(
        !rust_code.contains("Some((cur).clone())"),
        "owned optional parameter wrapping should move cur instead of cloning it:\n{rust_code}"
    );
}

#[test]
fn test_borrowed_recursive_option_field_still_clones() {
    let rust_code = generate_rust_from_source(
        r#"class ListNode:
    val: int
    next: ListNode | None

    def __init__(self, val: int = 0, next: ListNode | None = None):
        self.val = val
        self.next = next

def nodeNext(node: ListNode | None) -> ListNode | None:
    if node is None:
        return None
    return node.next
"#,
    );

    assert!(
        rust_code.contains("(node.next).as_deref().cloned()"),
        "borrowed recursive field read must keep cloning semantics:\n{rust_code}"
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
            is_async: false,
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
    assert_eq!(int_code, "7 as i64");

    let bool_op = HirExpr::BoolOp {
        op: "and".to_string(),
        values: vec![HirExpr::BoolLiteral(true), HirExpr::BoolLiteral(false)],
        ty: Type::Bool,
    };
    let bool_code = render_strict_lowered_expr(&mut emitter, &bool_op);
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

    let code = render_strict_lowered_expr(&mut emitter, &expr);
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

    let code = render_strict_lowered_expr(&mut emitter, &expr);
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
            is_async: false,
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
            is_async: false,
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
            is_async: false,
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
            is_async: false,
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
    assert!(utils_rs.contains("pub const ANSWER: i64 = 7 as i64;"));
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
    assert!(result.required_crates.contains("num-bigint"));
    assert!(result.required_crates.contains("num-traits"));
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
                        "name = \"phase-five\"\nvalue = 5".to_string(),
                    )],
                    ty: Type::Result(Box::new(Type::Str), Box::new(Type::Any)),
                },
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
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

    let result = generate_rust_multi_with_metadata(
        &[("main", &main_module), ("helper", &helper_module)],
        &StdlibCode::default(),
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
        helper_rs.contains("impl std::fmt::Display for TOMLDecodeError"),
        "stdlib trait impls should be preserved in publicized helper modules"
    );
    assert!(
        !helper_rs.contains("pub fn fmt("),
        "trait impl methods must not receive pub visibility during support-module publicization"
    );
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
            is_async: false,
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
