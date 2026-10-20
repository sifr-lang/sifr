use super::*;

#[test]
fn nominal_callable_field_nested_in_conversion_remains_a_field_invocation() {
    let rust = generate_rust_from_source(include_str!(
        "../../../sifr/tests/e2e/pass/callable_struct_field.sifr"
    ));
    assert!(rust.contains("(c.callback)("), "{rust}");
    assert!(rust.contains("(c2.callback)(c2.value.clone())"), "{rust}");
    assert!(!rust.contains("c2.callback("), "{rust}");
}

#[test]
fn nominal_callable_field_contexts_preserve_borrows_and_same_named_methods() {
    let rust = generate_rust_from_source(include_str!(
        "../../../sifr/tests/e2e/pass/callable_field_call_contexts.sifr"
    ));
    assert!(
        rust.matches("(holder.callback)(&text)").count() >= 3,
        "{rust}"
    );
    assert!(!rust.contains("holder.callback("), "{rust}");
    assert!(rust.contains("method.callback(&text)"), "{rust}");
    assert!(!rust.contains("(method.callback)("), "{rust}");
}
