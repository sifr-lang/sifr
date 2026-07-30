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
