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
