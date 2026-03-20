use crate::{
    generate_rust, generate_rust_multi, generate_rust_multi_with_metadata, generate_rust_test,
    generate_rust_with_metadata, RustEmitter, RustExpr, RustStmt, RustType, StdlibCode,
};
use sifr_hir::{
    lower_module, HirClass, HirClassKind, HirExceptHandler, HirExpr, HirFStringPart, HirFunction,
    HirImport, HirMatchArm, HirModule, HirParam, HirPattern, HirStmt, MethodKind,
};
use sifr_python_parser::parse_module;
use sifr_type_system::{ParamConvention, Type};
use std::collections::HashSet;

fn generate_rust_from_source(source: &str) -> String {
    let parsed = parse_module(source).expect("parse failed");
    let lowering = lower_module(parsed.suite()).expect("lowering failed");
    generate_rust(&lowering.module)
}

#[test]
fn test_class_method_mutable_self_propagates_through_delegation() {
    let rust_code = generate_rust_from_source(
        "class ConfigParser:\n    text: str\n\n    def __init__(self):\n        self.text = \"\"\n\n    def read_string(self, text: str) -> None:\n        self.text = text\n\n    def read(self, text: str) -> None:\n        self.read_string(text)\n",
    );

    assert!(rust_code.contains("fn read_string(&mut self"));
    assert!(rust_code.contains("fn read(&mut self"));
    assert!(!rust_code.contains("fn read(&self"));
}

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
                    convention: ParamConvention::own(),
                },
                HirParam {
                    name: "b".to_string(),
                    ty: Type::Int,
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::own(),
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
fn test_generate_rust_recursive_tree_traversal_uses_option_let_else_and_cloned_box_reads() {
    let rust_code = generate_rust_from_source(
        "class TreeNode:\n    val: int\n    left: TreeNode | None\n    right: TreeNode | None\n\n    def __init__(self, val: int, left: TreeNode | None, right: TreeNode | None):\n        self.val = val\n        self.left = left\n        self.right = right\n\ndef tree_sum(node: TreeNode | None) -> int:\n    if not node:\n        return 0\n    left: TreeNode | None = node.left\n    right: TreeNode | None = node.right\n    return node.val + tree_sum(left) + tree_sum(right)\n\ndef same_shape_and_sum(p: TreeNode | None, q: TreeNode | None) -> int:\n    if not p and not q:\n        return 0\n    if not p or not q:\n        return -1\n    return p.val + q.val + same_shape_and_sum(p.left, q.left) + same_shape_and_sum(p.right, q.right)\n",
    );

    assert!(rust_code.contains("let Some(node) = node.as_ref() else {"));
    assert!(rust_code.contains("(node.left).as_deref().cloned()"));
    assert!(rust_code.contains("let (Some(p), Some(q)) = (p.as_ref(), q.as_ref()) else {"));
    assert!(rust_code.contains("(p.left).as_deref().cloned()"));
}

#[test]
fn test_generate_rust_mutually_recursive_classes_box_same_scc_fields() {
    let rust_code = generate_rust_from_source(
        "class Expr:\n    value: int\n    term: Term | None\n\n    def __init__(self, value: int, term: Term | None):\n        self.value = value\n        self.term = term\n\nclass Term:\n    factor: int\n    expr: Expr | None\n\n    def __init__(self, factor: int, expr: Expr | None):\n        self.factor = factor\n        self.expr = expr\n",
    );

    assert!(rust_code.contains("term: Option<Box<Term>>"));
    assert!(rust_code.contains("expr: Option<Box<Expr>>"));
    assert!(!rust_code.contains("term: Option<Term>"));
    assert!(!rust_code.contains("expr: Option<Expr>"));
}

#[test]
fn test_generate_rust_recursive_generic_node_preserves_instantiated_type_arguments() {
    let rust_code = generate_rust_from_source(
        "class Node[T]:\n    value: T\n    next: Node[T] | None\n\n    def __init__(self, value: T, next: Node[T] | None):\n        self.value = value\n        self.next = next\n\ndef total(node: Node[int] | None) -> int:\n    if not node:\n        return 0\n    rest: Node[int] | None = node.next\n    return node.value + total(rest)\n",
    );

    assert!(rust_code.contains("next: Option<Box<Node<T>>>"));
    assert!(rust_code.contains("fn new(value: T, next: Option<Box<Node<T>>>) -> Self"));
    assert!(rust_code.contains("fn total(node: &Option<Node<i64>>) -> i64"));
    assert!(rust_code.contains("let rest: Option<Node<i64>> = (node.next).as_deref().cloned();"));
}

#[test]
fn test_generate_rust_own_mut_param_emits_mut_binding_without_shadow() {
    let rust_code = generate_rust_from_source(
        "def replace_elements(own mut arr: list[int]) -> list[int]:\n    arr[0] = 8\n    return arr\n\ndef touch(mut arr: list[int]) -> int:\n    arr[0] = 7\n    return len(arr)\n",
    );

    assert!(rust_code.contains("fn replace_elements(mut arr: Vec<i64>) -> Vec<i64>"));
    assert!(
        !rust_code.contains("let mut arr = arr;"),
        "owned mutable params should lower directly to mutable Rust params"
    );
    assert!(rust_code.contains("fn touch(arr: &mut Vec<i64>) -> i64"));
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
fn test_structured_codegen_lowers_comprehension_local_initializers() {
    let items_name = HirExpr::Name {
        name: "items".to_string(),
        ty: Type::List(Box::new(Type::Int)),
    };
    let comp_item = HirExpr::Name {
        name: "x".to_string(),
        ty: Type::Int,
    };
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
                        elements: vec![HirExpr::IntLiteral(1), HirExpr::IntLiteral(2)],
                        ty: Type::List(Box::new(Type::Int)),
                    },
                    is_mutable: false,
                },
                HirStmt::Let {
                    name: "values".to_string(),
                    ty: Type::List(Box::new(Type::Int)),
                    value: HirExpr::ListComp {
                        expr: Box::new(comp_item.clone()),
                        generators: vec![("x".to_string(), items_name.clone(), None)],
                        ty: Type::List(Box::new(Type::Int)),
                    },
                    is_mutable: false,
                },
                HirStmt::Let {
                    name: "lookup".to_string(),
                    ty: Type::Dict(Box::new(Type::Int), Box::new(Type::Int)),
                    value: HirExpr::DictComp {
                        key_expr: Box::new(comp_item.clone()),
                        val_expr: Box::new(comp_item.clone()),
                        generators: vec![("x".to_string(), items_name.clone(), None)],
                        ty: Type::Dict(Box::new(Type::Int), Box::new(Type::Int)),
                    },
                    is_mutable: false,
                },
                HirStmt::Let {
                    name: "unique".to_string(),
                    ty: Type::Set(Box::new(Type::Int)),
                    value: HirExpr::SetComp {
                        expr: Box::new(comp_item),
                        generators: vec![("x".to_string(), items_name, None)],
                        ty: Type::Set(Box::new(Type::Int)),
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

    assert!(rust_code.contains("let values: Vec<i64> = {"));
    assert!(rust_code.contains("let lookup: HashMap<i64, i64> = {"));
    assert!(rust_code.contains("let unique: HashSet<i64> = {"));
    assert!(rust_code.contains("__sifr_list_comp"));
    assert!(rust_code.contains("__sifr_dict_comp"));
    assert!(rust_code.contains("__sifr_set_comp"));
}

#[test]
fn test_reverse_range_for_loop_uses_rev_iterator_for_unary_negative_step() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::For {
                target: "i".to_string(),
                target_ty: Type::Int,
                iter: HirExpr::RangeLiteral {
                    start: Box::new(HirExpr::IntLiteral(4)),
                    end: Box::new(HirExpr::UnaryOp {
                        op: "-".to_string(),
                        operand: Box::new(HirExpr::IntLiteral(1)),
                        ty: Type::Int,
                    }),
                    step: Some(Box::new(HirExpr::UnaryOp {
                        op: "-".to_string(),
                        operand: Box::new(HirExpr::IntLiteral(1)),
                        ty: Type::Int,
                    })),
                    ty: Type::Range,
                },
                body: vec![HirStmt::Expr {
                    expr: HirExpr::Call {
                        func: "print".to_string(),
                        args: vec![HirExpr::Name {
                            name: "i".to_string(),
                            ty: Type::Int,
                        }],
                        ty: Type::None,
                    },
                }],
                else_body: None,
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

    assert!(rust_code.contains(".rev()"));
    assert!(!rust_code.contains("step_by(-(1 as i64) as usize)"));
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
fn test_generate_rust_while_else_with_borrowed_condition_uses_broke_marker() {
    let list_ty = Type::List(Box::new(Type::Int));
    let module = HirModule {
        functions: vec![
            HirFunction {
                name: "iterate".to_string(),
                params: vec![HirParam {
                    name: "xs".to_string(),
                    ty: list_ty.clone(),
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::borrow(),
                }],
                return_type: Type::None,
                body: vec![HirStmt::While {
                    condition: HirExpr::Name {
                        name: "xs".to_string(),
                        ty: list_ty.clone(),
                    },
                    body: vec![HirStmt::Break],
                    else_body: Some(vec![HirStmt::Expr {
                        expr: HirExpr::Call {
                            func: "print".to_string(),
                            args: vec![HirExpr::StringLiteral("empty".to_string())],
                            ty: Type::None,
                        },
                    }]),
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
                        func: "iterate".to_string(),
                        args: vec![HirExpr::ListLiteral {
                            elements: vec![],
                            ty: list_ty.clone(),
                        }],
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

    let rust_code = generate_rust(&module);
    assert!(rust_code.contains("fn iterate(xs: &Vec<i64>)"));
    assert!(rust_code.contains("let mut _broke: bool = false;"));
    assert!(rust_code.contains("_broke = true;"));
    assert!(rust_code.contains("if !(_broke) {"));
}

#[test]
fn test_generate_rust_generator_try_except_materializes_without_shape_panic() {
    let module = HirModule {
        functions: vec![
            HirFunction {
                name: "gen".to_string(),
                params: vec![],
                return_type: Type::Iterator(Box::new(Type::Int)),
                body: vec![HirStmt::TryExcept {
                    body: vec![HirStmt::Yield {
                        value: HirExpr::IntLiteral(1),
                    }],
                    handlers: vec![HirExceptHandler {
                        error_type: Some("Error".to_string()),
                        error_resolved_type: None,
                        name: Some("e".to_string()),
                        body: vec![HirStmt::Yield {
                            value: HirExpr::IntLiteral(2),
                        }],
                    }],
                    body_error_types: vec!["Error".to_string()],
                }],
                method_kind: MethodKind::Regular,
                decorators: vec![],
                type_params: vec![],
            },
            HirFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![HirStmt::For {
                    target: "v".to_string(),
                    target_ty: Type::Int,
                    iter: HirExpr::Call {
                        func: "gen".to_string(),
                        args: vec![],
                        ty: Type::Iterator(Box::new(Type::Int)),
                    },
                    body: vec![HirStmt::Expr {
                        expr: HirExpr::Call {
                            func: "print".to_string(),
                            args: vec![HirExpr::Name {
                                name: "v".to_string(),
                                ty: Type::Int,
                            }],
                            ty: Type::None,
                        },
                    }],
                    else_body: None,
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

    let rust_code = generate_rust(&module);
    assert!(rust_code.contains("if !__sifr_generator_initialized {"));
    assert!(rust_code.contains("_yields.push(1 as i64);"));
    assert!(rust_code.contains("_yields.push(2 as i64);"));
    assert!(rust_code.contains("__sifr_generator_iter.next()"));
}

#[test]
fn test_generate_rust_generator_conditional_yield_preserves_else_branch() {
    let rust_code = generate_rust_from_source(
        "def gen() -> Iterator[int]:\n    i: int = 0\n    while i < 4:\n        if i < 3:\n            yield i\n            i = i + 1\n        else:\n            i = i + 1\n\ndef main():\n    g: Iterator[int] = gen()\n    print(next(g))\n",
    );

    let cond_idx = rust_code
        .find("if i < (3 as i64) {")
        .expect("generator conditional branch should be emitted");
    let cond_region = &rust_code[cond_idx..];

    assert!(cond_region.contains("_yields.push(i);"));
    assert!(
        cond_region.contains("} else {"),
        "generator else branch should be preserved"
    );
    assert!(
        cond_region.contains("i = i + (1 as i64);"),
        "generator else branch body should be preserved"
    );
}

#[test]
fn test_generate_rust_generator_expression_without_filter_lowers_to_map_chain() {
    let rust_code = generate_rust_from_source(
        "def main():\n    xs: list[int] = [1, 2, 3]\n    squares: Iterator[int] = (x * x for x in xs)\n    print(list(squares))\n",
    );

    assert!(rust_code.contains("let mut squares: Box<dyn Iterator<Item = i64>>"));
    assert!(rust_code.contains("into_iter().map(|x| x * x)"));
}

#[test]
fn test_generate_rust_filter_over_list_lowers_to_lazy_boxed_iterator() {
    let rust_code = generate_rust_from_source(
        "def main():\n    nums: list[int] = [1, 2, 3, 4]\n    evens: Iterator[int] = filter(lambda x: x % 2 == 0, nums)\n    print(list(evens))\n",
    );

    assert!(rust_code.contains("let mut evens: Box<dyn Iterator<Item = i64>>"));
    assert!(rust_code.contains("Box::new("));
    assert!(rust_code.contains(".into_iter().filter("));
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
                    convention: ParamConvention::own(),
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
fn test_generate_rust_test_emits_local_module_import_uses() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "test_import".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Expr {
                expr: HirExpr::Call {
                    func: "helper".to_string(),
                    args: vec![],
                    ty: Type::Int,
                },
            }],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![HirImport {
            module: "support".to_string(),
            names: vec!["helper".to_string()],
            aliases: vec![],
        }],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_test(&module);
    assert!(
        generated
            .rust_source
            .contains("use crate::support::helper;"),
        "test codegen should emit local module uses for imported names"
    );
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
                        convention: ParamConvention::own(),
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
fn test_stmt_path_handles_recursive_nested_function_with_structured_captures() {
    let generated = generate_rust_from_source(
        r#"
def main():
    values: list[int] = [1, 2]
    subset: list[int] = []
    res: list[list[int]] = []

    def dfs(i: int):
        if i >= values.len():
            res.append(subset.copy())
            return
        subset.append(i)
        dfs(i + 1)
        subset.pop()
        dfs(i + 1)

    dfs(0)
"#,
    );

    assert!(generated.contains(
        "fn dfs(i: i64, values: &Vec<i64>, subset: &mut Vec<i64>, res: &mut Vec<Vec<i64>>)"
    ));
    assert!(generated.contains("dfs(0 as i64, &values, &mut subset, &mut res);"));
    assert!(generated.contains("dfs((i + 1 as i64), values, subset, res);"));
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
fn test_structured_stmt_path_wraps_non_optional_string_index_into_option_local() {
    let stmt = HirStmt::Let {
        name: "part".to_string(),
        ty: Type::Union(vec![Type::Str, Type::None]),
        value: HirExpr::Index {
            object: Box::new(HirExpr::Name {
                name: "text".to_string(),
                ty: Type::Str,
            }),
            index: Box::new(HirExpr::Name {
                name: "j".to_string(),
                ty: Type::Int,
            }),
            ty: Type::Str,
        },
        is_mutable: false,
    };
    let mut emitter = RustEmitter::new();

    let captured = emitter.capture_structured_stmts(|inner| inner.emit_stmt(&stmt));

    assert!(matches!(
        captured.first(),
        Some(RustStmt::Let {
            name,
            ty: Some(RustType::Option(inner)),
            value:
                RustExpr::FnCall {
                    func,
                    args,
                },
            ..
        }) if name == "part"
            && matches!(inner.as_ref(), RustType::String_)
            && matches!(func.as_ref(), RustExpr::Path(path) if path == &vec!["Some".to_string()])
            && matches!(args.as_slice(), [RustExpr::Block { .. }])
    ));
}

#[test]
fn test_structured_stmt_path_handles_non_optional_string_index_return_expr() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "char_at".to_string(),
            params: vec![
                HirParam {
                    name: "text".to_string(),
                    ty: Type::Str,
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::borrow(),
                },
                HirParam {
                    name: "j".to_string(),
                    ty: Type::Int,
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::own(),
                },
            ],
            return_type: Type::Str,
            body: vec![HirStmt::Return {
                value: Some(HirExpr::Index {
                    object: Box::new(HirExpr::Name {
                        name: "text".to_string(),
                        ty: Type::Str,
                    }),
                    index: Box::new(HirExpr::Name {
                        name: "j".to_string(),
                        ty: Type::Int,
                    }),
                    ty: Type::Str,
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

    let generated = generate_rust_with_metadata(&module);
    assert!(generated.rust_source.contains("return {"));
    assert!(generated
        .rust_source
        .contains("let Some(__indexed_char) = text.chars().nth(j as usize) else {"));
    assert!(generated.rust_source.contains("__indexed_char.to_string()"));
    assert!(
        generated.lowering_stats.stmt_structured >= 1,
        "non-optional string index return should stay on the structured stmt path"
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

    let lowered = emitter
        .try_lower_registry_expr_strict(&expr)
        .expect("name expression should lower through structured registry path");

    assert_eq!(crate::render_expr(&lowered), "clock");
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

    let lowered = emitter
        .try_lower_registry_expr_strict(&expr)
        .expect("borrowed compare should lower through structured registry path");
    let rendered = crate::render_expr(&lowered);

    assert!(rendered.contains("lhs"));
    assert!(rendered.contains("=="));
}

#[test]
fn test_lib_decomposition_guards_keep_stmt_expr_logic_out_of_lib_rs() {
    let lib_src = include_str!("lib.rs");

    assert!(!lib_src.contains("mod stmt_emitter;"));
    assert!(!lib_src.contains("mod expr_emitter;"));
    assert!(!lib_src.contains("CodegenLoweringMode"));
    assert!(!lib_src.contains("StructuredPreferred"));
    assert!(!lib_src.contains("should_force_stmt_string_path"));
    assert!(!lib_src.contains("should_force_expr_string_path"));
    assert!(!lib_src.contains("fn emit_expr(&mut self, expr: &HirExpr) {"));
    assert!(!lib_src.contains("fn try_lower_structured_expr("));

    let emit_stmt_start = lib_src
        .find("fn emit_stmt(&mut self, stmt: &HirStmt) {")
        .expect("emit_stmt wrapper should exist");
    let body_contains_yield_start = lib_src
        .find("pub fn body_contains_yield(stmts: &[HirStmt]) -> bool {")
        .expect("body_contains_yield should exist");
    let emit_stmt_wrapper = &lib_src[emit_stmt_start..body_contains_yield_start];
    assert!(emit_stmt_wrapper.contains("structured statement emission missing for production path"));
    assert!(!emit_stmt_wrapper.contains("self.try_emit_stmt_string_"));
    assert!(
        !emit_stmt_wrapper.contains("match stmt"),
        "emit_stmt should stay orchestration-only"
    );

    let lib_lines = lib_src.lines().count();
    assert!(
        lib_lines <= 1450,
        "lib.rs should stay decomposed (current lines: {lib_lines})"
    );
}

#[test]
fn test_production_lowering_contract_uses_result_helpers_only() {
    let lib_src = include_str!("lib.rs");
    let lower_expr_src = include_str!("lower_expr.rs");
    let module_constants_src = include_str!("module_constants.rs");
    let expr_render_helpers_src = include_str!("expr_render_helpers.rs");

    assert!(lib_src.contains("try_lower_simple_stmt_with_scope_result("));
    assert!(lower_expr_src.contains("pub fn try_lower_leaf_expr_result("));
    assert!(module_constants_src.contains("try_lower_simple_module_constant_item_result("));
    assert!(expr_render_helpers_src.contains("try_lower_registry_expr_result("));

    assert!(!lib_src.contains("try_lower_simple_stmt_with_scope("));
    assert!(!lib_src.contains("try_lower_leaf_expr("));
    assert!(!module_constants_src.contains("try_lower_simple_module_constant_item("));
}

#[test]
fn test_capture_structured_stmts_collects_ir_without_output_writes() {
    let mut emitter = RustEmitter::new();
    let stmt = HirStmt::Let {
        name: "x".to_string(),
        ty: Type::Int,
        value: HirExpr::IntLiteral(1),
        is_mutable: false,
    };

    let captured = emitter.capture_structured_stmts(|inner| inner.emit_stmt(&stmt));

    assert_eq!(captured.len(), 1);
    assert!(matches!(
        captured.first(),
        Some(RustStmt::Let {
            name,
            mutable: false,
            ..
        }) if name == "x"
    ));
}

#[test]
fn test_union_display_impl_uses_structured_ir() {
    let union_src = include_str!("union_type_helpers.rs");
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
    assert!(!generate_block.contains("assert_output_drained("));
    assert!(!generate_block.contains("emitter.output"));
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
    assert!(!generate_block.contains("assert_output_drained("));
    assert!(!generate_block.contains("emitter.output"));
    assert!(!generate_block.contains("module_import_prelude"));
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
    assert!(!module_constants_src.contains("render_items(&[item])"));

    assert!(entrypoints_src.contains("if !emitter.body_items.is_empty() {"));
    assert!(lib_src.contains("if !emitter.body_items.is_empty() {"));
    assert!(!entrypoints_src.contains("assert_output_drained("));
    assert!(!entrypoints_src.contains("emitter.output"));
    assert!(!lib_src.contains("emitter.output"));
}

#[test]
fn test_module_body_flows_through_assembled_body_items() {
    let module_body_src = include_str!("module_body.rs");
    let lib_src = include_str!("lib.rs");

    assert!(!module_body_src.contains("self.drain_emitted_output_items("));
    assert!(!module_body_src.contains("self.push_syn_items_from_source(&emitted"));
    assert!(module_body_src.contains("self.emit_class(class, module, module_public);"));
    assert!(module_body_src.contains("self.emit_function(func, module_public, test_mode);"));
    assert!(!module_body_src.contains("self.output"));
    assert!(lib_src.contains("if !emitter.body_items.is_empty() {"));
}

#[test]
fn test_generator_init_emission_is_structured_only() {
    let stmt_support_src = include_str!("stmt_support_emitter.rs");
    assert!(stmt_support_src.contains("self.lower_stmt_expr_for_ir(value)"));
    assert!(stmt_support_src.contains("self.try_lower_structured_stmt(stmt)"));
    assert!(stmt_support_src
        .contains("structured generator-init expression emission missing for production path"));
    assert!(stmt_support_src
        .contains("structured generator-init statement emission missing for production path"));
    assert!(!stmt_support_src.contains("self.try_emit_expr_string_"));
    assert!(!stmt_support_src.contains("self.try_emit_stmt_string_"));
    assert!(!stmt_support_src.contains("self.emit_expr(value);"));
    assert!(!stmt_support_src.contains("self.emit_stmt(stmt);"));
}

#[test]
fn test_expr_side_effect_emitter_layer_is_removed() {
    let expr_render_helpers_src = include_str!("expr_render_helpers.rs");
    let output_helpers_src = include_str!("output_helpers.rs");
    let intrinsic_emitters_src = include_str!("intrinsic_method_emitters.rs");
    let lib_support_src = include_str!("lib_support.rs");

    assert!(!expr_render_helpers_src.contains("fn try_emit_structured_"));
    assert!(!expr_render_helpers_src.contains("emit_fstring_macro("));
    assert!(!output_helpers_src.contains("expression string emission is forbidden"));
    assert!(!intrinsic_emitters_src.contains("write_registry_expr"));
    assert!(!lib_support_src.contains("reserved_plain_builtin"));
}

#[test]
fn test_production_codegen_source_has_no_non_ir_tokens() {
    let src_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let banned_tokens = [
        "RawCode",
        "SynItem",
        "self.write(",
        "self.writeln(",
        "emit_rust_expr(",
        "emit_rust_stmt_with_current_indent(",
        "write_registry_expr(",
    ];

    let mut stack = vec![src_root];
    let mut violations = Vec::new();
    while let Some(dir) = stack.pop() {
        let entries = std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("failed to read source dir {}: {e}", dir.display()));
        for entry in entries {
            let path = entry
                .unwrap_or_else(|e| {
                    panic!("failed to read directory entry in {}: {e}", dir.display())
                })
                .path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                continue;
            }
            if path.file_name().and_then(|name| name.to_str()) == Some("lib_codegen_tests.rs") {
                continue;
            }
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
            for token in banned_tokens {
                if content.contains(token) {
                    violations.push(format!(
                        "{} contains forbidden token `{token}`",
                        path.display()
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "production codegen source contains forbidden non-IR tokens:\n{}",
        violations.join("\n")
    );
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
