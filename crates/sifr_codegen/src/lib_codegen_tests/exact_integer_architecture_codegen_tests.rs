use super::*;

#[test]
fn source_int_uses_one_exact_representation_at_every_generated_boundary() {
    let rust = generate_rust_from_source(
        r#"LIMIT: int = 7

class IntBox:
    value: int

def identity(own value: int) -> int:
    local: int = value + LIMIT
    return local

def optional(flag: bool, own value: int) -> int | None:
    if flag:
        return value
    return None

def main():
    boxed: IntBox = IntBox(9)
    values: list[int] = [boxed.value, identity(10)]
    mapping: dict[str, int] = {"value": 17}
    result: int | None = optional(True, 17)
    assert result == 17
"#,
    );

    assert!(
        rust.contains("fn __sifr_const_4c494d4954() -> SifrInt"),
        "{rust}"
    );
    assert!(rust.contains("value: SifrInt"), "{rust}");
    assert!(rust.contains("Vec<SifrInt>"), "{rust}");
    assert!(rust.contains("HashMap<String, SifrInt>"), "{rust}");
    assert!(rust.contains("Option<SifrInt>"), "{rust}");
    assert!(rust.contains("let local: SifrInt"), "{rust}");
    assert!(rust.contains("fn identity(value: SifrInt"), "{rust}");
    assert!(!rust.contains("value: i64"), "{rust}");
    assert!(!rust.contains("Vec<i64>"), "{rust}");
}

#[test]
fn emitted_integer_operators_use_exact_runtime_operations() {
    let rust = generate_rust_from_source(
        r#"def main():
    huge: int = 9223372036854775807 + 2
    product: int = huge * huge
    quotient: int = -7 // 3
    remainder: int = -7 % 3
    shifted: int = 3 << 10
    powered: int = 2 ** 100
    assert quotient == -3
    assert remainder == 2
    assert shifted == 3072
    assert powered > product
"#,
    );

    assert!(rust.contains("SifrInt::from"), "{rust}");
    assert!(rust.contains("floor_div_known_nonzero"), "{rust}");
    assert!(rust.contains("floor_mod_known_nonzero"), "{rust}");
    assert!(rust.contains(".pow_known_valid("), "{rust}");
    assert!(
        rust.contains(
            "std::ops::Add::add(&SifrInt::from_i64(9223372036854775807), &SifrInt::from_i64(2))"
        ),
        "{rust}"
    );
}

#[test]
fn emitted_mixed_integer_float_comparison_does_not_narrow_the_integer() {
    let rust = generate_rust_from_source(
        r#"def main():
    exact: int = 9007199254740993
    rounded: float = 9007199254740992.0
    assert exact > rounded
    assert rounded < exact
"#,
    );

    assert!(!rust.contains("exact as f64"), "{rust}");
    assert!(!rust.contains("(exact).to_f64"), "{rust}");
    assert_eq!(rust.matches("exact.gt_f64(rounded)").count(), 2, "{rust}");
}

#[test]
fn emitted_dynamic_integer_float_arithmetic_checks_the_conversion() {
    let rust = generate_rust_from_source(
        r#"def add(value: int, amount: float) -> Result[float, FloatOverflowError | FloatPrecisionLossError]:
    return value + amount
"#,
    );

    assert!(rust.contains("value.checked_to_f64()"), "{rust}");
    assert!(
        rust.contains("IntegerFloatConversionError::Overflow"),
        "{rust}"
    );
    assert!(
        rust.contains("IntegerFloatConversionError::PrecisionLoss"),
        "{rust}"
    );
    assert!(!rust.contains("value as f64"), "{rust}");
}

#[test]
fn emitted_hash_normalizes_exact_and_fixed_width_integers() {
    let rust = generate_rust_from_source(
        r#"def main():
    exact: int = 1
    fixed: int8 = 1
    assert hash(exact) == hash(fixed)
"#,
    );

    assert!(rust.matches("normalized_hash_key()").count() >= 2, "{rust}");
}

#[test]
fn callable_field_owned_exact_integer_argument_clones_a_borrowed_method_parameter() {
    let rust = generate_rust_from_source(
        r#"class Owner:
    callback: Callable[[int], int]

    def __init__(self, callback: Callable[[int], int]):
        self.callback = callback

    def apply(self, value: int) -> int:
        return self.callback(value)

def identity(value: int) -> int:
    return value
"#,
    );

    assert!(rust.contains("(self.callback)(value.clone())"), "{rust}");
}

#[test]
fn constructor_field_initialization_preserves_an_exact_parameter_used_later() {
    let rust = generate_rust_from_source(
        r#"class Owner:
    value: int
    values: list[int]

    def __init__(self, value: int):
        self.values = []
        self.values.append(value)
        self.value = value
"#,
    );

    assert!(rust.contains("value: value.clone()"), "{rust}");
}

#[test]
fn exact_integer_runtime_preambles_do_not_reintroduce_i64_source_boundaries() {
    let parallel = crate::parallel_runtime_rust_code();
    assert!(parallel.contains("workers: SifrInt"), "{parallel}");
    assert!(parallel.contains("fn new(workers: SifrInt)"), "{parallel}");
    assert!(!parallel.contains("workers: i64"), "{parallel}");

    let task_scope = crate::build_task_scope_process_items();
    let rendered = crate::render_items(&task_scope);
    let compact = rendered
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    assert!(
        compact.contains("::sifr_runtime::SifrInt::from(__handle)"),
        "{rendered}"
    );
}
