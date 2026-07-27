use super::*;

const SAME_WORKSPACE_EVIDENCE: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/same_workspace_crate/positive/declared_path_dependency_resolves.sifr"
);
const SAME_WORKSPACE_NEGATIVE: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/same_workspace_crate/negative/undeclared_workspace_crate_rejected.sifr"
);
const SAME_WORKSPACE_NEGATIVE_CARGO: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/same_workspace_crate/negative/Cargo.toml"
);
const SAME_WORKSPACE_NEGATIVE_LOCK: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/same_workspace_crate/negative/Cargo.lock"
);
const SHARED_BRIDGE_EVIDENCE: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/shared_bridge_crate/positive/stable_runtime_types_only.sifr"
);
const SHARED_BRIDGE_NEGATIVE: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/shared_bridge_crate/negative/package_generated_type_import_rejected.sifr"
);
const SHARED_BRIDGE_NEGATIVE_MANIFEST: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/shared_bridge_crate/negative/sifr.toml"
);
const SHARED_BRIDGE_NEGATIVE_RUST: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/shared_bridge_crate/negative/shared_bridge_lib.rs"
);
const LOCAL_BRIDGE_EVIDENCE: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/local_bridge_blake3/positive/local_bridge_hash_bytes.sifr"
);
const LOCAL_BRIDGE_NEGATIVE: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/local_bridge_blake3/negative/missing_local_bridge_export.sifr"
);
const BRIDGE_TYPE_ROUNDTRIP_EVIDENCE: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/bridge_type_matrix/positive/supported_type_roundtrips.sifr"
);
const PANIC_WRAPPER_EVIDENCE: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/panic_boundary_wrapper_emission/positive/generated_wrapper_maps_panic_to_declared_error.sifr"
);
const PANIC_WRAPPER_INVALID_MAPPER: &str = include_str!(
    "../../../../verification/areas/rust_interop/fixtures/panic_boundary_wrapper_emission/negative/invalid_map_error_signature_rejected.sifr"
);

fn fixture_scenario_root(fixture_id: &str, scenario_id: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../verification/areas/rust_interop/fixtures")
        .join(fixture_id)
        .join("examples")
        .join(scenario_id)
}

fn copy_fixture_tree(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).expect("fixture destination should be created");
    for entry in std::fs::read_dir(source).expect("fixture directory should be readable") {
        let entry = entry.expect("fixture entry should be readable");
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            copy_fixture_tree(&source_path, &destination_path);
        } else {
            std::fs::copy(&source_path, &destination_path).expect("fixture file should be copied");
        }
    }
}

fn copied_scenario(fixture_id: &str, scenario_id: &str, test_name: &str) -> PathBuf {
    let destination = mktemp_dir(test_name);
    copy_fixture_tree(
        &fixture_scenario_root(fixture_id, scenario_id),
        &destination,
    );
    destination
}

fn package_entrypoint_from_cargo_layout(
    package_root: &Path,
    sifr_package_name: &str,
) -> PackageEntrypoint {
    let output = std::process::Command::new("cargo")
        .args(["metadata", "--format-version=1", "--locked", "--offline"])
        .arg("--manifest-path")
        .arg(package_root.join("Cargo.toml"))
        .output()
        .expect("fixture Cargo metadata should execute");
    assert!(
        output.status.success(),
        "fixture Cargo metadata should pass: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata = sifr_package::parse_metadata_json(&String::from_utf8_lossy(&output.stdout))
        .expect("fixture Cargo metadata should parse");
    let graph = sifr_package::derive_package_graph(metadata)
        .expect("fixture Cargo package graph should derive");
    let source_map =
        sifr_package::PackageSourceMap::build(&graph).expect("fixture source map should build");
    let package_id = graph
        .packages
        .values()
        .find(|metadata| metadata.sifr_name.0 == sifr_package_name)
        .expect("fixture Sifr package should exist")
        .package_id
        .clone();
    PackageEntrypoint {
        main_file: package_root.join("src/main.sifr"),
        package_id,
        graph,
        source_map,
        python_runtime: None,
    }
}

fn install_evidence_source(package_root: &Path, source: &str) {
    std::fs::write(package_root.join("src/main.sifr"), source)
        .expect("checked-in evidence source should be installed");
}

fn rebase_sifr_runtime_dependency(package_root: &Path) {
    let manifest_path = package_root.join("Cargo.toml");
    let manifest = std::fs::read_to_string(&manifest_path)
        .expect("scenario Cargo manifest should be readable");
    let runtime_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("driver crate should have a crates parent")
        .join("sifr_runtime");
    let rebased = manifest.replace(
        "../../../../../../../crates/sifr_runtime",
        &runtime_path.display().to_string(),
    );
    assert_ne!(
        rebased, manifest,
        "panic wrapper scenario must declare the checked-in runtime path dependency"
    );
    std::fs::write(manifest_path, rebased)
        .expect("copied scenario runtime dependency should be rebased");
}

fn built_package_output(entrypoint: &PackageEntrypoint) -> std::process::Output {
    let artifact =
        build_cached_package_project(entrypoint).expect("Rust interop package should build");
    let output = std::process::Command::new(artifact.binary_path())
        .output()
        .expect("Rust interop package binary should run");
    assert!(
        output.status.success(),
        "package binary should pass: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn run_built_package(entrypoint: &PackageEntrypoint) -> String {
    let output = built_package_output(entrypoint);
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_build_local_bridge_blake3_positive_cargo_probe() {
    let package_root = copied_scenario(
        "local_bridge_blake3",
        "local_blake3_bridge",
        "rust_interop_local_bridge_positive",
    );
    let pristine_entrypoint =
        package_entrypoint_from_cargo_layout(&package_root, "local-blake3-bridge");
    assert!(
        check_package_project(&pristine_entrypoint).is_empty(),
        "checked-in local bridge scenario should pass package checking"
    );
    install_evidence_source(
        &package_root,
        &format!(
            "{LOCAL_BRIDGE_EVIDENCE}\n\ndef main() -> None:\n    result: bytes = verify_local_bridge_hash_bytes()\n    print(result.to_ints())\n"
        ),
    );
    let entrypoint = package_entrypoint_from_cargo_layout(&package_root, "local-blake3-bridge");

    assert_eq!(
        run_built_package(&entrypoint),
        "[20, 38, 50, 184, 100, 68, 224, 154]"
    );
    let _ = std::fs::remove_dir_all(package_root);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_check_local_bridge_blake3_missing_export_cargo_probe() {
    let package_root = copied_scenario(
        "local_bridge_blake3",
        "local_blake3_bridge",
        "rust_interop_local_bridge_negative",
    );
    install_evidence_source(&package_root, LOCAL_BRIDGE_NEGATIVE);
    let manifest_path = package_root.join("sifr.toml");
    let manifest = std::fs::read_to_string(&manifest_path)
        .expect("local bridge Sifr manifest should be readable");
    let expanded_manifest = manifest.replace(
        "rust-no-panic = [\"bridge.blake3.hash_bytes\", \"bridge.blake3.hash_hex\"]",
        "rust-no-panic = [\"bridge.blake3.hash_bytes\", \"bridge.blake3.hash_hex\", \"bridge.blake3.missing_export\"]",
    );
    assert_ne!(
        expanded_manifest, manifest,
        "negative local bridge trust mutation must match the checked-in manifest"
    );
    std::fs::write(&manifest_path, expanded_manifest)
        .expect("negative local bridge trust should be installed");
    let entrypoint = package_entrypoint_from_cargo_layout(&package_root, "local-blake3-bridge");

    let errors = check_package_project(&entrypoint);

    assert!(
        errors.iter().any(|error| {
            error.code == DiagnosticCode::RUST_RESOLVE_TARGET_ROOT.code()
                && error.message.contains("missing_export")
        }),
        "missing local bridge export must be a stable resolution diagnostic: {errors:#?}"
    );
    let _ = std::fs::remove_dir_all(package_root);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
#[doc = "sifr-evidence: executes-cargo-probe"]
fn test_build_bridge_type_matrix_positive_cargo_probe() {
    let package_root = copied_scenario(
        "bridge_type_matrix",
        "bridge_type_roundtrip",
        "rust_interop_bridge_type_roundtrip",
    );
    let pristine_entrypoint =
        package_entrypoint_from_cargo_layout(&package_root, "bridge-type-roundtrip");
    assert!(
        check_package_project(&pristine_entrypoint).is_empty(),
        "checked-in bridge type scenario should pass package checking"
    );
    install_evidence_source(&package_root, BRIDGE_TYPE_ROUNDTRIP_EVIDENCE);
    let entrypoint = package_entrypoint_from_cargo_layout(&package_root, "bridge-type-roundtrip");

    assert_eq!(
        run_built_package(&entrypoint),
        "serde:nested|bytes:6|invalid nested payload"
    );
    let _ = std::fs::remove_dir_all(package_root);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
#[doc = "sifr-evidence: executes-runtime-observed"]
fn test_build_panic_boundary_wrapper_runtime() {
    let package_root = copied_scenario(
        "panic_boundary_wrapper_emission",
        "panic_wrapper_runtime",
        "rust_interop_panic_wrapper_runtime",
    );
    rebase_sifr_runtime_dependency(&package_root);
    let pristine_entrypoint =
        package_entrypoint_from_cargo_layout(&package_root, "panic-wrapper-runtime");
    let pristine_errors = check_package_project(&pristine_entrypoint);
    assert!(
        pristine_errors.is_empty(),
        "checked-in panic wrapper scenario should pass package checking: {pristine_errors:#?}"
    );
    install_evidence_source(
        &package_root,
        &format!(
            "{PANIC_WRAPPER_EVIDENCE}\n\ndef main() -> None:\n    print(verify_generated_wrapper_maps_panic_to_declared_error())\n"
        ),
    );
    let entrypoint = package_entrypoint_from_cargo_layout(&package_root, "panic-wrapper-runtime");

    let output = built_package_output(&entrypoint);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "ok:safe|ordinary bridge error|mapped: Rust bridge panicked|Rust bridge panicked|Rust bridge panicked|mapped: Rust bridge panicked"
    );
    assert!(
        output.stderr.is_empty(),
        "caught target and mapper panic payloads must not reach stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = std::fs::remove_dir_all(package_root);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_check_panic_boundary_invalid_mapper_signature() {
    let package_root = copied_scenario(
        "panic_boundary_wrapper_emission",
        "panic_wrapper_runtime",
        "rust_interop_panic_wrapper_invalid_mapper",
    );
    rebase_sifr_runtime_dependency(&package_root);
    install_evidence_source(&package_root, PANIC_WRAPPER_INVALID_MAPPER);
    let entrypoint = package_entrypoint_from_cargo_layout(&package_root, "panic-wrapper-runtime");

    let errors = check_package_project(&entrypoint);

    assert!(
        errors.iter().any(|error| {
            error.code == DiagnosticCode::RUST_PANIC_CONTRACT.code()
                && error.message.contains("RustPanicErrorBridge")
        }),
        "invalid panic mapper signature must fail with the panic contract diagnostic: {errors:#?}"
    );
    let _ = std::fs::remove_dir_all(package_root);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_build_same_workspace_crate_positive_cargo_probe() {
    let evidence_root = copied_scenario(
        "same_workspace_crate",
        "workspace_hash_crate",
        "rust_interop_same_workspace_evidence",
    );
    install_evidence_source(&evidence_root, SAME_WORKSPACE_EVIDENCE);
    let evidence_entrypoint =
        package_entrypoint_from_cargo_layout(&evidence_root, "workspace-hash-consumer");
    assert_eq!(
        run_built_package(&evidence_entrypoint),
        "1451903697411170458"
    );

    let scenario_root = copied_scenario(
        "same_workspace_crate",
        "workspace_hash_crate",
        "rust_interop_same_workspace_scenario",
    );
    let scenario_entrypoint =
        package_entrypoint_from_cargo_layout(&scenario_root, "workspace-hash-consumer");
    assert_eq!(
        run_built_package(&scenario_entrypoint),
        "5625995497597281285"
    );
    let _ = std::fs::remove_dir_all(evidence_root);
    let _ = std::fs::remove_dir_all(scenario_root);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_check_same_workspace_crate_negative_cargo_probe() {
    let package_root = copied_scenario(
        "same_workspace_crate",
        "workspace_hash_crate",
        "rust_interop_same_workspace_negative",
    );
    install_evidence_source(&package_root, SAME_WORKSPACE_NEGATIVE);
    std::fs::write(
        package_root.join("Cargo.toml"),
        SAME_WORKSPACE_NEGATIVE_CARGO,
    )
    .expect("checked-in negative Cargo layout should be installed");
    std::fs::write(
        package_root.join("Cargo.lock"),
        SAME_WORKSPACE_NEGATIVE_LOCK,
    )
    .expect("checked-in negative Cargo lock should be installed");
    let entrypoint = package_entrypoint_from_cargo_layout(&package_root, "workspace-hash-consumer");

    let errors = check_package_project(&entrypoint);

    assert!(errors.iter().any(|error| {
        error.code == DiagnosticCode::RUST_RESOLVE_TARGET_ROOT.code()
            && error.message.contains("workspace_hash")
    }));
    let _ = std::fs::remove_dir_all(package_root);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_build_shared_bridge_crate_positive_cargo_probe() {
    let evidence_root = copied_scenario(
        "shared_bridge_crate",
        "shared_hash_bridge",
        "rust_interop_shared_bridge_evidence",
    );
    install_evidence_source(&evidence_root, SHARED_BRIDGE_EVIDENCE);
    let evidence_entrypoint =
        package_entrypoint_from_cargo_layout(&evidence_root, "shared-hash-consumer");
    assert_eq!(run_built_package(&evidence_entrypoint), "8");

    let scenario_root = copied_scenario(
        "shared_bridge_crate",
        "shared_hash_bridge",
        "rust_interop_shared_bridge_scenario",
    );
    let scenario_entrypoint =
        package_entrypoint_from_cargo_layout(&scenario_root, "shared-hash-consumer");
    assert_eq!(run_built_package(&scenario_entrypoint), "4e138d18e63ba405");
    let _ = std::fs::remove_dir_all(evidence_root);
    let _ = std::fs::remove_dir_all(scenario_root);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_check_shared_bridge_crate_negative_cargo_probe() {
    let package_root = copied_scenario(
        "shared_bridge_crate",
        "shared_hash_bridge",
        "rust_interop_shared_bridge_negative",
    );
    install_evidence_source(&package_root, SHARED_BRIDGE_NEGATIVE);
    std::fs::write(
        package_root.join("sifr.toml"),
        SHARED_BRIDGE_NEGATIVE_MANIFEST,
    )
    .expect("checked-in negative Sifr manifest should be installed");
    std::fs::write(
        package_root.join("rust/sifr_shared_hash_bridge/src/lib.rs"),
        SHARED_BRIDGE_NEGATIVE_RUST,
    )
    .expect("checked-in rejected shared bridge source should be installed");
    let entrypoint = package_entrypoint_from_cargo_layout(&package_root, "shared-hash-consumer");

    let errors = check_package_project(&entrypoint);

    assert!(errors.iter().any(|error| {
        error.code == DiagnosticCode::RUST_RESOLVE_TARGET_ROOT.code()
            && error.message.contains("package-specific")
    }));
    let _ = std::fs::remove_dir_all(package_root);
}
