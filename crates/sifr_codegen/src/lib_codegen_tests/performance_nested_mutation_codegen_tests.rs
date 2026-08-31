use super::*;

#[test]
fn attribute_dict_assignment_mutates_field_in_place() {
    let generated = generate_rust_from_source(
        r#"
class Store:
    rows: dict[str, int]

    def __init__(self):
        self.rows = {}

    def put(mut self, key: str, value: int) -> None:
        self.rows[key] = value
"#,
    );

    assert!(generated.contains("let __assign_key = key.to_owned()"));
    assert!(generated.contains("self.rows.insert(__assign_key, __assign_value)"));
    assert!(!generated.contains(".cloned()"));
}

#[test]
fn guarded_attribute_list_assignment_carries_a_static_presence_proof() {
    let generated = generate_rust_from_source(
        r#"
class Store:
    values: list[int]

    def __init__(self):
        self.values = [1]

    def put(mut self, index: int, value: int) -> None:
        if 0 <= index and index < len(self.values):
            self.values[index] = value
"#,
    );

    assert!(generated.contains("self.values.get_mut(__index_normalized)"));
    assert!(!generated.contains("IndexError::new"), "{generated}");
}

#[test]
fn attribute_list_indexed_dict_assignment_mutates_row_in_place() {
    let generated = generate_rust_from_source(
        r#"
class Store:
    rows: list[dict[str, int]]

    def __init__(self):
        self.rows = []

    def put(mut self, row: int, key: str, value: int) -> None:
        try:
            self.rows[row][key] = value
        except IndexError:
            pass
"#,
    );

    assert!(
        generated.contains("self.rows.get_mut(__outer_normalized)"),
        "{generated}"
    );
    assert!(
        generated.contains("let __inner_key = key.to_owned()"),
        "{generated}"
    );
    assert!(
        generated.contains("__row.insert(__inner_key, __nested_assign_value)"),
        "{generated}"
    );
    assert!(!generated.contains("self.rows.clone()"));
}

#[test]
fn list_indexed_dict_assignment_mutates_row_in_place() {
    let generated = generate_rust_from_source(
        r#"
def put(mut rows: list[dict[str, int]], row: int, key: str, value: int) -> None:
    try:
        rows[row][key] = value
    except IndexError:
        pass
"#,
    );

    assert!(
        generated.contains("rows.get_mut(__outer_normalized)"),
        "{generated}"
    );
    assert!(
        generated.contains("__row.insert(__inner_key, __nested_assign_value)"),
        "{generated}"
    );
    assert!(!generated.contains("rows.clone()"));
}

#[test]
fn dict_indexed_list_assignment_checks_key_then_index() {
    let generated = generate_rust_from_source(
        r#"
def put(mut rows: dict[str, list[int]], key: str, column: int, value: int) -> None:
    try:
        rows[key][column] = value
    except KeyError:
        pass
    except IndexError:
        pass
"#,
    );

    let outer = generated
        .find("rows.get_mut(&__outer_key)")
        .expect("outer dict projection should be checked");
    let inner = generated
        .find("__row.get_mut(__inner_normalized)")
        .expect("inner list projection should be checked");
    assert!(outer < inner, "{generated}");
    assert!(generated.contains("KeyError::new"), "{generated}");
    assert!(generated.contains("IndexError::new"), "{generated}");
    assert!(
        !generated.contains(".to_string().to_string()"),
        "{generated}"
    );
}

#[test]
fn list_indexed_dict_assignment_checks_outer_index_and_upserts_inner_key() {
    let generated = generate_rust_from_source(
        r#"
def put(mut rows: list[dict[str, int]], row: int, key: str, value: int) -> None:
    try:
        rows[row][key] = value
    except IndexError:
        pass
"#,
    );

    assert!(generated.contains("rows.get_mut(__outer_normalized)"));
    assert!(
        generated.contains("__row.insert(__inner_key, __nested_assign_value)"),
        "{generated}"
    );
    assert!(generated.contains("IndexError::new"), "{generated}");
    assert!(!generated.contains("KeyError::new"), "{generated}");
}

#[test]
fn list_indexed_dict_augassign_checks_index_then_existing_key() {
    let generated = generate_rust_from_source(
        r#"
def add(mut rows: list[dict[str, int]], row: int, key: str, value: int) -> None:
    try:
        rows[row][key] += value
    except IndexError:
        pass
    except KeyError:
        pass
"#,
    );

    let outer = generated
        .find("rows.get_mut(__outer_normalized)")
        .expect("outer list projection should be checked");
    let inner = generated
        .find("__row.get_mut(&__inner_key)")
        .expect("inner dict projection should be checked");
    assert!(outer < inner, "{generated}");
    assert!(generated.contains("IndexError::new"), "{generated}");
    assert!(generated.contains("KeyError::new"), "{generated}");
}

#[test]
fn dict_indexed_list_augassign_checks_key_then_index() {
    let generated = generate_rust_from_source(
        r#"
def add(mut rows: dict[str, list[int]], key: str, column: int, value: int) -> None:
    try:
        rows[key][column] += value
    except KeyError:
        pass
    except IndexError:
        pass
"#,
    );

    let outer = generated
        .find("rows.get_mut(&__outer_key)")
        .expect("outer dict projection should be checked");
    let inner = generated
        .find("__row.get_mut(__inner_normalized)")
        .expect("inner list projection should be checked");
    assert!(outer < inner, "{generated}");
    assert!(generated.contains("KeyError::new"), "{generated}");
    assert!(generated.contains("IndexError::new"), "{generated}");
}

#[test]
fn attribute_dict_indexed_list_assignment_uses_the_same_projection_plan() {
    let generated = generate_rust_from_source(
        r#"
class Store:
    rows: dict[str, list[int]]

    def __init__(self):
        self.rows = {}

    def put(mut self, key: str, column: int, value: int) -> None:
        try:
            self.rows[key][column] = value
        except KeyError:
            pass
        except IndexError:
            pass
"#,
    );

    assert!(generated.contains("self.rows.get_mut(&__outer_key)"));
    assert!(generated.contains("__row.get_mut(__inner_normalized)"));
    assert!(generated.contains("KeyError::new"), "{generated}");
    assert!(generated.contains("IndexError::new"), "{generated}");
}

#[test]
fn list_write_and_delete_share_negative_index_normalization_and_index_error() {
    let generated = generate_rust_from_source(
        r#"
def update(mut values: list[int], index: int, value: int) -> None:
    try:
        values[index] = value
    except IndexError:
        pass
    try:
        del values[index]
    except IndexError:
        pass
"#,
    );

    assert!(
        generated.contains("normalize_index_or_len(values.len())")
            && generated.contains("normalize_index_or_len(__delete_target.len())"),
        "{generated}"
    );
    assert!(generated.contains("values.get_mut(__index_normalized)"));
    assert!(generated.contains("__delete_target.remove(__idx_norm)"));
    assert!(generated.matches("IndexError::new").count() >= 2);
    assert!(
        !generated.contains(".to_string().to_string()"),
        "{generated}"
    );
}

#[test]
fn starred_unpack_uses_a_refutable_slice_pattern_and_value_error() {
    let generated = generate_rust_from_source(
        r#"
def split(values: list[int]) -> None:
    try:
        first, *middle, last = values
    except ValueError:
        pass
"#,
    );

    assert!(
        generated.contains("let [__sifr_before_0, __sifr_star @ .., __sifr_after_0] = __sifr_unpack_source.as_slice() else"),
        "{generated}"
    );
    assert!(generated.contains("ValueError::new"), "{generated}");
    assert!(generated.contains("__sifr_star.to_vec()"), "{generated}");
    assert!(!generated.contains(".try_into()"), "{generated}");
}

#[test]
fn starred_unpack_borrows_a_reused_named_source() {
    let generated = generate_rust_from_source(
        r#"
def split(values: list[int]) -> int:
    try:
        first, *middle, last = values
        assert first <= last
    except ValueError:
        pass
    return len(values)
"#,
    );

    assert!(
        generated.contains("let __sifr_unpack_source = &values"),
        "{generated}"
    );
    assert!(!generated.contains("let __sifr_unpack_source = values.clone()"));
}

#[test]
fn starred_unpack_assigns_predeclared_targets_in_a_try_closure() {
    let generated = generate_rust_from_source(
        r#"
def split(values: list[int]) -> int:
    first = 0
    middle: list[int] = []
    try:
        first, *middle = values
    except ValueError:
        pass
    return first + len(middle)
"#,
    );

    assert!(generated.contains("let mut first: SifrInt"), "{generated}");
    assert!(
        generated.contains("let mut middle: Vec<SifrInt>"),
        "{generated}"
    );
    assert!(
        generated.contains("first = __sifr_before_0.clone();"),
        "{generated}"
    );
    assert!(
        generated.contains("middle = __sifr_star.to_vec();"),
        "{generated}"
    );
    assert!(!generated.contains("let first = __sifr_before_0"));
    assert!(!generated.contains("let middle = __sifr_star.to_vec()"));
}

#[test]
fn repeated_unpack_discards_remain_immutable_wildcard_patterns() {
    let generated = generate_rust_from_source(
        r#"
def sizes(values: list[tuple[int, int, int]]) -> int:
    total = 0
    for value in values:
        _, _, size = value
        total += size
    return total
"#,
    );

    assert!(
        generated.contains("let (_, _, size) = value;"),
        "{generated}"
    );
    assert!(!generated.contains("let mut _"), "{generated}");
    assert!(
        !generated.contains("_ = __sifr_tuple_unpack"),
        "{generated}"
    );
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

    def rebalance(mut self, value: int) -> None:
        push_value(self.lower, value)
        moved: int | None = pop_value(self.lower)
        if moved is not None:
            push_value(self.upper, moved)
"#,
    );

    assert!(
        generated.contains("fn rebalance(&mut self, value: &SifrInt)"),
        "{generated}"
    );
    assert!(
        !generated.contains("fn rebalance(&self, value: &SifrInt)"),
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
        generated.contains("if let Some(__sifr_checked_value_0) = values.get(key)"),
        "{generated}"
    );
    assert!(
        generated.contains("__sifr_checked_value_0.iter().cloned()"),
        "{generated}"
    );
    assert!(!generated.contains("values.get(key).cloned()"));
    assert!(!generated.contains("values[&key]"), "{generated}");
    assert!(!generated.contains("unreachable!"), "{generated}");
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
