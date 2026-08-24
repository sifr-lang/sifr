use super::*;

const PROC_MACRO_EVIDENCE: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/proc_macro_trust/positive/trusted_proc_macro.sifr"
);
const PROC_MACRO_NEGATIVE: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/proc_macro_trust/negative/untrusted_proc_macro_rejected_pre_execution.sifr"
);
const GENERATED_FILES: [&str; 2] = ["sifr.probe.rs", "sifr-prost-build-evidence.txt"];

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
#[doc = "sifr-evidence: executes-cargo-probe"]
fn test_build_trusted_proc_macro_and_deterministic_codegen() {
    let package_root = copied_scenario(
        "proc_macro_trust",
        "proc_macro_trust_package",
        "rust_interop_proc_macro_trusted",
    );
    install_evidence_source(
        &package_root,
        &format!(
            "{PROC_MACRO_EVIDENCE}\n\ndef main() -> Result[None, DecodeError | RustPanicError]:\n    try:\n        print(verify_trusted_proc_macro())\n    except DecodeError as error:\n        raise error\n    except RustPanicError as error:\n        raise error\n    return None\n"
        ),
    );

    let first_target = mktemp_dir("rust_interop_proc_macro_codegen_first");
    let second_target = mktemp_dir("rust_interop_proc_macro_codegen_second");
    let first_artifacts = cargo_codegen_artifacts(&package_root, &first_target);
    let second_artifacts = cargo_codegen_artifacts(&package_root, &second_target);
    assert_eq!(
        first_artifacts, second_artifacts,
        "fresh locked prost-build outputs must be byte-identical"
    );
    assert_eq!(
        first_artifacts
            .get("sifr-prost-build-evidence.txt")
            .map(Vec::as_slice),
        Some(b"prost-build=0.14.4;message=sifr.probe.Probe".as_slice())
    );
    let generated = &first_artifacts["sifr.probe.rs"];
    for token in [
        b"pub struct Probe".as_slice(),
        b"pub id: u64",
        b"pub payload:",
    ] {
        assert!(
            generated.windows(token.len()).any(|window| window == token),
            "prost-build output must contain {}",
            String::from_utf8_lossy(token)
        );
    }

    let entrypoint =
        package_entrypoint_from_cargo_layout(&package_root, "proc-macro-trust-package");
    let errors = check_package_project(&entrypoint, &mut sifr_frontend::DiskSourceProvider::new());
    assert!(
        errors.is_empty(),
        "trusted proc-macro package must pass compiler checking: {errors:#?}"
    );
    let output = built_package_output(&entrypoint);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "Ok(\"id=1404|payload=sifr-rust-interop|serde_derive=1.0.229;upstream=compiled;sifr_wrapper_macro=executed|prost-build=0.14.4;message=sifr.probe.Probe\")"
    );
    assert!(
        output.stderr.is_empty(),
        "trusted proc-macro package must not emit stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let _ = std::fs::remove_dir_all(first_target);
    let _ = std::fs::remove_dir_all(second_target);
    let _ = std::fs::remove_dir_all(package_root);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
#[doc = "sifr-evidence: executes-compiler-diagnostic"]
fn test_check_untrusted_proc_macro_rejected_pre_execution() {
    assert_armed_build_time_dependencies_write_sentinels();
    for (
        case,
        manifest_line,
        untrusted_line,
        required_trust,
        required_allowlist,
        required_evidence,
    ) in [
        (
            "proc_macro",
            "rust-proc-macros = [\"serde_derive\"]",
            "rust-proc-macros = []",
            "serde_derive",
            "[trust].rust-proc-macros",
            "proc-macro target in Cargo dependency `serde_derive`",
        ),
        (
            "build_script",
            "rust-build-scripts = [\"prost_build\"]",
            "rust-build-scripts = []",
            "prost_build",
            "[trust].rust-build-scripts",
            "build script in Cargo dependency `prost_build`",
        ),
    ] {
        assert_untrusted_build_time_dependency_rejected(
            case,
            manifest_line,
            untrusted_line,
            required_trust,
            required_allowlist,
            required_evidence,
        );
    }
}

fn assert_armed_build_time_dependencies_write_sentinels() {
    let package_root = copied_scenario(
        "proc_macro_trust",
        "proc_macro_trust_package",
        "rust_interop_proc_macro_sentinel_control",
    );
    let (proc_macro_sentinel, build_script_sentinel) = arm_build_time_sentinels(&package_root);
    let target_dir = mktemp_dir("rust_interop_proc_macro_sentinel_control_target");
    let output = cargo_build(&package_root, &target_dir);
    assert!(
        output.status.success(),
        "armed build-time control must pass: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        proc_macro_sentinel.exists(),
        "armed proc macro must create its sentinel when rustc executes it"
    );
    assert!(
        build_script_sentinel.exists(),
        "armed prost build script must create its sentinel when Cargo executes it"
    );
    install_evidence_source(&package_root, PROC_MACRO_NEGATIVE);
    let entrypoint =
        package_entrypoint_from_cargo_layout(&package_root, "proc-macro-trust-package");
    let errors = check_package_project(&entrypoint, &mut sifr_frontend::DiskSourceProvider::new());
    assert!(
        errors.is_empty(),
        "negative evidence must be valid while build-time trust is present: {errors:#?}"
    );
    let _ = std::fs::remove_dir_all(target_dir);
    let _ = std::fs::remove_dir_all(package_root);
}

fn assert_untrusted_build_time_dependency_rejected(
    case: &str,
    manifest_line: &str,
    untrusted_line: &str,
    required_trust: &str,
    required_allowlist: &str,
    required_evidence: &str,
) {
    let package_root = copied_scenario(
        "proc_macro_trust",
        "proc_macro_trust_package",
        &format!("rust_interop_proc_macro_untrusted_{case}"),
    );
    install_evidence_source(&package_root, PROC_MACRO_NEGATIVE);
    let (proc_macro_sentinel, build_script_sentinel) = arm_build_time_sentinels(&package_root);
    let manifest_path = package_root.join("sifr.toml");
    let manifest =
        std::fs::read_to_string(&manifest_path).expect("scenario manifest should be readable");
    let untrusted = manifest.replace(manifest_line, untrusted_line);
    assert_ne!(
        untrusted, manifest,
        "negative scenario must remove {case} trust"
    );
    std::fs::write(&manifest_path, untrusted).expect("negative trust manifest should be installed");
    let entrypoint =
        package_entrypoint_from_cargo_layout(&package_root, "proc-macro-trust-package");
    let errors = check_package_project(&entrypoint, &mut sifr_frontend::DiskSourceProvider::new());
    let rendered = format!("{errors:#?}");

    assert_eq!(
        errors.len(),
        1,
        "missing {case} trust must stop before Cargo or rustc: {errors:#?}"
    );
    assert_eq!(errors[0].code, DiagnosticCode::RUST_TRUST_MISSING.code());
    assert!(
        rendered.contains(required_trust)
            && rendered.contains(required_allowlist)
            && rendered.contains(required_evidence)
            && rendered.contains("before Cargo executes this dependency"),
        "diagnostic must identify missing {case} trust: {errors:#?}"
    );
    assert!(
        !proc_macro_sentinel.exists() && !build_script_sentinel.exists(),
        "untrusted build-time code must be rejected before either sentinel"
    );
    let _ = std::fs::remove_dir_all(package_root);
}

fn arm_build_time_sentinels(package_root: &Path) -> (PathBuf, PathBuf) {
    let proc_macro_root = package_root.join("rust/serde_derive");
    let build_script_root = package_root.join("rust/prost_build");
    std::fs::write(proc_macro_root.join("ARM_PROC_MACRO_SENTINEL"), "armed")
        .expect("proc-macro sentinel should be armed");
    std::fs::write(build_script_root.join("ARM_BUILD_SCRIPT_SENTINEL"), "armed")
        .expect("build-script sentinel should be armed");
    (
        proc_macro_root.join("PROC_MACRO_EXECUTED"),
        build_script_root.join("BUILD_SCRIPT_EXECUTED"),
    )
}

fn cargo_codegen_artifacts(
    package_root: &Path,
    target_dir: &Path,
) -> std::collections::BTreeMap<String, Vec<u8>> {
    let output = cargo_build(package_root, target_dir);
    assert!(
        output.status.success(),
        "locked proc-macro evidence build must pass: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut artifacts = std::collections::BTreeMap::new();
    collect_generated_files(target_dir, &mut artifacts);
    assert_eq!(
        artifacts.len(),
        GENERATED_FILES.len(),
        "every deterministic codegen artifact must be present: {artifacts:#?}"
    );
    artifacts
}

fn cargo_build(package_root: &Path, target_dir: &Path) -> std::process::Output {
    std::process::Command::new("cargo")
        .args([
            "build",
            "--workspace",
            "--release",
            "--locked",
            "--offline",
            "--frozen",
            "--target-dir",
        ])
        .arg(target_dir)
        .current_dir(package_root)
        .output()
        .expect("locked proc-macro evidence build should execute")
}

fn collect_generated_files(
    path: &Path,
    artifacts: &mut std::collections::BTreeMap<String, Vec<u8>>,
) {
    for entry in std::fs::read_dir(path).expect("artifact directory should be readable") {
        let entry = entry.expect("artifact entry should be readable");
        let entry_path = entry.path();
        if entry_path.is_dir() {
            collect_generated_files(&entry_path, artifacts);
            continue;
        }
        let file_name = entry.file_name().to_string_lossy().into_owned();
        if !GENERATED_FILES.contains(&file_name.as_str()) {
            continue;
        }
        let contents = std::fs::read(&entry_path).expect("generated file should be readable");
        assert!(
            artifacts.insert(file_name, contents).is_none(),
            "generated filename must be unique within a fresh target"
        );
    }
}
