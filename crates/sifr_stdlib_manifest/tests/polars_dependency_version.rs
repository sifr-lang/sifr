#![allow(clippy::expect_used)]

use std::collections::BTreeSet;

const POLARS_VERSION: &str = "0.55.2";
const POLARS_PACKAGE_HASH: &str =
    "d52d3ed4e6b3917427f6d3c43edbd2740babe228bb4ccfa3431eac105844045d";

const CATALOG_MANIFEST: &str = include_str!("../../sifr_rust_interop_catalog/Cargo.toml");
const FIXTURE_MANIFEST: &str = include_str!(
    "../../../verification/areas/rust_interop/fixtures/advanced_data_runtime_matrix/examples/advanced_data_runtime/Cargo.toml"
);
const BRIDGE_SOURCE: &str = include_str!(
    "../../../verification/areas/rust_interop/fixtures/advanced_data_runtime_matrix/examples/advanced_data_runtime/rust/sifr_arrow_bridge/src/record_batch.rs"
);
const WORKSPACE_LOCK: &str = include_str!("../../../Cargo.lock");
const FIXTURE_LOCK: &str = include_str!(
    "../../../verification/areas/rust_interop/fixtures/advanced_data_runtime_matrix/examples/advanced_data_runtime/Cargo.lock"
);

const POLARS_PACKAGES: &[&str] = &[
    "polars",
    "polars-arrow",
    "polars-async",
    "polars-buffer",
    "polars-compute",
    "polars-config",
    "polars-core",
    "polars-dtype",
    "polars-error",
    "polars-expr",
    "polars-io",
    "polars-lazy",
    "polars-mem-engine",
    "polars-ooc",
    "polars-ops",
    "polars-parquet",
    "polars-plan",
    "polars-row",
    "polars-schema",
    "polars-sql",
    "polars-stream",
    "polars-time",
    "polars-utils",
];

#[test]
fn maintained_manifests_select_latest_stable_polars() {
    let catalog: toml::Value =
        toml::from_str(CATALOG_MANIFEST).expect("catalog manifest must parse");
    let catalog_dependencies = catalog
        .get("dependencies")
        .expect("catalog must declare dependencies");
    assert_dependency(catalog_dependencies, Some(true));

    let fixture: toml::Value =
        toml::from_str(FIXTURE_MANIFEST).expect("fixture manifest must parse");
    let fixture_dependencies = fixture
        .get("workspace")
        .and_then(|workspace| workspace.get("dependencies"))
        .expect("fixture must declare workspace dependencies");
    assert_dependency(fixture_dependencies, None);
}

#[test]
fn maintained_locks_use_one_current_polars_family() {
    for (label, source) in [
        ("workspace", WORKSPACE_LOCK),
        ("advanced-data fixture", FIXTURE_LOCK),
    ] {
        let lock: toml::Value =
            toml::from_str(source).unwrap_or_else(|error| panic!("{label} lock: {error}"));
        let packages = lock
            .get("package")
            .and_then(toml::Value::as_array)
            .expect("lock packages must be an array");
        let actual = packages
            .iter()
            .filter_map(|package| {
                let name = package.get("name").and_then(toml::Value::as_str)?;
                let version = package.get("version").and_then(toml::Value::as_str)?;
                (version == POLARS_VERSION && (name == "polars" || name.starts_with("polars-")))
                    .then_some(name)
            })
            .collect::<BTreeSet<_>>();
        let expected = POLARS_PACKAGES.iter().copied().collect::<BTreeSet<_>>();
        assert_eq!(actual, expected, "{label} Polars package family");

        for name in POLARS_PACKAGES {
            let matching = packages
                .iter()
                .filter(|package| package.get("name").and_then(toml::Value::as_str) == Some(name))
                .collect::<Vec<_>>();
            assert_eq!(matching.len(), 1, "{label} must contain one {name}");
            assert_eq!(
                matching[0].get("version").and_then(toml::Value::as_str),
                Some(POLARS_VERSION),
                "{label} {name} version"
            );
        }

        let polars = packages
            .iter()
            .find(|package| package.get("name").and_then(toml::Value::as_str) == Some("polars"))
            .expect("lock must contain polars");
        assert_eq!(
            polars.get("checksum").and_then(toml::Value::as_str),
            Some(POLARS_PACKAGE_HASH),
            "{label} Polars checksum"
        );
    }
}

#[test]
fn runtime_bridge_uses_polars_0_55_dataframe_sortedness() {
    assert!(BRIDGE_SOURCE.contains("DataFrameIsSorted"));
    assert!(BRIDGE_SOURCE.contains(".is_sorted(&[\"value\".into()], &[false], &[false])"));
    assert!(BRIDGE_SOURCE.contains("|| !polars_sorted"));
    assert!(BRIDGE_SOURCE.contains(".map_err(display_error)?"));
    assert!(!BRIDGE_SOURCE.contains("unwrap_or"));
}

fn assert_dependency(dependencies: &toml::Value, optional: Option<bool>) {
    let dependency = dependencies
        .get("polars")
        .expect("manifest must declare polars");
    assert_eq!(
        dependency.get("version").and_then(toml::Value::as_str),
        Some("=0.55.2")
    );
    assert_eq!(
        dependency
            .get("default-features")
            .and_then(toml::Value::as_bool),
        Some(true)
    );
    if let Some(expected) = optional {
        assert_eq!(
            dependency.get("optional").and_then(toml::Value::as_bool),
            Some(expected)
        );
    }
}
