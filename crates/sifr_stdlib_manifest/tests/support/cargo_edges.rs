use super::cargo_inventory::{self, REGISTRY};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Identity {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) source: Option<String>,
}

pub(crate) fn identity(package: &toml::Value) -> Result<Identity, String> {
    Ok(Identity {
        name: package
            .get("name")
            .and_then(toml::Value::as_str)
            .ok_or("missing package name")?
            .into(),
        version: package
            .get("version")
            .and_then(toml::Value::as_str)
            .ok_or("missing package version")?
            .into(),
        source: package
            .get("source")
            .map(|source| source.as_str().map(str::to_owned).ok_or("malformed source"))
            .transpose()?,
    })
}

pub(crate) fn packages(lock: &toml::Value) -> Result<&[toml::Value], String> {
    let packages = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .ok_or("lock package array missing")?;
    let mut seen = BTreeSet::new();
    for package in packages {
        let identity = identity(package)?;
        if !seen.insert(identity.clone()) {
            return Err(format!("duplicate lock identity: {identity:?}"));
        }
    }
    Ok(packages)
}

pub(crate) fn edges(package: &toml::Value) -> Result<Vec<&str>, String> {
    let Some(dependencies) = package.get("dependencies") else {
        return Ok(Vec::new());
    };
    let rows = dependencies
        .as_array()
        .ok_or("lock dependencies must be an array")?;
    let mut seen = BTreeSet::new();
    let mut result = Vec::new();
    for row in rows {
        let edge = row.as_str().ok_or("lock edge must be a string")?;
        if !seen.insert(edge) {
            return Err(format!("duplicate lock edge: {edge}"));
        }
        result.push(edge);
    }
    Ok(result)
}

pub(crate) fn resolve<'a>(
    packages: &'a [toml::Value],
    edge: &str,
) -> Result<&'a toml::Value, String> {
    let components = edge.split_whitespace().collect::<Vec<_>>();
    let (name, version, source) = match components.as_slice() {
        [name] => (*name, None, None),
        [name, version] => (*name, Some(*version), None),
        [name, version, source] if source.starts_with('(') && source.ends_with(')') => {
            (*name, Some(*version), Some(&source[1..source.len() - 1]))
        }
        _ => return Err(format!("malformed lock edge: {edge}")),
    };
    let mut matching = Vec::new();
    for package in packages {
        let id = identity(package)?;
        if id.name == name
            && version.is_none_or(|version| id.version == version)
            && source.is_none_or(|source| id.source.as_deref() == Some(source))
        {
            matching.push(package);
        }
    }
    match matching.as_slice() {
        [package] => Ok(package),
        _ => Err(format!(
            "missing or ambiguous lock identity: {edge} ({} matches)",
            matching.len()
        )),
    }
}

pub(crate) fn current_edge(
    packages: &[toml::Value],
    edge: &str,
    name: &str,
    version: &str,
) -> Result<(), String> {
    let actual = identity(resolve(packages, edge)?)?;
    let expected = Identity {
        name: name.into(),
        version: version.into(),
        source: Some(REGISTRY.into()),
    };
    if actual != expected {
        return Err(format!("{edge}: expected {expected:?}, got {actual:?}"));
    }
    Ok(())
}

pub(crate) fn first_party_edges(name: &str, version: &str) -> BTreeSet<(String, String)> {
    let root = cargo_inventory::root();
    let owners = cargo_inventory::local_owners();
    let mut actual = BTreeSet::new();
    for path in cargo_inventory::maintained_paths("Cargo.lock") {
        let lock = cargo_inventory::read_toml(&root.join(&path));
        let packages = packages(&lock).unwrap_or_else(|error| panic!("{path}: {error}"));
        for package in packages {
            let id = identity(package).unwrap_or_else(|error| panic!("{path}: {error}"));
            if id.source.is_some() {
                continue;
            }
            let Some(manifests) = owners.get(&(id.name.clone(), id.version)) else {
                continue;
            };
            let relevant = edges(package)
                .unwrap_or_else(|error| panic!("{path}: {error}"))
                .into_iter()
                .filter(|edge| edge.split_whitespace().next() == Some(name))
                .collect::<Vec<_>>();
            if relevant.is_empty() {
                continue;
            }
            assert_eq!(
                relevant.len(),
                1,
                "{path}: {} has multiple direct {name} edges",
                id.name
            );
            assert!(
                manifests
                    .iter()
                    .any(|manifest| cargo_inventory::owner_declares(manifest, name)),
                "{path}: {} has an undeclared registry {name} edge; check alias/source ownership",
                id.name
            );
            current_edge(packages, relevant[0], name, version)
                .unwrap_or_else(|error| panic!("{path}: {}: {error}", id.name));
            actual.insert((path.clone(), id.name));
        }
    }
    let inventory = cargo_inventory::inventory();
    let expected = inventory["edges"]
        .as_array()
        .unwrap_or_else(|| panic!("edge inventory array"))
        .iter()
        .filter(|row| row["package"].as_str() == Some(name))
        .map(|row| {
            let path = row["lock"].as_str().unwrap_or_else(|| panic!("edge lock"));
            let owner = row["owner"]
                .as_str()
                .unwrap_or_else(|| panic!("edge owner"));
            (path.to_owned(), owner.to_owned())
        })
        .collect::<BTreeSet<_>>();
    assert!(
        !expected.is_empty(),
        "missing expected {name} owner inventory"
    );
    assert_eq!(
        actual, expected,
        "{name}: exact maintained owner/lock edge inventory drift"
    );
    actual
}
