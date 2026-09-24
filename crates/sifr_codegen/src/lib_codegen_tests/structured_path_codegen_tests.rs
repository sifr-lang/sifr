use super::*;

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
        receiver: None,
        decorators: vec![],
        rust_interop: Vec::new(),
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: vec![],
    };

    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![
                HirStmt::NestedFunction {
                    func: nested,
                    move_captures: false,
                    capture_clones: Vec::new(),
                },
                HirStmt::Expr {
                    expr: HirExpr::Call {
                        mutable_arg_places: Vec::new(),
                        func: "inner".to_string(),
                        args: vec![],
                        ty: Type::Int,
                    },
                },
            ],
            is_async: false,
            method_kind: MethodKind::Regular,
            receiver: None,
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

    let generated = generate_rust_with_metadata(&module);
    assert!(generated.rust_source.contains("let inner = || {"));
    assert!(generated.rust_source.contains("inner()"));
}

#[test]
fn retained_rust_callback_nested_handler_owns_captures() {
    let generated = generate_rust_from_source(
        r#"
class SubscriptionError(Error):
    message: str

class Subscription:
    lifecycle_token: int

@rust.callback(backpressure=bounded(2), overflow=error, shutdown=drain)
@rust(bridge.events.subscribe, panic=map_error(bridge.events.map_panic))
def subscribe(own handler: Callable[[str], Result[None, SubscriptionError]]) -> Result[Subscription, SubscriptionError | RustPanicError]: ...

def run() -> Result[Subscription, SubscriptionError | RustPanicError]:
    prefix: str = "event"
    def handler(event: str) -> Result[None, SubscriptionError]:
        _ = prefix
        return None
    prefix = "outer-after"
    result: Result[Subscription, SubscriptionError | RustPanicError] = subscribe(handler)
    print(prefix)
    return result
"#,
    );

    assert!(
        generated.contains("let prefix = prefix.clone();")
            && generated.contains("move |event: &str|")
            && generated.contains("prefix = \"outer-after\".to_string();"),
        "{generated}"
    );
    let snapshot = generated
        .find("let prefix = prefix.clone();")
        .expect("capture snapshot");
    let rebind = generated
        .find("prefix = \"outer-after\".to_string();")
        .expect("outer rebind");
    let attachment = generated.find("subscribe(handler)").expect("attachment");
    assert!(snapshot < rebind && rebind < attachment, "{generated}");
}

#[test]
fn retained_rust_callback_nested_handler_owns_loop_capture() {
    let generated = generate_rust_from_source(
        r#"
class SubscriptionError(Error):
    message: str

class Subscription:
    lifecycle_token: int

@rust.callback(backpressure=bounded(2), overflow=error, shutdown=drain)
@rust(bridge.events.subscribe, panic=map_error(bridge.events.map_panic))
def subscribe(own handler: Callable[[str], Result[None, SubscriptionError]]) -> Result[Subscription, SubscriptionError | RustPanicError]: ...

def run() -> Result[Subscription, SubscriptionError | RustPanicError]:
    labels: list[str] = ["first", "second"]
    for label in labels:
        def handler(event: str) -> Result[None, SubscriptionError]:
            _ = label
            return None
        print(label)
        return subscribe(handler)
    raise SubscriptionError("missing label")
"#,
    );

    assert!(
        generated.contains("for label in labels.iter().cloned()")
            && generated.contains("let label = label.clone();")
            && generated.contains("move |event: &str|"),
        "{generated}"
    );
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
        "fn dfs(i: SifrInt, res: &mut Vec<Vec<SifrInt>>, subset: &mut Vec<SifrInt>, values: &[SifrInt])"
    ));
    assert!(generated.contains("dfs(SifrInt::from_i64(0), &mut res, &mut subset, &values);"));
    assert!(
        generated
            .contains("dfs(::std::ops::Add::add(&i, &SifrInt::from_i64(1)), res, subset, values);")
    );
}

#[test]
fn recursive_nested_checked_reads_use_the_injected_capture_binding() {
    let source = include_str!(
        "../../../../verification/areas/algorithmic_compatibility/corpora/leetcode/src/1905_count_sub_islands.sifr"
    );
    let generated = crate::canonicalize_generated_rust_source(&generate_rust_from_source(source))
        .expect("1905 emitted Rust must canonicalize");
    let nested = generated
        .split("fn dfs(")
        .nth(1)
        .expect("1905 dfs must be emitted as a nested function");
    let nested = nested
        .split("let (rows_value_")
        .next()
        .expect("outer bindings must follow dfs");
    let capture_start = nested
        .find("grid2_argument_")
        .expect("dfs must receive the explicit grid2 capture");
    let capture = nested[capture_start..]
        .split(": &[Vec<SifrInt>]")
        .next()
        .expect("dfs capture must be a borrowed nested list");
    assert!(
        nested.contains(&format!("{capture}: &[Vec<SifrInt>]")),
        "{generated}"
    );
    assert!(nested.contains(&format!("&{capture};")), "{generated}");
    assert_eq!(
        nested.matches(&format!("{capture},")).count(),
        4,
        "{generated}"
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
                    mutable_arg_places: Vec::new(),
                    func: "print".to_string(),
                    args: vec![HirExpr::StringLiteral("marker".to_string())],
                    ty: Type::None,
                },
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            receiver: None,
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

    let generated = generate_rust_with_metadata(&module);
    assert!(generated.rust_source.contains("println!"));
    assert!(generated.rust_source.contains("marker"));
}
