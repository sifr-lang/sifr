use super::*;
use std::time::Duration;

fn case(name: &str, rust: &str) -> CompiledCase {
    harness_behavior_tests::compiled_case_for_test(
        FixtureCase {
            name: name.into(),
            path: PathBuf::from(format!("{name}.sifr")),
            source: "def main(): pass".into(),
            source_hash: name.into(),
            _expected_stderr: vec![],
        },
        rust,
        HashSet::new(),
    )
}

fn capture(code: Option<i32>, target: &str, diagnostic: Option<&str>) -> CommandCapture {
    CommandCapture { exit_code: code, status_ok: false, stderr: String::new(),
        stdout: serde_json::json!({"reason":"compiler-message", "target":{"name":target},
            "message":{"level":"error","code":diagnostic.map(|code| serde_json::json!({"code":code}))}}).to_string() }
}

#[test]
fn test_isolation_classifies_typed_source_errors_only() {
    assert!(batch_isolation::is_source_failure(
        &capture(Some(101), "group", Some("E0599")),
        "group"
    ));
    for value in [
        capture(None, "group", Some("E0599")),
        capture(Some(124), "group", Some("E0599")),
        capture(Some(101), "dependency", Some("E0599")),
        capture(Some(101), "group", None),
        capture(Some(101), "group", Some("E0463")),
    ] {
        assert!(!batch_isolation::is_source_failure(&value, "group"));
    }
    let mut mixed = capture(Some(101), "group", Some("E0599"));
    mixed.stdout.push('\n');
    mixed
        .stdout
        .push_str(&capture(Some(101), "group", None).stdout);
    assert!(!batch_isolation::is_source_failure(&mixed, "group"));
}

#[test]
fn test_isolation_preserves_failure_and_recovers_healthy_sibling() {
    let temp = tempfile::tempdir().unwrap().keep();
    let lease = native_cache::NativeCache::acquire(&temp).unwrap();
    let config = RunnerConfig {
        sifr_jobs: 1,
        rust_jobs: 1,
        run_jobs: 1,
        cargo_build_jobs: 1,
        cache: CacheConfig {
            enabled: false,
            root: lease.root.clone(),
        },
    };
    let toolchain = toolchain_info();
    let manifest = CacheManifest {
        schema_version: E2E_CACHE_SCHEMA_VERSION,
        entries: BTreeMap::new(),
    };
    let group = build_group_sources(vec![
        case("broken", "fn main() { missing_function(); }"),
        case("healthy", "fn main() {}"),
    ])
    .unwrap();
    let (original, next) = build_batch_suite(
        &lease,
        &[group.clone()],
        &config,
        &toolchain,
        "control",
        &manifest,
    );
    assert!(original[0].source_failure, "{:?}", original[0].build_error);
    let preserved = original[0].group_root.clone();
    let (isolated, failures) = batch_isolation::isolate_failed_batches(
        &lease,
        original.clone(),
        &config,
        &toolchain,
        "control",
        &next,
        Duration::from_secs(60),
    );
    assert_eq!(isolated.len(), 2);
    assert_eq!(failures.len(), 1);
    let (cases, _) = run_batch_suite(&lease, &isolated, &config);
    assert_eq!(
        cases.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(),
        vec!["broken", "healthy"]
    );
    assert!(cases[0].status.is_err());
    assert!(cases[1].status.is_ok());
    assert!(preserved.join("preserve").is_file());
    let report = PassReport {
        cases,
        batch_failures: failures,
    };
    assert_eq!(report.failed_count(), 1);
    assert!(std::panic::catch_unwind(|| assert_report("control", &report)).is_err());
    let (capped, retained) = batch_isolation::isolate_failed_batches(
        &lease,
        original,
        &config,
        &toolchain,
        "control",
        &next,
        Duration::ZERO,
    );
    assert_eq!(retained.len(), 1);
    assert_eq!(capped.len(), 2);
    assert!(
        capped
            .iter()
            .all(|c| c.build_ms == 0 && c.artifact_path.is_none())
    );
    // Combined source alone fails: isolated passes cannot erase interaction failure.
    let mut interaction = build_group_sources(vec![
        case("one", "fn main() {}"),
        case("two", "fn main() {}"),
    ])
    .unwrap();
    interaction
        .generated_main
        .push_str("\nfn interaction_error() { missing_function(); }\n");
    let (failed, next) = build_batch_suite(
        &lease,
        &[interaction],
        &config,
        &toolchain,
        "control",
        &manifest,
    );
    let (isolated, failures) = batch_isolation::isolate_failed_batches(
        &lease,
        failed,
        &config,
        &toolchain,
        "control",
        &next,
        Duration::from_secs(60),
    );
    let (cases, _) = run_batch_suite(&lease, &isolated, &config);
    assert!(cases.iter().all(|c| c.status.is_ok()));
    assert!(
        std::panic::catch_unwind(|| assert_report(
            "interaction",
            &PassReport {
                cases,
                batch_failures: failures
            }
        ))
        .is_err()
    );
    // An uncoded parse error suppresses isolation, not unrelated original groups.
    let mut parse = build_group_sources(vec![case("parse", "fn main() {}")]).unwrap();
    parse.generated_main.push_str("\nfn malformed(\n");
    let healthy = build_group_sources(vec![case("after_parse", "fn main() {}")]).unwrap();
    let (mixed, _) = build_batch_suite(
        &lease,
        &[parse, healthy],
        &config,
        &toolchain,
        "control",
        &manifest,
    );
    assert!(mixed[0].build_error.is_some());
    assert!(!mixed[0].source_failure && !mixed[0].infrastructure_failure);
    assert!(
        mixed[1].artifact_path.is_some(),
        "{:?}",
        mixed[1].build_error
    );
    assert!(run_batch_outcomes(&lease, &mixed[1])[0].status.is_ok());
    // Multiple source groups exhaust one shared budget; no member disappears.
    let mut first = batch_isolation::blocked(group.clone(), &config, "injected source failure");
    first.source_failure = true;
    first.infrastructure_failure = false;
    let mut second = first.clone();
    second.group.id.push_str("-second");
    for member in &mut second.group.cases {
        member.fixture.name.push_str("_second");
    }
    let (capped, retained) = batch_isolation::isolate_failed_batches(
        &lease,
        vec![first, second],
        &config,
        &toolchain,
        "control",
        &manifest,
        Duration::ZERO,
    );
    assert_eq!(retained.len(), 2);
    assert_eq!(capped.len(), 4);
    assert!(
        capped
            .iter()
            .all(|row| row.artifact_path.is_none() && row.build_ms == 0)
    );
    let unavailable = batch_isolation::blocked(group, &config, "disk unavailable");
    let (attempts, _) = batch_isolation::isolate_failed_batches(
        &lease,
        vec![unavailable],
        &config,
        &toolchain,
        "control",
        &manifest,
        Duration::from_secs(60),
    );
    assert_eq!(attempts.len(), 1);
    assert_eq!(attempts[0].group.cases.len(), 2);
    assert_eq!(attempts[0].build_ms, 0);
}

#[test]
fn test_canonical_compiler_errors_stop_before_native_build() {
    let source = "def main():\n    unknown_lowering_control\n";
    let errors =
        match sifr_driver::compile_with_metadata(&sifr_driver::CompilerContext::for_test(), source)
        {
            sifr_driver::CompileResultFull::Errors { errors } => errors,
            sifr_driver::CompileResultFull::Success { .. } => {
                panic!("undefined name compiled successfully")
            }
        };
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("unknown_lowering_control"))
    );
    let fixture = FixtureCase {
        name: "canonical_error_control".into(),
        path: PathBuf::from("canonical_error_control.sifr"),
        source: source.into(),
        source_hash: deterministic_hash(source),
        _expected_stderr: Vec::new(),
    };
    let (result, invocations) =
        sifr_driver::capture_cargo_invocations(|| compile_fixture(&fixture));
    assert!(
        invocations.is_empty(),
        "failed source reached Cargo: {invocations:?}"
    );
    let failure = result.expect_err("compiler errors must not produce a native batch member");
    assert!(failure.starts_with("sifr compilation failed:"));
    for error in errors {
        assert!(
            failure.contains(&error.message),
            "lost canonical diagnostic: {failure}"
        );
    }
}

#[test]
fn test_parallel_worker_panic_cannot_drop_selected_outcome() {
    assert!(
        std::panic::catch_unwind(|| run_in_parallel(&[1, 2, 3], 2, |value| {
            assert_ne!(*value, 2, "injected worker failure");
            *value
        }))
        .is_err()
    );
}
