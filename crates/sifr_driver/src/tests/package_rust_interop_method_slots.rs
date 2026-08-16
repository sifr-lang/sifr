use super::*;

const VALID_BRIDGE: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/method_slot_table/examples/method_slot_runtime/src/bridges/slots.rs"
);
const HANDLER_RETURN: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/method_slot_table/examples/method_slot_runtime/negative_handler_return.rs"
);
const HANDLER_THREAD: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/method_slot_table/examples/method_slot_runtime/negative_handler_thread.rs"
);
const SHARED_CONTEXT_MUTATION: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/method_slot_table/examples/method_slot_runtime/negative_shared_context_mutation.rs"
);

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
#[doc = "sifr-evidence: executes-runtime-observed"]
fn test_method_slot_runtime() {
    let package_root = copied_scenario(
        "method_slot_table",
        "method_slot_runtime",
        "rust_interop_method_slot_runtime",
    );
    rebase_sifr_runtime_dependency(&package_root);
    let entrypoint = package_entrypoint_from_cargo_layout(&package_root, "method_slot_runtime");
    let errors = check_package_project(&entrypoint);
    assert!(
        errors.is_empty(),
        "method-slot package must pass checking: {errors:#?}"
    );
    assert_eq!(
        run_built_package(&entrypoint),
        "value-normalized\ninput-receiver\nvalue-no-context\nvalue-shared-2"
    );
    let _ = std::fs::remove_dir_all(package_root);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_method_slot_lifetime_thread_and_shared_context_rejections() {
    for (case, source, code, rustc_marker) in [
        (
            "return",
            HANDLER_RETURN,
            DiagnosticCode::RUST_SLOT_HANDLER,
            "lifetime may not live long enough",
        ),
        (
            "thread",
            HANDLER_THREAD,
            DiagnosticCode::RUST_SLOT_HANDLER,
            "cannot be sent between threads safely",
        ),
        (
            "shared_context",
            SHARED_CONTEXT_MUTATION,
            DiagnosticCode::RUST_SLOT_CONTEXT,
            "cannot borrow",
        ),
    ] {
        let package_root = copied_scenario(
            "method_slot_table",
            "method_slot_runtime",
            &format!("rust_interop_method_slot_{case}"),
        );
        rebase_sifr_runtime_dependency(&package_root);
        std::fs::write(
            package_root.join("src/bridges/slots.rs"),
            format!("{VALID_BRIDGE}\n{source}"),
        )
        .expect("negative method-slot bridge should be installed");
        let entrypoint = package_entrypoint_from_cargo_layout(&package_root, "method_slot_runtime");

        let errors = check_package_project(&entrypoint);

        assert!(
            errors.iter().any(|error| {
                error.code == code.code()
                    && error
                        .children
                        .iter()
                        .any(|child| child.message.contains(rustc_marker))
            }),
            "{case} must fail for the exact method-slot lifetime/context reason: {errors:#?}"
        );
        let _ = std::fs::remove_dir_all(package_root);
    }
}
