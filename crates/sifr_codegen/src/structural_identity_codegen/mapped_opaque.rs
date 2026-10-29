use sifr_ir::HirClass;
use std::collections::HashMap;

const STRUCTURAL: &str = "::sifr_runtime::interop::structural";

pub(super) fn mapped_opaque_identity_expression(class: &HirClass) -> Option<String> {
    if !crate::structural_impl_codegen::structural_mapped_opaque_supported(class) {
        return None;
    }
    let native = sifr_ir::rust_opaque_type_path(&class.rust_interop)?;
    let mapping = sifr_ir::rust_opaque_structural_mapping(&class.rust_interop)?;
    Some(format!(
        "<{} as {STRUCTURAL}::StructuralMapping<{}>>::shape_identity()",
        absolute_rust_path(&mapping.dotted()),
        absolute_rust_path(&native.dotted()),
    ))
}

pub(super) fn mapped_opaque_identity_expression_for_imported_type(
    class: &HirClass,
    module_name: &str,
    nominal_type_paths: &HashMap<String, String>,
) -> Option<String> {
    if !crate::structural_impl_codegen::structural_mapped_opaque_supported(class) {
        return None;
    }
    let canonical = class
        .identity
        .clone()
        .unwrap_or_else(|| format!("{module_name}.{}", class.name));
    let generated_path = nominal_type_paths.get(&canonical)?;
    Some(format!(
        "<{generated_path} as {STRUCTURAL}::StructuralType>::shape_identity()"
    ))
}

fn absolute_rust_path(path: &str) -> String {
    let path = path.replace('.', "::");
    if path.starts_with("::") {
        path
    } else {
        format!("::{path}")
    }
}

#[cfg(test)]
#[path = "../../tests/unit/structural_identity_codegen/mapped_opaque/tests.rs"]
mod tests;
