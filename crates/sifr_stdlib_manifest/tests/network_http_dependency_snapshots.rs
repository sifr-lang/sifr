use serde_json::Value;
use sifr_stdlib_manifest::{StdlibFeature, try_generated_cargo_dependencies};
use std::collections::HashSet;

const SNAPSHOT_JSON: &str = include_str!(
    "../../../verification/areas/stdlib_parity/data/network_http_dependency_snapshots.json"
);
const WORKSPACE_MANIFEST: &str = include_str!("../../../Cargo.toml");
const WORKSPACE_LOCK: &str = include_str!("../../../Cargo.lock");

fn normalize_sysroot_paths(dependency: String) -> String {
    normalize_path_dependency(
        normalize_path_dependency(dependency, "sifr_runtime", "<sifr_runtime_path>"),
        "sifr_stdlib",
        "<sifr_stdlib_path>",
    )
}

fn normalize_path_dependency(dependency: String, package: &str, placeholder: &str) -> String {
    if !dependency.starts_with(&format!("{package} = ")) {
        return dependency;
    }
    let Some(path_start) = dependency.find("path = \"") else {
        return dependency;
    };
    let value_start = path_start + "path = \"".len();
    let Some(value_end_offset) = dependency[value_start..].find('"') else {
        return dependency;
    };
    let value_end = value_start + value_end_offset;
    format!(
        "{}{}{}",
        &dependency[..value_start],
        placeholder,
        &dependency[value_end..]
    )
}

fn normalized_generated_dependencies(
    modules: &[&str],
    extra_features: &[StdlibFeature],
) -> Vec<String> {
    generated_cargo_dependencies(
        &modules.iter().map(|module| (*module).to_string()).collect(),
        &extra_features.iter().copied().collect(),
    )
    .into_iter()
    .map(normalize_sysroot_paths)
    .collect()
}

fn generated_cargo_dependencies(
    stdlib_modules: &HashSet<String>,
    required_features: &HashSet<StdlibFeature>,
) -> Vec<String> {
    try_generated_cargo_dependencies(stdlib_modules, required_features)
        .expect("source-tree sysroot dependencies should resolve")
}

fn snapshot_field_strings(payload: &Value, snapshot_id: &str, field: &str) -> Vec<String> {
    payload
        .get("production_snapshots")
        .and_then(Value::as_array)
        .expect("production snapshots must be an array")
        .iter()
        .find(|snapshot| snapshot.get("id").and_then(Value::as_str) == Some(snapshot_id))
        .unwrap_or_else(|| panic!("missing production snapshot {snapshot_id}"))
        .get(field)
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("{field} must be an array"))
        .iter()
        .map(|value| {
            value
                .as_str()
                .unwrap_or_else(|| panic!("{field} entries must be strings"))
                .to_string()
        })
        .collect()
}

fn snapshot_dependencies(payload: &Value, snapshot_id: &str) -> Vec<String> {
    snapshot_field_strings(payload, snapshot_id, "production_dependencies")
}

#[test]
fn network_http_tls_versions_match_latest_stable_locks() {
    let manifest: toml::Value =
        toml::from_str(WORKSPACE_MANIFEST).expect("workspace manifest must parse");
    let dependencies = manifest
        .get("workspace")
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(toml::Value::as_table)
        .expect("workspace dependencies must be a table");
    let lock: toml::Value = toml::from_str(WORKSPACE_LOCK).expect("workspace lock must parse");
    let locked_packages = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .expect("workspace lock packages must be an array");

    for (package, manifest_version, locked_version) in [
        ("rustls", "=0.23.43", "0.23.43"),
        ("rcgen", "0.14.9", "0.14.9"),
    ] {
        assert_eq!(
            dependencies
                .get(package)
                .and_then(|dependency| dependency.get("version"))
                .and_then(toml::Value::as_str),
            Some(manifest_version),
            "workspace dependency {package} must select the latest stable version"
        );
        let locked_versions = locked_packages
            .iter()
            .filter(|entry| entry.get("name").and_then(toml::Value::as_str) == Some(package))
            .filter_map(|entry| entry.get("version").and_then(toml::Value::as_str))
            .collect::<Vec<_>>();
        assert_eq!(
            locked_versions,
            vec![locked_version],
            "workspace lock must contain exactly one {package} version"
        );
    }
}

#[test]
fn network_http_dependency_snapshots_exclude_ring5_from_production() {
    let payload: Value =
        serde_json::from_str(SNAPSHOT_JSON).expect("network HTTP dependency snapshot must parse");
    assert_eq!(
        payload.get("schema_version").and_then(Value::as_i64),
        Some(1)
    );

    let ring5_crates = payload
        .get("ring5_dev_test_demo_crates")
        .and_then(Value::as_array)
        .expect("ring5 crate list must be present");
    let ring5_crates = ring5_crates
        .iter()
        .map(|value| value.as_str().expect("ring5 crate names must be strings"))
        .collect::<Vec<_>>();

    let snapshots = payload
        .get("production_snapshots")
        .and_then(Value::as_array)
        .expect("production snapshots must be an array");
    let mut ids = Vec::new();
    for snapshot in snapshots {
        let id = snapshot
            .get("id")
            .and_then(Value::as_str)
            .expect("snapshot id must be a string");
        ids.push(id.to_string());

        let dependencies = snapshot
            .get("production_dependencies")
            .and_then(Value::as_array)
            .expect("production dependencies must be an array");
        let dependency_text = dependencies
            .iter()
            .map(|value| value.as_str().expect("dependency entries must be strings"))
            .collect::<Vec<_>>()
            .join("\n");

        for crate_name in &ring5_crates {
            assert!(
                !dependency_text.contains(crate_name),
                "{id} must not include Ring 5 crate {crate_name} in production dependencies"
            );
        }
    }

    let mut sorted_ids = ids.clone();
    sorted_ids.sort();
    sorted_ids.dedup();
    assert_eq!(ids.len(), sorted_ids.len(), "snapshot ids must be unique");
}

#[test]
fn network_http_snapshot_json_matches_generated_dependency_output() {
    let payload: Value =
        serde_json::from_str(SNAPSHOT_JSON).expect("network HTTP dependency snapshot must parse");
    assert_eq!(
        payload.get("status").and_then(Value::as_str),
        Some("closed-audited")
    );

    assert_eq!(
        snapshot_dependencies(&payload, "network-runtime-core"),
        normalized_generated_dependencies(&["sifr.net"], &[])
    );
    assert_eq!(
        snapshot_field_strings(&payload, "network-runtime-core", "required_features"),
        vec![
            "tokio/macros",
            "tokio/rt",
            "tokio/sync",
            "tokio/time",
            "tokio/net",
            "tokio/io-util",
            "tokio/process",
            "tokio/signal",
            "sifr_runtime/net",
            "tracing/std",
        ]
    );
    assert_eq!(
        snapshot_field_strings(&payload, "network-runtime-core", "must_not_include"),
        vec!["tokio-test", "proptest", "rcgen", "tracing-subscriber"]
    );

    assert_eq!(
        snapshot_dependencies(&payload, "tls-runtime"),
        normalized_generated_dependencies(&["sifr.tls"], &[])
    );
    assert_eq!(
        snapshot_field_strings(&payload, "tls-runtime", "required_features"),
        vec![
            "rustls/aws_lc_rs",
            "tokio-rustls/aws_lc_rs",
            "rustls-platform-verifier",
            "rustls-pemfile",
            "sifr_runtime/net,tls",
            "tokio/net",
            "tracing/std",
        ]
    );
    assert_eq!(
        snapshot_field_strings(&payload, "tls-runtime", "must_not_include"),
        vec!["rcgen", "webpki-roots", "x509-parser", "tracing-subscriber"]
    );

    assert_eq!(
        snapshot_dependencies(&payload, "url-header-cookie"),
        normalized_generated_dependencies(&["sifr.url", "sifr.http"], &[])
    );
    assert_eq!(
        snapshot_field_strings(&payload, "url-header-cookie", "required_features"),
        vec![
            "sifr_stdlib/url",
            "sifr_stdlib/http",
            "sifr-owned-cookie-header-validation",
        ]
    );
    assert_eq!(
        snapshot_field_strings(&payload, "url-header-cookie", "must_not_include"),
        vec!["proptest", "serde", "x509-parser"]
    );

    assert_eq!(
        snapshot_dependencies(&payload, "http-transport"),
        normalized_generated_dependencies(
            &["sifr.http"],
            &[
                StdlibFeature::SifrRuntime,
                StdlibFeature::Tokio,
                StdlibFeature::TokioRustls,
                StdlibFeature::Rustls,
                StdlibFeature::RustlsPemfile,
                StdlibFeature::RustlsPlatformVerifier,
                StdlibFeature::Tracing,
                StdlibFeature::Bytes,
                StdlibFeature::Http,
                StdlibFeature::HttpBody,
                StdlibFeature::HttpBodyUtil,
                StdlibFeature::Hyper,
                StdlibFeature::HyperUtil,
                StdlibFeature::H2,
                StdlibFeature::TowerService,
            ],
        )
    );
    assert_eq!(
        snapshot_field_strings(&payload, "http-transport", "required_features"),
        vec![
            "hyper/http1",
            "hyper/http2",
            "hyper/client",
            "hyper/server",
            "http-body",
            "tower-service",
        ]
    );
    assert_eq!(
        snapshot_field_strings(&payload, "http-transport", "must_not_include"),
        vec![
            "tokio-test",
            "proptest",
            "tracing-subscriber",
            "tower",
            "tower-http",
            "reqwest",
        ]
    );
}

#[test]
fn network_http_net_module_emits_locked_runtime_dependencies() {
    let deps =
        generated_cargo_dependencies(&HashSet::from(["sifr.net".to_string()]), &HashSet::new())
            .into_iter()
            .map(normalize_sysroot_paths)
            .collect::<Vec<_>>();

    assert_eq!(
        deps,
        vec![
            "sifr_runtime = { path = \"<sifr_runtime_path>\", default-features = false, features = [\"net\"] }",
            "sifr_stdlib = { path = \"<sifr_stdlib_path>\", default-features = false, features = [\"net\"] }",
        ]
    );
    assert_eq!(
        sifr_stdlib_manifest::feature_for_codegen_requirement("tokio"),
        Some(StdlibFeature::Tokio)
    );
}

#[test]
fn network_http_tls_module_emits_locked_runtime_dependencies() {
    let deps =
        generated_cargo_dependencies(&HashSet::from(["sifr.tls".to_string()]), &HashSet::new())
            .into_iter()
            .map(normalize_sysroot_paths)
            .collect::<Vec<_>>();

    assert_eq!(
        deps,
        vec![
            "sifr_runtime = { path = \"<sifr_runtime_path>\", default-features = false, features = [\"net\", \"tls\"] }",
            "sifr_stdlib = { path = \"<sifr_stdlib_path>\", default-features = false, features = [\"tls\"] }",
        ]
    );
    assert!(!deps.iter().any(|dep| dep.contains("rcgen")));
    assert!(!deps.iter().any(|dep| dep.contains("webpki-roots")));
    assert!(!deps.iter().any(|dep| dep.contains("x509-parser")));
    assert_eq!(
        sifr_stdlib_manifest::feature_for_codegen_requirement("rustls"),
        Some(StdlibFeature::Rustls)
    );
    assert_eq!(
        sifr_stdlib_manifest::feature_for_codegen_requirement("tokio-rustls"),
        Some(StdlibFeature::TokioRustls)
    );
    assert_eq!(
        sifr_stdlib_manifest::feature_for_codegen_requirement("rustls-platform-verifier"),
        Some(StdlibFeature::RustlsPlatformVerifier)
    );
}

#[test]
fn network_http_url_module_emits_locked_parser_dependencies() {
    let deps =
        generated_cargo_dependencies(&HashSet::from(["sifr.url".to_string()]), &HashSet::new())
            .into_iter()
            .map(normalize_sysroot_paths)
            .collect::<Vec<_>>();

    assert_eq!(
        deps,
        vec![
            "sifr_stdlib = { path = \"<sifr_stdlib_path>\", default-features = false, features = [\"url\"] }"
        ]
    );
    assert!(!deps.iter().any(|dep| dep.contains("cookie")));
    assert!(!deps.iter().any(|dep| dep.starts_with("http = ")));
    assert_eq!(
        sifr_stdlib_manifest::feature_for_codegen_requirement("url"),
        Some(StdlibFeature::Url)
    );
    assert_eq!(
        sifr_stdlib_manifest::feature_for_codegen_requirement("percent-encoding"),
        Some(StdlibFeature::PercentEncoding)
    );
}

#[test]
fn network_http_http_module_emits_locked_header_dependencies_without_cookie_crate() {
    let deps =
        generated_cargo_dependencies(&HashSet::from(["sifr.http".to_string()]), &HashSet::new())
            .into_iter()
            .map(normalize_sysroot_paths)
            .collect::<Vec<_>>();

    assert_eq!(
        deps,
        vec![
            "sifr_stdlib = { path = \"<sifr_stdlib_path>\", default-features = false, features = [\"http\"] }",
        ]
    );
    assert!(!deps.iter().any(|dep| dep.contains("cookie")));
    assert!(!deps.iter().any(|dep| dep.starts_with("url = ")));
    assert!(!deps.iter().any(|dep| dep.contains("percent-encoding")));
    assert_eq!(
        sifr_stdlib_manifest::feature_for_codegen_requirement("http"),
        Some(StdlibFeature::Http)
    );
    assert_eq!(
        sifr_stdlib_manifest::feature_for_codegen_requirement("cookie"),
        Some(StdlibFeature::Cookie)
    );
}

#[test]
fn network_http_combined_modules_emit_all_locked_url_http_dependencies_without_ring5() {
    let deps = generated_cargo_dependencies(
        &HashSet::from(["sifr.url".to_string(), "sifr.http".to_string()]),
        &HashSet::new(),
    )
    .into_iter()
    .map(normalize_sysroot_paths)
    .collect::<Vec<_>>();

    assert_eq!(
        deps,
        vec![
            "sifr_stdlib = { path = \"<sifr_stdlib_path>\", default-features = false, features = [\"http\", \"url\"] }",
        ]
    );
    assert!(!deps.iter().any(|dep| dep.contains("cookie")));
    assert!(!deps.iter().any(|dep| dep.contains("proptest")));
    assert!(!deps.iter().any(|dep| dep.contains("tracing-subscriber")));
}

#[test]
fn network_http_url_and_http_modules_emit_locked_dependencies() {
    let deps = generated_cargo_dependencies(
        &HashSet::from(["sifr.url".to_string(), "sifr.http".to_string()]),
        &HashSet::new(),
    )
    .into_iter()
    .map(normalize_sysroot_paths)
    .collect::<Vec<_>>();

    assert_eq!(
        deps,
        vec![
            "sifr_stdlib = { path = \"<sifr_stdlib_path>\", default-features = false, features = [\"http\", \"url\"] }",
        ]
    );
    assert!(!deps.iter().any(|dep| dep.contains("cookie")));
    assert!(!deps.iter().any(|dep| dep.contains("proptest")));
    assert!(!deps.iter().any(|dep| dep.starts_with("serde = ")));
    assert!(!deps.iter().any(|dep| dep.contains("x509-parser")));
    assert_eq!(
        sifr_stdlib_manifest::feature_for_codegen_requirement("url"),
        Some(StdlibFeature::Url)
    );
    assert_eq!(
        sifr_stdlib_manifest::feature_for_codegen_requirement("percent-encoding"),
        Some(StdlibFeature::PercentEncoding)
    );
    assert_eq!(
        sifr_stdlib_manifest::feature_for_codegen_requirement("http"),
        Some(StdlibFeature::Http)
    );
    assert_eq!(
        sifr_stdlib_manifest::feature_for_codegen_requirement("cookie"),
        Some(StdlibFeature::Cookie)
    );
}
