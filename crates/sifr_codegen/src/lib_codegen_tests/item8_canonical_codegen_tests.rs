use super::*;

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
