mod support;

use support::TestUnwrap as _;

const WORKSPACE_MANIFEST: &str = include_str!("../../../Cargo.toml");
const CODEGEN_MANIFEST: &str = include_str!("../../sifr_codegen/Cargo.toml");
const WORKSPACE_LOCK: &str = include_str!("../../../Cargo.lock");

const MAINTAINED_LOCKS: &[(&str, &str)] = &[
    ("Cargo.lock", WORKSPACE_LOCK),
    (
        "advanced_data_runtime",
        include_str!(
            "../../../verification/areas/rust_interop/fixtures/advanced_data_runtime_matrix/examples/advanced_data_runtime/Cargo.lock"
        ),
    ),
    (
        "async_runtime_reqwest",
        include_str!(
            "../../../verification/areas/rust_interop/fixtures/async_runtime_reqwest/examples/reqwest_loopback_runtime/Cargo.lock"
        ),
    ),
    (
        "bridge_type_matrix",
        include_str!(
            "../../../verification/areas/rust_interop/fixtures/bridge_type_matrix/examples/bridge_type_roundtrip/Cargo.lock"
        ),
    ),
    (
        "callback_subscription_ecosystem",
        include_str!(
            "../../../verification/areas/rust_interop/fixtures/callback_subscription_ecosystem/examples/subscription_lifecycle_runtime/Cargo.lock"
        ),
    ),
    (
        "ecosystem_backend_certification",
        include_str!(
            "../../../verification/areas/rust_interop/fixtures/ecosystem_backend_certification/examples/backend_feature_package/Cargo.lock"
        ),
    ),
    (
        "native_build_script",
        include_str!(
            "../../../verification/areas/rust_interop/fixtures/native_build_script/examples/native_trust_package/Cargo.lock"
        ),
    ),
    (
        "opaque_resource_matrix",
        include_str!(
            "../../../verification/areas/rust_interop/fixtures/opaque_resource_matrix/examples/resource_lifecycle_runtime/Cargo.lock"
        ),
    ),
    (
        "proc_macro_trust",
        include_str!(
            "../../../verification/areas/rust_interop/fixtures/proc_macro_trust/examples/proc_macro_trust_package/Cargo.lock"
        ),
    ),
];

const VENDORED_RELEASES: &[(&str, &str, &str, &str)] = &[
    (
        "prettyplease",
        "0.3.0",
        include_str!("../../../vendor/prettyplease/Cargo.toml"),
        include_str!("../../../vendor/prettyplease/.cargo-checksum.json"),
    ),
    (
        "prettyplease",
        "0.2.37",
        include_str!("../../../vendor/prettyplease-0.2.37/Cargo.toml"),
        include_str!("../../../vendor/prettyplease-0.2.37/.cargo-checksum.json"),
    ),
    (
        "syn",
        "3.0.4",
        include_str!("../../../vendor/syn/Cargo.toml"),
        include_str!("../../../vendor/syn/.cargo-checksum.json"),
    ),
    (
        "syn",
        "2.0.117",
        include_str!("../../../vendor/syn-2.0.117/Cargo.toml"),
        include_str!("../../../vendor/syn-2.0.117/.cargo-checksum.json"),
    ),
];

#[test]
fn direct_syntax_dependencies_use_the_latest_stable_unit() {
    let workspace: toml::Value =
        toml::from_str(WORKSPACE_MANIFEST).test_unwrap("workspace manifest must parse");
    let syn = workspace
        .get("workspace")
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(|dependencies| dependencies.get("syn"))
        .test_unwrap("workspace must declare Syn");
    assert_eq!(
        syn.get("version").and_then(toml::Value::as_str),
        Some("3.0.4")
    );
    assert_eq!(
        string_array(syn, "features"),
        [
            "clone-impls".to_string(),
            "full".to_string(),
            "parsing".to_string(),
            "visit".to_string(),
        ]
    );
    assert_eq!(
        syn.get("default-features").and_then(toml::Value::as_bool),
        Some(false)
    );

    let codegen: toml::Value =
        toml::from_str(CODEGEN_MANIFEST).test_unwrap("codegen manifest must parse");
    let dependencies = codegen
        .get("dependencies")
        .test_unwrap("codegen must have dependencies");
    let codegen_syn = dependencies.get("syn").test_unwrap("codegen must use Syn");
    assert_eq!(
        codegen_syn.get("workspace").and_then(toml::Value::as_bool),
        Some(true)
    );
    assert_eq!(string_array(codegen_syn, "features"), ["visit-mut"]);
    assert_eq!(
        dependencies
            .get("prettyplease")
            .and_then(toml::Value::as_str),
        Some("0.3.0")
    );
}

#[test]
fn first_party_edges_and_maintained_locks_use_syn_3_0_4() {
    let lock = parse_lock(WORKSPACE_LOCK);
    let packages = lock_packages(&lock);
    assert_package_edges(
        packages,
        "sifr_codegen",
        &["prettyplease 0.3.0", "syn 3.0.4"],
    );
    assert_package_edges(packages, "sifr_driver", &["syn 3.0.4"]);

    let mut checked = 0;
    for (name, source) in MAINTAINED_LOCKS {
        let lock = parse_lock(source);
        let syn_3_versions = lock_packages(&lock)
            .iter()
            .filter(|package| package_name(package) == Some("syn"))
            .filter_map(package_version)
            .filter(|version| version.starts_with("3."))
            .collect::<Vec<_>>();
        if syn_3_versions.is_empty() {
            continue;
        }
        checked += 1;
        assert_eq!(
            syn_3_versions,
            ["3.0.4"],
            "{name} must contain only the current Syn 3 release"
        );
    }
    assert_eq!(checked, MAINTAINED_LOCKS.len());
}

#[test]
fn vendor_contains_each_required_syntax_release_with_registry_hashes() {
    let lock = parse_lock(WORKSPACE_LOCK);
    let packages = lock_packages(&lock);
    for (name, version, manifest_source, checksum_source) in VENDORED_RELEASES {
        let manifest: toml::Value =
            toml::from_str(manifest_source).test_unwrap("vendored manifest must parse");
        assert_eq!(
            manifest
                .get("package")
                .and_then(|package| package.get("name"))
                .and_then(toml::Value::as_str),
            Some(*name)
        );
        assert_eq!(
            manifest
                .get("package")
                .and_then(|package| package.get("version"))
                .and_then(toml::Value::as_str),
            Some(*version)
        );

        let lock_checksum = packages
            .iter()
            .find(|package| {
                package_name(package) == Some(*name) && package_version(package) == Some(*version)
            })
            .and_then(|package| package.get("checksum"))
            .and_then(toml::Value::as_str)
            .test_unwrap("workspace lock must contain the vendored release");
        let checksum: serde_json::Value =
            serde_json::from_str(checksum_source).test_unwrap("vendor checksum must parse");
        assert_eq!(
            checksum.get("package").and_then(serde_json::Value::as_str),
            Some(lock_checksum)
        );
    }
}

fn string_array(value: &toml::Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_str)
        .map(str::to_string)
        .collect()
}

fn parse_lock(source: &str) -> toml::Value {
    toml::from_str(source).test_unwrap("Cargo.lock must parse")
}

fn lock_packages(lock: &toml::Value) -> &[toml::Value] {
    lock.get("package")
        .and_then(toml::Value::as_array)
        .test_unwrap("Cargo.lock packages must be an array")
}

fn package_name(package: &toml::Value) -> Option<&str> {
    package.get("name").and_then(toml::Value::as_str)
}

fn package_version(package: &toml::Value) -> Option<&str> {
    package.get("version").and_then(toml::Value::as_str)
}

fn assert_package_edges(packages: &[toml::Value], name: &str, expected: &[&str]) {
    let package = packages
        .iter()
        .find(|package| package_name(package) == Some(name))
        .test_unwrap("first-party package must exist in Cargo.lock");
    let dependencies = package
        .get("dependencies")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_str)
        .collect::<Vec<_>>();
    for edge in expected {
        assert!(
            dependencies.contains(edge),
            "{name} must contain lock edge {edge}: {dependencies:?}"
        );
    }
}
