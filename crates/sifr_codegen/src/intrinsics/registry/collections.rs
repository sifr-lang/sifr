//! Shared classification for language-owned typed defaultdict storage aliases.

pub(crate) fn is_defaultdict_storage_alias(name: &str) -> bool {
    matches!(
        name,
        "__sifr_defaultdict_int" | "__sifr_defaultdict_list" | "__sifr_defaultdict_set"
    )
}

pub(crate) fn is_collection_defaultdict_storage_alias(name: &str) -> bool {
    matches!(name, "__sifr_defaultdict_list" | "__sifr_defaultdict_set")
}

#[cfg(test)]
mod tests {
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
}
