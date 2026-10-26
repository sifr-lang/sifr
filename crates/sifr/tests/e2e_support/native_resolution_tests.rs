use super::harness_behavior_tests::compiled_case_for_test;
use super::*;

fn group(dependency: &Path, epoch: &str) -> BatchGroup {
    named_group(dependency, epoch, "resolution_probe")
}

fn named_group(dependency: &Path, epoch: &str, name: &str) -> BatchGroup {
    let mut case = compiled_case_for_test(
        FixtureCase {
            name: name.into(),
            path: PathBuf::from(format!("{name}.sifr")),
            source: "def main(): pass".into(),
            source_hash: "same-fixture-source".into(),
            _expected_stderr: Vec::new(),
        },
        "fn main() { println!(\"{}\", resolution_dep::value()); }",
        HashSet::new(),
    );
    case.dependency_plan
        .retained_direct_dependencies
        .push(format!(
            "resolution_dep = {{ path = {} }}",
            serde_json::to_string(&dependency.to_string_lossy()).unwrap()
        ));
    case.dependency_plan.cache_fingerprint.push_str(epoch);
    build_group_sources(vec![case]).unwrap()
}

fn dependency(root: &Path, version: &str, value: u32) {
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("Cargo.toml"),
        format!(
            "[package]\nname=\"resolution_dep\"\nversion=\"{version}\"\nedition=\"2024\"\n[workspace]\n"
        ),
    )
    .unwrap();
    std::fs::write(
        root.join("src/lib.rs"),
        format!("pub fn value() -> u32 {{ {value} }}\n"),
    )
    .unwrap();
}

fn config(lease: &native_cache::NativeCache) -> RunnerConfig {
    RunnerConfig {
        sifr_jobs: 1,
        rust_jobs: 1,
        run_jobs: 1,
        cargo_build_jobs: 1,
        cache: CacheConfig {
            enabled: true,
            root: lease.root.clone(),
        },
    }
}

fn build(
    lease: &native_cache::NativeCache,
    config: &RunnerConfig,
    groups: &[BatchGroup],
) -> Vec<GroupBuildOutcome> {
    let manifest = initial_cache_manifest(lease, config, groups);
    build_batch_suite(
        lease,
        groups,
        config,
        &toolchain_info(),
        &cache_env_signature(),
        &manifest,
    )
    .0
}

fn execution(lease: &native_cache::NativeCache, built: &GroupBuildOutcome) -> String {
    assert!(built.build_error.is_none(), "{:?}", built.build_error);
    let mut command = Command::new(built.artifact_path.as_ref().unwrap());
    command.args(["--case", "resolution_probe"]);
    lease.bind_command(&mut command).unwrap();
    let capture = run_capture(command);
    assert!(capture.status_ok, "{}", capture.stderr);
    capture.stdout
}

#[test]
fn test_native_resolution_reuses_matching_inputs_and_rebuilds_changed_dependencies() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let dep = root.join("dependency");
    dependency(&dep, "1.0.0", 1);
    let lease = native_cache::NativeCache::acquire(&root.join("cache")).unwrap();
    let config = config(&lease);
    let first_group = group(&dep, "workspace-lock-a");
    let first = build(&lease, &config, &[first_group.clone()]);
    assert_eq!(execution(&lease, &first[0]), "1\n");
    assert!(!first[0].cache_hit);
    let project = &first[0].group_root;
    let first_lock = std::fs::read(project.join("Cargo.lock")).unwrap();
    let first_receipt = std::fs::read(project.join("resolution.json")).unwrap();

    let unrelated = group(&dep, "workspace-lock-unrelated-change");
    assert_eq!(first_group.id, unrelated.id);
    assert_ne!(first_group.fingerprint, unrelated.fingerprint);
    assert_ne!(
        cache_key_for_group(&first_group, &toolchain_info(), &cache_env_signature()),
        cache_key_for_group(&unrelated, &toolchain_info(), &cache_env_signature())
    );
    let reused = build(&lease, &config, &[unrelated.clone()]);
    assert_eq!(execution(&lease, &reused[0]), "1\n");
    assert!(reused[0].cache_hit, "{:?}", reused[0].build_error);
    assert_eq!(
        first_lock,
        std::fs::read(project.join("Cargo.lock")).unwrap()
    );
    assert_ne!(
        first_receipt,
        std::fs::read(project.join("resolution.json")).unwrap()
    );
    assert!(
        std::fs::read_dir(project.join("resolution-history"))
            .unwrap()
            .count()
            > 0
    );

    // A real path dependency changes version and behavior without changing its
    // path in the generated manifest. Fresh resolution must see the new graph.
    dependency(&dep, "1.0.1", 2);
    let changed = group(&dep, "workspace-lock-relevant-change");
    assert_eq!(first_group.id, changed.id);
    let rebuilt = build(&lease, &config, &[changed.clone()]);
    assert_eq!(execution(&lease, &rebuilt[0]), "2\n");
    assert!(!rebuilt[0].cache_hit);
    assert_ne!(
        first_lock,
        std::fs::read(project.join("Cargo.lock")).unwrap()
    );
    assert!(
        std::fs::read_to_string(project.join("Cargo.lock"))
            .unwrap()
            .contains("1.0.1")
    );

    // Path manifest edits also belong to Cargo freshness when the outer plan is
    // unchanged. Its updated lock must not poison the next unchanged invocation.
    dependency(&dep, "1.0.2", 3);
    let path_edit = build(&lease, &config, &[changed.clone()]);
    assert_eq!(execution(&lease, &path_edit[0]), "3\n");
    assert!(!path_edit[0].cache_hit);
    let path_warm = build(&lease, &config, &[changed]);
    assert_eq!(execution(&lease, &path_warm[0]), "3\n");
    assert!(path_warm[0].cache_hit);
}

#[test]
fn test_native_resolution_missing_interrupted_and_corrupted_preparation() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let dep = root.join("dependency");
    dependency(&dep, "1.0.0", 7);
    let lease = native_cache::NativeCache::acquire(&root.join("cache")).unwrap();
    let config = config(&lease);
    let original = group(&dep, "original");
    let first = build(&lease, &config, &[original.clone()]);
    assert_eq!(execution(&lease, &first[0]), "7\n");
    let project = &first[0].group_root;
    let lock = std::fs::read(project.join("Cargo.lock")).unwrap();
    let receipt = std::fs::read(project.join("resolution.json")).unwrap();

    std::fs::remove_file(project.join("Cargo.lock")).unwrap();
    let missing_lock = build(&lease, &config, &[original.clone()]);
    assert_eq!(execution(&lease, &missing_lock[0]), "7\n");
    assert!(missing_lock[0].cache_hit);
    std::fs::remove_file(project.join("resolution.json")).unwrap();
    let interrupted = build(&lease, &config, &[original.clone()]);
    assert_eq!(execution(&lease, &interrupted[0]), "7\n");
    assert!(interrupted[0].cache_hit);

    // Model interruption after publishing a new lock but before its input
    // receipt. The old receipt must not authorize the changed input plan.
    dependency(&dep, "1.0.1", 8);
    let next = group(&dep, "next-inputs");
    let changed = build(&lease, &config, &[next.clone()]);
    assert_eq!(execution(&lease, &changed[0]), "8\n");
    std::fs::write(project.join("resolution.json"), &receipt).unwrap();
    let recovered = build(&lease, &config, &[next.clone()]);
    assert_eq!(execution(&lease, &recovered[0]), "8\n");
    assert!(recovered[0].cache_hit);

    let current_lock = std::fs::read(project.join("Cargo.lock")).unwrap();
    assert_ne!(current_lock, lock);
    std::fs::write(project.join("Cargo.lock"), b"corrupted lock").unwrap();
    let corrupt = build(&lease, &config, &[next.clone()]);
    assert!(corrupt[0].artifact_path.is_none());
    assert!(corrupt[0].build_error.is_some());
    std::fs::write(project.join("Cargo.lock"), current_lock).unwrap();
    std::fs::write(project.join("resolution.json"), b"invalid receipt").unwrap();
    let corrupt_receipt = build(&lease, &config, &[next]);
    assert!(corrupt_receipt[0].artifact_path.is_none());
    assert!(corrupt_receipt[0].build_error.is_some());
}

#[test]
fn test_native_resolution_failure_keeps_evidence_and_continues_independent_groups() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let dep = root.join("dependency");
    dependency(&dep, "1.0.0", 9);
    let lease = native_cache::NativeCache::acquire(&root.join("cache")).unwrap();
    let config = config(&lease);
    let bad = group(&root.join("missing-dependency"), "missing");
    let good = group(&dep, "valid");
    assert_ne!(bad.id, good.id);
    let outcomes = build(&lease, &config, &[bad, good]);
    assert!(outcomes[0].artifact_path.is_none());
    assert!(!outcomes[0].infrastructure_failure);
    assert!(!outcomes[0].group_root.join("resolution.json").exists());
    let retained = std::fs::read_dir(&outcomes[0].group_root)
        .unwrap()
        .filter_map(Result::ok)
        .any(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("resolution-")
                && entry.path().join("stderr.log").is_file()
        });
    assert!(retained);
    assert_eq!(execution(&lease, &outcomes[1]), "9\n");
}

#[test]
fn test_native_resolution_expired_selected_group_remains_cargo_fresh() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let dep = root.join("dependency");
    dependency(&dep, "1.0.0", 3);
    let lease = native_cache::NativeCache::acquire(&root.join("cache")).unwrap();
    let config = config(&lease);
    let selected = group(&dep, "selected");
    let first = build(&lease, &config, &[selected.clone()]);
    assert_eq!(execution(&lease, &first[0]), "3\n");
    let project = &first[0].group_root;
    std::fs::remove_file(project.join("pending")).unwrap();
    std::fs::write(project.join("completed"), b"completed").unwrap();
    let mut manifest = read_cache_manifest(&config.cache.root);
    for entry in manifest.entries.values_mut() {
        entry.built_at_unix_secs = 0;
    }
    write_cache_manifest(&config.cache.root, &manifest);
    let prepared = initial_cache_manifest(&lease, &config, &[selected.clone()]);
    assert!(
        prepared.entries.is_empty(),
        "expired success records are not current evidence"
    );
    assert!(project.is_dir());
    let fresh = build(&lease, &config, &[selected]);
    assert_eq!(execution(&lease, &fresh[0]), "3\n");
    assert!(fresh[0].cache_hit);
}

#[test]
fn test_current_groups_do_not_protect_obsolete_unselected_groups() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    for name in ["selected", "obsolete"] {
        let group = cache_group_path(root, name);
        std::fs::create_dir_all(&group).unwrap();
        std::fs::write(group.join("completed"), b"completed").unwrap();
    }
    let mut retired = Vec::new();
    prune_cache_manifest(
        root,
        CacheManifest {
            schema_version: E2E_CACHE_SCHEMA_VERSION,
            entries: BTreeMap::new(),
        },
        20000,
        &HashSet::from(["selected".to_string()]),
        |path| {
            retired.push(path.to_path_buf());
            Ok(())
        },
    );
    assert_eq!(retired, vec![cache_group_path(root, "obsolete")]);
    assert!(cache_group_path(root, "selected").is_dir());
    assert!(!cache_group_path(root, "obsolete").exists());
}

#[test]
fn test_native_retirement_removes_only_unselected_native_package() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let dep = root.join("dependency");
    dependency(&dep, "1.0.0", 4);
    let lease = native_cache::NativeCache::acquire(&root.join("cache")).unwrap();
    let config = config(&lease);
    let selected = group(&dep, "selected");
    let retired = named_group(&dep, "retired", "retired_probe");
    let first = build(&lease, &config, &[selected.clone(), retired.clone()]);
    assert_eq!(execution(&lease, &first[0]), "4\n");
    assert!(first[1].build_error.is_none(), "{:?}", first[1].build_error);
    for built in &first {
        std::fs::remove_file(built.group_root.join("pending")).unwrap();
        std::fs::write(built.group_root.join("completed"), b"completed").unwrap();
    }
    let dependencies = std::fs::read_dir(config.cache.root.join("native-target/debug/deps"))
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("libresolution_dep")
        })
        .collect::<Vec<_>>();
    assert!(!dependencies.is_empty());
    let mut manifest = read_cache_manifest(&config.cache.root);
    for entry in manifest.entries.values_mut() {
        entry.built_at_unix_secs = 0;
    }
    write_cache_manifest(&config.cache.root, &manifest);
    initial_cache_manifest(&lease, &config, &[selected]);
    assert!(
        !first[1].group_root.exists(),
        "retirement must actually succeed"
    );
    assert!(!first[1].artifact_path.as_ref().unwrap().exists());
    assert!(first[0].artifact_path.as_ref().unwrap().is_file());
    assert!(dependencies.iter().all(|path| path.is_file()));
}
