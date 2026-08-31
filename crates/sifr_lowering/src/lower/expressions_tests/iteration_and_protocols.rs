use super::*;
use crate::lower::{LowerCtx, expressions::lower_named_expr};
#[test]
pub(super) fn test_iter_rejects_heterogeneous_tuple_argument() {
    let result = lower_source(
        "def main():\n    values: tuple[int, str] = (1, \"x\")\n    _it = iter(values)\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("iter() tuple argument must have one statically provable element type")
    }));
}

#[test]
pub(super) fn test_for_accepts_homogeneous_tuple_iterable() {
    let result = lower_source(
        "def main():\n    values: tuple[int, int, int] = (1, 2, 3)\n    total: int = 0\n    for value in values:\n        total = total + value\n",
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
pub(super) fn test_for_rejects_heterogeneous_tuple_iterable() {
    let result = lower_source(
        "def main():\n    values: tuple[int, str] = (1, \"x\")\n    for value in values:\n        print(value)\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("for-loop tuple iteration requires one statically provable element type")
    }));
}

#[test]
pub(super) fn test_next_rejects_plain_iterable_argument() {
    let result = lower_source("def main():\n    values: list[int] = [1, 2, 3]\n    next(values)\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("next() argument must be an iterator"))
    );
}

#[test]
pub(super) fn test_user_defined_iterable_class_participates_in_builtin_iteration_surface() {
    let result = lower_source(
        "class Boxed:\n    items: list[int]\n\n    def __init__(self, items: list[int]):\n        self.items = items\n\n    def __iter__(self) -> Iterator[int]:\n        return iter(self.items)\n\n    def __reversed__(self) -> Iterator[int]:\n        return reversed(self.items)\n\n\ndef main():\n    boxed: Boxed = Boxed([1, 2, 3])\n    vals: list[int] = list(boxed)\n    rev_vals: list[int] = list(reversed(boxed))\n    total: int = 0\n    for value in boxed:\n        total = total + value\n",
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
pub(super) fn test_next_accepts_user_defined_iterator_class() {
    let result = lower_source(
        "class CounterIter:\n    value: int\n\n    def __init__(self, start: int):\n        self.value = start\n\n    def __iter__(self) -> Iterator[int]:\n        return iter([self.value])\n\n    def __next__(mut self) -> int | None:\n        if self.value <= 0:\n            return None\n        out: int = self.value\n        self.value = self.value - 1\n        return out\n\n\ndef main():\n    it: CounterIter = CounterIter(2)\n    first: int | None = next(it)\n",
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
pub(super) fn test_user_defined_iterable_protocol_rejects_invalid_iter_signature() {
    let result = lower_source("class BadIter:\n    def __iter__(self) -> int:\n        return 1\n");
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error
            .message
            .contains("class 'BadIter.__iter__' must return 'Iterator[T]' or 'Iterable[T]'")
    }));
}

#[test]
pub(super) fn test_user_defined_iterable_protocol_rejects_invalid_next_signature() {
    let result = lower_source(
        "class BadNext:\n    def __iter__(self) -> Iterator[int]:\n        return iter([1])\n\n    def __next__(self) -> int:\n        return 1\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error
            .message
            .contains("class 'BadNext.__next__' must return 'T | None'")
    }));
}

#[test]
pub(super) fn test_for_rejects_mutation_of_collection_with_live_iterator() {
    let result = lower_source(
        "def main():\n    values: list[int] = [1, 2, 3]\n    for value in values:\n        values.append(value)\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("cannot mutate 'values' while iterating over it in a for loop")
    }));
}

#[test]
pub(super) fn test_generator_function_infers_iterator_return_type() {
    let module = lower_source(
        "def count_up(n: int):\n    i: int = 0\n    while i < n:\n        yield i\n        i = i + 1\n",
    )
    .unwrap();
    assert_eq!(
        module.functions[0].return_type,
        Type::Iterator(Box::new(Type::Int))
    );
}

#[test]
pub(super) fn test_generator_function_rejects_non_iterator_annotation() {
    let source = "def count_up(n: int) -> list[int]:\n    i: int = 0\n    while i < n:\n        yield i\n        i = i + 1\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message.contains("must declare return type 'Iterator[T]'")
            && e.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && e.primary_range == Some(range_for_after_anchor(source, "-> ", "list[int]"))
    }));
}

#[test]
pub(super) fn test_generator_expression_is_typed_as_iterator() {
    let module = lower_source(
        "def main():\n    nums: list[int] = [1, 2, 3]\n    g: Iterator[int] = (x * x for x in nums)\n    _first: int | None = next(g)\n",
    )
    .unwrap();
    let main_fn = module
        .functions
        .iter()
        .find(|func| func.name == "main")
        .expect("main function should exist");
    let Some(HirStmt::Let { ty, .. }) = main_fn
        .body
        .iter()
        .find(|stmt| matches!(stmt, HirStmt::Let { name, .. } if name == "g"))
    else {
        panic!("expected let binding for generator expression");
    };
    assert!(matches!(ty, Type::Iterator(_)));
}

#[test]
pub(super) fn test_generator_accepts_nested_yield_shapes() {
    let module = lower_source(
        "def nested(n: int):\n    i: int = 0\n    while i < n:\n        while i < n:\n            yield i\n            i = i + 1\n",
    )
    .unwrap();
    assert_eq!(
        module.functions[0].return_type,
        Type::Iterator(Box::new(Type::Int))
    );
}

#[test]
pub(super) fn test_generator_accepts_trailing_statements_after_loop() {
    let module = lower_source(
        "def trailing(n: int):\n    i: int = 0\n    while i < n:\n        yield i\n        i = i + 1\n    i = i + 1\n",
    )
    .unwrap();
    assert_eq!(
        module.functions[0].return_type,
        Type::Iterator(Box::new(Type::Int))
    );
}

#[test]
pub(super) fn test_generators_accept_bare_return_as_exhaustion() {
    let module = lower_source(
        "def sync_values() -> Iterator[int]:\n    yield 1\n    return\n\nasync def async_values() -> AsyncGenerator[int, GeneratorCloseError]:\n    yield 1\n    return\n",
    )
    .unwrap();
    assert!(matches!(
        module.functions[0].body.last(),
        Some(HirStmt::Return { value: None })
    ));
    assert!(matches!(
        module.functions[1].body.last(),
        Some(HirStmt::Return { value: None })
    ));
}

#[test]
pub(super) fn test_generator_rejects_non_none_return_value() {
    let result = lower_source("def values() -> Iterator[int]:\n    yield 1\n    return 2\n");
    let errors = result.expect_err("generator return value should be rejected");
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error
                .message
                .contains("non-None generator return values are rejected")
    }));
}

#[test]
pub(super) fn test_reversed_enumerate_zip_are_typed_as_iterators() {
    let module = lower_source(
        "def main():\n    nums: list[int] = [1, 2, 3]\n    labels: list[str] = [\"a\", \"b\", \"c\"]\n    rev: Iterator[int] = reversed(nums)\n    indexed: Iterator[tuple[int, int]] = enumerate(nums, start=1)\n    paired: Iterator[tuple[int, str]] = zip(nums, labels)\n    _rev_list: list[int] = list(rev)\n    _indexed_list: list[tuple[int, int]] = list(indexed)\n    _paired_list: list[tuple[int, str]] = list(paired)\n",
    )
    .unwrap();
    let main_fn = module
        .functions
        .iter()
        .find(|func| func.name == "main")
        .expect("main function should exist");
    let Some(HirStmt::Let { ty, .. }) = main_fn
        .body
        .iter()
        .find(|stmt| matches!(stmt, HirStmt::Let { name, .. } if name == "rev"))
    else {
        panic!("expected let binding for rev");
    };
    assert!(matches!(ty, Type::Iterator(_)));
    let Some(HirStmt::Let { ty, .. }) = main_fn
        .body
        .iter()
        .find(|stmt| matches!(stmt, HirStmt::Let { name, .. } if name == "indexed"))
    else {
        panic!("expected let binding for indexed");
    };
    assert!(matches!(ty, Type::Iterator(_)));
    let Some(HirStmt::Let { ty, .. }) = main_fn
        .body
        .iter()
        .find(|stmt| matches!(stmt, HirStmt::Let { name, .. } if name == "paired"))
    else {
        panic!("expected let binding for paired");
    };
    assert!(matches!(ty, Type::Iterator(_)));
}

#[test]
pub(super) fn test_zip_keyword_diagnostics_are_stable() {
    let strict_result = lower_source(
        "def main():\n    nums: list[int] = [1, 2]\n    _paired = zip(nums, nums, strict=True)\n",
    );
    assert!(strict_result.is_err());
    let strict_errors = strict_result.unwrap_err();
    assert!(strict_errors.iter().any(|error| {
        error
            .message
            .contains("zip() keyword argument 'strict' is not supported")
    }));

    let unexpected_source =
        "def main():\n    nums: list[int] = [1, 2]\n    _paired = zip(nums, nums, bogus=True)\n";
    let unexpected_result = lower_source(unexpected_source);
    assert!(unexpected_result.is_err());
    let unexpected_errors = unexpected_result.unwrap_err();
    assert!(unexpected_errors.iter().any(|error| {
        error.message == "zip() got an unexpected keyword argument 'bogus'"
            && error.code == Some(DiagnosticCode::CALL_UNEXPECTED_KEYWORD)
            && error.primary_range
                == Some(range_for_after_anchor(
                    unexpected_source,
                    "zip(nums, nums, ",
                    "bogus",
                ))
    }));
}

#[test]
pub(super) fn test_zip_non_iterable_argument_has_type_code() {
    let source = "def main():\n    nums: list[int] = [1, 2]\n    _paired = zip(nums, 1)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message
            == "zip() argument 2 must be an iterable with a statically-known element type, got 'int'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for_after_anchor(source, "zip(nums, ", "1"))
    }));
}

#[test]
pub(super) fn test_any_all_wrong_arity_have_call_codes() {
    let any_source = "def main():\n    _value = any()\n";
    let any_result = lower_source(any_source);
    assert!(any_result.is_err());
    let any_errors = any_result.unwrap_err();
    assert!(any_errors.iter().any(|error| {
        error.message == "any() takes exactly 1 argument"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range == Some(range_for(any_source, "any"))
    }));

    let all_source =
        "def main():\n    flags: list[bool] = [True]\n    _value = all(flags, flags)\n";
    let all_result = lower_source(all_source);
    assert!(all_result.is_err());
    let all_errors = all_result.unwrap_err();
    assert!(all_errors.iter().any(|error| {
        error.message == "all() takes exactly 1 argument"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range
                == Some(range_for_after_anchor(all_source, "all(flags, ", "flags"))
    }));
}

#[test]
pub(super) fn test_range_and_enumerate_unexpected_keywords_have_call_code() {
    let range_source = "def main():\n    print(list(range(stop=3, bogus=1)))\n";
    let range_result = lower_source(range_source);
    assert!(range_result.is_err());
    let range_errors = range_result.unwrap_err();
    assert!(range_errors.iter().any(|error| {
        error.message == "range() got an unexpected keyword argument 'bogus'"
            && error.code == Some(DiagnosticCode::CALL_UNEXPECTED_KEYWORD)
            && error.primary_range
                == Some(range_for_after_anchor(range_source, "stop=3, ", "bogus"))
    }));

    let enumerate_source =
        "def main():\n    nums: list[int] = [1, 2]\n    _items = enumerate(nums, bogus=1)\n";
    let enumerate_result = lower_source(enumerate_source);
    assert!(enumerate_result.is_err());
    let enumerate_errors = enumerate_result.unwrap_err();
    assert!(enumerate_errors.iter().any(|error| {
        error.message == "enumerate() got an unexpected keyword argument 'bogus'"
            && error.code == Some(DiagnosticCode::CALL_UNEXPECTED_KEYWORD)
            && error.primary_range
                == Some(range_for_after_anchor(
                    enumerate_source,
                    "enumerate(nums, ",
                    "bogus",
                ))
    }));
}

#[test]
pub(super) fn test_enumerate_duplicate_start_keyword_has_call_code() {
    let source = "\
def main():
    nums: list[int] = [1, 2]
    _items = enumerate(nums, 10, start=1)
";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "enumerate() got multiple values for argument 'start'"
            && error.code == Some(DiagnosticCode::CALL_DUPLICATE_ARGUMENT)
            && error.primary_range
                == Some(range_for_after_anchor(
                    source,
                    "enumerate(nums, 10, ",
                    "start",
                ))
    }));
}

#[test]
pub(super) fn test_reversed_rejects_non_reversible_iterator_argument() {
    let source = "def main():\n    nums: list[int] = [1, 2, 3]\n    it: Iterator[int] = iter(nums)\n    _rev = reversed(it)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "reversed() argument must be reversible, got 'Iterator[int]'"
            && error.code == Some(DiagnosticCode::PROTO_BOUND_NOT_SATISFIED)
            && error.primary_range == Some(range_for_after_anchor(source, "reversed(", "it"))
    }));
}

#[test]
pub(super) fn test_reversed_and_enumerate_argument_errors_have_codes() {
    let reversed_source = "def main():\n    _rev = reversed(1)\n";
    let reversed_result = lower_source(reversed_source);
    assert!(reversed_result.is_err());
    let reversed_errors = reversed_result.unwrap_err();
    assert!(reversed_errors.iter().any(|error| {
        error.message
            == "reversed() argument must be an iterable with a statically-known element type, got 'int'"
            && error.code == Some(DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE)
            && error.primary_range == Some(range_for_after_anchor(reversed_source, "reversed(", "1"))
    }));

    let enumerate_source = "def main():\n    _items = enumerate(1)\n";
    let enumerate_result = lower_source(enumerate_source);
    assert!(enumerate_result.is_err());
    let enumerate_errors = enumerate_result.unwrap_err();
    assert!(enumerate_errors.iter().any(|error| {
        error.message
            == "enumerate() argument must be an iterable with a statically-known element type, got 'int'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for_after_anchor(enumerate_source, "enumerate(", "1"))
    }));
}

#[test]
pub(super) fn test_enumerate_start_type_errors_have_codes() {
    let positional_source =
        "def main():\n    nums: list[int] = [1, 2]\n    _items = enumerate(nums, \"bad\")\n";
    let positional_result = lower_source(positional_source);
    assert!(positional_result.is_err());
    let positional_errors = positional_result.unwrap_err();
    assert!(positional_errors.iter().any(|error| {
        error.message == "enumerate() start argument must be 'int', got 'str'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range
                == Some(range_for_after_anchor(
                    positional_source,
                    "enumerate(nums, ",
                    "\"bad\"",
                ))
    }));

    let keyword_source =
        "def main():\n    nums: list[int] = [1, 2]\n    _items = enumerate(nums, start=\"bad\")\n";
    let keyword_result = lower_source(keyword_source);
    assert!(keyword_result.is_err());
    let keyword_errors = keyword_result.unwrap_err();
    assert!(keyword_errors.iter().any(|error| {
        error.message == "enumerate() keyword argument 'start' must be 'int', got 'str'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range
                == Some(range_for_after_anchor(
                    keyword_source,
                    "enumerate(nums, start=",
                    "\"bad\"",
                ))
    }));
}

#[test]
pub(super) fn test_enumerate_arity_and_unpacked_keyword_errors_have_codes() {
    let arity_source =
        "def main():\n    nums: list[int] = [1, 2]\n    _items = enumerate(nums, 1, 2)\n";
    let arity_result = lower_source(arity_source);
    assert!(arity_result.is_err());
    let arity_errors = arity_result.unwrap_err();
    assert!(arity_errors.iter().any(|error| {
        error.message == "enumerate() takes 1 or 2 arguments"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range
                == Some(range_for_after_anchor(
                    arity_source,
                    "enumerate(nums, 1, ",
                    "2",
                ))
    }));

    let unpacked_source = "def main():\n    nums: list[int] = [1, 2]\n    kwargs: dict[str, int] = {\"start\": 1}\n    _items = enumerate(nums, **kwargs)\n";
    let unpacked_result = lower_source(unpacked_source);
    assert!(unpacked_result.is_err());
    let unpacked_errors = unpacked_result.unwrap_err();
    assert!(unpacked_errors.iter().any(|error| {
        error.message == "enumerate() does not support unpacked keyword arguments"
            && error.code == Some(DiagnosticCode::CALL_UNEXPECTED_KEYWORD)
            && error.primary_range
                == Some(range_for_after_anchor(
                    unpacked_source,
                    "enumerate(nums, ",
                    "**kwargs",
                ))
    }));
}

#[test]
pub(super) fn test_reversible_annotation_accepts_list_and_rejects_set() {
    let ok = lower_source(
        "def consume(xs: Reversible[int]) -> int:\n    rev: Iterator[int] = reversed(xs)\n    first: int | None = next(rev)\n    if first is None:\n        return 0\n    return first\n\ndef main():\n    nums: list[int] = [1, 2, 3]\n    consume(nums)\n",
    );
    assert!(ok.is_ok(), "{ok:?}");

    let err = lower_source(
        "def consume(xs: Reversible[int]) -> int:\n    rev: Iterator[int] = reversed(xs)\n    first: int | None = next(rev)\n    if first is None:\n        return 0\n    return first\n\ndef main():\n    nums: set[int] = {1, 2, 3}\n    consume(nums)\n",
    );
    assert!(err.is_err());
    let errors = err.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("expected 'Reversible[int]', got 'set[int]'")
    }));
}

#[test]
pub(super) fn test_comprehensions_accept_iterator_inputs() {
    let result = lower_source(
        "def main():\n    nums: list[int] = [1, 2, 3]\n    it_list: Iterator[int] = iter(nums)\n    list_comp: list[int] = [x for x in it_list]\n    it_set: Iterator[int] = iter(nums)\n    set_comp: set[int] = {x for x in it_set}\n    it_dict: Iterator[tuple[int, int]] = enumerate(nums)\n    dict_comp: dict[int, int] = {i: x for i, x in it_dict}\n",
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
pub(super) fn test_list_comprehension_invalid_target_has_flow_code() {
    let source = "def main():\n    values: list[int] = [1]\n    out: list[int] = [x for values[0] in values]\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "comprehension target must be a simple name or tuple"
            && error.code == Some(DiagnosticCode::FLOW_INVALID_ASSIGNMENT_TARGET)
            && error.primary_range == Some(range_for_after(source, "for ", "values[0]"))
    }));
}

#[test]
pub(super) fn test_list_comprehension_non_iterable_has_flow_code() {
    let source = "def main():\n    value: int = 1\n    out: list[int] = [x for x in value]\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "cannot iterate over type 'int'"
            && error.code == Some(DiagnosticCode::FLOW_INVALID_ITERATION)
            && error.primary_range == Some(range_for_after(source, "in ", "value"))
    }));
}

#[test]
pub(super) fn test_set_comprehension_invalid_target_has_flow_code() {
    let source = "def main():\n    values: list[int] = [1]\n    out: set[int] = {x for values[0] in values}\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "set comprehension target must be a simple name"
            && error.code == Some(DiagnosticCode::FLOW_INVALID_ASSIGNMENT_TARGET)
            && error.primary_range == Some(range_for_after(source, "for ", "values[0]"))
    }));
}

#[test]
pub(super) fn test_set_comprehension_non_iterable_has_flow_code() {
    let source = "def main():\n    value: int = 1\n    out: set[int] = {x for x in value}\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "cannot iterate over type 'int'"
            && error.code == Some(DiagnosticCode::FLOW_INVALID_ITERATION)
            && error.primary_range == Some(range_for_after(source, "in ", "value"))
    }));
}

#[test]
pub(super) fn test_dict_comprehension_invalid_tuple_target_has_flow_code() {
    let source = "def main():\n    values: list[int] = [0]\n    pairs: list[tuple[int, int]] = [(1, 2)]\n    out: dict[int, int] = {left: right for (left, values[0]) in pairs}\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "dict comprehension tuple target must contain only simple names"
            && error.code == Some(DiagnosticCode::FLOW_INVALID_ASSIGNMENT_TARGET)
            && error.primary_range == Some(range_for(source, "(left, values[0])"))
    }));
}

#[test]
pub(super) fn test_dict_comprehension_non_iterable_has_flow_code() {
    let source =
        "def main():\n    value: int = 1\n    out: dict[int, int] = {x: x for x in value}\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "cannot iterate over type 'int'"
            && error.code == Some(DiagnosticCode::FLOW_INVALID_ITERATION)
            && error.primary_range == Some(range_for_after(source, "in ", "value"))
    }));
}

#[test]
pub(super) fn test_dict_unpacking_comprehension_is_rejected_without_a_panic() {
    let source = "def main():\n    items: list[dict[int, int]] = [{1: 2}]\n    out: dict[int, int] = {**item for item in items}\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "dictionary unpacking comprehensions are not supported in Sifr"
            && error.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM)
            && error.primary_range == Some(range_for(source, "{**item for item in items}"))
    }));
}

#[test]
pub(super) fn test_generator_expression_multi_generator_has_type_code() {
    let source = "def main():\n    xs: list[int] = [1]\n    ys: list[int] = [2]\n    out: Iterator[int] = (x for x in xs for y in ys)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "only single-generator generator expressions are supported"
            && error.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM)
            && error.primary_range == Some(range_for(source, "(x for x in xs for y in ys)"))
    }));
}

#[test]
pub(super) fn test_generator_expression_invalid_target_has_flow_code() {
    let source = "def main():\n    values: list[int] = [1]\n    out: Iterator[int] = (x for values[0] in values)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "generator target must be a simple name"
            && error.code == Some(DiagnosticCode::FLOW_INVALID_ASSIGNMENT_TARGET)
            && error.primary_range == Some(range_for_after(source, "for ", "values[0]"))
    }));
}

#[test]
pub(super) fn test_generator_expression_non_iterable_has_flow_code() {
    let source = "def main():\n    value: int = 1\n    out: Iterator[int] = (x for x in value)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "cannot iterate over type 'int'"
            && error.code == Some(DiagnosticCode::FLOW_INVALID_ITERATION)
            && error.primary_range == Some(range_for_after(source, "in ", "value"))
    }));
}

#[test]
pub(super) fn test_walrus_invalid_target_has_flow_code() {
    let target_range = TextRange::new(TextSize::new(10), TextSize::new(14));
    let value_range = TextRange::new(TextSize::new(18), TextSize::new(22));
    let named = ExprNamed {
        node_index: AtomicNodeIndex::NONE,
        range: TextRange::new(TextSize::new(10), TextSize::new(22)),
        target: Box::new(Expr::NoneLiteral(ExprNoneLiteral {
            node_index: AtomicNodeIndex::NONE,
            range: target_range,
        })),
        value: Box::new(Expr::NoneLiteral(ExprNoneLiteral {
            node_index: AtomicNodeIndex::NONE,
            range: value_range,
        })),
    };
    let mut ctx = LowerCtx::new();

    let result = lower_named_expr(&named, &mut ctx);

    assert!(result.is_none());
    assert!(ctx.errors.iter().any(|error| {
        error.message == "walrus operator target must be a simple name"
            && error.code == Some(DiagnosticCode::FLOW_INVALID_ASSIGNMENT_TARGET)
            && error.primary_range == Some(target_range)
    }));
}

#[test]
pub(super) fn test_map_is_typed_as_iterator() {
    let module = lower_source(
        "def add(x: int, y: int) -> int:\n    return x + y\n\ndef main():\n    left: list[int] = [1, 2]\n    right: list[int] = [3, 4]\n    mapped: Iterator[int] = map(add, left, right)\n    _vals: list[int] = list(mapped)\n",
    )
    .unwrap();
    let main_fn = module
        .functions
        .iter()
        .find(|func| func.name == "main")
        .expect("main function should exist");
    let Some(HirStmt::Let { ty, .. }) = main_fn
        .body
        .iter()
        .find(|stmt| matches!(stmt, HirStmt::Let { name, .. } if name == "mapped"))
    else {
        panic!("expected let binding for mapped");
    };
    assert!(matches!(ty, Type::Iterator(_)));
}

#[test]
pub(super) fn test_map_rejects_plain_list_annotation_without_materialization() {
    let result = lower_source(
        "def add(x: int, y: int) -> int:\n    return x + y\n\ndef main():\n    values: list[int] = map(add, [1, 2], [3, 4])\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("expected 'list[int]', got 'Iterator[int]'")
    }));
}

#[test]
pub(super) fn test_map_rejects_keywords_with_stable_diagnostic() {
    let source = "def add(x: int) -> int:\n    return x + 1\n\ndef main():\n    nums: list[int] = [1, 2]\n    _mapped = map(function=add, iterable=nums)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "map() does not accept keyword arguments"
            && error.code == Some(DiagnosticCode::CALL_UNEXPECTED_KEYWORD)
            && error.primary_range == Some(range_for_after_anchor(source, "map(", "function=add"))
    }));
}

#[test]
pub(super) fn test_map_argument_errors_have_codes() {
    let missing_source =
        "def inc(x: int) -> int:\n    return x + 1\n\ndef main():\n    _mapped = map(inc)\n";
    let missing_result = lower_source(missing_source);
    assert!(missing_result.is_err());
    let missing_errors = missing_result.unwrap_err();
    assert!(missing_errors.iter().any(|error| {
        error.message == "map() takes a callable followed by at least one iterable"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range == Some(range_for_after_anchor(missing_source, "map(", "inc"))
    }));

    let iterable_source =
        "def inc(x: int) -> int:\n    return x + 1\n\ndef main():\n    _mapped = map(inc, 1)\n";
    let iterable_result = lower_source(iterable_source);
    assert!(iterable_result.is_err());
    let iterable_errors = iterable_result.unwrap_err();
    assert!(iterable_errors.iter().any(|error| {
        error.message
            == "map() iterable arguments must have statically-known element types, got 'int'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range
                == Some(range_for_after_anchor(iterable_source, "map(inc, ", "1"))
    }));

    let callable_source = "def main():\n    nums: list[int] = [1, 2]\n    _mapped = map(1, nums)\n";
    let callable_result = lower_source(callable_source);
    assert!(callable_result.is_err());
    let callable_errors = callable_result.unwrap_err();
    assert!(callable_errors.iter().any(|error| {
        error.message == "map() first argument must be callable"
            && error.code == Some(DiagnosticCode::CALL_NOT_CALLABLE_OR_ARITY)
            && error.primary_range == Some(range_for_after_anchor(callable_source, "map(", "1"))
    }));
}

#[test]
pub(super) fn test_filter_is_typed_as_iterator() {
    let module = lower_source(
        "def pred(x: int) -> bool:\n    return x % 2 == 0\n\ndef main():\n    nums: list[int] = [1, 2, 3, 4]\n    filtered: Iterator[int] = filter(pred, nums)\n    _vals: list[int] = list(filtered)\n",
    )
    .unwrap();
    let main_fn = module
        .functions
        .iter()
        .find(|func| func.name == "main")
        .expect("main function should exist");
    let Some(HirStmt::Let { ty, .. }) = main_fn
        .body
        .iter()
        .find(|stmt| matches!(stmt, HirStmt::Let { name, .. } if name == "filtered"))
    else {
        panic!("expected let binding for filtered");
    };
    assert!(matches!(ty, Type::Iterator(_)));
}

#[test]
pub(super) fn test_filter_rejects_plain_list_annotation_without_materialization() {
    let result = lower_source(
        "def pred(x: int) -> bool:\n    return x % 2 == 0\n\ndef main():\n    values: list[int] = filter(pred, [1, 2, 3, 4])\n",
    );
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("expected 'list[int]', got 'Iterator[int]'")
    }));
}

#[test]
pub(super) fn test_filter_rejects_keywords_with_stable_diagnostic() {
    let source = "def pred(x: int) -> bool:\n    return x > 0\n\ndef main():\n    nums: list[int] = [1, 2]\n    _filtered = filter(function=pred, iterable=nums)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "filter() does not accept keyword arguments"
            && error.code == Some(DiagnosticCode::CALL_UNEXPECTED_KEYWORD)
            && error.primary_range
                == Some(range_for_after_anchor(source, "filter(", "function=pred"))
    }));
}

#[test]
pub(super) fn test_filter_argument_errors_have_codes() {
    let arity_source = "def pred(x: int) -> bool:\n    return x > 0\n\ndef main():\n    _filtered = filter(pred)\n";
    let arity_result = lower_source(arity_source);
    assert!(arity_result.is_err());
    let arity_errors = arity_result.unwrap_err();
    assert!(arity_errors.iter().any(|error| {
        error.message == "filter() takes exactly 2 arguments (function, iterable)"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range == Some(range_for_after_anchor(arity_source, "filter(", "pred"))
    }));

    let iterable_source = "def pred(x: int) -> bool:\n    return x > 0\n\ndef main():\n    _filtered = filter(pred, 1)\n";
    let iterable_result = lower_source(iterable_source);
    assert!(iterable_result.is_err());
    let iterable_errors = iterable_result.unwrap_err();
    assert!(iterable_errors.iter().any(|error| {
        error.message
            == "filter() argument must be an iterable with a statically-known element type, got 'int'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range
                == Some(range_for_after_anchor(iterable_source, "filter(pred, ", "1"))
    }));

    let callable_source =
        "def main():\n    nums: list[int] = [1, 2]\n    _filtered = filter(1, nums)\n";
    let callable_result = lower_source(callable_source);
    assert!(callable_result.is_err());
    let callable_errors = callable_result.unwrap_err();
    assert!(callable_errors.iter().any(|error| {
        error.message == "filter() first argument must be callable"
            && error.code == Some(DiagnosticCode::CALL_NOT_CALLABLE_OR_ARITY)
            && error.primary_range == Some(range_for_after_anchor(callable_source, "filter(", "1"))
    }));

    let return_source = "def ident(x: int) -> int:\n    return x\n\ndef main():\n    nums: list[int] = [1, 2]\n    _filtered = filter(ident, nums)\n";
    let return_result = lower_source(return_source);
    assert!(return_result.is_err());
    let return_errors = return_result.unwrap_err();
    assert!(return_errors.iter().any(|error| {
        error.message == "filter() callable must return 'bool', got 'int'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range
                == Some(range_for_after_anchor(return_source, "filter(", "ident"))
    }));
}

#[test]
pub(super) fn test_sum_min_max_accept_iterator_inputs() {
    let result = lower_source(
        "def main():\n    nums: list[int] = [3, 1, 2]\n    total: int = sum(iter(nums))\n    lo: int | None = min(iter(nums))\n    hi: int | None = max(iter(nums))\n",
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
pub(super) fn test_min_max_accept_variadic_scalar_inputs() {
    let result = lower_source(
        "def main() -> int:\n    lo: int = min(3, 1, 2)\n    hi: int = max(1, 5, 2, 4)\n    return lo + hi\n",
    );
    assert!(result.is_ok(), "{result:?}");
}
