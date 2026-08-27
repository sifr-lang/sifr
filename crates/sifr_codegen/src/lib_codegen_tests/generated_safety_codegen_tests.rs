use super::generate_rust_from_source;

#[test]
fn user_callables_named_like_safety_sentinels_are_preserved() {
    let rust_code = generate_rust_from_source(
        r#"class Wrapper:
    value: int

    def unwrap(self) -> int:
        return self.value

def expect(value: int) -> int:
    return value

def main():
    wrapper = Wrapper(7)
    result: int = wrapper.unwrap()
    other: int = expect(result)
    values: list[int] = []
    values.append(wrapper.unwrap())
"#,
    );

    assert!(rust_code.contains("fn r#unwrap("), "{rust_code}");
    assert!(rust_code.contains(".r#unwrap()"), "{rust_code}");
    assert!(rust_code.contains("fn r#expect("), "{rust_code}");
    assert!(rust_code.contains("r#expect(result)"), "{rust_code}");
    assert!(
        rust_code.contains("values.push(wrapper.r#unwrap())"),
        "{rust_code}"
    );
}
