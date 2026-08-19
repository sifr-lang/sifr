use super::*;

const CLI_EVIDENCE: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/ecosystem_cli_certification/positive/cli_tooling_probe_coverage.sifr"
);
const CLI_NEGATIVE: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/ecosystem_cli_certification/negative/unsupported_anyhow_surface.sifr"
);

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
#[doc = "sifr-evidence: executes-cargo-probe"]
fn test_build_cli_tooling_probe_and_anyhow_adapter() {
    let package_root = copied_scenario(
        "ecosystem_cli_certification",
        "cli_feature_package",
        "rust_interop_cli_tooling_positive",
    );
    rebase_sifr_runtime_dependency(&package_root);
    assert_exact_cli_dependency_graph(&package_root);
    install_evidence_source(
        &package_root,
        &format!(
            "{CLI_EVIDENCE}\n\ndef main() -> None:\n    print(verify_cli_tooling_probe_coverage())\n    print(verify_cli_tooling_anyhow_adapter())\n"
        ),
    );
    let entrypoint = package_entrypoint_from_cargo_layout(&package_root, "cli-feature-package");
    let errors = check_package_project(&entrypoint, &mut sifr_frontend::DiskSourceProvider::new());
    assert!(
        errors.is_empty(),
        "bridge-safe CLI/tooling package must pass compiler checking: {errors:#?}"
    );

    let output = built_package_output(&entrypoint);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(
            "clap=4.6.1;mode=check;tracing=0.1.44;subscriber=0.3.23;env-filter=enabled;event=observed;anyhow=1.0.102;adapter=CliError"
        ),
        "real CLI/tooling execution marker must be observed: {stdout}"
    );
    assert!(
        stdout.contains("clap parse failed through the anyhow adapter"),
        "internal anyhow failure must cross only as the declared CliError: {stdout}"
    );
    assert!(
        output.stderr.is_empty(),
        "CLI/tooling evidence must not emit stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = std::fs::remove_dir_all(package_root);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
#[doc = "sifr-evidence: executes-compiler-diagnostic"]
fn test_check_direct_anyhow_surface_rejected() {
    let package_root = copied_scenario(
        "ecosystem_cli_certification",
        "cli_feature_package",
        "rust_interop_cli_tooling_anyhow_negative",
    );
    rebase_sifr_runtime_dependency(&package_root);

    install_evidence_source(&package_root, CLI_EVIDENCE);
    let accepted_entrypoint =
        package_entrypoint_from_cargo_layout(&package_root, "cli-feature-package");
    let accepted = check_package_project(
        &accepted_entrypoint,
        &mut sifr_frontend::DiskSourceProvider::new(),
    );
    assert!(
        accepted.is_empty(),
        "explicit CliError adapter must remain accepted before the negative mutation: {accepted:#?}"
    );

    install_evidence_source(&package_root, CLI_NEGATIVE);
    let rejected_entrypoint =
        package_entrypoint_from_cargo_layout(&package_root, "cli-feature-package");
    let errors = check_package_project(
        &rejected_entrypoint,
        &mut sifr_frontend::DiskSourceProvider::new(),
    );
    let rendered = format!("{errors:#?}");
    assert!(
        errors.iter().any(|error| {
            error.code == DiagnosticCode::RUST_TYPE_PROBE_FAILURE.code()
                && error.message.contains("main.expose_anyhow_error")
        }),
        "direct anyhow::Error crossing must be a stable type diagnostic: {errors:#?}"
    );
    assert!(
        rendered.contains("anyhow_surface::direct_error")
            && (rendered.contains("anyhow::Error") || rendered.contains("anyhow :: Error")),
        "diagnostic evidence must identify the unsupported anyhow error type: {errors:#?}"
    );
    assert!(
        errors
            .iter()
            .all(|error| error.code != DiagnosticCode::RUST_TRUST_MISSING.code()),
        "negative evidence must isolate the unsupported type rather than trust policy: {errors:#?}"
    );
    let _ = std::fs::remove_dir_all(package_root);
}

fn assert_exact_cli_dependency_graph(package_root: &Path) {
    let output = std::process::Command::new("cargo")
        .args([
            "tree",
            "--workspace",
            "--edges",
            "features",
            "--locked",
            "--offline",
        ])
        .current_dir(package_root)
        .output()
        .expect("locked CLI/tooling Cargo tree should execute");
    assert!(
        output.status.success(),
        "locked CLI/tooling Cargo tree must pass: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let tree = String::from_utf8_lossy(&output.stdout);
    for package in [
        "anyhow v1.0.102",
        "clap v4.6.1",
        "tracing v0.1.44",
        "tracing-subscriber v0.3.23",
        "tracing-subscriber feature \"env-filter\"",
    ] {
        assert!(
            tree.contains(package),
            "locked graph must contain {package}: {tree}"
        );
    }
}
