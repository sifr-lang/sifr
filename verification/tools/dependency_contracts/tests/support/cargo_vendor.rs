use super::cargo_edges::{Identity, identity};
use super::cargo_inventory::{REGISTRY, read_toml};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path};

pub(crate) fn selected_release(
    directory: &Path,
    name: &str,
    version: &str,
    hash: &str,
) -> Result<(), String> {
    let manifest = read_toml(&directory.join("Cargo.toml"));
    let package = manifest.get("package").ok_or("vendor package missing")?;
    if package.get("name").and_then(toml::Value::as_str) != Some(name)
        || package.get("version").and_then(toml::Value::as_str) != Some(version)
    {
        return Err(format!("{}: wrong vendor identity", directory.display()));
    }
    let source =
        fs::read(directory.join(".cargo-checksum.json")).map_err(|error| error.to_string())?;
    let checksum: serde_json::Value =
        serde_json::from_slice(&source).map_err(|error| error.to_string())?;
    if checksum["package"].as_str() != Some(hash) {
        return Err(format!("{name} {version}: package checksum drift"));
    }
    let files = checksum["files"]
        .as_object()
        .ok_or("vendor file checksums missing")?;
    if files.is_empty() {
        return Err("empty vendor file checksum inventory".into());
    }
    for (relative, expected) in files {
        if Path::new(relative)
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
        {
            return Err(format!("unsafe vendor checksum path: {relative}"));
        }
        let path = directory.join(relative);
        if fs::symlink_metadata(&path)
            .map_err(|error| error.to_string())?
            .file_type()
            .is_symlink()
        {
            return Err(format!("symlink vendor checksum input: {relative}"));
        }
        let actual =
            sifr_sysroot::sha256_file(&path).map_err(|error| format!("{relative}: {error}"))?;
        if expected.as_str() != Some(&actual) {
            return Err(format!(
                "{name} {version}: vendor file hash drift: {relative}"
            ));
        }
    }
    Ok(())
}

/// Certify one dependency family against its actual selected consumers. This
/// does not claim that the entire optional catalog belongs in the sysroot.
pub(crate) fn family(
    root: &Path,
    name: &str,
    selected: &[&str],
    packages: &[toml::Value],
) -> Result<(), String> {
    let expected = selected
        .iter()
        .map(|version| Identity {
            name: name.into(),
            version: (*version).into(),
            source: Some(REGISTRY.into()),
        })
        .collect::<BTreeSet<_>>();
    let mut actual = BTreeSet::new();
    for entry in fs::read_dir(root.join("vendor")).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        if !entry
            .file_type()
            .map_err(|error| error.to_string())?
            .is_dir()
        {
            continue;
        }
        let path = entry.path();
        if !path.join("Cargo.toml").is_file() {
            continue;
        }
        let manifest = read_toml(&path.join("Cargo.toml"));
        if manifest["package"]["name"].as_str() != Some(name) {
            continue;
        }
        let version = manifest["package"]["version"]
            .as_str()
            .ok_or("vendor version missing")?;
        let id = Identity {
            name: name.into(),
            version: version.into(),
            source: Some(REGISTRY.into()),
        };
        if !expected.contains(&id) || !actual.insert(id.clone()) {
            return Err(format!("stale or duplicate vendor identity: {id:?}"));
        }
        let matching = packages
            .iter()
            .filter(|package| identity(package).is_ok_and(|actual| actual == id))
            .collect::<Vec<_>>();
        let [package] = matching.as_slice() else {
            return Err(format!(
                "missing/duplicate selected vendor lock identity: {id:?}"
            ));
        };
        let hash = package
            .get("checksum")
            .and_then(toml::Value::as_str)
            .ok_or("lock checksum missing")?;
        selected_release(&path, name, version, hash)?;
    }
    if actual != expected {
        return Err(format!(
            "{name}: selected vendor identities are missing: {:?}",
            expected.difference(&actual)
        ));
    }
    Ok(())
}
