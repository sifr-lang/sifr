use super::*;

const STRUCTURAL_BRIDGE_EVIDENCE: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/structural_bridge_calls/positive/specialized_structural_value_and_output.sifr"
);
const STRUCTURAL_BRIDGE_NEGATIVE: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/structural_bridge_calls/negative/shape_callback_and_lifetime_mismatches_rejected.sifr"
);

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
#[doc = "sifr-evidence: executes-runtime-observed"]
fn test_build_structural_bridge_runtime() {
    let package_root = copied_scenario(
        "structural_bridge_calls",
        "structural_bridge_runtime",
        "rust_interop_structural_bridge_runtime",
    );
    rebase_sifr_runtime_dependency(&package_root);
    let source_entrypoint =
        package_entrypoint_from_cargo_layout(&package_root, "structural-bridge-runtime");
    let errors = check_package_project(&source_entrypoint);
    assert!(
        errors.is_empty(),
        "checked-in structural bridge scenario should pass package checking: {errors:#?}"
    );
    assert_eq!(
        run_built_package(&source_entrypoint),
        "records=3;sequences=1;optionals=1;strings=input,x,input-box;construction=root/a,b/boxed/tail;callback=typed;sums=sum/WAITING;mapped=stable-token-m6;output=mapped-output"
    );

    install_evidence_source(&package_root, STRUCTURAL_BRIDGE_EVIDENCE);
    let installed_entrypoint =
        package_entrypoint_from_cargo_layout(&package_root, "structural-bridge-runtime");
    assert_eq!(
        run_built_package(&installed_entrypoint),
        "mapped=installed-token;output=mapped-output;drops=1"
    );
    let _ = std::fs::remove_dir_all(package_root);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
#[doc = "sifr-evidence: executes-generated-build-contract"]
fn test_check_structural_bridge_rejects_abort_profile() {
    let package_root = copied_scenario(
        "structural_bridge_calls",
        "structural_bridge_runtime",
        "rust_interop_structural_abort_profile",
    );
    rebase_sifr_runtime_dependency(&package_root);
    let manifest_path = package_root.join("Cargo.toml");
    let manifest = std::fs::read_to_string(&manifest_path)
        .expect("structural bridge manifest should be readable");
    std::fs::write(
        &manifest_path,
        format!("{manifest}\n[profile.release]\npanic = \"abort\"\n"),
    )
    .expect("abort profile should be installed");
    let entrypoint =
        package_entrypoint_from_cargo_layout(&package_root, "structural-bridge-runtime");
    let errors = check_package_project(&entrypoint);
    assert!(
        errors.iter().any(|error| {
            error.code == DiagnosticCode::RUST_PANIC_CONTRACT.code()
                && error.message.contains("`panic=abort`")
        }),
        "structural bridges must reject abort profiles: {errors:#?}"
    );
    let _ = std::fs::remove_dir_all(package_root);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
#[doc = "sifr-evidence: executes-cargo-probe"]
fn test_check_structural_bridge_mismatches_rejected() {
    let package_root = copied_scenario(
        "structural_bridge_calls",
        "structural_bridge_runtime",
        "rust_interop_structural_invalid_placement",
    );
    rebase_sifr_runtime_dependency(&package_root);
    install_evidence_source(&package_root, STRUCTURAL_BRIDGE_NEGATIVE);
    let entrypoint =
        package_entrypoint_from_cargo_layout(&package_root, "structural-bridge-runtime");
    let errors = check_package_project(&entrypoint);
    assert!(
        errors
            .iter()
            .any(|error| error.code == DiagnosticCode::RUST_TYPE_PROBE_FAILURE.code()),
        "owned structural type parameters must fail deliberately: {errors:#?}"
    );
    let _ = std::fs::remove_dir_all(package_root);

    for (case, original, replacement) in [
        (
            "callback_signature",
            "CallScopedCallbackBridge<'_, (T,), Result<T, String>>",
            "CallScopedCallbackBridge<'_, (T,), Result<T, u32>>",
        ),
        (
            "projection_bound",
            "T: StructuralConstruct + StructuralProject,",
            "T: StructuralConstruct,",
        ),
    ] {
        let package_root = copied_scenario(
            "structural_bridge_calls",
            "structural_bridge_runtime",
            &format!("rust_interop_structural_{case}"),
        );
        rebase_sifr_runtime_dependency(&package_root);
        let bridge_path = package_root.join("src/bridges/structural.rs");
        let bridge = std::fs::read_to_string(&bridge_path)
            .expect("structural bridge source should be readable");
        let mutated = bridge.replacen(original, replacement, 1);
        assert_ne!(mutated, bridge, "{case} mutation must match the bridge");
        std::fs::write(&bridge_path, mutated).expect("negative structural bridge should install");
        let entrypoint =
            package_entrypoint_from_cargo_layout(&package_root, "structural-bridge-runtime");
        let errors = check_package_project(&entrypoint);
        assert!(
            errors
                .iter()
                .any(|error| error.code == DiagnosticCode::RUST_TYPE_PROBE_FAILURE.code()),
            "{case} must fail the concrete structural Cargo probe: {errors:#?}"
        );
        let _ = std::fs::remove_dir_all(package_root);
    }
}
