use super::generate_rust_from_source_with_stdlib_collections;

#[test]
fn variable_string_key_defaultdict_counter_keeps_entry_default_codegen() {
    let rust_code = generate_rust_from_source_with_stdlib_collections(
        "from sifr.collections import defaultdict\n\ndef solve(tasks: list[str]) -> int:\n    counts = defaultdict(int)\n    for task in tasks:\n        counts[task] += 1\n    total = 0\n    for _, value in counts.items():\n        total += value\n    return total\n",
    );

    assert!(rust_code.contains("let mut counts: HashMap<String, i64> = HashMap::new();"));
    assert!(rust_code.contains("counts.entry(task.clone()).or_insert(0)"));
    assert!(rust_code.contains("*__elem += 1_i64;"));
}

#[test]
fn variable_integer_key_defaultdict_counter_keeps_entry_default_codegen() {
    let rust_code = generate_rust_from_source_with_stdlib_collections(
        "from sifr.collections import defaultdict\n\ndef solve(items: list[int]) -> int:\n    counts = defaultdict(int)\n    for item in items:\n        counts[item] += 1\n    return len(counts)\n",
    );

    assert!(rust_code.contains("let mut counts: HashMap<i64, i64> = HashMap::new();"));
    assert!(
        rust_code.contains("counts.entry(item.clone()).or_insert(0)"),
        "{rust_code}"
    );
    assert!(rust_code.contains("*__elem += 1_i64;"));
}
