use super::*;

#[test]
fn guarded_dict_list_index_unwraps_present_value() {
    let generated = generate_rust_from_source(
        r#"
def values_for(grouped: dict[int, list[str]], key: int) -> list[str]:
    if key not in grouped:
        return []
    return grouped[key]
"#,
    );
    assert!(
        generated.contains("let Some(__sifr_checked_value_0) = grouped.get(&key) else"),
        "{generated}"
    );
    assert!(generated.contains("(*__sifr_checked_value_0).clone()"));
    assert!(!generated.contains("grouped.get(&key).cloned()"));
    assert!(!generated.contains("grouped[&key]"));
    assert!(!generated.contains("abort()"));
}

#[test]
fn guarded_list_index_uses_a_structural_read_witness() {
    let generated = generate_rust_from_source(
        r#"
def head_or_zero(values: list[int]) -> int:
    if len(values) == 0:
        return 0
    return values[0]
"#,
    );
    assert!(generated.contains("let Some(__sifr_checked_value_0) = ({"));
    assert!(
        generated.contains("__sifr_checked_read_collection.get(__sifr_checked_read_normalized)")
    );
    assert!(generated.contains("__sifr_checked_value_0.clone()"));
    assert!(!generated.contains("as_slice()["));
}

#[test]
fn positive_list_guard_scopes_a_structural_read_witness() {
    let generated = generate_rust_from_source(
        r#"
def head_or_zero(values: list[int]) -> int:
    if values:
        return values[0]
    return 0
"#,
    );
    assert!(generated.contains("if !values.is_empty()"));
    assert!(generated.contains("if let Some(__sifr_checked_value_0) = {"));
    assert!(generated.contains("__sifr_checked_value_0.clone()"));
    assert!(!generated.contains("as_slice()["));
}

#[test]
fn while_bound_scopes_a_structural_read_witness() {
    let generated = generate_rust_from_source(
        r#"
def sum_values(values: list[int]) -> int:
    total: int = 0
    index: int = 0
    while index < len(values):
        total = total + values[index]
        index = index + 1
    return total
"#,
    );
    assert!(generated.contains("let Some(__sifr_checked_value_0) = ({"));
    assert!(generated.contains("else {\n            break;\n        }"));
    assert!(!generated.contains("as_slice()["));
}

#[test]
fn range_len_loop_scopes_a_structural_read_witness() {
    let generated = generate_rust_from_source(
        r#"
def sum_values(values: list[int]) -> int:
    total: int = 0
    for index in range(len(values)):
        total = total + values[index]
    return total
"#,
    );
    assert!(generated.contains("let Some(__sifr_checked_value_0) = ({"));
    assert!(generated.contains("else {\n            continue;\n        }"));
    assert!(!generated.contains("as_slice()["));
}

#[test]
fn nested_list_string_compare_borrows_row_without_temporary_clone() {
    let generated = generate_rust_from_source(
        r#"
def cell_matches(mut board: list[list[str]], row: int, col: int, value: str) -> bool:
    while row >= 0 and row < len(board) and col >= 0 and col < len(board[0]):
        if board[row][col] == value:
            return True
        break
    return False
"#,
    );
    assert!(generated.contains("__sifr_cmp_outer_list.get(__sifr_cmp_outer_norm).and_then"));
    assert!(!generated.contains("&board[row as usize].clone()"));
    assert!(!generated.contains("__sifr_cmp_outer_list["));
}

#[test]
fn short_circuit_conditions_lower_checked_reads_inside_their_boolean_operand() {
    let generated = generate_rust_from_source(
        r#"
def observed(values: list[int], mapping: dict[str, int]) -> bool:
    if len(values) > 3 and values[3] != 0:
        return True
    if "ready" in mapping and mapping["ready"] > 0:
        return True
    return len(values) <= 3 or values[3] != 0
"#,
    );
    assert!(
        generated.matches(".is_some_and(").count() >= 3,
        "{generated}"
    );
    assert!(generated.contains("values.len()"));
    assert!(generated.contains("mapping.get(\"ready\")"));
    assert!(!generated.contains("values["));
    assert!(!generated.contains("mapping["));
    assert!(!generated.contains("let Some("));
    assert!(!generated.contains("compile_error!"));
}

#[test]
fn minimum_length_exit_guard_witnesses_each_literal_read() {
    let generated = generate_rust_from_source(
        r#"
def first_three(values: list[int]) -> int:
    if len(values) < 3:
        return 0
    return values[0] + values[1] + values[2]
"#,
    );
    assert_eq!(generated.matches("let Some(").count(), 3);
    assert!(generated.contains("else {\n        return SifrInt::from_i64(0);"));
    assert!(!generated.contains("values["));
    assert!(!generated.contains("compile_error!"));
}

#[test]
fn repeated_checked_condition_reads_do_not_move_the_index() {
    let generated = generate_rust_from_source(
        r#"
def equal(actual: list[int], expected: list[int]) -> bool:
    i: int = 0
    while i < len(actual) and i < len(expected):
        if actual[i] != expected[i]:
            return False
        i = i + 1
    return len(actual) == len(expected)
"#,
    );
    assert!(
        generated
            .matches("let __sifr_checked_read_index = i.clone();")
            .count()
            >= 2
    );
    assert!(!generated.contains("let __sifr_checked_read_index = i;"));
    assert!(!generated.contains("compile_error!"));
}

#[test]
fn repeated_optional_assert_reads_do_not_move_the_index() {
    let generated = generate_rust_from_source(
        r#"
def equal(actual: list[int], expected: list[int]):
    i: int = 0
    while i < len(actual):
        assert actual[i] == expected[i]
        i = i + 1
"#,
    );
    assert!(
        generated
            .matches("let __sifr_condition_index = i.clone();")
            .count()
            >= 2
    );
    assert!(!generated.contains("let __sifr_condition_index = i;"));
}

#[test]
fn optional_destinations_lower_contextually_non_optional_indexes_to_checked_gets() {
    let generated = generate_rust_from_source(
        r#"
def lookup(values: list[str], mapping: dict[str, str], index: int, key: str) -> str | None:
    size: int = len(values)
    if index < size:
        item: str | None = values[index]
        if item is not None:
            return item
    mapped: str | None = mapping[key]
    return mapped

def last_value(result: list[str]) -> str | None:
    if len(result) == 0:
        return None
    last_index: int = len(result) - 1
    last_opt: str | None = result[last_index]
    return last_opt
"#,
    );
    assert!(
        generated.contains(".get(__sifr_checked_read_normalized)"),
        "{generated}"
    );
    assert!(
        generated.contains("mapping.get(key).cloned()"),
        "{generated}"
    );
    assert!(!generated.contains("compile_error!"));
    assert!(!generated.contains("values["));
    assert!(!generated.contains("mapping["));
}

#[test]
fn optional_checked_indexes_in_ordering_comparisons_use_present_values() {
    let generated = generate_rust_from_source(
        r#"
def valid_usage(usage: list[int]) -> bool:
    checks: list[bool] = []
    checks.append(len(usage) == 3 and usage[0] > 0 and usage[1] >= 0 and usage[2] >= 0)
    return len(checks) == 1
"#,
    );
    assert_eq!(generated.matches(".is_some_and(").count(), 3, "{generated}");
    assert!(!generated.contains(".cloned() >"), "{generated}");
    assert!(!generated.contains(".cloned() >="), "{generated}");
    assert!(!generated.contains("compile_error!"), "{generated}");
}

#[test]
fn nested_projection_conditions_witness_the_optional_parent_read() {
    let generated = generate_rust_from_source(
        r#"
def validate(items: list[tuple[str, str]]):
    assert items[1][0] == "accept"
    assert items[1][1].upper() == "JSON"
"#,
    );
    assert_eq!(generated.matches(".is_some_and(").count(), 2, "{generated}");
    assert!(!generated.contains("items["), "{generated}");
    assert!(!generated.contains("compile_error!"), "{generated}");
}
