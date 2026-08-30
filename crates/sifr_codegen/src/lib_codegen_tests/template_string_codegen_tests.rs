use super::{generate_rust, lower_module, parse_module};

fn generated(source: &str) -> String {
    let parsed = parse_module(source).expect("source should parse");
    let lowered = lower_module(parsed.suite()).expect("source should lower");
    generate_rust(&lowered.module)
}

#[test]
fn template_codegen_emits_opaque_carrier_and_ordered_single_evaluation() {
    let rust = generated(
        "def consume(value: Template) -> int:\n    return 1\n\ndef make(user_id: int, name: str) -> int:\n    return consume(t\"id={user_id}; name={name!r:>8}\")\n",
    );
    assert!(rust.contains("struct __SifrTemplate"));
    assert!(rust.contains("struct __SifrTemplateInterpolation"));
    assert_eq!(rust.matches("let __sifr_template_value_0").count(), 1);
    assert_eq!(rust.matches("let __sifr_template_value_1").count(), 1);
    let first = rust
        .find("let __sifr_template_value_0")
        .expect("first value");
    let second = rust
        .find("let __sifr_template_value_1")
        .expect("second value");
    assert!(first < second);
    assert!(rust.contains("Box::new(__sifr_template_value_0) as Box<dyn ::std::any::Any>"));
    assert!(
        rust.contains("expression: \"user_id\".to_string()")
            || rust.contains("expression: \"\".to_string()")
    );
}

#[test]
fn template_runtime_is_not_emitted_without_template_types() {
    let rust = generated("def main() -> int:\n    return 1\n");
    assert!(!rust.contains("struct __SifrTemplate"));
}

#[test]
fn nested_format_specs_remain_recursive_runtime_metadata() {
    let rust = generated(
        "def consume(value: Template) -> int:\n    return 1\n\ndef make(amount: float, width: int) -> int:\n    return consume(t\"{amount:>{width:03}}\")\n",
    );
    assert!(rust.contains("enum __SifrTemplateFormatSpecPart"));
    assert!(rust.contains("struct __SifrTemplateFormatSpec"));
    assert!(rust.contains("__SifrTemplateFormatSpecPart::Interpolation"));
    assert_eq!(rust.matches("let __sifr_template_spec_t0_1").count(), 1);
    assert_eq!(
        rust.matches("let __sifr_template_nested_spec_t0_1").count(),
        1
    );
    assert!(rust.contains("value: \"03\".to_string()"));
}
