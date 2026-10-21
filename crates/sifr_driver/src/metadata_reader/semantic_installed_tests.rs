use sifr_semantic_services::SemanticContext;
use std::path::Path;
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
#[test]
fn semantic_and_native_share_installed_admission_without_sources() {
    let native = crate::CompilerContext::for_test();
    let provider = native.metadata_provider().unwrap();
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path();
    std::fs::create_dir_all(root.join("lib/sifr")).unwrap();
    let metadata_path = root.join("lib/sifr/stdlib.sifrmeta");
    std::fs::copy(&provider.metadata.path, &metadata_path).unwrap();
    let descriptor_path = root.join("lib/sifr/stdlib.metadata.json");
    let descriptor = serde_json::json!({
        "schema_version": 1,
        "compiler_identity": native.identity().as_str(),
        "semantic_target": super::selection::target(),
        "semantic_target_id": hex(&provider.metadata.compatibility.semantic_target),
        "stdlib_inputs_id": hex(&provider.metadata.compatibility.stdlib_inputs),
        "metadata_id": provider.metadata.metadata_id,
        "compiler_binary_sha256": "0".repeat(64),
    });
    let write = |value: &serde_json::Value| {
        std::fs::write(&descriptor_path, serde_json::to_vec(value).unwrap()).unwrap()
    };
    write(&descriptor);
    let mut selected = native.sysroot().unwrap().clone();
    selected.root = root.to_owned();
    selected.manifest.target_triple = super::selection::target().into();
    selected.paths.stdlib_root = root.join("absent-sources");
    let make_native =
        || crate::CompilerContext::with_sysroot(native.identity().clone(), selected.clone());
    let make_semantic =
        || SemanticContext::with_sysroot(native.identity().clone(), selected.clone());
    let source = include_str!(
        "../../../../verification/areas/sql_platform/fixtures/migration_source/add_status.sifr"
    );
    assert_eq!(
        crate::compile_sql_migration_source(&make_native(), source).unwrap(),
        sifr_semantic_services::compile_sql_migration_source(&make_semantic(), source).unwrap()
    );
    assert!(!selected.paths.stdlib_root.exists());
    let compare_rejection = || {
        let expected = crate::compile_sql_migration_source(&make_native(), source).unwrap_err();
        let actual = sifr_semantic_services::compile_sql_migration_source(&make_semantic(), source)
            .unwrap_err();
        assert_eq!(
            serde_json::to_value(expected).unwrap(),
            serde_json::to_value(actual).unwrap()
        );
    };
    for (field, value) in [
        ("schema_version", serde_json::json!(2)),
        ("compiler_identity", serde_json::json!("1".repeat(64))),
        ("semantic_target", serde_json::json!("other-target")),
        ("semantic_target_id", serde_json::json!("2".repeat(64))),
        ("stdlib_inputs_id", serde_json::json!("3".repeat(64))),
        ("metadata_id", serde_json::json!("4".repeat(64))),
        ("compiler_binary_sha256", serde_json::json!("bad")),
    ] {
        let mut changed = descriptor.clone();
        changed[field] = value;
        write(&changed);
        compare_rejection();
    }
    std::fs::remove_file(&descriptor_path).unwrap();
    compare_rejection();
    write(&descriptor);
    std::fs::write(&metadata_path, b"corrupt").unwrap();
    compare_rejection();
    std::fs::remove_file(&metadata_path).unwrap();
    compare_rejection();
    let retry = make_semantic();
    assert!(retry.external_defs().is_err());
    std::fs::copy(&provider.metadata.path, &metadata_path).unwrap();
    assert_eq!(
        sifr_semantic_services::compile_sql_migration_source(&retry, source).unwrap(),
        crate::compile_sql_migration_source(&make_native(), source).unwrap()
    );
}
#[test]
fn source_semantic_override_is_exact_and_failure_is_retryable() {
    let native = crate::CompilerContext::for_test();
    let original = native.metadata_provider().unwrap();
    let temporary = tempfile::tempdir().unwrap();
    let selected = temporary.path().join("selected.sifrmeta");
    let semantic =
        SemanticContext::with_sysroot(native.identity().clone(), native.sysroot().unwrap().clone())
            .with_metadata_override(selected.clone());
    assert!(semantic.external_defs().is_err());
    std::fs::copy(&original.metadata.path, &selected).unwrap();
    assert!(semantic.external_defs().unwrap().provider.is_some());
    let incompatible = SemanticContext::with_sysroot(
        sifr_identity::CompilerIdentity::product(&"a".repeat(64)).unwrap(),
        native.sysroot().unwrap().clone(),
    )
    .with_metadata_override(selected);
    assert!(incompatible.external_defs().is_err());
    assert!(Path::new(&original.metadata.path).is_file());
}
