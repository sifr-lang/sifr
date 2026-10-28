use super::cargo_inventory::{Declaration, REGISTRY};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn stable_version(version: &str) -> bool {
    let (core, build) = version
        .split_once('+')
        .map_or((version, None), |(core, build)| (core, Some(build)));
    let parts = core.split('.').collect::<Vec<_>>();
    parts.len() == 3
        && parts.iter().all(|part| {
            !part.is_empty()
                && part.bytes().all(|byte| byte.is_ascii_digit())
                && (part.len() == 1 || !part.starts_with('0'))
        })
        && build.is_none_or(|build| {
            !build.is_empty()
                && build.split('.').all(|part| {
                    !part.is_empty()
                        && part
                            .bytes()
                            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
                })
        })
}

pub(crate) fn checksum(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(crate) fn releases(
    audit: &serde_json::Value,
) -> Result<BTreeMap<String, &serde_json::Value>, String> {
    let rows = audit
        .get("packages")
        .and_then(serde_json::Value::as_array)
        .ok_or("audit package array")?;
    let mut result = BTreeMap::new();
    for row in rows {
        let name = row
            .get("name")
            .and_then(serde_json::Value::as_str)
            .ok_or("audit name")?;
        let version = row
            .get("latest_stable")
            .and_then(serde_json::Value::as_str)
            .ok_or("audit version")?;
        let hash = row
            .get("checksum")
            .and_then(serde_json::Value::as_str)
            .ok_or("audit checksum")?;
        if name.is_empty() || !stable_version(version) || !checksum(hash) {
            return Err(format!("invalid release: {name} {version}"));
        }
        if result.insert(name.into(), row).is_some() {
            return Err(format!("duplicate audit package: {name}"));
        }
    }
    Ok(result)
}

pub(crate) fn exact_packages(
    audit: &serde_json::Value,
    declarations: &[Declaration],
) -> Result<(), String> {
    let releases = releases(audit)?;
    let actual = declarations
        .iter()
        .map(|row| row.package.clone())
        .collect::<BTreeSet<_>>();
    let expected = releases.keys().cloned().collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(format!(
            "audit package set: missing {:?}; extra {:?}",
            actual.difference(&expected),
            expected.difference(&actual)
        ));
    }
    Ok(())
}

pub(crate) fn requirement_matches_latest(requirement: &str, latest: &str) -> bool {
    if !stable_version(latest) {
        return false;
    }
    let requirement = requirement
        .strip_prefix(['=', '^', '~'])
        .unwrap_or(requirement);
    let (required_core, required_build) = requirement
        .split_once('+')
        .map_or((requirement, None), |(core, build)| (core, Some(build)));
    let (latest_core, latest_build) = latest
        .split_once('+')
        .map_or((latest, None), |(core, build)| (core, Some(build)));
    let required = required_core.split('.').collect::<Vec<_>>();
    let latest = latest_core.split('.').collect::<Vec<_>>();
    !required.is_empty()
        && required.len() <= latest.len()
        && required.iter().zip(&latest).all(|(a, b)| a == b)
        && latest[required.len()..].iter().all(|part| *part == "0")
        && required_build.is_none_or(|build| Some(build) == latest_build)
}

pub(crate) fn lock_checksums(
    audit: &serde_json::Value,
    packages: &[toml::Value],
) -> Result<(), String> {
    let releases = releases(audit)?;
    for package in packages {
        let id = super::cargo_edges::identity(package)?;
        let Some(release) = releases.get(&id.name) else {
            continue;
        };
        if id.source.is_none() {
            continue;
        } // A local probe may deliberately share a registry name.
        if id.source.as_deref() != Some(REGISTRY) {
            return Err(format!(
                "{}: unexpected locked source {:?}",
                id.name, id.source
            ));
        }
        let rows = release["locked_releases"]
            .as_array()
            .ok_or("audited locked releases missing")?;
        let matching = rows
            .iter()
            .filter(|row| row["version"].as_str() == Some(&id.version))
            .collect::<Vec<_>>();
        let [row] = matching.as_slice() else {
            return Err(format!(
                "{} {}: missing/duplicate official checksum row",
                id.name, id.version
            ));
        };
        let expected = row["checksum"]
            .as_str()
            .filter(|hash| checksum(hash))
            .ok_or("invalid official checksum")?;
        if package.get("checksum").and_then(toml::Value::as_str) != Some(expected) {
            return Err(format!(
                "{} {}: locked checksum differs from official source",
                id.name, id.version
            ));
        }
    }
    Ok(())
}
