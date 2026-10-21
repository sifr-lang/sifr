use super::*;

const ZERO_COPY_EVIDENCE: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/zero_copy_runtime_matrix/positive/crate_backed_view_lifecycle.sifr"
);
const ZERO_COPY_NEGATIVE: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/zero_copy_runtime_matrix/negative/borrow_escape_and_invalid_mutability_rejected.sifr"
);

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
#[doc = "sifr-evidence: executes-runtime-observed"]
fn test_build_zero_copy_crate_backed_view_lifecycle() {
    let mut inputs = ScenarioInputs::new(
        "zero_copy_runtime_matrix",
        "crate_backed_view_runtime",
        "rust_interop_zero_copy_crate_backed_view_lifecycle",
    );
    inputs = inputs.with_runtime();
    inputs = inputs.with_source(
        &format!(
            "{ZERO_COPY_EVIDENCE}\n\ndef main() -> Result[None, ViewError]:\n    try:\n        verified: str = verify_crate_backed_view_lifecycle()\n        print(verified)\n    except ViewError as error:\n        raise error\n    return None\n"
        ),
    );
    let package_root = inputs.finish();
    let entrypoint = package_root.entrypoint("crate-backed-view-runtime");

    let output = package_root.output(&entrypoint);

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "bytes=alias+owner;memmap2=alias+readonly;bytemuck=alias+mutated-value;zerocopy=alias+parsed-value;mutation=exclusive+sealed;send-sync=type-probed;release=released=1;active=0"
    );
    assert!(
        output.stderr.is_empty(),
        "zero-copy runtime scenario must not emit stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    drop(package_root);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
#[doc = "sifr-evidence: executes-compiler-diagnostic"]
fn test_check_zero_copy_borrow_escape_and_invalid_mutability_rejected() {
    let mut inputs = ScenarioInputs::new(
        "zero_copy_runtime_matrix",
        "crate_backed_view_runtime",
        "rust_interop_zero_copy_invalid_contracts",
    );
    inputs = inputs.with_runtime();
    inputs = inputs.with_source(ZERO_COPY_NEGATIVE);
    let package_root = inputs.finish();
    let entrypoint = package_root.entrypoint("crate-backed-view-runtime");

    let errors = package_root.check(&crate::CompilerContext::for_test(), &entrypoint);

    assert_eq!(
        errors.len(),
        3,
        "invalid zero-copy contracts must stop before Cargo probing: {errors:#?}"
    );
    for reason in [
        "mutable Rust views require an exclusive owner parameter",
        "returned Rust views cannot declare `lifetime=call`",
        "async Rust interop views must use `lifetime=static`",
    ] {
        assert!(
            errors.iter().any(|error| {
                error.code == DiagnosticCode::RUST_ZERO_COPY_CONTRACT.code()
                    && error.message.contains(reason)
            }),
            "missing zero-copy rejection for {reason}: {errors:#?}"
        );
    }
    drop(package_root);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_check_zero_copy_view_send_sync_obligations() {
    let mut inputs = ScenarioInputs::new(
        "zero_copy_runtime_matrix",
        "crate_backed_view_runtime",
        "rust_interop_zero_copy_send_sync",
    );
    inputs = inputs.with_runtime();
    let source = ZERO_COPY_EVIDENCE.replace(
        "send=True, sync=True, clone=none",
        "send=False, sync=False, clone=none",
    );
    inputs = inputs.with_source(&source);
    let bridge = inputs.text("src/bridges/zero_copy.rs");
    let non_send_bridge = bridge
        .replace("use std::fmt;", "use std::fmt;\nuse std::rc::Rc;")
        .replace(
            "pub struct CrateBackedView {\n",
            "pub struct CrateBackedView {\n    _not_send_or_sync: Rc<()>,\n",
        )
        .replace(
            "Ok(Handle::new(CrateBackedView {\n",
            "Ok(Handle::new(CrateBackedView {\n        _not_send_or_sync: Rc::new(()),\n",
        );
    inputs.write("src/bridges/zero_copy.rs", non_send_bridge);
    let package_root = inputs.finish();
    let entrypoint = package_root.entrypoint("crate-backed-view-runtime");

    let errors = package_root.check(&crate::CompilerContext::for_test(), &entrypoint);

    let obligation_error = errors.iter().find(|error| {
        error.code == DiagnosticCode::RUST_ZERO_COPY_CONTRACT.code()
            && error.message.contains("Send/Sync obligations")
    });
    assert!(
        obligation_error.is_some(),
        "view type obligations must be enforced by the direct probe: {errors:#?}"
    );
    assert!(
        obligation_error.is_some_and(|error| error.children.is_empty()),
        "recognized view obligation failures must not leak raw rustc output: {errors:#?}"
    );
    drop(package_root);
}
