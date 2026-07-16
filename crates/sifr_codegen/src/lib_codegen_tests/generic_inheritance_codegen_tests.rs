use super::*;

#[test]
fn fieldless_generic_class_uses_phantom_representation_and_contextual_type() {
    let rust_code = generate_rust_from_source(
        r#"class Marker[T]:
    pass

def make[T]() -> Marker[T]:
    return Marker()

def main():
    marker: Marker[int] = make()
"#,
    );

    assert!(rust_code.contains("struct Marker<T>"));
    assert!(rust_code.contains("__sifr_type_marker: std::marker::PhantomData<(T,)>"));
    assert!(rust_code.contains("let marker: Marker<i64> = make();"));
}

#[test]
fn consuming_class_upcasts_use_generated_parent_conversions() {
    let rust_code = generate_rust_from_source(
        r#"class Root:
    value: int

class Mid(Root):
    middle: int

    def __init__(self, value: int, middle: int):
        super().__init__(value)
        self.middle = middle

class Child(Mid):
    extra: int

    def __init__(self, value: int, middle: int, extra: int):
        super().__init__(value, middle)
        self.extra = extra

def consume(own value: Root) -> int:
    return value.value

def as_root(own value: Child) -> Root:
    return value

def main():
    child: Child = Child(1, 2, 3)
    result: int = consume(child)
    root: Root = as_root(Child(4, 5, 6))
"#,
    );

    assert!(rust_code.contains("impl std::convert::From<Child> for Mid"));
    assert!(rust_code.contains("impl std::convert::From<Mid> for Root"));
    let transitive = "std::convert::Into::<Root>::into(std::convert::Into::<Mid>::into";
    assert_eq!(rust_code.matches(transitive).count(), 2);
}
