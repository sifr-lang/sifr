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
    let mut inputs = ScenarioInputs::new(
        "structural_bridge_calls",
        "structural_bridge_runtime",
        "rust_interop_structural_bridge_runtime",
    );
    inputs = inputs.with_runtime();
    let package_root = inputs.finish();
    let source_entrypoint = package_root.entrypoint("structural-bridge-runtime");
    let errors = package_root.check(&crate::CompilerContext::for_test(), &source_entrypoint);
    assert!(
        errors.is_empty(),
        "checked-in structural bridge scenario should pass package checking: {errors:#?}"
    );
    assert_eq!(
        package_root.stdout(&source_entrypoint),
        "records=3;sequences=1;optionals=1;strings=input,x,input-box;construction=root/a,b/boxed/tail;callback=typed;sums=sum/WAITING;mapped=stable-mapped-token;output=mapped-output"
    );

    install_evidence_source(&package_root, STRUCTURAL_BRIDGE_EVIDENCE);
    let installed_entrypoint = package_root.entrypoint("structural-bridge-runtime");
    assert_eq!(
        package_root.stdout(&installed_entrypoint),
        "mapped=installed-token;output=mapped-output;drops=1"
    );
    drop(package_root);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
#[doc = "sifr-evidence: executes-generated-build-contract"]
fn test_check_structural_bridge_rejects_abort_profile() {
    let mut inputs = ScenarioInputs::new(
        "structural_bridge_calls",
        "structural_bridge_runtime",
        "rust_interop_structural_abort_profile",
    );
    inputs = inputs.with_runtime();
    let manifest = inputs.text("Cargo.toml");
    inputs.write(
        "Cargo.toml",
        format!("{manifest}\n[profile.release]\npanic = \"abort\"\n"),
    );
    let package_root = inputs.finish();
    let entrypoint = package_root.entrypoint("structural-bridge-runtime");
    let errors = package_root.check(&crate::CompilerContext::for_test(), &entrypoint);
    assert!(
        errors.iter().any(|error| {
            error.code == DiagnosticCode::RUST_PANIC_CONTRACT.code()
                && error.message.contains("`panic=abort`")
        }),
        "structural bridges must reject abort profiles: {errors:#?}"
    );
    drop(package_root);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
#[doc = "sifr-evidence: executes-cargo-probe"]
fn test_check_structural_bridge_mismatches_rejected() {
    let mut inputs = ScenarioInputs::new(
        "structural_bridge_calls",
        "structural_bridge_runtime",
        "rust_interop_structural_invalid_placement",
    );
    inputs = inputs.with_runtime();
    inputs = inputs.with_source(STRUCTURAL_BRIDGE_NEGATIVE);
    let package_root = inputs.finish();
    let entrypoint = package_root.entrypoint("structural-bridge-runtime");
    let errors = package_root.check(&crate::CompilerContext::for_test(), &entrypoint);
    assert!(
        errors
            .iter()
            .any(|error| error.code == DiagnosticCode::RUST_TYPE_PROBE_FAILURE.code()),
        "owned structural type parameters must fail deliberately: {errors:#?}"
    );
    drop(package_root);

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
        let mut inputs = ScenarioInputs::new(
            "structural_bridge_calls",
            "structural_bridge_runtime",
            &format!("rust_interop_structural_{case}"),
        );
        inputs = inputs.with_runtime();
        let bridge = inputs.text("src/bridges/structural.rs");
        let mutated = bridge.replacen(original, replacement, 1);
        assert_ne!(mutated, bridge, "{case} mutation must match the bridge");
        inputs.write("src/bridges/structural.rs", mutated);
        let package_root = inputs.finish();
        let entrypoint = package_root.entrypoint("structural-bridge-runtime");
        let errors = package_root.check(&crate::CompilerContext::for_test(), &entrypoint);
        assert!(
            errors
                .iter()
                .any(|error| error.code == DiagnosticCode::RUST_TYPE_PROBE_FAILURE.code()),
            "{case} must fail the concrete structural Cargo probe: {errors:#?}"
        );
        drop(package_root);
    }
}
