use super::*;

const ADVANCED_DATA_EVIDENCE: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/advanced_data_runtime_matrix/positive/crate_backed_arrow_tensor_roundtrips.sifr"
);
const ADVANCED_DATA_NEGATIVE: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/advanced_data_runtime_matrix/negative/schema_shape_device_mismatch_rejected.sifr"
);

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
#[doc = "sifr-evidence: executes-runtime-observed"]
fn test_build_advanced_data_crate_backed_arrow_tensor_roundtrips() {
    let package_root = copied_scenario(
        "advanced_data_runtime_matrix",
        "advanced_data_runtime",
        "rust_interop_advanced_data_roundtrips",
    );
    rebase_sifr_runtime_dependency(&package_root);
    install_evidence_source(
        &package_root,
        &format!(
            "{ADVANCED_DATA_EVIDENCE}\n\ndef main() -> Result[None, DataExchangeError]:\n    try:\n        print(verify_crate_backed_arrow_tensor_roundtrips())\n    except DataExchangeError as error:\n        raise error\n    return None\n"
        ),
    );
    let entrypoint = package_entrypoint_from_cargo_layout(&package_root, "advanced-data-runtime");

    let output = built_package_output(&entrypoint);

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "Ok(\"arrow=schema=value:float64;rows=6;datafusion=registered+fill-nan-planned;polars=value:float64x6+sorted;polars-copy=explicit;copy=input->arrow:none;tensor=dtype=f64;rank=2;shape=2x3;layout=c;strides=3x1;device=cpu;ndarray-copy=none;candle-copy=none;dlpack=protocol=managed-tensor;ownership=transferred;dtype=f64;rank=2;shape=2x3;strides=3x1;device=cpu;copy=none;cleanup-before=arrow-released=0;active=1;tensor-released=0;active=1;cleanup-after=arrow-released=1;active=0;tensor-released=1;active=0\")"
    );
    assert!(
        output.stderr.is_empty(),
        "advanced-data runtime scenario must not emit stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = std::fs::remove_dir_all(package_root);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
#[doc = "sifr-evidence: executes-compiler-diagnostic"]
fn test_check_advanced_data_schema_shape_device_mismatch_rejected() {
    let package_root = copied_scenario(
        "advanced_data_runtime_matrix",
        "advanced_data_runtime",
        "rust_interop_advanced_data_mismatch",
    );
    rebase_sifr_runtime_dependency(&package_root);
    install_evidence_source(&package_root, ADVANCED_DATA_NEGATIVE);
    let entrypoint = package_entrypoint_from_cargo_layout(&package_root, "advanced-data-runtime");

    let errors = check_package_project(&entrypoint, &mut sifr_frontend::DiskSourceProvider::new());

    assert_eq!(
        errors.len(),
        3,
        "advanced-data mismatches must stop before Cargo probing: {errors:#?}"
    );
    for reason in [
        "`schema=` must be a dotted `sifr_arrow_bridge` schema path",
        "tensor `rank=` must match `shape=` and `strides=` length",
        "`device=` must be cpu",
    ] {
        assert!(
            errors.iter().any(|error| {
                error.code == DiagnosticCode::RUST_ZERO_COPY_CONTRACT.code()
                    && error.message.contains(reason)
            }),
            "missing advanced-data rejection for {reason}: {errors:#?}"
        );
    }
    let _ = std::fs::remove_dir_all(package_root);
}
