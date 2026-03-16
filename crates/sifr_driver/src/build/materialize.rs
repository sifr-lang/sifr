use super::cargo_manifest::generate_dependency_cargo_toml;
use super::project_codegen::GeneratedBinaryProject;
use super::{prepare_cached_artifact, CachedArtifactEntry, PreparedArtifactCache};
use crate::diagnostics::{CompileError, CompilePhase};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(super) fn materialize_binary_project(
    output_dir: &Path,
    project_name: &str,
    generated_project: GeneratedBinaryProject,
) -> Result<PathBuf, Vec<CompileError>> {
    let project_path = output_dir.join(project_name);
    materialize_binary_project_at_path(&project_path, project_name, generated_project)?;
    Ok(cached_binary_path(output_dir, project_name))
}

pub(super) fn materialize_cached_binary_project(
    cache_namespace: &str,
    cache_scope: &Path,
    project_name: &str,
    generated_project: GeneratedBinaryProject,
) -> Result<CachedArtifactEntry, Vec<CompileError>> {
    let cache_key = binary_project_cache_key(project_name, &generated_project);
    let required_paths = [
        Path::new(project_name).join("target"),
        binary_relative_path(project_name),
    ];
    let required_refs: Vec<&Path> = required_paths.iter().map(PathBuf::as_path).collect();
    let prepared =
        prepare_cached_artifact(cache_namespace, cache_scope, &cache_key, &required_refs)?;
    match prepared {
        PreparedArtifactCache::Hit(entry) => Ok(entry),
        PreparedArtifactCache::Miss(pending) => {
            let project_root = pending.workspace_root().join(project_name);
            materialize_binary_project_at_path(&project_root, project_name, generated_project)?;
            pending.commit(&required_refs)
        }
    }
}

pub(super) fn cached_binary_path(workspace_root: &Path, project_name: &str) -> PathBuf {
    workspace_root.join(binary_relative_path(project_name))
}

fn binary_relative_path(project_name: &str) -> PathBuf {
    let binary_name = if cfg!(target_os = "windows") {
        format!("{project_name}.exe")
    } else {
        project_name.to_string()
    };
    PathBuf::from(project_name)
        .join("target")
        .join("release")
        .join(binary_name)
}

fn materialize_binary_project_at_path(
    project_path: &Path,
    project_name: &str,
    generated_project: GeneratedBinaryProject,
) -> Result<(), Vec<CompileError>> {
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
    Ok(())
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

fn binary_project_cache_key(
    project_name: &str,
    generated_project: &GeneratedBinaryProject,
) -> String {
    let stdlib_modules = sorted_lines(&generated_project.used_stdlib_modules);
    let required_crates = sorted_lines(&generated_project.required_crates);
    let support_modules = generated_project
        .support_modules
        .iter()
        .map(|(name, code)| format!("{name}\n{code}"))
        .collect::<Vec<_>>()
        .join("\n===\n");
    format!(
        "project_name={project_name}\n[Cargo.toml]\n{}\n[main.rs]\n{}\n[support]\n{}\n[stdlib]\n{}\n[crates]\n{}",
        generate_dependency_cargo_toml(
            project_name,
            &generated_project.used_stdlib_modules,
            &generated_project.required_crates
        ),
        generated_project.main_rs,
        support_modules,
        stdlib_modules.join("\n"),
        required_crates.join("\n")
    )
}

fn sorted_lines(values: &std::collections::HashSet<String>) -> Vec<String> {
    let mut ordered: BTreeMap<&str, ()> = BTreeMap::new();
    for value in values {
        ordered.insert(value.as_str(), ());
    }
    ordered.into_keys().map(str::to_string).collect()
}
