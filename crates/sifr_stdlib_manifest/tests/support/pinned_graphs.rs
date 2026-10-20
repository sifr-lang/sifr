//! Explicit exceptions to release uniformity, authenticated before classification.
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};
use std::sync::OnceLock;

pub(crate) const POLICY: &str = include_str!("pinned_verification_graphs.json");
pub(crate) const ROOTS: [&str; 2] = [
    "verification/fuzz",
    "verification/tools/maintainability_builtin_input",
];

fn in_scope(path: &str) -> bool {
    let path_value = Path::new(path);
    path_value
        .components()
        .all(|part| matches!(part, Component::Normal(_)))
        && matches!(
            path_value.file_name().and_then(|part| part.to_str()),
            Some("Cargo.toml" | "Cargo.lock")
        )
        && ROOTS
            .iter()
            .any(|root| path.starts_with(&format!("{root}/")))
}

pub(crate) fn inputs() -> BTreeMap<String, Vec<u8>> {
    let root = super::root();
    ["Cargo.toml", "Cargo.lock"]
        .into_iter()
        .flat_map(super::maintained_paths)
        .filter(|path| in_scope(path))
        .map(|path| {
            let absolute = root.join(&path);
            assert!(
                std::fs::symlink_metadata(&absolute)
                    .is_ok_and(|metadata| metadata.file_type().is_file()),
                "pinned input must be a regular file: {path}"
            );
            let source = std::fs::read(absolute).unwrap_or_else(|error| panic!("{path}: {error}"));
            (path, source)
        })
        .collect()
}

fn hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn index_url(name: &str) -> Result<String, String> {
    if name.is_empty()
        || !name.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'_'
        })
    {
        return Err("invalid registry name".into());
    }
    let prefix = match name.len() {
        1 => "1".to_owned(),
        2 => "2".to_owned(),
        3 => format!("3/{}", &name[..1]),
        _ => format!("{}/{}", &name[..2], &name[2..4]),
    };
    Ok(format!("https://index.crates.io/{prefix}/{name}"))
}

pub(crate) fn validate(
    policy: &serde_json::Value,
    inputs: &BTreeMap<String, Vec<u8>>,
) -> Result<BTreeSet<String>, String> {
    if policy["schema_version"].as_u64() != Some(1)
        || policy["roots"] != serde_json::json!(ROOTS)
        || policy["approved_at"].as_str() != Some("2026-10-20")
    {
        return Err("unapproved pinned graph policy".into());
    }
    let files = policy["files"]
        .as_object()
        .ok_or("pinned input map missing")?;
    let expected = files.keys().cloned().collect::<BTreeSet<_>>();
    if expected != inputs.keys().cloned().collect()
        || expected.iter().any(|path| !in_scope(path))
        || ROOTS.iter().any(|root| {
            !expected.contains(&format!("{root}/Cargo.toml"))
                || !expected.contains(&format!("{root}/Cargo.lock"))
        })
    {
        return Err("pinned graph input inventory drift".into());
    }
    let mut actual_registry = BTreeMap::new();
    for (path, bytes) in inputs {
        if files[path].as_str() != Some(&sifr_sysroot::sha256_hex(bytes)) {
            return Err(format!("pinned graph input hash drift: {path}"));
        }
        if !path.ends_with("Cargo.lock") {
            continue;
        }
        let lock: toml::Value =
            toml::from_str(std::str::from_utf8(bytes).map_err(|error| error.to_string())?)
                .map_err(|error| error.to_string())?;
        let packages = lock["package"].as_array().ok_or("lock packages missing")?;
        let mut identities = BTreeSet::new();
        for package in packages {
            let name = package["name"].as_str().ok_or("lock name missing")?;
            let version = package["version"].as_str().ok_or("lock version missing")?;
            let source = package
                .get("source")
                .map(|value| value.as_str().ok_or("malformed source"))
                .transpose()?;
            if !identities.insert((name, version, source)) {
                return Err("duplicate pinned lock identity".into());
            }
            let Some(source) = source else { continue };
            if source.starts_with("git+") {
                // Full lock bytes above authenticate the owner's exact git revision.
                continue;
            }
            if source != super::REGISTRY {
                return Err(format!("unapproved pinned registry source: {source}"));
            }
            let checksum = package["checksum"]
                .as_str()
                .filter(|value| hash(value))
                .ok_or("invalid locked checksum")?;
            let key = (name.to_owned(), version.to_owned());
            if actual_registry
                .insert(key, checksum.to_owned())
                .is_some_and(|previous| previous != checksum)
            {
                return Err("conflicting pinned checksums".into());
            }
        }
    }
    let mut official = BTreeMap::new();
    for row in policy["registry"]
        .as_array()
        .ok_or("official registry rows missing")?
    {
        let name = row["name"].as_str().ok_or("official name missing")?;
        let version = row["version"].as_str().ok_or("official version missing")?;
        let checksum = row["checksum"]
            .as_str()
            .filter(|value| hash(value))
            .ok_or("invalid official checksum")?;
        if row["source_url"].as_str() != Some(&index_url(name)?)
            || !row["response_sha256"].as_str().is_some_and(hash)
            || !row["observed_at"]
                .as_str()
                .is_some_and(|value| !value.is_empty())
            || official
                .insert((name.to_owned(), version.to_owned()), checksum.to_owned())
                .is_some()
        {
            return Err("invalid/duplicate official registry evidence".into());
        }
    }
    if actual_registry != official {
        return Err("pinned graph official checksum inventory drift".into());
    }
    Ok(expected)
}

pub(crate) fn approved() -> &'static BTreeSet<String> {
    static APPROVED: OnceLock<BTreeSet<String>> = OnceLock::new();
    APPROVED.get_or_init(|| {
        let policy =
            serde_json::from_str(POLICY).unwrap_or_else(|error| panic!("pinned policy: {error}"));
        validate(&policy, &inputs()).unwrap_or_else(|error| panic!("{error}"))
    })
}
