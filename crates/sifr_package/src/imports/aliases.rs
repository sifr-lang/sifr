use crate::manifest::metadata::CargoSifrAliasMetadata;
use std::collections::BTreeMap;

#[must_use]
pub fn alias_imports(
    aliases: &BTreeMap<String, CargoSifrAliasMetadata>,
) -> BTreeMap<String, String> {
    aliases
        .iter()
        .map(|(alias, metadata)| (alias.clone(), metadata.import.clone()))
        .collect()
}
