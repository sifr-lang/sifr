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

fn copy_fixture_tree(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).expect("fixture destination should be created");
    for entry in std::fs::read_dir(source).expect("fixture directory should be readable") {
        let entry = entry.expect("fixture entry should be readable");
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if entry.file_name() == "target" {
            continue;
        }
        if source_path.is_dir() {
            copy_fixture_tree(&source_path, &destination_path);
        } else {
            std::fs::copy(&source_path, &destination_path).expect("fixture file should be copied");
        }
    }
}

pub(super) fn copied_scenario(fixture_id: &str, scenario_id: &str, test_name: &str) -> PathBuf {
    let destination = mktemp_dir(test_name);
    copy_fixture_tree(
        &fixture_scenario_root(fixture_id, scenario_id),
        &destination,
    );
    destination
}

/// A copied authority used by explicit native preparation and its assertion.
/// The lease serializes reset, all Cargo work, and output capture at one owned
/// path. Current fixture bytes are recopied on every acquisition; no semantic
/// result or authorization is restored from the previous invocation.
pub(super) struct ReusableScenario {
    root: PathBuf,
    #[cfg(unix)]
    _activity: crate::cache_storage::LeaseActivity,
    _lease: std::fs::File,
}

impl std::ops::Deref for ReusableScenario {
    type Target = Path;
    fn deref(&self) -> &Path {
        &self.root
    }
}

impl Drop for ReusableScenario {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

pub(super) fn reusable_scenario(
    fixture_id: &str,
    scenario_id: &str,
    test_name: &str,
) -> ReusableScenario {
    let owner = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("owned compiler checkout");
    let mut identity = sifr_identity::IdentityEncoder::new("native-test-fixture-v1");
    for (name, value) in [
        ("fixture", fixture_id.as_bytes()),
        ("scenario", scenario_id.as_bytes()),
        ("test", test_name.as_bytes()),
        ("owner", owner.as_os_str().as_encoded_bytes()),
    ] {
        identity.field(name, value);
    }
    let base = crate::cache_storage::root().join("test-fixtures");
    crate::cache_storage::directory(&base).expect("private fixture directory");
    let key = identity.finish();
    let lease = crate::cache_storage::entry_lock(&base, &key).expect("fixture lease");
    #[cfg(unix)]
    crate::cache_storage::lock_bounded(
        &lease,
        &base.join(".locks").join(&key),
        false,
        crate::cache_storage::LEASE_WAIT,
    )
    .expect("own fixture mutation and native capture");
    #[cfg(windows)]
    lease
        .lock()
        .expect("own fixture mutation and native capture");
    #[cfg(unix)]
    let activity =
        crate::cache_storage::LeaseActivity::start(&lease).expect("renew active fixture lease");
    let root = base.join(key);
    if root.exists() {
        // Validate private ownership and reject symlinks before resetting.
        crate::cache_storage::directory(&root).expect("owned prior fixture");
        std::fs::remove_dir_all(&root).expect("reset inactive fixture");
    }
    crate::cache_storage::directory(&root).expect("owned fixture root");
    copy_fixture_tree(&fixture_scenario_root(fixture_id, scenario_id), &root);
    ReusableScenario {
        root,
        #[cfg(unix)]
        _activity: activity,
        _lease: lease,
    }
}

pub(super) fn package_entrypoint_from_cargo_layout(
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

pub(super) fn install_evidence_source(package_root: &Path, source: &str) {
    std::fs::write(package_root.join("src/main.sifr"), source)
        .expect("checked-in evidence source should be installed");
}

pub(super) fn rebase_sifr_runtime_dependency(package_root: &Path) {
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
        "runtime scenario must declare the checked-in runtime path dependency"
    );
    std::fs::write(manifest_path, rebased)
        .expect("copied scenario runtime dependency should be rebased");
}

pub(super) fn built_package_output(entrypoint: &PackageEntrypoint) -> std::process::Output {
    let artifact = build_cached_package_project(
        &crate::CompilerContext::for_test(),
        entrypoint,
        &mut sifr_frontend::DiskSourceProvider::new(),
    )
    .expect("Rust interop package should build");
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

pub(super) fn run_built_package(entrypoint: &PackageEntrypoint) -> String {
    let output = built_package_output(entrypoint);
    String::from_utf8_lossy(&output.stdout).trim().to_string()
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
    let first = reusable_scenario(
        "advanced_data_runtime_matrix",
        "advanced_data_runtime",
        "reuse-ownership-regression",
    );
    let path = first.root.clone();
    let original = std::fs::read(path.join("Cargo.toml")).expect("canonical fixture authority");
    let contender = crate::cache_storage::entry_lock(
        path.parent().expect("private parent"),
        path.file_name()
            .expect("fixture key")
            .to_str()
            .expect("encoded key"),
    )
    .expect("independent contender");
    assert!(matches!(
        contender.try_lock(),
        Err(std::fs::TryLockError::WouldBlock)
    ));
    std::fs::write(path.join("Cargo.toml"), "invalid prior authority")
        .expect("mutate test fixture");
    drop(first);
    assert!(!path.exists());
    let second = reusable_scenario(
        "advanced_data_runtime_matrix",
        "advanced_data_runtime",
        "reuse-ownership-regression",
    );
    assert_eq!(second.root, path);
    assert_eq!(
        std::fs::read(second.join("Cargo.toml")).expect("current authority"),
        original
    );
}
