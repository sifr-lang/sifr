use serde_json::Value;
use sifr_stdlib::{generated_cargo_dependencies, StdlibFeature};
use std::collections::HashSet;

const SNAPSHOT_JSON: &str = include_str!(
    "../../../verification/areas/stdlib_parity/data/network_http_dependency_snapshots.json"
);

fn normalize_runtime_path(dependency: String) -> String {
    if dependency.starts_with("sifr_runtime = ") {
        if dependency.contains("features = [\"net\", \"tls\", \"http\"]") {
            return "sifr_runtime = { path = \"<sifr_runtime_path>\", features = [\"net\", \"tls\", \"http\"] }"
                .to_string();
        }
        if dependency.contains("features = [\"net\", \"tls\"]") {
            return "sifr_runtime = { path = \"<sifr_runtime_path>\", features = [\"net\", \"tls\"] }"
                .to_string();
        }
        if dependency.contains("features = [\"net\"]") {
            return "sifr_runtime = { path = \"<sifr_runtime_path>\", features = [\"net\"] }"
                .to_string();
        }
        return "sifr_runtime = { path = \"<sifr_runtime_path>\" }".to_string();
    }
    dependency
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
    .map(normalize_runtime_path)
    .collect()
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
            "url/std",
            "percent-encoding/std",
            "http/std",
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
    let deps = generated_cargo_dependencies(
        &HashSet::from(["sifr.net".to_string()]),
        &HashSet::new(),
    )
    .into_iter()
    .map(|dependency| {
        if dependency.starts_with("sifr_runtime = ") {
            if dependency.contains("features = [\"net\"]") {
                return "sifr_runtime = { path = \"<sifr_runtime_path>\", features = [\"net\"] }"
                    .to_string();
            }
            return "sifr_runtime = { path = \"<sifr_runtime_path>\" }".to_string();
        }
        dependency
    })
    .collect::<Vec<_>>();

    assert_eq!(
        deps,
        vec![
            "sifr_runtime = { path = \"<sifr_runtime_path>\", features = [\"net\"] }",
            "tokio = { version = \"1.52.3\", features = [\"io-util\", \"macros\", \"net\", \"process\", \"rt\", \"signal\", \"sync\", \"time\"] }",
            "tracing = { version = \"0.1.44\", default-features = false, features = [\"std\"] }",
        ]
    );
    assert_eq!(
        sifr_stdlib::feature_for_codegen_requirement("tokio"),
        Some(StdlibFeature::Tokio)
    );
}

#[test]
fn network_http_tls_module_emits_locked_runtime_dependencies() {
    let deps = generated_cargo_dependencies(
        &HashSet::from(["sifr.tls".to_string()]),
        &HashSet::new(),
    )
    .into_iter()
    .map(|dependency| {
        if dependency.starts_with("sifr_runtime = ") {
            if dependency.contains("features = [\"net\", \"tls\"]") {
                return "sifr_runtime = { path = \"<sifr_runtime_path>\", features = [\"net\", \"tls\"] }"
                    .to_string();
            }
            return "sifr_runtime = { path = \"<sifr_runtime_path>\" }".to_string();
        }
        dependency
    })
    .collect::<Vec<_>>();

    assert_eq!(
        deps,
        vec![
            "sifr_runtime = { path = \"<sifr_runtime_path>\", features = [\"net\", \"tls\"] }",
            "tokio = { version = \"1.52.3\", features = [\"io-util\", \"macros\", \"net\", \"process\", \"rt\", \"signal\", \"sync\", \"time\"] }",
            "tokio-rustls = \"0.26.4\"",
            "rustls = \"=0.23.35\"",
            "rustls-pemfile = \"2.2.0\"",
            "rustls-platform-verifier = { version = \"0.7.0\", default-features = false }",
            "tracing = { version = \"0.1.44\", default-features = false, features = [\"std\"] }",
        ]
    );
    assert!(!deps.iter().any(|dep| dep.contains("rcgen")));
    assert!(!deps.iter().any(|dep| dep.contains("webpki-roots")));
    assert!(!deps.iter().any(|dep| dep.contains("x509-parser")));
    assert_eq!(
        sifr_stdlib::feature_for_codegen_requirement("rustls"),
        Some(StdlibFeature::Rustls)
    );
    assert_eq!(
        sifr_stdlib::feature_for_codegen_requirement("tokio-rustls"),
        Some(StdlibFeature::TokioRustls)
    );
    assert_eq!(
        sifr_stdlib::feature_for_codegen_requirement("rustls-platform-verifier"),
        Some(StdlibFeature::RustlsPlatformVerifier)
    );
}

#[test]
fn network_http_url_module_emits_locked_parser_dependencies() {
    let deps =
        generated_cargo_dependencies(&HashSet::from(["sifr.url".to_string()]), &HashSet::new());

    assert_eq!(
        deps,
        vec!["url = \"2.5.8\"", "percent-encoding = \"2.3.2\"",]
    );
    assert!(!deps.iter().any(|dep| dep.contains("cookie")));
    assert!(!deps.iter().any(|dep| dep.starts_with("http = ")));
    assert_eq!(
        sifr_stdlib::feature_for_codegen_requirement("url"),
        Some(StdlibFeature::Url)
    );
    assert_eq!(
        sifr_stdlib::feature_for_codegen_requirement("percent-encoding"),
        Some(StdlibFeature::PercentEncoding)
    );
}

#[test]
fn network_http_http_module_emits_locked_header_dependencies_without_cookie_crate() {
    let deps =
        generated_cargo_dependencies(&HashSet::from(["sifr.http".to_string()]), &HashSet::new());

    assert_eq!(deps, vec!["http = \"1.4.1\""]);
    assert!(!deps.iter().any(|dep| dep.contains("cookie")));
    assert!(!deps.iter().any(|dep| dep.starts_with("url = ")));
    assert!(!deps.iter().any(|dep| dep.contains("percent-encoding")));
    assert_eq!(
        sifr_stdlib::feature_for_codegen_requirement("http"),
        Some(StdlibFeature::Http)
    );
    assert_eq!(
        sifr_stdlib::feature_for_codegen_requirement("cookie"),
        Some(StdlibFeature::Cookie)
    );
}

#[test]
fn network_http_combined_modules_emit_all_locked_url_http_dependencies_without_ring5() {
    let deps = generated_cargo_dependencies(
        &HashSet::from(["sifr.url".to_string(), "sifr.http".to_string()]),
        &HashSet::new(),
    );

    assert_eq!(
        deps,
        vec![
            "http = \"1.4.1\"",
            "url = \"2.5.8\"",
            "percent-encoding = \"2.3.2\"",
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
    );

    assert_eq!(
        deps,
        vec![
            "http = \"1.4.1\"",
            "url = \"2.5.8\"",
            "percent-encoding = \"2.3.2\"",
        ]
    );
    assert!(!deps.iter().any(|dep| dep.contains("cookie")));
    assert!(!deps.iter().any(|dep| dep.contains("proptest")));
    assert!(!deps.iter().any(|dep| dep.starts_with("serde = ")));
    assert!(!deps.iter().any(|dep| dep.contains("x509-parser")));
    assert_eq!(
        sifr_stdlib::feature_for_codegen_requirement("url"),
        Some(StdlibFeature::Url)
    );
    assert_eq!(
        sifr_stdlib::feature_for_codegen_requirement("percent-encoding"),
        Some(StdlibFeature::PercentEncoding)
    );
    assert_eq!(
        sifr_stdlib::feature_for_codegen_requirement("http"),
        Some(StdlibFeature::Http)
    );
    assert_eq!(
        sifr_stdlib::feature_for_codegen_requirement("cookie"),
        Some(StdlibFeature::Cookie)
    );
}

#[test]
fn network_http_transport_intrinsics_emit_locked_hyper_runtime_dependencies() {
    let deps = generated_cargo_dependencies(
        &HashSet::from(["sifr.http".to_string()]),
        &HashSet::from([
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
        ]),
    )
    .into_iter()
    .map(|dependency| {
        if dependency.starts_with("sifr_runtime = ") {
            if dependency.contains("features = [\"net\", \"tls\", \"http\"]") {
                return "sifr_runtime = { path = \"<sifr_runtime_path>\", features = [\"net\", \"tls\", \"http\"] }"
                    .to_string();
            }
            return "sifr_runtime = { path = \"<sifr_runtime_path>\" }".to_string();
        }
        dependency
    })
    .collect::<Vec<_>>();

    assert_eq!(
        deps,
        vec![
            "http = \"1.4.1\"",
            "bytes = \"1.11.1\"",
            "h2 = \"0.4.14\"",
            "http-body = \"1.0.1\"",
            "http-body-util = { version = \"0.1.3\", default-features = false }",
            "hyper = { version = \"1.10.1\", default-features = false, features = [\"client\", \"http1\", \"http2\", \"server\"] }",
            "hyper-util = { version = \"0.1.20\", default-features = false, features = [\"tokio\"] }",
            "rustls = \"=0.23.35\"",
            "rustls-pemfile = \"2.2.0\"",
            "rustls-platform-verifier = { version = \"0.7.0\", default-features = false }",
            "sifr_runtime = { path = \"<sifr_runtime_path>\", features = [\"net\", \"tls\", \"http\"] }",
            "tokio = { version = \"1.52.3\", features = [\"io-util\", \"macros\", \"net\", \"process\", \"rt\", \"signal\", \"sync\", \"time\"] }",
            "tokio-rustls = \"0.26.4\"",
            "tower-service = \"0.3.3\"",
            "tracing = { version = \"0.1.44\", default-features = false, features = [\"std\"] }",
        ]
    );
}
