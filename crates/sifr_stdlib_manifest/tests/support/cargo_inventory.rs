//! Root-tracked Cargo inputs. Excluded repositories retain their own ownership.
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

pub(crate) const INVENTORY: &str = include_str!("maintained_cargo_inventory.json");
pub(crate) const REGISTRY: &str = "registry+https://github.com/rust-lang/crates.io-index";

pub(crate) fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap_or_else(|error| panic!("workspace root: {error}"))
}

pub(crate) fn read_toml(path: &Path) -> toml::Value {
    let source =
        fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    toml::from_str(&source).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

pub(crate) fn inventory() -> serde_json::Value {
    serde_json::from_str(INVENTORY).unwrap_or_else(|error| panic!("Cargo inventory: {error}"))
}

pub(crate) fn select_tracked_paths(
    listing: &[u8],
    filename: &str,
) -> Result<BTreeSet<String>, String> {
    let mut paths = BTreeSet::new();
    for record in listing
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
    {
        let entry = std::str::from_utf8(record).map_err(|error| error.to_string())?;
        let (metadata, path) = entry.split_once('\t').ok_or("malformed Git index entry")?;
        let fields = metadata.split_whitespace().collect::<Vec<_>>();
        if fields.len() != 3 || fields[2] != "0" {
            return Err(format!("unmerged/malformed entry: {entry}"));
        }
        let path_value = Path::new(path);
        if path_value
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
        {
            return Err(format!("non-relative tracked path: {path}"));
        }
        if fields[0] == "160000" || path.starts_with("vendor/") || path.starts_with("third_party/")
        {
            continue;
        }
        if path_value.file_name().and_then(|name| name.to_str()) != Some(filename) {
            continue;
        }
        if !matches!(fields[0], "100644" | "100755") {
            return Err(format!("Cargo input is not a regular tracked file: {path}"));
        }
        if !paths.insert(path.to_owned()) {
            return Err(format!("duplicate tracked input: {path}"));
        }
    }
    Ok(paths)
}

pub(crate) fn check_inventory(
    actual: &BTreeSet<String>,
    expected: &serde_json::Value,
) -> Result<(), String> {
    let rows = expected.as_array().ok_or("inventory must be an array")?;
    let mut paths = BTreeSet::new();
    for row in rows {
        let path = row.as_str().ok_or("inventory path must be a string")?;
        if !paths.insert(path.to_owned()) {
            return Err(format!("duplicate inventory path: {path}"));
        }
    }
    if actual != &paths {
        return Err(format!(
            "Cargo inventory drift: missing {:?}; extra {:?}",
            paths.difference(actual),
            actual.difference(&paths)
        ));
    }
    Ok(())
}

pub(crate) fn maintained_paths(filename: &str) -> Vec<String> {
    let root = root();
    let paths = tracked_paths(&root, filename).unwrap_or_else(|error| panic!("{error}"));
    let key = match filename {
        "Cargo.toml" => "manifests",
        "Cargo.lock" => "locks",
        _ => panic!("unsupported Cargo input"),
    };
    check_inventory(&paths, &inventory()[key]).unwrap_or_else(|error| panic!("{error}"));
    for path in &paths {
        assert!(
            root.join(path).is_file(),
            "missing tracked Cargo input: {path}"
        );
    }
    paths.into_iter().collect()
}

pub(crate) fn tracked_paths(root: &Path, filename: &str) -> Result<BTreeSet<String>, String> {
    let output = Command::new("git")
        .args(["ls-files", "--stage", "-z"])
        .current_dir(root)
        .output()
        .map_err(|error| format!("tracked Cargo discovery: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "git ls-files: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    select_tracked_paths(&output.stdout, filename)
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Declaration {
    pub(crate) path: String,
    pub(crate) table: String,
    pub(crate) alias: String,
    pub(crate) package: String,
    pub(crate) requirement: String,
}

pub(crate) fn registry_requirement(
    alias: &str,
    specification: &toml::Value,
) -> Result<Option<(String, String)>, String> {
    if let Some(requirement) = specification.as_str() {
        return Ok(Some((alias.into(), requirement.into())));
    }
    let table = specification
        .as_table()
        .ok_or("dependency specification must be a string or table")?;
    if table.get("workspace").and_then(toml::Value::as_bool) == Some(true)
        || table.contains_key("path")
        || table.contains_key("git")
    {
        return Ok(None);
    }
    if table.contains_key("registry") || table.contains_key("registry-index") {
        return Err(format!(
            "{alias}: a non-crates.io registry requires its own audit"
        ));
    }
    let requirement = table
        .get("version")
        .and_then(toml::Value::as_str)
        .ok_or("registry dependency version missing")?;
    let name = match table.get("package") {
        Some(value) => value.as_str().ok_or("package alias must be a string")?,
        None => alias,
    };
    Ok(Some((name.into(), requirement.into())))
}

pub(crate) fn dependency_tables(manifest: &toml::Value) -> Vec<(String, &toml::value::Table)> {
    let mut owners = vec![(String::new(), manifest)];
    if let Some(workspace) = manifest.get("workspace") {
        owners.push(("workspace.".into(), workspace));
    }
    if let Some(targets) = manifest.get("target").and_then(toml::Value::as_table) {
        for (target, table) in targets {
            owners.push((format!("target.{target}."), table));
        }
    }
    let mut result = Vec::new();
    for (prefix, owner) in owners {
        for kind in ["dependencies", "dev-dependencies", "build-dependencies"] {
            if let Some(value) = owner.get(kind) {
                let table = value
                    .as_table()
                    .unwrap_or_else(|| panic!("{prefix}{kind} is not a table"));
                result.push((format!("{prefix}{kind}"), table));
            }
        }
    }
    result
}

pub(crate) fn declarations() -> Vec<Declaration> {
    let root = root();
    let mut result = Vec::new();
    for path in maintained_paths("Cargo.toml") {
        let manifest = read_toml(&root.join(&path));
        for (table, dependencies) in dependency_tables(&manifest) {
            for (alias, specification) in dependencies {
                if let Some((package, requirement)) = registry_requirement(alias, specification)
                    .unwrap_or_else(|error| panic!("{path} {table} {alias}: {error}"))
                {
                    result.push(Declaration {
                        path: path.clone(),
                        table: table.clone(),
                        alias: alias.clone(),
                        package,
                        requirement,
                    });
                }
            }
        }
    }
    result.sort();
    result
}

pub(crate) fn local_owners() -> BTreeMap<(String, String), Vec<String>> {
    let root = root();
    let mut owners: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
    for path in maintained_paths("Cargo.toml") {
        let manifest = read_toml(&root.join(&path));
        let Some(package) = manifest.get("package") else {
            continue;
        };
        let name = package
            .get("name")
            .and_then(toml::Value::as_str)
            .unwrap_or_else(|| panic!("{path}: package name"));
        let version = package
            .get("version")
            .and_then(toml::Value::as_str)
            .unwrap_or_else(|| panic!("{path}: explicit package version"));
        owners
            .entry((name.into(), version.into()))
            .or_default()
            .push(path);
    }
    owners
}

/// Resolve workspace aliases only at the declaring workspace boundary.
pub(crate) fn owner_declares(path: &str, dependency: &str) -> bool {
    let root = root();
    let manifest_path = root.join(path);
    let manifest = read_toml(&manifest_path);
    let mut directory = manifest_path.parent();
    let mut workspace = None;
    while let Some(parent) = directory {
        let candidate = parent.join("Cargo.toml");
        if candidate.is_file() {
            let parsed = read_toml(&candidate);
            if parsed.get("workspace").is_some() {
                workspace = Some(parsed);
                break;
            }
        }
        if parent == root {
            break;
        }
        directory = parent.parent();
    }
    dependency_tables(&manifest)
        .into_iter()
        .filter(|(table, _)| !table.starts_with("workspace."))
        .any(|(_, dependencies)| {
            dependencies.iter().any(|(alias, spec)| {
                let inherited = spec.get("workspace").and_then(toml::Value::as_bool) == Some(true);
                let specification = if inherited {
                    workspace
                        .as_ref()
                        .and_then(|value| value.get("workspace"))
                        .and_then(|value| value.get("dependencies"))
                        .and_then(|value| value.get(alias))
                        .unwrap_or_else(|| panic!("{path}: missing workspace dependency {alias}"))
                } else {
                    spec
                };
                registry_requirement(alias, specification)
                    .unwrap_or_else(|error| panic!("{path}: {error}"))
                    .is_some_and(|(name, _)| name == dependency)
            })
        })
}
