#![allow(clippy::expect_used)]

const CATALOG_MANIFEST: &str = include_str!("../../sifr_rust_interop_catalog/Cargo.toml");
const FIXTURE_MANIFEST: &str = include_str!(
    "../../../verification/areas/rust_interop/fixtures/ecosystem_backend_certification/examples/backend_feature_package/Cargo.toml"
);
const FIXTURE_POLICY: &str = include_str!(
    "../../../verification/areas/rust_interop/fixtures/ecosystem_backend_certification/fixture.json"
);
const MATRIX_POLICY: &str =
    include_str!("../../../verification/areas/rust_interop/data/rust_interop_fixture_matrix.json");

const WORKSPACE_LOCK: &str = include_str!("../../../Cargo.lock");
const FIXTURE_LOCK: &str = include_str!(
    "../../../verification/areas/rust_interop/fixtures/ecosystem_backend_certification/examples/backend_feature_package/Cargo.lock"
);

const SQLX_VERSION: &str = "0.9.0";
const SQLX_PACKAGE_HASH: &str = "378620ccc25c62c89d8be1c819e76a88d59bdcc3304733330788948e619bfd71";
const SQLX_CORE_PACKAGE_HASH: &str =
    "05b44e85bf579a8eeb4ceaa77a3a523baf2bf0e9bac7e40f405d537b5d2d5ccb";
const SQLX_MACROS_PACKAGE_HASH: &str =
    "bd2b84f2bc39a5705ef27ec785a11c934a41bbd4a24941e257927cddc26b60bf";
const SQLX_MACROS_CORE_PACKAGE_HASH: &str =
    "fb8d96de5fdc85a5c4ec813432b523ec637e80ba98f046555f75f7908ddac7c3";
const SQLX_POSTGRES_PACKAGE_HASH: &str =
    "87a2bdd6e83f6b3ea525ca9fee568030508b58355a43d0b2c1674d5f79dcd65e";

const CATALOG_FEATURES: &[&str] = &["runtime-tokio", "tls-rustls-ring-webpki", "postgres"];
const FIXTURE_FEATURES: &[&str] = &[
    "runtime-tokio",
    "tls-rustls-ring-webpki",
    "postgres",
    "macros",
];

#[test]
fn maintained_sqlx_dependencies_use_the_latest_stable_policy() {
    let catalog = dependency(CATALOG_MANIFEST, "catalog");
    assert_dependency("catalog", &catalog, CATALOG_FEATURES);
    assert_eq!(
        catalog.get("optional").and_then(toml::Value::as_bool),
        Some(true),
        "catalog SQLx must remain optional"
    );

    let fixture = dependency(FIXTURE_MANIFEST, "backend fixture");
    assert_dependency("backend fixture", &fixture, FIXTURE_FEATURES);

    let fixture_policy: serde_json::Value =
        serde_json::from_str(FIXTURE_POLICY).expect("fixture policy must parse");
    let fixture_sqlx = fixture_policy
        .get("features")
        .and_then(|features| features.get("sqlx"))
        .expect("fixture must declare SQLx policy");
    assert_json_policy("fixture", fixture_sqlx);

    let matrix: serde_json::Value =
        serde_json::from_str(MATRIX_POLICY).expect("matrix policy must parse");
    let matrix_sqlx = matrix
        .get("fixtures")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .find(|fixture| {
            fixture.get("id").and_then(serde_json::Value::as_str)
                == Some("ecosystem_backend_certification")
        })
        .and_then(|fixture| fixture.get("features"))
        .and_then(|features| features.get("sqlx"))
        .expect("matrix must declare SQLx policy");
    assert_json_policy("matrix", matrix_sqlx);
}

#[test]
fn maintained_locks_use_the_official_sqlx_0_9_0_packages() {
    for (label, source) in [
        ("workspace", WORKSPACE_LOCK),
        ("backend fixture", FIXTURE_LOCK),
    ] {
        let lock: toml::Value =
            toml::from_str(source).unwrap_or_else(|error| panic!("{label} lock: {error}"));
        let packages = lock_packages(&lock);
        assert_package(packages, "sqlx", SQLX_PACKAGE_HASH);
        assert_package(packages, "sqlx-core", SQLX_CORE_PACKAGE_HASH);
        assert_package(packages, "sqlx-macros", SQLX_MACROS_PACKAGE_HASH);
        assert_package(packages, "sqlx-macros-core", SQLX_MACROS_CORE_PACKAGE_HASH);
        assert_package(packages, "sqlx-postgres", SQLX_POSTGRES_PACKAGE_HASH);
    }
}

#[test]
fn workspace_catalog_keeps_inactive_database_drivers_out_of_the_shared_lock() {
    let lock: toml::Value = toml::from_str(WORKSPACE_LOCK).expect("workspace lock must parse");
    let packages = lock_packages(&lock);
    for inactive_driver in ["sqlx-mysql", "sqlx-sqlite"] {
        assert!(
            packages
                .iter()
                .all(|package| package_name(package) != Some(inactive_driver)),
            "workspace lock must not contain inactive {inactive_driver}"
        );
    }

    let sqlx = package(packages, "sqlx");
    let edges = dependency_edges(sqlx).collect::<Vec<_>>();
    assert_eq!(
        edges,
        ["sqlx-core", "sqlx-macros", "sqlx-postgres"],
        "workspace SQLx lock edges"
    );
}

fn dependency(source: &str, label: &str) -> toml::Value {
    let manifest: toml::Value =
        toml::from_str(source).unwrap_or_else(|error| panic!("{label} must parse: {error}"));
    manifest
        .get("workspace")
        .and_then(|workspace| workspace.get("dependencies"))
        .or_else(|| manifest.get("dependencies"))
        .and_then(|dependencies| dependencies.get("sqlx"))
        .unwrap_or_else(|| panic!("{label} must declare SQLx"))
        .clone()
}

fn assert_dependency(label: &str, dependency: &toml::Value, expected_features: &[&str]) {
    assert_eq!(
        dependency.get("version").and_then(toml::Value::as_str),
        Some("=0.9.0"),
        "{label} SQLx version"
    );
    assert_eq!(
        dependency
            .get("default-features")
            .and_then(toml::Value::as_bool),
        Some(false),
        "{label} must disable SQLx default features"
    );
    let features = dependency
        .get("features")
        .and_then(toml::Value::as_array)
        .expect("SQLx features must be an array")
        .iter()
        .filter_map(toml::Value::as_str)
        .collect::<Vec<_>>();
    assert_eq!(features, expected_features, "{label} SQLx features");
    for legacy in ["runtime-tokio-rustls", "tls-rustls", "tls-rustls-ring"] {
        assert!(
            !features.contains(&legacy),
            "{label} must not use legacy feature {legacy}"
        );
    }
}

fn assert_json_policy(label: &str, policy: &serde_json::Value) {
    assert_eq!(
        policy
            .get("default_features")
            .and_then(serde_json::Value::as_bool),
        Some(false),
        "{label} must disable SQLx default features"
    );
    let features = policy
        .get("features")
        .and_then(serde_json::Value::as_array)
        .expect("SQLx policy features must be an array")
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    assert_eq!(features, FIXTURE_FEATURES, "{label} SQLx features");
}

fn lock_packages(lock: &toml::Value) -> &[toml::Value] {
    lock.get("package")
        .and_then(toml::Value::as_array)
        .expect("lock packages must be an array")
}

fn assert_package(packages: &[toml::Value], name: &str, checksum: &str) {
    let matching = packages
        .iter()
        .filter(|package| {
            package_name(package) == Some(name)
                && package.get("version").and_then(toml::Value::as_str) == Some(SQLX_VERSION)
        })
        .collect::<Vec<_>>();
    assert_eq!(matching.len(), 1, "lock must contain one {name} 0.9.0");
    assert_eq!(
        matching[0].get("checksum").and_then(toml::Value::as_str),
        Some(checksum),
        "{name} checksum"
    );
}

fn package<'a>(packages: &'a [toml::Value], name: &str) -> &'a toml::Value {
    packages
        .iter()
        .find(|package| {
            package_name(package) == Some(name)
                && package.get("version").and_then(toml::Value::as_str) == Some(SQLX_VERSION)
        })
        .unwrap_or_else(|| panic!("lock must contain {name} {SQLX_VERSION}"))
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
