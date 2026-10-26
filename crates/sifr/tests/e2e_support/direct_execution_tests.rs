use super::direct_execution::{direct_group, execute};
use super::*;
use std::time::Duration;

fn config(root: &Path) -> RunnerConfig {
    RunnerConfig {
        sifr_jobs: 1,
        rust_jobs: 1,
        run_jobs: 1,
        cargo_build_jobs: 1,
        cache: CacheConfig {
            enabled: true,
            root: root.to_path_buf(),
        },
    }
}

fn plan(dependency: &Path) -> SysrootDependencyPlan {
    let mut plan = sifr_driver::try_generate_standalone_dependency_plan(
        &HashSet::new(),
        &HashSet::new(),
        &sifr_driver::InteropBuildPlan::default(),
    )
    .unwrap();
    plan.retained_direct_dependencies.push(format!(
        "direct_dep = {{ path = {} }}",
        serde_json::to_string(&dependency.to_string_lossy()).unwrap(),
    ));
    plan
}

fn dependency(root: &Path, version: &str, value: u32) {
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("Cargo.toml"),
        format!("[package]\nname='direct_dep'\nversion='{version}'\nedition='2024'\n[workspace]\n"),
    )
    .unwrap();
    std::fs::write(
        root.join("src/lib.rs"),
        format!("pub fn value() -> u32 {{ {value} }}\n"),
    )
    .unwrap();
}

fn run(group: BatchGroup, config: &RunnerConfig) -> direct_execution::DirectOutcome {
    execute(group, config, Duration::from_secs(120)).unwrap()
}

fn dependency_fingerprints(root: &Path) -> Vec<(PathBuf, SystemTime)> {
    let fingerprints = root.join("standalone/cargo-v3/native-target/debug/.fingerprint");
    let mut result = Vec::new();
    for item in std::fs::read_dir(fingerprints).unwrap() {
        let path = item.unwrap().path().join("lib-direct_dep");
        if path.exists() {
            result.push((
                path.clone(),
                std::fs::metadata(path).unwrap().modified().unwrap(),
            ));
        }
    }
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

#[test]
fn standalone_reuses_dependencies_preserves_crate_root_and_invalidates_real_inputs() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let dep = root.join("dependency");
    dependency(&dep, "1.0.0", 1);
    let config = config(&root.join("cache"));
    let plan = plan(&dep);
    let source = r#"#![deny(dead_code)]
fn value() -> u32 { direct_dep::value() }
fn main() {
    assert_eq!(std::env::args().len(), 1);
    assert!(std::env::current_dir().unwrap().ends_with("work"));
    assert!(!std::path::Path::new("Cargo.toml").exists());
    println!("{}", crate::value());
}"#;
    let group = direct_group(source, "first", plan.clone());
    let first = run(group.clone(), &config);
    assert_eq!(first.capture.stdout, "1\n");
    assert!(!first.build.cache_hit);
    let fingerprints = dependency_fingerprints(&config.cache.root);
    assert_eq!(fingerprints.len(), 1);

    let other = run(direct_group(source, "second", plan.clone()), &config);
    assert_eq!(other.capture.stdout, "1\n");
    assert_ne!(first.build.artifact_path, other.build.artifact_path);
    assert_ne!(first.build.group_root, other.build.group_root);
    assert!(!other.build.cache_hit);
    assert_eq!(fingerprints, dependency_fingerprints(&config.cache.root));
    assert!(run(group.clone(), &config).build.cache_hit);

    let mut drifted = plan.clone();
    drifted.cache_fingerprint.push_str("-unrelated-plan");
    let drifted = direct_group(source, "first", drifted);
    assert_eq!(group.id, drifted.id);
    let reused = run(drifted.clone(), &config);
    assert!(reused.build.cache_hit);
    assert_eq!(reused.capture.stdout, "1\n");

    dependency(&dep, "1.0.1", 2);
    let changed = run(drifted.clone(), &config);
    assert!(!changed.build.cache_hit);
    assert_eq!(changed.capture.stdout, "2\n");
    assert!(run(drifted, &config).build.cache_hit);

    let edited = run(
        direct_group(
            &source.replace("crate::value()", "crate::value() + 10"),
            "first",
            plan,
        ),
        &config,
    );
    assert!(!edited.build.cache_hit);
    assert_eq!(edited.capture.stdout, "12\n");
    assert_ne!(first.build.artifact_path, edited.build.artifact_path);
}

#[test]
fn standalone_parallel_callers_keep_distinct_programs_and_working_directories() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let dep = root.join("dependency");
    dependency(&dep, "1.0.0", 3);
    let plan = plan(&dep);
    let config = config(&root.join("cache"));
    let outcomes = thread::scope(|scope| {
        let mut handles = Vec::new();
        for index in 0..4 {
            let group = direct_group(
                &format!(
                    "fn main() {{ assert_eq!(std::env::args().len(),1); std::fs::write(\"marker\", \"{index}\").unwrap(); println!(\"{{}}\", direct_dep::value()+{index}); }}"
                ),
                &format!("parallel-{index}"),
                plan.clone(),
            );
            let config = &config;
            handles.push(scope.spawn(move || run(group, config)));
        }
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>()
    });
    let mut binaries = BTreeSet::new();
    for (index, result) in outcomes.iter().enumerate() {
        assert_eq!(result.capture.stdout, format!("{}\n", 3 + index));
        assert_eq!(
            std::fs::read_to_string(result.build.group_root.join("work/marker")).unwrap(),
            index.to_string()
        );
        assert!(binaries.insert(result.build.artifact_path.clone().unwrap()));
    }
    assert_eq!(dependency_fingerprints(&config.cache.root).len(), 1);
}

#[test]
fn standalone_preserves_failures_rejects_busy_owner_and_recovers_missing_resolution() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let dep = root.join("dependency");
    dependency(&dep, "1.0.0", 1);
    let config = config(&root.join("cache"));
    let plan = plan(&dep);
    let group = direct_group(
        "fn main() { eprintln!(\"distinct failure\"); std::process::exit(7); }",
        "failure",
        plan.clone(),
    );
    let failed = run(group.clone(), &config);
    assert!(!failed.capture.status_ok);
    assert_eq!(failed.capture.exit_code, Some(7));
    assert_eq!(failed.capture.stderr, "distinct failure\n");
    assert!(failed.build.group_root.join("preserve").exists());

    std::fs::remove_file(failed.build.group_root.join("Cargo.lock")).unwrap();
    let recovered = run(group.clone(), &config);
    assert!(recovered.build.cache_hit);
    assert_eq!(recovered.capture.exit_code, Some(7));
    assert!(recovered.build.group_root.join("Cargo.lock").exists());

    let broken = direct_group("fn main() { missing_function(); }", "broken", plan.clone());
    let broken_root = config
        .cache
        .root
        .join("standalone/cargo-v3/groups")
        .join(&broken.id);
    assert!(execute(broken, &config, Duration::from_secs(120)).is_err());
    assert!(broken_root.join("preserve").exists());
    let repaired = run(
        direct_group("fn main() { println!(\"fixed\"); }", "broken", plan),
        &config,
    );
    assert_eq!(repaired.capture.stdout, "fixed\n");

    let owner = native_cache::NativeCache::acquire_with_wait(
        &config.cache.root.join("standalone"),
        Duration::from_secs(1),
    )
    .unwrap();
    let error = execute(group.clone(), &config, Duration::from_secs(120))
        .err()
        .unwrap();
    assert!(error.contains("cache is busy"), "{error}");
    drop(owner);
    assert!(execute(group, &config, Duration::ZERO).is_err());
}
