use super::materialize::materialize_binary_project;
use super::project_codegen::{
    generated_project_binary_project, generated_single_file_binary_project, GeneratedBinaryProject,
};
use crate::diagnostics::{run_codegen_with_boundary, CompileError, CompilePhase};
use crate::frontend::{parse_source, FrontendCompiled, FrontendDiagnosticStyle};
use crate::project::{
    collect_project_hir_modules, compile_frontend_modules, emit_project_frontend_diagnostics,
    parse_import_closure_modules, DiscoveryDiagnosticStyle, ProjectLowering,
};
use crate::stdlib::{compile_stdlib, StdlibCompiled};
use sifr_codegen::generate_rust_with_stdlib;
use sifr_hir::LoweringResult;
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RootedEntrypointShape {
    SingleFile,
    Project,
}

pub(crate) enum RootedEntrypoint<'a> {
    SingleFile { source: &'a str },
    Project { main_file: &'a Path },
}

pub(crate) struct RootedEntrypointPlan {
    shape: RootedEntrypointShape,
    stdlib: StdlibCompiled,
    project_lowering: ProjectLowering,
}

pub(crate) fn compile_single_file_frontend(
    source: &str,
) -> Result<FrontendCompiled, Vec<CompileError>> {
    RootedEntrypointPlan::from_entrypoint(RootedEntrypoint::SingleFile { source })?
        .into_single_file_frontend()
}

pub(crate) fn compile_single_file_entrypoint_with_metadata(
    source: &str,
) -> Result<sifr_codegen::CodegenResult, Vec<CompileError>> {
    let plan = RootedEntrypointPlan::from_entrypoint(RootedEntrypoint::SingleFile { source })?;
    plan.emit_frontend_diagnostics();
    plan.into_single_file_codegen_result()
}

pub(crate) fn resolve_project_entrypoint_plan(
    main_file: &Path,
) -> Result<RootedEntrypointPlan, Vec<CompileError>> {
    RootedEntrypointPlan::from_entrypoint(RootedEntrypoint::Project { main_file })
}

pub(crate) fn build_rooted_entrypoint_binary(
    entrypoint: RootedEntrypoint<'_>,
    output_dir: &Path,
) -> Result<PathBuf, Vec<CompileError>> {
    let plan = RootedEntrypointPlan::from_entrypoint(entrypoint)?;
    plan.emit_frontend_diagnostics();
    let generated_project = plan.into_generated_binary_project()?;
    materialize_binary_project(output_dir, "sifr_output", generated_project)
}

impl RootedEntrypointPlan {
    fn from_entrypoint(entrypoint: RootedEntrypoint<'_>) -> Result<Self, Vec<CompileError>> {
        let stdlib = compile_stdlib()?;
        let (shape, project_lowering) = match entrypoint {
            RootedEntrypoint::SingleFile { source } => {
                let parsed_suite = parse_source(source)?;
                let mut parsed_modules = HashMap::new();
                parsed_modules.insert("main".to_string(), parsed_suite);
                let project_lowering = compile_frontend_modules(
                    &parsed_modules,
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
                    return Err(vec![CompileError {
                        message: format!(
                            "invalid project entrypoint path '{}'",
                            main_file.display()
                        ),
                        phase: CompilePhase::Build,
                    }]);
                };
                let root_modules = BTreeSet::from([main_module_name]);
                let parsed_modules = parse_import_closure_modules(
                    project_dir,
                    &root_modules,
                    DiscoveryDiagnosticStyle::ModuleName,
                )?;
                let project_lowering =
                    collect_project_hir_modules(&parsed_modules, stdlib.defs.clone())?;
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

    fn into_single_file_frontend(self) -> Result<FrontendCompiled, Vec<CompileError>> {
        if self.shape != RootedEntrypointShape::SingleFile {
            return Err(vec![CompileError {
                message:
                    "internal error: rooted project entrypoint cannot be converted into a single-file frontend result"
                        .to_string(),
                phase: CompilePhase::Build,
            }]);
        }

        let mut project_lowering = self.project_lowering;
        let main_module = project_lowering.hir_modules.remove("main").ok_or_else(|| {
            vec![CompileError {
                message: "internal error: frontend lowering missing 'main' module".to_string(),
                phase: CompilePhase::TypeCheck,
            }]
        })?;
        let main_diag = project_lowering
            .module_diagnostics
            .remove("main")
            .unwrap_or_default();
        let lowering_result = LoweringResult {
            module: main_module,
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
    ) -> Result<sifr_codegen::CodegenResult, Vec<CompileError>> {
        let frontend = self.into_single_file_frontend()?;
        run_codegen_with_boundary(
            "internal compiler panic during single-file code generation",
            || generate_rust_with_stdlib(&frontend.lowering_result.module, &frontend.stdlib.code),
        )
        .map_err(|error| vec![error])
    }

    fn into_generated_binary_project(self) -> Result<GeneratedBinaryProject, Vec<CompileError>> {
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
        let plan = RootedEntrypointPlan::from_entrypoint(RootedEntrypoint::SingleFile {
            source: "def main():\n    print(\"ok\")\n",
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

        let plan = RootedEntrypointPlan::from_entrypoint(RootedEntrypoint::Project {
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

        let errors = match RootedEntrypointPlan::from_entrypoint(RootedEntrypoint::Project {
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

        let plan = RootedEntrypointPlan::from_entrypoint(RootedEntrypoint::Project {
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

        let plan = RootedEntrypointPlan::from_entrypoint(RootedEntrypoint::Project {
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
}
