use super::{
    CodegenResult, HirModule, StdlibCode, generate_rust_with_stdlib_for_module_with_project_policy,
};

pub(crate) fn generate_rust_with_stdlib_for_module_with_structural_policy(
    module: &HirModule,
    stdlib_code: &StdlibCode,
    module_name: Option<&str>,
    structural_interop_enabled: bool,
) -> CodegenResult {
    generate_rust_with_stdlib_for_module_with_project_policy(
        module,
        stdlib_code,
        module_name,
        None,
        structural_interop_enabled,
        None,
        None,
        None,
        None,
        None,
    )
}
