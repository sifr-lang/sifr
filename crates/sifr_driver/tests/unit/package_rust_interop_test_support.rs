use super::*;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum ObservedRuntimeState {
    Closed,
    Poisoned,
}

fn fixture_scenario_root(fixture_id: &str, scenario_id: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../verification/areas/rust_interop/fixtures")
        .join(fixture_id)
        .join("examples")
        .join(scenario_id)
}

#[path = "package_rust_interop_reusable.rs"]
mod reusable;
pub(super) use reusable::{ReusableScenario, ScenarioInputs};

fn package_entrypoint_with_leases(
    package_root: &Path,
    sifr_package_name: &str,
    leases: &crate::build::native_child_leases::NativeChildLeases,
) -> PackageEntrypoint {
    let mut command = std::process::Command::new("cargo");
    command
        .args(["metadata", "--format-version=1", "--locked", "--offline"])
        .arg("--manifest-path")
        .arg(package_root.join("Cargo.toml"));
    leases
        .bind(&mut command)
        .expect("bind fixture metadata ownership");
    let output = command
        .output()
        .expect("fixture Cargo metadata should execute");
    assert!(
        output.status.success(),
        "fixture Cargo metadata should pass: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata = sifr_package::parse_metadata_json(&String::from_utf8_lossy(&output.stdout))
        .expect("fixture Cargo metadata should parse");
    let graph =
        sifr_package::derive_package_graph(metadata, &mut sifr_frontend::DiskSourceProvider::new())
            .expect("fixture Cargo package graph should derive");
    let source_map = sifr_package::PackageSourceMap::build(
        &graph,
        &mut sifr_frontend::DiskSourceProvider::new(),
    )
    .expect("fixture source map should build");
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
        lock_mode: sifr_package::CargoLockMode::Normal,
    }
}

pub(super) fn install_evidence_source(package_root: &ReusableScenario, source: &str) {
    std::fs::write(package_root.join("src/main.sifr"), source)
        .expect("checked-in evidence source should be installed");
}

fn rebased_runtime_manifest(manifest: &str) -> String {
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
        "runtime scenario must declare the checked-in runtime path dependency"
    );
    rebased
}

pub(super) fn observed_resource_state(output: &std::process::Output) -> ObservedRuntimeState {
    assert!(
        output.stderr.is_empty(),
        "resource-state observation must not emit stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    match String::from_utf8_lossy(&output.stdout).trim() {
        "resource-state=closed" => ObservedRuntimeState::Closed,
        "resource-state=poisoned" => ObservedRuntimeState::Poisoned,
        other => panic!("unexpected resource runtime state: {other}"),
    }
}

#[test]
fn reusable_scenario_preserves_identity_and_resets_authority() {
    reusable::assert_reconciliation();
}
