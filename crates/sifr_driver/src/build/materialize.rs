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
        vec![build_error(format!(
            "failed to create output directory: {error}"
        ))]
    })?;

    let cargo_toml = generate_dependency_cargo_toml(
        project_name,
        &generated_project.used_stdlib_modules,
        &generated_project.required_crates,
    );

    write_project_file(&project_path.join("Cargo.toml"), cargo_toml, "Cargo.toml")?;

    write_project_file(
        &src_dir.join("main.rs"),
        generated_project.main_rs,
        "main.rs",
    )?;

    for (module_name, code) in generated_project.support_modules {
        let file_name = format!("{module_name}.rs");
        write_project_file(&src_dir.join(&file_name), code, &file_name)?;
    }

    let output = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(&project_path)
        .output()
        .map_err(|error| vec![build_error(format!("failed to run cargo build: {error}"))])?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(vec![build_error(format!("cargo build failed:\n{stderr}"))]);
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

fn write_project_file(
    path: &Path,
    contents: impl AsRef<[u8]>,
    label: &str,
) -> Result<(), Vec<CompileError>> {
    std::fs::write(path, contents)
        .map_err(|error| vec![build_error(format!("failed to write {label}: {error}"))])
}

fn build_error(message: String) -> CompileError {
    CompileError {
        message,
        phase: CompilePhase::Build,
    }
}
