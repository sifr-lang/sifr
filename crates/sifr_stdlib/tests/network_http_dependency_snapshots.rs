use serde_json::Value;

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
