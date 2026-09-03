mod support;

use support::TestUnwrap as _;

use serde_json::Value;
use sifr_stdlib_manifest::{
    StdlibFeature, feature_for_codegen_requirement, try_generated_cargo_dependencies,
};
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
    normalize_path_dependency(
        normalize_path_dependency(
            dependency.to_string(),
            "sifr_runtime",
            "<sifr_runtime_path>",
        ),
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

fn generated_cargo_dependencies(
    stdlib_modules: &HashSet<String>,
    required_features: &HashSet<StdlibFeature>,
) -> Vec<String> {
    try_generated_cargo_dependencies(stdlib_modules, required_features)
        .test_unwrap("source-tree sysroot dependencies should resolve")
}

#[test]
fn concurrency_runtime_dependency_snapshots_match_codegen_resolver() {
    let payload: Value =
        serde_json::from_str(SNAPSHOT_JSON).test_unwrap("dependency snapshot JSON must parse");
    assert_eq!(
        payload.get("schema_version").and_then(Value::as_i64),
        Some(1)
    );
    assert_eq!(
        payload.get("source").and_then(Value::as_str),
        Some("sifr_stdlib_manifest::try_generated_cargo_dependencies")
    );

    let snapshots = payload
        .get("snapshots")
        .and_then(Value::as_array)
        .test_unwrap("snapshots must be an array");
    let mut ids = Vec::new();
    for snapshot in snapshots {
        let id = snapshot
            .get("id")
            .and_then(Value::as_str)
            .test_unwrap("snapshot id must be a string");
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
