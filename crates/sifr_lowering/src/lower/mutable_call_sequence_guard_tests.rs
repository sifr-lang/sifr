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

#[test]
fn user_defined_mutable_receiver_invalidates_field_guard() {
    let source = r#"
class Bucket:
    values: list[int]

    def clear(mut self) -> None:
        self.values.clear()

def read_after_method(mut bucket: Bucket) -> int:
    if len(bucket.values) == 0:
        return 0
    bucket.clear()
    return bucket.values[0] + 1
"#;
    let diagnostics = lower_source(source).expect_err("mut self must invalidate field guards");
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR)
            && diagnostic.message.contains("None")
    }));
}

#[test]
fn builtin_shrinking_receiver_invalidates_sequence_guard() {
    let source = r#"
def read_after_pop(mut values: list[int]) -> int:
    if len(values) == 0:
        return 0
    values.pop()
    return values[0] + 1
"#;
    let diagnostics = lower_source(source).expect_err("pop must invalidate sequence guards");
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR)
            && diagnostic.message.contains("None")
    }));
}

#[test]
fn builtin_growth_and_reorder_preserve_nonempty_list_guard() {
    let source = r#"
def read_after_growth_and_reorder(mut values: list[int]) -> int:
    if len(values) == 0:
        return 0
    values.append(4)
    values.insert(0, 3)
    values.extend([2, 1])
    values.sort()
    values.reverse()
    return values[0] + 1
"#;
    let result = lower_source(source);
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn dict_value_mutation_preserves_existing_membership_guard() {
    let source = r#"
def read_after_setdefault(mut values: dict[str, int]) -> int:
    if "a" not in values:
        return 0
    values.setdefault("b", 2)
    values.update({"c": 3})
    return values["a"] + 1
"#;
    let result = lower_source(source);
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn builtin_clear_and_remove_invalidate_sequence_guards() {
    let source = r#"
def read_after_clear(mut values: list[int]) -> int:
    if len(values) == 0:
        return 0
    values.clear()
    return values[0] + 1

def read_after_remove(mut values: list[int]) -> int:
    if len(values) == 0:
        return 0
    values.remove(values[0])
    return values[0] + 1
"#;
    let diagnostics = lower_source(source).expect_err("removal must invalidate sequence guards");
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == Some(DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR)
            && diagnostic.message.contains("None")
    }));
}
