mod support;

#[allow(dead_code)]
#[path = "support/cargo_edges.rs"]
mod cargo_edges;
#[allow(dead_code)]
#[path = "support/cargo_inventory.rs"]
mod cargo_inventory;
#[allow(dead_code)]
#[path = "support/cargo_vendor.rs"]
mod cargo_vendor;

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
    cargo_edges::first_party_edges("num-bigint", "0.5.1");
}

#[test]
fn sole_num_bigint_target_must_resolve_to_the_current_registry_identity() {
    for version in ["0.4.8", "0.5.1"] {
        let source = format!(
            r#"[[package]]
name = "num-bigint"
version = "{version}"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#
        );
        let lock: toml::Value = toml::from_str(&source).test_unwrap("lock");
        let packages = cargo_edges::packages(&lock).test_unwrap("packages");
        assert_eq!(
            cargo_edges::current_edge(packages, "num-bigint", "num-bigint", "0.5.1").is_ok(),
            version == "0.5.1"
        );
    }
    assert!(cargo_edges::current_edge(&[], "num-bigint", "num-bigint", "0.5.1").is_err());
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

fn dependencies(package: &toml::Value) -> impl Iterator<Item = &str> {
    package
        .get("dependencies")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_str)
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

#[test]
fn num_bigint_vendor_files_and_exact_selected_versions_are_authenticated() {
    let lock: toml::Value = toml::from_str(WORKSPACE_LOCK).test_unwrap("lock");
    cargo_vendor::family(
        &cargo_inventory::root(),
        "num-bigint",
        &["0.4.8", "0.5.1"],
        cargo_edges::packages(&lock).test_unwrap("packages"),
    )
    .test_unwrap("Num BigInt vendor closure");
}
