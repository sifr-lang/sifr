mod support;

use support::TestUnwrap as _;

const CATALOG_MANIFEST: &str = include_str!("../../sifr_rust_interop_catalog/Cargo.toml");
const ASYNC_MANIFEST: &str = include_str!(
    "../../../verification/areas/rust_interop/fixtures/async_runtime_reqwest/examples/reqwest_loopback_runtime/Cargo.toml"
);
const OPAQUE_MANIFEST: &str = include_str!(
    "../../../verification/areas/rust_interop/fixtures/opaque_resource_matrix/examples/resource_lifecycle_runtime/Cargo.toml"
);
const DEMO_HTTP_MANIFEST: &str = include_str!(
    "../../../verification/areas/package_management/corpora/demo_repositories/sifr-demo-http/Cargo.toml"
);

const WORKSPACE_LOCK: &str = include_str!("../../../Cargo.lock");
const ASYNC_LOCK: &str = include_str!(
    "../../../verification/areas/rust_interop/fixtures/async_runtime_reqwest/examples/reqwest_loopback_runtime/Cargo.lock"
);
const OPAQUE_LOCK: &str = include_str!(
    "../../../verification/areas/rust_interop/fixtures/opaque_resource_matrix/examples/resource_lifecycle_runtime/Cargo.lock"
);
const DEMO_HTTP_LOCK: &str = include_str!(
    "../../../verification/areas/package_management/corpora/demo_repositories/sifr-demo-http/Cargo.lock"
);
const DEMO_APP_LOCK: &str = include_str!(
    "../../../verification/areas/package_management/corpora/demo_repositories/sifr-demo-app/Cargo.lock"
);

const VENDOR_MANIFEST: &str = include_str!("../../../vendor/reqwest-0.13.4/Cargo.toml");
const VENDOR_CHECKSUM: &str = include_str!("../../../vendor/reqwest-0.13.4/.cargo-checksum.json");
const VENDOR_VCS_INFO: &str = include_str!("../../../vendor/reqwest-0.13.4/.cargo_vcs_info.json");

const REQWEST_VERSION: &str = "0.13.4";
const REQWEST_PACKAGE_HASH: &str =
    "219c5811de6525e5416c7d5d53bb656d3afdbc6c5af816e0802bcfa42dbdc1c3";
const REQWEST_RELEASE_COMMIT: &str = "11489b34eda6d32b15ad4033e62beba2ee401350";

#[test]
fn maintained_reqwest_dependencies_use_the_latest_stable_policy() {
    for (label, source, expected_version) in [
        ("catalog", CATALOG_MANIFEST, "=0.13.4"),
        ("async runtime", ASYNC_MANIFEST, "=0.13.4"),
        ("opaque resources", OPAQUE_MANIFEST, "=0.13.4"),
        ("HTTP demo", DEMO_HTTP_MANIFEST, "0.13"),
    ] {
        let manifest: toml::Value =
            toml::from_str(source).unwrap_or_else(|error| panic!("{label} must parse: {error}"));
        let dependency = manifest
            .get("dependencies")
            .and_then(|dependencies| dependencies.get("reqwest"))
            .unwrap_or_else(|| panic!("{label} must declare reqwest"));
        assert_eq!(
            dependency.get("version").and_then(toml::Value::as_str),
            Some(expected_version),
            "{label} reqwest version"
        );
        assert_eq!(
            dependency
                .get("default-features")
                .and_then(toml::Value::as_bool),
            Some(false),
            "{label} must disable default features"
        );
        let mut features = dependency
            .get("features")
            .and_then(toml::Value::as_array)
            .test_unwrap("reqwest features must be an array")
            .iter()
            .filter_map(toml::Value::as_str)
            .collect::<Vec<_>>();
        features.sort_unstable();
        assert_eq!(features, ["json", "rustls"], "{label} reqwest features");
    }

    let catalog: toml::Value =
        toml::from_str(CATALOG_MANIFEST).test_unwrap("catalog manifest must parse");
    assert_eq!(
        catalog
            .get("dependencies")
            .and_then(|dependencies| dependencies.get("reqwest"))
            .and_then(|dependency| dependency.get("optional"))
            .and_then(toml::Value::as_bool),
        Some(true)
    );
}

#[test]
fn maintained_lock_edges_select_reqwest_0_13_4() {
    let workspace: toml::Value =
        toml::from_str(WORKSPACE_LOCK).test_unwrap("workspace lock must parse");
    let packages = lock_packages(&workspace);
    let reqwest = package(packages, "reqwest", REQWEST_VERSION);
    assert_eq!(
        reqwest.get("checksum").and_then(toml::Value::as_str),
        Some(REQWEST_PACKAGE_HASH)
    );

    for first_party in packages.iter().filter(|package| {
        package_name(package).is_some_and(|name| name.starts_with("sifr"))
            && package.get("source").is_none()
    }) {
        for edge in dependency_edges(first_party).filter(|edge| edge.starts_with("reqwest")) {
            assert_eq!(
                edge,
                "reqwest 0.13.4",
                "{} must use the maintained reqwest line",
                package_name(first_party).test_unwrap("first-party package name")
            );
        }
    }

    for (label, source) in [
        ("async runtime", ASYNC_LOCK),
        ("opaque resources", OPAQUE_LOCK),
        ("HTTP demo", DEMO_HTTP_LOCK),
        ("application demo", DEMO_APP_LOCK),
    ] {
        let lock: toml::Value =
            toml::from_str(source).unwrap_or_else(|error| panic!("{label} lock: {error}"));
        let reqwest = lock_packages(&lock)
            .iter()
            .filter(|package| package_name(package) == Some("reqwest"))
            .collect::<Vec<_>>();
        assert_eq!(reqwest.len(), 1, "{label} must resolve one reqwest line");
        assert_eq!(package_version(reqwest[0]), Some(REQWEST_VERSION));
        assert_eq!(
            reqwest[0].get("checksum").and_then(toml::Value::as_str),
            Some(REQWEST_PACKAGE_HASH)
        );
    }

    for (label, source) in [
        ("async runtime", ASYNC_LOCK),
        ("opaque resources", OPAQUE_LOCK),
    ] {
        let lock: toml::Value =
            toml::from_str(source).unwrap_or_else(|error| panic!("{label} lock: {error}"));
        let packages = lock_packages(&lock);
        package(packages, "aws-lc-sys", "0.44.0");
        package(packages, "rustls-platform-verifier", "0.7.0");
    }
}

#[test]
fn vendor_contains_the_official_reqwest_release_and_provider_policy() {
    let manifest: toml::Value =
        toml::from_str(VENDOR_MANIFEST).test_unwrap("vendor manifest must parse");
    assert_eq!(
        manifest
            .get("package")
            .and_then(|package| package.get("name"))
            .and_then(toml::Value::as_str),
        Some("reqwest")
    );
    assert_eq!(
        manifest
            .get("package")
            .and_then(|package| package.get("version"))
            .and_then(toml::Value::as_str),
        Some(REQWEST_VERSION)
    );
    let rustls_features = manifest
        .get("features")
        .and_then(|features| features.get("rustls"))
        .and_then(toml::Value::as_array)
        .test_unwrap("vendor rustls feature must be an array")
        .iter()
        .filter_map(toml::Value::as_str)
        .collect::<Vec<_>>();
    assert_eq!(
        rustls_features,
        [
            "__rustls-aws-lc-rs",
            "dep:rustls-platform-verifier",
            "__rustls"
        ]
    );

    let checksum: serde_json::Value =
        serde_json::from_str(VENDOR_CHECKSUM).test_unwrap("vendor checksum must parse");
    assert_eq!(
        checksum.get("package").and_then(serde_json::Value::as_str),
        Some(REQWEST_PACKAGE_HASH)
    );

    let vcs: serde_json::Value =
        serde_json::from_str(VENDOR_VCS_INFO).test_unwrap("vendor VCS metadata must parse");
    assert_eq!(
        vcs.get("git")
            .and_then(|git| git.get("sha1"))
            .and_then(serde_json::Value::as_str),
        Some(REQWEST_RELEASE_COMMIT)
    );
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
            package_name(package) == Some(name) && package_version(package) == Some(version)
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

fn package_name(package: &toml::Value) -> Option<&str> {
    package.get("name").and_then(toml::Value::as_str)
}

fn package_version(package: &toml::Value) -> Option<&str> {
    package.get("version").and_then(toml::Value::as_str)
}
