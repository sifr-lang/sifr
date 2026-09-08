use super::project_codegen::{GeneratedBinaryProject, format_generated_binary_project};
use super::rust_interop_resolution::resolve_package_rust_interop_metadata;
use super::sysroot_interop::attach_stdlib_rust_interop;
use crate::diagnostics::RenderedDiagnostic;
use crate::stdlib::StdlibRustInterop;
use crate::test_runner::GeneratedTestRunnerProject;
use std::collections::BTreeMap;

/// Resolve test-project demand through the same trusted inventory and bridge
/// finalization as a binary, before materializing its Cargo test workspace.
pub(crate) fn finalize_test_runner_project(
    project: GeneratedTestRunnerProject,
    stdlib: &StdlibRustInterop,
) -> Result<GeneratedTestRunnerProject, Vec<RenderedDiagnostic>> {
    let generated = GeneratedBinaryProject {
        main_rs: project.all_rust_code,
        support_modules: project.support_rust_files.into_iter().collect(),
        bridge_modules: BTreeMap::new(),
        used_stdlib_modules: project.all_stdlib_modules,
        required_features: project.all_required_features,
        interop: project.interop,
        cache_key_fragment: None,
        python_runtime: None,
    };
    let (generated, context) = attach_stdlib_rust_interop(generated, None, stdlib);
    let generated = resolve_package_rust_interop_metadata(generated, context)?;
    let generated = format_generated_binary_project(generated)?;
    let mut support_module_names = project.support_module_names;
    support_module_names.extend(generated.bridge_modules.keys().cloned());
    support_module_names.sort();
    support_module_names.dedup();
    Ok(GeneratedTestRunnerProject {
        cache_scope: project.cache_scope,
        support_module_names,
        support_rust_files: generated
            .support_modules
            .into_iter()
            .chain(generated.bridge_modules)
            .collect(),
        all_rust_code: generated.main_rs,
        all_stdlib_modules: generated.used_stdlib_modules,
        all_required_features: generated.required_features,
        interop: generated.interop,
    })
}
