use crate::CargoPackageId;
use crate::diag::PackageDiagnostic;
use crate::manifest::sifr::PackageSourceRoot;
use sha2::{Digest as _, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub(super) fn misplaced_root_diagnostics(
    package_root: &Path,
    cargo_package_id: &CargoPackageId,
    source_root: &PackageSourceRoot,
    canonical_root: &Path,
) -> Vec<PackageDiagnostic> {
    let mut candidates = BTreeSet::from([package_root.join("python_bridges")]);
    candidates.insert(package_root.join(&source_root.0).join("python_bridges"));
    candidates
        .into_iter()
        .filter(|candidate| candidate != canonical_root && candidate.exists())
        .map(|candidate| {
            PackageDiagnostic::invalid_python_bridge_source(
                cargo_package_id,
                &candidate,
                format!(
                    "bridge source root must be '{}', not '{}'",
                    canonical_root.display(),
                    candidate.display()
                ),
            )
        })
        .collect()
}

pub(super) fn discover_source_paths(
    package_root: &Path,
    cargo_package_id: &CargoPackageId,
    root: &Path,
    diagnostics: &mut Vec<PackageDiagnostic>,
) -> BTreeSet<PathBuf> {
    let mut paths = BTreeSet::new();
    if let Some(symlink) = symlinked_bridge_component(package_root, root) {
        diagnostics.push(PackageDiagnostic::invalid_python_bridge_source(
            cargo_package_id,
            &symlink,
            "symbolic links are not allowed in bridge source roots or their package-relative ancestors",
        ));
        return paths;
    }
    match fs::symlink_metadata(root) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            diagnostics.push(PackageDiagnostic::invalid_python_bridge_source(
                cargo_package_id,
                root,
                "symbolic links are not allowed in bridge source roots",
            ));
            return paths;
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return paths,
        Err(error) => {
            diagnostics.push(PackageDiagnostic::invalid_python_bridge_source(
                cargo_package_id,
                root,
                format!("could not inspect bridge source root: {error}"),
            ));
            return paths;
        }
    }
    collect_python_paths(root, &mut paths, diagnostics, Some(cargo_package_id));
    paths
}

fn symlinked_bridge_component(package_root: &Path, root: &Path) -> Option<PathBuf> {
    let relative = root.strip_prefix(package_root).ok()?;
    let mut current = package_root.to_path_buf();
    for component in relative.components() {
        current.push(component.as_os_str());
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => return Some(current),
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return None,
            Err(_) => return None,
        }
    }
    None
}

pub(super) fn collect_python_paths(
    directory: &Path,
    paths: &mut BTreeSet<PathBuf>,
    diagnostics: &mut Vec<PackageDiagnostic>,
    cargo_package_id: Option<&CargoPackageId>,
) {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
        Err(error) => {
            if let Some(cargo_package_id) = cargo_package_id {
                diagnostics.push(PackageDiagnostic::invalid_python_bridge_source(
                    cargo_package_id,
                    directory,
                    format!("could not read bridge directory: {error}"),
                ));
            }
            return;
        }
    };
    let mut valid_entries = Vec::new();
    for entry in entries {
        match entry {
            Ok(entry) => valid_entries.push(entry),
            Err(error) => {
                if let Some(cargo_package_id) = cargo_package_id {
                    diagnostics.push(PackageDiagnostic::invalid_python_bridge_source(
                        cargo_package_id,
                        directory,
                        format!("could not read bridge directory entry: {error}"),
                    ));
                }
            }
        }
    }
    valid_entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in valid_entries {
        let path = entry.path();
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(error) => {
                if let Some(cargo_package_id) = cargo_package_id {
                    diagnostics.push(PackageDiagnostic::invalid_python_bridge_source(
                        cargo_package_id,
                        &path,
                        format!("could not inspect bridge source: {error}"),
                    ));
                }
                continue;
            }
        };
        if file_type.is_symlink() {
            if let Some(cargo_package_id) = cargo_package_id {
                diagnostics.push(PackageDiagnostic::invalid_python_bridge_source(
                    cargo_package_id,
                    &path,
                    "symbolic links are not allowed in bridge source roots",
                ));
            }
        } else if file_type.is_dir() {
            collect_python_paths(&path, paths, diagnostics, cargo_package_id);
        } else if path.extension().is_some_and(|extension| extension == "py") {
            paths.insert(path);
        }
    }
}

pub(super) enum ModuleNameError {
    RootPackageReserved,
    InvalidPath,
}

impl ModuleNameError {
    pub(super) const fn reason(&self) -> &'static str {
        match self {
            Self::RootPackageReserved => {
                "root __init__.py is reserved because the runtime creates the package entry"
            }
            Self::InvalidPath => "module path components must be valid Python identifiers",
        }
    }
}

pub(super) fn module_name(
    root: &Path,
    source_path: &Path,
) -> Result<(String, bool), ModuleNameError> {
    let relative = source_path
        .strip_prefix(root)
        .map_err(|_| ModuleNameError::InvalidPath)?;
    if relative
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(ModuleNameError::InvalidPath);
    }
    let is_package = relative
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(ModuleNameError::InvalidPath)?
        == "__init__.py";
    let mut parts = relative
        .parent()
        .into_iter()
        .flat_map(Path::components)
        .filter_map(|component| match component {
            Component::Normal(value) => value.to_str().map(str::to_string),
            _ => None,
        })
        .collect::<Vec<_>>();
    if !is_package {
        parts.push(
            relative
                .file_stem()
                .and_then(|stem| stem.to_str())
                .ok_or(ModuleNameError::InvalidPath)?
                .to_string(),
        );
    }
    if parts.is_empty() {
        return Err(ModuleNameError::RootPackageReserved);
    }
    let module = parts.join(".");
    sifr_syntax::parse_module_suite(&format!("import {module}\n"), None)
        .map(|_| (module, is_package))
        .map_err(|_| ModuleNameError::InvalidPath)
}

pub(super) fn path_string(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(value) => value.to_str(),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

pub(super) fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
