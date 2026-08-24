use super::*;

const NATIVE_BUILD_EVIDENCE: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/native_build_script/positive/trusted_build_script_native_evidence.sifr"
);
const NATIVE_BUILD_NEGATIVE: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/native_build_script/negative/untrusted_native_link_rejected.sifr"
);
const EVIDENCE_FILES: [&str; 5] = [
    "sifr-bindgen-bindings.rs",
    "sifr-bindgen-evidence.txt",
    "sifr-cc-evidence.txt",
    "sifr-cxx-evidence.txt",
    "sifr-zstd-evidence.txt",
];

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
#[doc = "sifr-evidence: executes-cargo-probe"]
fn test_build_native_build_script_trusted_artifacts() {
    let package_root = copied_scenario(
        "native_build_script",
        "native_trust_package",
        "rust_interop_native_build_trusted",
    );
    install_evidence_source(
        &package_root,
        &format!(
            "{NATIVE_BUILD_EVIDENCE}\n\ndef main() -> Result[None, NativeError | RustPanicError]:\n    try:\n        print(verify_trusted_build_script_native_evidence())\n    except NativeError as error:\n        raise error\n    except RustPanicError as error:\n        raise error\n    return None\n"
        ),
    );
    let first_target = mktemp_dir("rust_interop_native_build_artifacts_first");
    let second_target = mktemp_dir("rust_interop_native_build_artifacts_second");
    let first_artifacts = cargo_build_artifacts(&package_root, &first_target);
    let second_artifacts = cargo_build_artifacts(&package_root, &second_target);
    assert_eq!(
        first_artifacts, second_artifacts,
        "fresh locked builds must emit byte-identical evidence"
    );
    assert_eq!(
        first_artifacts
            .get("sifr-cc-evidence.txt")
            .map(Vec::as_slice),
        Some(b"cc=1.4.4;compiled=sifr_cc_probe".as_slice())
    );
    assert_eq!(
        first_artifacts
            .get("sifr-bindgen-evidence.txt")
            .map(Vec::as_slice),
        Some(b"bindgen=0.72.1;function=sifr_bindgen_probe".as_slice())
    );
    assert_eq!(
        first_artifacts
            .get("sifr-cxx-evidence.txt")
            .map(Vec::as_slice),
        Some(b"cxx=1.0.199;bridge=sifr_cxx_probe".as_slice())
    );
    assert_eq!(
        first_artifacts
            .get("sifr-zstd-evidence.txt")
            .map(Vec::as_slice),
        Some(b"zstd=0.13.3;level=3".as_slice())
    );
    assert!(
        first_artifacts["sifr-bindgen-bindings.rs"]
            .windows(b"sifr_bindgen_probe".len())
            .any(|window| window == b"sifr_bindgen_probe"),
        "bindgen must generate the allowlisted function"
    );

    let entrypoint = package_entrypoint_from_cargo_layout(&package_root, "native-trust-package");
    let errors = check_package_project(&entrypoint, &mut sifr_frontend::DiskSourceProvider::new());
    assert!(
        errors.is_empty(),
        "trusted native package must pass compiler checking: {errors:#?}"
    );
    let output = built_package_output(&entrypoint);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "Ok(\"cc=1.4.4;compiled=sifr_cc_probe|bindgen=0.72.1;function=sifr_bindgen_probe|cxx=1.0.199;bridge=sifr_cxx_probe;value=1000198|zstd=0.13.3;level=3|compressed=zstd-roundtrip\")"
    );
    assert!(
        output.stderr.is_empty(),
        "trusted native package must not emit stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let _ = std::fs::remove_dir_all(first_target);
    let _ = std::fs::remove_dir_all(second_target);
    let _ = std::fs::remove_dir_all(package_root);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
#[doc = "sifr-evidence: executes-compiler-diagnostic"]
fn test_check_native_build_script_untrusted_native_link_rejected_pre_execution() {
    assert_armed_build_script_writes_sentinel();
    for (case, manifest_line, untrusted_line, required_trust, required_evidence) in [
        (
            "build_script",
            "rust-build-scripts = [\"cc\", \"bindgen\", \"cxx\", \"zstd\"]",
            "rust-build-scripts = [\"cc\", \"bindgen\", \"cxx\"]",
            "zstd",
            "build script in Cargo dependency `zstd`",
        ),
        (
            "native_link",
            "native-links = [\"c++\", \"cxxbridge1\", \"link-cplusplus\", \"sifr_cc_probe\", \"sifr_zstd_probe\", \"stdc++\", \"zstd\"]",
            "native-links = [\"c++\", \"cxxbridge1\", \"link-cplusplus\", \"sifr_cc_probe\", \"stdc++\", \"zstd\"]",
            "sifr_zstd_probe",
            "native links `sifr_zstd_probe` declared by Cargo dependency `zstd`",
        ),
    ] {
        assert_untrusted_native_build_rejected(
            case,
            manifest_line,
            untrusted_line,
            required_trust,
            required_evidence,
        );
    }
    assert_untrusted_transitive_native_link_rejected();
}

fn assert_untrusted_native_build_rejected(
    case: &str,
    manifest_line: &str,
    untrusted_line: &str,
    required_trust: &str,
    required_evidence: &str,
) {
    let package_root = copied_scenario(
        "native_build_script",
        "native_trust_package",
        &format!("rust_interop_native_build_untrusted_{case}"),
    );
    install_evidence_source(&package_root, NATIVE_BUILD_NEGATIVE);
    let manifest_path = package_root.join("sifr.toml");
    let manifest =
        std::fs::read_to_string(&manifest_path).expect("scenario manifest should be readable");
    let untrusted = manifest.replace(manifest_line, untrusted_line);
    assert_ne!(
        untrusted, manifest,
        "negative scenario must remove the {case} trust entry"
    );
    std::fs::write(&manifest_path, untrusted).expect("negative trust manifest should be installed");

    let sentinel = arm_zstd_build_script_sentinel(&package_root);
    assert!(!sentinel.exists(), "sentinel must start absent");

    let entrypoint = package_entrypoint_from_cargo_layout(&package_root, "native-trust-package");
    let errors = check_package_project(&entrypoint, &mut sifr_frontend::DiskSourceProvider::new());
    let rendered = format!("{errors:#?}");

    assert_eq!(
        errors.len(),
        1,
        "missing {case} trust must stop before any Cargo probe: {errors:#?}"
    );
    assert_eq!(errors[0].code, DiagnosticCode::RUST_TRUST_MISSING.code());
    assert!(
        rendered.contains(required_trust)
            && rendered.contains(required_evidence)
            && rendered.contains("before Cargo executes this dependency"),
        "diagnostic must identify missing {case} trust: {errors:#?}"
    );
    assert!(
        !sentinel.exists(),
        "untrusted build script must be rejected before sentinel creation"
    );
    let _ = std::fs::remove_dir_all(package_root);
}

fn assert_armed_build_script_writes_sentinel() {
    let package_root = copied_scenario(
        "native_build_script",
        "native_trust_package",
        "rust_interop_native_build_sentinel_control",
    );
    let sentinel = arm_zstd_build_script_sentinel(&package_root);
    let target_dir = mktemp_dir("rust_interop_native_build_sentinel_control_target");
    let output = std::process::Command::new("cargo")
        .args([
            "build",
            "--workspace",
            "--locked",
            "--offline",
            "--frozen",
            "--target-dir",
        ])
        .arg(&target_dir)
        .current_dir(&package_root)
        .output()
        .expect("sentinel control build should execute");
    assert!(
        output.status.success(),
        "sentinel control build must pass: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        sentinel.exists(),
        "armed build-script control must create its sentinel when Cargo executes it"
    );
    let _ = std::fs::remove_dir_all(target_dir);
    let _ = std::fs::remove_dir_all(package_root);
}

fn arm_zstd_build_script_sentinel(package_root: &Path) -> PathBuf {
    let build_script_path = package_root.join("rust/zstd/build.rs");
    let build_script = std::fs::read_to_string(&build_script_path)
        .expect("zstd wrapper build script should be readable");
    let armed_build_script = build_script.replace(
        "fn main() -> Result<(), Box<dyn Error>> {\n",
        "fn main() -> Result<(), Box<dyn Error>> {\n    std::fs::write(PathBuf::from(std::env::var(\"CARGO_MANIFEST_DIR\")?).join(\"UNTRUSTED_BUILD_SCRIPT_EXECUTED\"), \"executed\")?;\n",
    );
    assert_ne!(
        armed_build_script, build_script,
        "negative sentinel mutation must arm the checked-in build script"
    );
    std::fs::write(build_script_path, armed_build_script)
        .expect("armed build script should be installed");
    package_root.join("rust/zstd/UNTRUSTED_BUILD_SCRIPT_EXECUTED")
}

fn assert_untrusted_transitive_native_link_rejected() {
    let package_root = copied_scenario(
        "native_build_script",
        "native_trust_package",
        "rust_interop_native_build_untrusted_transitive_link",
    );
    install_evidence_source(&package_root, NATIVE_BUILD_NEGATIVE);
    let manifest_path = package_root.join("sifr.toml");
    let manifest =
        std::fs::read_to_string(&manifest_path).expect("scenario manifest should be readable");
    let untrusted = manifest.replace(
        "native-links = [\"c++\", \"cxxbridge1\", \"link-cplusplus\", \"sifr_cc_probe\", \"sifr_zstd_probe\", \"stdc++\", \"zstd\"]",
        "native-links = [\"c++\", \"cxxbridge1\", \"link-cplusplus\", \"sifr_cc_probe\", \"sifr_zstd_probe\", \"stdc++\"]",
    );
    assert_ne!(
        untrusted, manifest,
        "negative scenario must remove transitive zstd link trust"
    );
    std::fs::write(&manifest_path, untrusted)
        .expect("negative transitive-link manifest should be installed");
    let entrypoint = package_entrypoint_from_cargo_layout(&package_root, "native-trust-package");
    let errors = match build_cached_package_project(
        &entrypoint,
        &mut sifr_frontend::DiskSourceProvider::new(),
    ) {
        Ok(_) => panic!("undeclared transitive native link must fail the generated package build"),
        Err(errors) => errors,
    };
    assert!(
        errors.iter().any(|error| {
            error.code == DiagnosticCode::RUST_TRUST_MISSING.code()
                && error.message.contains("untrusted native link evidence")
                && error.message.contains("zstd")
        }),
        "undeclared transitive native link must be rejected after Cargo evidence: {errors:#?}"
    );
    let _ = std::fs::remove_dir_all(package_root);
}

fn cargo_build_artifacts(
    package_root: &Path,
    target_dir: &Path,
) -> std::collections::BTreeMap<String, Vec<u8>> {
    let output = std::process::Command::new("cargo")
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
        .expect("locked native evidence build should execute");
    assert!(
        output.status.success(),
        "locked native evidence build must pass: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut artifacts = std::collections::BTreeMap::new();
    collect_evidence_files(target_dir, &mut artifacts);
    assert_eq!(
        artifacts.len(),
        EVIDENCE_FILES.len(),
        "every build-script evidence file must be present: {artifacts:#?}"
    );
    artifacts
}

fn collect_evidence_files(
    path: &Path,
    artifacts: &mut std::collections::BTreeMap<String, Vec<u8>>,
) {
    for entry in std::fs::read_dir(path).expect("artifact directory should be readable") {
        let entry = entry.expect("artifact entry should be readable");
        let entry_path = entry.path();
        if entry_path.is_dir() {
            collect_evidence_files(&entry_path, artifacts);
            continue;
        }
        let file_name = entry.file_name().to_string_lossy().into_owned();
        if !EVIDENCE_FILES.contains(&file_name.as_str()) {
            continue;
        }
        let contents = std::fs::read(&entry_path).expect("evidence file should be readable");
        assert!(
            artifacts.insert(file_name, contents).is_none(),
            "evidence filename must be unique within a fresh target"
        );
    }
}
