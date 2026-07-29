use std::collections::{BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

#[derive(Clone)]
struct LockedPackage {
    name: String,
    version: String,
    source: Option<String>,
}

/// Best-effort detail for a Cargo stale-lock failure.
///
/// Cargo intentionally collapses version, source, and feature changes into a
/// single `cannot update the lock file` message under `--frozen`. This helper
/// compares the checked-in lock with the package's reachable Cargo manifests
/// so Rust-interop diagnostics can retain a stable, useful reason without
/// retrying resolution or mutating the lock.
#[must_use]
pub fn package_lock_drift_reason(workspace_root: &Path) -> Option<&'static str> {
    let locked = read_locked_packages(&workspace_root.join("Cargo.lock"))?;
    let mut manifests = VecDeque::from([workspace_root.join("Cargo.toml")]);
    let mut visited = BTreeSet::new();
    let mut feature_selection_seen = false;
    let mut git_dependency_seen = false;

    while let Some(manifest_path) = manifests.pop_front() {
        let manifest_path = manifest_path.canonicalize().unwrap_or(manifest_path);
        if !visited.insert(manifest_path.clone()) {
            continue;
        }
        let Some(manifest_root) = manifest_path.parent() else {
            continue;
        };
        let Ok(source) = std::fs::read_to_string(&manifest_path) else {
            continue;
        };
        let Ok(manifest) = source.parse::<toml::Table>() else {
            continue;
        };
        collect_workspace_members(&manifest, manifest_root, &mut manifests);
        if let Some(reason) = inspect_dependency_tables(
            &toml::Value::Table(manifest),
            manifest_root,
            &locked,
            &mut manifests,
            &mut feature_selection_seen,
            &mut git_dependency_seen,
        ) {
            return Some(reason);
        }
    }

    if !git_dependency_seen
        && locked.iter().any(|package| {
            package
                .source
                .as_deref()
                .is_some_and(|source| !source.starts_with("registry+"))
        })
    {
        return Some("dependency source drift");
    }
    feature_selection_seen.then_some("requested feature selection drift")
}

fn read_locked_packages(path: &Path) -> Option<Vec<LockedPackage>> {
    let lock = std::fs::read_to_string(path)
        .ok()?
        .parse::<toml::Table>()
        .ok()?;
    Some(
        lock.get("package")
            .and_then(toml::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|package| {
                let package = package.as_table()?;
                Some(LockedPackage {
                    name: package.get("name")?.as_str()?.to_string(),
                    version: package.get("version")?.as_str()?.to_string(),
                    source: package
                        .get("source")
                        .and_then(toml::Value::as_str)
                        .map(str::to_string),
                })
            })
            .collect(),
    )
}

fn collect_workspace_members(
    manifest: &toml::Table,
    manifest_root: &Path,
    manifests: &mut VecDeque<PathBuf>,
) {
    let members = manifest
        .get("workspace")
        .and_then(toml::Value::as_table)
        .and_then(|workspace| workspace.get("members"))
        .and_then(toml::Value::as_array);
    for member in members
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_str)
    {
        if !member.contains(['*', '?', '[']) {
            manifests.push_back(manifest_root.join(member).join("Cargo.toml"));
        }
    }
}

fn inspect_dependency_tables(
    value: &toml::Value,
    manifest_root: &Path,
    locked: &[LockedPackage],
    manifests: &mut VecDeque<PathBuf>,
    feature_selection_seen: &mut bool,
    git_dependency_seen: &mut bool,
) -> Option<&'static str> {
    match value {
        toml::Value::Table(table) => {
            for (key, nested) in table {
                if matches!(
                    key.as_str(),
                    "dependencies" | "dev-dependencies" | "build-dependencies"
                ) {
                    if let Some(reason) = nested.as_table().and_then(|dependencies| {
                        inspect_dependencies(
                            dependencies,
                            manifest_root,
                            locked,
                            manifests,
                            feature_selection_seen,
                            git_dependency_seen,
                        )
                    }) {
                        return Some(reason);
                    }
                } else if let Some(reason) = inspect_dependency_tables(
                    nested,
                    manifest_root,
                    locked,
                    manifests,
                    feature_selection_seen,
                    git_dependency_seen,
                ) {
                    return Some(reason);
                }
            }
        }
        toml::Value::Array(values) => {
            for nested in values {
                if let Some(reason) = inspect_dependency_tables(
                    nested,
                    manifest_root,
                    locked,
                    manifests,
                    feature_selection_seen,
                    git_dependency_seen,
                ) {
                    return Some(reason);
                }
            }
        }
        _ => {}
    }
    None
}

fn inspect_dependencies(
    dependencies: &toml::Table,
    manifest_root: &Path,
    locked: &[LockedPackage],
    manifests: &mut VecDeque<PathBuf>,
    feature_selection_seen: &mut bool,
    git_dependency_seen: &mut bool,
) -> Option<&'static str> {
    for (alias, dependency) in dependencies {
        let Some(specification) = dependency.as_table() else {
            continue;
        };
        if let Some(path) = specification.get("path").and_then(toml::Value::as_str) {
            manifests.push_back(manifest_root.join(path).join("Cargo.toml"));
            continue;
        }
        if specification.contains_key("git") {
            *git_dependency_seen = true;
            continue;
        }
        let package_name = specification
            .get("package")
            .and_then(toml::Value::as_str)
            .unwrap_or(alias);
        let candidates = locked
            .iter()
            .filter(|package| package.name == package_name)
            .collect::<Vec<_>>();
        if let Some(expected) = specification
            .get("version")
            .and_then(toml::Value::as_str)
            .and_then(exact_version)
        {
            if !candidates.is_empty()
                && !candidates.iter().any(|package| package.version == expected)
            {
                return Some("stale selected version");
            }
        }
        if !candidates.is_empty()
            && candidates.iter().all(|package| {
                package
                    .source
                    .as_deref()
                    .is_some_and(|source| !source.starts_with("registry+"))
            })
        {
            return Some("dependency source drift");
        }
        *feature_selection_seen |= specification
            .get("features")
            .and_then(toml::Value::as_array)
            .is_some_and(|features| !features.is_empty());
    }
    None
}

fn exact_version(requirement: &str) -> Option<&str> {
    requirement.strip_prefix('=').map(str::trim)
}
