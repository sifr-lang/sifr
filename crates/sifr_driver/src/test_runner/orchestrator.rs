use super::execution::execute_test_runner_project;
use crate::diagnostics::{run_codegen_with_boundary, write_stderr_line, RenderedDiagnostic};
use crate::project::{
    collect_project_hir_source_modules, discover_test_root_modules,
    parse_import_closure_source_modules, DiscoveryDiagnosticStyle, ModuleResolver,
    ParsedProjectModule,
};
use crate::stdlib::compile_stdlib;
use sifr_codegen::{generate_rust_multi_with_metadata, generate_rust_test};
use sifr_diagnostics::DiagnosticCode;
use sifr_frontend::{
    compile_module_hir_with_source, FrontendDiagnosticStyle, FrontendSourceContext,
};
use sifr_lowering::HirModule;
use sifr_stdlib_manifest::StdlibFeature;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

pub(crate) struct GeneratedTestRunnerProject {
    pub(crate) cache_scope: PathBuf,
    pub(crate) support_module_names: Vec<String>,
    pub(crate) support_rust_files: HashMap<String, String>,
    pub(crate) all_rust_code: String,
    pub(crate) all_stdlib_modules: HashSet<String>,
    pub(crate) all_required_features: HashSet<StdlibFeature>,
}

pub fn run_tests(test_dir: &Path) -> Result<bool, Vec<RenderedDiagnostic>> {
    let test_files_by_module = discover_test_root_modules(test_dir);

    if test_files_by_module.is_empty() {
        write_stderr_line(&format!("No test files found in {}", test_dir.display()));
        return Ok(true);
    }

    write_stderr_line(&format!(
        "Found {} test file(s)",
        test_files_by_module.len()
    ));

    let generated_project = build_test_runner_project(test_dir, &test_files_by_module)?;
    execute_test_runner_project(&generated_project).map(|outcome| outcome.success)
}

pub(crate) fn build_test_runner_project(
    test_dir: &Path,
    test_files_by_module: &BTreeMap<String, PathBuf>,
) -> Result<GeneratedTestRunnerProject, Vec<RenderedDiagnostic>> {
    let test_roots: BTreeSet<String> = test_files_by_module.keys().cloned().collect();
    let resolver = ModuleResolver::entry_parent(test_dir);
    let parsed_modules = parse_import_closure_source_modules(
        &resolver,
        &test_roots,
        DiscoveryDiagnosticStyle::FilePath,
    )?;
    let mut support_modules: HashMap<String, ParsedProjectModule> = HashMap::new();
    let mut test_modules: HashMap<String, ParsedProjectModule> = HashMap::new();
    for (module_name, parsed_module) in parsed_modules {
        if test_roots.contains(module_name.as_str()) {
            test_modules.insert(module_name, parsed_module);
        } else {
            support_modules.insert(module_name, parsed_module);
        }
    }

    let stdlib_compiled = compile_stdlib()?;
    let project_lowering =
        collect_project_hir_source_modules(&support_modules, stdlib_compiled.defs)?;
    let project_externals = project_lowering.external_defs.clone();
    let mut support_module_names: Vec<String> =
        project_lowering.hir_modules.keys().cloned().collect();
    support_module_names.sort();
    let support_module_refs: Vec<(&str, &HirModule)> = support_module_names
        .iter()
        .filter_map(|name| {
            project_lowering
                .hir_modules
                .get(name)
                .map(|module| (name.as_str(), module))
        })
        .collect();
    let support_codegen = run_codegen_with_boundary(
        "internal compiler panic during support-module code generation",
        || generate_rust_multi_with_metadata(&support_module_refs, &stdlib_compiled.code),
    )
    .map_err(|error| vec![*error])?;

    let mut all_rust_code = String::new();
    let mut all_stdlib_modules = support_codegen.used_stdlib_modules;
    let mut all_required_features = support_codegen.required_features;

    for (module_name, test_file) in test_files_by_module {
        let Some(parsed) = test_modules.get(module_name.as_str()) else {
            return Err(vec![crate::diagnostics::diagnostic_with_code(
                format!(
                    "missing parsed test module '{}' from '{}'",
                    module_name,
                    test_file.display()
                ),
                DiagnosticCode::INTERNAL_COMPILER_PANIC,
            )]);
        };

        let lowering_result = match compile_module_hir_with_source(
            module_name,
            &parsed.suite,
            &project_externals,
            FrontendDiagnosticStyle::Bare,
            Some(FrontendSourceContext {
                display_path: &parsed.display_path,
                source: &parsed.source,
            }),
        ) {
            Ok(result) => result,
            Err(errors) => {
                let diagnostics: Vec<RenderedDiagnostic> = errors
                    .into_iter()
                    .map(|mut error| {
                        error.message = format!("[{}] {}", test_file.display(), error.message);
                        error
                    })
                    .collect();
                return Err(diagnostics);
            }
        };

        let codegen_result = run_codegen_with_boundary(
            format!(
                "internal compiler panic during test-module code generation for '{}'",
                test_file.display()
            ),
            || generate_rust_test(&lowering_result.module),
        )
        .map_err(|error| vec![*error])?;
        all_rust_code.push_str("// Tests from: ");
        if let Some(file_name) = test_file.file_name() {
            all_rust_code.push_str(&file_name.to_string_lossy());
        } else {
            all_rust_code.push_str(&test_file.display().to_string());
        }
        all_rust_code.push('\n');
        all_rust_code.push_str(&codegen_result.rust_source);
        all_rust_code.push('\n');
        all_stdlib_modules.extend(codegen_result.used_stdlib_modules);
        all_required_features.extend(codegen_result.required_features);
    }

    Ok(GeneratedTestRunnerProject {
        cache_scope: test_dir.to_path_buf(),
        support_module_names,
        support_rust_files: support_codegen.rust_files,
        all_rust_code,
        all_stdlib_modules,
        all_required_features,
    })
}
