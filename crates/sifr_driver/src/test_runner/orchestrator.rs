use super::execution::execute_test_runner_project;
use crate::diagnostics::{
    RenderedDiagnostic, render_codegen_error, run_codegen_with_boundary, write_stderr_line,
};
use crate::project::{
    DiscoveryDiagnosticStyle, ModuleResolver, ParsedProjectModule,
    collect_project_hir_source_modules, compile_project_source_modules, discover_test_root_modules,
    parse_import_closure_source_modules,
};
use crate::stdlib::compile_stdlib;
use sifr_codegen::generate_rust_test_project_with_metadata;
use sifr_diagnostics::DiagnosticCode;
use sifr_frontend::{FrontendDiagnosticStyle, SourceProvider};
use sifr_lowering::{HirModule, LoweringOptions};
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
    pub(crate) interop: sifr_codegen::InteropBuildPlan,
}

pub fn run_tests(
    test_dir: &Path,
    provider: &mut dyn SourceProvider,
    lock_mode: sifr_package::CargoLockMode,
) -> Result<bool, Vec<RenderedDiagnostic>> {
    let test_files_by_module = discover_test_root_modules(test_dir, provider);

    if test_files_by_module.is_empty() {
        write_stderr_line(&format!("No test files found in {}", test_dir.display()));
        return Ok(true);
    }

    write_stderr_line(&format!(
        "Found {} test file(s)",
        test_files_by_module.len()
    ));

    let generated_project = build_test_runner_project(test_dir, &test_files_by_module, provider)?;
    execute_test_runner_project(&generated_project, lock_mode).map(|outcome| outcome.success)
}

pub(crate) fn build_test_runner_project(
    test_dir: &Path,
    test_files_by_module: &BTreeMap<String, PathBuf>,
    provider: &mut dyn SourceProvider,
) -> Result<GeneratedTestRunnerProject, Vec<RenderedDiagnostic>> {
    let test_roots: BTreeSet<String> = test_files_by_module.keys().cloned().collect();
    let resolver = ModuleResolver::entry_parent(test_dir);
    let parsed_modules = parse_import_closure_source_modules(
        &resolver,
        &test_roots,
        DiscoveryDiagnosticStyle::FilePath,
        provider,
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
    let test_modules = test_modules.into_iter().collect::<BTreeMap<_, _>>();
    let test_compilation = compile_project_source_modules(
        &test_modules,
        project_externals,
        FrontendDiagnosticStyle::Bare,
        &LoweringOptions::default(),
    )
    .map_err(|errors| {
        errors
            .into_iter()
            .map(|mut error| {
                let display_path = error
                    .spans
                    .iter()
                    .find(|span| span.is_primary)
                    .and_then(|span| span.file.clone())
                    .unwrap_or_else(|| "test project".to_string());
                error.message = format!("[{display_path}] {}", error.message);
                error
            })
            .collect::<Vec<_>>()
    })?;
    let lowered_test_modules = test_compilation.hir_modules;
    let test_module_refs = lowered_test_modules
        .iter()
        .map(|(name, module)| (name.as_str(), module))
        .collect::<Vec<_>>();
    let generated = run_codegen_with_boundary(
        "internal compiler panic during test-project code generation",
        || {
            generate_rust_test_project_with_metadata(
                &support_module_refs,
                &test_module_refs,
                &stdlib_compiled.code,
            )
        },
    )
    .map_err(|error| vec![*error])?
    .map_err(|error| vec![render_codegen_error(&error)])?;

    let mut all_rust_code = generated.project_union_prelude;
    if !all_rust_code.is_empty() {
        all_rust_code.push('\n');
    }
    for (module_name, test_file) in test_files_by_module {
        let Some(rust_source) = generated.test_rust_files.get(module_name) else {
            return Err(vec![crate::diagnostics::diagnostic_with_code(
                format!("missing generated test module '{module_name}'"),
                DiagnosticCode::INTERNAL_COMPILER_PANIC,
            )]);
        };
        all_rust_code.push_str("// Tests from: ");
        if let Some(file_name) = test_file.file_name() {
            all_rust_code.push_str(&file_name.to_string_lossy());
        } else {
            all_rust_code.push_str(&test_file.display().to_string());
        }
        all_rust_code.push('\n');
        all_rust_code.push_str(rust_source);
        all_rust_code.push('\n');
    }

    Ok(GeneratedTestRunnerProject {
        cache_scope: test_dir.to_path_buf(),
        support_module_names,
        support_rust_files: generated.support_rust_files,
        all_rust_code,
        all_stdlib_modules: generated.used_stdlib_modules,
        all_required_features: generated.required_features,
        interop: generated.interop,
    })
}
