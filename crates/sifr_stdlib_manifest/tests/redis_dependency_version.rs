mod support;

use support::TestUnwrap as _;

const REDIS_VERSION: &str = "1.6.0";
const REDIS_PACKAGE_HASH: &str = "e37a4ca5c6ca42aa3e6df2fd32b987a65d32a4c2159a6f3fe0fd1df306a2658f";

const CATALOG_MANIFEST: &str = include_str!("../../sifr_rust_interop_catalog/Cargo.toml");
const RESOURCE_MANIFEST: &str = include_str!(
    "../../../verification/areas/rust_interop/fixtures/opaque_resource_matrix/examples/resource_lifecycle_runtime/Cargo.toml"
);
const SUBSCRIPTION_MANIFEST: &str = include_str!(
    "../../../verification/areas/rust_interop/fixtures/callback_subscription_ecosystem/examples/subscription_lifecycle_runtime/Cargo.toml"
);
const RESOURCE_BRIDGE: &str = include_str!(
    "../../../verification/areas/rust_interop/fixtures/opaque_resource_matrix/examples/resource_lifecycle_runtime/src/bridges/resources.rs"
);
const WORKSPACE_LOCK: &str = include_str!("../../../Cargo.lock");
const RESOURCE_LOCK: &str = include_str!(
    "../../../verification/areas/rust_interop/fixtures/opaque_resource_matrix/examples/resource_lifecycle_runtime/Cargo.lock"
);
const SUBSCRIPTION_LOCK: &str = include_str!(
    "../../../verification/areas/rust_interop/fixtures/callback_subscription_ecosystem/examples/subscription_lifecycle_runtime/Cargo.lock"
);

#[test]
fn maintained_manifests_select_latest_stable_redis() {
    assert_dependency(CATALOG_MANIFEST, &["connection-manager", "tokio-comp"]);
    assert_dependency(RESOURCE_MANIFEST, &["connection-manager", "tokio-comp"]);
    assert_dependency(SUBSCRIPTION_MANIFEST, &["connection-manager", "tokio-comp"]);
}

#[test]
fn maintained_locks_use_current_redis_with_registry_checksum() {
    for (label, source) in [
        ("workspace", WORKSPACE_LOCK),
        ("resource fixture", RESOURCE_LOCK),
        ("subscription fixture", SUBSCRIPTION_LOCK),
    ] {
        let lock: toml::Value =
            toml::from_str(source).unwrap_or_else(|error| panic!("{label} lock: {error}"));
        let packages = lock["package"]
            .as_array()
            .test_unwrap("lock packages must be an array");
        let matching = packages
            .iter()
            .filter(|package| package["name"].as_str() == Some("redis"))
            .collect::<Vec<_>>();
        assert_eq!(matching.len(), 1, "{label} must contain one Redis package");
        assert_eq!(
            matching[0]["version"].as_str(),
            Some(REDIS_VERSION),
            "{label} Redis version"
        );
        assert_eq!(
            matching[0]["checksum"].as_str(),
            Some(REDIS_PACKAGE_HASH),
            "{label} Redis checksum"
        );
    }
}

#[test]
fn resource_bridge_uses_redis_1_6_bounded_reconnect_attempts() {
    assert!(RESOURCE_BRIDGE.contains("redis: Option<redis::aio::ConnectionManager>"));
    assert!(RESOURCE_BRIDGE.contains("const REDIS_RECONNECT_ATTEMPTS: usize = 2;"));
    assert!(RESOURCE_BRIDGE.contains(".set_number_of_retries(REDIS_RECONNECT_ATTEMPTS)"));
    assert!(RESOURCE_BRIDGE.contains(".get_connection_manager_with_config(redis_config)"));
    assert!(
        RESOURCE_BRIDGE
            .contains("redis={redis}/retries={REDIS_RECONNECT_ATTEMPTS};postgres={postgres}")
    );
}

fn assert_dependency(source: &str, expected_features: &[&str]) {
    let manifest: toml::Value = toml::from_str(source).test_unwrap("manifest must parse");
    let dependency = manifest["dependencies"]
        .get("redis")
        .test_unwrap("manifest must declare Redis");
    assert_eq!(dependency["version"].as_str(), Some("=1.6.0"));
    assert_eq!(dependency["default-features"].as_bool(), Some(false));
    let features = dependency["features"]
        .as_array()
        .test_unwrap("Redis features must be an array")
        .iter()
        .map(|feature| feature.as_str().test_unwrap("feature must be a string"))
        .collect::<Vec<_>>();
    assert_eq!(features, expected_features);
}
