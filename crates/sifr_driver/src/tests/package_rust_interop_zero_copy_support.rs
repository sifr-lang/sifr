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
    let package_root = copied_scenario(
        "zero_copy_runtime_matrix",
        "crate_backed_view_runtime",
        "rust_interop_zero_copy_crate_backed_view_lifecycle",
    );
    rebase_sifr_runtime_dependency(&package_root);
    install_evidence_source(
        &package_root,
        &format!(
            "{ZERO_COPY_EVIDENCE}\n\ndef main() -> Result[None, ViewError]:\n    try:\n        verified: str = verify_crate_backed_view_lifecycle()\n        print(verified)\n    except ViewError as error:\n        raise error\n    return None\n"
        ),
    );
    let entrypoint =
        package_entrypoint_from_cargo_layout(&package_root, "crate-backed-view-runtime");

    let output = built_package_output(&entrypoint);

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "bytes=alias+owner;memmap2=alias+readonly;bytemuck=alias;zerocopy=alias;mutation=exclusive;send-sync=required;release=released=1;active=0"
    );
    assert!(
        output.stderr.is_empty(),
        "zero-copy runtime scenario must not emit stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = std::fs::remove_dir_all(package_root);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
#[doc = "sifr-evidence: executes-compiler-diagnostic"]
fn test_check_zero_copy_borrow_escape_and_invalid_mutability_rejected() {
    let package_root = copied_scenario(
        "zero_copy_runtime_matrix",
        "crate_backed_view_runtime",
        "rust_interop_zero_copy_invalid_contracts",
    );
    rebase_sifr_runtime_dependency(&package_root);
    install_evidence_source(&package_root, ZERO_COPY_NEGATIVE);
    let entrypoint =
        package_entrypoint_from_cargo_layout(&package_root, "crate-backed-view-runtime");

    let errors = check_package_project(&entrypoint);

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
    let _ = std::fs::remove_dir_all(package_root);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_check_zero_copy_view_send_sync_obligations() {
    let package_root = copied_scenario(
        "zero_copy_runtime_matrix",
        "crate_backed_view_runtime",
        "rust_interop_zero_copy_send_sync",
    );
    rebase_sifr_runtime_dependency(&package_root);
    let source = ZERO_COPY_EVIDENCE.replace(
        "send=True, sync=True, clone=none",
        "send=False, sync=False, clone=none",
    );
    install_evidence_source(&package_root, &source);
    let bridge_path = package_root.join("src/bridges/zero_copy.rs");
    let bridge =
        std::fs::read_to_string(&bridge_path).expect("zero-copy bridge should be readable");
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
    std::fs::write(&bridge_path, non_send_bridge)
        .expect("non-Send zero-copy bridge should be installed");
    let entrypoint =
        package_entrypoint_from_cargo_layout(&package_root, "crate-backed-view-runtime");

    let errors = check_package_project(&entrypoint);

    assert!(
        errors.iter().any(|error| {
            error.code == DiagnosticCode::RUST_ZERO_COPY_CONTRACT.code()
                && error.message.contains("Send/Sync obligations")
        }),
        "view type obligations must be enforced by the direct probe: {errors:#?}"
    );
    let _ = std::fs::remove_dir_all(package_root);
}
