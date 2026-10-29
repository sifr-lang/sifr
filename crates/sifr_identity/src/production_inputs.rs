//! Manifest-owned production roots, independent of private test layout.
use super::input_tree::InputInventory;
use std::{
    collections::BTreeSet,
    fs, io,
    path::{Component, Path, PathBuf},
};

fn manifest(path: &Path) -> io::Result<toml::Value> {
    toml::from_str(&fs::read_to_string(path)?).map_err(io::Error::other)
}

fn relative(value: &str) -> io::Result<&Path> {
    let path = Path::new(value);
    if value.is_empty()
        || !path
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
    {
        return Err(io::Error::other(
            "production input must be a nonempty relative path",
        ));
    }
    Ok(path)
}

pub(super) fn visit_package(
    root: &Path,
    package: &Path,
    inputs: &mut InputInventory,
) -> io::Result<()> {
    package_inputs(root, package, inputs, &mut BTreeSet::new())
}

fn package_inputs(
    root: &Path,
    package: &Path,
    inventory: &mut InputInventory,
    active: &mut BTreeSet<PathBuf>,
) -> io::Result<()> {
    if !active.insert(package.to_path_buf()) {
        return Err(io::Error::other("cyclic production package inputs"));
    }
    let manifest_path = package.join("Cargo.toml");
    let data = manifest(&manifest_path)?;
    let ownership = data
        .get("package")
        .and_then(|v| v.get("metadata"))
        .and_then(|v| v.get("sifr-build"))
        .ok_or_else(|| io::Error::other("missing production input ownership"))?;
    let inputs = ownership
        .get("production-inputs")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| {
            io::Error::other(format!(
                "missing production-inputs in {}",
                manifest_path.display()
            ))
        })?;
    let mut paths = Vec::new();
    for input in inputs {
        let text = input
            .as_str()
            .ok_or_else(|| io::Error::other("production input must be a string"))?;
        let path = relative(text)?;
        if paths
            .iter()
            .any(|existing: &&Path| existing.starts_with(path) || path.starts_with(existing))
            || path == Path::new("Cargo.toml")
        {
            return Err(io::Error::other(
                "duplicate or overlapping production input",
            ));
        }
        if !package.join(path).exists() {
            return Err(io::Error::other(format!(
                "missing production input {}",
                package.join(path).display()
            )));
        }
        paths.push(path);
    }
    if !paths.contains(&Path::new("src"))
        || (package.join("build.rs").exists() && !paths.contains(&Path::new("build.rs")))
    {
        return Err(io::Error::other(
            "production inputs must own src and any build.rs",
        ));
    }
    paths.sort();
    inventory.add(&manifest_path);
    for path in paths {
        inventory.add(&package.join(path));
    }
    if let Some(inputs) = ownership.get("production-workspace-inputs") {
        let inputs = inputs
            .as_array()
            .ok_or_else(|| io::Error::other("workspace production inputs must be an array"))?;
        let mut paths = BTreeSet::new();
        for input in inputs {
            let path =
                relative(input.as_str().ok_or_else(|| {
                    io::Error::other("workspace production input must be a string")
                })?)?;
            if !paths.insert(path) || !root.join(path).exists() {
                return Err(io::Error::other(
                    "duplicate or missing workspace production input",
                ));
            }
        }
        for path in paths {
            if path.starts_with("crates") && path.components().count() == 2 {
                package_inputs(root, &root.join(path), inventory, active)?;
            } else {
                inventory.add(&root.join(path));
            }
        }
    }
    active.remove(package);
    Ok(())
}

pub(super) fn workspace_packages(root: &Path) -> io::Result<Vec<PathBuf>> {
    let data = manifest(&root.join("Cargo.toml"))?;
    let members = data
        .get("workspace")
        .and_then(|v| v.get("members"))
        .and_then(toml::Value::as_array)
        .ok_or_else(|| io::Error::other("workspace must declare its production packages"))?;
    let mut result = Vec::new();
    for member in members {
        let value = member
            .as_str()
            .ok_or_else(|| io::Error::other("workspace member must be a string"))?;
        let path = relative(value)?;
        if path.starts_with("crates") {
            if path.components().count() != 2 || value.contains(['*', '?', '[', ']']) {
                return Err(io::Error::other(
                    "production workspace members must name packages explicitly",
                ));
            }
            result.push(root.join(path));
        }
    }
    result.sort();
    result.dedup();
    Ok(result)
}
