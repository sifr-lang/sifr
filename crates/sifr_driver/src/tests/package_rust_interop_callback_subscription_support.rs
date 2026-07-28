use super::*;

const CALLBACK_SUBSCRIPTION_EVIDENCE: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/callback_subscription_ecosystem/positive/subscription_cancel_shutdown.sifr"
);
const CALLBACK_SUBSCRIPTION_NEGATIVE: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/callback_subscription_ecosystem/negative/invalid_thread_capture_rejected.sifr"
);

pub(super) fn run_lifecycle_runtime() {
    let package_root = copied_scenario(
        "callback_subscription_ecosystem",
        "subscription_lifecycle_runtime",
        "rust_interop_callback_subscription_lifecycle",
    );
    rebase_sifr_runtime_dependency(&package_root);
    install_evidence_source(
        &package_root,
        &format!(
            "{CALLBACK_SUBSCRIPTION_EVIDENCE}\n\nasync def main() -> Result[None, SubscriptionError | RustPanicError]:\n    try:\n        verified: str = await verify_subscription_cancel_shutdown()\n        print(verified)\n    except SubscriptionError as error:\n        raise error\n    except RustPanicError as error:\n        raise error\n    return None\n"
        ),
    );
    let entrypoint =
        package_entrypoint_from_cargo_layout(&package_root, "subscription-lifecycle-runtime");

    let output = built_package_output(&entrypoint);

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "ws=ws:hello;redis=redis:hello;notify=notify:create;overflow=error;handler-error=expected-handler-error;panic=Rust bridge panicked;foreign-thread=true;queue-drained=2;shutdown=drain;cancelled=true;active=0;temp-removed=true"
    );
    assert!(
        output.stderr.is_empty(),
        "callback subscription scenario must not emit stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = std::fs::remove_dir_all(package_root);
}

pub(super) fn check_invalid_thread_capture_rejected() {
    let package_root = copied_scenario(
        "callback_subscription_ecosystem",
        "subscription_lifecycle_runtime",
        "rust_interop_callback_subscription_invalid_capture",
    );
    rebase_sifr_runtime_dependency(&package_root);
    install_evidence_source(&package_root, CALLBACK_SUBSCRIPTION_NEGATIVE);
    let entrypoint =
        package_entrypoint_from_cargo_layout(&package_root, "subscription-lifecycle-runtime");

    let errors = check_package_project(&entrypoint);

    assert_eq!(
        errors.len(),
        5,
        "invalid retained capture must stop before Cargo probing: {errors:#?}"
    );
    assert!(
        errors.iter().any(|error| {
            error.code == DiagnosticCode::RUST_CALLBACK_CONTRACT.code()
                && error.message.contains("handler `handler` capture `state`")
                && error.message.contains("not sendable")
        }),
        "invalid retained capture must use the callback contract diagnostic: {errors:#?}"
    );
    assert!(
        errors.iter().any(|error| {
            error.code == DiagnosticCode::RUST_CALLBACK_CONTRACT.code()
                && error.message.contains("handler `handler` capture `hook`")
                && error
                    .message
                    .contains("captures cannot be proven thread-safe")
        }),
        "unprovable callable captures must use the callback contract diagnostic: {errors:#?}"
    );
    assert!(
        errors.iter().any(|error| {
            error.code == DiagnosticCode::OWN_USE_AFTER_MOVE.code()
                && error.message.contains("handler")
        }),
        "retained handler reuse must fail in Sifr ownership checking: {errors:#?}"
    );
    assert!(
        errors.iter().any(|error| {
            error.code == DiagnosticCode::RUST_CALLBACK_CONTRACT.code()
                && error
                    .message
                    .contains("handler `handler` capture `counter`")
                && error.message.contains("requires `FnMut`")
        }),
        "direct mutating captures must fail the retained Fn contract: {errors:#?}"
    );
    assert!(
        errors.iter().any(|error| {
            error.code == DiagnosticCode::RUST_CALLBACK_CONTRACT.code()
                && error
                    .message
                    .contains("handler `handler` capture `bump.counter`")
                && error.message.contains("requires `FnMut`")
        }),
        "transitive mutating captures must fail the retained Fn contract: {errors:#?}"
    );
    assert!(
        errors
            .iter()
            .all(|error| !error.message.contains("Rust bridge probe failed")),
        "invalid retained capture must reject before Cargo probing: {errors:#?}"
    );
    let _ = std::fs::remove_dir_all(package_root);
}
