#![allow(clippy::expect_used)]

use std::collections::BTreeSet;

const ARROW_VERSION: &str = "59.2.0";
const ARROW_PACKAGE_HASH: &str = "61d285d16bce7d0be61912f7928342b673067b6b7d7ef6cc179258ba7de1fecf";
const DATAFUSION_VERSION: &str = "55.0.0";
const DATAFUSION_PACKAGE_HASH: &str =
    "96f76f0167ed0842b29a3d1e41be3c034c0a46409a3a703cc4cc84ee8c24abf4";

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

const ARROW_PACKAGES: &[&str] = &[
    "arrow",
    "arrow-arith",
    "arrow-array",
    "arrow-buffer",
    "arrow-cast",
    "arrow-csv",
    "arrow-data",
    "arrow-ipc",
    "arrow-json",
    "arrow-ord",
    "arrow-row",
    "arrow-schema",
    "arrow-select",
    "arrow-string",
];

const DATAFUSION_PACKAGES: &[&str] = &[
    "datafusion",
    "datafusion-catalog",
    "datafusion-catalog-listing",
    "datafusion-common",
    "datafusion-common-runtime",
    "datafusion-datasource",
    "datafusion-datasource-arrow",
    "datafusion-datasource-csv",
    "datafusion-datasource-json",
    "datafusion-datasource-parquet",
    "datafusion-doc",
    "datafusion-execution",
    "datafusion-expr",
    "datafusion-expr-common",
    "datafusion-functions",
    "datafusion-functions-aggregate",
    "datafusion-functions-aggregate-common",
    "datafusion-functions-nested",
    "datafusion-functions-table",
    "datafusion-functions-window",
    "datafusion-functions-window-common",
    "datafusion-macros",
    "datafusion-optimizer",
    "datafusion-physical-expr",
    "datafusion-physical-expr-adapter",
    "datafusion-physical-expr-common",
    "datafusion-physical-optimizer",
    "datafusion-physical-plan",
    "datafusion-pruning",
    "datafusion-session",
    "datafusion-sql",
];

#[test]
fn maintained_manifests_select_the_latest_stable_analytical_stack() {
    let catalog: toml::Value =
        toml::from_str(CATALOG_MANIFEST).expect("catalog manifest must parse");
    let catalog_dependencies = catalog
        .get("dependencies")
        .expect("catalog must declare dependencies");
    assert_dependency(catalog_dependencies, "arrow", ARROW_VERSION, Some(true));
    assert_dependency(
        catalog_dependencies,
        "datafusion",
        DATAFUSION_VERSION,
        Some(true),
    );

    let fixture: toml::Value =
        toml::from_str(FIXTURE_MANIFEST).expect("fixture manifest must parse");
    let fixture_dependencies = fixture
        .get("workspace")
        .and_then(|workspace| workspace.get("dependencies"))
        .expect("fixture must declare workspace dependencies");
    assert_dependency(fixture_dependencies, "arrow", ARROW_VERSION, None);
    assert_dependency(fixture_dependencies, "datafusion", DATAFUSION_VERSION, None);
}

#[test]
fn maintained_locks_use_one_current_arrow_and_datafusion_family() {
    for (label, source) in [
        ("workspace", WORKSPACE_LOCK),
        ("advanced-data fixture", FIXTURE_LOCK),
    ] {
        let lock: toml::Value =
            toml::from_str(source).unwrap_or_else(|error| panic!("{label} lock: {error}"));
        let packages = lock_packages(&lock);
        assert_package_hash(label, packages, "arrow", ARROW_VERSION, ARROW_PACKAGE_HASH);
        assert_package_hash(
            label,
            packages,
            "datafusion",
            DATAFUSION_VERSION,
            DATAFUSION_PACKAGE_HASH,
        );
        assert_family(label, packages, "arrow", ARROW_PACKAGES, ARROW_VERSION);
        assert_family(
            label,
            packages,
            "datafusion",
            DATAFUSION_PACKAGES,
            DATAFUSION_VERSION,
        );

        let parquet = package(packages, "parquet");
        assert_eq!(
            package_version(parquet),
            Some(ARROW_VERSION),
            "{label} Parquet must share the Arrow release"
        );

        let datafusion = package(packages, "datafusion");
        let edges = dependency_edges(datafusion).collect::<BTreeSet<_>>();
        for expected in ["arrow", "itertools 0.15.0", "parquet"] {
            assert!(
                edges.contains(expected),
                "{label} DataFusion must depend on {expected}: {edges:?}"
            );
        }
    }
}

#[test]
fn runtime_bridge_uses_datafusion_55_nan_fill_and_propagates_catalog_errors() {
    assert!(BRIDGE_SOURCE.contains("fill_nan(&ScalarValue::from(0.0), &[\"value\"])"));
    assert!(BRIDGE_SOURCE.contains(".table_exist(\"input\")"));
    assert!(BRIDGE_SOURCE.contains(".map_err(display_error)?"));
    assert!(!BRIDGE_SOURCE.contains("unwrap_or(false)"));
}

fn assert_dependency(
    dependencies: &toml::Value,
    name: &str,
    version: &str,
    optional: Option<bool>,
) {
    let dependency = dependencies
        .get(name)
        .unwrap_or_else(|| panic!("manifest must declare {name}"));
    let expected_version = format!("={version}");
    assert_eq!(
        dependency.get("version").and_then(toml::Value::as_str),
        Some(expected_version.as_str()),
        "{name} exact version"
    );
    assert_eq!(
        dependency
            .get("default-features")
            .and_then(toml::Value::as_bool),
        Some(true),
        "{name} default-feature policy"
    );
    if let Some(expected) = optional {
        assert_eq!(
            dependency.get("optional").and_then(toml::Value::as_bool),
            Some(expected),
            "{name} optional policy"
        );
    }
}

fn assert_family(
    label: &str,
    packages: &[toml::Value],
    prefix: &str,
    expected_names: &[&str],
    expected_version: &str,
) {
    let actual = packages
        .iter()
        .filter_map(|package| {
            let name = package_name(package)?;
            (name == prefix || name.starts_with(&format!("{prefix}-")))
                .then_some((name, package_version(package)))
        })
        .collect::<BTreeSet<_>>();
    let expected = expected_names
        .iter()
        .map(|name| (*name, Some(expected_version)))
        .collect::<BTreeSet<_>>();
    assert_eq!(actual, expected, "{label} {prefix} package family");
}

fn assert_package_hash(
    label: &str,
    packages: &[toml::Value],
    name: &str,
    version: &str,
    expected_hash: &str,
) {
    let matching = packages
        .iter()
        .filter(|package| package_name(package) == Some(name))
        .collect::<Vec<_>>();
    assert_eq!(matching.len(), 1, "{label} must contain one {name}");
    assert_eq!(package_version(matching[0]), Some(version));
    assert_eq!(
        matching[0].get("checksum").and_then(toml::Value::as_str),
        Some(expected_hash),
        "{label} {name} checksum"
    );
}

fn lock_packages(lock: &toml::Value) -> &[toml::Value] {
    lock.get("package")
        .and_then(toml::Value::as_array)
        .expect("lock packages must be an array")
}

fn package<'a>(packages: &'a [toml::Value], name: &str) -> &'a toml::Value {
    packages
        .iter()
        .find(|package| package_name(package) == Some(name))
        .unwrap_or_else(|| panic!("lock must contain {name}"))
}

fn dependency_edges(package: &toml::Value) -> impl Iterator<Item = &str> {
    package
        .get("dependencies")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_str)
}

fn package_name(package: &toml::Value) -> Option<&str> {
    package.get("name").and_then(toml::Value::as_str)
}

fn package_version(package: &toml::Value) -> Option<&str> {
    package.get("version").and_then(toml::Value::as_str)
}
