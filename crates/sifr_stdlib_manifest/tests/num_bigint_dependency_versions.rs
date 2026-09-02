mod support;

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use support::TestUnwrap as _;

const WORKSPACE_MANIFEST: &str = include_str!("../../../Cargo.toml");
const WORKSPACE_LOCK: &str = include_str!("../../../Cargo.lock");
const DEPENDENCY_PLAN: &str = include_str!("../src/features/dependency_plan.rs");
const VENDORED_0_4_MANIFEST: &str = include_str!("../../../vendor/num-bigint-0.4.8/Cargo.toml");
const VENDORED_0_4_CHECKSUM: &str =
    include_str!("../../../vendor/num-bigint-0.4.8/.cargo-checksum.json");
const VENDORED_0_5_MANIFEST: &str = include_str!("../../../vendor/num-bigint/Cargo.toml");
const VENDORED_0_5_CHECKSUM: &str = include_str!("../../../vendor/num-bigint/.cargo-checksum.json");

#[test]
fn num_bigint_direct_dependencies_use_the_latest_stable_release() {
    let workspace: toml::Value =
        toml::from_str(WORKSPACE_MANIFEST).test_unwrap("workspace manifest must parse");
    let dependency = workspace
        .get("workspace")
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(|dependencies| dependencies.get("num-bigint"))
        .test_unwrap("workspace must declare num-bigint");

    assert_eq!(
        dependency.get("version").and_then(toml::Value::as_str),
        Some("0.5.1")
    );
    assert_eq!(
        dependency
            .get("default-features")
            .and_then(toml::Value::as_bool),
        Some(false)
    );
    assert_eq!(
        dependency.get("features").and_then(toml::Value::as_array),
        Some(&vec![toml::Value::String("std".to_string())])
    );
    assert!(DEPENDENCY_PLAN.contains(
        r#"num-bigint = { version = \"=0.5.1\", default-features = false, features = [\"std\"] }"#
    ));
}

#[test]
fn maintained_first_party_lock_edges_use_num_bigint_0_5_1() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut lock_paths = Vec::new();
    collect_lock_paths(&workspace_root, &mut lock_paths);
    lock_paths.sort();

    let mut checked_edges = 0;
    for lock_path in lock_paths {
        let lock_source =
            fs::read_to_string(&lock_path).test_unwrap("maintained Cargo.lock must be readable");
        let lock: toml::Value =
            toml::from_str(&lock_source).test_unwrap("maintained Cargo.lock must parse");
        let packages = lock
            .get("package")
            .and_then(toml::Value::as_array)
            .test_unwrap("maintained Cargo.lock packages must be an array");
        let first_party_edges = packages
            .iter()
            .filter(|package| is_first_party(package))
            .flat_map(dependencies)
            .filter(|dependency| dependency.starts_with("num-bigint"))
            .collect::<Vec<_>>();
        if first_party_edges.is_empty() {
            continue;
        }
        let versions = num_bigint_versions(packages);
        assert!(
            versions
                .iter()
                .all(|version| *version == "0.4.8" || *version == "0.5.1"),
            "{} contains an unsupported num-bigint line: {versions:?}",
            lock_path.display()
        );
        let expected_edge = if versions.len() == 1 {
            "num-bigint"
        } else {
            "num-bigint 0.5.1"
        };

        for dependency in first_party_edges {
            checked_edges += 1;
            assert_eq!(
                dependency,
                expected_edge,
                "{} gives a first-party package the wrong num-bigint line",
                lock_path.display()
            );
        }
    }

    assert!(
        checked_edges > 0,
        "the maintained locks must contain first-party num-bigint edges"
    );
}

#[test]
fn bigdecimal_and_vendor_sources_keep_their_owned_num_bigint_lines() {
    let lock: toml::Value = toml::from_str(WORKSPACE_LOCK).test_unwrap("workspace lock must parse");
    let packages = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .test_unwrap("workspace lock packages must be an array");
    let bigdecimal = packages
        .iter()
        .find(|package| package.get("name").and_then(toml::Value::as_str) == Some("bigdecimal"))
        .test_unwrap("workspace lock must contain BigDecimal");
    assert!(
        dependencies(bigdecimal).any(|dependency| dependency == "num-bigint 0.4.8"),
        "BigDecimal must retain its latest compatible 0.4 release"
    );

    assert_vendored_release(
        packages,
        VENDORED_0_4_MANIFEST,
        VENDORED_0_4_CHECKSUM,
        "0.4.8",
    );
    assert_vendored_release(
        packages,
        VENDORED_0_5_MANIFEST,
        VENDORED_0_5_CHECKSUM,
        "0.5.1",
    );
}

fn collect_lock_paths(directory: &Path, lock_paths: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(directory).test_unwrap("workspace directory must be readable") {
        let entry = entry.test_unwrap("workspace entry must be readable");
        let path = entry.path();
        let name = entry.file_name();
        if path.is_dir() {
            if !matches!(name.to_str(), Some(".git" | "target" | "vendor")) {
                collect_lock_paths(&path, lock_paths);
            }
        } else if name == "Cargo.lock" {
            lock_paths.push(path);
        }
    }
}

fn is_first_party(package: &toml::Value) -> bool {
    package
        .get("name")
        .and_then(toml::Value::as_str)
        .is_some_and(|name| name == "sifr" || name.starts_with("sifr_"))
}

fn dependencies(package: &toml::Value) -> impl Iterator<Item = &str> {
    package
        .get("dependencies")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_str)
}

fn num_bigint_versions(packages: &[toml::Value]) -> BTreeSet<&str> {
    packages
        .iter()
        .filter(|package| package.get("name").and_then(toml::Value::as_str) == Some("num-bigint"))
        .filter_map(|package| package.get("version").and_then(toml::Value::as_str))
        .collect()
}

fn assert_vendored_release(
    packages: &[toml::Value],
    manifest_source: &str,
    checksum_source: &str,
    expected_version: &str,
) {
    let manifest: toml::Value =
        toml::from_str(manifest_source).test_unwrap("vendored num-bigint manifest must parse");
    assert_eq!(
        manifest
            .get("package")
            .and_then(|package| package.get("version"))
            .and_then(toml::Value::as_str),
        Some(expected_version)
    );

    let lock_checksum = packages
        .iter()
        .find(|package| {
            package.get("name").and_then(toml::Value::as_str) == Some("num-bigint")
                && package.get("version").and_then(toml::Value::as_str) == Some(expected_version)
        })
        .and_then(|package| package.get("checksum"))
        .and_then(toml::Value::as_str)
        .test_unwrap("workspace lock must contain the certified num-bigint release");
    let checksum: serde_json::Value =
        serde_json::from_str(checksum_source).test_unwrap("vendored checksum must parse");
    assert_eq!(
        checksum.get("package").and_then(serde_json::Value::as_str),
        Some(lock_checksum)
    );
}
