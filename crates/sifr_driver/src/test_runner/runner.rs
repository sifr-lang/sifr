use crate::build::{
    create_invocation_workspace, generate_dependency_cargo_toml, InvocationWorkspaceGuard,
};
use crate::diagnostics::{
    run_codegen_with_boundary, write_stderr, write_stderr_line, CompileError, CompilePhase,
};
use crate::frontend::{lower_frontend_module, FrontendDiagnosticStyle};
use crate::project::{
    collect_project_hir_modules, discover_test_root_modules, parse_import_closure_modules,
    DiscoveryDiagnosticStyle,
};
use crate::stdlib::compile_stdlib;
use sifr_codegen::{generate_rust_multi_with_metadata, generate_rust_test};
use sifr_hir::HirModule;
use sifr_python_ast::Stmt;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::Path;

pub fn run_tests(test_dir: &Path) -> Result<bool, Vec<CompileError>> {
    let test_files_by_module = discover_test_root_modules(test_dir);

    if test_files_by_module.is_empty() {
        write_stderr_line(&format!("No test files found in {}", test_dir.display()));
        return Ok(true);
    }

    write_stderr_line(&format!(
        "Found {} test file(s)",
        test_files_by_module.len()
    ));

    let test_roots: BTreeSet<String> = test_files_by_module.keys().cloned().collect();
    let parsed_modules =
        parse_import_closure_modules(test_dir, &test_roots, DiscoveryDiagnosticStyle::FilePath)?;
    let mut support_modules: HashMap<String, Vec<Stmt>> = HashMap::new();
    let mut test_modules: HashMap<String, Vec<Stmt>> = HashMap::new();
    for (module_name, suite) in parsed_modules {
        if test_roots.contains(module_name.as_str()) {
            test_modules.insert(module_name, suite);
        } else {
            support_modules.insert(module_name, suite);
        }
    }

    let stdlib_compiled = compile_stdlib()?;
    let project_lowering = collect_project_hir_modules(&support_modules, stdlib_compiled.defs)?;
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
    .map_err(|e| vec![e])?;

    let mut all_rust_code = String::new();
    let mut all_stdlib_modules = support_codegen.used_stdlib_modules;
    let mut all_required_crates = support_codegen.required_crates;

    for (module_name, test_file) in &test_files_by_module {
        let Some(parsed) = test_modules.get(module_name.as_str()) else {
            return Err(vec![CompileError {
                message: format!(
                    "missing parsed test module '{}' from '{}'",
                    module_name,
                    test_file.display()
                ),
                phase: CompilePhase::Build,
            }]);
        };

        let lowering_result = match lower_frontend_module(
            module_name,
            parsed,
            &project_externals,
            FrontendDiagnosticStyle::Bare,
        ) {
            Ok(result) => result,
            Err(errors) => {
                let compile_errors: Vec<CompileError> = errors
                    .into_iter()
                    .map(|e| CompileError {
                        message: format!("[{}] {}", test_file.display(), e.message),
                        phase: CompilePhase::TypeCheck,
                    })
                    .collect();
                return Err(compile_errors);
            }
        };

        let codegen_result = run_codegen_with_boundary(
            format!(
                "internal compiler panic during test-module code generation for '{}'",
                test_file.display()
            ),
            || generate_rust_test(&lowering_result.module),
        )
        .map_err(|e| vec![e])?;
        all_rust_code.push_str("// Tests from: ");
        all_rust_code.push_str(&test_file.file_name().unwrap().to_string_lossy());
        all_rust_code.push('\n');
        all_rust_code.push_str(&codegen_result.rust_source);
        all_rust_code.push('\n');
        all_stdlib_modules.extend(codegen_result.used_stdlib_modules);
        all_required_crates.extend(codegen_result.required_crates);
    }

    let project_dir = create_invocation_workspace("test_runner")?;
    let _workspace_guard = InvocationWorkspaceGuard::new(project_dir.clone());
    let src_dir = project_dir.join("src");
    std::fs::create_dir_all(&src_dir).map_err(|e| {
        vec![CompileError {
            message: format!("failed to create test directory: {e}"),
            phase: CompilePhase::Build,
        }]
    })?;

    let cargo_toml = generate_test_runner_cargo_toml(&all_stdlib_modules, &all_required_crates);
    std::fs::write(project_dir.join("Cargo.toml"), cargo_toml).map_err(|e| {
        vec![CompileError {
            message: format!("failed to write Cargo.toml: {e}"),
            phase: CompilePhase::Build,
        }]
    })?;

    for module_name in &support_module_names {
        if let Some(code) = support_codegen.rust_files.get(module_name) {
            std::fs::write(src_dir.join(format!("{module_name}.rs")), code).map_err(|e| {
                vec![CompileError {
                    message: format!("failed to write {module_name}.rs: {e}"),
                    phase: CompilePhase::Build,
                }]
            })?;
        }
    }

    let test_lib = compose_test_runner_lib(&support_module_names, &all_rust_code);
    std::fs::write(src_dir.join("lib.rs"), &test_lib).map_err(|e| {
        vec![CompileError {
            message: format!("failed to write lib.rs: {e}"),
            phase: CompilePhase::Build,
        }]
    })?;

    let output = std::process::Command::new("cargo")
        .args(["test"])
        .current_dir(&project_dir)
        .output()
        .map_err(|e| {
            vec![CompileError {
                message: format!("failed to run cargo test: {e}"),
                phase: CompilePhase::Build,
            }]
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stdout.is_empty() {
        write_stderr(&stdout);
    }
    if !stderr.is_empty() {
        write_stderr(&stderr);
    }

    Ok(output.status.success())
}

pub(crate) fn compose_test_runner_lib(
    support_module_names: &[String],
    all_rust_code: &str,
) -> String {
    let mut test_lib = String::from("#![cfg(test)]\n\n");
    for module_name in support_module_names {
        test_lib.push_str("mod ");
        test_lib.push_str(module_name);
        test_lib.push_str(";\n");
    }
    if !support_module_names.is_empty() {
        test_lib.push('\n');
    }
    test_lib.push_str(all_rust_code);
    test_lib
}

pub(crate) fn generate_test_runner_cargo_toml(
    stdlib_modules: &HashSet<String>,
    required_crates: &HashSet<String>,
) -> String {
    generate_dependency_cargo_toml("sifr_tests", stdlib_modules, required_crates)
}
