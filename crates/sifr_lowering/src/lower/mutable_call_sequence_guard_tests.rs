use super::expressions_tests::lower_source;
use sifr_diagnostics::DiagnosticCode;

#[test]
fn mutable_call_invalidates_straight_line_sequence_guard() {
    let source = r#"
def clear_values(mut values: list[int]) -> None:
    values.clear()

def read_after_call(mut values: list[int]) -> int:
    if len(values) == 0:
        return 0
    clear_values(values)
    return values[0] + 1
"#;
    let diagnostics = lower_source(source).expect_err("mutable call must invalidate the guard");
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR)
            && diagnostic.message.contains("None")
    }));
}

#[test]
fn nested_mutable_call_uses_current_optional_contract() {
    let source = r#"
def clear_values(mut values: list[int]) -> None:
    values.clear()

def read_in_nested_region(mut values: list[int]) -> int:
    if len(values) == 0:
        return 0
    total: int = 0
    while values[0] > 0:
        for index in range(1):
            if index == 0:
                clear_values(values)
                total += values[0]
    else:
        total += 1
    return total
"#;
    let diagnostics = lower_source(source).expect_err("nested call must invalidate the guard");
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR)
            && diagnostic.message.contains("None")
    }));
}

#[test]
fn explicit_reguard_after_mutable_call_restores_checked_read() {
    let source = r#"
def replace_first(mut values: list[int]) -> None:
    if len(values) > 0:
        values[0] = 42

def read_after_reguard(mut values: list[int]) -> int:
    if len(values) == 0:
        return 0
    replace_first(values)
    if len(values) == 0:
        return 0
    return values[0]
"#;
    let result = lower_source(source);
    assert!(result.is_ok(), "{result:?}");
}
