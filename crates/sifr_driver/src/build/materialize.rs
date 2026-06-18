use super::cargo_manifest::generate_dependency_cargo_toml;
use super::project_codegen::GeneratedBinaryProject;
use super::{prepare_cached_artifact, CachedArtifactEntry, PreparedArtifactCache};
use crate::diagnostics::RenderedDiagnostic;
use crate::project::{namespace_module_files, rust_module_file_path};
use sifr_diagnostics::DiagnosticCode;
use sifr_stdlib::StdlibFeature;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

pub(super) struct MaterializedBinaryProject {
    pub(super) binary_path: PathBuf,
    pub(super) materialize_elapsed: Duration,
    pub(super) cargo_elapsed: Duration,
}

pub(super) fn materialize_binary_project_with_report(
    output_dir: &Path,
    project_name: &str,
    generated_project: GeneratedBinaryProject,
) -> Result<MaterializedBinaryProject, Vec<RenderedDiagnostic>> {
    let project_path = output_dir.join(project_name);
    materialize_binary_project_at_path(&project_path, project_name, generated_project).map(
        |mut report| {
            report.binary_path = cached_binary_path(output_dir, project_name);
            report
        },
    )
}

pub(super) fn materialize_cached_binary_project_with_report(
    cache_namespace: &str,
    cache_scope: &Path,
    project_name: &str,
    generated_project: GeneratedBinaryProject,
) -> Result<(CachedArtifactEntry, Option<MaterializedBinaryProject>), Vec<RenderedDiagnostic>> {
    let cache_key = binary_project_cache_key(project_name, &generated_project);
    let required_paths = [
        Path::new(project_name).join("target"),
        binary_relative_path(project_name),
    ];
    let required_refs: Vec<&Path> = required_paths.iter().map(PathBuf::as_path).collect();
    let prepared =
        prepare_cached_artifact(cache_namespace, cache_scope, &cache_key, &required_refs)?;
    match prepared {
        PreparedArtifactCache::Hit(entry) => Ok((entry, None)),
        PreparedArtifactCache::Miss(pending) => {
            let project_root = pending.workspace_root().join(project_name);
            let report =
                materialize_binary_project_at_path(&project_root, project_name, generated_project)?;
            pending
                .commit(&required_refs)
                .map(|entry| (entry, Some(report)))
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
) -> Result<MaterializedBinaryProject, Vec<RenderedDiagnostic>> {
    let materialize_start = std::time::Instant::now();
    materialize_binary_project_files(project_path, project_name, generated_project)?;
    let materialize_elapsed = materialize_start.elapsed();

    let cargo_start = std::time::Instant::now();
    run_cargo_build(project_path)?;
    let cargo_elapsed = cargo_start.elapsed();

    Ok(MaterializedBinaryProject {
        binary_path: cached_binary_path(
            project_path.parent().unwrap_or(Path::new(".")),
            project_name,
        ),
        materialize_elapsed,
        cargo_elapsed,
    })
}

fn materialize_binary_project_files(
    project_path: &Path,
    project_name: &str,
    generated_project: GeneratedBinaryProject,
) -> Result<(), Vec<RenderedDiagnostic>> {
    let src_dir = project_path.join("src");
    std::fs::create_dir_all(&src_dir).map_err(|error| {
        vec![build_error(format!(
            "failed to create output directory: {error}"
        ))]
    })?;

    let cargo_toml = generate_dependency_cargo_toml(
        project_name,
        &generated_project.used_stdlib_modules,
        &generated_project.required_features,
    );

    write_project_file(&project_path.join("Cargo.toml"), cargo_toml, "Cargo.toml")?;

    write_project_file(
        &src_dir.join("main.rs"),
        generated_project.main_rs,
        "main.rs",
    )?;

    let mut support_modules = generated_project.support_modules;
    let support_module_names: Vec<String> = support_modules.keys().cloned().collect();
    let mut namespace_contents: BTreeMap<PathBuf, String> = BTreeMap::new();
    for namespace_file in namespace_module_files(&support_module_names) {
        let mut contents = String::new();
        for module_name in &namespace_file.declarations {
            contents.push_str("pub mod ");
            contents.push_str(module_name);
            contents.push_str(";\n");
        }
        namespace_contents.insert(namespace_file.path, contents);
    }

    for (module_name, code) in std::mem::take(&mut support_modules) {
        let namespace_path = namespace_module_file_path(&module_name);
        if let Some(contents) = namespace_contents.get_mut(&namespace_path) {
            if !contents.is_empty() && !contents.ends_with('\n') {
                contents.push('\n');
            }
            contents.push_str(&code);
            continue;
        }
        let file_name = rust_module_file_path(&module_name);
        write_project_file(
            &src_dir.join(&file_name),
            code,
            &file_name.display().to_string(),
        )?;
    }

    for (namespace_path, contents) in namespace_contents {
        write_project_file(
            &src_dir.join(&namespace_path),
            contents,
            &namespace_path.display().to_string(),
        )?;
    }

    Ok(())
}

fn run_cargo_build(project_path: &Path) -> Result<(), Vec<RenderedDiagnostic>> {
    let output = Command::new("cargo")
        .args(["build", "--release", "--quiet"])
        .current_dir(project_path)
        .output()
        .map_err(|error| {
            vec![cargo_build_error(format!(
                "failed to run cargo build: {error}"
            ))]
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(vec![cargo_build_error(format!(
            "cargo build failed:\n{stderr}"
        ))]);
    }
    Ok(())
}

fn namespace_module_file_path(module_name: &str) -> PathBuf {
    let mut path = PathBuf::new();
    for component in module_name.split('.') {
        path.push(component);
    }
    path.push("mod.rs");
    path
}

fn write_project_file(
    path: &Path,
    contents: impl AsRef<[u8]>,
    label: &str,
) -> Result<(), Vec<RenderedDiagnostic>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| vec![build_error(format!("failed to create {label}: {error}"))])?;
    }
    std::fs::write(path, contents)
        .map_err(|error| vec![build_error(format!("failed to write {label}: {error}"))])
}

fn build_error(message: String) -> RenderedDiagnostic {
    crate::diagnostics::diagnostic_with_code(message, DiagnosticCode::BUILD_MATERIALIZATION_FAILURE)
}

fn cargo_build_error(message: String) -> RenderedDiagnostic {
    crate::diagnostics::diagnostic_with_code(message, DiagnosticCode::BUILD_RUSTC_OR_CARGO_FAILURE)
}

fn binary_project_cache_key(
    project_name: &str,
    generated_project: &GeneratedBinaryProject,
) -> String {
    let stdlib_modules = sorted_lines(&generated_project.used_stdlib_modules);
    let required_features = sorted_feature_lines(&generated_project.required_features);
    let support_modules = generated_project
        .support_modules
        .iter()
        .map(|(name, code)| format!("{name}\n{code}"))
        .collect::<Vec<_>>()
        .join("\n===\n");
    format!(
        "project_name={project_name}\n[Cargo.toml]\n{}\n[main.rs]\n{}\n[support]\n{}\n[stdlib]\n{}\n[crates]\n{}\n[cache-key-fragment]\n{}",
        generate_dependency_cargo_toml(
            project_name,
            &generated_project.used_stdlib_modules,
            &generated_project.required_features
        ),
        generated_project.main_rs,
        support_modules,
        stdlib_modules.join("\n"),
        required_features.join("\n"),
        generated_project.cache_key_fragment.as_deref().unwrap_or("")
    )
}

fn sorted_lines(values: &std::collections::HashSet<String>) -> Vec<String> {
    let mut ordered: BTreeSet<&str> = BTreeSet::new();
    for value in values {
        ordered.insert(value.as_str());
    }
    ordered.into_iter().map(str::to_string).collect()
}

fn sorted_feature_lines(values: &std::collections::HashSet<StdlibFeature>) -> Vec<String> {
    let mut ordered: BTreeSet<&str> = BTreeSet::new();
    for value in values {
        ordered.insert(value.id());
    }
    ordered.into_iter().map(str::to_string).collect()
}

#[cfg(test)]
mod tests {
    use super::binary_project_cache_key;
    use crate::build::project_codegen::GeneratedBinaryProject;
    use std::collections::{BTreeMap, HashSet};

    #[test]
    fn binary_project_cache_key_includes_package_cache_fragment() {
        let base = GeneratedBinaryProject {
            main_rs: "fn main() {}\n".to_string(),
            support_modules: BTreeMap::new(),
            used_stdlib_modules: HashSet::new(),
            required_features: HashSet::new(),
            cache_key_fragment: None,
        };
        let mut with_python_probe = GeneratedBinaryProject {
            cache_key_fragment: Some("python-probe-a".to_string()),
            ..base
        };
        let first = binary_project_cache_key("sifr_output", &with_python_probe);
        with_python_probe.cache_key_fragment = Some("python-probe-b".to_string());
        let second = binary_project_cache_key("sifr_output", &with_python_probe);

        assert_ne!(first, second);
    }
}
