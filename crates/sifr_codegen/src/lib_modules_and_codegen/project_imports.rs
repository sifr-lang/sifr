use crate::RustItem;
use std::collections::{HashMap, HashSet};

pub(super) fn structural_layout_import_items(
    record_types: &HashMap<String, sifr_type_system::StructuralRecordType>,
) -> Vec<RustItem> {
    let mut layout_names = record_types
        .values()
        .map(crate::structural_identity_codegen::structural_record_layout_rust_name)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    layout_names.sort();
    layout_names
        .into_iter()
        .map(|layout_name| RustItem::Use(vec!["crate".to_string(), layout_name]))
        .collect()
}
