#[allow(dead_code)]
#[path = "support/cargo_edges.rs"]
mod cargo_edges;
#[allow(dead_code)]
#[path = "support/cargo_inventory.rs"]
mod cargo_inventory;
#[allow(dead_code)]
#[path = "support/cargo_vendor.rs"]
mod cargo_vendor;
mod support;

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use support::TestUnwrap as _;

const WORKSPACE_MANIFEST: &str = include_str!("../../../Cargo.toml");
const WORKSPACE_LOCK: &str = include_str!("../../../Cargo.lock");
const STDLIB_MANIFEST: &str = include_str!("../../sifr_stdlib/Cargo.toml");
const VENDORED_MANIFEST: &str = include_str!("../../../vendor/base64/Cargo.toml");

#[test]
fn base64_direct_dependency_uses_latest_stable_safe_features() {
    let workspace: toml::Value =
        toml::from_str(WORKSPACE_MANIFEST).test_unwrap("workspace manifest must parse");
    let dependency = workspace
        .get("workspace")
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(|dependencies| dependencies.get("base64"))
        .test_unwrap("workspace must declare base64");

    assert_eq!(
        dependency.get("version").and_then(toml::Value::as_str),
        Some("0.23.1")
    );
    assert_eq!(
        dependency
            .get("default-features")
            .and_then(toml::Value::as_bool),
        Some(false),
        "base64 0.23 enables unsafe SIMD by default"
    );
    assert_eq!(
        dependency
            .get("features")
            .and_then(toml::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(toml::Value::as_str)
            .collect::<Vec<_>>(),
        ["std"]
    );

    let stdlib: toml::Value =
        toml::from_str(STDLIB_MANIFEST).test_unwrap("stdlib manifest must parse");
    let stdlib_dependency = stdlib
        .get("dependencies")
        .and_then(|dependencies| dependencies.get("base64"))
        .test_unwrap("stdlib must inherit base64");
    assert_eq!(
        stdlib_dependency
            .get("workspace")
            .and_then(toml::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        stdlib_dependency
            .get("optional")
            .and_then(toml::Value::as_bool),
        Some(true)
    );

    let vendored: toml::Value =
        toml::from_str(VENDORED_MANIFEST).test_unwrap("vendored base64 manifest must parse");
    assert_eq!(
        vendored
            .get("package")
            .and_then(|package| package.get("version"))
            .and_then(toml::Value::as_str),
        Some("0.23.1")
    );
}

#[test]
fn first_party_lock_edges_use_base64_0_23_1() {
    cargo_edges::first_party_edges("base64", "0.23.1");
}

#[test]
fn vendored_base64_packages_cover_vendored_and_first_party_lock_edges() {
    let lock: toml::Value = toml::from_str(WORKSPACE_LOCK).test_unwrap("workspace lock must parse");
    let packages = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .test_unwrap("workspace lock packages must be an array");
    let vendor_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor");
    let mut vendored_packages = BTreeSet::new();
    for entry in fs::read_dir(&vendor_root).test_unwrap("vendor directory must be readable") {
        let entry = entry.test_unwrap("vendor entry must be readable");
        if !entry
            .file_type()
            .test_unwrap("vendor entry type must be readable")
            .is_dir()
        {
            continue;
        }
        let manifest_path = entry.path().join("Cargo.toml");
        if !manifest_path.is_file() {
            continue;
        }
        let manifest = fs::read_to_string(&manifest_path).unwrap_or_else(|error| {
            panic!(
                "vendored manifest {} must be readable: {error}",
                manifest_path.display()
            )
        });
        let manifest: toml::Value = toml::from_str(&manifest).unwrap_or_else(|error| {
            panic!(
                "vendored manifest {} must parse: {error}",
                manifest_path.display()
            )
        });
        let package = manifest
            .get("package")
            .test_unwrap("vendored manifest must contain package metadata");
        let name = package
            .get("name")
            .and_then(toml::Value::as_str)
            .test_unwrap("vendored package must have a string name");
        let version = package
            .get("version")
            .and_then(toml::Value::as_str)
            .test_unwrap("vendored package must have a string version");
        vendored_packages.insert((name.to_string(), version.to_string()));
    }

    let owners = cargo_inventory::local_owners();
    let mut required_base64_versions = BTreeSet::new();
    for package in packages {
        let Some(name) = package.get("name").and_then(toml::Value::as_str) else {
            continue;
        };
        let Some(version) = package.get("version").and_then(toml::Value::as_str) else {
            continue;
        };
        let is_owned_package = package.get("source").is_none()
            && owners.contains_key(&(name.to_owned(), version.to_owned()));
        let is_vendored_package =
            vendored_packages.contains(&(name.to_string(), version.to_string()));
        if !is_owned_package && !is_vendored_package {
            continue;
        }

        for dependency in package
            .get("dependencies")
            .and_then(toml::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(toml::Value::as_str)
        {
            let mut components = dependency.split_whitespace();
            if components.next() != Some("base64") {
                continue;
            }
            let selected =
                cargo_edges::resolve(packages, dependency).test_unwrap("Base64 target identity");
            assert_eq!(selected["source"].as_str(), Some(cargo_inventory::REGISTRY));
            let dependency_version = selected["version"].as_str().test_unwrap("Base64 version");
            required_base64_versions.insert(dependency_version);
        }
    }

    assert_eq!(required_base64_versions, ["0.22.1", "0.23.1"].into());
    for version in required_base64_versions {
        assert!(
            vendored_packages.contains(&("base64".to_string(), version.to_string())),
            "vendor must contain the Base64 {version} package required by a vendored or first-party lock edge"
        );
    }
}

#[test]
fn base64_vendor_rejects_extra_versioned_directories_and_authenticates_files() {
    let lock: toml::Value = toml::from_str(WORKSPACE_LOCK).test_unwrap("lock");
    let packages = cargo_edges::packages(&lock).test_unwrap("packages");
    cargo_vendor::family(
        &cargo_inventory::root(),
        "base64",
        &["0.22.1", "0.23.1"],
        packages,
    )
    .test_unwrap("Base64 vendor closure");

    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .test_unwrap("clock")
        .as_nanos();
    let temporary =
        std::env::temp_dir().join(format!("sifr-base64-vendor-{}-{nonce}", std::process::id()));
    let stale = temporary.join("vendor/base64-0.1.0");
    fs::create_dir_all(&stale).test_unwrap("isolated vendor fixture");
    fs::write(
        stale.join("Cargo.toml"),
        "[package]\nname = \"base64\"\nversion = \"0.1.0\"\n",
    )
    .test_unwrap("stale manifest");
    let result = cargo_vendor::family(&temporary, "base64", &["0.22.1", "0.23.1"], packages);
    fs::remove_dir_all(&temporary).test_unwrap("remove owned synthetic fixture");
    assert!(result.is_err_and(|error| error.contains("stale or duplicate vendor identity")));
}
