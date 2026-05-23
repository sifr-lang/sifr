use super::materialize::{
    cached_binary_path, materialize_binary_project, materialize_cached_binary_project,
};
use super::project_codegen::{
    generated_project_binary_project, generated_single_file_binary_project, GeneratedBinaryProject,
};
use super::ArtifactCacheReport;
use crate::diagnostics::{run_codegen_with_boundary, CompileResult, RenderedDiagnostic};
use crate::frontend::{parse_source, FrontendCompiled};
use crate::project::{
    collect_project_hir_source_modules, compile_single_frontend_module_with_source,
    emit_project_frontend_diagnostics, parse_import_closure_source_modules,
    parse_package_import_closure_source_modules, DiscoveryDiagnosticStyle, ModuleResolver,
    ProjectLowering,
};
use crate::stdlib::{compile_stdlib, StdlibCompiled};
use crate::workspace::find_workspace_root;
use sifr_codegen::generate_rust_with_stdlib;
use sifr_diagnostics::DiagnosticCode;
use sifr_frontend::{FrontendDiagnosticStyle, FrontendSourceContext};
use sifr_hir::LoweringResult;
use sifr_package::{PackageSourceMap, SifrPackageGraph, SifrPackageId};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RootedEntrypointShape {
    SingleFile,
    Project,
}

pub(crate) enum RootedEntrypoint<'a> {
    SingleFile {
        source: &'a str,
        display_path: &'a str,
    },
    Project {
        main_file: &'a Path,
    },
    PackageProject {
        entrypoint: &'a PackageEntrypoint,
    },
}

#[derive(Clone, Debug)]
pub struct PackageEntrypoint {
    pub main_file: PathBuf,
    pub package_id: SifrPackageId,
    pub graph: SifrPackageGraph,
    pub source_map: PackageSourceMap,
}

pub struct CachedBinaryArtifact {
    binary_path: PathBuf,
    cache_report: ArtifactCacheReport,
}

impl CachedBinaryArtifact {
    pub fn binary_path(&self) -> &Path {
        &self.binary_path
    }

    pub fn cache_status_line(&self) -> String {
        self.cache_report.status_line()
    }

    #[cfg(test)]
    pub(crate) fn cache_report(&self) -> &ArtifactCacheReport {
        &self.cache_report
    }
}

pub(crate) struct RootedEntrypointPlan {
    shape: RootedEntrypointShape,
    stdlib: StdlibCompiled,
    project_lowering: ProjectLowering,
}

pub(crate) fn compile_single_file_frontend(
    source: &str,
) -> Result<FrontendCompiled, Vec<RenderedDiagnostic>> {
    RootedEntrypointPlan::from_entrypoint(&RootedEntrypoint::SingleFile {
        source,
        display_path: "main",
    })?
    .into_single_file_frontend()
}

pub(crate) fn compile_single_file_entrypoint_with_metadata(
    source: &str,
) -> Result<sifr_codegen::CodegenResult, Vec<RenderedDiagnostic>> {
    let plan = RootedEntrypointPlan::from_entrypoint(&RootedEntrypoint::SingleFile {
        source,
        display_path: "main",
    })?;
    plan.emit_frontend_diagnostics();
    plan.into_single_file_codegen_result()
}

pub(crate) fn check_single_file_entrypoint(
    source: &str,
    entrypoint_file: &Path,
) -> Vec<RenderedDiagnostic> {
    let display_path = entrypoint_file.to_string_lossy();
    match RootedEntrypointPlan::from_entrypoint(&RootedEntrypoint::SingleFile {
        source,
        display_path: &display_path,
    }) {
        Ok(plan) => plan.frontend_diagnostics(),
        Err(errors) => errors,
    }
}

pub(crate) fn resolve_project_entrypoint_plan(
    main_file: &Path,
) -> Result<RootedEntrypointPlan, Vec<RenderedDiagnostic>> {
    RootedEntrypointPlan::from_entrypoint(&RootedEntrypoint::Project { main_file })
}

pub(crate) fn resolve_package_project_entrypoint_plan(
    entrypoint: &PackageEntrypoint,
) -> Result<RootedEntrypointPlan, Vec<RenderedDiagnostic>> {
    RootedEntrypointPlan::from_entrypoint(&RootedEntrypoint::PackageProject { entrypoint })
}

pub(crate) fn emit_project_entrypoint(main_file: &Path) -> CompileResult {
    let plan = match resolve_project_entrypoint_plan(main_file) {
        Ok(plan) => plan,
        Err(errors) => return CompileResult::Errors { errors },
    };
    plan.emit_frontend_diagnostics();
    match plan.into_generated_binary_project() {
        Ok(generated_project) => CompileResult::Success {
            rust_source: generated_project.emit_source_listing(),
        },
        Err(errors) => CompileResult::Errors { errors },
    }
}

pub(crate) fn build_rooted_entrypoint_binary(
    entrypoint: &RootedEntrypoint<'_>,
    output_dir: &Path,
) -> Result<PathBuf, Vec<RenderedDiagnostic>> {
    let plan = RootedEntrypointPlan::from_entrypoint(entrypoint)?;
    plan.emit_frontend_diagnostics();
    let generated_project = plan.into_generated_binary_project()?;
    materialize_binary_project(output_dir, "sifr_output", generated_project)
}

pub(crate) fn build_cached_project_binary(
    main_file: &Path,
) -> Result<CachedBinaryArtifact, Vec<RenderedDiagnostic>> {
    build_cached_rooted_entrypoint_binary(
        &RootedEntrypoint::Project { main_file },
        main_file.parent().unwrap_or(Path::new(".")),
        "run",
    )
}

pub(crate) fn build_cached_package_project_binary(
    entrypoint: &PackageEntrypoint,
) -> Result<CachedBinaryArtifact, Vec<RenderedDiagnostic>> {
    build_cached_rooted_entrypoint_binary(
        &RootedEntrypoint::PackageProject { entrypoint },
        entrypoint.main_file.parent().unwrap_or(Path::new(".")),
        "run",
    )
}

pub(crate) fn build_cached_single_file_binary(
    source: &str,
    entrypoint_file: &Path,
) -> Result<CachedBinaryArtifact, Vec<RenderedDiagnostic>> {
    let display_path = entrypoint_file.to_string_lossy();
    build_cached_rooted_entrypoint_binary(
        &RootedEntrypoint::SingleFile {
            source,
            display_path: &display_path,
        },
        entrypoint_file.parent().unwrap_or(Path::new(".")),
        "run",
    )
}

fn build_cached_rooted_entrypoint_binary(
    entrypoint: &RootedEntrypoint<'_>,
    cache_scope: &Path,
    cache_namespace: &str,
) -> Result<CachedBinaryArtifact, Vec<RenderedDiagnostic>> {
    let plan = RootedEntrypointPlan::from_entrypoint(entrypoint)?;
    plan.emit_frontend_diagnostics();
    let generated_project = plan.into_generated_binary_project()?;
    let cache_entry = materialize_cached_binary_project(
        cache_namespace,
        cache_scope,
        "sifr_output",
        generated_project,
    )?;
    Ok(CachedBinaryArtifact {
        binary_path: cached_binary_path(cache_entry.workspace_root(), "sifr_output"),
        cache_report: cache_entry.report().clone(),
    })
}

impl RootedEntrypointPlan {
    fn from_entrypoint(entrypoint: &RootedEntrypoint<'_>) -> Result<Self, Vec<RenderedDiagnostic>> {
        let stdlib = compile_stdlib()?;
        let (shape, project_lowering) = match entrypoint {
            RootedEntrypoint::SingleFile {
                source,
                display_path,
            } => {
                let parsed_suite = parse_source(source)?;
                let project_lowering = compile_single_frontend_module_with_source(
                    "main",
                    &parsed_suite,
                    FrontendSourceContext {
                        display_path,
                        source,
                    },
                    stdlib.defs.clone(),
                    FrontendDiagnosticStyle::Bare,
                )?;
                (RootedEntrypointShape::SingleFile, project_lowering)
            }
            RootedEntrypoint::Project { main_file } => {
                let project_dir = main_file.parent().unwrap_or(Path::new("."));
                let Some(main_module_name) = main_file
                    .file_stem()
                    .map(|stem| stem.to_string_lossy().to_string())
                else {
                    return Err(vec![crate::diagnostics::diagnostic_with_code(
                        format!("invalid project entrypoint path '{}'", main_file.display()),
                        DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
                    )]);
                };
                let root_modules = BTreeSet::from([main_module_name.clone()]);
                let resolver = match find_workspace_root(main_file)? {
                    Some(workspace_root) => {
                        ModuleResolver::with_workspace(project_dir, workspace_root)
                    }
                    None => ModuleResolver::entry_parent(project_dir),
                };
                let mut parsed_modules = parse_import_closure_source_modules(
                    &resolver,
                    &root_modules,
                    DiscoveryDiagnosticStyle::ModuleName,
                )?;
                if main_module_name != "main" {
                    if let Some(entry_module) = parsed_modules.remove(&main_module_name) {
                        parsed_modules.insert("main".to_string(), entry_module);
                    }
                }
                let project_lowering =
                    collect_project_hir_source_modules(&parsed_modules, stdlib.defs.clone())?;
                (RootedEntrypointShape::Project, project_lowering)
            }
            RootedEntrypoint::PackageProject { entrypoint } => {
                let (entry_module_name, mut parsed_modules) =
                    parse_package_import_closure_source_modules(
                        &entrypoint.graph,
                        &entrypoint.source_map,
                        &entrypoint.package_id,
                        &entrypoint.main_file,
                        DiscoveryDiagnosticStyle::ModuleName,
                    )?;
                if entry_module_name != "main" {
                    if let Some(entry_module) = parsed_modules.remove(&entry_module_name) {
                        parsed_modules.insert("main".to_string(), entry_module);
                    }
                }
                let project_lowering =
                    collect_project_hir_source_modules(&parsed_modules, stdlib.defs.clone())?;
                (RootedEntrypointShape::Project, project_lowering)
            }
        };

        Ok(Self {
            shape,
            stdlib,
            project_lowering,
        })
    }

    pub(crate) fn emit_frontend_diagnostics(&self) {
        emit_project_frontend_diagnostics(&self.project_lowering);
    }

    pub(crate) fn frontend_diagnostics(&self) -> Vec<RenderedDiagnostic> {
        self.project_lowering
            .compile_order
            .iter()
            .filter_map(|module_name| {
                self.project_lowering
                    .module_diagnostics
                    .get(module_name.as_str())
            })
            .flat_map(|diagnostics| {
                diagnostics
                    .rendered_warnings
                    .clone()
                    .into_iter()
                    .chain(diagnostics.rendered_reveal_types.clone())
            })
            .collect()
    }

    fn into_single_file_frontend(self) -> Result<FrontendCompiled, Vec<RenderedDiagnostic>> {
        if self.shape != RootedEntrypointShape::SingleFile {
            return Err(vec![crate::diagnostics::diagnostic_with_code(
                "internal error: rooted project entrypoint cannot be converted into a single-file frontend result",
                DiagnosticCode::INTERNAL_COMPILER_PANIC,
            )]);
        }

        let mut project_lowering = self.project_lowering;
        let main_module = project_lowering.hir_modules.remove("main").ok_or_else(|| {
            vec![crate::diagnostics::diagnostic_with_code(
                "internal error: frontend lowering missing 'main' module",
                DiagnosticCode::INTERNAL_COMPILER_PANIC,
            )]
        })?;
        let main_diag = project_lowering
            .module_diagnostics
            .remove("main")
            .unwrap_or_default();
        let lowering_result = LoweringResult {
            module: main_module,
            function_defaults: std::collections::HashMap::new(),
            function_varargs: std::collections::HashMap::new(),
            constant_integer_values: std::collections::HashMap::new(),
            reveal_types: main_diag.reveal_types,
            warnings: main_diag.warnings,
        };

        Ok(FrontendCompiled {
            stdlib: self.stdlib,
            lowering_result,
        })
    }

    fn into_single_file_codegen_result(
        self,
    ) -> Result<sifr_codegen::CodegenResult, Vec<RenderedDiagnostic>> {
        let frontend = self.into_single_file_frontend()?;
        run_codegen_with_boundary(
            "internal compiler panic during single-file code generation",
            || generate_rust_with_stdlib(&frontend.lowering_result.module, &frontend.stdlib.code),
        )
        .map_err(|error| vec![*error])
    }

    fn into_generated_binary_project(
        self,
    ) -> Result<GeneratedBinaryProject, Vec<RenderedDiagnostic>> {
        match self.shape {
            RootedEntrypointShape::SingleFile => {
                let codegen_result = self.into_single_file_codegen_result()?;
                Ok(generated_single_file_binary_project(codegen_result))
            }
            RootedEntrypointShape::Project => {
                generated_project_binary_project(&self.stdlib.code, self.project_lowering)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mktemp_dir(name: &str) -> PathBuf {
        let unique = format!(
            "sifr_rooted_entrypoint_{name}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        );
        let dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&dir).expect("temp dir should be created");
        dir
    }

    #[test]
    fn test_single_file_entrypoint_plan_generates_main_only_project() {
        let plan = RootedEntrypointPlan::from_entrypoint(&RootedEntrypoint::SingleFile {
            source: "def main():\n    print(\"ok\")\n",
            display_path: "main",
        })
        .expect("single-file entrypoint should compile");

        let generated_project = plan
            .into_generated_binary_project()
            .expect("single-file generated project should succeed");

        assert!(generated_project.support_modules.is_empty());
        assert!(generated_project.main_rs.contains("fn main"));
        assert!(generated_project.used_stdlib_modules.is_empty());
        assert!(generated_project.required_crates.is_empty());
    }

    #[test]
    fn test_project_entrypoint_plan_generates_support_modules() {
        let dir = mktemp_dir("project_positive");
        let main_file = dir.join("main.sifr");
        std::fs::write(
            &main_file,
            "from helper import message\n\ndef main():\n    print(message())\n",
        )
        .expect("main should be written");
        std::fs::write(
            dir.join("helper.sifr"),
            "def message() -> str:\n    return \"ok\"\n",
        )
        .expect("helper should be written");

        let plan = RootedEntrypointPlan::from_entrypoint(&RootedEntrypoint::Project {
            main_file: &main_file,
        })
        .expect("project entrypoint should compile");
        let generated_project = plan
            .into_generated_binary_project()
            .expect("project generated project should succeed");

        assert_eq!(
            generated_project
                .support_modules
                .keys()
                .cloned()
                .collect::<Vec<_>>(),
            vec!["helper".to_string()]
        );
        assert!(generated_project.main_rs.starts_with("mod helper;"));
        assert!(generated_project.main_rs.contains("fn main"));

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_project_entrypoint_plan_reports_reachable_frontend_errors() {
        let dir = mktemp_dir("project_negative");
        let main_file = dir.join("main.sifr");
        std::fs::write(
            &main_file,
            "from helper import broken\n\ndef main():\n    print(broken())\n",
        )
        .expect("main should be written");
        std::fs::write(
            dir.join("helper.sifr"),
            "def broken() -> int:\n    return \"bad\"\n",
        )
        .expect("helper should be written");

        let errors = match RootedEntrypointPlan::from_entrypoint(&RootedEntrypoint::Project {
            main_file: &main_file,
        }) {
            Ok(_) => panic!("reachable project type error should fail plan construction"),
            Err(errors) => errors,
        };

        assert!(errors
            .iter()
            .any(|error| error.message.contains("[helper] return type mismatch")));

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_project_entrypoint_plan_aggregates_reachable_dependency_metadata() {
        let dir = mktemp_dir("project_metadata_positive");
        let main_file = dir.join("main.sifr");
        std::fs::write(
            &main_file,
            "from helper import helper\n\ndef main():\n    print(helper())\n",
        )
        .expect("main should be written");
        std::fs::write(
            dir.join("helper.sifr"),
            "from sifr.statistics import mean\n\n\
def helper() -> bigint:\n    return bigint(1)\n",
        )
        .expect("helper should be written");

        let plan = RootedEntrypointPlan::from_entrypoint(&RootedEntrypoint::Project {
            main_file: &main_file,
        })
        .expect("project entrypoint should compile");
        let generated_project = plan
            .into_generated_binary_project()
            .expect("project metadata aggregation should succeed");

        assert!(generated_project
            .used_stdlib_modules
            .contains("sifr.statistics"));
        assert!(generated_project.used_stdlib_modules.contains("sifr.math"));
        assert!(generated_project.required_crates.contains("num-bigint"));
        assert!(generated_project.required_crates.contains("num-traits"));

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_project_entrypoint_plan_ignores_unreachable_dependency_metadata() {
        let dir = mktemp_dir("project_metadata_negative");
        let main_file = dir.join("main.sifr");
        std::fs::write(
            &main_file,
            "from helper import helper\n\ndef main():\n    print(helper())\n",
        )
        .expect("main should be written");
        std::fs::write(
            dir.join("helper.sifr"),
            "def helper() -> int:\n    return 1\n",
        )
        .expect("helper should be written");
        std::fs::write(
            dir.join("unused_dependency.sifr"),
            "from sifr.json import dumps\n\ndef unused() -> str:\n    return dumps({\"x\": 1})\n",
        )
        .expect("unused dependency should be written");

        let plan = RootedEntrypointPlan::from_entrypoint(&RootedEntrypoint::Project {
            main_file: &main_file,
        })
        .expect("project entrypoint should compile");
        let generated_project = plan
            .into_generated_binary_project()
            .expect("project metadata aggregation should succeed");

        assert!(!generated_project.used_stdlib_modules.contains("sifr.json"));
        assert!(!generated_project.required_crates.contains("serde_json"));

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_cached_project_binary_reuses_workspace_for_unchanged_input() {
        let dir = mktemp_dir("cached_project_reuse");
        let main_file = dir.join("main.sifr");
        std::fs::write(
            &main_file,
            "from helper import value\n\ndef main():\n    print(value())\n",
        )
        .expect("main should be written");
        std::fs::write(
            dir.join("helper.sifr"),
            "def value() -> int:\n    return 11\n",
        )
        .expect("helper should be written");

        let first =
            build_cached_project_binary(&main_file).expect("first cached build should succeed");
        assert!(first.binary_path().exists());
        assert!(!first.cache_report().cache_hit());

        let first_output = std::process::Command::new(first.binary_path())
            .output()
            .expect("first cached binary should run");
        assert!(first_output.status.success());
        assert_eq!(String::from_utf8_lossy(&first_output.stdout).trim(), "11");

        let second =
            build_cached_project_binary(&main_file).expect("second cached build should succeed");
        assert!(second.cache_report().cache_hit());
        assert_eq!(first.binary_path(), second.binary_path());

        let second_output = std::process::Command::new(second.binary_path())
            .output()
            .expect("second cached binary should run");
        assert!(second_output.status.success());
        assert_eq!(String::from_utf8_lossy(&second_output.stdout).trim(), "11");

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_cached_project_binary_invalidates_when_sources_change() {
        let dir = mktemp_dir("cached_project_invalidation");
        let main_file = dir.join("main.sifr");
        std::fs::write(
            &main_file,
            "from helper import value\n\ndef main():\n    print(value())\n",
        )
        .expect("main should be written");
        let helper = dir.join("helper.sifr");
        std::fs::write(&helper, "def value() -> int:\n    return 21\n")
            .expect("helper should be written");

        let first =
            build_cached_project_binary(&main_file).expect("first cached build should succeed");
        assert!(!first.cache_report().cache_hit());

        std::fs::write(&helper, "def value() -> int:\n    return 22\n")
            .expect("helper should be updated");
        let second =
            build_cached_project_binary(&main_file).expect("second cached build should succeed");
        assert!(!second.cache_report().cache_hit());
        assert_ne!(first.binary_path(), second.binary_path());
        assert_ne!(first.cache_report().key(), second.cache_report().key());

        let output = std::process::Command::new(second.binary_path())
            .output()
            .expect("updated cached binary should run");
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "22");

        let _ = std::fs::remove_dir_all(dir);
    }
}
