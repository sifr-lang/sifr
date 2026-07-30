use super::generate_rust_from_source_with_stdlib_collections;

#[test]
fn read_before_write_defaultdict_set_has_concrete_declaration_codegen() {
    let rust_code = generate_rust_from_source_with_stdlib_collections(
        "from sifr.collections import defaultdict\n\ndef solve(cells: list[tuple[int, str]]) -> bool:\n    rows = defaultdict(set)\n    for row, cell in cells:\n        if cell in rows[row]:\n            return False\n        rows[row].add(cell)\n    return True\n",
    );

    assert!(rust_code.contains("let mut rows: HashMap<i64, HashSet<String>> = HashMap::new();"));
}

#[test]
fn tuple_key_defaultdict_set_has_concrete_declaration_codegen() {
    let rust_code = generate_rust_from_source_with_stdlib_collections(
        "from sifr.collections import defaultdict\n\ndef solve(cell: str) -> int:\n    squares = defaultdict(set)\n    square = (1, 2)\n    if cell in squares[square]:\n        return 0\n    squares[square].add(cell)\n    return len(squares)\n",
    );

    assert!(rust_code
        .contains("let mut squares: HashMap<(i64, i64), HashSet<String>> = HashMap::new();"));
}

#[test]
fn list_slice_append_uses_defaultdict_entry_insertion() {
    let rust_code = generate_rust_from_source_with_stdlib_collections(
        "from sifr.collections import defaultdict\n\ndef solve(values: list[int]) -> int:\n    groups = defaultdict(list)\n    groups[1].append(values[0:2])\n    return len(groups[1])\n",
    );

    assert!(rust_code.contains(".entry(1_i64).or_insert(Vec::new()).push("));
    assert!(!rust_code.contains("groups.get_mut("));
}

#[test]
fn string_slice_append_uses_defaultdict_entry_insertion() {
    let rust_code = generate_rust_from_source_with_stdlib_collections(
        "from sifr.collections import defaultdict\n\ndef solve(text: str) -> int:\n    groups = defaultdict(list)\n    groups[1].append(text[0:2])\n    return len(groups[1])\n",
    );

    assert!(rust_code.contains(".entry(1_i64).or_insert(Vec::new()).push("));
    assert!(!rust_code.contains("groups.get_mut("));
}

#[test]
fn borrowed_string_defaultdict_set_operations_use_owned_storage_and_direct_lookup() {
    let rust_code = generate_rust_from_source_with_stdlib_collections(
        "from sifr.collections import defaultdict\n\ndef solve(text: str) -> int:\n    groups = defaultdict(set)\n    groups[1].add(text)\n    if text in groups[1]:\n        return 1\n    return 0\n",
    );

    assert!(rust_code.contains(".or_insert(HashSet::new()).insert(text.clone())"));
    assert!(rust_code.contains(".or_insert(HashSet::new()).contains(text)"));
    assert!(!rust_code.contains(".contains(&(text))"));
}
