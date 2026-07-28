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
                        func: "inner".to_string(),
                        args: vec![],
                        ty: Type::Int,
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
    result: Result[Subscription, SubscriptionError | RustPanicError] = subscribe(handler)
    print(prefix)
    return result
"#,
    );

    assert!(
        generated.contains("let prefix = prefix.clone();")
            && generated.contains("move |event: &String|"),
        "{generated}"
    );
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
            && generated.contains("move |event: &String|"),
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
