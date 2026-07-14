use crate::generate_rust;
use sifr_ir::{HirExpr, HirFStringPart, HirFunction, HirModule, HirParam, HirStmt, MethodKind};
use sifr_lowering::{lower_module, lower_module_with_externals, ExternalDefs};
use sifr_python_parser::parse_module;
use sifr_type_system::{ParamConvention, Type};

pub(crate) fn generate_rust_from_source(source: &str) -> String {
    let parsed = parse_module(source).expect("parse failed");
    let lowering = lower_module(parsed.suite()).expect("lowering failed");
    generate_rust(&lowering.module)
}

pub(crate) fn generate_rust_from_source_with_stdlib_collections(source: &str) -> String {
    let parsed = parse_module(source).expect("parse failed");
    let mut externals = ExternalDefs::default();
    externals
        .functions
        .entry("sifr.collections".to_string())
        .or_default();
    let lowering =
        lower_module_with_externals(parsed.suite(), &externals).expect("lowering failed");
    generate_rust(&lowering.module)
}

#[test]
fn async_callable_parameter_emits_send_future_contract() {
    let rust_code = generate_rust_from_source(
        "async def invoke(handler: AsyncCallable[[int], int]) -> int:\n    return await handler(4)\n",
    );

    assert!(rust_code.contains(
        "Fn(i64) -> std::pin::Pin<Box<dyn std::future::Future<Output = i64> + Send>> + Send + Sync"
    ));
    assert!(rust_code.contains("handler(4_i64).await"));
    syn::parse_file(&rust_code).expect("AsyncCallable Rust should parse");
}

#[test]
fn async_function_argument_is_adapted_to_send_future_callable() {
    let rust_code = generate_rust_from_source(
        "async def plus_one(value: int) -> int:\n    await task.sleep(0.0)\n    return value + 1\n\nasync def invoke(handler: AsyncCallable[[int], int]) -> int:\n    return await handler(4)\n\nasync def main() -> None:\n    value = await invoke(plus_one)\n    print(value)\n",
    );

    assert!(
        rust_code.contains("__sifr_send_async_callable"),
        "{rust_code}"
    );
}

#[test]
fn nested_async_function_argument_is_adapted_to_send_future_callable() {
    let rust_code = generate_rust_from_source(
        "async def invoke(handler: AsyncCallable[[str], str]) -> str:\n    return await handler(\"value\")\n\nasync def main() -> None:\n    prefix: str = \"nested:\"\n    async def nested(own value: str) -> str:\n        await task.sleep(0.0)\n        return prefix + value\n    result = await invoke(nested)\n    print(result)\n",
    );

    assert!(
        rust_code.contains("let __sifr_send_async_callable = std::sync::Arc::new(nested)"),
        "{rust_code}"
    );
    assert!(
        rust_code.contains("let nested = async move |value: String|"),
        "{rust_code}"
    );
    assert!(!rust_code.contains("invoke(nested).await"), "{rust_code}");
}

#[test]
fn async_callable_class_field_uses_a_boxed_future_adapter() {
    let rust_code = generate_rust_from_source(
        "class AsyncRunner:\n    handler: AsyncCallable[[int], int]\n\n    def __init__(self, handler: AsyncCallable[[int], int]):\n        self.handler = handler\n\n    async def run(self, value: int) -> int:\n        return await self.handler(value)\n\nasync def plus_one(value: int) -> int:\n    await task.sleep(0.0)\n    return value + 1\n\nasync def main() -> None:\n    runner = AsyncRunner(plus_one)\n    print(await runner.run(41))\n",
    );

    assert!(rust_code.contains("Box<dyn Fn(i64) -> std::pin::Pin<Box<dyn std::future::Future<Output = i64> + Send>> + Send + Sync>"));
    assert!(rust_code.contains("move |__sifr_async_arg_0|"));
    assert!(rust_code.contains("std::sync::Arc::clone(&__sifr_async_callable)"));
    assert!(rust_code.contains("__sifr_async_callable(__sifr_async_arg_0).await"));
    assert!(rust_code.contains("(self.handler)(value).await"));
    syn::parse_file(&rust_code).expect("stored AsyncCallable Rust should parse");
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

pub(crate) fn empty_module() -> HirModule {
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
        r#"def main():
    def recurse(n: int) -> int:
        for i in [1]:
            pass
        else:
            if n > 0:
                return recurse(n - 1)
        return 0

    print(recurse(4))
"#,
    );
    assert!(loop_else_recursion.contains("fn recurse(n: i64) -> i64"));
    assert!(
        loop_else_recursion.contains("if !_broke") || loop_else_recursion.contains("if !(_broke)")
    );

    let try_except_return = generate_rust_from_source(
        r#"def classify(n: int) -> int:
    try:
        if n > 0:
            return n
        else:
            raise ValueError('non-positive')
    except ValueError as e:
        return 99
"#,
    );
    assert!(
        try_except_return.contains("return Err(ValueError::new(\"non-positive\".to_string()));")
    );
    assert!(try_except_return.contains("return 99_i64;"));

    let loop_guard_narrowing = generate_rust_from_source(
        r#"def summarize(values: list[int]) -> int:
    total: int = 0
    for value in values:
        if value > 10:
            total = total + value
        else:
            total = total + 1
    return total
"#,
    );
    assert!(loop_guard_narrowing.contains("fn summarize(values: &Vec<i64>) -> i64"));
    assert!(loop_guard_narrowing.contains("if value > (10_i64)"));
}

#[test]
fn test_generate_rust_elides_unreachable_returns_after_always_exit_paths() {
    let always_exit_try = generate_rust_from_source(
        r#"def classify(flag: bool) -> int:
    try:
        if flag:
            return 5
        raise ValueError('bad value')
        return 11
    except ValueError as e:
        return 77
"#,
    );
    assert!(always_exit_try.contains("return Err(ValueError::new(\"bad value\".to_string()));"));
    assert!(always_exit_try.contains("return 77_i64;"));
    assert!(!always_exit_try.contains("11_i64"));

    let unreachable_tail = generate_rust_from_source(
        r#"def inferred(flag: bool):
    if flag:
        return 1
    return 2
    return 'never'
"#,
    );
    assert!(unreachable_tail.contains("fn inferred(flag: bool) -> i64"));
    assert!(unreachable_tail.contains("2_i64\n}"));
    assert!(!unreachable_tail.contains("return 2_i64"));
    assert!(!unreachable_tail.contains("never"));
}

#[test]
fn test_direct_try_capture_converts_result_none_to_unit() {
    let rust_code = generate_rust_from_source(
        r#"async def finish() -> Result[None, ValueError]:
    try:
        await task.sleep(0.0)
        return None
    except ValueError as error:
        raise error
"#,
    );

    assert!(rust_code.contains("return Ok(Ok(()));"), "{rust_code}");
    assert!(!rust_code.contains("return Ok(Ok(None));"), "{rust_code}");
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
    assert!(rust_code.contains("fn add(a: i64, b: i64) -> i64"));
    assert!(rust_code.contains("a + b"));
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
        rust_code.contains("let mut x: i64"),
        "should emit `let mut x` for reassigned var"
    );
}

#[test]
fn test_generate_rust_recursive_tree_traversal_uses_option_let_else_and_cloned_box_reads() {
    let rust_code = generate_rust_from_source(
        "class TreeNode:\n    val: int\n    left: TreeNode | None\n    right: TreeNode | None\n\n    def __init__(self, val: int, left: TreeNode | None, right: TreeNode | None):\n        self.val = val\n        self.left = left\n        self.right = right\n\ndef tree_value_sum(node: TreeNode | None) -> int:\n    if not node:\n        return 0\n    left: TreeNode | None = node.left\n    right: TreeNode | None = node.right\n    return node.val + tree_value_sum(left) + tree_value_sum(right)\n\ndef paired_tree_value_sum(p: TreeNode | None, q: TreeNode | None) -> int:\n    if not p and not q:\n        return 0\n    if not p or not q:\n        return -1\n    return p.val + q.val + paired_tree_value_sum(p.left, q.left) + paired_tree_value_sum(p.right, q.right)\n",
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
