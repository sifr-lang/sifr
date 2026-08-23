use super::*;
use crate::lower::{LowerCtx, expressions::resolve_method_type};
#[test]
pub(super) fn test_min_max_missing_args_have_call_code() {
    let cases = [
        ("min", "min() takes at least 1 argument"),
        ("max", "max() takes at least 1 argument"),
    ];

    for (callable, message) in cases {
        let source = format!("def main():\n    _value = {callable}()\n");
        let result = lower_source(&source);
        assert!(
            result.is_err(),
            "{callable} should reject missing arguments"
        );
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
pub(super) fn test_min_max_keywords_have_call_code() {
    for callable in ["min", "max"] {
        let source = format!(
            "def main():\n    values: list[int] = [1, 2]\n    _value = {callable}(values=values)\n"
        );
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
                            "values=values",
                        ))
            }),
            "{callable} errors: {errors:?}"
        );
    }
}

#[test]
pub(super) fn test_min_max_single_non_iterable_has_type_code() {
    let cases = [
        (
            "min",
            "min() argument must be an iterable with a statically-known element type, got 'int'",
        ),
        (
            "max",
            "max() argument must be an iterable with a statically-known element type, got 'int'",
        ),
    ];

    for (callable, message) in cases {
        let source = format!("def main():\n    _value = {callable}(1)\n");
        let result = lower_source(&source);
        assert!(
            result.is_err(),
            "{callable} should reject single non-iterable argument"
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
                            "1",
                        ))
            }),
            "{callable} errors: {errors:?}"
        );
    }
}

#[test]
pub(super) fn test_max_two_arg_rejects_optional_operand() {
    let source = "def pick(d: dict[str, int], k: str) -> int:\n    best = 0\n    best = max(best, d[k])\n    return best\n";
    let result = lower_source(source);
    assert!(result.is_err(), "max(i64, i64|None) should be rejected");
    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                && error
                    .message
                    .contains("max() with 2 arguments does not accept optional operands")
                && error.primary_range == Some(range_for(source, "d[k]"))),
        "max optional operand diagnostic should be structured and ranged: {errors:?}"
    );
}

#[test]
pub(super) fn test_min_max_incompatible_operands_have_type_codes() {
    let source = "def main() -> None:\n    lo = min(1, \"x\")\n";
    let result = lower_source(source);
    let errors = result.expect_err("expected min incompatible operand error");

    assert!(
        errors
            .iter()
            .any(|error| error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                && error.message
                    == "min() arguments must be comparable and type-compatible; got 'int' and 'str'"
                && error.primary_range == Some(range_for(source, "\"x\""))),
        "min incompatible operand diagnostic should be structured and ranged: {errors:?}"
    );
}

#[test]
pub(super) fn test_sorted_accepts_iterable_keyword_and_key_none() {
    let result = lower_source(
        "def main():\n    nums: list[int] = [3, 1, 2]\n    ordered: list[int] = sorted(iterable=nums, key=None, reverse=True)\n    assert ordered == [3, 2, 1]\n",
    );
    assert!(result.is_ok());
}

#[test]
pub(super) fn test_sorted_rejects_non_total_order_elements_and_keys() {
    let sources = [
        "class Item:\n    value: int\n\ndef order(values: list[Item]) -> list[Item]:\n    return sorted(values)\n",
        "class Item:\n    value: int\n\ndef item_key(value: int) -> Item:\n    return Item(value)\n\ndef order(values: list[int]) -> list[int]:\n    return sorted(values, key=item_key)\n",
        "class Item:\n    value: int\n\ndef order(values: list[int]) -> list[int]:\n    return sorted(values, key=lambda value: Item(value))\n",
    ];
    for source in sources {
        let errors = lower_source(source).expect_err("non-Ord sorted input should fail");
        assert!(
            errors.iter().any(|error| {
                error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                    && error.message.contains("generated Rust total Ord support")
            }),
            "{source}: {errors:?}"
        );
    }
}

#[test]
pub(super) fn test_sorted_aligns_non_clone_elements_with_source_and_key_ownership() {
    let borrowed_temporary = lower_source(
        "class Local(NonSend):\n    pass\n\ndef key(value: Local) -> int:\n    return 0\n\ndef order() -> list[Local]:\n    return sorted([Local(), Local()], key=key)\n",
    );
    assert!(borrowed_temporary.is_ok(), "{borrowed_temporary:?}");

    for source in [
        "class Local(NonSend):\n    pass\n\ndef key(value: Local) -> int:\n    return 0\n\ndef order(values: list[Local]) -> list[Local]:\n    return sorted(values, key=key)\n",
        "class Local(NonSend):\n    pass\n\ndef key(own value: Local) -> int:\n    return 0\n\ndef order() -> list[Local]:\n    return sorted([Local(), Local()], key=key)\n",
    ] {
        let errors = lower_source(source).expect_err("non-Clone sorted ownership should fail");
        assert!(
            errors.iter().any(|error| {
                error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                    && error.message.contains("Clone-capable")
            }),
            "{source}: {errors:?}"
        );
    }
}

#[test]
pub(super) fn test_sum_keyword_and_type_errors_have_codes() {
    let keyword_source =
        "def main():\n    nums: list[int] = [1, 2]\n    _total = sum(values=nums)\n";
    let keyword_result = lower_source(keyword_source);
    assert!(keyword_result.is_err());
    let keyword_errors = keyword_result.unwrap_err();
    assert!(keyword_errors.iter().any(|error| {
        error.message == "sum() does not accept keyword arguments"
            && error.code == Some(DiagnosticCode::CALL_UNEXPECTED_KEYWORD)
            && error.primary_range
                == Some(range_for_after_anchor(
                    keyword_source,
                    "sum(",
                    "values=nums",
                ))
    }));

    let type_source = "def main():\n    _total = sum(1)\n";
    let type_result = lower_source(type_source);
    assert!(type_result.is_err());
    let type_errors = type_result.unwrap_err();
    assert!(type_errors.iter().any(|error| {
        error.message
            == "sum() argument must be an iterable with a statically-known element type, got 'int'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for_after_anchor(type_source, "sum(", "1"))
    }));
}

#[test]
pub(super) fn test_sum_fixed_width_iterable_widens_to_int() {
    let module = lower_source(
        "def main():\n    left: int32 = 2000000000\n    right: int32 = 2000000000\n    values: list[int32] = [left, right]\n    total: int = sum(values)\n",
    )
    .expect("fixed-width sum should widen to int");
    let main_fn = module
        .functions
        .iter()
        .find(|func| func.name == "main")
        .expect("main should lower");
    let HirStmt::Let { ty, .. } = &main_fn.body[3] else {
        panic!("expected total let");
    };
    assert_eq!(ty, &Type::Int);
}

#[test]
pub(super) fn test_sorted_positional_and_duplicate_errors_have_codes() {
    let too_many_source =
        "def main():\n    nums: list[int] = [3, 1, 2]\n    ordered = sorted(nums, nums)\n";
    let too_many_result = lower_source(too_many_source);
    assert!(too_many_result.is_err());
    let too_many_errors = too_many_result.unwrap_err();
    assert!(too_many_errors.iter().any(|error| {
        error.message == "sorted() takes at most 1 positional argument"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range
                == Some(range_for_after_anchor(
                    too_many_source,
                    "sorted(nums, ",
                    "nums",
                ))
    }));
}

#[test]
pub(super) fn test_sorted_rejects_duplicate_iterable_argument() {
    let source = "def main():\n    nums: list[int] = [3, 1, 2]\n    ordered: list[int] = sorted(nums, iterable=nums)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "sorted() got multiple values for argument 'iterable'"
            && error.code == Some(DiagnosticCode::CALL_DUPLICATE_ARGUMENT)
            && error.primary_range
                == Some(range_for_after_anchor(
                    source,
                    "sorted(nums, ",
                    "iterable=nums",
                ))
    }));
}

#[test]
pub(super) fn test_sorted_type_and_key_errors_have_codes() {
    let iterable_source = "def main():\n    ordered = sorted(1)\n";
    let iterable_result = lower_source(iterable_source);
    assert!(iterable_result.is_err());
    let iterable_errors = iterable_result.unwrap_err();
    assert!(iterable_errors.iter().any(|error| {
        error.message
            == "sorted() argument must be an iterable with a statically-known element type, got 'int'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for_after_anchor(iterable_source, "sorted(", "1"))
    }));

    let key_source =
        "def main():\n    nums: list[int] = [3, 1, 2]\n    ordered = sorted(nums, key=1)\n";
    let key_result = lower_source(key_source);
    assert!(key_result.is_err());
    let key_errors = key_result.unwrap_err();
    assert!(key_errors.iter().any(|error| {
        error.message == "sorted() keyword argument 'key' must be callable"
            && error.code == Some(DiagnosticCode::CALL_NOT_CALLABLE_OR_ARITY)
            && error.primary_range == Some(range_for_after_anchor(key_source, "key=", "1"))
    }));

    let reverse_source =
        "def main():\n    nums: list[int] = [3, 1, 2]\n    ordered = sorted(nums, reverse=1)\n";
    let reverse_result = lower_source(reverse_source);
    assert!(reverse_result.is_err());
    let reverse_errors = reverse_result.unwrap_err();
    assert!(reverse_errors.iter().any(|error| {
        error.message == "sorted() keyword argument 'reverse' must be 'bool', got 'int'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for_after_anchor(reverse_source, "reverse=", "1"))
    }));
}

#[test]
pub(super) fn test_list_sort_accepts_reverse_keyword() {
    let result =
        lower_source("def main():\n    nums: list[int] = [3, 1, 2]\n    nums.sort(reverse=True)\n");
    assert!(result.is_ok(), "{result:?}");
}

#[test]
pub(super) fn test_list_sort_rejects_non_bool_reverse_keyword() {
    let source = "def main():\n    nums: list[int] = [3, 1, 2]\n    nums.sort(reverse=1)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("list.sort() argument 'reverse' must be 'bool'")
            && e.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && e.primary_range == Some(range_for_after_anchor(source, "reverse=", "1"))
    }));
}

#[test]
pub(super) fn test_tuple_constructor_rejects_dynamic_list_shape() {
    let source = "def main():\n    nums: list[int] = [1, 2, 3]\n    t = tuple(nums)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| {
        e.message
            .contains("tuple() currently requires a tuple, list literal, or string literal")
            && e.code == Some(DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE)
            && e.primary_range == Some(range_for_after_anchor(source, "tuple(", "nums"))
    }));
}

#[test]
pub(super) fn test_list_pop_index_and_tuple_index_optional_forms_lower() {
    let result = lower_source(
        "def main():\n    xs: list[int] = [1, 2, 3, 2]\n    popped: int | None = xs.pop(0)\n    idx: int | None = xs.index(2, start=0, stop=3)\n    pair: tuple[int, int, int] = (4, 5, 4)\n    tidx: int | None = pair.index(4, start=1)\n",
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
pub(super) fn test_index_stop_only_keyword_forms_lower() {
    let result = lower_source(
        "def main():\n    xs: list[int] = [1, 2, 3, 2]\n    list_idx: int | None = xs.index(2, stop=3)\n    pair: tuple[int, int, int] = (4, 5, 4)\n    tuple_idx: int | None = pair.index(4, stop=2)\n",
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
pub(super) fn test_index_optional_keyword_duplicate_forms_are_rejected() {
    let list_result =
        lower_source("def main():\n    xs: list[int] = [1, 2, 3]\n    xs.index(2, 0, start=1)\n");
    assert!(list_result.is_err());
    let list_errors = list_result.unwrap_err();
    assert!(list_errors.iter().any(|e| {
        e.message
            .contains("index() got multiple values for argument 'start'")
    }));

    let tuple_result = lower_source(
        "def main():\n    pair: tuple[int, int, int] = (1, 2, 3)\n    pair.index(2, 0, 2, stop=3)\n",
    );
    assert!(tuple_result.is_err());
    let tuple_errors = tuple_result.unwrap_err();
    assert!(tuple_errors.iter().any(|e| {
        e.message
            .contains("index() got multiple values for argument 'stop'")
    }));
}

#[test]
pub(super) fn test_dict_update_kwargs_and_pop_default_lower() {
    let result = lower_source(
        "def main():\n    data: dict[str, int] = {\"x\": 1}\n    data.update(a=2)\n    other: dict[str, int] = {\"b\": 3}\n    data.update(other, c=4)\n    fallback: int = data.pop(\"missing\", default=9)\n",
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
pub(super) fn test_string_split_and_replace_keyword_forms_lower() {
    let result = lower_source(
        "def main():\n    parts: list[str] = \"a,b,c\".split(sep=\",\", maxsplit=1)\n    replaced: str = \"aaaa\".replace(\"a\", \"b\", count=2)\n",
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
pub(super) fn test_unexpected_method_keyword_is_rejected() {
    let source = "def main():\n    xs: list[int] = [1]\n    xs.append(value=2)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "append() got an unexpected keyword argument 'value'"
            && error.code == Some(DiagnosticCode::CALL_UNEXPECTED_KEYWORD)
            && error.primary_range == Some(range_for(source, "value"))
    }));
}

#[test]
pub(super) fn test_unpacked_method_keyword_has_call_code() {
    let source = "def main():\n    xs: list[int] = [1]\n    xs.append(**{\"value\": 2})\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "append() does not support unpacked keyword arguments"
            && error.code == Some(DiagnosticCode::CALL_UNEXPECTED_KEYWORD)
            && error.primary_range == Some(range_for(source, "**{\"value\": 2}"))
    }));
}

#[test]
pub(super) fn test_list_extend_non_iterable_has_protocol_code() {
    let source = "def main():\n    xs: list[int] = []\n    xs.extend(1)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message
            == "list.extend() argument must be an iterable with a statically-known element type, got 'int'"
            && error.code == Some(DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE)
            && error.primary_range == Some(range_for(source, "1"))
    }));
}

#[test]
pub(super) fn test_list_method_wrong_positional_count_has_call_code() {
    let source = "def main():\n    xs: list[int] = []\n    xs.append(1, 2)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "list.append() takes exactly 1 argument, got 2"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range == Some(range_for_after_anchor(source, "xs.append(1, ", "2"))
    }));
}

#[test]
pub(super) fn test_list_method_type_mismatch_has_type_code() {
    let source = "def main():\n    xs: list[int] = [1]\n    xs.pop(\"0\")\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "list.pop() index must be 'int', got 'str'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for(source, "\"0\""))
    }));
}

#[test]
pub(super) fn test_list_missing_method_has_stdlib_code() {
    let source = "def main():\n    xs: list[int] = []\n    xs.missing()\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "list has no method 'missing'"
            && error.code == Some(DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE)
            && error.primary_range == Some(range_for_after(source, "xs.", "missing"))
    }));
}

#[test]
pub(super) fn test_dict_update_keyword_value_mismatch_has_type_code() {
    let source = "def main():\n    data: dict[str, int] = {}\n    data.update(bad=\"x\")\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message
            == "dict.update() value type 'str' is not compatible with dict value type 'int'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for(source, "bad=\"x\""))
    }));
}

#[test]
pub(super) fn test_dict_method_wrong_positional_count_has_call_code() {
    let source = "def main():\n    data: dict[str, int] = {}\n    data.clear(1)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "dict.clear() takes no arguments"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range == Some(range_for(source, "1"))
    }));
}

#[test]
pub(super) fn test_dict_method_type_mismatch_has_type_code() {
    let source = "def main():\n    data: dict[str, int] = {\"x\": 1}\n    data.get(1)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "dict.get() key type 'int' is not compatible with dict key type 'str'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for_after(source, "data.get(", "1"))
    }));
}

#[test]
pub(super) fn test_dict_get_default_keyword_type_mismatch_has_type_code_and_range() {
    let source = "def main():\n    data: dict[int, int] = {0: 1}\n    value = data.get(0, default=\"bad\")\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message
            == "dict.get() default type 'str' is not compatible with dict value type 'int'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for_after_anchor(source, "default=", "\"bad\""))
    }));
}

#[test]
pub(super) fn test_dict_pop_default_keyword_type_mismatch_has_type_code_and_range() {
    let source = "def main():\n    data: dict[int, int] = {0: 1}\n    value = data.pop(0, default=\"bad\")\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message
            == "dict.pop() default type 'str' is not compatible with dict value type 'int'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for_after_anchor(source, "default=", "\"bad\""))
    }));
}

#[test]
pub(super) fn test_dict_setdefault_keyword_type_mismatch_has_type_code_and_range() {
    let source = "def main():\n    data: dict[int, int] = {0: 1}\n    value = data.setdefault(0, default=\"bad\")\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message
            == "dict.setdefault() default type 'str' is not compatible with dict value type 'int'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for_after_anchor(source, "default=", "\"bad\""))
    }));
}

#[test]
pub(super) fn test_dict_missing_method_has_stdlib_code() {
    let source = "def main():\n    data: dict[str, int] = {}\n    data.missing()\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "dict has no method 'missing'"
            && error.code == Some(DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE)
            && error.primary_range == Some(range_for_after(source, "data.", "missing"))
    }));
}

#[test]
pub(super) fn test_set_method_wrong_positional_count_has_call_code() {
    let source = "def main():\n    values: set[int] = {1}\n    values.add(1, 2)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "set.add() takes exactly 1 argument, got 2"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range == Some(range_for_after_anchor(source, "values.add(1, ", "2"))
    }));
}

#[test]
pub(super) fn test_set_missing_method_has_stdlib_code() {
    let source = "def main():\n    values: set[int] = {1}\n    values.missing()\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "set has no method 'missing'"
            && error.code == Some(DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE)
            && error.primary_range == Some(range_for_after(source, "values.", "missing"))
    }));
}

#[test]
pub(super) fn test_str_method_wrong_positional_count_has_call_code() {
    let source = "def main():\n    text: str = \"abc\"\n    text.find(\"a\", 1)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "str.find() takes exactly 1 argument, got 2"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range == Some(range_for_after_anchor(source, "text.find(\"a\", ", "1"))
    }));
}

#[test]
pub(super) fn test_str_rfind_lowers_to_optional_int() {
    let source =
        "def main():\n    text: str = \"abcabc\"\n    index: int | None = text.rfind(\"a\")\n";
    lower_source(source).expect("str.rfind should lower");
}

#[test]
pub(super) fn test_str_method_type_mismatch_has_type_code() {
    let source = "def main():\n    text: str = \"a,b\"\n    text.split(\",\", \"bad\")\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "str.split() maxsplit must be 'int', got 'str'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for(source, "\"bad\""))
    }));
}

#[test]
pub(super) fn test_str_replace_keyword_count_type_mismatch_has_type_code() {
    let source =
        "def main():\n    text: str = \"aaaa\"\n    text.replace(\"a\", \"b\", count=\"bad\")\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "str.replace() count must be 'int', got 'str'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for(source, "\"bad\""))
    }));
}

#[test]
pub(super) fn test_str_missing_method_has_stdlib_code() {
    let source = "def main():\n    text: str = \"abc\"\n    text.missing()\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "str has no method 'missing'"
            && error.code == Some(DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE)
            && error.primary_range == Some(range_for_after(source, "text.", "missing"))
    }));
}

#[test]
pub(super) fn test_tuple_method_wrong_positional_count_has_call_code() {
    let source = "def main():\n    pair: tuple[int, int] = (1, 2)\n    pair.count(1, 2)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "tuple.count() takes exactly 1 argument, got 2"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range == Some(range_for_after_anchor(source, "pair.count(1, ", "2"))
    }));
}

#[test]
pub(super) fn test_tuple_method_type_mismatch_has_type_code() {
    let source =
        "def main():\n    pair: tuple[int, int, int] = (1, 2, 3)\n    pair.index(1, \"bad\")\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "tuple.index() bounds must be 'int', got 'str'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for(source, "\"bad\""))
    }));
}

#[test]
pub(super) fn test_tuple_missing_method_has_stdlib_code() {
    let source = "def main():\n    pair: tuple[int, int] = (1, 2)\n    pair.missing()\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "tuple has no method 'missing'"
            && error.code == Some(DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE)
            && error.primary_range == Some(range_for_after(source, "pair.", "missing"))
    }));
}

#[test]
pub(super) fn test_class_method_argument_type_has_type_code() {
    let source = "class Box:\n    def take(self, value: int) -> None:\n        pass\n\ndef main():\n    box: Box = Box()\n    box.take(\"bad\")\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "argument 1 ('value') of Box.take(): expected 'int', got 'str'"
            && error.code == Some(DiagnosticCode::TYPE_MISMATCH)
            && error.primary_range == Some(range_for(source, "\"bad\""))
    }));
}

#[test]
pub(super) fn test_callable_field_wrong_arity_has_call_code() {
    let source = "class Runner:\n    callback: Callable[[int], int]\n\n    def __init__(self, callback: Callable[[int], int]):\n        self.callback = callback\n\ndef double(x: int) -> int:\n    return x * 2\n\ndef main():\n    runner: Runner = Runner(double)\n    runner.callback()\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "Runner.callback() (callable field) takes 1 argument(s), got 0"
            && error.code == Some(DiagnosticCode::CALL_NOT_CALLABLE_OR_ARITY)
            && error.primary_range == Some(range_for_after(source, "runner.", "callback"))
    }));
}

#[test]
pub(super) fn test_class_field_not_callable_has_call_code() {
    let source = "class Box:\n    value: int\n\n    def __init__(self, value: int):\n        self.value = value\n\ndef main():\n    box: Box = Box(1)\n    box.value()\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "field 'value' of class 'Box' is not callable (type: 'int')"
            && error.code == Some(DiagnosticCode::CALL_NOT_CALLABLE_OR_ARITY)
            && error.primary_range == Some(range_for_after(source, "box.", "value"))
    }));
}

#[test]
pub(super) fn test_class_missing_method_has_class_code() {
    let source = "class Box:\n    value: int\n\n    def __init__(self, value: int):\n        self.value = value\n\ndef main():\n    box: Box = Box(1)\n    box.missing()\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "class 'Box' has no method 'missing'"
            && error.code == Some(DiagnosticCode::CLASS_MISSING_MEMBER)
            && error.primary_range == Some(range_for_after(source, "box.", "missing"))
    }));
}

#[test]
pub(super) fn test_protocol_method_wrong_arity_has_call_code() {
    let protocol_ty = Type::Protocol {
        identity: None,
        name: "Runner".to_string(),
        methods: vec![(
            "run".to_string(),
            FunctionType::new(vec![("value".to_string(), Type::Int)], Type::Str),
        )],
    };
    let mut ctx = LowerCtx::new();
    let method_range = TextRange::new(TextSize::new(10), TextSize::new(13));

    let result = resolve_method_type(&protocol_ty, "run", &[], &[], method_range, &mut ctx);

    assert!(result.is_none());
    assert!(ctx.errors.iter().any(|error| {
        error.message == "Runner.run() takes 1 argument(s), got 0"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range == Some(method_range)
    }));
}

#[test]
pub(super) fn test_protocol_missing_method_has_protocol_code() {
    let protocol_ty = Type::Protocol {
        identity: None,
        name: "Runner".to_string(),
        methods: vec![(
            "run".to_string(),
            FunctionType::new(vec![("value".to_string(), Type::Int)], Type::Str),
        )],
    };
    let mut ctx = LowerCtx::new();
    let method_range = TextRange::new(TextSize::new(20), TextSize::new(27));

    let result = resolve_method_type(&protocol_ty, "missing", &[], &[], method_range, &mut ctx);

    assert!(result.is_none());
    assert!(ctx.errors.iter().any(|error| {
        error.message == "protocol 'Runner' has no method 'missing'"
            && error.code == Some(DiagnosticCode::PROTO_BOUND_NOT_SATISFIED)
            && error.primary_range == Some(method_range)
    }));
}

#[test]
pub(super) fn test_newtype_value_wrong_arity_has_call_code() {
    let source = "class Port(int):\n    pass\n\ndef main():\n    port: Port = Port(8080)\n    port.value(1)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "Port.value() takes no arguments"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range == Some(range_for_after(source, "port.value(", "1"))
    }));
}

#[test]
pub(super) fn test_enum_value_wrong_arity_has_call_code() {
    let source = "from enum import Enum\n\nclass Status(Enum):\n    OK = 200\n\ndef main():\n    status: Status = Status.OK\n    status.value(1)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "Status.value() takes no arguments"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range == Some(range_for_after(source, "status.value(", "1"))
    }));
}

#[test]
pub(super) fn test_enum_missing_method_has_class_code() {
    let source = "from enum import Enum\n\nclass Status(Enum):\n    OK = 200\n\ndef main():\n    status: Status = Status.OK\n    status.missing()\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "enum 'Status' has no method 'missing'"
            && error.code == Some(DiagnosticCode::CLASS_MISSING_MEMBER)
            && error.primary_range == Some(range_for_after(source, "status.", "missing"))
    }));
}

#[test]
pub(super) fn test_generic_type_missing_method_has_stdlib_code() {
    let source = "def use_value[T](value: T):\n    value.missing()\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "type 'T' has no method 'missing'"
            && error.code == Some(DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE)
            && error.primary_range == Some(range_for_after(source, "value.", "missing"))
    }));
}
