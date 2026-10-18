use super::*;

fn compiled_case_for_test(
    fixture: FixtureCase,
    rust_source: &str,
    required_features: HashSet<StdlibFeature>,
) -> CompiledCase {
    let used_stdlib_modules = HashSet::new();
    let interop = sifr_driver::InteropBuildPlan::default();
    let dependency_plan = sifr_driver::try_generate_standalone_dependency_plan(
        &used_stdlib_modules,
        &required_features,
        &interop,
    )
    .expect("test dependency plan should resolve");
    CompiledCase {
        fixture,
        rust_source: rust_source.to_string(),
        used_stdlib_modules,
        required_features,
        dependency_plan,
        _compile_duration_ms: 0,
    }
}

#[test]
pub(crate) fn test_failure_matching_consumes_failures_and_honors_columns() {
    let failures = vec![
        CompiledFailure {
            code: "SIFR-TYPE-0002".to_string(),
            message: "first".to_string(),
            column: Some(4),
        },
        CompiledFailure {
            code: "SIFR-TYPE-0002".to_string(),
            message: "second".to_string(),
            column: Some(8),
        },
    ];

    let expected = vec![
        CompileFailureExpectation {
            code: "SIFR-TYPE-0002".to_string(),
            column: Some(4),
        },
        CompileFailureExpectation {
            code: "SIFR-TYPE-0002".to_string(),
            column: Some(8),
        },
    ];
    assert!(match_compile_failure_expectations(&expected, &failures).is_ok());

    let duplicate_expectations = vec![
        CompileFailureExpectation {
            code: "SIFR-TYPE-0002".to_string(),
            column: Some(4),
        },
        CompileFailureExpectation {
            code: "SIFR-TYPE-0002".to_string(),
            column: Some(4),
        },
    ];
    assert!(match_compile_failure_expectations(&duplicate_expectations, &failures).is_err());

    let too_many_code_only_expectations = vec![
        CompileFailureExpectation {
            code: "SIFR-TYPE-0002".to_string(),
            column: None,
        },
        CompileFailureExpectation {
            code: "SIFR-TYPE-0002".to_string(),
            column: None,
        },
        CompileFailureExpectation {
            code: "SIFR-TYPE-0002".to_string(),
            column: None,
        },
    ];
    assert!(
        match_compile_failure_expectations(&too_many_code_only_expectations, &failures).is_err()
    );
}

#[test]
pub(crate) fn test_rendered_diagnostic_column_is_used_for_expect_error_matching() {
    let rendered = sifr_diagnostics::RenderedDiagnostic {
        code: "SIFR-TYPE-0002".to_string(),
        severity: sifr_diagnostics::Severity::Error,
        message: "type mismatch".to_string(),
        message_template: "{message}".to_string(),
        args: BTreeMap::new(),
        url: "https://docs.sifr.sh/errors/SIFR-TYPE-0002".to_string(),
        spans: vec![sifr_diagnostics::DiagnosticSpan {
            file: Some("unit.sifr".to_string()),
            byte_start: 4,
            byte_end: 5,
            line: Some(2),
            column: Some(9),
            end_line: Some(2),
            end_column: Some(10),
            is_primary: true,
            label: Some("value".to_string()),
            lines: Vec::new(),
        }],
        children: Vec::new(),
        help: None,
        suggestions: Vec::new(),
    };

    let failure = compiled_failure_from_rendered(rendered);
    assert_eq!(failure.column, Some(9));

    let expected = parse_expect_error_line("# expect-error[col=9]: SIFR-TYPE-0002")
        .unwrap()
        .unwrap();
    assert!(
        match_compile_failure_expectations(&[expected], std::slice::from_ref(&failure)).is_ok()
    );
    assert_eq!(
        compile_failures_to_messages(&[failure]),
        vec!["SIFR-TYPE-0002@col9: type mismatch".to_string()]
    );
}

#[test]
pub(crate) fn test_failure_summary_is_grouped_and_order_stable() {
    let cases = vec![
        FixtureExecution {
            name: "z_run".to_string(),
            status: Err(
                "FAIL [z_run]: stdout mismatch\n  expected: \"a\"\n  actual:   \"b\"".to_string(),
            ),
        },
        FixtureExecution {
            name: "a_compile".to_string(),
            status: Err("FAIL [a_compile]: sifr compilation failed:\n  unknown symbol".to_string()),
        },
        FixtureExecution {
            name: "b_build".to_string(),
            status: Err(
                "FAIL [b_build]: Rust compilation failed. Check build log: /tmp/log".to_string(),
            ),
        },
        FixtureExecution {
            name: "c_plan".to_string(),
            status: Err(
                "FAIL [c_plan]: failed to generate grouped crate source: bad crate layout"
                    .to_string(),
            ),
        },
        FixtureExecution {
            name: "ok_case".to_string(),
            status: Ok(()),
        },
    ];

    let mut reversed = cases.clone();
    reversed.reverse();

    let first = format_failures("new", &cases);
    let second = format_failures("new", &reversed);
    assert_eq!(first, second);
    assert!(first.contains("[compile] 1 failure(s)"));
    assert!(first.contains("[planning] 1 failure(s)"));
    assert!(first.contains("[build] 1 failure(s)"));
    assert!(first.contains("[run] 1 failure(s)"));
}

#[test]
pub(crate) fn test_failure_group_stage_classification_rules() {
    assert_eq!(
        failure_group("FAIL [x]: sifr compilation failed:\n  err"),
        "compile"
    );
    assert_eq!(
        failure_group("FAIL [x]: failed to generate grouped crate source: err"),
        "planning"
    );
    assert_eq!(
        failure_group("FAIL [x]: Rust compilation failed. Check build log: /tmp/log"),
        "build"
    );
    assert_eq!(
        failure_group("FAIL [x]: binary exited with error:\nboom"),
        "run"
    );
    assert_eq!(failure_group("FAIL [x]: unknown condition"), "other");
}

#[test]
pub(crate) fn test_report_signature_is_order_invariant() {
    let cases = vec![
        FixtureExecution {
            name: "case_b".to_string(),
            status: Err("FAIL [case_b]: binary exited with error:\nboom".to_string()),
        },
        FixtureExecution {
            name: "case_a".to_string(),
            status: Err("FAIL [case_a]: sifr compilation failed:\n  error".to_string()),
        },
        FixtureExecution {
            name: "case_ok".to_string(),
            status: Ok(()),
        },
    ];
    let report = PassReport {
        cases: cases.clone(),
    };

    let mut shuffled = cases;
    shuffled.swap(0, 1);
    let report_shuffled = PassReport { cases: shuffled };

    assert_eq!(
        report_signature("new", &report),
        report_signature("new", &report_shuffled)
    );
}

#[test]
pub(crate) fn test_report_signature_changes_on_failure_delta() {
    let base = PassReport {
        cases: vec![
            FixtureExecution {
                name: "case_a".to_string(),
                status: Err("FAIL [case_a]: sifr compilation failed:\n  err-a".to_string()),
            },
            FixtureExecution {
                name: "case_ok".to_string(),
                status: Ok(()),
            },
        ],
    };
    let changed = PassReport {
        cases: vec![
            FixtureExecution {
                name: "case_a".to_string(),
                status: Err("FAIL [case_a]: sifr compilation failed:\n  err-b".to_string()),
            },
            FixtureExecution {
                name: "case_ok".to_string(),
                status: Ok(()),
            },
        ],
    };

    assert_ne!(
        report_signature("new", &base),
        report_signature("new", &changed)
    );
}

pub(crate) fn smoke_rand_next(seed: &mut u64) -> u64 {
    // xorshift64* for deterministic smoke fuzz/property generation.
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}

pub(crate) fn smoke_ascii(seed: &mut u64, max_len: usize) -> String {
    let bound = u64::try_from(max_len.max(1)).unwrap_or(u64::MAX);
    let len = usize::try_from(smoke_rand_next(seed) % bound).unwrap_or(0);
    let mut output = String::with_capacity(len);
    for _ in 0..len {
        let bucket = (smoke_rand_next(seed) % 8) as u8;
        let ch = match bucket {
            0 => '\n',
            1 => '#',
            2 => ':',
            3 => ' ',
            _ => (b'a' + (smoke_rand_next(seed) % 26) as u8) as char,
        };
        output.push(ch);
    }
    output
}

#[test]
pub(crate) fn test_smoke_property_deterministic_hash_rules() {
    let mut seed = 0x5A17_C9D3_12EF_0042u64;
    let mut unique = BTreeSet::new();
    for _ in 0..256 {
        let sample = smoke_ascii(&mut seed, 64);
        let hash_a = deterministic_hash(&sample);
        let hash_b = deterministic_hash(&sample);
        assert_eq!(hash_a, hash_b);
        assert_eq!(hash_a.len(), 16);
        assert!(hash_a.chars().all(|ch| ch.is_ascii_hexdigit()));
        unique.insert(hash_a);
    }
    assert!(unique.len() > 200, "hash entropy smoke check regressed");
}

#[test]
pub(crate) fn test_smoke_fuzz_valid_expectation_extractors_no_panic() {
    let mut seed = 0xBADC_0FFE_EE11_2233u64;
    for _ in 0..512 {
        let mut sample = smoke_ascii(&mut seed, 120);
        if (smoke_rand_next(&mut seed) & 1) == 0 {
            sample.push_str("\n# expect-stderr: err");
        }
        if (smoke_rand_next(&mut seed) & 1) == 0 {
            sample.push_str("\n# expect-error: SIFR-TYPE-0002");
        }

        let _ = extract_expect_stderr(&sample);
        let _ = extract_compile_failure_expectations(&sample, Path::new("smoke.sifr"));
    }
}

#[test]
pub(crate) fn test_smoke_expectation_extractors_unicode_inputs() {
    let samples = [
        "# expect-stderr: λάθος\n# expect-error: SIFR-TYPE-0002",
        "# expect-error: SIFR-TYPE-0002\nこんにちは世界",
        "# expect-stderr: emoji-😀\nplain-text",
        "no-directives-🧪",
    ];

    for sample in samples {
        let _ = extract_expect_stderr(sample);
        let _ = extract_compile_failure_expectations(sample, Path::new("unicode.sifr"));
    }
}

#[test]
pub(crate) fn test_fixture_discovery_is_deterministic() {
    let root = env::temp_dir().join(format!(
        "sifr-e2e-discovery-{}",
        deterministic_hash(
            &SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
                .to_string()
        )
    ));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("zeta.sifr"), "fixture-zeta\n").unwrap();
    std::fs::write(root.join("alpha.sifr"), "fixture-alpha\n").unwrap();
    std::fs::write(root.join("beta.sifr"), "fixture-beta\n").unwrap();

    let names: Vec<_> = discover_fixtures(&root)
        .into_iter()
        .map(|fixture| fixture.name)
        .collect();
    let _ = std::fs::remove_dir_all(root);
    assert_eq!(
        names,
        vec!["alpha".to_string(), "beta".to_string(), "zeta".to_string()]
    );
}

#[test]
pub(crate) fn test_parse_fixture_selection_manifest_requires_non_empty_fixture_names() {
    let selected = parse_fixture_selection_manifest(r#"{"fixture_names":["beta","alpha","beta"]}"#)
        .expect("fixture manifest should parse");
    assert_eq!(
        selected,
        BTreeSet::from(["alpha".to_string(), "beta".to_string(),])
    );
    assert!(parse_fixture_selection_manifest(r#"{"fixture_names":[]}"#).is_err());
    assert!(parse_fixture_selection_manifest(r#"{"fixture_names":[""]}"#).is_err());
}

#[test]
pub(crate) fn test_filter_fixtures_by_selection_rejects_unknown_fixture_names() {
    let fixtures = vec![
        FixtureCase {
            name: "alpha".to_string(),
            path: PathBuf::from("alpha.sifr"),
            source: String::new(),
            source_hash: "a".to_string(),
            _expected_stderr: Vec::new(),
        },
        FixtureCase {
            name: "beta".to_string(),
            path: PathBuf::from("beta.sifr"),
            source: String::new(),
            source_hash: "b".to_string(),
            _expected_stderr: Vec::new(),
        },
    ];

    let filtered =
        filter_fixtures_by_selection(fixtures.clone(), &BTreeSet::from(["beta".to_string()]))
            .expect("fixture filtering should succeed");
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "beta");

    let error = filter_fixtures_by_selection(fixtures, &BTreeSet::from(["missing".to_string()]))
        .expect_err("unknown fixtures should be rejected");
    assert!(error.contains("unknown pass fixtures"));
}

#[test]
pub(crate) fn test_dependency_fingerprint_and_cache_key_determinism() {
    let fixture = DependencyFingerprint {
        dependency_inputs: "[stdlib]\na\nb\n[features]\nx\ny\n".to_string(),
        resolved_plan: "plan-a".to_string(),
    };
    let same = fixture.clone();
    assert_eq!(fixture.signature(), same.signature());
    assert_eq!(fixture.hash(), same.hash());

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let case = FixtureCase {
        name: "fixture-a".to_string(),
        path: PathBuf::from("tests/e2e/pass/fixture-a.sifr"),
        source: "print('x')".to_string(),
        source_hash: deterministic_hash(&format!("hash-{}", now.as_nanos())),
        _expected_stderr: Vec::new(),
    };
    let compiled = compiled_case_for_test(case.clone(), "fn main() {}", HashSet::new());
    let group = build_group_sources(vec![compiled]).unwrap();
    let toolchain = toolchain_info();
    let env_signature = cache_env_signature();
    let key_a = cache_key_for_group(&group, &toolchain, &env_signature);
    let key_b = cache_key_for_group(&group, &toolchain, "different");
    assert_ne!(key_a, key_b);
    assert!(!group.id.is_empty());
}

#[test]
pub(crate) fn test_batch_group_dispatch_uses_entry_termination_trait() {
    let case = FixtureCase {
        name: "async_result_fixture".to_string(),
        path: PathBuf::from("tests/e2e/pass/async_result_fixture.sifr"),
        source: "async def main() -> Result[None, ValueError]:\n    return None".to_string(),
        source_hash: "async-result".to_string(),
        _expected_stderr: Vec::new(),
    };
    let compiled = compiled_case_for_test(
        case,
        "#[tokio::main]\nasync fn main() -> Result<(), ValueError> {\n    Ok(())\n}\n",
        HashSet::from([StdlibFeature::Tokio]),
    );
    let group = build_group_sources(vec![compiled]).expect("batch group");

    assert!(
        group
            .generated_main
            .contains("impl<E: std::fmt::Debug> __SifrBatchTermination for Result<(), E>")
    );
    assert!(
        group
            .generated_main
            .contains("=> __SifrBatchTermination::__sifr_finish(")
    );
    assert!(
        group
            .generated_main
            .contains("pub fn sifr_case_async_result_fixture_0_")
    );
    assert!(
        group
            .generated_main
            .contains("super::__SifrBatchTermination::__sifr_finish(")
    );
    assert!(!group.generated_main.contains("pub async fn"));
}

pub(crate) fn sample_cache_root(label: &str) -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    env::temp_dir().join(format!("sifr-e2e-cache-test-{label}-{}", now.as_nanos()))
}

#[test]
pub(crate) fn test_prune_cache_manifest_removes_expired_entries_and_orphan_groups() {
    let root = sample_cache_root("prune-expired");
    let groups_root = cache_groups_root(&root);
    std::fs::create_dir_all(&groups_root).expect("cache groups root");

    let stale_group = "stale-group";
    let live_group = "live-group";
    let orphan_group = "orphan-group";
    let missing_group = "missing-group";

    for group in [stale_group, live_group, orphan_group] {
        let group_root = cache_group_path(&root, group);
        std::fs::create_dir_all(group_root.join("target")).expect("group root");
        std::fs::write(group_root.join("completed"), b"completed").unwrap();
    }

    let now_unix_secs = 20_000;
    let manifest = CacheManifest {
        schema_version: E2E_CACHE_SCHEMA_VERSION,
        entries: BTreeMap::from([
            (
                "stale-key".to_string(),
                CacheEntry {
                    schema_version: E2E_CACHE_SCHEMA_VERSION,
                    cache_key: "stale-key".to_string(),
                    group_id: stale_group.to_string(),
                    group_fingerprint: "stale".to_string(),
                    group_rust_hash: "stale".to_string(),
                    fixture_sources: Vec::new(),
                    compiler_signature: "toolchain".to_string(),
                    rustc_v: "rustc".to_string(),
                    rustc_vv: "rustc-vv".to_string(),
                    cargo_v: "cargo".to_string(),
                    target: "target".to_string(),
                    os: "os".to_string(),
                    arch: "arch".to_string(),
                    env_signature: "env".to_string(),
                    artifact_path: cache_group_path(&root, stale_group)
                        .join("target")
                        .display()
                        .to_string(),
                    build_log_path: None,
                    built_at_unix_secs: now_unix_secs - E2E_CACHE_TTL_SECS - 1,
                },
            ),
            (
                "live-key".to_string(),
                CacheEntry {
                    schema_version: E2E_CACHE_SCHEMA_VERSION,
                    cache_key: "live-key".to_string(),
                    group_id: live_group.to_string(),
                    group_fingerprint: "live".to_string(),
                    group_rust_hash: "live".to_string(),
                    fixture_sources: Vec::new(),
                    compiler_signature: "toolchain".to_string(),
                    rustc_v: "rustc".to_string(),
                    rustc_vv: "rustc-vv".to_string(),
                    cargo_v: "cargo".to_string(),
                    target: "target".to_string(),
                    os: "os".to_string(),
                    arch: "arch".to_string(),
                    env_signature: "env".to_string(),
                    artifact_path: cache_group_path(&root, live_group)
                        .join("target")
                        .display()
                        .to_string(),
                    build_log_path: None,
                    built_at_unix_secs: now_unix_secs,
                },
            ),
            (
                "missing-key".to_string(),
                CacheEntry {
                    schema_version: E2E_CACHE_SCHEMA_VERSION,
                    cache_key: "missing-key".to_string(),
                    group_id: missing_group.to_string(),
                    group_fingerprint: "missing".to_string(),
                    group_rust_hash: "missing".to_string(),
                    fixture_sources: Vec::new(),
                    compiler_signature: "toolchain".to_string(),
                    rustc_v: "rustc".to_string(),
                    rustc_vv: "rustc-vv".to_string(),
                    cargo_v: "cargo".to_string(),
                    target: "target".to_string(),
                    os: "os".to_string(),
                    arch: "arch".to_string(),
                    env_signature: "env".to_string(),
                    artifact_path: cache_group_path(&root, missing_group)
                        .join("target")
                        .display()
                        .to_string(),
                    build_log_path: None,
                    built_at_unix_secs: now_unix_secs,
                },
            ),
        ]),
    };

    let pruned = prune_cache_manifest(&root, manifest, now_unix_secs, |_| Ok(()));
    assert_eq!(pruned.entries.len(), 1);
    assert!(pruned.entries.contains_key("live-key"));
    assert!(cache_group_path(&root, live_group).is_dir());
    assert!(!cache_group_path(&root, stale_group).exists());
    assert!(!cache_group_path(&root, orphan_group).exists());
    assert!(!pruned.entries.contains_key("missing-key"));

    let _ = std::fs::remove_dir_all(root);
}

#[test]
pub(crate) fn test_prune_cache_manifest_keeps_shared_live_group_for_fresh_entry() {
    let root = sample_cache_root("prune-shared");
    let shared_group = "shared-group";
    std::fs::create_dir_all(cache_group_path(&root, shared_group).join("target"))
        .expect("shared group root");

    let now_unix_secs = 30_000;
    let manifest = CacheManifest {
        schema_version: E2E_CACHE_SCHEMA_VERSION,
        entries: BTreeMap::from([
            (
                "old-key".to_string(),
                CacheEntry {
                    schema_version: E2E_CACHE_SCHEMA_VERSION,
                    cache_key: "old-key".to_string(),
                    group_id: shared_group.to_string(),
                    group_fingerprint: "group".to_string(),
                    group_rust_hash: "hash".to_string(),
                    fixture_sources: Vec::new(),
                    compiler_signature: "toolchain".to_string(),
                    rustc_v: "rustc".to_string(),
                    rustc_vv: "rustc-vv".to_string(),
                    cargo_v: "cargo".to_string(),
                    target: "target".to_string(),
                    os: "os".to_string(),
                    arch: "arch".to_string(),
                    env_signature: "env".to_string(),
                    artifact_path: cache_group_path(&root, shared_group)
                        .join("target")
                        .display()
                        .to_string(),
                    build_log_path: None,
                    built_at_unix_secs: now_unix_secs - E2E_CACHE_TTL_SECS - 1,
                },
            ),
            (
                "fresh-key".to_string(),
                CacheEntry {
                    schema_version: E2E_CACHE_SCHEMA_VERSION,
                    cache_key: "fresh-key".to_string(),
                    group_id: shared_group.to_string(),
                    group_fingerprint: "group".to_string(),
                    group_rust_hash: "hash".to_string(),
                    fixture_sources: Vec::new(),
                    compiler_signature: "toolchain".to_string(),
                    rustc_v: "rustc".to_string(),
                    rustc_vv: "rustc-vv".to_string(),
                    cargo_v: "cargo".to_string(),
                    target: "target".to_string(),
                    os: "os".to_string(),
                    arch: "arch".to_string(),
                    env_signature: "env".to_string(),
                    artifact_path: cache_group_path(&root, shared_group)
                        .join("target")
                        .display()
                        .to_string(),
                    build_log_path: None,
                    built_at_unix_secs: now_unix_secs,
                },
            ),
        ]),
    };

    let pruned = prune_cache_manifest(&root, manifest, now_unix_secs, |_| Ok(()));
    assert_eq!(pruned.entries.len(), 1);
    assert!(pruned.entries.contains_key("fresh-key"));
    assert!(cache_group_path(&root, shared_group).is_dir());

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn test_pruning_preserves_failed_interrupted_unowned_groups_and_native_dependencies() {
    let root = sample_cache_root("retention");
    for name in ["failed", "interrupted", "unmarked", "completed"] {
        let group = cache_group_path(&root, name);
        std::fs::create_dir_all(&group).unwrap();
        if name != "unmarked" {
            std::fs::write(group.join("completed"), b"done").unwrap();
        }
        if name == "failed" {
            std::fs::write(group.join("preserve"), b"failure").unwrap();
        }
        if name == "interrupted" {
            std::fs::write(group.join("pending"), b"pending").unwrap();
        }
    }
    let dependency = root.join("native-target/debug/deps/retained");
    std::fs::create_dir_all(dependency.parent().unwrap()).unwrap();
    std::fs::write(&dependency, b"native dependency").unwrap();
    prune_cache_manifest(
        &root,
        CacheManifest {
            schema_version: E2E_CACHE_SCHEMA_VERSION,
            entries: BTreeMap::new(),
        },
        20000,
        |_| Ok(()),
    );
    for name in ["failed", "interrupted", "unmarked"] {
        assert!(cache_group_path(&root, name).is_dir());
    }
    assert!(!cache_group_path(&root, "completed").exists());
    assert!(dependency.is_file());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn test_no_cache_retains_records_but_still_validates_cargo_and_artifact_integrity() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let lease = native_cache::NativeCache::acquire(&root).unwrap();
    let config = RunnerConfig {
        sifr_jobs: 1,
        rust_jobs: 2,
        run_jobs: 1,
        cargo_build_jobs: 1,
        cache: CacheConfig {
            enabled: false,
            root: lease.root.clone(),
        },
    };
    let old_group = cache_group_path(&config.cache.root, "completed-old-group");
    sifr_cache_storage::directory(&old_group).unwrap();
    std::fs::write(old_group.join("completed"), b"old evidence").unwrap();
    let manifest_path = cache_manifest_path(&config.cache.root);
    let sentinel = b"existing invalid success manifest must remain untouched";
    native_cache::write_changed(&manifest_path, sentinel).unwrap();
    let manifest = initial_cache_manifest(&config);
    assert!(manifest.entries.is_empty());
    let case = FixtureCase {
        name: "no_cache_control".into(),
        path: PathBuf::from("no_cache_control.sifr"),
        source: "def main(): pass".into(),
        source_hash: "no-cache-control".into(),
        _expected_stderr: Vec::new(),
    };
    let mut group = build_group_sources(vec![compiled_case_for_test(
        case,
        "fn main() {}",
        HashSet::new(),
    )])
    .unwrap();
    let toolchain = toolchain_info();
    let env_signature = cache_env_signature();
    let (first, _) = build_batch_suite(
        &[group.clone()],
        &config,
        &toolchain,
        &env_signature,
        &manifest,
    );
    assert!(first[0].build_error.is_none(), "{:?}", first[0].build_error);
    let (fresh, _) = build_batch_suite(
        &[group.clone()],
        &config,
        &toolchain,
        &env_signature,
        &manifest,
    );
    assert!(fresh[0].cache_hit);
    std::fs::write(
        fresh[0].artifact_path.as_ref().unwrap(),
        b"damaged native executable",
    )
    .unwrap();
    let (corrupt, _) = build_batch_suite(
        &[group.clone()],
        &config,
        &toolchain,
        &env_signature,
        &manifest,
    );
    assert!(corrupt[0].artifact_path.is_none());
    assert!(corrupt[0].build_error.is_some());
    let failure = run_batch_outcomes(&corrupt[0]);
    assert!(
        failure[0]
            .status
            .as_ref()
            .unwrap_err()
            .contains(&corrupt[0].group_root.display().to_string())
    );
    group
        .generated_main
        .push_str("\ncompile_error!(\"no-cache must validate Cargo\");\n");
    let (invalid, _) = build_batch_suite(&[group], &config, &toolchain, &env_signature, &manifest);
    assert!(invalid[0].artifact_path.is_none());
    assert!(invalid[0].build_error.is_some());
    assert_eq!(std::fs::read(manifest_path).unwrap(), sentinel);
    assert!(old_group.join("completed").is_file());
    assert!(config.cache.root.join("native-target/debug/deps").is_dir());
}

#[test]
fn test_failed_native_retirement_keeps_completed_group_evidence() {
    let root = sample_cache_root("retirement-failed");
    let group = cache_group_path(&root, "completed");
    std::fs::create_dir_all(&group).unwrap();
    std::fs::write(group.join("completed"), b"done").unwrap();
    prune_cache_manifest(
        &root,
        CacheManifest {
            schema_version: E2E_CACHE_SCHEMA_VERSION,
            entries: BTreeMap::new(),
        },
        20000,
        |_| Err("intentional retirement failure".into()),
    );
    assert!(group.is_dir());
    std::fs::remove_dir_all(root).unwrap();
}
