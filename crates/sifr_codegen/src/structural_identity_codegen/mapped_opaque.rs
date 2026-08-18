use sifr_ir::HirClass;

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

fn absolute_rust_path(path: &str) -> String {
    let path = path.replace('.', "::");
    if path.starts_with("::") {
        path
    } else {
        format!("::{path}")
    }
}
