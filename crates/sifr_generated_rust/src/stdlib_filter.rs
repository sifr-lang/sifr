mod implementation;
pub use implementation::*;
mod external_refs;
mod inherent_trait_demand;
pub(crate) use inherent_trait_demand::{InherentMethods, collect_inherent_methods};
mod item_refs;
mod support_import_refs;
pub use external_refs::{rust_source_referenced_item_names, rust_source_required_trait_names};
pub(crate) use support_import_refs::rust_source_unqualified_item_names;
mod dedup_keys;
mod relocation;
pub use relocation::*;

#[cfg(test)]
#[path = "../tests/unit/stdlib_filter/tests.rs"]
mod tests;
pub(crate) use external_refs::rust_source_required_trait_names_with_inherent;
