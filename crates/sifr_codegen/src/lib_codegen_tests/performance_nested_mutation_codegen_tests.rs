use super::*;

#[test]
fn attribute_dict_assignment_mutates_field_in_place() {
    let generated = generate_rust_from_source(
        r#"
class Store:
    rows: dict[str, int]

    def __init__(self):
        self.rows = {}

    def put(self, key: str, value: int) -> None:
        self.rows[key] = value
"#,
    );

    assert!(generated.contains("self.rows.insert(key.clone(), value)"));
    assert!(!generated.contains(".cloned()"));
}

#[test]
fn attribute_list_indexed_dict_assignment_mutates_row_in_place() {
    let generated = generate_rust_from_source(
        r#"
class Store:
    rows: list[dict[str, int]]

    def __init__(self):
        self.rows = []

    def put(self, row: int, key: str, value: int) -> None:
        self.rows[row][key] = value
"#,
    );

    assert!(
        generated.contains("self.rows.get_mut(__oi_norm as usize)"),
        "{generated}"
    );
    assert!(
        generated.contains("let __nested_assign_key = key.clone()"),
        "{generated}"
    );
    assert!(
        generated.contains("__row.insert(__nested_assign_key, __nested_assign_value)"),
        "{generated}"
    );
    assert!(!generated.contains("self.rows.clone()"));
}

#[test]
fn list_indexed_dict_assignment_mutates_row_in_place() {
    let generated = generate_rust_from_source(
        r#"
def put(mut rows: list[dict[str, int]], row: int, key: str, value: int) -> None:
    rows[row][key] = value
"#,
    );

    assert!(
        generated.contains("rows.get_mut(__oi_norm as usize)"),
        "{generated}"
    );
    assert!(
        generated.contains("__row.insert(__nested_assign_key, __nested_assign_value)"),
        "{generated}"
    );
    assert!(!generated.contains("rows.clone()"));
}

#[test]
fn class_method_mut_borrow_helper_call_on_field_emits_mutable_self_receiver() {
    let generated = generate_rust_from_source(
        r#"
def push_value(mut values: list[int], value: int) -> None:
    values.append(value)

def pop_value(mut values: list[int]) -> int | None:
    return values.pop()

class PriorityBuckets:
    lower: list[int]
    upper: list[int]

    def __init__(self):
        self.lower = []
        self.upper = []

    def rebalance(self, value: int) -> None:
        push_value(self.lower, value)
        moved: int | None = pop_value(self.lower)
        if moved is not None:
            push_value(self.upper, moved)
"#,
    );

    assert!(
        generated.contains("fn rebalance(&mut self, value: i64)"),
        "{generated}"
    );
    assert!(
        !generated.contains("fn rebalance(&self, value: i64)"),
        "{generated}"
    );
}

#[test]
fn for_loop_over_proven_dict_list_value_borrows_bucket_for_iteration() {
    let generated = generate_rust_from_source(
        r#"
def collect_bucket(values: dict[str, list[str]], key: str) -> list[str]:
    out: list[str] = []
    if key in values:
        for value in values[key]:
            out.append(value)
    return out
"#,
    );

    assert!(
        generated.contains("let Some(__sifr_dict_iter_source) = values.get"),
        "{generated}"
    );
    assert!(
        generated.contains("__sifr_dict_iter_source.iter().cloned()"),
        "{generated}"
    );
    assert!(!generated.contains("compile_error!"), "{generated}");
}

#[test]
fn recursive_nested_function_does_not_capture_outer_string_char_cache() {
    let generated = generate_rust_from_source(
        r#"
def reaches_end(value: str) -> bool:
    def walk(index: int) -> bool:
        if index == len(value):
            return True
        part: str = value[index:index + 1]
        if part == "":
            return False
        return walk(index + 1)

    for start in range(len(value)):
        if walk(start):
            return True
    return False
"#,
    );

    let walk_start = generated.find("fn walk").expect("walk should lower as fn");
    let outer_loop_start = generated[walk_start..]
        .find("for start")
        .map(|offset| walk_start + offset)
        .expect("outer loop should follow nested function");
    let walk_body = &generated[walk_start..outer_loop_start];
    assert!(
        !walk_body.contains("__sifr_chars_value"),
        "recursive local fn must not capture outer string char cache:\n{generated}"
    );
    assert!(!generated.contains("compile_error!"), "{generated}");
}
