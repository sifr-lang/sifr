use super::generate_rust_from_source;

#[test]
fn integer_field_augassign_uses_exact_floor_operations_inside_and_outside_try() {
    let rust = generate_rust_from_source(include_str!(
        "../../../sifr/tests/e2e/pass/integer_field_augassign.sifr"
    ));
    assert!(!rust.contains("compile_error!"), "{rust}");
    assert!(
        rust.matches("floor_mod_known_nonzero").count() >= 4,
        "{rust}"
    );
    assert!(
        rust.matches("floor_div_known_nonzero").count() >= 3,
        "{rust}"
    );
    assert!(!rust.contains("%="), "{rust}");
    assert!(!rust.contains("/="), "{rust}");
    assert!(!rust.contains(".unwrap()"), "{rust}");
    assert!(!rust.contains(".expect("), "{rust}");
}
