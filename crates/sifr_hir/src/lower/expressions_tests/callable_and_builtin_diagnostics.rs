use super::*;
#[test]
pub(super) fn test_callable_variable_call_errors_have_codes() {
    let arity_source = "def apply(f: Callable[[int], int]) -> int:\n    return f()\n";
    let arity_result = lower_source(arity_source);
    assert!(arity_result.is_err());
    let arity_errors = arity_result.unwrap_err();
    assert!(arity_errors.iter().any(|error| {
        error.message == "callable 'f' expects 1 argument(s), got 0"
            && error.code == Some(DiagnosticCode::CALL_NOT_CALLABLE_OR_ARITY)
            && error.primary_range == Some(range_for_after_anchor(arity_source, "return ", "f"))
    }));

    let type_source = "def apply(f: Callable[[int], int]) -> int:\n    return f(\"bad\")\n";
    let type_result = lower_source(type_source);
    assert!(type_result.is_err());
    let type_errors = type_result.unwrap_err();
    assert!(type_errors.iter().any(|error| {
        error.message == "argument 1 of callable 'f': expected 'int', got 'str'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for_after_anchor(type_source, "f(", "\"bad\""))
    }));
}

#[test]
pub(super) fn test_iter_keyword_has_call_code() {
    let source = "def main():\n    values: list[int] = [1, 2, 3]\n    _it = iter(source=values)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "iter() does not accept keyword arguments"
            && error.code == Some(DiagnosticCode::CALL_UNEXPECTED_KEYWORD)
            && error.primary_range == Some(range_for_after_anchor(source, "iter(", "source=values"))
    }));
}

#[test]
pub(super) fn test_iter_wrong_arg_count_has_call_code() {
    let source = "def main():\n    values: list[int] = [1, 2, 3]\n    _it = iter(values, values)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "iter() takes exactly 1 argument, got 2"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range
                == Some(range_for_after_anchor(source, "iter(values, ", "values"))
    }));
}

#[test]
pub(super) fn test_iter_non_iterable_has_type_code() {
    let source = "def main():\n    _it = iter(1)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "iter() argument must be iterable, got 'int'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for_after_anchor(source, "iter(", "1"))
    }));
}

#[test]
pub(super) fn test_next_non_iterator_has_type_code() {
    let source = "def main():\n    values: list[int] = [1, 2, 3]\n    next(values)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "next() argument must be an iterator, got 'list[int]'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for_after_anchor(source, "next(", "values"))
    }));
}

#[test]
pub(super) fn test_pow_wrong_arg_count_has_call_code() {
    let source = "def main():\n    value: int = pow(2)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "pow() takes exactly 2 arguments"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range == Some(range_for_after_anchor(source, "pow(", "2"))
    }));
}

#[test]
pub(super) fn test_scalar_builtin_wrong_arg_counts_have_call_code() {
    let cases = [
        ("abs", "abs()", "abs() takes exactly 1 argument, got 0"),
        ("hash", "hash()", "hash() takes exactly 1 argument, got 0"),
        ("round", "round()", "round() takes 1 or 2 arguments, got 0"),
        ("repr", "repr()", "repr() takes exactly 1 argument, got 0"),
        ("int", "int()", "int() takes exactly 1 argument, got 0"),
        (
            "bigint",
            "bigint()",
            "bigint() takes exactly 1 argument, got 0",
        ),
        (
            "float",
            "float()",
            "float() takes exactly 1 argument, got 0",
        ),
        ("bool", "bool()", "bool() takes exactly 1 argument, got 0"),
    ];

    for (callable, call, message) in cases {
        let source = format!("def main():\n    _value = {call}\n");
        let result = lower_source(&source);
        assert!(result.is_err(), "{callable} should reject wrong arity");
        let errors = result.unwrap_err();
        assert!(
            errors.iter().any(|error| {
                error.message == message
                    && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
                    && error.primary_range
                        == Some(range_for_after_anchor(&source, "_value = ", callable))
            }),
            "{callable} errors: {errors:?}"
        );
    }
}

#[test]
pub(super) fn test_scalar_builtin_keywords_have_call_code() {
    let callables = [
        "abs", "hash", "round", "repr", "int", "bigint", "float", "bool",
    ];

    for callable in callables {
        let source = format!("def main():\n    _value = {callable}(value=1)\n");
        let result = lower_source(&source);
        assert!(result.is_err(), "{callable} should reject keywords");
        let errors = result.unwrap_err();
        assert!(
            errors.iter().any(|error| {
                error.message == format!("{callable}() does not accept keyword arguments")
                    && error.code == Some(DiagnosticCode::CALL_UNEXPECTED_KEYWORD)
                    && error.primary_range
                        == Some(range_for_after_anchor(
                            &source,
                            &format!("{callable}("),
                            "value=1",
                        ))
            }),
            "{callable} errors: {errors:?}"
        );
    }
}

#[test]
pub(super) fn test_scalar_builtin_type_mismatches_have_type_code() {
    let cases = [
        (
            "abs",
            "abs(\"x\")",
            "abs() argument must be numeric, got 'str'",
        ),
        (
            "round",
            "round(\"x\")",
            "round() argument must be numeric, got 'str'",
        ),
        (
            "bigint",
            "bigint(\"x\")",
            "bigint() requires int, bigint, decimal, or bigdecimal argument, got 'str'",
        ),
    ];

    for (callable, call, message) in cases {
        let source = format!("def main():\n    _value = {call}\n");
        let result = lower_source(&source);
        assert!(
            result.is_err(),
            "{callable} should reject invalid argument type"
        );
        let errors = result.unwrap_err();
        assert!(
            errors.iter().any(|error| {
                error.message == message
                    && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                    && error.primary_range
                        == Some(range_for_after_anchor(
                            &source,
                            &format!("{callable}("),
                            "\"x\"",
                        ))
            }),
            "{callable} errors: {errors:?}"
        );
    }
}

#[test]
pub(super) fn test_abs_fixed_width_builtin_widens_to_int() {
    let module =
        lower_source("def main():\n    value: int8 = -128\n    widened: int = abs(value)\n")
            .expect("fixed-width abs should widen to int");
    let main_fn = module
        .functions
        .iter()
        .find(|func| func.name == "main")
        .expect("main should lower");
    let HirStmt::Let { ty, .. } = &main_fn.body[1] else {
        panic!("expected widened let");
    };
    assert_eq!(ty, &Type::Int);
}

#[test]
pub(super) fn test_hash_unhashable_argument_has_proto_code() {
    let result = lower_source(
        "class Measurement:\n    value: float\n\n    def __init__(self, value: float):\n        self.value = value\n\ndef main():\n    m: Measurement = Measurement(3.14)\n    print(hash(m))\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "hash() argument must be hashable, got 'Measurement'"
            && error.code == Some(DiagnosticCode::PROTO_HASHABLE_OR_COMPARABLE_REQUIRED)
    }));
}

#[test]
pub(super) fn test_function_wrong_arg_count_has_call_code() {
    let source =
        "def takes_one(x: int) -> int:\n    return x\n\ndef main():\n    print(takes_one(1, 2))\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "takes_one() takes at most 1 argument(s), got 2"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range == Some(range_for_after_anchor(source, "takes_one(1, ", "2"))
    }));
}

#[test]
pub(super) fn test_missing_required_argument_has_call_code() {
    let source = "def display(name: str, *, verbose: bool) -> str:\n    if verbose:\n        return \"verbose\"\n    return \"quiet\"\n\ndef main():\n    print(display(\"Alice\"))\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "display() missing required argument 'verbose'"
            && error.code == Some(DiagnosticCode::CALL_MISSING_REQUIRED_ARGUMENT)
            && error.primary_range == Some(range_for_after_anchor(source, "print(", "display"))
    }));
}

#[test]
pub(super) fn test_defaultdict_accepts_counter_initial_mapping() {
    let result = lower_source(
        "class Counter[K: Hashable]:\n    counts: dict[K, int]\n\n    def __init__(self):\n        self.counts = {}\n\ndef main():\n    c = Counter()\n    d = defaultdict(int, c)\n    assert d is not None\n",
    );
    assert!(
        result.is_ok(),
        "defaultdict(int, Counter(...)) should lower via Counter.counts mapping bridge: {:?}",
        result.err()
    );
}

#[test]
pub(super) fn test_defaultdict_subscript_read_is_non_optional_value_type() {
    let result = lower_source(
        "def main() -> int:\n    counts = defaultdict(int)\n    counts[1] += 1\n    value: int = counts[2]\n    return value\n",
    );
    assert!(
        result.is_ok(),
        "defaultdict index reads should resolve to the factory value type, not Optional"
    );
}

#[test]
pub(super) fn test_defaultdict_membership_checks_lower() {
    let result = lower_source(
        "def main() -> bool:\n    groups = defaultdict(list)\n    groups[\"a\"].append(1)\n    return \"a\" in groups and \"b\" not in groups\n",
    );
    assert!(
        result.is_ok(),
        "defaultdict membership checks should lower through compat mapping surface: {:?}",
        result.err()
    );
}

#[test]
pub(super) fn test_range_membership_checks_lower() {
    let result =
        lower_source("def main() -> bool:\n    return (2 in range(5)) and (9 not in range(5))\n");
    assert!(result.is_ok(), "{result:?}");
}

#[test]
pub(super) fn test_imported_counter_iterable_constructor_remains_unsupported() {
    let result = lower_source(
        "from sifr.collections import Counter\n\ndef main():\n    c: Counter[str] = Counter([\"a\", \"b\", \"a\"])\n",
    );
    assert!(
        result.is_err(),
        "imported sifr.collections.Counter(list[T]) should remain unsupported"
    );
}

#[test]
pub(super) fn test_constructor_assigned_fields_infer_class_instance_types() {
    let result = lower_source(
        "class Marker:\n    def __init__(self):\n        self.marked = False\n\nclass Registry:\n    def __init__(self):\n        self.root = Marker()\n\n    def is_marked(self) -> bool:\n        return self.root.marked\n\ndef main() -> bool:\n    registry = Registry()\n    return registry.is_marked()\n",
    );
    assert!(
        result.is_ok(),
        "constructor-assigned class instance fields should be registered and typed"
    );
}

#[test]
pub(super) fn test_constructor_branch_assignments_register_all_fields() {
    let module = lower_source(
        "class Pair:\n    def __init__(self, flag: bool):\n        if flag:\n            self.left = 1\n        else:\n            self.right = 2\n",
    )
    .expect("constructor field registration should succeed");
    let pair = module
        .classes
        .iter()
        .find(|class| class.name == "Pair")
        .expect("Pair class should lower");
    assert!(pair.fields.iter().any(|(name, _)| name == "left"));
    assert!(pair.fields.iter().any(|(name, _)| name == "right"));
}

#[test]
pub(super) fn test_attribute_subscript_augassign_lowers_for_class_fields() {
    let result = lower_source(
        "class Counter:\n    def __init__(self):\n        self.counts = {}\n\n    def bump(self, key: int) -> None:\n        if key not in self.counts:\n            self.counts[key] = 0\n        self.counts[key] += 1\n\ndef main() -> None:\n    c = Counter()\n    c.bump(1)\n",
    );
    assert!(
        result.is_err(),
        "fixture should still fail due optional indexing semantics"
    );
    let errors = result.unwrap_err();
    assert!(
        !errors.iter().any(|error| {
            error
                .message
                .contains("augmented subscript assignment target must be a simple name")
        }),
        "attribute subscript augassign should lower past target-shape validation: {errors:?}"
    );
    assert!(
        errors.iter().any(
            |error| error.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR)
                && error.message.contains("unsupported operand type(s) for +")
        ),
        "lowering should reach operand typing for attribute subscript augassign: {errors:?}"
    );
}

#[test]
pub(super) fn test_nested_subscript_augassign_lowers_for_name_targets() {
    let result =
        lower_source("def bump(mut grid: list[list[int]]) -> None:\n    grid[0][0] += 1\n");
    assert!(
        result.is_err(),
        "fixture should still fail due optional indexing semantics"
    );
    let errors = result.unwrap_err();
    assert!(
        !errors.iter().any(|error| {
            error
                .message
                .contains("augmented subscript assignment target must be a simple name")
        }),
        "nested subscript augassign should lower past target-shape validation: {errors:?}"
    );
    assert!(
        errors.iter().any(
            |error| error.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR)
                && error.message.contains("unsupported operand type(s) for +")
        ),
        "lowering should reach operand typing for nested subscript augassign: {errors:?}"
    );
}

#[test]
pub(super) fn test_matrix_augassign_has_unsupported_operator_code() {
    let source = "def bad(mut value: int) -> None:\n    value @= 1\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected matrix augassign unsupported operator error");

    assert!(
        errors.iter().any(
            |error| error.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR)
                && error.message == "matrix multiplication operator (@) is not supported"
                && error.primary_range
                    == Some(range_for_after(source, ") -> None:\n    ", "value"))
        ),
        "matrix augassign diagnostic should be structured and ranged: {errors:?}"
    );
}

#[test]
pub(super) fn test_matrix_binop_has_unsupported_operator_code() {
    let source = "def main():\n    x: int = 1 @ 2\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected matrix binop unsupported operator error");

    assert!(
        errors.iter().any(
            |error| error.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR)
                && error.message == "matrix multiplication operator (@) is not supported"
                && error.primary_range == Some(range_for(source, "1 @ 2"))
        ),
        "matrix binop diagnostic should be structured and ranged: {errors:?}"
    );
}

#[test]
pub(super) fn test_unsupported_expression_form_has_type_code() {
    let source = "def main():\n    x = (yield 1)\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected unsupported expression form error");

    assert!(
        errors.iter().any(|error| error.code
            == Some(DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM)
            && error.message == "unsupported expression form: unsupported expression type"
            && error.primary_range == Some(range_for(source, "yield 1"))),
        "unsupported expression form diagnostic should be structured and ranged: {errors:?}"
    );
}

#[test]
pub(super) fn test_in_operator_non_collection_has_unsupported_operator_code() {
    let source = "def main() -> bool:\n    return 1 in 2\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected unsupported in operator error");

    assert!(
        errors.iter().any(
            |error| error.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR)
                && error.message == "unsupported operator in for int"
                && error.primary_range == Some(range_for_after(source, " in ", "2"))
        ),
        "in operator diagnostic should be structured and ranged: {errors:?}"
    );
}

#[test]
pub(super) fn test_dict_unpacking_has_type_code() {
    let source = "def main():\n    other: dict[str, int] = {}\n    merged = {\"a\": 1, **other}\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected dict unpacking error");
    assert!(errors.iter().any(|error| {
        error.message == "dict unpacking (**) not supported"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for_after_anchor(source, "**", "other"))
    }));
}

#[test]
pub(super) fn test_tuple_slice_errors_have_type_codes() {
    let out_of_range_source =
        "def main():\n    pair: tuple[int, str] = (1, \"x\")\n    _bad = pair[0:3]\n";
    let out_of_range_result = lower_source(out_of_range_source);
    let out_of_range_errors = out_of_range_result.expect_err("expected tuple slice range error");
    assert!(out_of_range_errors.iter().any(|error| {
        error.message == "tuple slice indices out of range"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range
                == Some(range_for_after_anchor(out_of_range_source, "pair[", "0:3"))
    }));

    let dynamic_source =
        "def main():\n    pair: tuple[int, str] = (1, \"x\")\n    start: int = 0\n    _bad = pair[start:2]\n";
    let dynamic_result = lower_source(dynamic_source);
    let dynamic_errors = dynamic_result.expect_err("expected tuple dynamic slice error");
    assert!(dynamic_errors.iter().any(|error| {
        error.message == "tuple slicing requires compile-time constant indices"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range
                == Some(range_for_after_anchor(dynamic_source, "pair[", "start:2"))
    }));
}

#[test]
pub(super) fn test_unsupported_slice_receiver_has_type_code() {
    let source = "def main():\n    value: int = 1\n    _bad = value[0:1]\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected unsupported slice receiver error");
    assert!(errors.iter().any(|error| {
        error.message == "cannot slice type 'int'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for_after_anchor(source, "value[", "0:1"))
    }));
}

#[test]
pub(super) fn test_augassign_complex_targets_have_type_codes() {
    let cases = [
        (
            "attribute receiver",
            "def make_box() -> int:\n    return 1\n\ndef bad() -> None:\n    make_box().field += 1\n",
            "augmented attribute assignment target must be a simple name",
            "make_box()",
        ),
        (
            "subscript receiver",
            "def make_items() -> list[int]:\n    return [1]\n\ndef bad() -> None:\n    make_items()[0] += 1\n",
            "augmented subscript assignment target must be a simple name",
            "make_items()",
        ),
        (
            "attribute subscript receiver",
            "def make_box() -> int:\n    return 1\n\ndef bad() -> None:\n    make_box().counts[0] += 1\n",
            "augmented subscript assignment target must be a simple name",
            "make_box()",
        ),
        (
            "nested subscript receiver",
            "def make_grid() -> list[list[int]]:\n    return [[1]]\n\ndef bad() -> None:\n    make_grid()[0][0] += 1\n",
            "augmented subscript assignment target must be a simple name",
            "make_grid()",
        ),
        (
            "nested subscript expression receiver",
            "def bad(mut xs: list[list[int]], mut ys: list[list[int]]) -> None:\n    (xs + ys)[0][0] += 1\n",
            "augmented subscript assignment target must be a simple name",
            "xs + ys",
        ),
    ];

    for (label, source, message, range_needle) in cases {
        let result = lower_source(source);
        let errors = result.expect_err("expected complex augassign target error");
        assert!(
            errors
                .iter()
                .any(|error| error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                    && error.message == message
                    && error.primary_range
                        == Some(range_for_after(source, ") -> None:\n    ", range_needle))),
            "{label} diagnostic should be structured and ranged: {errors:?}"
        );
    }
}

#[test]
pub(super) fn test_bytes_subscript_assignment_has_ownership_code() {
    let source = "def main() -> None:\n    payload: bytes = b\"abc\"\n    payload[0] = 65\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected bytes subscript assignment error");

    assert!(
        errors.iter().any(|error| error.code
            == Some(DiagnosticCode::OWN_IMMUTABLE_BYTES_ASSIGNMENT)
            && error.message == "bytes is immutable; subscript assignment is not supported"
            && error.primary_range == Some(range_for(source, "payload[0]"))),
        "bytes subscript assignment should preserve ownership code: {errors:?}"
    );
}

#[test]
pub(super) fn test_bytes_augmented_subscript_assignment_has_ownership_code() {
    let source = "def main() -> None:\n    payload: bytes = b\"abc\"\n    payload[0] += 1\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected bytes augmented subscript assignment error");

    assert!(
        errors.iter().any(|error| error.code
            == Some(DiagnosticCode::OWN_IMMUTABLE_BYTES_AUGMENTED_ASSIGNMENT)
            && error.message
                == "bytes is immutable; augmented subscript assignment is not supported"
            && error.primary_range == Some(range_for(source, "payload[0]"))),
        "bytes augmented subscript assignment should preserve ownership code: {errors:?}"
    );
}

#[test]
pub(super) fn test_bytes_index_and_iteration_expose_uint8() {
    let source =
        "def main() -> None:\n    payload: bytes = b\"abc\"\n    first = payload[0]\n    for value in payload:\n        seen: uint8 = value\n";
    let module = lower_source(source).expect("bytes uint8 lowering should succeed");
    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("main function should exist");

    let Some(HirStmt::Let { ty: first_ty, .. }) = function
        .body
        .iter()
        .find(|stmt| matches!(stmt, HirStmt::Let { name, .. } if name == "first"))
    else {
        panic!("expected first binding");
    };
    assert_eq!(
        first_ty,
        &Type::Union(vec![Type::FixedInt(FixedIntType::U8), Type::None])
    );

    let Some(HirStmt::For { target_ty, .. }) = function
        .body
        .iter()
        .find(|stmt| matches!(stmt, HirStmt::For { target, .. } if target == "value"))
    else {
        panic!("expected bytes for loop");
    };
    assert_eq!(target_ty, &Type::FixedInt(FixedIntType::U8));
}

#[test]
pub(super) fn test_bytes_codec_type_errors_have_structured_codes() {
    let encode_source = "def main() -> None:\n    _bad: bytes = \"abc\".encode(1)\n";
    let encode_result = lower_source(encode_source);
    let encode_errors = encode_result.expect_err("expected str.encode codec type error");
    assert!(
        encode_errors
            .iter()
            .any(|error| error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                && error.message == "str.encode() encoding must be 'str', got 'int'"
                && error.primary_range == Some(range_for(encode_source, "1"))),
        "str.encode codec type diagnostic should be structured and ranged: {encode_errors:?}"
    );

    let decode_source = "def main() -> None:\n    _bad: str = b\"abc\".decode(1)\n";
    let decode_result = lower_source(decode_source);
    let decode_errors = decode_result.expect_err("expected bytes.decode codec type error");
    assert!(
        decode_errors
            .iter()
            .any(|error| error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                && error.message == "bytes.decode() encoding must be 'str', got 'int'"
                && error.primary_range == Some(range_for(decode_source, "1"))),
        "bytes.decode codec type diagnostic should be structured and ranged: {decode_errors:?}"
    );
}

#[test]
pub(super) fn test_decimal_method_surface_errors_have_structured_codes() {
    let arity_source =
        "def main() -> None:\n    d: decimal = Decimal(\"1.25\")\n    _bad: decimal = d.sqrt(1)\n";
    let arity_result = lower_source(arity_source);
    let arity_errors = arity_result.expect_err("expected decimal.sqrt arity error");
    assert!(
        arity_errors.iter().any(|error| error.code
            == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.message == "decimal.sqrt() takes no arguments"
            && error.primary_range == Some(range_for_after_anchor(arity_source, "sqrt(", "1"))),
        "decimal.sqrt arity diagnostic should be structured and ranged: {arity_errors:?}"
    );

    let method_source = "def main() -> None:\n    d: decimal = Decimal(\"1.25\")\n    _bad: decimal = d.magnitude()\n";
    let method_result = lower_source(method_source);
    let method_errors = method_result.expect_err("expected decimal unknown method error");
    assert!(
        method_errors.iter().any(|error| error.code
            == Some(DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE)
            && error.message == "type 'decimal' has no method 'magnitude'"
            && error.primary_range == Some(range_for(method_source, "magnitude"))),
        "decimal unknown method diagnostic should be structured and ranged: {method_errors:?}"
    );
}

#[test]
pub(super) fn test_list_subscript_augassign_type_error_keeps_code() {
    let source = "def bad(mut xs: list[int]) -> None:\n    xs[0] += \"x\"\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected list subscript augassign type error");

    assert!(
        errors.iter().any(
            |error| error.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR)
                && error.message.contains("unsupported operand type(s) for +")
                && error.primary_range == Some(range_for(source, "\"x\""))
        ),
        "list subscript augassign should preserve the operator helper code: {errors:?}"
    );
}

#[test]
pub(super) fn test_dict_subscript_augassign_type_error_keeps_code() {
    let source =
        "def bad(mut data: dict[str, int]) -> None:\n    data[\"x\"] = 1\n    data[\"x\"] += \"x\"\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected dict subscript augassign type error");

    assert!(
        errors.iter().any(
            |error| error.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR)
                && error.message.contains("unsupported operand type(s) for +")
                && error.primary_range == Some(range_for_after(source, "+= ", "\"x\""))
        ),
        "dict subscript augassign should preserve the operator helper code: {errors:?}"
    );
}

#[test]
pub(super) fn test_list_subscript_assignment_index_error_has_type_code() {
    let source = "def bad(mut xs: list[int]) -> None:\n    xs[\"0\"] = 1\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected list subscript assignment index error");

    assert!(
        errors
            .iter()
            .any(|error| error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                && error.message == "list subscript assignment index must be 'int', got 'str'"
                && error.primary_range == Some(range_for(source, "xs[\"0\"]"))),
        "list subscript assignment index diagnostic should be structured and ranged: {errors:?}"
    );
}

#[test]
pub(super) fn test_list_subscript_assignment_value_error_has_type_code() {
    let source = "def bad(mut xs: list[int]) -> None:\n    xs[0] = \"x\"\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected list subscript assignment value error");

    assert!(
        errors
            .iter()
            .any(|error| error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                && error.message
                    == "list subscript assignment value type 'str' is not compatible with list element type 'int'"
                && error.primary_range == Some(range_for(source, "xs[0]"))),
        "list subscript assignment value diagnostic should be structured and ranged: {errors:?}"
    );
}

#[test]
pub(super) fn test_unsupported_subscript_assignment_has_type_code() {
    let source = "def bad(mut value: int) -> None:\n    value[0] = 1\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected unsupported subscript assignment error");

    assert!(
        errors
            .iter()
            .any(|error| error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                && error.message == "subscript assignment is not supported for type 'int'"
                && error.primary_range == Some(range_for(source, "value[0]"))),
        "unsupported subscript assignment diagnostic should be structured and ranged: {errors:?}"
    );
}

#[test]
pub(super) fn test_unsupported_subscript_augassign_has_type_code() {
    let source = "def bad(mut value: int) -> None:\n    value[0] += 1\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected unsupported subscript augassign error");

    assert!(
        errors
            .iter()
            .any(|error| error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                && error.message
                    == "augmented subscript assignment is not supported for type 'int'"
                && error.primary_range == Some(range_for(source, "value[0]"))),
        "unsupported subscript augassign diagnostic should be structured and ranged: {errors:?}"
    );
}

#[test]
pub(super) fn test_tuple_index_out_of_range_has_type_code() {
    let source = "def main():\n    pair: tuple[int, str] = (1, \"x\")\n    value: int = pair[2]\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected tuple index error");

    assert!(
        errors
            .iter()
            .any(|error| error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                && error.message == "tuple index out of range"
                && error.primary_range == Some(range_for(source, "2"))),
        "tuple index diagnostic should preserve type code and literal index range: {errors:?}"
    );
}

#[test]
pub(super) fn test_invalid_subscript_receiver_has_type_code() {
    let source = "def main():\n    value: int = 1\n    bad: int = value[0]\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected invalid subscript receiver error");

    assert!(
        errors.iter().any(|error| error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.message == "cannot index type 'int' with 'int'"
            && error.primary_range == Some(range_for_after_anchor(source, "bad: int = ", "value[0]"))),
        "invalid subscript receiver diagnostic should preserve type code and subscript range: {errors:?}"
    );
}

#[test]
pub(super) fn test_nested_attribute_assignment_target_lowers_for_self_fields() {
    let result = lower_source(
        "class ChainCell:\n    next: ChainCell | None\n\n    def __init__(self):\n        self.next = None\n\nclass Wrapper:\n    head: ChainCell\n\n    def __init__(self):\n        self.head = ChainCell()\n        self.head.next = ChainCell()\n",
    );
    assert!(
        result.is_ok(),
        "nested attribute assignment on class fields should lower: {:?}",
        result.err()
    );
}

#[test]
pub(super) fn test_nested_attribute_assignment_lowers_for_optional_field_base() {
    let result = lower_source(
        "class ChainCell:\n    next: ChainCell | None\n    prev: ChainCell | None\n\n    def __init__(self):\n        self.next = None\n        self.prev = None\n\ndef relink(mut node: ChainCell) -> None:\n    if node.prev is not None:\n        node.prev.next = node.next\n",
    );
    assert!(
        result.is_ok(),
        "nested attribute assignment through optional field bases should lower under explicit narrowing: {:?}",
        result.err()
    );
}
