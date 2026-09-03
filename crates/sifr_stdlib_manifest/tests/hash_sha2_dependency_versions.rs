mod support;

use support::TestUnwrap as _;

const WORKSPACE_MANIFEST: &str = include_str!("../../../Cargo.toml");
const WORKSPACE_LOCK: &str = include_str!("../../../Cargo.lock");
const RUST_INTEROP_CATALOG: &str = include_str!("../../sifr_rust_interop_catalog/Cargo.toml");

#[test]
fn sha2_direct_dependencies_use_one_canonical_stable_release() {
    let workspace: toml::Value =
        toml::from_str(WORKSPACE_MANIFEST).test_unwrap("workspace manifest must parse");
    let dependencies = workspace
        .get("workspace")
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(toml::Value::as_table)
        .test_unwrap("workspace dependencies must be a table");

    assert_eq!(
        dependencies
            .get("sha2")
            .and_then(|dependency| dependency.get("version"))
            .and_then(toml::Value::as_str),
        Some("0.11.0")
    );
    assert!(
        dependencies
            .keys()
            .all(|name| name == "sha2" || !name.starts_with("sha2_")),
        "the workspace must not retain a version-named SHA-2 alias"
    );

    let catalog: toml::Value =
        toml::from_str(RUST_INTEROP_CATALOG).test_unwrap("Rust interop catalog must parse");
    assert_eq!(
        catalog
            .get("dependencies")
            .and_then(|dependencies| dependencies.get("sha2"))
            .and_then(|dependency| dependency.get("version"))
            .and_then(toml::Value::as_str),
        Some("=0.11.0")
    );
}

#[test]
fn first_party_lock_edges_use_sha2_0_11() {
    let lock: toml::Value = toml::from_str(WORKSPACE_LOCK).test_unwrap("workspace lock must parse");
    let packages = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .test_unwrap("workspace lock packages must be an array");

    let mut first_party_sha2_edges = packages
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
        .filter(|dependency| dependency.starts_with("sha2"))
        .collect::<Vec<_>>();
    first_party_sha2_edges.sort_unstable();

    let locked_sha2_versions = packages
        .iter()
        .filter(|package| package.get("name").and_then(toml::Value::as_str) == Some("sha2"))
        .filter_map(|package| package.get("version").and_then(toml::Value::as_str))
        .collect::<Vec<_>>();
    let expected_edge = match locked_sha2_versions.as_slice() {
        ["0.11.0"] => "sha2",
        versions if versions.contains(&"0.11.0") => "sha2 0.11.0",
        _ => panic!("Cargo.lock must contain SHA-2 0.11.0: {locked_sha2_versions:?}"),
    };

    assert!(!first_party_sha2_edges.is_empty());
    assert!(
        first_party_sha2_edges
            .iter()
            .all(|dependency| *dependency == expected_edge),
        "all first-party SHA-2 edges must use 0.11.0: {first_party_sha2_edges:?}"
    );
}
