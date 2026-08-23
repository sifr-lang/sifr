use sifr_lowering::{
    ExternalDefs, HirClass, RustInteropDecoratorKind, rust_opaque_structural_mapping,
};
use std::collections::HashSet;

pub(super) fn record_local(
    class: &HirClass,
    opaque: &mut HashSet<String>,
    structural: &mut HashSet<String>,
) {
    if !class
        .rust_interop
        .iter()
        .any(|declaration| declaration.kind == RustInteropDecoratorKind::Opaque)
    {
        return;
    }
    opaque.insert(class.name.clone());
    if rust_opaque_structural_mapping(&class.rust_interop).is_some() {
        structural.insert(class.name.clone());
    }
}

pub(super) fn record_imported(
    external_defs: &ExternalDefs,
    module: &str,
    source_name: &str,
    local_name: &str,
    opaque: &mut HashSet<String>,
    structural: &mut HashSet<String>,
) {
    if external_defs
        .rust_opaque_classes
        .get(module)
        .is_some_and(|classes| classes.contains(source_name))
    {
        opaque.insert(local_name.to_string());
    }
    if external_defs
        .rust_structural_classes
        .get(module)
        .is_some_and(|classes| classes.contains(source_name))
    {
        structural.insert(local_name.to_string());
    }
}

pub(super) fn replace_module(
    external_defs: &mut ExternalDefs,
    module: &str,
    opaque: HashSet<String>,
    structural: HashSet<String>,
) {
    if opaque.is_empty() {
        external_defs.rust_opaque_classes.remove(module);
    } else {
        external_defs
            .rust_opaque_classes
            .insert(module.to_string(), opaque);
    }
    if structural.is_empty() {
        external_defs.rust_structural_classes.remove(module);
    } else {
        external_defs
            .rust_structural_classes
            .insert(module.to_string(), structural);
    }
}
