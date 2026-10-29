use super::*;

#[test]
pub(super) fn test_builtin_sum_wrong_arity_has_call_code() {
    let source = "def main():\n    data: list[int] = [1, 2, 3]\n    print(sum(data, data))\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "sum() takes exactly 1 argument(s), got 2"
            && error.code == Some(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT)
            && error.primary_range == Some(range_for_after_anchor(source, "sum(data, ", "data"))
    }));
}

#[test]
pub(super) fn test_sorted_unexpected_keyword_has_call_code() {
    let source = "def main():\n    nums: list[int] = [3, 1, 2]\n    ordered: list[int] = sorted(nums, bogus=True)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "sorted() got an unexpected keyword argument 'bogus'"
            && error.code == Some(DiagnosticCode::CALL_UNEXPECTED_KEYWORD)
            && error.primary_range == Some(range_for_after_anchor(source, "sorted(nums, ", "bogus"))
    }));
}

#[test]
pub(super) fn test_sorted_and_range_missing_required_argument_have_call_code() {
    let sorted_source = "def main():\n    values: list[int] = sorted()\n";
    let sorted_result = lower_source(sorted_source);
    assert!(sorted_result.is_err());
    let sorted_errors = sorted_result.unwrap_err();
    assert!(sorted_errors.iter().any(|error| {
        error.message == "sorted() missing required argument 'iterable'"
            && error.code == Some(DiagnosticCode::CALL_MISSING_REQUIRED_ARGUMENT)
            && error.primary_range == Some(range_for(sorted_source, "sorted"))
    }));

    let range_source = "def main():\n    values: list[int] = list(range())\n";
    let range_result = lower_source(range_source);
    assert!(range_result.is_err());
    let range_errors = range_result.unwrap_err();
    assert!(range_errors.iter().any(|error| {
        error.message == "range() missing required argument 'stop'"
            && error.code == Some(DiagnosticCode::CALL_MISSING_REQUIRED_ARGUMENT)
            && error.primary_range == Some(range_for(range_source, "range"))
    }));
}

#[test]
pub(super) fn test_function_unexpected_keyword_has_call_code() {
    let source = "def greet(name: str) -> str:\n    return \"hello\"\n\ndef main():\n    print(greet(\"Alice\", punctuation=\"!\"))\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "greet() got an unexpected keyword argument 'punctuation'"
            && error.code == Some(DiagnosticCode::CALL_UNEXPECTED_KEYWORD)
            && error.primary_range
                == Some(range_for_after_anchor(
                    source,
                    "greet(\"Alice\", ",
                    "punctuation",
                ))
    }));
}

#[test]
pub(super) fn test_keyword_after_positional_has_call_code() {
    let source = "def greet(name: str, greeting: str) -> str:\n    return greeting\n\ndef main():\n    print(greet(\"Alice\", name=\"Bob\"))\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "greet() got multiple values for argument 'name'"
            && error.code == Some(DiagnosticCode::CALL_DUPLICATE_ARGUMENT)
            && error.primary_range
                == Some(range_for_after_anchor(source, "greet(\"Alice\", ", "name"))
    }));
}

#[test]
pub(super) fn test_duplicate_keyword_has_call_code() {
    let source = "def greet(name: str) -> str:\n    return name\n\ndef main():\n    print(greet(name=\"Alice\", name=\"Bob\"))\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "greet() got multiple values for keyword argument 'name'"
            && error.code == Some(DiagnosticCode::CALL_DUPLICATE_ARGUMENT)
            && error.primary_range
                == Some(range_for_after_anchor(
                    source,
                    "greet(name=\"Alice\", ",
                    "name",
                ))
    }));
}

#[test]
pub(super) fn test_range_duplicate_stop_keyword_has_call_code() {
    let source = "def main():\n    print(list(range(10, stop=20)))\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "range() got multiple values for argument 'stop'"
            && error.code == Some(DiagnosticCode::CALL_DUPLICATE_ARGUMENT)
            && error.primary_range == Some(range_for_after_anchor(source, "range(10, ", "stop"))
    }));
}

#[test]
pub(super) fn test_map_callable_arity_mismatch_has_call_code() {
    let source = "def inc(x: int) -> int:\n    return x + 1\n\ndef main():\n    values: list[int] = map(inc, [1, 2], [3, 4])\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "map() callable expects 1 argument(s), got 2 iterable(s)"
            && error.code == Some(DiagnosticCode::CALL_NOT_CALLABLE_OR_ARITY)
            && error.primary_range
                == Some(range_for_after_anchor(
                    source,
                    "map(inc, [1, 2], ",
                    "[3, 4]",
                ))
    }));
}

#[test]
pub(super) fn test_non_simple_call_target_has_call_code() {
    let source = "def make() -> int:\n    return 1\n\ndef main():\n    value: int = make()(1)\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "only simple function calls are supported"
            && error.code == Some(DiagnosticCode::CALL_NOT_CALLABLE_OR_ARITY)
            && error.primary_range
                == Some(range_for_after_anchor(source, "value: int = ", "make()"))
    }));
}

#[test]
pub(super) fn test_open_missing_path_has_call_code() {
    let source = "def main():\n    _file = open()\n";
    let result = lower_source(source);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|error| {
        error.message == "open() requires at least 1 argument: open(path) or open(path, mode)"
            && error.code == Some(DiagnosticCode::CALL_MISSING_REQUIRED_ARGUMENT)
            && error.primary_range == Some(range_for(source, "open"))
    }));
}
