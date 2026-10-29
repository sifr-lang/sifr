
fn lower_errors(source: &str) -> Vec<crate::HirDiagnostic> {
    let parsed = sifr_python_parser::parse_module(source).expect("source should parse");
    match crate::lower_module(parsed.suite()) {
        Ok(_) => panic!("source should fail lowering"),
        Err(errors) => errors,
    }
}

#[test]
fn map_rejects_optional_elements_for_a_required_callback_parameter() {
    let source = r#"
def values() -> list[int | None]:
    return []

def use_int(value: int) -> int:
    return value + 1

def main():
    for value in map(use_int, values()):
        print(value)
"#;

    let errors = lower_errors(source);

    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(
        errors[0].code,
        Some(sifr_diagnostics::DiagnosticCode::TYPE_MISMATCH)
    );
    assert_eq!(
        errors[0].message,
        "map() callable parameter 1 expects 'int', but iterable 1 yields 'None | int'"
    );
}

#[test]
fn filter_rejects_optional_elements_for_a_required_callback_parameter() {
    let source = r#"
def values() -> list[int | None]:
    return []

def keep_int(value: int) -> bool:
    return value > 0

def main():
    for value in filter(keep_int, values()):
        print(value)
"#;

    let errors = lower_errors(source);

    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(
        errors[0].code,
        Some(sifr_diagnostics::DiagnosticCode::TYPE_MISMATCH)
    );
    assert_eq!(
        errors[0].message,
        "filter() callable parameter 1 expects 'int', but iterable 1 yields 'None | int'"
    );
}
