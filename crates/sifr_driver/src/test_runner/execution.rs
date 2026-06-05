use super::artifacts::{compose_test_runner_lib, generate_test_runner_cargo_toml};
use super::orchestrator::GeneratedTestRunnerProject;
use crate::build::{prepare_cached_artifact, ArtifactCacheReport, PreparedArtifactCache};
use crate::diagnostics::{write_stderr, write_stderr_line, RenderedDiagnostic};
use crate::project::{namespace_module_files, rust_module_file_path};
use sifr_diagnostics::DiagnosticCode;
use std::path::Path;

pub(crate) struct TestRunnerExecutionOutcome {
    pub(crate) success: bool,
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) cache_report: ArtifactCacheReport,
}

pub(crate) fn execute_test_runner_project(
    generated_project: &GeneratedTestRunnerProject,
) -> Result<TestRunnerExecutionOutcome, Vec<RenderedDiagnostic>> {
    let cargo_toml = generate_test_runner_cargo_toml(
        &generated_project.all_stdlib_modules,
        &generated_project.all_required_features,
    );
    let test_lib = compose_test_runner_lib(
        &generated_project.support_module_names,
        &generated_project.all_rust_code,
    );
    let cache_key = test_runner_cache_key(generated_project, &cargo_toml, &test_lib);
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

        std::fs::write(project_dir.join("Cargo.toml"), cargo_toml).map_err(|error| {
            vec![crate::diagnostics::diagnostic_with_code(
                format!("failed to write Cargo.toml: {error}"),
                DiagnosticCode::BUILD_CARGO_MANIFEST_FAILURE,
            )]
        })?;

        for module_name in &generated_project.support_module_names {
            if let Some(code) = generated_project.support_rust_files.get(module_name) {
                let module_path = rust_module_file_path(module_name);
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

    let output = std::process::Command::new("cargo")
        .args(["test"])
        .current_dir(&project_dir)
        .output()
        .map_err(|error| {
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
    let mut stdlib_modules: Vec<&str> = generated_project
        .all_stdlib_modules
        .iter()
        .map(String::as_str)
        .collect();
    stdlib_modules.sort_unstable();
    let mut required_features: Vec<&str> = generated_project
        .all_required_features
        .iter()
        .map(|feature| feature.id())
        .collect();
    required_features.sort_unstable();
    format!(
        "[scope]\n{}\n[Cargo.toml]\n{cargo_toml}\n[src/lib.rs]\n{test_lib}\n[support]\n{support_modules}\n[stdlib]\n{}\n[crates]\n{}",
        generated_project.cache_scope.display(),
        stdlib_modules.join("\n"),
        required_features.join("\n")
    )
}
