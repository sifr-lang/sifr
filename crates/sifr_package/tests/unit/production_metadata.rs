use super::*;

#[test]
fn compiler_build_inputs_do_not_claim_sifr_application_metadata() {
    let id = CargoPackageId("compiler-owner".to_string());
    let inputs = serde_json::json!({"production-inputs": ["src", "build.rs"]});
    let native = serde_json::json!({"sifr-build": inputs.clone()});
    assert!(
        CargoSifrMetadata::from_cargo_metadata_value(&id, "compiler-owner", &native)
            .test_unwrap("compiler-owned Cargo metadata is independent")
            .is_none()
    );
    let application = serde_json::json!({
        "sifr-build": inputs.clone(), "sifr": {"manifest": "Sifr.toml"}
    });
    let discovered = CargoSifrMetadata::from_cargo_metadata_value(&id, "application", &application)
        .test_unwrap("application discovery retains its own metadata")
        .test_unwrap("application metadata is present");
    assert_eq!(discovered.manifest, PathBuf::from("Sifr.toml"));
    assert!(
        CargoSifrMetadata::from_cargo_metadata_value(
            &id,
            "invalid-application",
            &serde_json::json!({"sifr": inputs})
        )
        .is_err(),
        "application metadata must still reject compiler ownership fields"
    );
}
