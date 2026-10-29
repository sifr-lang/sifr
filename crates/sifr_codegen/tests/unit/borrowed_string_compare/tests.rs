use super::*;

#[test]
fn literal_typed_name_uses_constant_character_state() {
    let expression = HirExpr::Name {
        name: "expected".to_string(),
        binding_id: None,
        ty: Type::LiteralStr("🦀".to_string()),
    };
    let mut emitter = RustEmitter::new();

    let value = emitter
        .lower_string_compare_value(&expression)
        .expect("literal-typed name should lower")
        .expect("literal-typed name should be a string comparison value");

    let StringCompareValue::Str {
        comparison_state, ..
    } = value
    else {
        panic!("literal-typed name should lower as a string");
    };
    assert_eq!(
        comparison_state,
        Some(literal_string_comparison_state("🦀"))
    );
}
