use super::*;
use num_bigint::BigInt;

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

#[test]
fn async_for_body_assignment_invalidates_pre_loop_integer_fact() {
    assert_loop_numeric_operations_require_typed_handling(
        "\
class StreamError(Error):
    pass

class CounterStream:
    current: int

    def __init__(self):
        self.current = 0

    async def anext(mut self) -> Result[Option[int], StreamError]:
        if self.current >= 1:
            return None
        self.current = self.current + 1
        value: Option[int] = self.current
        return value

async def main() -> Result[None, StreamError]:
    value: int = 4
    stream: CounterStream = CounterStream()
    async for iteration in stream:
        divided: float = value / 2
        converted: float = float(value)
        mixed: float = value * 1.5
        value = value + iteration
    return None
",
    );
}

#[test]
fn called_nonlocal_mutator_invalidates_loop_carried_integer_fact() {
    assert_loop_numeric_operations_require_typed_handling(
        "\
def main() -> None:
    value: int = 0
    def advance() -> None:
        nonlocal value
        value = value + 1
    while value < 3:
        divided: float = value / 2
        converted: float = float(value)
        mixed: float = value * 1.5
        advance()
",
    );
}

#[test]
fn nonlocal_mutator_declared_in_loop_invalidates_pre_lowering_integer_fact() {
    assert_loop_numeric_operations_require_typed_handling(
        "\
def main() -> None:
    value: int = 0
    while value < 3:
        divided: float = value / 2
        converted: float = float(value)
        mixed: float = value * 1.5
        def advance() -> None:
            nonlocal value
            value = value + 1
        advance()
",
    );
}

#[test]
fn transitive_nonlocal_mutator_invalidates_loop_carried_integer_fact() {
    assert_loop_numeric_operations_require_typed_handling(
        "\
def main() -> None:
    value: int = 0
    def tick() -> None:
        nonlocal value
        value = value + 1
    def advance() -> None:
        tick()
    while value < 3:
        divided: float = value / 2
        converted: float = float(value)
        mixed: float = value * 1.5
        advance()
",
    );
}

#[test]
fn callable_alias_effect_propagates_through_nested_helper() {
    assert_loop_numeric_operations_require_typed_handling(
        "\
def main() -> None:
    value: int = 0
    def tick() -> None:
        nonlocal value
        value = value + 1
    alias: Callable[[], None] = tick
    def advance() -> None:
        alias()
    while value < 3:
        divided: float = value / 2
        converted: float = float(value)
        mixed: float = value * 1.5
        advance()
",
    );
}

#[test]
fn cleared_local_fact_does_not_fall_back_to_shadowed_module_constant() {
    assert_loop_numeric_operations_require_typed_handling(
        "\
value: int = 8

def main() -> None:
    value: int = 1
    value = value + 1
    while value < 4:
        divided: float = value / 2
        converted: float = float(value)
        mixed: float = value * 1.5
        value = value + 1
",
    );
}

#[test]
fn imported_integer_constant_retains_exact_value_proof() {
    let mut externals = crate::ExternalDefs::default();
    externals
        .constants
        .entry("settings".to_string())
        .or_default()
        .insert("LIMIT".to_string(), Type::Int);
    externals
        .constant_integer_values
        .entry("settings".to_string())
        .or_default()
        .insert("LIMIT".to_string(), BigInt::from(8));

    let source = "\
from settings import LIMIT

def main() -> None:
    value: float = float(LIMIT)
";

    lower_source_with_externals(source, &externals)
        .expect("an imported integer constant should retain its exact value proof");
}
