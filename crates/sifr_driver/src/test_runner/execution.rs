use super::artifacts::{
    compose_test_runner_lib, test_support_module_file_path, try_generate_test_runner_cargo_plan,
};
use super::orchestrator::GeneratedTestRunnerProject;
use crate::build::{
    prepare_cached_artifact, sysroot_cargo_config_args, ArtifactCacheReport, PreparedArtifactCache,
};
use crate::diagnostics::{write_stderr, write_stderr_line, RenderedDiagnostic};
use crate::project::namespace_module_files;
use sifr_diagnostics::DiagnosticCode;
use sifr_stdlib_manifest::SysrootDependencyPlan;
use std::path::Path;

pub(crate) struct TestRunnerExecutionOutcome {
    pub(crate) success: bool,
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) cache_report: ArtifactCacheReport,
}

pub(crate) fn execute_test_runner_project(
    generated_project: &GeneratedTestRunnerProject,
) -> Result<TestRunnerExecutionOutcome, Vec<RenderedDiagnostic>> {
    let cargo_plan = try_generate_test_runner_cargo_plan(
        &generated_project.all_stdlib_modules,
        &generated_project.all_required_features,
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
    let cache_key = test_runner_cache_key(
        generated_project,
        &cargo_plan.cargo_toml,
        &test_lib,
        &cargo_plan.dependency_plan,
    );
    let required_paths = [
        Path::new("Cargo.toml"),
        Path::new("src/lib.rs"),
        Path::new("target"),
    ];
    let prepared = prepare_cached_artifact(
        "test_runner",
        &generated_project.cache_scope,
        &cache_key,
        &required_paths,
    )?;
    let project_dir = prepared.workspace_root().to_path_buf();
    if let PreparedArtifactCache::Miss(_) = &prepared {
        let src_dir = project_dir.join("src");
        std::fs::create_dir_all(&src_dir).map_err(|error| {
            vec![crate::diagnostics::diagnostic_with_code(
                format!("failed to create test directory: {error}"),
                DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
            )]
        })?;

        std::fs::write(project_dir.join("Cargo.toml"), &cargo_plan.cargo_toml).map_err(
            |error| {
                vec![crate::diagnostics::diagnostic_with_code(
                    format!("failed to write Cargo.toml: {error}"),
                    DiagnosticCode::BUILD_CARGO_MANIFEST_FAILURE,
                )]
            },
        )?;

        for module_name in &generated_project.support_module_names {
            if let Some(code) = generated_project.support_rust_files.get(module_name) {
                let module_path = test_support_module_file_path(module_name);
                let output_path = src_dir.join(&module_path);
                if let Some(parent) = output_path.parent() {
                    std::fs::create_dir_all(parent).map_err(|error| {
                        vec![crate::diagnostics::diagnostic_with_code(
                            format!(
                                "failed to create test support module directory '{}': {error}",
                                parent.display()
                            ),
                            DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
                        )]
                    })?;
                }
                std::fs::write(&output_path, code).map_err(|error| {
                    vec![crate::diagnostics::diagnostic_with_code(
                        format!(
                            "failed to write test support module '{}': {error}",
                            output_path.display()
                        ),
                        DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
                    )]
                })?;
            }
        }

        for namespace_file in namespace_module_files(&generated_project.support_module_names) {
            let output_path = src_dir.join(&namespace_file.path);
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent).map_err(|error| {
                    vec![crate::diagnostics::diagnostic_with_code(
                        format!(
                            "failed to create test support namespace directory '{}': {error}",
                            parent.display()
                        ),
                        DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
                    )]
                })?;
            }
            let mut contents = String::new();
            for declaration in namespace_file.declarations {
                contents.push_str("pub mod ");
                contents.push_str(&declaration);
                contents.push_str(";\n");
            }
            std::fs::write(&output_path, contents).map_err(|error| {
                vec![crate::diagnostics::diagnostic_with_code(
                    format!(
                        "failed to write test support namespace '{}': {error}",
                        output_path.display()
                    ),
                    DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
                )]
            })?;
        }

        std::fs::write(src_dir.join("lib.rs"), &test_lib).map_err(|error| {
            vec![crate::diagnostics::diagnostic_with_code(
                format!("failed to write lib.rs: {error}"),
                DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
            )]
        })?;
    }

    let mut command = std::process::Command::new("cargo");
    command
        .args(sysroot_cargo_config_args(&cargo_plan.dependency_plan))
        .args(["test"])
        .current_dir(&project_dir);
    command.env_remove("CARGO_TARGET_DIR");
    let output = command.output().map_err(|error| {
        vec![crate::diagnostics::diagnostic_with_code(
            format!("failed to run cargo test: {error}"),
            DiagnosticCode::BUILD_RUSTC_OR_CARGO_FAILURE,
        )]
    })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stdout.is_empty() {
        write_stderr(&stdout);
    }
    if !stderr.is_empty() {
        write_stderr(&stderr);
    }

    let cache_report = match prepared {
        PreparedArtifactCache::Hit(entry) => entry.report().clone(),
        PreparedArtifactCache::Miss(entry) => entry.commit(&required_paths)?.report().clone(),
    };
    write_stderr_line(&cache_report.status_line());

    Ok(TestRunnerExecutionOutcome {
        success: output.status.success(),
        cache_report,
    })
}

fn test_runner_cache_key(
    generated_project: &GeneratedTestRunnerProject,
    cargo_toml: &str,
    test_lib: &str,
    dependency_plan: &SysrootDependencyPlan,
) -> String {
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
    format!(
        "[scope]\n{}\n[Cargo.toml]\n{cargo_toml}\n[src/lib.rs]\n{test_lib}\n[support]\n{support_modules}\n[sysroot-dependency-inputs]\n{}[sysroot-dependency-plan]\n{}",
        generated_project.cache_scope.display(),
        dependency_plan.dependency_input_fingerprint(),
        dependency_plan.cache_fingerprint
    )
}

#[cfg(test)]
mod tests {
    use super::test_runner_cache_key;
    use crate::test_runner::orchestrator::GeneratedTestRunnerProject;
    use sifr_stdlib_manifest::{
        CargoVendorMode, StdlibFeature, SysrootCrate, SysrootCrateDependency, SysrootDependencyPlan,
    };
    use std::collections::{BTreeSet, HashMap, HashSet};
    use std::path::PathBuf;

    #[test]
    fn test_runner_cache_key_uses_sysroot_dependency_plan_inputs() {
        let generated_project = GeneratedTestRunnerProject {
            cache_scope: PathBuf::from("/tmp/sifr-tests"),
            support_module_names: Vec::new(),
            support_rust_files: HashMap::new(),
            all_rust_code: "#[test]\nfn test_case() {}\n".to_string(),
            all_stdlib_modules: HashSet::from(["sifr.json".to_string()]),
            all_required_features: HashSet::from([StdlibFeature::SerdeJson]),
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

        let cache_key = test_runner_cache_key(
            &generated_project,
            "[package]\nname = \"sifr_tests\"\n",
            "#[test]\nfn test_case() {}\n",
            &dependency_plan,
        );

        assert!(cache_key.contains(
            "[sysroot-dependency-inputs]\n[stdlib]\nsifr.json\n[features]\nserde_json\n"
        ));
        assert!(cache_key.contains("[sysroot-dependency-plan]\nfingerprint-a"));
    }
}
