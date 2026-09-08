use super::materialize::canonical_rust_module_path;
use super::project_codegen::{GeneratedBinaryProject, format_generated_binary_project};
use super::rust_interop_resolution::resolve_package_rust_interop_metadata;
use super::sysroot_interop::attach_stdlib_rust_interop;
use crate::diagnostics::RenderedDiagnostic;
use crate::project::rust_module_file_path;
use crate::stdlib::StdlibRustInterop;
use crate::test_runner::GeneratedTestRunnerProject;
use std::collections::BTreeMap;
use std::path::PathBuf;

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
    let all_rust_code = if generated.bridge_modules.is_empty() {
        generated.main_rs
    } else {
        format!(
            "pub mod {};\n{}",
            sifr_codegen::canonicalize_generated_rust_identifier("__sifr_bridge"),
            generated.main_rs
        )
    };
    let mut bridge_rust_files = BTreeMap::new();
    for (module, source) in generated.bridge_modules {
        let path = if module == "__sifr_bridge" {
            PathBuf::from("__sifr_bridge/mod.rs")
        } else {
            rust_module_file_path(&module.replace("::", "."))
        };
        bridge_rust_files.insert(canonical_rust_module_path(&path)?, source);
    }
    Ok(GeneratedTestRunnerProject {
        cache_scope: project.cache_scope,
        support_module_names: project.support_module_names,
        support_rust_files: generated.support_modules.into_iter().collect(),
        bridge_rust_files,
        all_rust_code,
        all_stdlib_modules: generated.used_stdlib_modules,
        all_required_features: generated.required_features,
        interop: generated.interop,
    })
}
