#![allow(clippy::expect_used)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const AUDIT: &str = include_str!("data/rust_latest_stable_registry.json");
const WORKSPACE_LOCK: &str = include_str!("../../../Cargo.lock");
const SKIPPED_DIRECTORIES: &[&str] = &[
    ".git",
    ".mypy_cache",
    ".pytest_cache",
    ".venv",
    "__pycache__",
    "node_modules",
    "sifr_output",
    "target",
    "third_party",
    "vendor",
];

#[derive(Debug)]
struct RegistryRelease {
    latest_stable: String,
    checksum: String,
}

#[test]
fn every_maintained_rust_direct_declaration_matches_the_registry_audit() {
    let audit: serde_json::Value = serde_json::from_str(AUDIT).expect("audit JSON must parse");
    assert_eq!(audit["schema_version"].as_u64(), Some(1));
    assert_eq!(audit["audited_at"].as_str(), Some("2026-08-26"));
    assert_eq!(
        audit["source"].as_str(),
        Some("https://crates.io/api/v1/crates/{crate}")
    );

    let releases = registry_releases(&audit);
    assert_eq!(releases.len(), 109, "audited direct package count");

    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root must resolve");
    let mut manifests = Vec::new();
    collect_manifests(&root, &root, &mut manifests);
    manifests.sort();
    assert_eq!(
        manifests.len(),
        usize::try_from(
            audit["maintained_manifests"]
                .as_u64()
                .expect("manifest count must be an integer"),
        )
        .expect("manifest count must fit usize"),
        "maintained Cargo.toml inventory drifted; refresh the official registry audit"
    );

    let mut declaration_count = 0_usize;
    let mut declared_packages = BTreeSet::new();
    for manifest in &manifests {
        let source = fs::read_to_string(manifest)
            .unwrap_or_else(|error| panic!("{}: {error}", manifest.display()));
        let parsed: toml::Value = toml::from_str(&source)
            .unwrap_or_else(|error| panic!("{}: {error}", manifest.display()));
        collect_manifest_declarations(
            manifest,
            &parsed,
            &releases,
            &mut declared_packages,
            &mut declaration_count,
        );
    }

    assert_eq!(
        declaration_count,
        usize::try_from(
            audit["direct_declarations"]
                .as_u64()
                .expect("declaration count must be an integer"),
        )
        .expect("declaration count must fit usize"),
        "direct declaration inventory drifted; refresh the official registry audit"
    );
    assert_eq!(
        declared_packages,
        releases.keys().cloned().collect(),
        "the registry audit and maintained direct package set must be exact"
    );
}

#[test]
fn audited_checksums_match_the_workspace_lock_when_present() {
    let audit: serde_json::Value = serde_json::from_str(AUDIT).expect("audit JSON must parse");
    let releases = registry_releases(&audit);
    let lock: toml::Value = toml::from_str(WORKSPACE_LOCK).expect("workspace lock must parse");
    let packages = lock["package"]
        .as_array()
        .expect("workspace lock packages must be an array");

    for (name, release) in releases {
        let matching = packages.iter().find(|package| {
            package["name"].as_str() == Some(&name)
                && stable_core(package["version"].as_str().unwrap_or_default())
                    == stable_core(&release.latest_stable)
        });
        let Some(package) = matching else {
            continue;
        };
        assert_eq!(
            package["checksum"].as_str(),
            Some(release.checksum.as_str()),
            "{name} registry checksum"
        );
    }
}

fn registry_releases(audit: &serde_json::Value) -> BTreeMap<String, RegistryRelease> {
    audit["packages"]
        .as_array()
        .expect("audit packages must be an array")
        .iter()
        .map(|package| {
            let name = package["name"]
                .as_str()
                .expect("audit package name")
                .to_string();
            let latest_stable = package["latest_stable"]
                .as_str()
                .expect("audit stable version")
                .to_string();
            let checksum = package["checksum"]
                .as_str()
                .expect("audit checksum")
                .to_string();
            assert_eq!(checksum.len(), 64, "{name} checksum length");
            assert!(
                checksum.bytes().all(|byte| byte.is_ascii_hexdigit()),
                "{name} checksum format"
            );
            (
                name,
                RegistryRelease {
                    latest_stable,
                    checksum,
                },
            )
        })
        .collect()
}

fn collect_manifests(root: &Path, directory: &Path, manifests: &mut Vec<PathBuf>) {
    if directory != root && directory.join(".git").exists() {
        return;
    }
    let entries =
        fs::read_dir(directory).unwrap_or_else(|error| panic!("{}: {error}", directory.display()));
    for entry in entries {
        let entry = entry.expect("directory entry must be readable");
        let path = entry.path();
        let file_type = entry.file_type().expect("entry type must be readable");
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            let name = entry.file_name();
            if !SKIPPED_DIRECTORIES.contains(&name.to_string_lossy().as_ref()) {
                collect_manifests(root, &path, manifests);
            }
        } else if entry.file_name() == "Cargo.toml" {
            manifests.push(path);
        }
    }
}

fn collect_manifest_declarations(
    path: &Path,
    manifest: &toml::Value,
    releases: &BTreeMap<String, RegistryRelease>,
    declared_packages: &mut BTreeSet<String>,
    declaration_count: &mut usize,
) {
    collect_dependency_tables(
        path,
        manifest,
        releases,
        declared_packages,
        declaration_count,
    );
    if let Some(workspace) = manifest.get("workspace") {
        collect_dependency_tables(
            path,
            workspace,
            releases,
            declared_packages,
            declaration_count,
        );
    }
    if let Some(targets) = manifest.get("target").and_then(toml::Value::as_table) {
        for target in targets.values() {
            collect_dependency_tables(path, target, releases, declared_packages, declaration_count);
        }
    }
}

fn collect_dependency_tables(
    path: &Path,
    owner: &toml::Value,
    releases: &BTreeMap<String, RegistryRelease>,
    declared_packages: &mut BTreeSet<String>,
    declaration_count: &mut usize,
) {
    for table_name in ["dependencies", "dev-dependencies", "build-dependencies"] {
        let Some(dependencies) = owner.get(table_name).and_then(toml::Value::as_table) else {
            continue;
        };
        for (alias, specification) in dependencies {
            let Some((name, requirement)) = registry_requirement(alias, specification) else {
                continue;
            };
            let release = releases.get(name).unwrap_or_else(|| {
                panic!(
                    "{}: {name} is absent from the official registry audit",
                    path.display()
                )
            });
            assert!(
                requirement_matches_latest(requirement, &release.latest_stable),
                "{}: {name} requirement {requirement:?} does not name latest stable {}",
                path.display(),
                release.latest_stable
            );
            declared_packages.insert(name.to_string());
            *declaration_count += 1;
        }
    }
}

fn registry_requirement<'a>(
    alias: &'a str,
    specification: &'a toml::Value,
) -> Option<(&'a str, &'a str)> {
    if let Some(requirement) = specification.as_str() {
        return Some((alias, requirement));
    }
    let table = specification.as_table()?;
    if table.get("workspace").and_then(toml::Value::as_bool) == Some(true)
        || table.contains_key("git")
        || table.contains_key("path")
    {
        return None;
    }
    let requirement = table.get("version")?.as_str()?;
    let name = table
        .get("package")
        .and_then(toml::Value::as_str)
        .unwrap_or(alias);
    Some((name, requirement))
}

fn requirement_matches_latest(requirement: &str, latest: &str) -> bool {
    let requirement = requirement.trim_start_matches(['=', '^', '~']);
    if requirement.contains(['*', '<', '>', ',']) {
        return false;
    }
    let required_parts = requirement.split('.').collect::<Vec<_>>();
    let latest_parts = stable_core(latest).split('.').collect::<Vec<_>>();
    required_parts.len() <= latest_parts.len()
        && required_parts
            .iter()
            .zip(&latest_parts)
            .all(|(required, stable)| required == stable)
        && latest_parts[required_parts.len()..]
            .iter()
            .all(|part| *part == "0")
}

fn stable_core(version: &str) -> &str {
    version.split_once('+').map_or(version, |(core, _)| core)
}
