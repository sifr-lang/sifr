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

fn run_built_package(entrypoint: &PackageEntrypoint) -> String {
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
    String::from_utf8_lossy(&output.stdout).trim().to_string()
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
