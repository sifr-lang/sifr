use super::tests::{base_project, test_dependency_plan};
use super::{binary_project_cache_key, materialize_binary_project_files};
use crate::build::project_codegen::format_generated_binary_project;
use sifr_codegen::{
    RustGeneratedBridgeField, RustGeneratedBridgeType, RustGeneratedBridgeTypeKind,
};

fn bridge(
    module: &str,
    kind: RustGeneratedBridgeTypeKind,
    fields: &[&str],
) -> RustGeneratedBridgeType {
    RustGeneratedBridgeType {
        module_name: Some(module.to_string()),
        name: "RecordBridge".to_string(),
        rust_type_path: format!("crate::__sifr_bridge::{module}::RecordBridge"),
        kind,
        supports_eq: true,
        fields: fields
            .iter()
            .map(|name| RustGeneratedBridgeField {
                name: (*name).to_string(),
                sifr_type: "int".to_string(),
                rust_type: "i64".to_string(),
            })
            .collect(),
        variants: Vec::new(),
    }
}

#[test]
fn project_field_identity_materializes_bridged_records_and_errors() {
    let mut project = base_project();
    project.main_rs = r#"
        mod consumers;
        use crate::__sifr_bridge::left::RecordBridge as Left;
        use crate::__sifr_bridge::right::RecordBridge as Right;
        fn main() {
            let left = Left { _name: 3, name: 8 };
            let right = Right { _name: 5 };
            assert_eq!(consumers::inspect(left, right), 16);
        }
    "#
    .to_string();
    project.support_modules.insert(
        "consumers".to_string(),
        r#"
        use crate::__sifr_bridge::left::RecordBridge as Left;
        use crate::__sifr_bridge::right::RecordBridge as Right;
        pub fn inspect(left: Left, right: Right) -> i64 {
            let Left { _name, name } = left;
            _name + name + right._name
        }
    "#
        .to_string(),
    );
    project.interop.rust.bridge_contracts.generated_types = vec![
        bridge(
            "left",
            RustGeneratedBridgeTypeKind::Record,
            &["_name", "name"],
        ),
        bridge("right", RustGeneratedBridgeTypeKind::Error, &["_name"]),
    ];
    let mut project =
        format_generated_binary_project(project).expect("complete project formatting");
    assert!(project.bridge_modules["__sifr_bridge::left"].contains("pub name_field: i64"));
    assert!(project.bridge_modules["__sifr_bridge::left"].contains("pub name: i64"));
    assert!(project.bridge_modules["__sifr_bridge::right"].contains("pub name: i64"));
    assert!(project.support_modules["consumers"].contains("right.name"));
    assert!(
        !project
            .emit_source_listing()
            .contains("// src/__sifr_bridge/")
    );
    let plan = test_dependency_plan("field-identity");
    let before = binary_project_cache_key("fields", &project, &plan);
    project
        .bridge_modules
        .get_mut("__sifr_bridge::left")
        .expect("left module")
        .push_str("\n// cache identity\n");
    assert_ne!(before, binary_project_cache_key("fields", &project, &plan));

    let root = std::env::temp_dir().join(format!(
        "sifr_bridge_field_identity_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    materialize_binary_project_files(&root, "fields", project, &plan)
        .expect("materialize once-canonicalized sources");
    let left = std::fs::read_to_string(root.join("src/sifr_generated_bridge/left.rs"))
        .expect("bridge source");
    assert!(left.contains("pub name_field: i64"));
    let binary = root.join("fields");
    let output = std::process::Command::new("rustc")
        .args(["--edition", "2024"])
        .arg(root.join("src/main.rs"))
        .arg("-o")
        .arg(&binary)
        .output()
        .expect("compile materialized project");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        std::process::Command::new(binary)
            .status()
            .expect("run bridge regression")
            .success()
    );
    std::fs::remove_dir_all(root).expect("remove owned test artifacts");
}

#[test]
fn project_field_identity_rejects_bridge_module_collision() {
    let mut project = base_project();
    project.support_modules.insert(
        "__sifr_bridge.left".to_string(),
        "pub struct Unrelated;".to_string(),
    );
    project.interop.rust.bridge_contracts.generated_types = vec![bridge(
        "left",
        RustGeneratedBridgeTypeKind::Record,
        &["_name"],
    )];
    let errors = match format_generated_binary_project(project) {
        Ok(_) => panic!("bridge and support module identities must not overwrite each other"),
        Err(errors) => errors,
    };
    assert!(
        errors[0]
            .message
            .contains("duplicate generated project module identity: __sifr_bridge::left")
    );
}
