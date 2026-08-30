use super::generate_rust_from_source;

#[test]
fn canonical_record_aliases_intern_one_rust_layout() {
    let rust = generate_rust_from_source(
        r#"
type First = {id: int, email: str}
type Second = {email: str, id: int}

def main():
    first: First = First(id=1, email="first")
    second: Second = Second(email="second", id=2)
    assert first.id == 1
    assert second.email == "second"
"#,
    );
    assert_eq!(
        rust.matches("pub struct __SifrRecord_").count(),
        1,
        "{rust}"
    );
    assert!(rust.contains("pub(crate) email: __SifrField0"), "{rust}");
    assert!(rust.contains("pub(crate) id: __SifrField1"), "{rust}");
    assert_eq!(rust.matches("<String, SifrInt>").count(), 2, "{rust}");
    syn::parse_file(&rust).expect("record codegen should produce valid Rust syntax");
}

#[test]
fn borrowed_width_call_uses_a_non_escaping_record_view() {
    let rust = generate_rust_from_source(
        r#"
type Wide = {active: bool, email: str, id: int}
type Narrow = {email: str, id: int}

def read(value: Narrow) -> int:
    return value.id

def main():
    wide: Wide = Wide(active=True, email="dev@sifr.dev", id=7)
    assert read(wide) == 7
"#,
    );
    assert!(rust.contains("__SifrRecordView_"), "{rust}");
    assert!(rust.contains("__sifr_record_field_id"), "{rust}");
    assert!(!rust.contains("__sifr_record_projection"), "{rust}");
    assert!(rust.contains("read(&"), "{rust}");
    syn::parse_file(&rust).expect("borrowed record projection should be valid Rust syntax");
}

#[test]
fn field_name_layout_family_supports_distinct_and_generic_field_types() {
    let rust = generate_rust_from_source(
        r#"
type Boxed[T] = {value: T}
type IntBox = {value: int}
type StrBox = {value: str}

def wrap[T](own value: T) -> Boxed[T]:
    return Boxed[T](value=value)

def main():
    integer: IntBox = wrap(1)
    text: StrBox = wrap("value")
    assert integer.value == 1
    assert text.value == "value"
"#,
    );
    assert_eq!(
        rust.matches("pub struct __SifrRecord_").count(),
        1,
        "{rust}"
    );
    assert!(rust.contains("<SifrInt>"), "{rust}");
    assert!(rust.contains("<String>"), "{rust}");
    syn::parse_file(&rust).expect("generic record layouts should produce valid Rust syntax");
}
