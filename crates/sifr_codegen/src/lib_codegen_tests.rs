use crate::{
    generate_project_with_deps_and_crates, generate_rust, generate_rust_multi,
    generate_rust_multi_with_metadata, generate_rust_test, generate_rust_with_metadata,
    RustEmitter, RustExpr, RustStmt, RustType, StdlibCode,
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
fn test_async_generator_codegen_uses_lazy_materialization() {
    let rust_code = generate_rust_from_source(
        "async def numbers() -> AsyncGenerator[int, GeneratorCloseError]:\n    yield 1\n    yield 2\n",
    );

    assert!(rust_code.contains("AsyncGenerator::new_lazy"));
    assert!(rust_code.contains("move ||"));
    assert!(!rust_code.contains("AsyncGenerator::new(_yields)"));
}

fn empty_module() -> HirModule {
    HirModule {
        functions: vec![],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    }
}

#[test]
fn test_generate_rust_preserves_loop_else_recursion_and_try_except_returns() {
    let loop_else_recursion = generate_rust_from_source(
        "def main():\n\
    def recurse(n: int) -> int:\n\
        for i in [1]:\n\
            pass\n\
        else:\n\
            if n > 0:\n\
                return recurse(n - 1)\n\
        return 0\n\
\n\
    print(recurse(4))\n",
    );
    assert!(loop_else_recursion.contains("fn recurse(n: i64) -> i64"));
    assert!(
        loop_else_recursion.contains("if !_broke") || loop_else_recursion.contains("if !(_broke)")
    );

    let try_except_return = generate_rust_from_source(
        "def classify(n: int) -> int:\n\
    try:\n\
        if n > 0:\n\
            return n\n\
        else:\n\
            raise ValueError('non-positive')\n\
    except ValueError as e:\n\
        return 99\n",
    );
    assert!(
        try_except_return.contains("return Err(ValueError::new(\"non-positive\".to_string()));")
    );
    assert!(try_except_return.contains("return 99 as i64;"));

    let loop_guard_narrowing = generate_rust_from_source(
        "def summarize(values: list[int]) -> int:\n\
    total: int = 0\n\
    for value in values:\n\
        if value > 10:\n\
            total = total + value\n\
        else:\n\
            total = total + 1\n\
    return total\n",
    );
    assert!(loop_guard_narrowing.contains("fn summarize(values: &Vec<i64>) -> i64"));
    assert!(loop_guard_narrowing.contains("if value > (10 as i64)"));
}

#[test]
fn test_generate_rust_elides_unreachable_returns_after_always_exit_paths() {
    let always_exit_try = generate_rust_from_source(
        "def classify(flag: bool) -> int:\n\
    try:\n\
        if flag:\n\
            return 5\n\
        raise ValueError('bad value')\n\
        return 11\n\
    except ValueError as e:\n\
        return 77\n",
    );
    assert!(always_exit_try.contains("return Err(ValueError::new(\"bad value\".to_string()));"));
    assert!(always_exit_try.contains("return 77 as i64;"));
    assert!(!always_exit_try.contains("11 as i64"));

    let unreachable_tail = generate_rust_from_source(
        "def inferred(flag: bool):\n\
    if flag:\n\
        return 1\n\
    return 2\n\
    return 'never'\n",
    );
    assert!(unreachable_tail.contains("fn inferred(flag: bool) -> i64"));
    assert!(unreachable_tail.contains("return 2 as i64;"));
    assert!(!unreachable_tail.contains("never"));
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
fn test_generate_rust_guarded_list_pop_unwraps_compiler_verified_nonempty() {
    let rust_code = generate_rust_from_source(
        "def main():\n    values: list[int] = [1, 2]\n    while values:\n        _: int = values.pop()\n",
    );

    assert!(rust_code.contains("let Some(__sifr_nonempty_pop_value) = values.pop() else {"));
    assert!(rust_code.contains("compiler-verified non-empty pop should return Some"));
}

#[test]
fn test_generate_rust_guarded_list_pop_zero_unwraps_compiler_verified_nonempty() {
    let rust_code = generate_rust_from_source(
        "def main():\n    values: list[int] = [1, 2]\n    while values:\n        _: int = values.pop(0)\n",
    );

    assert!(rust_code.contains("compiler-verified non-empty pop should return Some"));
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
                is_async: false,
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
                is_async: false,
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
                is_async: false,
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
                is_async: false,
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
    assert!(rust_code.contains("iter().copied().map(|x| x * x)"));
}

#[test]
fn test_generate_rust_filter_over_list_lowers_to_lazy_boxed_iterator() {
    let rust_code = generate_rust_from_source(
        "def main():\n    nums: list[int] = [1, 2, 3, 4]\n    evens: Iterator[int] = filter(lambda x: x % 2 == 0, nums)\n    print(list(evens))\n",
    );

    assert!(rust_code.contains("let mut evens: Box<dyn Iterator<Item = i64>>"));
    assert!(rust_code.contains("Box::new("));
    assert!(rust_code.contains(".iter().copied().filter("));
}

#[test]
fn test_generate_rust_iterable_binding_from_iterator_materializes_once() {
    let rust_code = generate_rust_from_source(
        "def main():\n    base: list[int] = [1, 2, 3]\n    it: Iterator[int] = iter(base)\n    xs: Iterable[int] = it\n    print(list(xs))\n",
    );

    assert!(rust_code.contains("let xs: Vec<i64> = (it).into_iter().collect::<Vec<_>>();"));
}

#[test]
fn test_generate_rust_iterable_return_from_iterator_materializes_for_signature() {
    let rust_code = generate_rust_from_source(
        "def adapt(own it: Iterator[int]) -> Iterable[int]:\n    return it\n\ndef main():\n    base: list[int] = [1, 2]\n    it: Iterator[int] = iter(base)\n    xs: Iterable[int] = adapt(it)\n    print(list(xs))\n",
    );

    assert!(rust_code.contains("fn adapt(it: Box<dyn Iterator<Item = i64>>) -> Vec<i64> {"));
    assert!(rust_code.contains("return it.collect::<Vec<_>>();"));
}

#[test]
fn test_generate_rust_iterator_return_consumes_local_list_binding() {
    let rust_code = generate_rust_from_source(
        "def build() -> Iterator[int]:\n    result: list[int] = [1, 2, 3]\n    return iter(result)\n\ndef main():\n    print(list(build()))\n",
    );

    assert!(rust_code.contains("fn build() -> Box<dyn Iterator<Item = i64>> {"));
    assert!(rust_code.contains("return Box::new(result.into_iter());"));
}

#[test]
fn test_generate_rust_iterator_return_consumes_owned_param_binding() {
    let rust_code = generate_rust_from_source(
        "def adapt(own items: list[int]) -> Iterator[int]:\n    return iter(items)\n\ndef main():\n    print(list(adapt([1, 2, 3])))\n",
    );

    assert!(rust_code.contains("fn adapt(items: Vec<i64>) -> Box<dyn Iterator<Item = i64>> {"));
    assert!(rust_code.contains("return Box::new(items.into_iter());"));
}

#[test]
fn test_generate_rust_open_uses_canonical_filehandle_constructor() {
    let rust_code = generate_rust_from_source(
        "def main():\n    f = open(\"/tmp/sifr_codegen_open.txt\", \"w\")\n",
    );

    assert!(rust_code.contains("struct FileHandle"));
    assert!(rust_code.contains("fn new(_handle: i64, _mode: String) -> Self"));
    assert!(rust_code.contains("return Ok(FileHandle::new(__handle_id, __mode.to_string()));"));
    assert!(!rust_code
        .contains("return Ok(FileHandle { _handle: __handle_id, _mode: __mode.to_string() });"));
}

#[test]
fn test_generate_rust_generator_clones_borrowed_params_into_owned_locals_before_calls() {
    let rust_code = generate_rust_from_source(
        "def glob(directory: str, pattern: str) -> list[str]:\n    return []\n\ndef iglob(directory: str, pattern: str) -> Iterator[str]:\n    matches: list[str] = glob(directory, pattern)\n    i: int = 0\n    while i < len(matches):\n        yield matches[i]\n        i = i + 1\n",
    );

    assert!(rust_code.contains("let directory = directory.clone();"));
    assert!(rust_code.contains("let pattern = pattern.clone();"));
    assert!(rust_code.contains("let matches: Vec<String> = glob(&directory, &pattern);"));
}

#[test]
fn test_generate_rust_recursive_constructor_argument_wraps_optional_box_field() {
    let rust_code = generate_rust_from_source(
        "class Entry:\n    value: int\n    next: Entry | None\n\n    def __init__(self, value: int = 0, next: Entry | None = None):\n        self.value = value\n        self.next = next\n\ndef main():\n    long = Entry(4, Entry(5, Entry(6)))\n    print(long.value)\n",
    );

    assert!(rust_code.contains(
        "let long: Entry = Entry::new(4 as i64, Some(Box::new(Entry::new(5 as i64, Some(Box::new(Entry::new(6 as i64, None)))))));"
    ));
}

#[test]
fn test_generate_rust_defaultdict_int_augassign_uses_entry_default() {
    let rust_code = generate_rust_from_source(
        "def main():\n    counts = defaultdict(int)\n    counts[\"steps\"] += 1\n    counts[\"steps\"] += 2\n    assert counts[\"steps\"] == 3\n",
    );

    assert!(rust_code.contains("let __elem = counts.entry(\"steps\".to_string()).or_insert(0);"));
    assert!(rust_code.contains("*__elem += 1 as i64;"));
    assert!(rust_code.contains("*__elem += 2 as i64;"));
}

#[test]
fn test_generate_rust_tuple_field_assignment_emits_mutable_self_receiver() {
    let rust_code = generate_rust_from_source(
        "class RunningBounds:\n    left: int\n    right: int\n\n    def __init__(self, left: int, right: int):\n        self.left = left\n        self.right = right\n\n    def rotate(self, next_value: int) -> None:\n        self.left, self.right = self.right, next_value\n",
    );

    assert!(rust_code.contains("fn rotate(&mut self, next_value: i64)"));
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
                is_async: false,
                method_kind: MethodKind::Regular,
                decorators: vec![],
                type_params: vec![],
            },
            HirFunction {
                name: "test_sample".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![HirStmt::Pass],
                is_async: false,
                method_kind: MethodKind::Regular,
                decorators: vec![],
                type_params: vec![],
            },
            HirFunction {
                name: "helper".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![HirStmt::Pass],
                is_async: false,
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
                is_async: false,
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
                is_async: false,
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
            is_async: false,
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
                    is_async: false,
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
                    is_async: false,
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
                    is_async: false,
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
    assert!(rust_code.contains("return self.table.get(\"k\").copied();"));
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
        is_async: false,
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
        "fn dfs(i: i64, res: &mut Vec<Vec<i64>>, subset: &mut Vec<i64>, values: &Vec<i64>)"
    ));
    assert!(
        generated.contains("dfs(0_i64, &mut res, &mut subset, &values);")
            || generated.contains("dfs(0 as i64, &mut res, &mut subset, &values);")
    );
    assert!(
        generated.contains("dfs(i + (1_i64), res, subset, values);")
            || generated.contains("dfs((i + 1 as i64), res, subset, values);")
    );
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
            is_async: false,
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
            is_async: false,
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
                is_async: false,
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
                is_async: false,
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
    let Some(RustStmt::Let {
        name, ty, value, ..
    }) = captured.first()
    else {
        panic!("expected structured let for optional local");
    };
    assert_eq!(name, "part");
    assert!(
        matches!(
            ty,
            Some(RustType::Option(inner)) if matches!(inner.as_ref(), RustType::String_)
        ) || matches!(ty, Some(RustType::Named(named)) if named == "Option<String>")
    );
    assert!(matches!(
        value,
        RustExpr::FnCall { func, args }
            if matches!(func.as_ref(), RustExpr::Path(path) if path == &vec!["Some".to_string()])
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
fn test_structured_stmt_path_handles_nested_subscript_augassign_inside_loop_if() {
    let stmt = HirStmt::For {
        target: "i".to_string(),
        target_ty: Type::Int,
        iter: HirExpr::RangeLiteral {
            start: Box::new(HirExpr::IntLiteral(0)),
            end: Box::new(HirExpr::IntLiteral(3)),
            step: None,
            ty: Type::Range,
        },
        body: vec![HirStmt::If {
            condition: HirExpr::BoolLiteral(true),
            then_body: vec![HirStmt::SubscriptAugAssign {
                object: "values".to_string(),
                index: HirExpr::BinOp {
                    left: Box::new(HirExpr::Name {
                        name: "i".to_string(),
                        ty: Type::Int,
                    }),
                    op: "+".to_string(),
                    right: Box::new(HirExpr::IntLiteral(1)),
                    ty: Type::Int,
                },
                op: "*=".to_string(),
                value: HirExpr::IntLiteral(2),
                object_ty: Type::List(Box::new(Type::Int)),
            }],
            elif_clauses: vec![],
            else_body: None,
        }],
        else_body: None,
    };

    let mut emitter = RustEmitter::new();
    let captured = emitter.capture_structured_stmts(|inner| inner.emit_stmt(&stmt));

    assert!(matches!(captured.first(), Some(RustStmt::For { .. })));
}

#[test]
fn test_structured_stmt_path_handles_delete_with_name_key_inside_loop_if() {
    let stmt = HirStmt::For {
        target: "ch".to_string(),
        target_ty: Type::Str,
        iter: HirExpr::Name {
            name: "order".to_string(),
            ty: Type::Str,
        },
        body: vec![HirStmt::If {
            condition: HirExpr::ContainsOp {
                element: Box::new(HirExpr::Name {
                    name: "ch".to_string(),
                    ty: Type::Str,
                }),
                collection: Box::new(HirExpr::Name {
                    name: "counts".to_string(),
                    ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
                }),
                ty: Type::Bool,
            },
            then_body: vec![HirStmt::Delete {
                object: HirExpr::Name {
                    name: "counts".to_string(),
                    ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
                },
                index: HirExpr::Name {
                    name: "ch".to_string(),
                    ty: Type::Str,
                },
            }],
            elif_clauses: vec![],
            else_body: None,
        }],
        else_body: None,
    };

    let mut emitter = RustEmitter::new();
    let captured = emitter.capture_structured_stmts(|inner| inner.emit_stmt(&stmt));

    assert!(matches!(captured.first(), Some(RustStmt::For { .. })));
}

#[test]
fn test_structured_stmt_path_handles_chained_compare_condition_inside_loop_if() {
    let stmt = HirStmt::While {
        condition: HirExpr::Compare {
            left: Box::new(HirExpr::Name {
                name: "left".to_string(),
                ty: Type::Int,
            }),
            ops: vec!["<=".to_string()],
            comparators: vec![HirExpr::Name {
                name: "right".to_string(),
                ty: Type::Int,
            }],
            ty: Type::Bool,
        },
        body: vec![HirStmt::If {
            condition: HirExpr::Compare {
                left: Box::new(HirExpr::Name {
                    name: "left".to_string(),
                    ty: Type::Int,
                }),
                ops: vec!["<=".to_string(), "<".to_string()],
                comparators: vec![
                    HirExpr::Name {
                        name: "target".to_string(),
                        ty: Type::Int,
                    },
                    HirExpr::Name {
                        name: "right".to_string(),
                        ty: Type::Int,
                    },
                ],
                ty: Type::Bool,
            },
            then_body: vec![HirStmt::Assign {
                name: "left".to_string(),
                value: HirExpr::IntLiteral(1),
            }],
            elif_clauses: vec![],
            else_body: Some(vec![HirStmt::AugAssign {
                name: "left".to_string(),
                op: "+=".to_string(),
                value: HirExpr::IntLiteral(1),
            }]),
        }],
        else_body: None,
    };

    let mut emitter = RustEmitter::new();
    let captured = emitter.capture_structured_stmts(|inner| inner.emit_stmt(&stmt));

    assert!(matches!(captured.first(), Some(RustStmt::While { .. })));
}

#[test]
fn test_structured_stmt_path_lowers_collection_truthiness_inside_boolop_condition() {
    let tuple_ty = Type::Tuple(vec![Type::Int, Type::Int]);
    let stmt = HirStmt::While {
        condition: HirExpr::BoolOp {
            op: "and".to_string(),
            values: vec![
                HirExpr::Name {
                    name: "stack".to_string(),
                    ty: Type::List(Box::new(tuple_ty.clone())),
                },
                HirExpr::Compare {
                    left: Box::new(HirExpr::Index {
                        object: Box::new(HirExpr::Index {
                            object: Box::new(HirExpr::Name {
                                name: "stack".to_string(),
                                ty: Type::List(Box::new(tuple_ty.clone())),
                            }),
                            index: Box::new(HirExpr::UnaryOp {
                                op: "-".to_string(),
                                operand: Box::new(HirExpr::IntLiteral(1)),
                                ty: Type::Int,
                            }),
                            ty: Type::Union(vec![tuple_ty.clone(), Type::None]),
                        }),
                        index: Box::new(HirExpr::IntLiteral(1)),
                        ty: Type::Int,
                    }),
                    ops: vec![">".to_string()],
                    comparators: vec![HirExpr::Name {
                        name: "h".to_string(),
                        ty: Type::Int,
                    }],
                    ty: Type::Bool,
                },
            ],
            ty: Type::Bool,
        },
        body: vec![HirStmt::Let {
            name: "x".to_string(),
            ty: Type::Int,
            value: HirExpr::IntLiteral(1),
            is_mutable: true,
        }],
        else_body: None,
    };

    let mut emitter = RustEmitter::new();
    let captured = emitter.capture_structured_stmts(|inner| inner.emit_stmt(&stmt));

    let Some(RustStmt::While { cond, .. }) = captured.first() else {
        panic!("expected while stmt");
    };
    let rendered = crate::render_expr(cond);
    assert!(rendered.contains("is_empty"));
    assert!(rendered.contains("&&"));
}

#[test]
fn test_structured_stmt_path_lowers_option_call_truthiness_to_bool_condition() {
    let stmt = HirStmt::If {
        condition: HirExpr::MethodCall {
            object: Box::new(HirExpr::Name {
                name: "nums".to_string(),
                ty: Type::Dict(Box::new(Type::Int), Box::new(Type::Int)),
            }),
            method: "get".to_string(),
            args: vec![HirExpr::Name {
                name: "i".to_string(),
                ty: Type::Int,
            }],
            ty: Type::Union(vec![Type::Int, Type::None]),
        },
        then_body: vec![HirStmt::Pass],
        elif_clauses: vec![],
        else_body: None,
    };

    let mut emitter = RustEmitter::new();
    let captured = emitter.capture_structured_stmts(|inner| inner.emit_stmt(&stmt));

    assert!(matches!(
        captured.first(),
        Some(RustStmt::If {
            cond: RustExpr::MethodCall { method, .. },
            ..
        }) if method == "is_some_and"
    ));
}

#[test]
fn test_structured_stmt_path_lowers_nested_string_augassign_to_push_str() {
    let stmt = HirStmt::While {
        condition: HirExpr::BoolLiteral(true),
        body: vec![HirStmt::If {
            condition: HirExpr::BoolLiteral(true),
            then_body: vec![HirStmt::AugAssign {
                name: "out".to_string(),
                op: "+=".to_string(),
                value: HirExpr::Name {
                    name: "part".to_string(),
                    ty: Type::Str,
                },
            }],
            elif_clauses: vec![],
            else_body: None,
        }],
        else_body: None,
    };

    let mut emitter = RustEmitter::new();
    let captured = emitter.capture_structured_stmts(|inner| inner.emit_stmt(&stmt));

    let Some(RustStmt::While { body, .. }) = captured.first() else {
        panic!("expected while stmt");
    };
    let Some(RustStmt::If { then_body, .. }) = body.first() else {
        panic!("expected nested if stmt");
    };
    assert!(matches!(
        then_body.first(),
        Some(RustStmt::Expr(RustExpr::MethodCall { method, .. })) if method == "push_str"
    ));
}

#[test]
fn test_structured_stmt_path_string_contains_avoids_double_borrow_pattern_arg() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "contains_self".to_string(),
            params: vec![
                HirParam {
                    name: "text".to_string(),
                    ty: Type::Str,
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::borrow(),
                },
                HirParam {
                    name: "s".to_string(),
                    ty: Type::Str,
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::borrow(),
                },
            ],
            return_type: Type::Bool,
            body: vec![HirStmt::Return {
                value: Some(HirExpr::ContainsOp {
                    element: Box::new(HirExpr::Name {
                        name: "s".to_string(),
                        ty: Type::Str,
                    }),
                    collection: Box::new(HirExpr::Name {
                        name: "text".to_string(),
                        ty: Type::Str,
                    }),
                    ty: Type::Bool,
                }),
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

    let generated = generate_rust_with_metadata(&module);
    assert!(!generated.rust_source.contains(".contains(&(s))"));
    assert!(generated.rust_source.contains(".contains("));
    assert!(!generated.rust_source.contains(".contains(&("));
}

#[test]
fn test_plain_call_canonicalizes_heapq_compat_symbol_name() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "touch".to_string(),
            params: vec![HirParam {
                name: "min_h".to_string(),
                ty: Type::List(Box::new(Type::Int)),
                default: None,
                keyword_only: false,
                convention: ParamConvention::mut_borrow(),
            }],
            return_type: Type::None,
            body: vec![HirStmt::Expr {
                expr: HirExpr::Call {
                    func: "__compat_sifr_heapq_heapify".to_string(),
                    args: vec![HirExpr::Name {
                        name: "min_h".to_string(),
                        ty: Type::List(Box::new(Type::Int)),
                    }],
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

    let generated = generate_rust_with_metadata(&module);
    assert!(generated.rust_source.contains("heapify("));
    assert!(!generated
        .rust_source
        .contains("__compat_sifr_heapq_heapify("));
}

#[test]
fn test_list_builtin_uses_owned_collection_for_unknown_set_with_list_hint() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "to_list".to_string(),
            params: vec![HirParam {
                name: "result".to_string(),
                ty: Type::Set(Box::new(Type::Any)),
                default: None,
                keyword_only: false,
                convention: ParamConvention::borrow(),
            }],
            return_type: Type::List(Box::new(Type::Str)),
            body: vec![HirStmt::Return {
                value: Some(HirExpr::Call {
                    func: "list".to_string(),
                    args: vec![HirExpr::Name {
                        name: "result".to_string(),
                        ty: Type::Set(Box::new(Type::Any)),
                    }],
                    ty: Type::List(Box::new(Type::Str)),
                }),
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

    let generated = generate_rust_with_metadata(&module);
    assert!(generated
        .rust_source
        .contains(".iter().cloned().collect::<Vec<_>>()"));
}

#[test]
fn test_set_builtin_with_generator_lowers_to_collect_not_plain_set_call() {
    let generator = HirExpr::GeneratorExpr {
        expr: Box::new(HirExpr::Call {
            func: "str".to_string(),
            args: vec![HirExpr::Name {
                name: "i".to_string(),
                ty: Type::Int,
            }],
            ty: Type::Str,
        }),
        var: "i".to_string(),
        iter: Box::new(HirExpr::RangeLiteral {
            start: Box::new(HirExpr::IntLiteral(0)),
            end: Box::new(HirExpr::IntLiteral(3)),
            step: None,
            ty: Type::Range,
        }),
        filter: None,
        ty: Type::Iterator(Box::new(Type::Str)),
    };
    let module = HirModule {
        functions: vec![HirFunction {
            name: "build_set".to_string(),
            params: vec![],
            return_type: Type::Set(Box::new(Type::Str)),
            body: vec![HirStmt::Return {
                value: Some(HirExpr::Call {
                    func: "set".to_string(),
                    args: vec![generator],
                    ty: Type::Set(Box::new(Type::Str)),
                }),
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

    let generated = generate_rust_with_metadata(&module);
    assert!(generated
        .rust_source
        .contains("collect::<std::collections::HashSet<_>>()"));
    assert!(!generated.rust_source.contains("return set("));
}

#[test]
fn test_list_repeat_lowers_without_vec_mul_shape() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "repeat_zero".to_string(),
            params: vec![HirParam {
                name: "n".to_string(),
                ty: Type::Int,
                default: None,
                keyword_only: false,
                convention: ParamConvention::own(),
            }],
            return_type: Type::List(Box::new(Type::Int)),
            body: vec![HirStmt::Return {
                value: Some(HirExpr::BinOp {
                    left: Box::new(HirExpr::ListLiteral {
                        elements: vec![HirExpr::IntLiteral(0)],
                        ty: Type::List(Box::new(Type::Int)),
                    }),
                    op: "*".to_string(),
                    right: Box::new(HirExpr::Name {
                        name: "n".to_string(),
                        ty: Type::Int,
                    }),
                    ty: Type::List(Box::new(Type::Int)),
                }),
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

    let generated = generate_rust_with_metadata(&module);
    assert!(!generated.rust_source.contains("vec![0 as i64] * n"));
    assert!(generated.rust_source.contains("__sifr_repeat_out.extend("));
}

#[test]
fn test_compare_lowers_int_float_mixed_operands_with_cast() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "cmp".to_string(),
            params: vec![
                HirParam {
                    name: "coins".to_string(),
                    ty: Type::Float,
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::own(),
                },
                HirParam {
                    name: "n".to_string(),
                    ty: Type::Int,
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::own(),
                },
            ],
            return_type: Type::Bool,
            body: vec![HirStmt::Return {
                value: Some(HirExpr::Compare {
                    left: Box::new(HirExpr::Name {
                        name: "coins".to_string(),
                        ty: Type::Float,
                    }),
                    ops: vec![">".to_string()],
                    comparators: vec![HirExpr::Name {
                        name: "n".to_string(),
                        ty: Type::Int,
                    }],
                    ty: Type::Bool,
                }),
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

    let generated = generate_rust_with_metadata(&module);
    assert!(generated.rust_source.contains("as f64"));
}

#[test]
fn test_bool_typed_boolop_coerces_optional_operand_to_condition_bool() {
    let optional_index = HirExpr::Index {
        object: Box::new(HirExpr::Name {
            name: "grid2".to_string(),
            ty: Type::List(Box::new(Type::Int)),
        }),
        index: Box::new(HirExpr::Name {
            name: "i".to_string(),
            ty: Type::Int,
        }),
        ty: Type::Union(vec![Type::Int, Type::None]),
    };
    let module = HirModule {
        functions: vec![HirFunction {
            name: "cond".to_string(),
            params: vec![
                HirParam {
                    name: "grid2".to_string(),
                    ty: Type::List(Box::new(Type::Int)),
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::borrow(),
                },
                HirParam {
                    name: "i".to_string(),
                    ty: Type::Int,
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::own(),
                },
            ],
            return_type: Type::Bool,
            body: vec![HirStmt::Return {
                value: Some(HirExpr::BoolOp {
                    op: "and".to_string(),
                    values: vec![optional_index, HirExpr::BoolLiteral(true)],
                    ty: Type::Bool,
                }),
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

    let generated = generate_rust_with_metadata(&module);
    assert!(generated.rust_source.contains("is_some_and"));
}

#[test]
fn test_string_slice_negative_stop_normalizes_against_length() {
    let rust_code = generate_rust_from_source(
        "def repeatedSubstringPattern(s: str) -> bool:\n    return s in (s + s)[1:-1]\n",
    );
    assert!(
        rust_code.contains("_slice_len_i64"),
        "string slice lowering should materialize source length for negative-stop normalization"
    );
    assert!(
        rust_code.contains("_slice_stop_i64"),
        "string slice lowering should compute normalized stop bound"
    );
    assert!(
        !rust_code.contains("((-(1 as i64)).max(0) - (1 as i64).max(0)).max(0)"),
        "negative stop must not be clamped directly to zero"
    );
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
fn test_generate_project_emits_sifr_runtime_path_dependency_when_required() {
    let module = empty_module();
    let required_crates = HashSet::from(["sifr_runtime".to_string()]);
    let (cargo_toml, _main_rs) = generate_project_with_deps_and_crates(
        &module,
        "sifr_output",
        &HashSet::new(),
        &required_crates,
    );

    assert!(cargo_toml.contains("sifr_runtime = { path = "));
}

#[test]
fn test_async_main_entrypoint_gets_tokio_bootstrap_dependency() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module("async def main() -> None:\n    return None\n")
                .expect("parse failed")
                .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result
        .rust_source
        .contains("#[tokio::main(flavor = \"current_thread\")]"));
    assert!(result.rust_source.contains("async fn main()"));
    assert!(result.required_crates.contains("tokio"));
}

#[test]
fn test_async_result_main_entrypoint_keeps_result_return() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module("async def main() -> Result[None, ValueError]:\n    return None\n")
                .expect("parse failed")
                .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result
        .rust_source
        .contains("#[tokio::main(flavor = \"current_thread\")]"));
    assert!(result
        .rust_source
        .contains("async fn main() -> Result<(), ValueError>"));
    assert!(result.rust_source.contains("return Ok(());"));
    assert!(result.required_crates.contains("tokio"));
}

#[test]
fn test_task_sleep_lowers_to_tokio_sleep_and_requires_tokio() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module("async def main() -> None:\n    await task.sleep(0.0)\n    return None\n")
                .expect("parse failed")
                .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("tokio::time::sleep"));
    assert!(result
        .rust_source
        .contains("std::time::Duration::from_secs_f64"));
    assert!(result.required_crates.contains("tokio"));
}

#[test]
fn test_task_sleep_requires_tokio_without_async_main() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def wait_once() -> None:\n    await task.sleep(0.0)\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(!result
        .rust_source
        .contains("#[tokio::main(flavor = \"current_thread\")]"));
    assert!(result.required_crates.contains("tokio"));
}

#[test]
fn test_task_scope_context_materializes_runtime_container() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def main() -> None:\n    async with task.scope() as scope:\n        await task.sleep(0.0)\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("struct __SifrTaskScope"));
    assert!(result.rust_source.contains("impl __SifrTaskScope"));
    assert!(result
        .rust_source
        .contains("let mut scope = __SifrTaskScope::new();"));
    assert!(result
        .rust_source
        .contains("scope.__sifr_join_all().await;"));
    assert!(result.required_crates.contains("tokio"));
}

#[test]
fn test_scope_spawn_lowers_to_owned_task_handle_substrate() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def worker() -> int:\n    return 41\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        handle = scope.spawn(worker())\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("struct __SifrTask<T, E>"));
    assert!(result.rust_source.contains("fn __sifr_spawn_infallible<"));
    assert!(result.rust_source.contains("struct __SifrScopeChild"));
    assert!(result.rust_source.contains("tokio::sync::oneshot::channel"));
    assert!(result
        .rust_source
        .contains("scope.__sifr_spawn_infallible(worker());"));
    assert!(result
        .rust_source
        .contains("if let Err(__sifr_scope_failure) = scope.__sifr_join_all().await"));
    assert!(result.required_crates.contains("tokio"));
}

#[test]
fn test_spawn_blocking_lowers_to_distinct_blocking_task_substrate() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "def compute_value() -> int:\n    return 42\n\nasync def main() -> Result[None, ScopeFailure]:\n    handle = task.spawn_blocking(compute_value)\n    result = await handle\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result
        .rust_source
        .contains("struct __SifrBlockingTask<T, E>"));
    assert!(result
        .rust_source
        .contains("fn __sifr_spawn_blocking_infallible<"));
    assert!(result
        .rust_source
        .contains("tokio::task::spawn_blocking(move || __SifrTaskResult::Ok(work()))"));
    assert!(result
        .rust_source
        .contains("__sifr_spawn_blocking_infallible(compute_value);"));
    assert!(result.required_crates.contains("tokio"));
}

#[test]
fn test_thread_pool_executor_submit_reuses_blocking_task_substrate() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "class ThreadPoolExecutor:\n    pass\n\n\ndef compute_value() -> int:\n    return 42\n\nasync def main() -> Result[None, ScopeFailure]:\n    executor: ThreadPoolExecutor = ThreadPoolExecutor()\n    handle = executor.submit(compute_value)\n    result = await handle\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result
        .rust_source
        .contains("struct __SifrBlockingTask<T, E>"));
    assert!(result
        .rust_source
        .contains("fn __sifr_spawn_blocking_infallible<"));
    assert!(result
        .rust_source
        .contains("__sifr_spawn_blocking_infallible(compute_value);"));
    assert!(result.required_crates.contains("tokio"));
}

#[test]
fn test_scope_spawn_lowers_owned_coroutine_arguments() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def worker(value: int) -> int:\n    return value\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        value: int = 41\n        handle = scope.spawn(worker(value))\n        result = await handle\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result
        .rust_source
        .contains("scope.__sifr_spawn_infallible(worker(value));"));
}

#[test]
fn test_scope_spawn_lowers_owned_move_coroutine_arguments() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def worker(own items: list[int]) -> int:\n    return len(items)\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        handle = scope.spawn(worker([1, 2]))\n        result = await handle\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result
        .rust_source
        .contains("scope.__sifr_spawn_infallible(worker(vec![1 as i64, 2 as i64]));"));
}

#[test]
fn test_task_group_basic_lowers_to_scope_runtime_substrate() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def worker() -> int:\n    return 41\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.TaskGroup() as group:\n        handle = group.spawn(worker())\n        result = await handle.join()\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("struct __SifrTaskScope"));
    assert!(result
        .rust_source
        .contains("let mut group = __SifrTaskScope::new_task_group();"));
    assert!(result.rust_source.contains("fail_fast"));
    assert!(result
        .rust_source
        .contains("group.__sifr_spawn_infallible(worker());"));
    assert!(result
        .rust_source
        .contains("if let Err(__sifr_scope_failure) = group.__sifr_join_all().await"));
    assert!(result.required_crates.contains("tokio"));
}

#[test]
fn test_task_gather_lowers_to_private_gather_helper() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def first() -> int:\n    return 1\n\nasync def second() -> int:\n    return 2\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        result = await task.gather([scope.spawn(first()), scope.spawn(second())])\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("async fn __sifr_task_gather"));
    assert!(result.rust_source.contains("__sifr_task_gather(vec!["));
    assert!(result.rust_source.contains("abort_handle.abort();"));
    assert!(result.rust_source.contains("failure_results"));
    assert!(result.rust_source.contains("push_secondary_message"));
    assert!(result.rust_source.contains("ordered_values.push(value);"));
    assert!(result
        .rust_source
        .contains("let result: __SifrTaskResult<Vec<i64>, std::convert::Infallible>"));
    assert!(result.required_crates.contains("tokio"));
}

#[test]
fn test_scope_spawn_fallible_coroutine_lowers_to_result_spawn_helper() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def worker() -> Result[int, ValueError]:\n    raise ValueError(\"bad\")\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        handle = scope.spawn(worker())\n        result = await handle\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result
        .rust_source
        .contains("tokio::sync::oneshot::Receiver<__SifrTaskResult<T, E>>"));
    assert!(result
        .rust_source
        .contains("scope.__sifr_spawn_result(worker());"));
    assert!(result.rust_source.contains("enum __SifrTaskResult<T, E>"));
    assert!(result.rust_source.contains("Err(__SifrFailure<E>)"));
    assert!(result
        .rust_source
        .contains("__SifrTaskResult::Err(__SifrFailure::new(err))"));
    assert!(result
        .rust_source
        .contains("let result: __SifrTaskResult<i64, ValueError>"));
    assert!(result.required_crates.contains("tokio"));
}

#[test]
fn test_task_gather_fallible_tasks_keeps_error_parameter_unwrapped() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def worker() -> Result[int, ValueError]:\n    raise ValueError(\"bad\")\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        result = await task.gather([scope.spawn(worker()), scope.spawn(worker())])\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("async fn __sifr_task_gather"));
    assert!(result.rust_source.contains("__SifrTaskResult<Vec<T>, E>"));
    assert!(result.rust_source.contains("Err(__SifrFailure<E>)"));
    assert!(result
        .rust_source
        .contains("sibling task failed\".to_string()"));
    assert!(result
        .rust_source
        .contains("sibling task was cancelled\".to_string()"));
    assert!(result
        .rust_source
        .contains("__SifrTaskResult::Err(__SifrFailure::new(err))"));
    assert!(result
        .rust_source
        .contains("let result: __SifrTaskResult<Vec<i64>, ValueError>"));
}

#[test]
fn test_task_race_lowers_to_private_race_helper() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def first() -> int:\n    return 1\n\nasync def second() -> int:\n    await task.sleep(1.0)\n    return 2\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        result = await task.race([scope.spawn(first()), scope.spawn(second())])\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("async fn __sifr_task_race"));
    assert!(result.rust_source.contains("__sifr_task_race(vec!["));
    assert!(result.rust_source.contains("let Some(mut first)"));
    assert!(result
        .rust_source
        .contains("race loser task failed\".to_string()"));
    assert!(result
        .rust_source
        .contains("let result: __SifrTaskResult<i64, std::convert::Infallible>"));
    assert!(result.required_crates.contains("tokio"));
}

#[test]
fn test_task_race_fallible_tasks_keeps_error_parameter_unwrapped() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def worker() -> Result[int, ValueError]:\n    raise ValueError(\"bad\")\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        result = await task.race([scope.spawn(worker()), scope.spawn(worker())])\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("async fn __sifr_task_race"));
    assert!(result.rust_source.contains("__SifrTaskResult<T, E>"));
    assert!(result.rust_source.contains("Err(__SifrFailure<E>)"));
    assert!(result
        .rust_source
        .contains("race loser task was cancelled\".to_string()"));
    assert!(result
        .rust_source
        .contains("__SifrTaskResult::Err(__SifrFailure::new(err))"));
    assert!(result
        .rust_source
        .contains("let result: __SifrTaskResult<i64, ValueError>"));
}

#[test]
fn test_task_select_lowers_to_private_select_helper() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def first() -> int:\n    return 1\n\nasync def second() -> str:\n    await task.sleep(1.0)\n    return \"two\"\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        first_handle = scope.spawn(first())\n        second_handle = scope.spawn(second())\n        result = await task.select(first_handle, second_handle)\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("enum __SifrSelect2<A, B>"));
    assert!(result.rust_source.contains("async fn __sifr_task_select"));
    assert!(result
        .rust_source
        .contains("__sifr_task_select(first_handle, second_handle)"));
    assert!(result
        .rust_source
        .contains("select loser task failed\".to_string()"));
    assert!(result
        .rust_source
        .contains("select loser task was cancelled\".to_string()"));
    assert!(result
        .rust_source
        .contains("second_observed.store(false, std::sync::atomic::Ordering::SeqCst)"));
    assert!(result
        .rust_source
        .contains("first_observed.store(false, std::sync::atomic::Ordering::SeqCst)"));
    assert!(result.rust_source.contains(
        "let result: __SifrSelect2<__SifrTaskResult<i64, std::convert::Infallible>, __SifrTaskResult<String, std::convert::Infallible>>"
    ));
    assert!(result.required_crates.contains("tokio"));
}

#[test]
fn test_task_select_fallible_tasks_preserves_distinct_error_parameters() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def first() -> Result[int, ValueError]:\n    raise ValueError(\"first\")\n\nasync def second() -> Result[str, IOError]:\n    raise IOError(\"second\")\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        first_handle = scope.spawn(first())\n        second_handle = scope.spawn(second())\n        result = await task.select(first_handle, second_handle)\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("__SifrTaskResult<A, EA>"));
    assert!(result.rust_source.contains("__SifrTaskResult<B, EB>"));
    assert!(result.rust_source.contains("Err(__SifrFailure<E>)"));
    assert!(result.rust_source.contains(
        "let result: __SifrSelect2<__SifrTaskResult<i64, ValueError>, __SifrTaskResult<String, IOError>>"
    ));
}

#[test]
fn test_task_handle_join_lowers_to_task_result_observation() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def worker() -> int:\n    return 41\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        handle = scope.spawn(worker())\n        result = await handle.join()\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("enum __SifrTaskResult<T, E>"));
    assert!(result.rust_source.contains("async fn join(self)"));
    assert!(result
        .rust_source
        .contains("Cancelled(__SifrFailure<CancellationError>)"));
    assert!(result.rust_source.contains("fn cancelled() -> Self"));
    assert!(result.rust_source.contains("handle.join().await"));
    assert!(result
        .rust_source
        .contains("let result: __SifrTaskResult<i64, std::convert::Infallible>"));
}

#[test]
fn test_await_task_handle_desugars_to_join_observation() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def worker() -> int:\n    return 41\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        handle = scope.spawn(worker())\n        result = await handle\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("handle.join().await"));
    assert!(result
        .rust_source
        .contains("let result: __SifrTaskResult<i64, std::convert::Infallible>"));
}

#[test]
fn test_task_handle_cancel_borrows_handle_and_aborts_child() {
    let source = concat!(
        "async def worker() -> int:\n    await task.sleep(10.0)\n    return 41\n\n",
        "async def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n",
        "        handle = scope.spawn(worker())\n        handle.",
        "cancel",
        "()\n        result = await handle\n    return None\n",
    );
    let result = generate_rust_with_metadata(
        &lower_module(parse_module(source).expect("parse failed").suite())
            .expect("lowering failed")
            .module,
    );

    assert!(result
        .rust_source
        .contains("abort_handle: tokio::task::AbortHandle"));
    assert!(result
        .rust_source
        .contains(&format!("fn {}{}", "can", "cel(&self)")));
    assert!(result
        .rust_source
        .contains(&format!("handle.{}{}", "can", "cel();")));
    assert!(result.rust_source.contains("struct CancellationError"));
    assert!(result.rust_source.contains("__SifrTaskResult::cancelled()"));
    assert!(result.rust_source.contains("handle.join().await"));
}

#[test]
fn test_task_timeout_handle_lowers_to_private_timeout_result() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def worker() -> int:\n    return 41\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        handle = scope.spawn(worker())\n        result = await task.timeout(handle, 1.0)\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("enum __SifrTimeoutResult<E>"));
    assert!(result.rust_source.contains("async fn __sifr_timeout"));
    assert!(result.rust_source.contains("biased;"));
    assert!(result.rust_source.contains("handle.__sifr_timeout"));
    assert!(result
        .rust_source
        .contains("failure.map_primary(__SifrTimeoutResult::Inner)"));
    assert!(result
        .rust_source
        .contains("__SifrFailure::new(__SifrTimeoutResult::Timeout)"));
    assert!(result.rust_source.contains(
        "let result: __SifrTaskResult<i64, __SifrTimeoutResult<std::convert::Infallible>>"
    ));
}

#[test]
fn test_failure_cancellation_error_annotation_lowers_to_private_evidence_type() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "def observe(failure: Failure[CancellationError]) -> None:\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("struct __SifrFailure<E>"));
    assert!(result.rust_source.contains("struct CancellationError"));
    assert!(result
        .rust_source
        .contains("fn observe(failure: &__SifrFailure<CancellationError>)"));
}

#[test]
fn test_failure_annotation_lowers_to_private_failure_type() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module("def observe(failure: Failure[ValueError]) -> None:\n    return None\n")
                .expect("parse failed")
                .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("struct __SifrFailure<E>"));
    assert!(result.rust_source.contains("primary: E"));
    assert!(result
        .rust_source
        .contains("secondary: Vec<SecondaryError>"));
    assert!(result
        .rust_source
        .contains("fn observe(failure: &__SifrFailure<ValueError>)"));
}

#[test]
fn test_task_timeout_context_manager_wraps_awaits() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def main() -> Result[None, TimeoutError]:\n    async with task.timeout(1.0):\n        await task.sleep(0.0)\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("match tokio::time::timeout"));
    assert!(result.rust_source.contains("return Err(TimeoutError::new"));
    assert!(result.rust_source.contains("struct TimeoutError"));
    assert!(result.required_crates.contains("tokio"));
}

#[test]
fn test_try_except_with_async_body_lowers_to_async_try_closure() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def main() -> Result[None, Error]:\n    try:\n        async with task.timeout(1.0):\n            await task.sleep(0.0)\n    except TimeoutError:\n        return None\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("async ||"));
    assert!(result.rust_source.contains(")().await"));
    assert!(result.rust_source.contains("let __sifr_try_res"));
}

#[test]
fn test_try_finally_runs_cleanup_before_timeout_propagates() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def main() -> Result[None, Error]:\n    try:\n        async with task.timeout(0.0):\n            try:\n                await task.sleep(10.0)\n            finally:\n                marker: int = 1\n    except TimeoutError:\n        return None\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    let cleanup_pos = result
        .rust_source
        .find("let marker: i64 = 1 as i64;")
        .expect("cleanup marker should be emitted");
    let rethrow_pos = result
        .rust_source
        .find("if let Err(__sifr_finally_err) = __sifr_try_finally_res")
        .expect("try/finally should rethrow after cleanup");

    assert!(result.rust_source.contains("let __sifr_try_finally_res"));
    assert!(cleanup_pos < rethrow_pos);
}

#[test]
fn test_try_finally_cleanup_try_except_lowers_question_mark_calls() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                r#"def fallible() -> Result[str, Error]:
    return "ok"

def main() -> Result[None, Error]:
    try:
        body_out: str = fallible()
    except Error as e:
        body_out = str(e.message)
    finally:
        try:
            cleanup_out: str = fallible()
        except Error as e:
            cleanup_out = str(e.message)
    return None
"#,
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(!result.rust_source.contains("compile_error!"));
    assert!(result.rust_source.contains("let __sifr_try_finally_res"));
    assert!(result.rust_source.contains("let __sifr_try_res"));
    assert!(result.rust_source.contains("fallible()?"));
}

#[test]
fn test_async_generated_errors_convert_to_error_return_type() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def worker() -> int:\n    return 41\n\nasync def main() -> Result[None, Error]:\n    async with task.timeout(1.0):\n        await task.sleep(0.0)\n    async with task.scope() as scope:\n        handle = scope.spawn(worker())\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result
        .rust_source
        .contains("impl From<TimeoutError> for Error"));
    assert!(result
        .rust_source
        .contains("impl From<ScopeFailure> for Error"));
    assert!(result
        .rust_source
        .contains("return Err(TimeoutError::new(\"task timeout expired\".to_string()).into())"));
    assert!(result
        .rust_source
        .contains("return Err(__sifr_scope_failure.into());"));
}

#[test]
fn test_sync_main_does_not_require_tokio() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module("def main() -> None:\n    return None\n")
                .expect("parse failed")
                .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(!result
        .rust_source
        .contains("#[tokio::main(flavor = \"current_thread\")]"));
    assert!(!result.required_crates.contains("tokio"));
}

#[test]
fn test_generate_project_emits_tokio_dependency_when_required() {
    let module = empty_module();
    let required_crates = HashSet::from(["tokio".to_string()]);
    let (cargo_toml, _main_rs) = generate_project_with_deps_and_crates(
        &module,
        "sifr_output",
        &HashSet::new(),
        &required_crates,
    );

    assert!(cargo_toml.contains(
        "tokio = { version = \"1.52.3\", features = [\"macros\", \"rt\", \"sync\", \"time\"] }"
    ));
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

#[test]
fn test_variadic_min_max_lower_to_nested_calls() {
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
                            args: vec![
                                HirExpr::IntLiteral(3),
                                HirExpr::IntLiteral(1),
                                HirExpr::IntLiteral(2),
                            ],
                            ty: Type::Int,
                        }],
                        ty: Type::None,
                    },
                },
                HirStmt::Expr {
                    expr: HirExpr::Call {
                        func: "print".to_string(),
                        args: vec![HirExpr::Call {
                            func: "max".to_string(),
                            args: vec![
                                HirExpr::IntLiteral(1),
                                HirExpr::IntLiteral(5),
                                HirExpr::IntLiteral(2),
                                HirExpr::IntLiteral(4),
                            ],
                            ty: Type::Int,
                        }],
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
        rust_code.matches("std::cmp::min").count() >= 2,
        "variadic min should lower to nested std::cmp::min calls: {rust_code}"
    );
    assert!(
        rust_code.matches("std::cmp::max").count() >= 3,
        "variadic max should lower to nested std::cmp::max calls: {rust_code}"
    );
}
