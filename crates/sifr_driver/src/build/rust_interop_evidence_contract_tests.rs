use super::rust_interop_contract_tests::{
    base_project_with_contracts, declaration_entry, interop_errors, none_contract, package_context,
    param_contract, set_bridge_roots, signature_contract, unsupported_contract,
};
use sifr_codegen::RustBridgeParamConvention;
use sifr_ir::RustInteropDecoratorKind;
use sifr_package::TrustPolicy;
use std::path::PathBuf;

fn unsupported_container_diagnostics() -> Vec<crate::diagnostics::RenderedDiagnostic> {
    let generated = base_project_with_contracts(
        vec![declaration_entry(
            "bridge.hash",
            RustInteropDecoratorKind::Function,
        )],
        vec![signature_contract(
            vec![param_contract(
                "items",
                RustBridgeParamConvention::Borrow,
                unsupported_contract(
                    "set[int]",
                    "set[T] is not a supported Rust bridge container",
                ),
            )],
            none_contract(),
        )],
    );
    let mut context = package_context(TrustPolicy::default(), Vec::new());
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);
    interop_errors(
        generated,
        Some(context),
        "unsupported container must fail before a Cargo probe",
    )
}

#[test]
fn direct_negative_type_reports_stable_unsupported_container_diagnostic() {
    let diagnostics = unsupported_container_diagnostics();

    assert_eq!(diagnostics[0].code, "SIFR-RUST-TYPE-0001");
    assert!(diagnostics[0].message.contains("set[int]"));
}

#[test]
fn direct_negative_type_stops_before_cargo_probe_execution() {
    let diagnostics = unsupported_container_diagnostics();

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "SIFR-RUST-TYPE-0001");
    assert!(diagnostics[0].message.contains("app.hash"));
    assert!(!diagnostics[0].message.contains("Rust bridge probe failed"));
}
