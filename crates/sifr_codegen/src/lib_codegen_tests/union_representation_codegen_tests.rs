use super::*;

#[test]
fn generic_option_none_comparisons_use_option_predicates_without_partial_eq() {
    let rust_code = generate_rust_from_source(
        r#"def both_missing[T](left: T | None, right: T | None) -> bool:
    return left is None and right == None
"#,
    );

    assert!(rust_code.contains("left.is_none()"), "{rust_code}");
    assert!(rust_code.contains("right.is_none()"), "{rust_code}");
    assert!(!rust_code.contains("Option<T> == None"), "{rust_code}");
    assert!(!rust_code.contains("T: PartialEq"), "{rust_code}");
}

#[test]
fn computed_option_none_comparisons_use_option_predicates() {
    let rust_code = generate_rust_from_source(
        r#"def missing(values: dict[str, int]) -> bool:
    return values.get("missing") == None
"#,
    );

    assert!(rust_code.contains(".is_none()"), "{rust_code}");
    assert!(!rust_code.contains("== None"), "{rust_code}");
}

#[test]
fn optional_string_length_keeps_a_payload_compatible_callback() {
    let rust_code = generate_rust_from_source(
        r#"def optional_length(value: str | None) -> int:
    return len(value)
"#,
    );

    assert!(
        rust_code.contains("|value| value.chars().count()"),
        "{rust_code}"
    );
    assert!(!rust_code.contains("::std::vec::Vec::len"), "{rust_code}");
}

#[test]
fn owned_optional_argument_widening_is_not_force_unwrapped_after_conversion() {
    let target = sifr_type_system::make_union(vec![Type::Int, Type::Str, Type::None]);
    let rust_code = generate_rust_from_source(
        r#"def consume(own value: int | str | None) -> bool:
    return value is None

def forward(value: str | None) -> bool:
    return consume(value)
"#,
    );

    assert!(rust_code.contains(&target.union_enum_name()), "{rust_code}");
    assert!(rust_code.contains(".map("), "{rust_code}");
    assert!(rust_code.contains(".unwrap_or("), "{rust_code}");
    assert!(
        !rust_code.contains("compiler-verified option argument should be Some"),
        "{rust_code}"
    );
}

#[test]
fn option_represented_union_argument_matches_the_payload_enum() {
    let payload = sifr_type_system::make_union(vec![Type::Int, Type::Str]);
    let runtime_source = sifr_type_system::safe_optional_result(payload.clone());
    let target = sifr_type_system::make_union(vec![Type::Bool, Type::Int, Type::Str, Type::None]);
    let rust_code = generate_rust_from_source(
        r#"def consume(own value: bool | int | str | None) -> bool:
    return value is None

def forward(values: dict[str, int | str]) -> bool:
    return consume(values.get("value"))
"#,
    );

    assert!(rust_code.contains(&target.union_enum_name()), "{rust_code}");
    assert!(
        rust_code.contains(&format!("{}::", payload.union_enum_name())),
        "{rust_code}"
    );
    assert!(
        !rust_code.contains(&format!("{}::", runtime_source.union_enum_name())),
        "{rust_code}"
    );
}

#[test]
fn owned_method_optional_argument_uses_the_same_widening_sequence() {
    let rust_code = generate_rust_from_source(
        r#"class Consumer:
    def accept(self, own value: int | str | None) -> bool:
        return value is None

def forward(consumer: Consumer, value: str | None) -> bool:
    return consumer.accept(value)
"#,
    );

    assert!(rust_code.contains(".map("), "{rust_code}");
    assert!(rust_code.contains(".unwrap_or("), "{rust_code}");
    assert!(
        !rust_code.contains("compiler-verified option argument should be Some"),
        "{rust_code}"
    );
}

#[test]
fn option_represented_exact_union_argument_flattens_only_once() {
    let rust_code = generate_rust_from_source(
        r#"def consume(own value: int | str | None) -> bool:
    return value is None

def forward(values: dict[str, int | str | None]) -> bool:
    return consume(values.get("value"))
"#,
    );

    assert!(rust_code.contains(".unwrap_or("), "{rust_code}");
    assert!(
        !rust_code.contains("__sifr_union_value| match __sifr_union_value"),
        "{rust_code}"
    );
}

#[test]
fn safe_nullable_union_boundaries_convert_each_value_once() {
    let rust_code = generate_rust_from_source(
        r#"def consume(value: int | str | None) -> bool:
    return value is None

def select(
    values: dict[str, int | str | None],
    choose_safe: bool,
    fallback: int | str | None = None,
) -> int | str | None:
    assigned: int | str | None = values.get("assigned")
    stored: list[int | str | None] = []
    stored.append(values.get("stored"))
    joined = values.get("joined") if choose_safe else fallback
    assert consume(values.get("argument"))
    if assigned is not None:
        return assigned
    return joined
"#,
    );

    assert_eq!(
        rust_code.matches(".unwrap_or(").count(),
        4,
        "each safe read must materialize its nullable union exactly once:\n{rust_code}"
    );
    assert_eq!(
        rust_code.matches(".map(").count(),
        0,
        "exact nullable-union conversions do not need a payload map:\n{rust_code}"
    );
}

#[test]
fn nested_option_represented_union_payload_widens_recursively() {
    let payload = sifr_type_system::make_union(vec![Type::Int, Type::Str, Type::None]);
    let target = sifr_type_system::make_union(vec![Type::Bool, Type::Int, Type::Str, Type::None]);
    let rust_code = generate_rust_from_source(
        r#"def consume(own value: bool | int | str | None) -> bool:
    return value is None

def forward(values: dict[str, int | str | None]) -> bool:
    return consume(values.get("value"))
"#,
    );

    assert!(
        rust_code.contains(&payload.union_enum_name()),
        "{rust_code}"
    );
    assert!(rust_code.contains(&target.union_enum_name()), "{rust_code}");
    assert!(rust_code.contains(".map("), "{rust_code}");
    assert!(
        rust_code.contains("match __sifr_option_value"),
        "{rust_code}"
    );
    assert!(rust_code.contains(".unwrap_or("), "{rust_code}");
    let none_pattern = format!(
        "{}::{}(_)",
        payload.union_enum_name(),
        Type::None.union_variant_name()
    );
    assert!(rust_code.contains(&none_pattern), "{rust_code}");
}

#[test]
fn option_union_members_receive_assignable_payload_conversions() {
    let rust_code = generate_rust_from_source(
        r#"class Root:
    value: int

class Child(Root):
    pass

def consume(own value: Root | str | None) -> bool:
    return value is None

def forward(values: dict[str, Child | str]) -> bool:
    return consume(values.get("value"))
"#,
    );

    assert!(
        rust_code.contains("::std::convert::Into::<Root>::into(__sifr_union_value)"),
        "{rust_code}"
    );
}

#[test]
fn union_assignment_uses_the_consuming_representation_sequence() {
    let rust_code = generate_rust_from_source(
        r#"class Root:
    value: int

class Child(Root):
    pass

def select(values: dict[str, Child | str]) -> Root | str | None:
    selected: Root | str | None = None
    selected = values.get("value")
    return selected
"#,
    );

    assert!(
        rust_code.contains("::std::convert::Into::<Root>::into(__sifr_union_value)"),
        "{rust_code}"
    );
    assert!(rust_code.contains("selected ="), "{rust_code}");
}

#[test]
fn isinstance_narrows_union_items_inside_for_loops() {
    let source = r#"def collect_text(values: list[int | str]) -> list[int | str]:
    output: list[int | str] = []
    for value in values:
        if isinstance(value, str):
            output.append(value)
    return output
"#;
    let rust_code = generate_rust_from_source(source);

    assert!(!rust_code.contains("if isinstance("), "{rust_code}");
    assert!(rust_code.contains("match value"), "{rust_code}");
    assert!(
        rust_code.contains(&format!("{}(value)", Type::Str.union_variant_name())),
        "{rust_code}"
    );
    assert!(!rust_code.contains("unreachable!"), "{rust_code}");
}
