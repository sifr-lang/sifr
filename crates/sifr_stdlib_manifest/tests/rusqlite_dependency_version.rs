#[allow(dead_code)]
#[path = "support/cargo_edges.rs"]
mod cargo_edges;
#[allow(dead_code)]
#[path = "support/cargo_inventory.rs"]
mod cargo_inventory;
mod support;

use support::TestUnwrap as _;

const CATALOG_MANIFEST: &str = include_str!("../../sifr_rust_interop_catalog/Cargo.toml");
const SQLITE_RUNTIME_MANIFEST: &str = include_str!("../../sifr_sql_sqlite_runtime/Cargo.toml");
const SQL_LOCK_MANIFEST: &str = include_str!("../../sifr_sql_dependency_lock/Cargo.toml");
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
    // Incoming dependency ownership complements the existing feature-context checks.
    let owners = cargo_edges::first_party_edges("rusqlite", RUSQLITE_VERSION);
    assert_eq!(owners.len(), 6);
    assert!(
        owners
            .iter()
            .any(|(_, owner)| owner == "resource-lifecycle-runtime")
    );
    // Cargo unifies the workspace's explicit cache requests. The standalone
    // resource fixture disables defaults and requests only bundled SQLite.
    for manifest in [SQLITE_RUNTIME_MANIFEST, SQL_LOCK_MANIFEST] {
        let manifest: toml::Value = toml::from_str(manifest).test_unwrap("manifest must parse");
        let dependency = &manifest["dependencies"]["rusqlite"];
        assert_eq!(dependency["workspace"].as_bool(), Some(true));
        assert!(
            dependency["features"]
                .as_array()
                .test_unwrap("rusqlite features must be an array")
                .iter()
                .any(|feature| feature.as_str() == Some("cache")),
            "workspace lock context must explicitly enable rusqlite cache"
        );
    }

    for context in [LockContext::WorkspaceCache, LockContext::BundledFixture] {
        let (label, source) = context.lock();
        let lock: toml::Value =
            toml::from_str(source).unwrap_or_else(|error| panic!("{label} lock: {error}"));
        let packages = lock_packages(&lock);
        let rusqlite = package(packages, "rusqlite", RUSQLITE_VERSION);
        assert_eq!(
            rusqlite.get("checksum").and_then(toml::Value::as_str),
            Some(RUSQLITE_PACKAGE_HASH)
        );
        check_dependency_edges(rusqlite, context)
            .unwrap_or_else(|error| panic!("{label}: {error}"));

        let libsqlite = package(packages, "libsqlite3-sys", LIBSQLITE_VERSION);
        assert_eq!(
            libsqlite.get("checksum").and_then(toml::Value::as_str),
            Some(LIBSQLITE_PACKAGE_HASH)
        );
    }
}

#[test]
fn lock_edge_contract_accepts_both_feature_contexts() {
    for context in [LockContext::WorkspaceCache, LockContext::BundledFixture] {
        let (_, source) = context.lock();
        let lock: toml::Value = toml::from_str(source).test_unwrap("lock must parse");
        let rusqlite = package(lock_packages(&lock), "rusqlite", RUSQLITE_VERSION);
        assert_eq!(check_dependency_edges(rusqlite, context), Ok(()));
        let other_context = match context {
            LockContext::WorkspaceCache => LockContext::BundledFixture,
            LockContext::BundledFixture => LockContext::WorkspaceCache,
        };
        assert!(check_dependency_edges(rusqlite, other_context).is_err());
    }
}

#[test]
fn lock_edge_contract_rejects_each_missing_required_edge() {
    for context in [LockContext::WorkspaceCache, LockContext::BundledFixture] {
        let (_, source) = context.lock();
        let lock: toml::Value = toml::from_str(source).test_unwrap("lock must parse");
        let rusqlite = package(lock_packages(&lock), "rusqlite", RUSQLITE_VERSION);
        for missing in context.expected_edges() {
            let mut mutated = rusqlite.clone();
            mutated["dependencies"]
                .as_array_mut()
                .test_unwrap("dependencies must be an array")
                .retain(|edge| edge.as_str() != Some(missing));
            assert!(
                check_dependency_edges(&mutated, context).is_err(),
                "{context:?} must reject missing {missing}"
            );
        }
    }
}

#[test]
fn lock_edge_contract_rejects_unexpected_duplicate_and_malformed_edges() {
    for context in [LockContext::WorkspaceCache, LockContext::BundledFixture] {
        let (_, source) = context.lock();
        let lock: toml::Value = toml::from_str(source).test_unwrap("lock must parse");
        let rusqlite = package(lock_packages(&lock), "rusqlite", RUSQLITE_VERSION);
        for extra in ["hashlink 0.12.1", "sqlite-wasm-rs", "libsqlite3-sys-extra"] {
            let mut mutated = rusqlite.clone();
            mutated["dependencies"]
                .as_array_mut()
                .test_unwrap("dependencies must be an array")
                .push(toml::Value::String(extra.to_owned()));
            assert!(
                check_dependency_edges(&mutated, context).is_err(),
                "{context:?} must reject extra {extra}"
            );
        }
        let mut malformed = rusqlite.clone();
        malformed["dependencies"]
            .as_array_mut()
            .test_unwrap("dependencies must be an array")
            .push(toml::Value::Integer(42));
        assert!(check_dependency_edges(&malformed, context).is_err());
        malformed["dependencies"] = toml::Value::String("libsqlite3-sys".to_owned());
        assert!(check_dependency_edges(&malformed, context).is_err());
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

#[derive(Clone, Copy, Debug)]
enum LockContext {
    WorkspaceCache,
    BundledFixture,
}

impl LockContext {
    fn lock(self) -> (&'static str, &'static str) {
        match self {
            Self::WorkspaceCache => ("workspace", WORKSPACE_LOCK),
            Self::BundledFixture => ("opaque resource runtime", FIXTURE_LOCK),
        }
    }

    fn expected_edges(self) -> &'static [&'static str] {
        // Keep Cargo's version qualifiers: these roots resolve different
        // package graphs, and only the workspace enables the cache feature.
        match self {
            Self::WorkspaceCache => &[
                "bitflags 2.13.1",
                "fallible-iterator 0.3.0",
                "fallible-streaming-iterator",
                "hashlink 0.12.1",
                "libsqlite3-sys",
                "smallvec",
            ],
            Self::BundledFixture => &[
                "bitflags",
                "fallible-iterator 0.3.0",
                "fallible-streaming-iterator",
                "libsqlite3-sys",
                "smallvec",
            ],
        }
    }
}

fn check_dependency_edges(package: &toml::Value, context: LockContext) -> Result<(), String> {
    let dependencies = package
        .get("dependencies")
        .and_then(toml::Value::as_array)
        .ok_or("rusqlite dependencies must be an array")?;
    let mut edges = dependencies
        .iter()
        .map(|edge| edge.as_str().ok_or("dependency edge must be a string"))
        .collect::<Result<Vec<_>, _>>()?;
    edges.sort_unstable();
    let expected = context.expected_edges();
    if edges != expected {
        return Err(format!(
            "{context:?} rusqlite edges: expected {expected:?}, got {edges:?}"
        ));
    }
    Ok(())
}

#[test]
fn rusqlite_incoming_edge_rejects_missing_target_wrong_source_and_wrong_version() {
    let lock: toml::Value = toml::from_str(FIXTURE_LOCK).test_unwrap("fixture lock");
    let packages = cargo_edges::packages(&lock).test_unwrap("packages");
    let current = cargo_edges::resolve(packages, "rusqlite").test_unwrap("Rusqlite");
    for (field, value) in [
        ("version", "0.39.0"),
        ("source", "git+https://example.invalid/rusqlite"),
    ] {
        let mut changed = current.clone();
        changed[field] = value.into();
        assert!(
            cargo_edges::current_edge(&[changed], "rusqlite", "rusqlite", RUSQLITE_VERSION)
                .is_err()
        );
    }
    assert!(cargo_edges::current_edge(&[], "rusqlite", "rusqlite", RUSQLITE_VERSION).is_err());
}
