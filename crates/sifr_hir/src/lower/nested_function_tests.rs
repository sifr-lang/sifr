use crate::{lower_module, HirDiagnostic, HirExpr, HirModule, HirStmt};
use ruff_text_size::{TextRange, TextSize};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_parser::parse_module;
use sifr_type_system::{ParamConvention, Type};

fn lower_source(source: &str) -> Result<HirModule, Vec<HirDiagnostic>> {
    let parsed = parse_module(source).expect("parse failed");
    lower_module(parsed.suite()).map(|result| result.module)
}

fn range_for_after(source: &str, after: &str, needle: &str) -> TextRange {
    let after_start = source.find(after).expect("anchor should exist");
    let relative_start = source[after_start..]
        .find(needle)
        .expect("needle should exist after anchor");
    let start = (after_start + relative_start) as u32;
    TextRange::new(
        TextSize::new(start),
        TextSize::new(start + needle.len() as u32),
    )
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
        "def collect_budget_routes(weights: list[int], budget: int) -> list[list[int]]:\n    routes = []\n\n    def visit(index, current, total):\n        if total == budget:\n            routes.append(current.copy())\n            return\n        if index >= len(weights) or total > budget:\n            return\n        current.append(weights[index])\n        visit(index, current, total + weights[index])\n        current.pop()\n        visit(index + 1, current, total)\n\n    visit(0, [], 0)\n    return routes\n",
    )
    .expect("recursive local helpers should infer mutable collection params from usage");

    let collect_budget_routes = module
        .functions
        .iter()
        .find(|function| function.name == "collect_budget_routes")
        .expect("collect_budget_routes function missing");
    let HirStmt::NestedFunction { func } = &collect_budget_routes.body[1] else {
        panic!("expected nested visit helper");
    };

    assert_eq!(func.params[1].name, "current");
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
    let source = "def outer() -> int:\n    left, right = 0, 1\n    def update() -> None:\n        nonlocal left, right\n        left, right = right, left + right\n    update()\n    return left + right\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "tuple unpacking cannot rebind captured state with `nonlocal` yet"
            && error.code == Some(DiagnosticCode::FLOW_INVALID_NONLOCAL)
            && error.primary_range
                == Some(range_for_after(
                    source,
                    "        left, right",
                    "left, right",
                ))
    }));
}

#[test]
fn test_augassign_to_capture_requires_nonlocal() {
    let source =
        "def outer() -> int:\n    total = 0\n    def apply() -> None:\n        total += 1\n    apply()\n    return total\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message
            == "captured variable `total` must be declared with `nonlocal` before augmented assignment"
            && error.code == Some(DiagnosticCode::FLOW_INVALID_NONLOCAL)
            && error.primary_range == Some(range_for_after(source, "        total += ", "total"))
    }));
}

#[test]
fn test_recursive_nonlocal_nested_helper_fails_explicitly() {
    let source = "def outer(limit: int) -> int:\n    total = 0\n    def visit(i: int) -> None:\n        nonlocal total\n        if i == limit:\n            total += 1\n            return\n        visit(i + 1)\n    visit(0)\n    return total\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message
            == "recursive nested function 'visit' cannot mutate captured state with `nonlocal` yet"
            && error.code == Some(DiagnosticCode::FLOW_INVALID_NONLOCAL)
            && error.primary_range == Some(range_for_after(source, "def visit", "visit"))
    }));
}

#[test]
fn test_top_level_nonlocal_requires_enclosing_binding_code() {
    let source = "def main() -> None:\n    nonlocal total\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "nonlocal declaration requires an enclosing function binding"
            && error.code == Some(DiagnosticCode::FLOW_INVALID_NONLOCAL)
            && error.primary_range
                == Some(range_for_after(source, "    nonlocal", "nonlocal total"))
    }));
}

#[test]
fn test_unresolved_nonlocal_has_flow_code() {
    let source =
        "def outer() -> int:\n    def inner() -> None:\n        nonlocal missing\n    inner()\n    return 0\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "nonlocal name 'missing' does not resolve to an enclosing function binding"
            && error.code == Some(DiagnosticCode::FLOW_INVALID_NONLOCAL)
            && error.primary_range == Some(range_for_after(source, "nonlocal ", "missing"))
    }));
}

#[test]
fn test_nonlocal_current_binding_conflict_has_flow_code() {
    let source =
        "def outer(value: int) -> int:\n    def inner(value: int) -> None:\n        nonlocal value\n    inner(1)\n    return value\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message
            == "nonlocal name 'value' conflicts with a binding in the current function scope"
            && error.code == Some(DiagnosticCode::FLOW_INVALID_NONLOCAL)
            && error.primary_range == Some(range_for_after(source, "nonlocal ", "value"))
    }));
}

#[test]
fn test_nested_helper_usage_refines_outer_empty_collection_types() {
    let module = lower_source(
        "def collect_route_prefixes(limit: int) -> list[list[int]]:\n    routes = []\n    prefix = []\n\n    def visit(depth: int):\n        routes.append(prefix.copy())\n        if depth >= limit:\n            return\n        prefix.append(depth)\n        visit(depth + 1)\n        prefix.pop()\n\n    visit(0)\n    return routes\n",
    )
    .expect("nested helper capture usage should refine outer empty collection types");

    let collect_route_prefixes = module
        .functions
        .iter()
        .find(|function| function.name == "collect_route_prefixes")
        .expect("collect_route_prefixes function missing");

    let HirStmt::Let { ty: routes_ty, .. } = &collect_route_prefixes.body[0] else {
        panic!("expected first collect_route_prefixes statement to be the routes binding");
    };
    let HirStmt::Let { ty: prefix_ty, .. } = &collect_route_prefixes.body[1] else {
        panic!("expected second collect_route_prefixes statement to be the prefix binding");
    };

    assert_eq!(
        routes_ty,
        &Type::List(Box::new(Type::List(Box::new(Type::Int))))
    );
    assert_eq!(prefix_ty, &Type::List(Box::new(Type::Int)));
}

#[test]
fn test_nested_missing_parameter_annotation_has_primary_range() {
    let source = "def outer() -> int:\n    def helper(value):\n        return 1\n    return 0\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message
            == "parameter 'value' in function 'helper' is missing a type annotation and could not be inferred"
            && error.code == Some(DiagnosticCode::TYPE_MISSING_ANNOTATION)
            && error.primary_range == Some(range_for_after(source, "helper(", "value"))
    }));
}

#[test]
fn test_nested_ambiguous_return_inference_has_code_and_primary_range() {
    let source = "def outer(flag: bool) -> None:\n    def helper():\n        if flag:\n            return 1\n        return \"x\"\n    helper()\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "function 'helper' return type could not be inferred deterministically"
            && error.code == Some(DiagnosticCode::TYPE_MISSING_ANNOTATION)
            && error.primary_range == Some(range_for_after(source, "def ", "helper"))
    }));
}

#[test]
fn test_recursive_memoized_nested_helper_infers_deterministic_int_return() {
    let result = lower_source(
        "def schedule_score(weights: list[int]) -> int:\n    memo = {}\n\n    def score(index, enabled):\n        if index >= len(weights):\n            return 0\n        if (index, enabled) in memo:\n            return memo[(index, enabled)]\n\n        skipped = score(index + 1, enabled)\n        if enabled:\n            accepted = score(index + 1, not enabled) + weights[index]\n            memo[(index, enabled)] = max(accepted, skipped)\n        else:\n            delayed = score(index + 2, not enabled) + weights[index]\n            memo[(index, enabled)] = max(delayed, skipped)\n        return memo[(index, enabled)]\n\n    return score(0, True)\n",
    );
    assert!(
        result.is_ok(),
        "recursive memoized helpers should infer stable int returns without Unknown/None leakage"
    );
}

#[test]
fn test_tuple_for_target_inference_specializes_empty_dict_for_membership_index_pattern() {
    let result = lower_source(
        "def first_repeated_bucket(events: list[int], seed: int) -> list[int]:\n    first_seen = {}\n    for index, event in enumerate(events):\n        bucket = event + seed\n        if bucket in first_seen:\n            return [first_seen[bucket], index]\n        first_seen[bucket] = index\n    fallback: list[int] = []\n    return fallback\n",
    );
    assert!(
        result.is_ok(),
        "tuple-target for-loop inference should specialize dict key/value types early enough for guarded indexed reads: {:?}",
        result.err()
    );
}
