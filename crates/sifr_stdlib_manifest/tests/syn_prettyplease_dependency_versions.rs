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

use support::TestUnwrap as _;

const WORKSPACE_MANIFEST: &str = include_str!("../../../Cargo.toml");
const CODEGEN_MANIFEST: &str = include_str!("../../sifr_codegen/Cargo.toml");
const WORKSPACE_LOCK: &str = include_str!("../../../Cargo.lock");

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
        "3.0.5",
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
        Some("3.0.5")
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
    // Canonicalization prints Syn nodes through ToTokens and mutates the AST.
    assert_eq!(
        string_array(codegen_syn, "features"),
        ["printing", "visit-mut"]
    );
    assert_eq!(
        dependencies.get("quote").and_then(toml::Value::as_str),
        Some("1.0.47")
    );
    let lock = parse_lock(WORKSPACE_LOCK);
    let quote = lock_packages(&lock)
        .iter()
        .find(|package| package_name(package) == Some("quote"))
        .test_unwrap("workspace must lock Quote");
    assert_eq!(package_version(quote), Some("1.0.47"));
    assert_eq!(
        quote.get("checksum").and_then(toml::Value::as_str),
        Some("1fbf4db142a473a8d80c26bbf18454ed458bf8d26c8219c331daecfdbd079001")
    );
    assert_eq!(
        dependencies
            .get("prettyplease")
            .and_then(toml::Value::as_str),
        Some("0.3.0")
    );
}

#[test]
fn first_party_edges_and_maintained_locks_use_current_syn_3() {
    cargo_edges::first_party_edges("syn", "3.0.5");
    cargo_edges::first_party_edges("prettyplease", "0.3.0");
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

#[test]
fn syntax_vendor_files_and_exact_selected_versions_are_authenticated() {
    let lock = parse_lock(WORKSPACE_LOCK);
    for (name, versions) in [
        ("syn", ["2.0.117", "3.0.5"]),
        ("prettyplease", ["0.2.37", "0.3.0"]),
    ] {
        cargo_vendor::family(
            &cargo_inventory::root(),
            name,
            &versions,
            lock_packages(&lock),
        )
        .test_unwrap("syntax vendor closure");
    }
}

#[test]
fn syntax_owner_rejects_an_extra_old_direct_edge() {
    let mut lock = parse_lock(WORKSPACE_LOCK);
    let package = lock["package"]
        .as_array_mut()
        .test_unwrap("packages")
        .iter_mut()
        .find(|package| package_name(package) == Some("sifr_codegen"))
        .test_unwrap("codegen");
    package["dependencies"]
        .as_array_mut()
        .test_unwrap("dependencies")
        .push(toml::Value::String("prettyplease 0.2.37".into()));
    let packages = lock_packages(&lock);
    let owner = packages
        .iter()
        .find(|package| package_name(package) == Some("sifr_codegen"))
        .test_unwrap("owner");
    let results = cargo_edges::edges(owner)
        .test_unwrap("edges")
        .into_iter()
        .filter(|edge| edge.split_whitespace().next() == Some("prettyplease"))
        .map(|edge| cargo_edges::current_edge(packages, edge, "prettyplease", "0.3.0"))
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 2);
    assert!(results.iter().any(Result::is_err));
}
