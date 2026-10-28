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
    let mut inputs = ScenarioInputs::new(
        "method_slot_table",
        "method_slot_runtime",
        "rust_interop_method_slot_runtime",
    );
    inputs = inputs.with_runtime();
    let package_root = inputs.finish();
    let entrypoint = package_root.entrypoint("method_slot_runtime");
    let errors = package_root.check(&crate::CompilerContext::for_test(), &entrypoint);
    assert!(
        errors.is_empty(),
        "method-slot package must pass checking: {errors:#?}"
    );
    assert_eq!(
        package_root.stdout(&entrypoint),
        "value-normalized\ninput-receiver\ninput-serialized\nvalue-no-context\nvalue-shared-3"
    );
    drop(package_root);
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
        let mut inputs = ScenarioInputs::new(
            "method_slot_table",
            "method_slot_runtime",
            &format!("rust_interop_method_slot_{case}"),
        );
        inputs = inputs.with_runtime();
        inputs.write("src/bridges/slots.rs", format!("{VALID_BRIDGE}\n{source}"));
        let package_root = inputs.finish();
        let entrypoint = package_root.entrypoint("method_slot_runtime");

        let errors = package_root.check(&crate::CompilerContext::for_test(), &entrypoint);

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
        drop(package_root);
    }
}
