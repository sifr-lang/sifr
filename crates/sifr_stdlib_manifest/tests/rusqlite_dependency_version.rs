mod support;

use support::TestUnwrap as _;

const CATALOG_MANIFEST: &str = include_str!("../../sifr_rust_interop_catalog/Cargo.toml");
const FIXTURE_MANIFEST: &str = include_str!(
    "../../../verification/areas/rust_interop/fixtures/opaque_resource_matrix/examples/resource_lifecycle_runtime/Cargo.toml"
);
const FIXTURE_POLICY: &str = include_str!(
    "../../../verification/areas/rust_interop/fixtures/opaque_resource_matrix/fixture.json"
);
const MATRIX_POLICY: &str =
    include_str!("../../../verification/areas/rust_interop/data/rust_interop_fixture_matrix.json");
const FIXTURE_SOURCE: &str = include_str!(
    "../../../verification/areas/rust_interop/fixtures/opaque_resource_matrix/examples/resource_lifecycle_runtime/src/bridges/resources.rs"
);
const FIXTURE_TRUST: &str = include_str!(
    "../../../verification/areas/rust_interop/fixtures/opaque_resource_matrix/examples/resource_lifecycle_runtime/sifr.toml"
);

const WORKSPACE_LOCK: &str = include_str!("../../../Cargo.lock");
const FIXTURE_LOCK: &str = include_str!(
    "../../../verification/areas/rust_interop/fixtures/opaque_resource_matrix/examples/resource_lifecycle_runtime/Cargo.lock"
);

const RUSQLITE_VERSION: &str = "0.40.2";
const RUSQLITE_PACKAGE_HASH: &str =
    "23f2a97da3e3873c73cb2a2e71b35c40ff95e0b1eefa8d72d8499a6928c3b5b3";
const LIBSQLITE_VERSION: &str = "0.38.2";
const LIBSQLITE_PACKAGE_HASH: &str =
    "f1d20bef17f513b9b3004532233187769cd072d790971f4e4da0e346eb6401e8";

#[test]
fn maintained_rusqlite_dependencies_use_the_latest_stable_policy() {
    for (label, source) in [
        ("catalog", CATALOG_MANIFEST),
        ("opaque resource runtime", FIXTURE_MANIFEST),
    ] {
        let manifest: toml::Value =
            toml::from_str(source).unwrap_or_else(|error| panic!("{label} must parse: {error}"));
        let dependency = manifest
            .get("dependencies")
            .and_then(|dependencies| dependencies.get("rusqlite"))
            .unwrap_or_else(|| panic!("{label} must declare rusqlite"));
        assert_eq!(
            dependency.get("version").and_then(toml::Value::as_str),
            Some("=0.40.2"),
            "{label} rusqlite version"
        );
        assert_eq!(
            dependency
                .get("default-features")
                .and_then(toml::Value::as_bool),
            Some(false),
            "{label} must disable default features"
        );
        assert_eq!(
            dependency
                .get("features")
                .and_then(toml::Value::as_array)
                .test_unwrap("rusqlite features must be an array")
                .iter()
                .filter_map(toml::Value::as_str)
                .collect::<Vec<_>>(),
            ["bundled"],
            "{label} rusqlite features"
        );
    }

    let catalog: toml::Value =
        toml::from_str(CATALOG_MANIFEST).test_unwrap("catalog manifest must parse");
    assert_eq!(
        catalog
            .get("dependencies")
            .and_then(|dependencies| dependencies.get("rusqlite"))
            .and_then(|dependency| dependency.get("optional"))
            .and_then(toml::Value::as_bool),
        Some(true)
    );

    let fixture: serde_json::Value =
        serde_json::from_str(FIXTURE_POLICY).test_unwrap("fixture policy must parse");
    let fixture_policy = fixture
        .get("features")
        .and_then(|features| features.get("rusqlite"))
        .test_unwrap("fixture must declare a rusqlite policy");
    assert_policy("fixture", fixture_policy);

    let matrix: serde_json::Value =
        serde_json::from_str(MATRIX_POLICY).test_unwrap("matrix policy must parse");
    let matrix_policy = matrix
        .get("fixtures")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .find(|fixture| {
            fixture.get("id").and_then(serde_json::Value::as_str) == Some("opaque_resource_matrix")
        })
        .and_then(|fixture| fixture.get("features"))
        .and_then(|features| features.get("rusqlite"))
        .test_unwrap("matrix must declare the opaque resource rusqlite policy");
    assert_policy("matrix", matrix_policy);
}

#[test]
fn maintained_lock_edges_use_rusqlite_0_40_2() {
    for (label, source) in [
        ("workspace", WORKSPACE_LOCK),
        ("opaque resource runtime", FIXTURE_LOCK),
    ] {
        let lock: toml::Value =
            toml::from_str(source).unwrap_or_else(|error| panic!("{label} lock: {error}"));
        let packages = lock_packages(&lock);
        let rusqlite = package(packages, "rusqlite", RUSQLITE_VERSION);
        assert_eq!(
            rusqlite.get("checksum").and_then(toml::Value::as_str),
            Some(RUSQLITE_PACKAGE_HASH)
        );
        let dependencies = dependency_edges(rusqlite).collect::<Vec<_>>();
        assert!(
            dependencies
                .iter()
                .any(|edge| edge.starts_with("libsqlite3-sys"))
        );
        assert!(!dependencies.iter().any(|edge| edge.starts_with("hashlink")));
        assert!(
            !dependencies
                .iter()
                .any(|edge| edge.starts_with("sqlite-wasm-rs"))
        );

        let libsqlite = package(packages, "libsqlite3-sys", LIBSQLITE_VERSION);
        assert_eq!(
            libsqlite.get("checksum").and_then(toml::Value::as_str),
            Some(LIBSQLITE_PACKAGE_HASH)
        );
    }
}

#[test]
fn runtime_certifies_safe_savepoint_names_and_exact_native_trust() {
    assert!(
        FIXTURE_SOURCE.contains("savepoint_with_name(\"sifr; DROP TABLE evidence; --\")"),
        "runtime must exercise the safe named-savepoint implementation"
    );
    assert!(
        FIXTURE_SOURCE.contains("SELECT value FROM evidence"),
        "runtime must prove that the table survives the tainted identifier"
    );

    let trust: toml::Value = toml::from_str(FIXTURE_TRUST).test_unwrap("fixture trust must parse");
    let build_scripts = trust
        .get("trust")
        .and_then(|trust| trust.get("rust-build-scripts"))
        .and_then(toml::Value::as_array)
        .test_unwrap("build-script trust must be an array")
        .iter()
        .filter_map(toml::Value::as_str)
        .collect::<Vec<_>>();
    assert!(build_scripts.contains(&"libsqlite3-sys"));

    let native_links = trust
        .get("trust")
        .and_then(|trust| trust.get("native-links"))
        .and_then(toml::Value::as_array)
        .test_unwrap("native-link trust must be an array")
        .iter()
        .filter_map(toml::Value::as_str)
        .collect::<Vec<_>>();
    assert!(native_links.contains(&"sqlite3"));
}

fn assert_policy(label: &str, policy: &serde_json::Value) {
    assert_eq!(
        policy
            .get("default_features")
            .and_then(serde_json::Value::as_bool),
        Some(false),
        "{label} must disable default features"
    );
    let features = policy
        .get("features")
        .and_then(serde_json::Value::as_array)
        .test_unwrap("rusqlite policy features must be an array")
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    assert_eq!(features, ["bundled"], "{label} rusqlite features");
}

fn lock_packages(lock: &toml::Value) -> &[toml::Value] {
    lock.get("package")
        .and_then(toml::Value::as_array)
        .test_unwrap("lock packages must be an array")
}

fn package<'a>(packages: &'a [toml::Value], name: &str, version: &str) -> &'a toml::Value {
    packages
        .iter()
        .find(|package| {
            package.get("name").and_then(toml::Value::as_str) == Some(name)
                && package.get("version").and_then(toml::Value::as_str) == Some(version)
        })
        .unwrap_or_else(|| panic!("lock must contain {name} {version}"))
}

fn dependency_edges(package: &toml::Value) -> impl Iterator<Item = &str> {
    package
        .get("dependencies")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_str)
}
