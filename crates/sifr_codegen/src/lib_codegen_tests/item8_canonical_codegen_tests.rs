use super::*;

#[test]
fn dict_keys_membership_guards_equivalent_indexed_reads() {
    let generated = generate_rust_from_source(
        r#"
def lookup(table: dict[int, int], base: int) -> int:
    if base + 1 in table.keys():
        return table[base + 1]
    return -1
"#,
    );

    assert!(
        !generated.contains("structured statement emission missing"),
        "{generated}"
    );
    assert!(generated.contains("if let Some("), "{generated}");
    assert!(generated.contains("table.get(&(&base +"), "{generated}");
    assert!(
        !generated.contains("table.keys().cloned().collect"),
        "{generated}"
    );
}

#[test]
fn fstrings_use_debug_format_for_result_values() {
    let generated = generate_rust_from_source(
        r#"
def render() -> str:
    text: str = f"round={round(3.7)}"
    print(f"round={round(3.7)}")
    return text
"#,
    );

    assert!(generated.contains("format!(\"round={:?}\""), "{generated}");
    assert!(generated.contains("println!(\"round={:?}\""), "{generated}");
    assert!(!generated.contains("round={}\""), "{generated}");
}

#[test]
fn class_len_method_returning_sifr_int_is_not_wrapped_again() {
    let generated = generate_rust_from_source(
        r#"
class Store:
    items: list[int]

    def __init__(self):
        self.items = [1, 2, 3]

    def len(self) -> int:
        return len(self.items)

def size(store: Store) -> int:
    return store.len()
"#,
    );

    assert!(generated.contains("store.len()"), "{generated}");
    assert!(
        !generated.contains("SifrInt::from(store.len())"),
        "{generated}"
    );
}
