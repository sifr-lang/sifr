use super::*;

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
