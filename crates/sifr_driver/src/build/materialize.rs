use super::cargo_manifest::generate_dependency_cargo_toml;
use super::project_codegen::GeneratedBinaryProject;
use crate::diagnostics::{CompileError, CompilePhase};
use std::path::{Path, PathBuf};
use std::process::Command;

pub(super) fn materialize_binary_project(
    output_dir: &Path,
    project_name: &str,
    generated_project: GeneratedBinaryProject,
) -> Result<PathBuf, Vec<CompileError>> {
    let project_path = output_dir.join(project_name);
    let src_dir = project_path.join("src");
    std::fs::create_dir_all(&src_dir).map_err(|error| {
        vec![CompileError {
            message: format!("failed to create output directory: {error}"),
            phase: CompilePhase::Build,
        }]
    })?;

    let cargo_toml = generate_dependency_cargo_toml(
        project_name,
        &generated_project.used_stdlib_modules,
        &generated_project.required_crates,
    );

    std::fs::write(project_path.join("Cargo.toml"), cargo_toml).map_err(|error| {
        vec![CompileError {
            message: format!("failed to write Cargo.toml: {error}"),
            phase: CompilePhase::Build,
        }]
    })?;

    std::fs::write(src_dir.join("main.rs"), generated_project.main_rs).map_err(|error| {
        vec![CompileError {
            message: format!("failed to write main.rs: {error}"),
            phase: CompilePhase::Build,
        }]
    })?;

    for (module_name, code) in generated_project.support_modules {
        std::fs::write(src_dir.join(format!("{module_name}.rs")), code).map_err(|error| {
            vec![CompileError {
                message: format!("failed to write {module_name}.rs: {error}"),
                phase: CompilePhase::Build,
            }]
        })?;
    }

    let output = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(&project_path)
        .output()
        .map_err(|error| {
            vec![CompileError {
                message: format!("failed to run cargo build: {error}"),
                phase: CompilePhase::Build,
            }]
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(vec![CompileError {
            message: format!("cargo build failed:\n{stderr}"),
            phase: CompilePhase::Build,
        }]);
    }

    let binary_name = if cfg!(target_os = "windows") {
        format!("{project_name}.exe")
    } else {
        project_name.to_string()
    };
    Ok(project_path
        .join("target")
        .join("release")
        .join(binary_name))
}
