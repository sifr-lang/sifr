use super::*;

#[test]
fn typed_defaultdict_alias_classification_excludes_legacy_serialized_helpers() {
    assert!(is_defaultdict_storage_alias("__sifr_defaultdict_int"));
    assert!(is_collection_defaultdict_storage_alias(
        "__sifr_defaultdict_set"
    ));
    assert!(!is_collection_defaultdict_storage_alias(
        "__sifr_defaultdict_int"
    ));
    assert!(!is_defaultdict_storage_alias("defaultdict"));
}
