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
    assert!(rust_code.contains("__sifr_type_marker: ::std::marker::PhantomData<fn() -> (T,)>"));
    assert!(rust_code.contains("let _marker: Marker<SifrInt> = make();"));
}

#[test]
fn inherited_constructor_evaluates_super_at_its_source_position() {
    let rust_code = generate_rust_from_source(
        r#"class Parent:
    value: int

def mark() -> None:
    return None

class Child(Parent):
    def __init__(self, value: int):
        super().__init__(value)
        mark()

def main() -> None:
    child: Child = Child(1)
"#,
    );

    let child_impl = rust_code
        .split("impl Child")
        .nth(1)
        .expect("Child implementation");
    let parent_init = child_impl.find("Parent::new(value)").expect("parent init");
    let mark = child_impl.find("mark();").expect("following statement");
    assert!(parent_init < mark, "{child_impl}");
    assert!(child_impl.contains("parent: __sifr_parent"), "{child_impl}");
}

#[test]
fn grandparent_super_call_uses_the_defining_ancestor() {
    let rust_code = generate_rust_from_source(
        r#"class Root:
    def value(self) -> int:
        return 5

class Middle(Root):
    pass

class Leaf(Middle):
    def value(self) -> int:
        return super().value()

def main() -> None:
    leaf: Leaf = Leaf()
    assert leaf.value() == 5
"#,
    );

    assert!(rust_code.contains("Root::value(self)"), "{rust_code}");
    assert!(!rust_code.contains("Middle::value(self)"), "{rust_code}");
}

#[test]
fn generic_parent_paths_and_match_patterns_use_base_names() {
    let rust_code = generate_rust_from_source(
        r#"class Box[T]:
    value: T

    def __init__(self, value: T):
        self.value = value

    def label(self) -> int:
        return 1

class IntBox(Box[int]):
    def __init__(self, value: int):
        super().__init__(value)

    def label(self) -> int:
        return super().label()

def read(box: Box[int]):
    match box:
        case Box(value=value):
            return value
"#,
    );

    assert!(rust_code.contains("Box::new(value)"), "{rust_code}");
    assert!(rust_code.contains("Box::label(self)"), "{rust_code}");
    assert!(rust_code.contains("Box { value:"), "{rust_code}");
    assert!(!rust_code.contains("Box<SifrInt>::"), "{rust_code}");
    assert!(!rust_code.contains("Box<SifrInt> { value:"), "{rust_code}");
}

#[test]
fn explicit_non_affine_pow_dunder_remains_an_inherent_method() {
    let rust_code = generate_rust_from_source(
        r#"class Power:
    def __pow__(self, exponent: int) -> int:
        return exponent

def main() -> None:
    power: Power = Power()
    value: int = power.__pow__(3)
    assert value == 3
"#,
    );

    assert!(rust_code.contains("fn __pow__"), "{rust_code}");
    assert!(
        rust_code.contains("power.__pow__(&SifrInt::from_i64(3))"),
        "{rust_code}"
    );
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
        rust_code.contains("(::std::convert::Into::<Root>::into(value))"),
        "{rust_code}"
    );
    assert!(rust_code.contains("__SifrUnion_"), "{rust_code}");
    assert!(rust_code.contains("::__SifrUnionVariant_"), "{rust_code}");
    assert!(
        rust_code.contains("Ok(::std::convert::Into::<Root>::into(value))"),
        "{rust_code}"
    );
}

#[test]
fn consuming_class_upcasts_remap_existing_union_and_result_representations() {
    let rust_code = generate_rust_from_source(
        r#"class Root:
    value: int

class Child(Root):
    extra: int

    def __init__(self, value: int, extra: int):
        super().__init__(value)
        self.extra = extra

def make_union() -> Child | int:
    return Child(1, 2)

def relay_union() -> Root | int:
    return make_union()

def consume_union(own value: Root | int) -> int:
    return 1

def make_result() -> Result[Child, ValueError]:
    return Child(3, 4)

def relay_result() -> Result[Root, ValueError]:
    return make_result()

def consume_result(own value: Result[Root, ValueError]) -> int:
    return 2

def main():
    union_result: int = consume_union(Child(5, 6))
    result_result: int = consume_result(make_result())
"#,
    );

    assert!(rust_code.contains("match make_union()"), "{rust_code}");
    assert!(rust_code.contains("(__sifr_union_value) =>"), "{rust_code}");
    assert!(
        rust_code.contains("(::std::convert::Into::<Root>::into(__sifr_union_value))"),
        "{rust_code}"
    );
    assert!(
        rust_code.contains("(make_result()).map(::std::convert::Into::<Root>::into)"),
        "{rust_code}"
    );
    assert!(
        rust_code.contains("consume_union(__SifrUnion_"),
        "{rust_code}"
    );
    assert!(
        rust_code.contains("(::std::convert::Into::<Root>::into(Child::new"),
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

    assert!(rust_code.contains("impl ::std::convert::From<Child> for Mid"));
    assert!(rust_code.contains("impl ::std::convert::From<Mid> for Root"));
    let transitive = "::std::convert::Into::<Root>::into(::std::convert::Into::<Mid>::into";
    assert_eq!(rust_code.matches(transitive).count(), 2);
}
