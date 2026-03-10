use crate::diagnostics::{run_codegen_with_boundary, CompileError};
use crate::project::{assemble_project_main_rs, ordered_non_main_module_names, ProjectLowering};
use sifr_codegen::{generate_rust_multi_with_metadata, StdlibCode};
use sifr_hir::HirModule;
use std::collections::{BTreeMap, HashSet};

pub(super) struct GeneratedBinaryProject {
    pub(super) main_rs: String,
    pub(super) support_modules: BTreeMap<String, String>,
    pub(super) used_stdlib_modules: HashSet<String>,
    pub(super) required_crates: HashSet<String>,
}

pub(super) fn generated_single_file_binary_project(
    codegen_result: sifr_codegen::CodegenResult,
) -> GeneratedBinaryProject {
    GeneratedBinaryProject {
        main_rs: codegen_result.rust_source,
        support_modules: BTreeMap::new(),
        used_stdlib_modules: codegen_result.used_stdlib_modules,
        required_crates: codegen_result.required_crates,
    }
}

pub(super) fn generated_project_binary_project(
    stdlib_code: &StdlibCode,
    project_lowering: ProjectLowering,
) -> Result<GeneratedBinaryProject, Vec<CompileError>> {
    let ProjectLowering {
        hir_modules,
        compile_order,
        ..
    } = project_lowering;
    let module_refs: Vec<(&str, &HirModule)> = compile_order
        .iter()
        .filter_map(|module_name| {
            hir_modules
                .get(module_name)
                .map(|module| (module_name.as_str(), module))
        })
        .collect();
    let codegen_result = run_codegen_with_boundary(
        "internal compiler panic during project code generation",
        || generate_rust_multi_with_metadata(&module_refs, stdlib_code),
    )
    .map_err(|error| vec![error])?;

    let main_rs = assemble_project_main_rs(&compile_order, &codegen_result.rust_files);
    let support_modules = ordered_non_main_module_names(&compile_order, &codegen_result.rust_files)
        .into_iter()
        .filter_map(|module_name| {
            codegen_result
                .rust_files
                .get(module_name.as_str())
                .map(|code| (module_name, code.clone()))
        })
        .collect();

    Ok(GeneratedBinaryProject {
        main_rs,
        support_modules,
        used_stdlib_modules: codegen_result.used_stdlib_modules,
        required_crates: codegen_result.required_crates,
    })
}
