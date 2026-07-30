use super::generate_rust_from_source;

#[test]
fn same_named_empty_dict_bindings_keep_distinct_rust_types() {
    let generated = generate_rust_from_source(
        "def solve(flag: bool) -> int:\n    if flag:\n        data = {}\n        data[\"key\"] = 1\n        return len(data)\n    data = {}\n    data[2] = 3\n    return len(data)\n",
    );
    assert!(generated.contains("let mut data: HashMap<String, i64>"));
    assert!(generated.contains("let mut data: HashMap<i64, i64>"));
}

#[test]
fn compatible_same_named_empty_dict_bindings_do_not_share_value_type() {
    let generated = generate_rust_from_source(
        "def solve(flag: bool) -> int:\n    if flag:\n        data = {}\n        data[1] = 2.5\n        return len(data)\n    data = {}\n    data[3] = 4\n    return len(data)\n",
    );
    assert!(generated.contains("let mut data: HashMap<i64, f64>"));
    assert!(generated.contains("let mut data: HashMap<i64, i64>"));
}
