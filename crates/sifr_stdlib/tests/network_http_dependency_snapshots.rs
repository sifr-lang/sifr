use serde_json::Value;
use sifr_stdlib::{generated_cargo_dependencies, StdlibFeature};
use std::collections::HashSet;

const SNAPSHOT_JSON: &str =
    include_str!("../../../verification/stdlib/network_http_dependency_snapshots.json");

#[test]
fn network_http_m0_dependency_snapshots_exclude_ring5_from_production() {
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
fn network_http_m1_net_module_emits_locked_runtime_dependencies() {
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
fn network_http_m2_tls_module_emits_locked_runtime_dependencies() {
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
