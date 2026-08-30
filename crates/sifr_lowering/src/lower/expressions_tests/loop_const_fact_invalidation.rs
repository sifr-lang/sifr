use super::*;

fn assert_loop_numeric_operations_require_typed_handling(source: &str) {
    let errors = lower_source(source)
        .expect_err("loop-carried integer values must not use stale constant facts");

    for expression in ["value / 2", "float(value)", "value * 1.5"] {
        assert!(
            errors.iter().any(|error| {
                error.code == Some(DiagnosticCode::TYPE_MISMATCH)
                    && error.message.contains("Result[float,")
                    && error.primary_range == Some(range_for(source, expression))
            }),
            "expected a typed-failure diagnostic for {expression}: {errors:?}"
        );
    }
}

#[test]
fn while_body_assignment_invalidates_pre_loop_integer_fact() {
    assert_loop_numeric_operations_require_typed_handling(
        "\
def main() -> None:
    value: int = 0
    while value < 3:
        divided: float = value / 2
        converted: float = float(value)
        mixed: float = value * 1.5
        value = value + 1
",
    );
}

#[test]
fn for_body_assignment_invalidates_pre_loop_integer_fact() {
    assert_loop_numeric_operations_require_typed_handling(
        "\
def main() -> None:
    value: int = 4
    for iteration in range(3):
        divided: float = value / 2
        converted: float = float(value)
        mixed: float = value * 1.5
        value = value + 2
",
    );
}
