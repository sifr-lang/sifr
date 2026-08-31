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

#[test]
fn owned_projection_moves_fields_without_generated_clones() {
    let rust = generate_rust_from_source(
        r#"
type Wide = {email: str, id: int, note: str}
type Narrow = {email: str, id: int}

def main():
    wide: Wide = Wide(email="dev@sifr.dev", id=1, note="drop me")
    narrow: Narrow = wide.project[Narrow]()
    assert narrow.email == "dev@sifr.dev"
"#,
    );
    let projection = rust
        .lines()
        .find(|line| line.contains("let narrow:"))
        .expect("projection binding should be emitted");
    assert!(projection.contains("wide.email"), "{projection}");
    assert!(projection.contains("wide.id"), "{projection}");
    assert!(!projection.contains("clone"), "{projection}");
    syn::parse_file(&rust).expect("owned projection should produce valid Rust syntax");
}

#[test]
fn all_copy_record_layout_derives_copy() {
    let rust = generate_rust_from_source(
        r#"
type Flags = {active: bool, admin: bool}

def active(flags: Flags) -> bool:
    return flags.active

def main():
    flags: Flags = Flags(active=True, admin=False)
    assert active(flags)
    assert active(flags)
"#,
    );
    assert!(
        rust.contains("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]"),
        "{rust}"
    );
    syn::parse_file(&rust).expect("copy record layout should produce valid Rust syntax");
}

#[test]
fn logical_copy_record_clones_when_a_rust_field_is_not_copy() {
    let rust = generate_rust_from_source(
        r#"
type Count = {value: int}

def value(count: Count) -> int:
    return count.value

def main():
    count: Count = Count(value=1)
    assert value(count) == 1
    assert value(count) == 1
"#,
    );
    assert!(rust.contains("value(count.clone())"), "{rust}");
    syn::parse_file(&rust).expect("logical-copy record should produce valid Rust syntax");
}
