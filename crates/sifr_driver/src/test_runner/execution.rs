use super::artifacts::{compose_test_runner_lib, generate_test_runner_cargo_toml};
use super::orchestrator::GeneratedTestRunnerProject;
use crate::build::{create_invocation_workspace, InvocationWorkspaceGuard};
use crate::diagnostics::{write_stderr, CompileError, CompilePhase};

pub(super) fn execute_test_runner_project(
    generated_project: &GeneratedTestRunnerProject,
) -> Result<bool, Vec<CompileError>> {
    let project_dir = create_invocation_workspace("test_runner")?;
    let _workspace_guard = InvocationWorkspaceGuard::new(project_dir.clone());
    let src_dir = project_dir.join("src");
    std::fs::create_dir_all(&src_dir).map_err(|error| {
        vec![CompileError {
            message: format!("failed to create test directory: {error}"),
            phase: CompilePhase::Build,
        }]
    })?;

    let cargo_toml = generate_test_runner_cargo_toml(
        &generated_project.all_stdlib_modules,
        &generated_project.all_required_crates,
    );
    std::fs::write(project_dir.join("Cargo.toml"), cargo_toml).map_err(|error| {
        vec![CompileError {
            message: format!("failed to write Cargo.toml: {error}"),
            phase: CompilePhase::Build,
        }]
    })?;

    for module_name in &generated_project.support_module_names {
        if let Some(code) = generated_project.support_rust_files.get(module_name) {
            std::fs::write(src_dir.join(format!("{module_name}.rs")), code).map_err(|error| {
                vec![CompileError {
                    message: format!("failed to write {module_name}.rs: {error}"),
                    phase: CompilePhase::Build,
                }]
            })?;
        }
    }

    let test_lib = compose_test_runner_lib(
        &generated_project.support_module_names,
        &generated_project.all_rust_code,
    );
    std::fs::write(src_dir.join("lib.rs"), &test_lib).map_err(|error| {
        vec![CompileError {
            message: format!("failed to write lib.rs: {error}"),
            phase: CompilePhase::Build,
        }]
    })?;

    let output = std::process::Command::new("cargo")
        .args(["test"])
        .current_dir(&project_dir)
        .output()
        .map_err(|error| {
            vec![CompileError {
                message: format!("failed to run cargo test: {error}"),
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
