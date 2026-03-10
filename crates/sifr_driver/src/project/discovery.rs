use crate::diagnostics::{CompileError, CompilePhase};
use sifr_python_ast::Stmt;
use sifr_python_parser::parse_module;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DiscoveryDiagnosticStyle {
    ModuleName,
    FilePath,
}

fn discover_project_sifr_files(project_dir: &Path) -> Vec<PathBuf> {
    let mut sifr_files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(project_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "sifr") {
                sifr_files.push(path);
            }
        }
    }
    sifr_files.sort();
    sifr_files
}

fn is_test_module_name(module_name: &str) -> bool {
    module_name.starts_with("test_") || module_name.ends_with("_test")
}

pub(crate) fn discover_test_root_modules(test_dir: &Path) -> BTreeMap<String, PathBuf> {
    let mut test_files_by_module = BTreeMap::new();
    for path in discover_project_sifr_files(test_dir) {
        let module_name = path.file_stem().unwrap().to_string_lossy().to_string();
        if is_test_module_name(&module_name) {
            test_files_by_module.insert(module_name, path);
        }
    }
    test_files_by_module
}

fn module_source_path(project_dir: &Path, module_name: &str) -> PathBuf {
    project_dir.join(format!("{module_name}.sifr"))
}

fn discovery_label(
    module_name: &str,
    path: &Path,
    diagnostic_style: DiscoveryDiagnosticStyle,
) -> String {
    match diagnostic_style {
        DiscoveryDiagnosticStyle::ModuleName => module_name.to_string(),
        DiscoveryDiagnosticStyle::FilePath => path.display().to_string(),
    }
}

fn collect_import_closure_module_dependencies(stmts: &[Stmt]) -> BTreeSet<String> {
    let mut deps = BTreeSet::new();
    for stmt in stmts {
        let Stmt::ImportFrom(import_from) = stmt else {
            continue;
        };
        if import_from.level > 1 {
            continue;
        }
        let Some(module) = &import_from.module else {
            continue;
        };
        let module_name = module.to_string();
        if module_name == "typing"
            || module_name == "enum"
            || module_name.starts_with("sifr.")
            || module_name.starts_with("_sifr.")
        {
            continue;
        }
        deps.insert(module_name);
    }
    deps
}

pub(crate) fn parse_import_closure_modules(
    project_dir: &Path,
    root_modules: &BTreeSet<String>,
    diagnostic_style: DiscoveryDiagnosticStyle,
) -> Result<HashMap<String, Vec<Stmt>>, Vec<CompileError>> {
    let mut parsed_modules: HashMap<String, Vec<Stmt>> = HashMap::new();
    let mut parsed_names: BTreeSet<String> = BTreeSet::new();
    let mut pending = root_modules.clone();

    while let Some(module_name) = pending.pop_first() {
        if !parsed_names.insert(module_name.clone()) {
            continue;
        }

        let path = module_source_path(project_dir, &module_name);
        let source = std::fs::read_to_string(&path).map_err(|e| {
            vec![CompileError {
                message: format!("failed to read '{}': {}", path.display(), e),
                phase: CompilePhase::Build,
            }]
        })?;
        let label = discovery_label(&module_name, &path, diagnostic_style);
        let parsed = match parse_module(&source) {
            Ok(parsed) => {
                if !parsed.is_valid() {
                    let errors: Vec<CompileError> = parsed
                        .errors()
                        .iter()
                        .map(|e| CompileError {
                            message: format!("[{label}] {e}"),
                            phase: CompilePhase::Parse,
                        })
                        .collect();
                    return Err(errors);
                }
                parsed
            }
            Err(e) => {
                return Err(vec![CompileError {
                    message: format!("[{label}] failed to parse: {e}"),
                    phase: CompilePhase::Parse,
                }]);
            }
        };
        let suite = parsed.into_suite();
        for dependency in collect_import_closure_module_dependencies(&suite) {
            if parsed_names.contains(dependency.as_str()) {
                continue;
            }
            if module_source_path(project_dir, &dependency).is_file() {
                pending.insert(dependency);
            }
        }
        parsed_modules.insert(module_name, suite);
    }

    Ok(parsed_modules)
}
