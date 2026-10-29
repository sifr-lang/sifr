use super::*;

#[test]
fn tuple_slice_errors_have_type_codes() {
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

    let dynamic_source = "def main():\n    pair: tuple[int, str] = (1, \"x\")\n    start: int = 0\n    _bad = pair[start:2]\n";
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
fn unsupported_slice_receiver_has_type_code() {
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
fn unproven_slice_step_is_rejected_before_codegen() {
    let source = "\
def main(values: list[int], step: int) -> None:
    sliced: list[int] = values[::step]
";
    let errors = lower_source(source).expect_err("an unproven slice step must fail closed");
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::INT_EXACT_DIVISION_REQUIRES_HANDLING)
            && error.primary_range == Some(range_for_after_anchor(source, "values[::", "step"))
    }));
}
