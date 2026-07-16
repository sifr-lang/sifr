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
    assert!(rust_code.contains("__sifr_type_marker: std::marker::PhantomData<fn() -> (T,)>"));
    assert!(rust_code.contains("let marker: Marker<i64> = make();"));
}

#[test]
fn consuming_class_upcasts_enter_union_and_result_payloads() {
    let rust_code = generate_rust_from_source(
        r#"class Root:
    value: int

class Child(Root):
    extra: int

    def __init__(self, value: int, extra: int):
        super().__init__(value)
        self.extra = extra

def as_union(own value: Child) -> Root | int:
    return value

def as_result(own value: Child) -> Result[Root, ValueError]:
    return value
"#,
    );

    assert!(
        rust_code.contains("IntOrRoot::Root(std::convert::Into::<Root>::into(value))"),
        "{rust_code}"
    );
    assert!(
        rust_code.contains("Ok(std::convert::Into::<Root>::into(value))"),
        "{rust_code}"
    );
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
