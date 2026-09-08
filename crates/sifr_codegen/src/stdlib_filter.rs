mod implementation;
pub(crate) use implementation::*;
mod external_refs;
mod item_refs;
mod support_import_refs;
pub(crate) use external_refs::{
    rust_source_referenced_item_names, rust_source_required_trait_names,
};
pub(crate) use support_import_refs::rust_source_unqualified_item_names;
mod dedup_keys;
mod relocation;
pub(crate) use relocation::*;
#[cfg(test)]
mod tests;
