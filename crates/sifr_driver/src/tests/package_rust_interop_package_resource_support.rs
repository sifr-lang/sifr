use super::*;

const PACKAGE_RESOURCE_POSITIVE: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/opaque_resource_package_core/positive/package_resource_construct_use_close.sifr"
);
const PACKAGE_RESOURCE_NEGATIVE: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/opaque_resource_package_core/negative/package_resource_alias_use_after_close_rejected.sifr"
);
const COMPILER_REJECTION_PREFIX: &str = "# compiler-rejection: ";

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
#[doc = "sifr-evidence: executes-runtime-observed"]
fn test_package_resource_construct_use_close() {
    let package_root = copied_scenario(
        "opaque_resource_package_core",
        "package_resource_runtime",
        "rust_interop_package_resource_positive",
    );
    rebase_sifr_runtime_dependency(&package_root);
    install_evidence_source(
        &package_root,
        &format!(
            "{PACKAGE_RESOURCE_POSITIVE}\n\ndef main() -> Result[None, PackageResourceError | RustPanicError]:\n    try:\n        result: str = verify_package_resource_construct_use_close()\n        print(result)\n    except PackageResourceError as error:\n        raise error\n    except RustPanicError as error:\n        raise error\n    return None\n"
        ),
    );
    let entrypoint =
        package_entrypoint_from_cargo_layout(&package_root, "package-resource-runtime");

    assert_eq!(
        run_built_package(&entrypoint),
        "number=7;label=sealed;state=open;close=closed"
    );
    let _ = std::fs::remove_dir_all(package_root);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
#[doc = "sifr-evidence: executes-runtime-observed"]
fn test_package_resource_alias_use_after_close_rejected() {
    let package_root = copied_scenario(
        "opaque_resource_package_core",
        "package_resource_runtime",
        "rust_interop_package_resource_negative_runtime",
    );
    rebase_sifr_runtime_dependency(&package_root);
    install_evidence_source(
        &package_root,
        &format!(
            "{PACKAGE_RESOURCE_NEGATIVE}\n\ndef main() -> Result[None, PackageResourceError | RustPanicError]:\n    try:\n        result: str = verify_package_resource_alias_use_after_close_rejected()\n        print(result)\n    except PackageResourceError as error:\n        raise error\n    except RustPanicError as error:\n        raise error\n    return None\n"
        ),
    );
    let entrypoint =
        package_entrypoint_from_cargo_layout(&package_root, "package-resource-runtime");
    let output = built_package_output(&entrypoint);
    assert_eq!(observed_resource_state(&output), ObservedRuntimeState::Closed);
    assert!(
        output.stderr.is_empty(),
        "negative package resource evidence must redact the panic hook: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = std::fs::remove_dir_all(package_root);

    let package_root = copied_scenario(
        "opaque_resource_package_core",
        "package_resource_runtime",
        "rust_interop_package_resource_compile_rejections",
    );
    rebase_sifr_runtime_dependency(&package_root);
    let compile_rejections = PACKAGE_RESOURCE_NEGATIVE.replace(COMPILER_REJECTION_PREFIX, "");
    assert_ne!(
        compile_rejections, PACKAGE_RESOURCE_NEGATIVE,
        "registered negative evidence must carry compiler-rejection mutations"
    );
    install_evidence_source(&package_root, &compile_rejections);
    let entrypoint =
        package_entrypoint_from_cargo_layout(&package_root, "package-resource-runtime");
    let errors = check_package_project(&entrypoint);
    assert_eq!(errors.len(), 2, "negative diagnostics must have exact owners");
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.code == DiagnosticCode::OWN_USE_AFTER_MOVE.code())
            .count(),
        1,
        "a second package resource close must be rejected: {errors:#?}"
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| {
                error.code == DiagnosticCode::RUST_TYPE_PROBE_FAILURE.code()
                    && error.message
                        == "sealed Rust opaque resource `PackageResource` cannot be constructed in Sifr; use its declared package factory"
            })
            .count(),
        1,
        "direct package resource construction must have one stable diagnostic: {errors:#?}"
    );
    let _ = std::fs::remove_dir_all(package_root);
}
