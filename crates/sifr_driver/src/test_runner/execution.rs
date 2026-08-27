use super::artifacts::{compose_test_runner_lib, try_generate_test_runner_cargo_plan};
use super::orchestrator::GeneratedTestRunnerProject;
use crate::build::{
    ArtifactCacheReport, CargoResolutionPolicy, GeneratedCargoCommand, GeneratedCargoExecution,
    GeneratedCargoProject, PreparedArtifactCache, cargo_resolution_cache_key_fragment,
    create_invocation_workspace, materialize_generated_cargo_project, prepare_cached_artifact,
    run_generated_cargo_command,
};
use crate::diagnostics::{RenderedDiagnostic, write_stderr, write_stderr_line};
use sifr_diagnostics::DiagnosticCode;
use sifr_package::CargoLockMode;
use sifr_stdlib_manifest::SysrootDependencyPlan;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(crate) struct TestRunnerExecutionOutcome {
    pub(crate) success: bool,
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) cache_report: ArtifactCacheReport,
}

pub(crate) fn execute_test_runner_project(
    generated_project: &GeneratedTestRunnerProject,
    lock_mode: CargoLockMode,
) -> Result<TestRunnerExecutionOutcome, Vec<RenderedDiagnostic>> {
    let cargo_plan = try_generate_test_runner_cargo_plan(
        &generated_project.all_stdlib_modules,
        &generated_project.all_required_features,
        &generated_project.interop,
    )
    .map_err(|error| {
        vec![crate::diagnostics::diagnostic_with_code(
            error.boundary_message(),
            DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
        )]
    })?;
    let test_lib = compose_test_runner_lib(
        &generated_project.support_module_names,
        &generated_project.all_rust_code,
    );
    let cargo_resolution = CargoResolutionPolicy::for_test_scope(
        &generated_project.cache_scope,
        lock_mode,
        &cargo_plan.dependency_plan,
    );
    let cache_key = test_runner_cache_key(
        generated_project,
        &cargo_plan.cargo_toml,
        &test_lib,
        &cargo_plan.dependency_plan,
        &cargo_resolution,
    )?;
    let required_paths = [Path::new("Cargo.toml"), Path::new("src/lib.rs")];
    let prepared = prepare_cached_artifact(
        "test_runner",
        &generated_project.cache_scope,
        &cache_key,
        &required_paths,
    )?;
    let cache_entry = match prepared {
        PreparedArtifactCache::Hit(entry) => entry,
        PreparedArtifactCache::Miss(pending) => {
            materialize_generated_cargo_project(
                pending.workspace_root(),
                GeneratedCargoProject {
                    name: "sifr_tests".to_string(),
                    crate_root_file: PathBuf::from("lib.rs"),
                    crate_root_source: test_lib,
                    support_modules: generated_project
                        .support_rust_files
                        .iter()
                        .map(|(name, source)| (name.clone(), source.clone()))
                        .collect::<BTreeMap<_, _>>(),
                    support_main_alias: Some(PathBuf::from("__sifr_support_main.rs")),
                    interop: generated_project.interop.clone(),
                },
                &cargo_plan.dependency_plan,
            )?;
            pending.commit(&required_paths)?
        }
    };
    let cache_report = cache_entry.report().clone();
    let invocation = TestInvocationWorkspace::create()?;
    let project_dir = invocation.root().join("sifr_tests");
    copy_cached_project(cache_entry.workspace_root(), &project_dir)?;
    let target_directory = test_target_directory(cache_entry.workspace_root());
    let output = run_generated_cargo_command(
        &project_dir,
        GeneratedCargoCommand::Test,
        GeneratedCargoExecution {
            python_interpreter: None,
            target_directory: Some(&target_directory),
            additional_trusted_native_links: &BTreeSet::new(),
        },
        &generated_project.interop,
        &cargo_plan.dependency_plan,
        &cargo_resolution,
    )?;

    let stdout = user_facing_cargo_stdout(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stdout.is_empty() {
        write_stderr(&stdout);
    }
    if !stderr.is_empty() {
        write_stderr(&stderr);
    }

    write_stderr_line(&cache_report.status_line());

    Ok(TestRunnerExecutionOutcome {
        success: output.status.success(),
        cache_report,
    })
}

fn user_facing_cargo_stdout(stdout: &[u8]) -> String {
    let mut output = String::new();
    for line in String::from_utf8_lossy(stdout).lines() {
        let is_cargo_message = serde_json::from_str::<serde_json::Value>(line).is_ok_and(|value| {
            value
                .as_object()
                .is_some_and(|object| object.contains_key("reason"))
        });
        if is_cargo_message {
            continue;
        }
        output.push_str(line);
        output.push('\n');
    }
    output
}

fn test_target_directory(cache_workspace: &Path) -> PathBuf {
    cache_workspace.with_extension("target")
}

fn test_runner_cache_key(
    generated_project: &GeneratedTestRunnerProject,
    cargo_toml: &str,
    test_lib: &str,
    dependency_plan: &SysrootDependencyPlan,
    cargo_resolution: &CargoResolutionPolicy,
) -> Result<String, Vec<RenderedDiagnostic>> {
    let mut support_modules: Vec<(&str, &str)> = generated_project
        .support_rust_files
        .iter()
        .map(|(name, code)| (name.as_str(), code.as_str()))
        .collect();
    support_modules.sort_unstable_by(|left, right| left.0.cmp(right.0));
    let support_modules = support_modules
        .into_iter()
        .map(|(name, code)| format!("{name}\n{code}"))
        .collect::<Vec<_>>()
        .join("\n===\n");
    let cargo_resolution = cargo_resolution_cache_key_fragment(cargo_resolution)?;
    Ok(format!(
        "[scope]\n{}\n[Cargo.toml]\n{cargo_toml}\n[src/lib.rs]\n{test_lib}\n[support]\n{support_modules}\n[sysroot-dependency-inputs]\n{}[sysroot-dependency-plan]\n{}\n[cargo-resolution]\n{cargo_resolution}",
        generated_project.cache_scope.display(),
        dependency_plan.dependency_input_fingerprint(),
        dependency_plan.cache_fingerprint
    ))
}

struct TestInvocationWorkspace {
    root: PathBuf,
}

impl TestInvocationWorkspace {
    fn create() -> Result<Self, Vec<RenderedDiagnostic>> {
        create_invocation_workspace("test_runner").map(|root| Self { root })
    }

    fn root(&self) -> &Path {
        &self.root
    }
}

impl Drop for TestInvocationWorkspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn copy_cached_project(source: &Path, destination: &Path) -> Result<(), Vec<RenderedDiagnostic>> {
    std::fs::create_dir_all(destination).map_err(|error| {
        vec![materialization_error(format!(
            "failed to create test execution directory '{}': {error}",
            destination.display()
        ))]
    })?;
    copy_project_file(source, destination, Path::new("Cargo.toml"))?;
    copy_project_directory(&source.join("src"), &destination.join("src"))
}

fn copy_project_directory(
    source: &Path,
    destination: &Path,
) -> Result<(), Vec<RenderedDiagnostic>> {
    std::fs::create_dir_all(destination).map_err(|error| {
        vec![materialization_error(format!(
            "failed to create test source directory '{}': {error}",
            destination.display()
        ))]
    })?;
    let entries = std::fs::read_dir(source).map_err(|error| {
        vec![materialization_error(format!(
            "failed to read cached test source directory '{}': {error}",
            source.display()
        ))]
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| {
            vec![materialization_error(format!(
                "failed to read cached test source entry: {error}"
            ))]
        })?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let file_type = entry.file_type().map_err(|error| {
            vec![materialization_error(format!(
                "failed to inspect cached test source '{}': {error}",
                source_path.display()
            ))]
        })?;
        if file_type.is_dir() {
            copy_project_directory(&source_path, &destination_path)?;
        } else if file_type.is_file() {
            std::fs::copy(&source_path, &destination_path).map_err(|error| {
                vec![materialization_error(format!(
                    "failed to copy cached test source '{}': {error}",
                    source_path.display()
                ))]
            })?;
        } else {
            return Err(vec![materialization_error(format!(
                "cached test source '{}' is not a regular file or directory",
                source_path.display()
            ))]);
        }
    }
    Ok(())
}

fn copy_project_file(
    source_root: &Path,
    destination_root: &Path,
    relative: &Path,
) -> Result<(), Vec<RenderedDiagnostic>> {
    std::fs::copy(source_root.join(relative), destination_root.join(relative)).map_err(
        |error| {
            vec![materialization_error(format!(
                "failed to copy cached test project file '{}': {error}",
                relative.display()
            ))]
        },
    )?;
    Ok(())
}

fn materialization_error(message: String) -> RenderedDiagnostic {
    crate::diagnostics::diagnostic_with_code(message, DiagnosticCode::BUILD_MATERIALIZATION_FAILURE)
}

#[cfg(test)]
mod tests {
    use super::{test_runner_cache_key, user_facing_cargo_stdout};
    use crate::build::CargoResolutionPolicy;
    use crate::test_runner::orchestrator::GeneratedTestRunnerProject;
    use sifr_codegen::InteropBuildPlan;
    use sifr_package::CargoLockMode;
    use sifr_stdlib_manifest::{
        CargoVendorMode, StdlibFeature, SysrootCrate, SysrootCrateDependency, SysrootDependencyPlan,
    };
    use std::collections::{BTreeSet, HashMap, HashSet};
    use std::path::PathBuf;

    #[test]
    fn cargo_stdout_filter_preserves_user_json_values() {
        let stdout = br#"{"reason":"compiler-artifact","fresh":true}
42
true
null
"done"
{"payload":1}
plain text
"#;

        assert_eq!(
            user_facing_cargo_stdout(stdout),
            "42\ntrue\nnull\n\"done\"\n{\"payload\":1}\nplain text\n"
        );
    }

    #[test]
    fn test_runner_cache_key_uses_sysroot_dependency_plan_inputs() {
        let generated_project = GeneratedTestRunnerProject {
            cache_scope: PathBuf::from("/tmp/sifr-tests"),
            support_module_names: Vec::new(),
            support_rust_files: HashMap::new(),
            all_rust_code: "#[test]\nfn test_case() {}\n".to_string(),
            all_stdlib_modules: HashSet::from(["sifr.json".to_string()]),
            all_required_features: HashSet::from([StdlibFeature::SerdeJson]),
            interop: InteropBuildPlan::default(),
        };
        let dependency_plan = SysrootDependencyPlan {
            stdlib_modules: BTreeSet::from(["sifr.json".to_string()]),
            required_features: BTreeSet::from([StdlibFeature::SerdeJson]),
            sysroot_root: "/sysroot".into(),
            toolchain_id: "0.1.0-test-aarch64-test".to_string(),
            sysroot_content_sha256: "0".repeat(64),
            cargo_config: "/sysroot/.cargo/config.toml".into(),
            vendor_dir: "/sysroot/vendor".into(),
            crates: vec![SysrootCrateDependency {
                krate: SysrootCrate::SifrStdlib,
                path: "/sysroot/crates/sifr_stdlib".into(),
                features: BTreeSet::from(["json".to_string()]),
            }],
            retained_direct_dependencies: Vec::new(),
            cargo_vendor_mode: CargoVendorMode::SysrootOnly,
            cache_fingerprint: "fingerprint-a".to_string(),
        };

        let cargo_resolution = CargoResolutionPolicy::for_test_scope(
            PathBuf::from("/tmp/sifr-tests").as_path(),
            CargoLockMode::Normal,
            &dependency_plan,
        );
        let cache_key = test_runner_cache_key(
            &generated_project,
            "[package]\nname = \"sifr_tests\"\n",
            "#[test]\nfn test_case() {}\n",
            &dependency_plan,
            &cargo_resolution,
        )
        .expect("normal Cargo resolution should have a cache identity");

        assert!(cache_key.contains(
            "[sysroot-dependency-inputs]\n[stdlib]\nsifr.json\n[features]\nserde_json\n"
        ));
        assert!(cache_key.contains("[sysroot-dependency-plan]\nfingerprint-a"));
    }
}
