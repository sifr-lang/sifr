#![allow(clippy::expect_used)]

use itertools::structs::AllEqualValueError;
use itertools::{Itertools, Position};

const ITERTOOLS_VERSION: &str = "0.15.0";
const ITERTOOLS_PACKAGE_HASH: &str =
    "8b4baf93f58d4425749ca49a51c50ebab072c5df6994d08fed93541c331481dc";
const DATAFUSION_ITERTOOLS_VERSION: &str = "0.14.0";
const DATAFUSION_ITERTOOLS_PACKAGE_HASH: &str =
    "2b192c782037fadd9cfa75548310488aabdbf3d2da73885b31bd0abd03351285";

const WORKSPACE_MANIFEST: &str = include_str!("../../../Cargo.toml");
const STDLIB_MANIFEST: &str = include_str!("../Cargo.toml");
const RUFF_MANIFEST: &str = include_str!("../../../third_party/ruff/Cargo.toml");
const WORKSPACE_LOCK: &str = include_str!("../../../Cargo.lock");
const ITERTOOLS_VENDOR_MANIFEST: &str = include_str!("../../../vendor/itertools/Cargo.toml");
const ITERTOOLS_VENDOR_CHECKSUM: &str =
    include_str!("../../../vendor/itertools/.cargo-checksum.json");
const DATAFUSION_ITERTOOLS_VENDOR_MANIFEST: &str =
    include_str!("../../../vendor/itertools-0.14.0/Cargo.toml");
const DATAFUSION_ITERTOOLS_VENDOR_CHECKSUM: &str =
    include_str!("../../../vendor/itertools-0.14.0/.cargo-checksum.json");

#[test]
fn maintained_itertools_dependencies_use_the_latest_stable_policy() {
    let workspace: toml::Value =
        toml::from_str(WORKSPACE_MANIFEST).expect("workspace manifest must parse");
    let dependency = workspace
        .get("workspace")
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(|dependencies| dependencies.get("itertools"))
        .expect("workspace must declare Itertools");
    assert_eq!(
        dependency.get("version").and_then(toml::Value::as_str),
        Some(ITERTOOLS_VERSION)
    );
    assert_eq!(
        dependency
            .get("default-features")
            .and_then(toml::Value::as_bool),
        Some(false)
    );
    assert_eq!(string_array(dependency, "features"), ["use_std"]);

    let stdlib: toml::Value = toml::from_str(STDLIB_MANIFEST).expect("stdlib manifest must parse");
    let stdlib_dependency = stdlib
        .get("dev-dependencies")
        .and_then(|dependencies| dependencies.get("itertools"))
        .expect("stdlib certification must inherit Itertools");
    assert_eq!(
        stdlib_dependency
            .get("workspace")
            .and_then(toml::Value::as_bool),
        Some(true)
    );

    let ruff: toml::Value = toml::from_str(RUFF_MANIFEST).expect("Ruff manifest must parse");
    assert_eq!(
        ruff.get("workspace")
            .and_then(|workspace| workspace.get("dependencies"))
            .and_then(|dependencies| dependencies.get("itertools"))
            .and_then(|dependency| dependency.get("version"))
            .and_then(toml::Value::as_str),
        Some(ITERTOOLS_VERSION),
        "the maintained Ruff fork must share the current Itertools line"
    );
}

#[test]
fn itertools_0_15_apis_compile_with_the_canonical_iterator_contracts() {
    let windows = (1..=5).array_windows::<3>().collect_vec();
    assert_eq!(windows, [[1, 2, 3], [2, 3, 4], [3, 4, 5]]);

    let combinations = (1..=3)
        .array_combinations_with_replacement::<2>()
        .collect_vec();
    assert_eq!(
        combinations,
        [[1, 1], [1, 2], [1, 3], [2, 2], [2, 3], [3, 3]]
    );

    let stripped = (1..=5)
        .strip_prefix([1, 2])
        .expect("matching prefix must be removed")
        .collect_vec();
    assert_eq!(stripped, [3, 4, 5]);
    assert!((1..=5).strip_prefix([1, 9]).is_err());

    let positioned = ["first", "middle", "last"]
        .into_iter()
        .with_position()
        .collect_vec();
    assert_eq!(
        positioned,
        [
            (
                Position {
                    is_first: true,
                    is_last: false,
                },
                "first",
            ),
            (
                Position {
                    is_first: false,
                    is_last: false,
                },
                "middle",
            ),
            (
                Position {
                    is_first: false,
                    is_last: true,
                },
                "last",
            ),
        ]
    );

    assert_eq!(
        [1, 1, 2].into_iter().all_equal_value(),
        Err(AllEqualValueError(Some([1, 2])))
    );
}

#[test]
fn first_party_lock_edges_use_itertools_0_15() {
    let lock: toml::Value = toml::from_str(WORKSPACE_LOCK).expect("Cargo.lock must parse");
    let packages = lock_packages(&lock);
    let current = packages
        .iter()
        .find(|package| {
            package_name(package) == Some("itertools")
                && package_version(package) == Some(ITERTOOLS_VERSION)
        })
        .expect("Cargo.lock must contain Itertools 0.15");
    assert_eq!(
        current.get("checksum").and_then(toml::Value::as_str),
        Some(ITERTOOLS_PACKAGE_HASH)
    );

    let first_party_edges = packages
        .iter()
        .filter(|package| {
            package_name(package).is_some_and(|name| name == "sifr" || name.starts_with("sifr_"))
        })
        .flat_map(|package| {
            package
                .get("dependencies")
                .and_then(toml::Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(toml::Value::as_str)
                .filter(|dependency| dependency.starts_with("itertools"))
                .map(move |dependency| {
                    (
                        package_name(package).expect("first-party package must have a name"),
                        dependency,
                    )
                })
        })
        .collect::<Vec<_>>();
    assert_eq!(
        first_party_edges,
        [("sifr_stdlib_manifest", "itertools 0.15.0")]
    );
}

#[test]
fn vendor_contains_current_and_datafusion_owned_itertools_releases() {
    assert_vendor_release(
        ITERTOOLS_VENDOR_MANIFEST,
        ITERTOOLS_VENDOR_CHECKSUM,
        ITERTOOLS_VERSION,
        ITERTOOLS_PACKAGE_HASH,
    );
    assert_vendor_release(
        DATAFUSION_ITERTOOLS_VENDOR_MANIFEST,
        DATAFUSION_ITERTOOLS_VENDOR_CHECKSUM,
        DATAFUSION_ITERTOOLS_VERSION,
        DATAFUSION_ITERTOOLS_PACKAGE_HASH,
    );

    let lock: toml::Value = toml::from_str(WORKSPACE_LOCK).expect("Cargo.lock must parse");
    let packages = lock_packages(&lock);
    let datafusion = packages
        .iter()
        .find(|package| package_name(package) == Some("datafusion"))
        .expect("Cargo.lock must contain DataFusion");
    let dependencies = datafusion
        .get("dependencies")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_str)
        .filter(|dependency| dependency.starts_with("itertools"))
        .collect::<Vec<_>>();
    assert_eq!(
        dependencies,
        ["itertools 0.14.0"],
        "Item 22 owns the upstream DataFusion transition to Itertools 0.15"
    );
}

fn assert_vendor_release(
    manifest_source: &str,
    checksum_source: &str,
    expected_version: &str,
    expected_hash: &str,
) {
    let manifest: toml::Value =
        toml::from_str(manifest_source).expect("vendor manifest must parse");
    assert_eq!(
        manifest
            .get("package")
            .and_then(|package| package.get("name"))
            .and_then(toml::Value::as_str),
        Some("itertools")
    );
    assert_eq!(
        manifest
            .get("package")
            .and_then(|package| package.get("version"))
            .and_then(toml::Value::as_str),
        Some(expected_version)
    );

    let checksum: serde_json::Value =
        serde_json::from_str(checksum_source).expect("vendor checksum must parse");
    assert_eq!(
        checksum.get("package").and_then(serde_json::Value::as_str),
        Some(expected_hash)
    );
}

fn string_array<'a>(value: &'a toml::Value, key: &str) -> Vec<&'a str> {
    value
        .get(key)
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_str)
        .collect()
}

fn lock_packages(lock: &toml::Value) -> &[toml::Value] {
    lock.get("package")
        .and_then(toml::Value::as_array)
        .expect("Cargo.lock packages must be an array")
}

fn package_name(package: &toml::Value) -> Option<&str> {
    package.get("name").and_then(toml::Value::as_str)
}

fn package_version(package: &toml::Value) -> Option<&str> {
    package.get("version").and_then(toml::Value::as_str)
}
