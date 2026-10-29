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
#[path = "../../../tests/unit/intrinsics/registry/collections/tests.rs"]
mod tests;
