use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const WORKSPACE_MANIFEST: &str = include_str!("../../../Cargo.toml");
const WORKSPACE_LOCK: &str = include_str!("../../../Cargo.lock");
const STDLIB_MANIFEST: &str = include_str!("../../sifr_stdlib/Cargo.toml");
const VENDORED_MANIFEST: &str = include_str!("../../../vendor/base64/Cargo.toml");

#[test]
fn base64_direct_dependency_uses_latest_stable_safe_features() {
    let workspace: toml::Value =
        toml::from_str(WORKSPACE_MANIFEST).expect("workspace manifest must parse");
    let dependency = workspace
        .get("workspace")
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(|dependencies| dependencies.get("base64"))
        .expect("workspace must declare base64");

    assert_eq!(
        dependency.get("version").and_then(toml::Value::as_str),
        Some("0.23.1")
    );
    assert_eq!(
        dependency
            .get("default-features")
            .and_then(toml::Value::as_bool),
        Some(false),
        "base64 0.23 enables unsafe SIMD by default"
    );
    assert_eq!(
        dependency
            .get("features")
            .and_then(toml::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(toml::Value::as_str)
            .collect::<Vec<_>>(),
        ["std"]
    );

    let stdlib: toml::Value = toml::from_str(STDLIB_MANIFEST).expect("stdlib manifest must parse");
    let stdlib_dependency = stdlib
        .get("dependencies")
        .and_then(|dependencies| dependencies.get("base64"))
        .expect("stdlib must inherit base64");
    assert_eq!(
        stdlib_dependency
            .get("workspace")
            .and_then(toml::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        stdlib_dependency
            .get("optional")
            .and_then(toml::Value::as_bool),
        Some(true)
    );

    let vendored: toml::Value =
        toml::from_str(VENDORED_MANIFEST).expect("vendored base64 manifest must parse");
    assert_eq!(
        vendored
            .get("package")
            .and_then(|package| package.get("version"))
            .and_then(toml::Value::as_str),
        Some("0.23.1")
    );
}

#[test]
fn first_party_lock_edges_use_base64_0_23_1() {
    let lock: toml::Value = toml::from_str(WORKSPACE_LOCK).expect("workspace lock must parse");
    let packages = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .expect("workspace lock packages must be an array");

    let locked_versions = packages
        .iter()
        .filter(|package| package.get("name").and_then(toml::Value::as_str) == Some("base64"))
        .filter_map(|package| package.get("version").and_then(toml::Value::as_str))
        .collect::<Vec<_>>();
    assert!(
        locked_versions.contains(&"0.23.1"),
        "Cargo.lock must contain base64 0.23.1: {locked_versions:?}"
    );

    let mut first_party_edges = packages
        .iter()
        .filter(|package| {
            package
                .get("name")
                .and_then(toml::Value::as_str)
                .is_some_and(|name| name == "sifr" || name.starts_with("sifr_"))
        })
        .flat_map(|package| {
            package
                .get("dependencies")
                .and_then(toml::Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter_map(toml::Value::as_str)
        .filter(|dependency| dependency.starts_with("base64"))
        .collect::<Vec<_>>();
    first_party_edges.sort_unstable();

    assert!(!first_party_edges.is_empty());
    assert!(
        first_party_edges
            .iter()
            .all(|dependency| *dependency == "base64 0.23.1"),
        "all first-party Base64 edges must use 0.23.1: {first_party_edges:?}"
    );
}

#[test]
fn vendored_base64_packages_cover_vendored_and_first_party_lock_edges() {
    let lock: toml::Value = toml::from_str(WORKSPACE_LOCK).expect("workspace lock must parse");
    let packages = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .expect("workspace lock packages must be an array");
    let locked_base64_versions = packages
        .iter()
        .filter(|package| package.get("name").and_then(toml::Value::as_str) == Some("base64"))
        .filter_map(|package| package.get("version").and_then(toml::Value::as_str))
        .collect::<BTreeSet<_>>();

    let vendor_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor");
    let mut vendored_packages = BTreeSet::new();
    for entry in fs::read_dir(&vendor_root).expect("vendor directory must be readable") {
        let entry = entry.expect("vendor entry must be readable");
        if !entry
            .file_type()
            .expect("vendor entry type must be readable")
            .is_dir()
        {
            continue;
        }
        let manifest_path = entry.path().join("Cargo.toml");
        if !manifest_path.is_file() {
            continue;
        }
        let manifest = fs::read_to_string(&manifest_path).unwrap_or_else(|error| {
            panic!(
                "vendored manifest {} must be readable: {error}",
                manifest_path.display()
            )
        });
        let manifest: toml::Value = toml::from_str(&manifest).unwrap_or_else(|error| {
            panic!(
                "vendored manifest {} must parse: {error}",
                manifest_path.display()
            )
        });
        let package = manifest
            .get("package")
            .expect("vendored manifest must contain package metadata");
        let name = package
            .get("name")
            .and_then(toml::Value::as_str)
            .expect("vendored package must have a string name");
        let version = package
            .get("version")
            .and_then(toml::Value::as_str)
            .expect("vendored package must have a string version");
        vendored_packages.insert((name.to_string(), version.to_string()));
    }

    let mut required_base64_versions = BTreeSet::new();
    for package in packages {
        let Some(name) = package.get("name").and_then(toml::Value::as_str) else {
            continue;
        };
        let Some(version) = package.get("version").and_then(toml::Value::as_str) else {
            continue;
        };
        let is_owned_package = name == "sifr" || name.starts_with("sifr_");
        let is_vendored_package =
            vendored_packages.contains(&(name.to_string(), version.to_string()));
        if !is_owned_package && !is_vendored_package {
            continue;
        }

        for dependency in package
            .get("dependencies")
            .and_then(toml::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(toml::Value::as_str)
        {
            let mut components = dependency.split_whitespace();
            if components.next() != Some("base64") {
                continue;
            }
            let dependency_version = components.next().unwrap_or_else(|| {
                assert_eq!(
                    locked_base64_versions.len(),
                    1,
                    "an unqualified Base64 lock edge requires one locked version"
                );
                locked_base64_versions
                    .first()
                    .copied()
                    .expect("the Base64 lock version must exist")
            });
            required_base64_versions.insert(dependency_version);
        }
    }

    assert_eq!(required_base64_versions, ["0.22.1", "0.23.1"].into());
    for version in required_base64_versions {
        assert!(
            vendored_packages.contains(&("base64".to_string(), version.to_string())),
            "vendor must contain the Base64 {version} package required by a vendored or first-party lock edge"
        );
    }
}
