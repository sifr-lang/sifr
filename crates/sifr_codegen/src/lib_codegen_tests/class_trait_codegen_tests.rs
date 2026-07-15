use super::*;

#[test]
fn generic_and_inherited_class_formatting_matches_emitted_traits() {
    let rust_code = generate_rust_from_source(
        r#"class Box[T]:
    value: T

class Parent:
    value: int

class Child(Parent):
    def __init__(self, value: int):
        super().__init__(value)

def show_box(value: Box[int], values: list[Box[int]]) -> None:
    print(value)
    print(values)

def show_child(value: Child) -> str:
    print(value)
    return str(value)
"#,
    );

    assert!(rust_code.contains("struct Box<T>"), "{rust_code}");
    assert!(
        rust_code.contains("#[derive(Debug, Clone, PartialEq)]\nstruct Box<T>"),
        "{rust_code}"
    );
    assert!(
        rust_code.contains("impl<T: std::fmt::Display> std::fmt::Display for Box<T>"),
        "{rust_code}"
    );
    assert!(
        rust_code.contains("impl std::fmt::Display for Child"),
        "{rust_code}"
    );
    assert!(rust_code.contains("Child(parent={})"), "{rust_code}");
}

#[test]
fn generic_method_clone_bounds_apply_only_to_consumed_type_parameters() {
    let rust_code = generate_rust_from_source(
        r#"class Pair[A, B]:
    first: A
    second: B

    def first_value(self) -> A:
        return self.first
"#,
    );

    assert!(
        rust_code.contains("impl<A: Clone, B> Pair<A, B>"),
        "{rust_code}"
    );
    assert!(!rust_code.contains("impl<A: Clone, B: Clone> Pair<A, B>"));
}

#[test]
fn generic_method_bounds_follow_direct_self_calls() {
    let rust_code = generate_rust_from_source(
        r#"class Holder[T]:
    value: T

    def read(self) -> T:
        return self.value

    def read_indirect(self) -> T:
        return self.read()

    def same(self, other: T) -> bool:
        return self.value == other

    def same_indirect(self, other: T) -> bool:
        return self.same(other)
"#,
    );

    assert_eq!(rust_code.matches("impl<T: Clone> Holder<T>").count(), 2);
    assert!(
        rust_code.contains("impl<T: Clone + PartialEq> Holder<T>"),
        "{rust_code}"
    );
}

#[test]
fn sorted_shared_borrow_key_does_not_clone_comparator_elements() {
    let rust_code = generate_rust_from_source(
        r#"class Local(NonSend):
    rank: int

def rank_of(value: Local) -> int:
    return value.rank

def order() -> list[Local]:
    return sorted([Local(2), Local(1)], key=rank_of)
"#,
    );

    assert!(rust_code.contains("rank_of(__left)"), "{rust_code}");
    assert!(rust_code.contains("rank_of(__right)"), "{rust_code}");
    assert!(!rust_code.contains("__left.clone()"), "{rust_code}");
    assert!(!rust_code.contains("__right.clone()"), "{rust_code}");
}

#[test]
fn generic_collection_and_arithmetic_bounds_are_recursive_and_exact() {
    let rust_code = generate_rust_from_source(
        r#"class Holder[T]:
    values: list[T]

    def same(self, other: list[T]) -> bool:
        return self.values == other

class Math[T]:
    value: T

    def product(self, other: T) -> T:
        return self.value * other
"#,
    );

    assert!(
        rust_code.contains("impl<T: Clone + PartialEq> Holder<T>"),
        "{rust_code}"
    );
    assert!(
        rust_code.contains("impl<T: Clone + std::ops::Mul<Output = T>> Math<T>"),
        "{rust_code}"
    );
    assert!(rust_code.contains("self.value.clone() * other.clone()"));
}

#[test]
fn generic_operator_protocol_impl_uses_generic_target_and_bounds() {
    let rust_code = generate_rust_from_source(
        r#"class Box[T]:
    value: T

    def __eq__(self, other: Box[T]) -> bool:
        return self.value == other.value

class Ordered[T]:
    value: T

    def __lt__(self, other: Ordered[T]) -> bool:
        return self.value < other.value

class NegBox[T]:
    value: T

    def __neg__(self) -> int:
        return 0
"#,
    );

    assert!(
        rust_code.contains("impl<T: Clone + PartialEq> PartialEq for Box<T>"),
        "{rust_code}"
    );
    assert!(rust_code.contains("other: &Box<T>"), "{rust_code}");
    assert!(!rust_code.contains("impl PartialEq for Box"));
    assert!(
        rust_code.contains("impl<T: PartialOrd> PartialOrd for Ordered<T>"),
        "{rust_code}"
    );
    assert!(
        rust_code.contains("impl<T> std::ops::Neg for NegBox<T>"),
        "{rust_code}"
    );
}
