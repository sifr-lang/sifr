use super::super::rust_interop_contract_tests::{
    backend_with_manifest, base_project_with_contracts, temp_package_root,
    trusted_no_panic_context, trusted_no_panic_declaration_entry,
};
use super::{apply_package_rust_interop_metadata, plan_rust_interop_contracts_for_test};
use sifr_ir::RustInteropDecoratorKind;

#[test]
fn planned_contract_does_not_qualify_an_absent_cargo_manifest() {
    let backend_root = temp_package_root("unqualified_contract");
    assert!(!backend_root.exists());
    let generated = base_project_with_contracts(
        vec![trusted_no_panic_declaration_entry(
            "native.hash",
            RustInteropDecoratorKind::Function,
        )],
        Vec::new(),
    );
    let context = trusted_no_panic_context(vec![backend_with_manifest(
        "native",
        backend_root.join("Cargo.toml"),
    )]);
    let planned = plan_rust_interop_contracts_for_test(&generated, &context)
        .expect("valid contract can be planned without native qualification");
    assert_eq!(planned.probes.len(), 1);
    let diagnostics = match apply_package_rust_interop_metadata(generated, Some(context)) {
        Ok(_) => panic!("planning must not replace actual Cargo qualification"),
        Err(diagnostics) => diagnostics,
    };
    assert_eq!(diagnostics[0].code, "SIFR-RUST-CARGO-0001");
    assert!(diagnostics[0].message.contains("missing or unreadable"));
}
