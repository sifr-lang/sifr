use crate::diag::PackageDiagnostic;
use crate::graph::derive::{SifrPackageGraph, SifrPackageId, SifrPackageMetadata};
use sifr_diagnostics::DiagnosticCode;
use std::path::{Path, PathBuf};

/// Root-owned paths selected for a package Python environment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PythonEnvironmentSelection {
    pub venv_root: PathBuf,
    pub interpreter: PathBuf,
    pub pyproject: Option<PathBuf>,
    pub lock: Option<PathBuf>,
}

/// Select canonical environment paths from the root manifest and uv project
/// discovery without probing or mutating the environment.
#[must_use]
pub fn select_root_python_environment(
    package_root: &Path,
    config: &crate::manifest::sifr::PythonConfig,
) -> Option<PythonEnvironmentSelection> {
    let project_root = configured_or_discovered_project_root(package_root, config)?;
    let venv_root = config.venv.as_ref().map_or_else(
        || project_root.join(".venv"),
        |path| absolutize(package_root, path),
    );
    Some(PythonEnvironmentSelection {
        interpreter: config.interpreter.as_ref().map_or_else(
            || default_interpreter(&venv_root),
            |path| absolutize(package_root, path),
        ),
        pyproject: Some(config.pyproject.as_ref().map_or_else(
            || project_root.join("pyproject.toml"),
            |path| absolutize(package_root, path),
        )),
        lock: Some(config.lock.as_ref().map_or_else(
            || project_root.join("uv.lock"),
            |path| absolutize(package_root, path),
        )),
        venv_root,
    })
}

pub(super) fn root_environment_selection(
    _root_package_id: &SifrPackageId,
    package: &SifrPackageMetadata,
) -> Result<PythonEnvironmentSelection, Vec<PackageDiagnostic>> {
    let selection = select_root_python_environment(
        &package.package_root,
        &package.manifest.python,
    )
    .ok_or_else(|| {
            vec![PackageDiagnostic::python_environment_graph(
                DiagnosticCode::PYENV_MISSING_SELECTION,
                format!(
                    "Python is required but no uv project was discovered from '{}'",
                    package.package_root.display()
                ),
                Some(package.cargo_package_id.clone()),
                "create pyproject.toml, uv.lock, and .venv in the project ancestry or configure root-owned [python] path overrides",
            )]
        })?;
    Ok(selection)
}

pub(super) fn non_root_environment_configuration(
    graph: &SifrPackageGraph,
    root_package_id: &SifrPackageId,
) -> Vec<PackageDiagnostic> {
    graph
        .packages
        .values()
        .filter(|package| &package.package_id != root_package_id)
        .filter_map(|package| {
            let config = &package.manifest.python;
            let key = if config.venv.is_some() {
                "python.venv"
            } else if config.interpreter.is_some() {
                "python.interpreter"
            } else if config.pyproject.is_some() {
                "python.pyproject"
            } else if config.lock.is_some() {
                "python.lock"
            } else {
                return None;
            };
            Some(PackageDiagnostic::python_environment_config(
                &package.cargo_package_id,
                &package.sifr_manifest,
                key,
                "only the root application package may select or override the Python environment",
            ))
        })
        .collect()
}

pub(super) fn selects_environment(config: &crate::manifest::sifr::PythonConfig) -> bool {
    config.selects_environment()
}

fn configured_or_discovered_project_root(
    package_root: &Path,
    config: &crate::manifest::sifr::PythonConfig,
) -> Option<PathBuf> {
    config
        .pyproject
        .as_ref()
        .and_then(|path| {
            absolutize(package_root, path)
                .parent()
                .map(Path::to_path_buf)
        })
        .or_else(|| {
            config.lock.as_ref().and_then(|path| {
                absolutize(package_root, path)
                    .parent()
                    .map(Path::to_path_buf)
            })
        })
        .or_else(|| discover_project_root(package_root))
        .or_else(|| selects_environment(config).then(|| package_root.to_path_buf()))
}

fn discover_project_root(package_root: &Path) -> Option<PathBuf> {
    package_root
        .ancestors()
        .find(|root| root.join("pyproject.toml").is_file() && root.join("uv.lock").is_file())
        .map(Path::to_path_buf)
}

pub(super) fn absolutize(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn default_interpreter(venv_root: &Path) -> PathBuf {
    if cfg!(windows) {
        venv_root.join("Scripts").join("python.exe")
    } else {
        venv_root.join("bin").join("python")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TestUnwrap as _;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn uv_defaults_discover_workspace_project_from_nested_package() {
        let root = temp_root("workspace-discovery");
        let package = root.join("packages/app");
        fs::create_dir_all(&package).test_unwrap("create nested package");
        fs::write(root.join("pyproject.toml"), "[project]\n").test_unwrap("write pyproject");
        fs::write(root.join("uv.lock"), "version = 1\n").test_unwrap("write lock marker");

        assert_eq!(discover_project_root(&package), Some(root.clone()));
        fs::remove_dir_all(&root).test_unwrap("remove workspace fixture");
    }

    fn temp_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .test_unwrap("clock after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "sifr-python-{label}-{}-{nonce}",
            std::process::id()
        ))
    }
}
