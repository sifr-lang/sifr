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

#[test]
fn nested_shadowed_defaultdict_counters_keep_independent_key_types() {
    let rust_code = generate_rust_from_source_with_stdlib_collections(
        "from sifr.collections import defaultdict\n\ndef solve(words: list[str], nums: list[int]) -> int:\n    counts = defaultdict(int)\n    def helper() -> int:\n        counts = defaultdict(int)\n        for n in nums:\n            counts[n] += 1\n        return len(counts)\n    for word in words:\n        counts[word] += 1\n    return len(counts) + helper()\n",
    );

    assert!(rust_code.contains("let mut counts: HashMap<String, i64> = HashMap::new();"));
    assert!(rust_code.contains("let mut counts: HashMap<i64, i64> = HashMap::new();"));
}

#[test]
fn nested_scalar_shadow_is_not_retyped_as_defaultdict() {
    let rust_code = generate_rust_from_source_with_stdlib_collections(
        "from sifr.collections import defaultdict\n\ndef solve(words: list[str]) -> int:\n    counts = defaultdict(int)\n    def helper() -> int:\n        counts = 7\n        return counts\n    for word in words:\n        counts[word] += 1\n    return len(counts) + helper()\n",
    );

    assert!(rust_code.contains("let mut counts: HashMap<String, i64> = HashMap::new();"));
    assert!(rust_code.contains("let counts: i64 = 7_i64;"));
    assert!(!rust_code.contains("HashMap<String, i64> = 7_i64"));
}

#[test]
fn late_nested_shadow_does_not_clear_enclosing_defaultdict_annotation() {
    let rust_code = generate_rust_from_source_with_stdlib_collections(
        "from sifr.collections import defaultdict\n\ndef solve(words: list[str], nums: list[int]) -> int:\n    counts = defaultdict(int)\n    if len(words) > 0:\n        for word in words:\n            counts[word] += 1\n        def helper() -> int:\n            counts = defaultdict(int)\n            for value in nums:\n                counts[value] += 1\n            return len(counts)\n        return len(counts) + helper()\n    return 0\n",
    );

    assert!(rust_code.contains("let mut counts: HashMap<String, i64> = HashMap::new();"));
    assert!(rust_code.contains("let mut counts: HashMap<i64, i64> = HashMap::new();"));
}
