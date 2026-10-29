use super::*;

#[test]
fn removes_pruned_stdlib_and_runtime_metadata() {
    let mut modules = HashSet::from(["sifr.pathlib".to_string(), "_sifr.fs".to_string()]);
    let mut features = HashSet::from([StdlibFeature::Fs, StdlibFeature::SifrRuntime]);

    retain_generated_dependency_metadata(
        ["struct Path { value: String }"],
        &mut modules,
        &mut features,
    )
    .expect("retained generated Rust should parse");

    assert!(modules.is_empty());
    assert!(features.is_empty());
}

#[test]
fn retains_dependencies_referenced_by_final_generated_paths() {
    let mut modules = HashSet::from(["sifr.io".to_string(), "_sifr.fs".to_string()]);
    let mut features = HashSet::from([
        StdlibFeature::Fs,
        StdlibFeature::SifrRuntime,
        StdlibFeature::Tokio,
    ]);

    retain_generated_dependency_metadata(
            ["fn read(path: &str) { let _ = ::sifr_stdlib::fs::read_text(path); let _ = ::sifr_runtime::SifrInt::from(1); }"],
            &mut modules,
            &mut features,
        )
        .expect("retained generated Rust should parse");

    assert_eq!(
        modules,
        HashSet::from(["sifr.io".to_string(), "_sifr.fs".to_string()])
    );
    assert_eq!(
        features,
        HashSet::from([StdlibFeature::Fs, StdlibFeature::SifrRuntime])
    );
}

#[test]
fn reconstructs_direct_crate_features_from_final_generated_paths() {
    let mut modules = HashSet::new();
    let mut features = HashSet::from([StdlibFeature::BigDecimal]);

    retain_generated_dependency_metadata(
        [r#"
                use ::bigdecimal::BigDecimal;
                use ::num_bigint::BigInt;
                use ::rust_decimal::Decimal;
            "#],
        &mut modules,
        &mut features,
    )
    .expect("retained generated Rust should parse");

    assert_eq!(
        features,
        HashSet::from([
            StdlibFeature::BigDecimal,
            StdlibFeature::NumBigint,
            StdlibFeature::RustDecimal,
        ])
    );
}

#[test]
fn retains_hyphenated_stdlib_features_for_rust_module_paths() {
    let mut modules = HashSet::from([
        "sifr.runtime".to_string(),
        "_sifr.runtime".to_string(),
        "sifr.json".to_string(),
    ]);
    let mut features = HashSet::new();

    retain_generated_dependency_metadata(
            ["fn emit() { let _ = ::sifr_stdlib::runtime_observability::emit_diagnostic(\"info\", \"demo\", \"event\", \"message\"); }"],
            &mut modules,
            &mut features,
        )
        .expect("observability bridge should parse");

    assert_eq!(
        modules,
        HashSet::from(["sifr.runtime".to_string(), "_sifr.runtime".to_string()])
    );
    assert_eq!(
        planned_sifr_stdlib_features(&modules, &features),
        std::collections::BTreeSet::from(["runtime-observability"])
    );

    retain_generated_dependency_metadata(["fn main() {}"], &mut modules, &mut features)
        .expect("empty program should parse");
    assert!(modules.is_empty());
    assert!(features.is_empty());
}
