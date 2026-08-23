use crate::{HirDiagnostic, HirExpr, HirModule, HirStmt, lower_module};
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
    let HirExpr::Name { name, ty, .. } = &args[0] else {
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
    assert!(
        errors
            .iter()
            .any(|error| error.message == "undefined variable: 'missing'")
    );
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
        "def accumulate_items(items: list[int], limit: int) -> list[list[int]]:\n    snapshots = []\n\n    def visit(index, current, total):\n        if total == limit:\n            snapshots.append(current.copy())\n            return\n        if index >= len(items) or total > limit:\n            return\n        current.append(items[index])\n        visit(index, current, total + items[index])\n        current.pop()\n        visit(index + 1, current, total)\n\n    visit(0, [], 0)\n    return snapshots\n",
    )
    .expect("recursive local helpers should infer mutable collection params from usage");

    let accumulate_items = module
        .functions
        .iter()
        .find(|function| function.name == "accumulate_items")
        .expect("accumulate_items function missing");
    let HirStmt::NestedFunction { func, .. } = &accumulate_items.body[1] else {
        panic!("expected nested visit helper");
    };

    assert_eq!(func.params[1].name, "current");
    assert_eq!(func.params[1].ty, Type::List(Box::new(Type::Int)));
    assert_eq!(func.params[1].convention, ParamConvention::mut_borrow());
}

#[test]
fn same_named_nested_helpers_keep_lexical_mutable_call_metadata() {
    let result = lower_source(
        "class Codec:\n    def serialize(self, root: int | None) -> int:\n        def dfs(node: int | None) -> int:\n            if node is None:\n                return 0\n            return dfs(None)\n        return dfs(root)\n\n    def deserialize(self, data: str) -> int:\n        values: list[str] = data.split(\",\")\n        def dfs(values: list[str]) -> int:\n            if len(values) == 0:\n                return 0\n            values.pop(0)\n            return dfs(values)\n        return dfs(values)\n",
    );
    assert!(
        result.is_ok(),
        "plain-call verification must use the lexically proven call metadata instead of a module-wide same-name signature: {result:?}"
    );
}

#[test]
fn shadowed_nested_helper_restores_defaults_keywords_and_varargs() {
    let result = lower_source(
        "def outer(flag: bool) -> int:\n    def helper(value: int = 40, *extra: int) -> int:\n        return value + len(extra)\n\n    if flag:\n        def helper(prefix: str, suffix: str = \"!\") -> str:\n            return prefix + suffix\n        assert helper(\"ok\") == \"ok!\"\n\n    return helper() + helper(value=1) + helper(1, 2, 3)\n",
    );
    assert!(
        result.is_ok(),
        "calls after a shadowing block must use the restored outer signature: {result:?}"
    );
}

#[test]
fn shadowed_nested_helper_does_not_inherit_outer_default_or_vararg() {
    let errors = lower_source(
        "def outer(flag: bool) -> int:\n    def helper(value: int = 1, *extra: int) -> int:\n        return value + len(extra)\n\n    if flag:\n        def helper(value: int) -> int:\n            return value\n        helper()\n        helper(1, 2)\n\n    return 0\n",
    )
    .expect_err("the inner helper must retain its required fixed-arity signature");

    assert!(
        errors
            .iter()
            .any(|error| error.message == "helper() missing required argument 'value'")
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message == "helper() takes at most 1 argument(s), got 2")
    );
}

#[test]
fn inferred_nested_vararg_keeps_its_list_shape() {
    let result = lower_source(
        "def outer() -> int:\n    def helper(*extra):\n        return len(extra)\n\n    return helper(1, 2)\n",
    );
    assert!(
        result.is_ok(),
        "vararg call sites must infer the packed list element type: {result:?}"
    );
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
    let source = "def outer() -> int:\n    total = 0\n    def apply() -> None:\n        total += 1\n    apply()\n    return total\n";
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
    let source = "def outer() -> int:\n    def inner() -> None:\n        nonlocal missing\n    inner()\n    return 0\n";
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
    let source = "def outer(value: int) -> int:\n    def inner(value: int) -> None:\n        nonlocal value\n    inner(1)\n    return value\n";
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
fn top_level_union_return_validation_is_declaration_order_neutral() {
    let forward = "def caller() -> int:\n    return later(False)\n\ndef later(flag: bool):\n    if flag:\n        return \"x\"\n    return 1\n";
    let reverse = "def later(flag: bool):\n    if flag:\n        return \"x\"\n    return 1\n\ndef caller() -> int:\n    return later(False)\n";

    for source in [forward, reverse] {
        let errors = lower_source(source).expect_err("union return must not satisfy int");
        assert!(errors.iter().any(|error| {
            error.message == "return type mismatch: expected 'int', got 'int | str'"
        }));
    }
}

#[test]
fn top_level_forward_return_inference_reaches_fixed_point_past_eight_calls() {
    let source = "def f0() -> int:\n    return f1()\n\ndef f1():\n    return f2()\n\ndef f2():\n    return f3()\n\ndef f3():\n    return f4()\n\ndef f4():\n    return f5()\n\ndef f5():\n    return f6()\n\ndef f6():\n    return f7()\n\ndef f7():\n    return f8()\n\ndef f8():\n    return f9()\n\ndef f9():\n    return f10()\n\ndef f10():\n    return f11()\n\ndef f11():\n    return 1\n";

    lower_source(source).expect("module inference should converge for the full declaration group");
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

#[test]
fn top_level_match_return_inference_ignores_unreachable_tail() {
    let module = lower_source(
        "def choose(value: int):\n    match value:\n        case 0:\n            return 1\n        case _:\n            return 2\n    return \"unreachable\"\n\ndef consume() -> int:\n    return choose(0)\n",
    )
    .expect("exhaustive match returns should determine the inferred return type");
    let choose = module
        .functions
        .iter()
        .find(|function| function.name == "choose")
        .expect("choose function missing");
    assert_eq!(choose.return_type, Type::Int);
}

#[test]
fn top_level_try_return_inference_ignores_unreachable_tail() {
    let module = lower_source(
        "def choose(flag: bool):\n    try:\n        if flag:\n            raise ValueError(\"bad\")\n        return 1\n    except ValueError:\n        return 2\n    return \"unreachable\"\n\ndef consume() -> int:\n    return choose(False)\n",
    )
    .expect("try and handler returns should determine the inferred return type");
    let choose = module
        .functions
        .iter()
        .find(|function| function.name == "choose")
        .expect("choose function missing");
    assert_eq!(choose.return_type, Type::Int);
}

#[test]
fn top_level_with_return_inference_ignores_unreachable_tail() {
    let module = lower_source(
        "class Resource:\n    def __enter__(self) -> Resource:\n        return self\n\n    def __exit__(self) -> None:\n        pass\n\ndef choose(resource: Resource):\n    with resource:\n        return 1\n    return \"unreachable\"\n\ndef consume(resource: Resource) -> int:\n    return choose(resource)\n",
    )
    .expect("with-body returns should determine the inferred return type");
    let choose = module
        .functions
        .iter()
        .find(|function| function.name == "choose")
        .expect("choose function missing");
    assert_eq!(choose.return_type, Type::Int);
}

#[test]
fn top_level_match_return_inference_uses_class_pattern_field_types() {
    let module = lower_source(
        "class A:\n    x: int\n\nclass B:\n    y: str\n\ndef choose(value: A | B):\n    match value:\n        case A(x=x):\n            return x\n        case B(y=y):\n            return y\n\ndef consume(value: A | B) -> int | str:\n    return choose(value)\n",
    )
    .expect("class-pattern captures should retain their declared field types");
    let choose = module
        .functions
        .iter()
        .find(|function| function.name == "choose")
        .expect("choose function missing");
    assert_eq!(choose.return_type, Type::Union(vec![Type::Int, Type::Str]));
}

#[test]
fn top_level_with_return_inference_uses_enter_result_type() {
    let module = lower_source(
        "class Resource:\n    def __enter__(self) -> int:\n        return 7\n\n    def __exit__(self) -> None:\n        pass\n\ndef choose(resource: Resource):\n    with resource as value:\n        return value\n\ndef consume(resource: Resource) -> int:\n    return choose(resource)\n",
    )
    .expect("with binding inference should use the __enter__ return type");
    let choose = module
        .functions
        .iter()
        .find(|function| function.name == "choose")
        .expect("choose function missing");
    assert_eq!(choose.return_type, Type::Int);
}

#[test]
fn top_level_match_return_inference_specializes_generic_pattern_fields() {
    let module = lower_source(
        "class Box[T]:\n    value: T\n\ndef choose(box: Box[int]):\n    match box:\n        case Box(value=x):\n            return x\n\ndef consume(box: Box[int]) -> int:\n    return choose(box)\n",
    )
    .expect("generic class-pattern captures should use the subject specialization");
    let choose = module
        .functions
        .iter()
        .find(|function| function.name == "choose")
        .expect("choose function missing");
    assert_eq!(choose.return_type, Type::Int);
}

#[test]
fn top_level_match_return_inference_specializes_nested_generic_patterns() {
    let module = lower_source(
        "class Inner[T]:\n    value: T\n\nclass Outer[T]:\n    inner: Inner[T]\n\ndef choose(outer: Outer[str]):\n    match outer:\n        case Outer(inner=Inner(value=x)):\n            return x\n\ndef consume(outer: Outer[str]) -> str:\n    return choose(outer)\n",
    )
    .expect("nested generic class patterns should retain concrete field types");
    let choose = module
        .functions
        .iter()
        .find(|function| function.name == "choose")
        .expect("choose function missing");
    assert_eq!(choose.return_type, Type::Str);
}

#[test]
fn class_pattern_rejects_union_of_same_generic_class_specializations() {
    let errors = lower_source(
        "class Box[T]:\n    value: T\n\ndef choose(box: Box[int] | Box[str]):\n    match box:\n        case Box(value=x):\n            return x\n",
    )
    .expect_err("same-class specialization unions must be rejected before code generation");
    assert!(errors.iter().any(|error| {
        error
            .message
            .contains("union cannot contain multiple specializations")
    }));
}

#[test]
fn nested_class_pattern_rejects_union_of_same_generic_class_specializations() {
    let errors = lower_source(
        "class Inner[T]:\n    value: T\n\nclass Outer[T]:\n    inner: Inner[T]\n\ndef choose(outer: Outer[int] | Outer[str]):\n    match outer:\n        case Outer(inner=Inner(value=x)):\n            return x\n",
    )
    .expect_err("nested same-class specialization unions must be rejected before code generation");
    assert!(errors.iter().any(|error| {
        error
            .message
            .contains("union cannot contain multiple specializations")
    }));
}

#[test]
fn generic_class_specializations_are_invariant_for_calls() {
    let errors = lower_source(
        "class Box[T]:\n    value: T\n\ndef consume(value: Box[str]) -> None:\n    pass\n\ndef main():\n    value: Box[int] = Box(1)\n    consume(value)\n",
    )
    .expect_err("a concrete generic specialization must not cross another specialization");
    assert!(errors.iter().any(|error| {
        error.message.contains("expected 'Box', got 'Box'")
            || error
                .message
                .contains("expected 'Box[str]', got 'Box[int]'")
    }));
}

#[test]
fn inferred_return_rejects_conflicting_generic_class_specializations() {
    let errors = lower_source(
        "class Box[T]:\n    value: T\n\ndef int_box() -> Box[int]:\n    return Box(1)\n\ndef str_box() -> Box[str]:\n    return Box(\"x\")\n\ndef choose(flag: bool):\n    if flag:\n        return int_box()\n    return str_box()\n",
    )
    .expect_err("inference must reject repeated specializations before HIR/codegen");
    assert!(errors.iter().any(|error| {
        error
            .message
            .contains("multiple specializations of the same generic class")
            || error
                .message
                .contains("return type could not be inferred deterministically")
    }));
}

#[test]
fn inferred_generic_constructor_return_is_specialized() {
    let module = lower_source(
        "class Box[T]:\n    value: T\n\ndef make():\n    return Box(1)\n\ndef main():\n    make()\n",
    )
    .expect("generic constructor inference should substitute its concrete argument");
    let make = module
        .functions
        .iter()
        .find(|function| function.name == "make")
        .expect("make function missing");
    let Type::Class { fields, .. } = &make.return_type else {
        panic!(
            "make should return a specialized class: {:?}",
            make.return_type
        );
    };
    assert_eq!(fields, &vec![("value".to_string(), Type::Int)]);
}

#[test]
fn inferred_generic_function_return_is_specialized() {
    let module = lower_source(
        "def identity[T](value: T) -> T:\n    return value\n\ndef make():\n    return identity(1)\n\ndef main():\n    make()\n",
    )
    .expect("generic function inference should substitute its concrete argument");
    let make = module
        .functions
        .iter()
        .find(|function| function.name == "make")
        .expect("make function missing");
    assert_eq!(make.return_type, Type::Int);
}

#[test]
fn annotated_initializer_context_specializes_zero_argument_generic_return() {
    lower_source(
        "class Marker[T]:\n    pass\n\ndef make[T]() -> Marker[T]:\n    return Marker()\n\ndef main():\n    marker: Marker[int] = make()\n",
    )
    .expect("the declared initializer type should specialize an otherwise unbound return type");
}
