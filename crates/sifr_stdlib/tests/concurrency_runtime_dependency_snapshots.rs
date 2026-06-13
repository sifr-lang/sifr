use serde_json::Value;
use sifr_stdlib::{feature_for_codegen_requirement, generated_cargo_dependencies};
use std::collections::HashSet;

const SNAPSHOT_JSON: &str = include_str!(
    "../../../verification/areas/stdlib_parity/data/concurrency_runtime_dependency_snapshots.json"
);

fn string_array<'a>(snapshot: &'a Value, field: &str) -> Vec<&'a str> {
    snapshot
        .get(field)
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("snapshot {field} must be an array"))
        .iter()
        .map(|value| {
            value
                .as_str()
                .unwrap_or_else(|| panic!("snapshot {field} entries must be strings"))
        })
        .collect()
}

fn normalize_runtime_dependency(dependency: &str) -> String {
    if dependency.starts_with("sifr_runtime = ") {
        return "sifr_runtime = { path = \"<sifr_runtime_path>\" }".to_string();
    }
    dependency.to_string()
}

#[test]
fn concurrency_runtime_dependency_snapshots_match_codegen_resolver() {
    let payload: Value =
        serde_json::from_str(SNAPSHOT_JSON).expect("dependency snapshot JSON must parse");
    assert_eq!(
        payload.get("schema_version").and_then(Value::as_i64),
        Some(1)
    );
    assert_eq!(
        payload.get("source").and_then(Value::as_str),
        Some("sifr_stdlib::generated_cargo_dependencies")
    );

    let snapshots = payload
        .get("snapshots")
        .and_then(Value::as_array)
        .expect("snapshots must be an array");
    let mut ids = Vec::new();
    for snapshot in snapshots {
        let id = snapshot
            .get("id")
            .and_then(Value::as_str)
            .expect("snapshot id must be a string");
        ids.push(id.to_string());

        let stdlib_modules = string_array(snapshot, "stdlib_modules")
            .into_iter()
            .map(str::to_string)
            .collect::<HashSet<_>>();
        let required_features = string_array(snapshot, "required_features")
            .into_iter()
            .map(|feature| {
                feature_for_codegen_requirement(feature).unwrap_or_else(|| {
                    panic!("unknown required feature in snapshot {id}: {feature}")
                })
            })
            .collect::<HashSet<_>>();
        let expected = string_array(snapshot, "dependencies")
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();
        let actual = generated_cargo_dependencies(&stdlib_modules, &required_features)
            .into_iter()
            .map(|dependency| normalize_runtime_dependency(&dependency))
            .collect::<Vec<_>>();

        assert_eq!(actual, expected, "{id}");
    }

    let mut sorted_ids = ids.clone();
    sorted_ids.sort();
    sorted_ids.dedup();
    assert_eq!(ids, sorted_ids, "snapshot ids must be unique and sorted");
}
