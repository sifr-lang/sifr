use crate::{lower_module, HirExpr, HirModule, HirStmt, LoweringError};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_parser::parse_module;
use sifr_type_system::{ParamConvention, Type};

fn lower_source(source: &str) -> Result<HirModule, Vec<LoweringError>> {
    let parsed = parse_module(source).expect("parse failed");
    lower_module(parsed.suite()).map(|result| result.module)
}

#[test]
fn test_nested_function_is_predeclared_as_typed_callable_for_forward_local_use() {
    let module = lower_source(
        "def apply(f: Callable[[int], int], value: int) -> int:\n    return f(value)\n\ndef outer(x: int) -> int:\n    result = apply(helper, x)\n    def helper(y: int) -> int:\n        return y + 1\n    return result\n",
    )
    .expect("nested helper should be predeclared before lowering the body");

    let outer = module
        .functions
        .iter()
        .find(|function| function.name == "outer")
        .expect("outer function missing");
    let HirStmt::Let { value, .. } = &outer.body[0] else {
        panic!("expected first outer statement to be a let binding");
    };
    let HirExpr::Call { args, .. } = value else {
        panic!("expected first outer statement to lower as a call");
    };
    let HirExpr::Name { name, ty } = &args[0] else {
        panic!("expected helper to lower as a local callable binding");
    };

    assert_eq!(name, "helper");
    assert_eq!(
        ty,
        &Type::Callable(
            vec![Type::Int],
            vec![ParamConvention::own()],
            Box::new(Type::Int),
        )
    );
}

#[test]
fn test_forward_direct_call_to_nested_function_type_checks() {
    let result = lower_source(
        "def outer(x: int) -> int:\n    result = helper(x)\n    def helper(y: int) -> int:\n        return y + 1\n    return result\n",
    );
    assert!(
        result.is_ok(),
        "forward direct calls should see the nested helper symbol during lowering"
    );
}

#[test]
fn test_missing_forward_local_helper_still_errors_explicitly() {
    let result = lower_source(
        "def apply(f: Callable[[int], int], value: int) -> int:\n    return f(value)\n\ndef outer(x: int) -> int:\n    return apply(missing, x)\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|error| error.message == "undefined variable: 'missing'"));
}

#[test]
fn test_recursive_nested_helper_infers_int_signature_from_usage() {
    let result = lower_source(
        "def power_two(exp: int) -> int:\n    def helper(n):\n        if n == 0:\n            return 1\n        return 2 * helper(n - 1)\n\n    return helper(exp)\n",
    );
    assert!(
        result.is_ok(),
        "recursive local helpers should infer integer parameter and return types from supported usage"
    );
}

#[test]
fn test_recursive_nested_helper_infers_mutable_collection_param_from_usage() {
    let module = lower_source(
        "def combination_sum(candidates: list[int], target: int) -> list[list[int]]:\n    res = []\n\n    def dfs(i, cur, total):\n        if total == target:\n            res.append(cur.copy())\n            return\n        if i >= len(candidates) or total > target:\n            return\n        cur.append(candidates[i])\n        dfs(i, cur, total + candidates[i])\n        cur.pop()\n        dfs(i + 1, cur, total)\n\n    dfs(0, [], 0)\n    return res\n",
    )
    .expect("recursive local helpers should infer mutable collection params from usage");

    let combination_sum = module
        .functions
        .iter()
        .find(|function| function.name == "combination_sum")
        .expect("combination_sum function missing");
    let HirStmt::NestedFunction { func } = &combination_sum.body[1] else {
        panic!("expected nested dfs helper");
    };

    assert_eq!(func.params[1].name, "cur");
    assert_eq!(func.params[1].ty, Type::List(Box::new(Type::Int)));
    assert_eq!(func.params[1].convention, ParamConvention::mut_borrow());
}

#[test]
fn test_conflicting_nested_helper_call_sites_fail_inference_explicitly() {
    let result = lower_source(
        "def outer(flag: bool) -> None:\n    def helper(value):\n        print(value)\n\n    if flag:\n        helper(1)\n    else:\n        helper(\"x\")\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "argument 1 of callable 'helper': expected 'str', got 'int'"
    }));
}

#[test]
fn test_nonlocal_nested_helper_rebinds_enclosing_name() {
    let result = lower_source(
        "def outer(values: list[int]) -> int:\n    total = 0\n    def apply() -> None:\n        nonlocal total\n        for value in values:\n            total += value\n    apply()\n    return total\n",
    );
    assert!(
        result.is_ok(),
        "nonlocal rebinding in a non-recursive nested helper should lower cleanly"
    );
}

#[test]
fn test_nonlocal_tuple_unpack_fails_explicitly() {
    let result = lower_source(
        "def outer() -> int:\n    left, right = 0, 1\n    def update() -> None:\n        nonlocal left, right\n        left, right = right, left + right\n    update()\n    return left + right\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "tuple unpacking cannot rebind captured state with `nonlocal` yet"
            && error.code == Some(DiagnosticCode::FLOW_INVALID_NONLOCAL)
    }));
}

#[test]
fn test_augassign_to_capture_requires_nonlocal() {
    let result = lower_source(
        "def outer() -> int:\n    total = 0\n    def apply() -> None:\n        total += 1\n    apply()\n    return total\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message
            == "captured variable `total` must be declared with `nonlocal` before augmented assignment"
            && error.code == Some(DiagnosticCode::FLOW_INVALID_NONLOCAL)
    }));
}

#[test]
fn test_recursive_nonlocal_nested_helper_fails_explicitly() {
    let result = lower_source(
        "def outer(limit: int) -> int:\n    total = 0\n    def visit(i: int) -> None:\n        nonlocal total\n        if i == limit:\n            total += 1\n            return\n        visit(i + 1)\n    visit(0)\n    return total\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message
            == "recursive nested function 'visit' cannot mutate captured state with `nonlocal` yet"
            && error.code == Some(DiagnosticCode::FLOW_INVALID_NONLOCAL)
    }));
}

#[test]
fn test_top_level_nonlocal_requires_enclosing_binding_code() {
    let result = lower_source("def main() -> None:\n    nonlocal total\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "nonlocal declaration requires an enclosing function binding"
            && error.code == Some(DiagnosticCode::FLOW_INVALID_NONLOCAL)
    }));
}

#[test]
fn test_unresolved_nonlocal_has_flow_code() {
    let result = lower_source(
        "def outer() -> int:\n    def inner() -> None:\n        nonlocal missing\n    inner()\n    return 0\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "nonlocal name 'missing' does not resolve to an enclosing function binding"
            && error.code == Some(DiagnosticCode::FLOW_INVALID_NONLOCAL)
    }));
}

#[test]
fn test_nonlocal_current_binding_conflict_has_flow_code() {
    let result = lower_source(
        "def outer(value: int) -> int:\n    def inner(value: int) -> None:\n        nonlocal value\n    inner(1)\n    return value\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message
            == "nonlocal name 'value' conflicts with a binding in the current function scope"
            && error.code == Some(DiagnosticCode::FLOW_INVALID_NONLOCAL)
    }));
}

#[test]
fn test_nested_helper_usage_refines_outer_empty_collection_types() {
    let module = lower_source(
        "def subsets(limit: int) -> list[list[int]]:\n    res = []\n    subset = []\n\n    def dfs(i: int):\n        if i >= limit:\n            res.append(subset.copy())\n            return\n        subset.append(i)\n        dfs(i + 1)\n        subset.pop()\n        dfs(i + 1)\n\n    dfs(0)\n    return res\n",
    )
    .expect("nested helper capture usage should refine outer empty collection types");

    let subsets = module
        .functions
        .iter()
        .find(|function| function.name == "subsets")
        .expect("subsets function missing");

    let HirStmt::Let { ty: res_ty, .. } = &subsets.body[0] else {
        panic!("expected first subsets statement to be the result binding");
    };
    let HirStmt::Let { ty: subset_ty, .. } = &subsets.body[1] else {
        panic!("expected second subsets statement to be the subset binding");
    };

    assert_eq!(
        res_ty,
        &Type::List(Box::new(Type::List(Box::new(Type::Int))))
    );
    assert_eq!(subset_ty, &Type::List(Box::new(Type::Int)));
}

#[test]
fn test_recursive_memoized_nested_helper_infers_deterministic_int_return() {
    let result = lower_source(
        "def max_profit(prices: list[int]) -> int:\n    dp = {}\n\n    def dfs(i, buying):\n        if i >= len(prices):\n            return 0\n        if (i, buying) in dp:\n            return dp[(i, buying)]\n\n        cooldown = dfs(i + 1, buying)\n        if buying:\n            buy = dfs(i + 1, not buying) - prices[i]\n            dp[(i, buying)] = max(buy, cooldown)\n        else:\n            sell = dfs(i + 2, not buying) + prices[i]\n            dp[(i, buying)] = max(sell, cooldown)\n        return dp[(i, buying)]\n\n    return dfs(0, True)\n",
    );
    assert!(
        result.is_ok(),
        "recursive memoized helpers should infer stable int returns without Unknown/None leakage"
    );
}

#[test]
fn test_tuple_for_target_inference_specializes_empty_dict_for_membership_index_pattern() {
    let result = lower_source(
        "def two_sum(nums: list[int], target: int) -> list[int]:\n    prev_map = {}\n    for i, n in enumerate(nums):\n        diff = target - n\n        if diff in prev_map:\n            return [prev_map[diff], i]\n        prev_map[n] = i\n    fallback: list[int] = []\n    return fallback\n",
    );
    assert!(
        result.is_ok(),
        "tuple-target for-loop inference should specialize dict key/value types early enough for guarded indexed reads: {:?}",
        result.err()
    );
}
