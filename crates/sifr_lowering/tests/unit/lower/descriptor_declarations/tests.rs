use super::special_base;

#[test]
fn adapter_parent_collection_preserves_language_special_bases() {
    for name in ["Error", "Protocol", "int", "float", "str", "bool", "Enum"] {
        assert!(special_base(name));
    }
    assert!(!special_base("DataParent"));
}
