use super::*;

#[test]
fn string_param_indexing_uses_cached_chars() {
    let generated = generate_rust_from_source(
        r#"
def count_marks(text: str) -> int:
    i: int = 0
    total: int = 0
    while i < len(text):
        if text[i] == "x":
            total += 1
        i += 1
    return total
"#,
    );

    assert!(generated.contains("__sifr_chars_text"));
    assert!(generated.contains("__sifr_chars_text.get"), "{generated}");
    assert!(!generated.contains("text.chars().nth"));
    assert!(!generated.contains("text.chars().count()"));
}

#[test]
fn recursive_string_param_indexing_uses_nth_not_per_call_vec_cache() {
    let generated = generate_rust_from_source(
        r#"
def score(mask: int, text: str) -> int:
    if mask <= 1:
        return 0
    if text[0] == text[1]:
        return score(mask - 1, text) + 1
    return score(mask - 1, text)
"#,
    );

    assert!(
        !generated.contains("let mut __sifr_chars_text"),
        "{generated}"
    );
    assert!(generated.contains(".chars().nth"), "{generated}");
}

#[test]
fn string_param_slicing_uses_cached_chars() {
    let generated = generate_rust_from_source(
        r#"
def has_prefix_at(text: str, prefix: str, start: int) -> bool:
    stop: int = start + len(prefix)
    return text[start:stop] == prefix
"#,
    );

    assert!(generated.contains("__sifr_chars_text"));
    assert!(generated.contains("let _slice_src = &__sifr_chars_text"));
    assert!(generated.contains("_slice_src.len()"));
    assert!(!generated.contains("_slice_src.chars().skip"));
    assert!(!generated.contains("_slice_src.chars().count()"));
}

#[test]
fn borrowed_string_param_literal_compare_does_not_clone() {
    let generated = generate_rust_from_source(
        r#"
def is_a(value: str) -> bool:
    return value == "a"
"#,
    );

    assert!(generated.contains("value == \"a\""));
    assert!(!generated.contains("value.clone() == \"a\""));
}

#[test]
fn borrowed_string_param_literal_if_compare_does_not_clone() {
    let generated = generate_rust_from_source(
        r#"
def code(value: str) -> int:
    if value == "a":
        return 1
    return 0
"#,
    );

    assert!(generated.contains("if value == \"a\""), "{generated}");
    assert!(!generated.contains("value.clone() == \"a\""), "{generated}");
}

#[test]
fn readonly_literal_dict_used_for_lookup_is_hoisted_once() {
    let generated = generate_rust_from_source(
        r#"
def is_pair_close(value: str) -> bool:
    pairs = {")": "(", "]": "[", "}": "{"}
    stack = ["("]
    if value not in pairs:
        return False
    return stack[-1] == pairs[value]
"#,
    );

    assert!(
        generated.contains("static __SIFR_HOISTED_DICT_"),
        "{generated}"
    );
    assert!(
        generated.contains("std::sync::LazyLock<HashMap<String, String>>"),
        "{generated}"
    );
    assert!(
        generated.contains("let pairs = &*__SIFR_HOISTED_DICT_"),
        "{generated}"
    );
    assert!(
        !generated.contains("let pairs: HashMap<String, String> = HashMap::from"),
        "{generated}"
    );
    assert!(generated.contains("pairs.get(value)"), "{generated}");
    assert!(
        !generated.contains("stack.get(__sifr_index_norm).cloned()"),
        "{generated}"
    );
    assert!(
        generated.contains("__sifr_cmp_i.normalize_index_or_len"),
        "{generated}"
    );
}

#[test]
fn defaultdict_set_membership_borrows_bucket_without_cloning() {
    let generated = generate_rust_from_source_with_stdlib_collections(
        r#"
from sifr.collections import defaultdict

def seen_twice(value: str) -> bool:
    buckets = defaultdict(set)
    buckets[0].add(value)
    return value in buckets[0]
"#,
    );

    assert!(
        generated.contains(
            "buckets.get(&SifrInt::from_i64(0)).is_some_and(|__sifr_defaultdict_bucket| __sifr_defaultdict_bucket.contains(value))"
        ),
        "{generated}"
    );
    assert!(
        !generated.contains(".or_insert(HashSet::new()).contains"),
        "{generated}"
    );
    assert!(
        !generated.contains("or_insert(HashSet::new()).clone()).contains"),
        "{generated}"
    );
}

#[test]
fn dict_string_lookup_compared_to_local_string_borrows_value() {
    let generated = generate_rust_from_source(
        r#"
def same(mapped: dict[str, str], key: str, value: str) -> bool:
    if key in mapped and mapped[key] != value:
        return False
    return True
"#,
    );

    assert!(generated.contains("mapped.get(key).map"), "{generated}");
    assert!(generated.contains("Some(value)"), "{generated}");
    assert!(generated.contains(".is_some_and("), "{generated}");
}

#[test]
fn large_homogeneous_tuple_hash_key_uses_array_backing() {
    let generated = generate_rust_from_source(
        r#"
def has_key() -> bool:
    values: dict[tuple[int, int, int, int, int, int, int, int, int, int, int, int, int], int] = {}
    key = (1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1)
    values[key] = 7
    return key in values
"#,
    );

    assert!(
        generated.contains("HashMap<[SifrInt; 13], SifrInt>"),
        "{generated}"
    );
    assert!(
        generated.contains("let key: [SifrInt; 13] = [SifrInt::from_i64(1), SifrInt::from_i64(1), SifrInt::from_i64(1)"),
        "{generated}"
    );
    assert!(
        !generated.contains("(SifrInt, SifrInt, SifrInt, SifrInt, SifrInt, SifrInt, SifrInt, SifrInt, SifrInt, SifrInt, SifrInt, SifrInt, SifrInt)"),
        "{generated}"
    );
}

#[test]
fn string_split_and_replace_literal_patterns_do_not_allocate_delimiters() {
    let generated = generate_rust_from_source(
        r#"
def normalize_email(email: str) -> str:
    parts = email.split("@")
    plus_parts = email.split("+")
    clean = email.replace(".", "")
    return clean
"#,
    );

    assert!(generated.contains(".split('@')"), "{generated}");
    assert!(generated.contains(".split('+')"), "{generated}");
    assert!(generated.contains(".replace('.', \"\")"), "{generated}");
    assert!(!generated.contains("\"@\".to_string()"), "{generated}");
    assert!(!generated.contains("\"+\".to_string()"), "{generated}");
    assert!(!generated.contains("\".\".to_string()"), "{generated}");
}

#[test]
fn for_loop_string_target_indexing_uses_body_local_char_cache() {
    let generated = generate_rust_from_source(
        r#"
def count_marks(words: list[str]) -> int:
    total = 0
    for word in words:
        for index in range(len(word)):
            if word[index] == ".":
                total += 1
    return total
"#,
    );

    assert!(
        generated.contains("let __sifr_chars_word: Vec<char> = word.chars().collect"),
        "{generated}"
    );
    assert!(generated.contains("__sifr_chars_word.len()"), "{generated}");
    assert!(generated.contains("__sifr_chars_word.get"), "{generated}");
    assert!(!generated.contains("word.chars().nth"), "{generated}");
    assert!(!generated.contains("word.chars().count()"), "{generated}");
}

#[test]
fn string_local_cache_decl_survives_speculative_if_lowering_rollback() {
    let generated = generate_rust_from_source(
        r#"
def parse_name(template: str) -> int:
    i: int = 0
    if template[0] == "{":
        name: str = ""
        while i < len(template):
            part: str = template[i]
            if part == "}":
                break
            name = name + part
            i += 1
        if len(name) == 0:
            return 0
    return 1
"#,
    );

    assert!(
        generated.contains("let mut __sifr_chars_name: Vec<char> = name.chars().collect"),
        "{generated}"
    );
    assert!(
        generated.contains("__sifr_chars_name.extend"),
        "{generated}"
    );
}

#[test]
fn set_add_moves_last_use_local_string_without_clone() {
    let generated = generate_rust_from_source(
        r#"
def collect(values: list[str]) -> int:
    seen: set[str] = set()
    for value in values:
        normalized = value + "!"
        seen.add(normalized)
    return len(seen)
"#,
    );

    assert!(generated.contains("seen.insert(normalized)"), "{generated}");
    assert!(
        !generated.contains("seen.insert((normalized).clone())"),
        "{generated}"
    );
}

#[test]
fn nested_set_add_preserves_local_used_by_outer_block() {
    let generated = generate_rust_from_source(
        r#"
def collect(values: list[str]) -> int:
    selected: set[str] = set()
    seen: set[str] = set()
    for value in values:
        current = value + "!"
        if current in seen:
            selected.add(current)
        seen.add(current)
    return len(selected)
"#,
    );

    assert!(
        generated.contains("selected.insert((current).clone())"),
        "{generated}"
    );
    assert!(generated.contains("seen.insert(current)"), "{generated}");
}

#[test]
fn break_guard_unwraps_optional_tuple_before_indexing() {
    let generated = generate_rust_from_source(
        r#"
def compute(values: list[int]) -> list[int]:
    output = [0] * len(values)
    stack: list[tuple[int, int]] = []
    for index, value in enumerate(values):
        while stack and value > stack[-1][0]:
            pair: tuple[int, int] | None = stack.pop()
            if pair is None:
                break
            previous_index = pair[1]
            try:
                output[previous_index] = index - previous_index
            except IndexError:
                pass
        stack.append((value, index))
    return output
"#,
    );

    assert!(
        generated.contains("let Some(pair) = pair.clone() else"),
        "{generated}"
    );
    assert!(
        !generated.contains("let Some(mut pair) = pair else"),
        "{generated}"
    );
    assert!(
        generated.contains("let previous_index: SifrInt = (pair).1.clone()"),
        "{generated}"
    );
    assert!(!generated.contains("if pair.is_none()"), "{generated}");
}

#[test]
fn string_concat_expression_uses_push_str_block_not_format_macro() {
    let generated = generate_rust_from_source(
        r#"
def join_pair(left: str, right: str) -> str:
    combined = left + "@" + right
    return combined
"#,
    );

    assert!(generated.contains("String::with_capacity"), "{generated}");
    assert!(
        generated.contains("__sifr_concat.push_str(left)"),
        "{generated}"
    );
    assert!(generated.contains("__sifr_concat.push('@')"), "{generated}");
    assert!(
        generated.contains("__sifr_concat.push_str(right)"),
        "{generated}"
    );
    assert!(!generated.contains("(left).as_str()"), "{generated}");
    assert!(!generated.contains("(right).as_str()"), "{generated}");
    assert!(!generated.contains("format!(\"{}{}{}\""), "{generated}");
}

#[test]
fn counter_from_string_counts_chars_before_materializing_string_keys() {
    let generated = generate_rust_from_source(
        r#"
class Counter:
    counts: dict[str, int]

    def __init__(self, source: dict[str, int] | None, iterable: list[str] | None):
        self.counts = {}

    def __getitem__(self, key: str) -> int:
        return 0

def count_marker(text: str) -> int:
    counts = Counter(None, list(text))
    return counts["x"]
"#,
    );

    assert!(
        generated.contains("let mut __sifr_counter_chars: HashMap<char, usize> = HashMap::new()"),
        "{generated}"
    );
    assert!(
        generated.contains("for __sifr_counter_char in text.chars()"),
        "{generated}"
    );
    assert!(
        generated.contains("__sifr_counter_char.to_string()"),
        "{generated}"
    );
    assert!(
        !generated.contains("text.chars().map(|__sifr_char| __sifr_char.to_string())"),
        "{generated}"
    );
}

#[test]
fn len_of_set_from_string_counts_distinct_chars_without_string_keys() {
    let generated = generate_rust_from_source(
        r#"
def unique_count(text: str, start: int, stop: int) -> int:
    return len(set(text[start:stop]))
"#,
    );

    assert!(
        generated.contains(".chars().collect::<std::collections::HashSet<_>>().len()"),
        "{generated}"
    );
    assert!(
        !generated.contains("to_string()).collect::<std::collections::HashSet<_>>().len()"),
        "{generated}"
    );
}

#[test]
fn set_from_string_materializes_only_distinct_string_keys() {
    let generated = generate_rust_from_source(
        r#"
def unique_values(text: str) -> int:
    values = set(text)
    return len(values)
"#,
    );

    assert!(
        generated.contains("let mut __sifr_set_chars: HashSet<char> = HashSet::new()"),
        "{generated}"
    );
    assert!(
        generated.contains("let mut __sifr_set_strings: HashSet<String> = HashSet::new()"),
        "{generated}"
    );
    assert!(
        !generated.contains("text.chars().map(|__sifr_char| __sifr_char.to_string())"),
        "{generated}"
    );
}

#[test]
fn string_loop_set_membership_uses_char_keys_without_string_allocations() {
    let generated = generate_rust_from_source(
        r#"
def partitions(text: str) -> int:
    count = 0
    seen = set()
    for ch in text:
        if ch in seen:
            count += 1
            seen = set()
        seen.add(ch)
    return count + 1
"#,
    );

    assert!(generated.contains("for ch in text.chars()"), "{generated}");
    assert!(generated.contains("seen.contains(&ch)"), "{generated}");
    assert!(!generated.contains("text.chars().map"), "{generated}");
    assert!(!generated.contains("ch.to_string()"), "{generated}");
}

#[test]
fn local_string_slicing_uses_chars_without_copied() {
    let generated = generate_rust_from_source(
        r#"
def trim_one(text: str) -> str:
    local: str = "abc"
    return local[:len(local) - 1]
"#,
    );

    assert!(generated.contains("_slice_src.chars().skip"));
    assert!(generated.contains("let _slice_src = &local"));
    assert!(!generated.contains("_slice_src.chars().skip(_slice_start_i64 as usize).take((_slice_stop_i64 - _slice_start_i64).max(0) as usize).copied()"));
}

#[test]
fn string_self_slice_assignment_borrows_slice_source() {
    let generated = generate_rust_from_source(
        r#"
def shrink() -> str:
    value: str = "abcd"
    value = value[:len(value) - 1]
    return value
"#,
    );

    assert!(generated.contains("let _slice_src = &value"), "{generated}");
    assert!(
        !generated.contains("let _slice_src = value;"),
        "{generated}"
    );
}

#[test]
fn full_reverse_string_slice_lowers_to_linear_reverse_collect() {
    let generated = generate_rust_from_source(
        r#"
def reversed_text(value: str) -> str:
    return value[::-1]
"#,
    );

    assert!(
        generated.contains(".chars().rev().collect::<String>()"),
        "{generated}"
    );
    assert!(!generated.contains(".chars().nth"), "{generated}");
}

#[test]
fn self_string_concat_assignment_lowers_to_push_str() {
    let generated = generate_rust_from_source(
        r##"
def join_marked(parts: list[str]) -> str:
    out: str = ""
    for part in parts:
        out = out + "#" + part
    return out
"##,
    );

    assert!(generated.contains("out.push('#')"), "{generated}");
    assert!(generated.contains("out.push_str(part.as_str())"));
    assert!(!generated.contains("out = format!"));
}

#[test]
fn self_string_concat_assignment_with_self_rhs_materializes_clone() {
    let generated = generate_rust_from_source(
        r#"
def doubled(own mut value: str) -> str:
    value = value + value
    return value
"#,
    );

    assert!(
        generated.contains("let __sifr_string_concat_value_0 = value.clone()"),
        "{generated}"
    );
    assert!(
        generated.contains("value.push_str(__sifr_string_concat_value_0.as_str())"),
        "{generated}"
    );
    assert!(!generated.contains("value.push_str(value.as_str())"));
}

#[test]
fn mutated_string_without_char_access_does_not_initialize_char_cache() {
    let generated = generate_rust_from_source(
        r#"
def strip_dots(value: str) -> str:
    local_name = value
    local_name = local_name.replace(".", "")
    return local_name
"#,
    );

    assert!(
        !generated.contains("__sifr_chars_local_name"),
        "{generated}"
    );
}

#[test]
fn mutated_local_string_indexing_uses_updated_char_cache() {
    let generated = generate_rust_from_source(
        r#"
def count_a(own mut value: str) -> int:
    value = value + value
    i: int = 0
    total: int = 0
    while i < len(value):
        if value[i] == "a":
            total += 1
        i += 1
    return total
"#,
    );

    assert!(
        generated.contains("let mut __sifr_chars_value"),
        "{generated}"
    );
    assert!(
        generated.contains("__sifr_chars_value.extend"),
        "{generated}"
    );
    assert!(
        generated.contains("__sifr_chars_value.get(__sifr_string_index_normalized)"),
        "{generated}"
    );
    assert!(!generated.contains("value.chars().nth"), "{generated}");
    assert!(!generated.contains("value.chars().count"), "{generated}");
}

#[test]
fn tuple_unpacked_mutated_strings_initialize_char_caches() {
    let generated = generate_rust_from_source(
        r#"
def count_marks(limit: int) -> int:
    left, right = "", ""
    for index in range(limit):
        left += "x"
        right += "y"
    return 1 if left[0] == "x" and right[0] == "y" else 0
"#,
    );

    assert!(
        generated.contains("let mut __sifr_chars_left"),
        "{generated}"
    );
    assert!(
        generated.contains("let mut __sifr_chars_right"),
        "{generated}"
    );
    assert!(
        generated.contains("__sifr_chars_left.extend"),
        "{generated}"
    );
    assert!(
        generated.contains("__sifr_chars_right.extend"),
        "{generated}"
    );
    assert!(generated.contains("__sifr_chars_left.get"), "{generated}");
    assert!(generated.contains("__sifr_chars_right.get"), "{generated}");
    assert!(!generated.contains("left.chars().nth"), "{generated}");
    assert!(!generated.contains("right.chars().nth"), "{generated}");
}

#[test]
fn string_index_compare_uses_cached_chars_without_string_allocations() {
    let generated = generate_rust_from_source(
        r#"
def same_at(text: str, left: int, right: int) -> bool:
    return text[left] == text[right]
"#,
    );

    assert!(generated.contains("__sifr_chars_text"), "{generated}");
    assert!(generated.contains(".copied()"), "{generated}");
    assert!(!generated.contains("map(|c| c.to_string())"), "{generated}");
}

#[test]
fn single_element_list_repeat_uses_std_repeat_not_extend_loop() {
    let generated = generate_rust_from_source(
        r#"
def zeros(n: int) -> list[int]:
    return [0] * n
"#,
    );

    assert!(
        generated.contains("std::iter::repeat(SifrInt::from_i64(0))"),
        "{generated}"
    );
    assert!(generated.contains("collect::<Vec<_>>()"), "{generated}");
    assert!(
        !generated.contains("__sifr_repeat_out.extend"),
        "{generated}"
    );
}

#[test]
fn dict_indexed_list_append_mutates_bucket_in_place() {
    let generated = generate_rust_from_source(
        r#"
def bucketize(values: list[int]) -> dict[int, list[int]]:
    buckets: dict[int, list[int]] = {}
    for value in values:
        key: int = value
        if key in buckets:
            buckets[key].append(value)
        else:
            buckets[key] = [value]
    return buckets
"#,
    );

    assert!(generated.contains("buckets.get_mut("), "{generated}");
    assert!(generated.contains("__elem.push"));
    assert!(!generated.contains("__sifr_proven_dict_value).push"));
}

#[test]
fn dict_indexed_list_append_clones_borrowed_string_inside_tuple_payload() {
    let generated = generate_rust_from_source(
        r#"
class Store:
    values: dict[str, list[tuple[str, int]]]

    def __init__(self):
        self.values = {}

    def put(mut self, key: str, value: str, timestamp: int) -> None:
        if key not in self.values:
            self.values[key] = []
        if key in self.values:
            self.values[key].append((value, timestamp))
"#,
    );

    assert!(generated.contains("fn put(&mut self, key: &str, value: &str, timestamp: &SifrInt)"));
    assert!(
        generated.contains("__elem.push((value.to_owned(), timestamp.clone()))"),
        "{generated}"
    );
    assert!(
        !generated.contains("__elem.push((value, timestamp))"),
        "{generated}"
    );
}

#[test]
fn dict_field_delete_mutates_field_in_place() {
    let generated = generate_rust_from_source(
        r#"
class Cache:
    entries: dict[int, int]

    def __init__(self):
        self.entries = {}

    def remove(mut self, key: int) -> None:
        try:
            del self.entries[key]
        except KeyError:
            pass
"#,
    );

    assert!(
        generated.contains("self.entries.remove(key)"),
        "{generated}"
    );
    assert!(!generated.contains("self.entries.clone().remove"));
}

#[test]
fn dict_indexed_list_pop_mutates_bucket_in_place() {
    let generated = generate_rust_from_source(
        r#"
class Buckets:
    values: dict[int, list[int]]

    def __init__(self):
        self.values = {}

    def push(mut self, key: int, value: int) -> None:
        if key not in self.values:
            self.values[key] = []
        if key in self.values:
            self.values[key].append(value)

    def take(mut self, key: int) -> int:
        if key in self.values:
            popped = self.values[key].pop()
            if popped is not None:
                if len(self.values[key]) == 0:
                    return popped
                return popped
        return 0
"#,
    );

    assert!(generated.contains("self.values.get_mut(&key)"));
    assert!(generated.contains("__sifr_bucket.pop()"));
    assert!(generated.contains("self.values.get(&key).map_or"));
    assert!(generated.contains("if let Some(__sifr_checked_value_"));
    assert!(generated.contains("self.values.get(key)"));
    assert!(!generated.contains(".entry("), "{generated}");
    assert!(!generated.contains("self.values.clone().get_mut"));
    assert!(!generated.contains("self.values.clone().contains_key"));
    assert!(!generated.contains("(self.values.clone()).contains_key"));
    assert!(!generated.contains("self.values.get(&key).cloned().unwrap_or"));
}

#[test]
fn dict_indexed_list_read_borrows_bucket_without_cloning() {
    let generated = generate_rust_from_source(
        r#"
class Tweets:
    values: dict[int, list[tuple[int, int]]]

    def __init__(self):
        self.values = {}

    def at(self, key: int, index: int) -> tuple[int, int]:
        if key in self.values:
            value = self.values[key][index]
            if value is not None:
                return value
        return (0, 0)
"#,
    );

    assert!(generated.contains(".get("));
    assert!(
        generated.contains(
            "__sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()"
        ),
        "{generated}"
    );
    assert!(
        !generated.contains("self.values.clone().get"),
        "{generated}"
    );
    assert!(!generated.contains("self.values.get(key).cloned()"));
}

#[test]
fn nested_list_read_borrows_row_without_cloning() {
    let generated = generate_rust_from_source(
        r#"
def cell_at(grid: list[list[int]], row: int, col: int) -> int:
    value: int | None = grid[row][col]
    if value is None:
        return 0
    return value
"#,
    );

    assert!(generated.contains("__sifr_outer_list"), "{generated}");
    assert!(generated.contains("__sifr_row.get"));
    assert!(!generated.contains("__sifr_index_value).cloned()"));
    assert!(!generated.contains(".as_ref().and_then(|__v| __v.get"));
}

#[test]
fn unscoped_optional_binop_declines_expression_level_discharge() {
    let lowered = crate::stmt_support_emitter::binop_with_optional_operands(
        RustExpr::Literal(crate::RustLiteral::Int(1)),
        RustExpr::Ident("maybe_cell".to_string()),
        "+",
        &Type::Int,
        &Type::Union(vec![Type::Int, Type::None]),
        &Type::Int,
    );
    assert!(lowered.is_none());
}
