#![allow(clippy::expect_used)]

const WORKSPACE_MANIFEST: &str = include_str!("../../../Cargo.toml");
const LSP_MANIFEST: &str = include_str!("../../sifr_lsp/Cargo.toml");
const WORKSPACE_LOCK: &str = include_str!("../../../Cargo.lock");
const VENDOR_MANIFEST: &str = include_str!("../../../vendor/lsp-server/Cargo.toml");
const VENDOR_CHECKSUM: &str = include_str!("../../../vendor/lsp-server/.cargo-checksum.json");

const LSP_SERVER_VERSION: &str = "0.10.0";
const LSP_SERVER_PACKAGE_HASH: &str =
    "3ee25a31f2e571e426eef2896179450cafc7e2f5be00d8a93b1c2d21c0ff7656";

#[test]
fn direct_lsp_server_dependency_uses_the_latest_stable_release() {
    let workspace: toml::Value =
        toml::from_str(WORKSPACE_MANIFEST).expect("workspace manifest must parse");
    let dependency = workspace
        .get("workspace")
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(|dependencies| dependencies.get("lsp-server"))
        .expect("workspace must declare LSP Server");
    assert_eq!(
        dependency.get("version").and_then(toml::Value::as_str),
        Some(LSP_SERVER_VERSION)
    );

    let lsp: toml::Value = toml::from_str(LSP_MANIFEST).expect("LSP manifest must parse");
    assert_eq!(
        lsp.get("dependencies")
            .and_then(|dependencies| dependencies.get("lsp-server"))
            .and_then(|dependency| dependency.get("workspace"))
            .and_then(toml::Value::as_bool),
        Some(true)
    );
}

#[test]
fn first_party_lock_edge_uses_only_lsp_server_0_10_0() {
    let lock: toml::Value = toml::from_str(WORKSPACE_LOCK).expect("Cargo.lock must parse");
    let packages = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .expect("Cargo.lock packages must be an array");
    let lsp_server = packages
        .iter()
        .filter(|package| package_name(package) == Some("lsp-server"))
        .collect::<Vec<_>>();
    assert_eq!(lsp_server.len(), 1);
    assert_eq!(package_version(lsp_server[0]), Some(LSP_SERVER_VERSION));
    assert_eq!(
        lsp_server[0].get("checksum").and_then(toml::Value::as_str),
        Some(LSP_SERVER_PACKAGE_HASH)
    );

    let sifr_lsp = packages
        .iter()
        .find(|package| package_name(package) == Some("sifr_lsp"))
        .expect("sifr_lsp must exist in Cargo.lock");
    let edges = sifr_lsp
        .get("dependencies")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_str)
        .filter(|edge| edge.starts_with("lsp-server"))
        .collect::<Vec<_>>();
    assert_eq!(edges, ["lsp-server"]);
}

#[test]
fn vendor_contains_the_official_lsp_server_release() {
    let manifest: toml::Value =
        toml::from_str(VENDOR_MANIFEST).expect("vendor manifest must parse");
    assert_eq!(
        manifest
            .get("package")
            .and_then(|package| package.get("name"))
            .and_then(toml::Value::as_str),
        Some("lsp-server")
    );
    assert_eq!(
        manifest
            .get("package")
            .and_then(|package| package.get("version"))
            .and_then(toml::Value::as_str),
        Some(LSP_SERVER_VERSION)
    );

    let checksum: serde_json::Value =
        serde_json::from_str(VENDOR_CHECKSUM).expect("vendor checksum must parse");
    assert_eq!(
        checksum.get("package").and_then(serde_json::Value::as_str),
        Some(LSP_SERVER_PACKAGE_HASH)
    );
}

fn package_name(package: &toml::Value) -> Option<&str> {
    package.get("name").and_then(toml::Value::as_str)
}

fn package_version(package: &toml::Value) -> Option<&str> {
    package.get("version").and_then(toml::Value::as_str)
}
