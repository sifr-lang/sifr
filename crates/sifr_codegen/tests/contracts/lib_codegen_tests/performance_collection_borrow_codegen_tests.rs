use super::*;

#[test]
fn list_indexed_dict_lookup_borrows_row_and_string_key() {
    let generated = generate_rust_from_source(
        r#"
def child_at(rows: list[dict[str, int]], row: int, key: str) -> int:
    if key in rows[row]:
        value: int | None = rows[row][key]
        if value is not None:
            return value
    return -1
"#,
    );

    assert!(generated.contains("rows.get(__idx_norm)"), "{generated}");
    assert!(generated.contains("__bucket.contains_key(key)"));
    assert!(generated.contains("__bucket.get(key)"));
    assert!(generated.contains(")).cloned()"));
    assert!(!generated.contains("__bucket.contains_key(&key)"));
    assert!(!generated.contains("__bucket.get(&key)"));
    assert!(!generated.contains("__sifr_index_value).cloned()"));
}
