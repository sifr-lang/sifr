use super::*;
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
        url: "https://sifr.sh/docs/errors/SIFR-TYPE-0002".to_string(),
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
pub(crate) fn test_failure_group_stage_classification_contract() {
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
    let len = (smoke_rand_next(seed) as usize) % max_len.max(1);
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
pub(crate) fn test_smoke_property_deterministic_hash_contract() {
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
            sample.push_str("\n# expect-stdout: ok");
        }
        if (smoke_rand_next(&mut seed) & 1) == 0 {
            sample.push_str("\n# expect-stderr: err");
        }
        if (smoke_rand_next(&mut seed) & 1) == 0 {
            sample.push_str("\n# expect-error: SIFR-TYPE-0002");
        }

        let _ = extract_expect_stdout(&sample);
        let _ = extract_expect_stderr(&sample);
        let _ = extract_compile_failure_expectations(&sample, Path::new("smoke.sifr"));
    }
}

#[test]
pub(crate) fn test_smoke_expectation_extractors_unicode_inputs() {
    let samples = [
        "# expect-stdout: مرحبا\n# expect-stderr: λάθος\n# expect-error: SIFR-TYPE-0002",
        "# expect-stdout: こんにちは世界\n# expect-error: SIFR-TYPE-0002",
        "# expect-stderr: emoji-😀\nplain-text",
        "no-directives-🧪",
    ];

    for sample in samples {
        let _ = extract_expect_stdout(sample);
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
    std::fs::write(root.join("zeta.sifr"), "# expect-stdout: ok\n").unwrap();
    std::fs::write(root.join("alpha.sifr"), "# expect-stdout: ok\n").unwrap();
    std::fs::write(root.join("beta.sifr"), "# expect-stdout: ok\n").unwrap();

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
            expected_stdout: None,
            _expected_stderr: Vec::new(),
        },
        FixtureCase {
            name: "beta".to_string(),
            path: PathBuf::from("beta.sifr"),
            source: String::new(),
            source_hash: "b".to_string(),
            expected_stdout: None,
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
        stdlib_modules: normalize_dependency_set(
            vec!["a", "b", "a"].into_iter().map(str::to_string),
        ),
        required_crates: normalize_dependency_set(
            vec!["x", "y", "x"].into_iter().map(str::to_string),
        ),
    };
    let same = DependencyFingerprint {
        stdlib_modules: normalize_dependency_set(vec!["b", "a"].into_iter().map(str::to_string)),
        required_crates: normalize_dependency_set(vec!["y", "x"].into_iter().map(str::to_string)),
    };
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
        expected_stdout: None,
        _expected_stderr: Vec::new(),
    };
    let compiled = CompiledCase {
        fixture: case.clone(),
        rust_source: "fn main() {}".to_string(),
        stdlib_modules: BTreeSet::new(),
        required_crates: BTreeSet::new(),
        _compile_duration_ms: 0,
    };
    let group = build_group_sources(vec![compiled]).unwrap();
    let toolchain = toolchain_info();
    let env_signature = cache_env_signature();
    let key_a = cache_key_for_group(&group, &toolchain, &env_signature);
    let key_b = cache_key_for_group(&group, &toolchain, "different");
    assert_ne!(key_a, key_b);
    assert!(group.id.len() > 0);
}

#[test]
pub(crate) fn test_batch_group_dispatch_uses_entry_termination_trait() {
    let case = FixtureCase {
        name: "async_result_fixture".to_string(),
        path: PathBuf::from("tests/e2e/pass/async_result_fixture.sifr"),
        source: "async def main() -> Result[None, ValueError]:\n    return None".to_string(),
        source_hash: "async-result".to_string(),
        expected_stdout: None,
        _expected_stderr: Vec::new(),
    };
    let compiled = CompiledCase {
        fixture: case,
        rust_source: "#[tokio::main]\nasync fn main() -> Result<(), ValueError> {\n    Ok(())\n}\n"
            .to_string(),
        stdlib_modules: BTreeSet::new(),
        required_crates: BTreeSet::new(),
        _compile_duration_ms: 0,
    };
    let group = build_group_sources(vec![compiled]).expect("batch group");

    assert!(group
        .generated_main
        .contains("impl<E: std::fmt::Debug> __SifrBatchTermination for Result<(), E>"));
    assert!(group
        .generated_main
        .contains("=> __SifrBatchTermination::__sifr_finish("));
    assert!(group
        .generated_main
        .contains("pub fn sifr_case_async_result_fixture_0_"));
    assert!(group
        .generated_main
        .contains("super::__SifrBatchTermination::__sifr_finish("));
    assert!(!group.generated_main.contains("pub async fn"));
}

#[test]
pub(crate) fn test_generate_cargo_toml_tomllib_uses_preserve_order_feature() {
    let stdlib_modules = normalize_dependency_set(vec!["sifr.tomllib".to_string()].into_iter());
    let required_crates = BTreeSet::new();

    let cargo_toml = generate_cargo_toml(&stdlib_modules, &required_crates, "sifr_output");
    assert!(cargo_toml.contains("toml = { version = \"1.1.2\", features = [\"preserve_order\"] }"));
}

#[test]
pub(crate) fn test_generate_cargo_toml_required_toml_uses_preserve_order_feature() {
    let stdlib_modules = BTreeSet::new();
    let required_crates = normalize_dependency_set(vec!["toml".to_string()].into_iter());

    let cargo_toml = generate_cargo_toml(&stdlib_modules, &required_crates, "sifr_output");
    assert!(cargo_toml.contains("toml = { version = \"1.1.2\", features = [\"preserve_order\"] }"));
}

#[test]
pub(crate) fn test_generate_cargo_toml_required_sifr_runtime_uses_path_dependency() {
    let stdlib_modules = BTreeSet::new();
    let required_crates = normalize_dependency_set(vec!["sifr_runtime".to_string()]);

    let cargo_toml = generate_cargo_toml(&stdlib_modules, &required_crates, "sifr_output");
    assert!(cargo_toml.contains("sifr_runtime = { path = "));
}

#[test]
pub(crate) fn test_generate_cargo_toml_text_i18n_modules_enable_runtime_features() {
    let unicode_modules = normalize_dependency_set(vec!["sifr.unicode".to_string()].into_iter());
    let i18n_modules = normalize_dependency_set(vec!["sifr.i18n".to_string()].into_iter());
    let combined_modules = normalize_dependency_set(
        vec![
            "sifr.encoding".to_string(),
            "sifr.unicode".to_string(),
            "sifr.i18n".to_string(),
        ]
        .into_iter(),
    );
    let required_crates = normalize_dependency_set(vec!["sifr_runtime".to_string()]);

    let unicode_toml = generate_cargo_toml(&unicode_modules, &required_crates, "sifr_output");
    assert!(unicode_toml.contains("sifr_runtime = { path = "));
    assert!(unicode_toml.contains("features = [\"unicode\"]"));
    assert!(unicode_toml.contains("unicode-segmentation = \"1.13.3\""));

    let i18n_toml = generate_cargo_toml(&i18n_modules, &required_crates, "sifr_output");
    assert!(i18n_toml.contains("sifr_runtime = { path = "));
    assert!(i18n_toml.contains("features = [\"i18n\"]"));
    assert!(i18n_toml.contains("icu_locale = \"2.2.0\""));

    let combined_toml = generate_cargo_toml(&combined_modules, &required_crates, "sifr_output");
    assert!(combined_toml.contains("features = [\"i18n\", \"unicode\"]"));
    assert!(combined_toml.contains("encoding_rs = \"0.8.35\""));
    assert!(combined_toml.matches("sifr_runtime = ").count() == 1);
}

#[test]
pub(crate) fn test_generate_cargo_toml_required_tokio_uses_runtime_features() {
    let stdlib_modules = BTreeSet::new();
    let required_crates = normalize_dependency_set(vec!["tokio".to_string()]);

    let cargo_toml = generate_cargo_toml(&stdlib_modules, &required_crates, "sifr_output");
    assert!(cargo_toml.contains(
        "tokio = { version = \"1.52.3\", features = [\"macros\", \"process\", \"rt\", \"sync\", \"time\"] }"
    ));
}

pub(crate) fn sample_cache_entry(
    group: &BatchGroup,
    toolchain: &ToolchainInfo,
    env_signature: &str,
) -> CacheEntry {
    let key = cache_key_for_group(group, toolchain, env_signature);
    let artifact_path = env::temp_dir().join("sifr-e2e-cache-key-sample").join(&key);

    std::fs::create_dir_all(&artifact_path).expect("cache sample dir");
    let binary_path = artifact_path.join(format!("{}.bin", group.package_name));
    std::fs::write(&binary_path, b"cache-test").expect("cache sample binary");

    CacheEntry {
        schema_version: E2E_CACHE_SCHEMA_VERSION,
        cache_key: key.clone(),
        group_id: group.id.clone(),
        group_fingerprint: group.fingerprint.signature(),
        group_rust_hash: group.generated_rust_hash.clone(),
        fixture_sources: group
            .cases
            .iter()
            .map(|case| FixtureSourceHash {
                fixture: case.fixture.name.clone(),
                hash: case.fixture.source_hash.clone(),
            })
            .collect(),
        compiler_signature: toolchain.signature(),
        rustc_v: toolchain.rustc_v.clone(),
        rustc_vv: toolchain.rustc_vv.clone(),
        cargo_v: toolchain.cargo_v.clone(),
        target: toolchain.target.clone(),
        os: toolchain.os.clone(),
        arch: toolchain.arch.clone(),
        env_signature: env_signature.to_string(),
        artifact_path: binary_path.to_string_lossy().to_string(),
        build_log_path: None,
        built_at_unix_secs: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or_default(),
    }
}

pub(crate) fn sample_cache_root(label: &str) -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    env::temp_dir().join(format!("sifr-e2e-cache-test-{label}-{}", now.as_nanos()))
}

#[test]
pub(crate) fn test_cache_entry_invalidation_rules() {
    let case = FixtureCase {
        name: "fixture-cache-a".to_string(),
        path: PathBuf::from("tests/e2e/pass/fixture-cache-a.sifr"),
        source: "print('x')".to_string(),
        source_hash: deterministic_hash("cache-fixture-a"),
        expected_stdout: None,
        _expected_stderr: Vec::new(),
    };
    let compiled = CompiledCase {
        fixture: case.clone(),
        rust_source: "fn main() {}".to_string(),
        stdlib_modules: BTreeSet::new(),
        required_crates: BTreeSet::new(),
        _compile_duration_ms: 0,
    };
    let group = build_group_sources(vec![compiled]).expect("cache sample group");
    let toolchain = toolchain_info();
    let env_signature = cache_env_signature();

    let valid_entry = sample_cache_entry(&group, &toolchain, &env_signature);
    let cache_key = cache_key_for_group(&group, &toolchain, &env_signature);
    assert!(cache_entry_valid(
        &valid_entry,
        &group,
        &cache_key,
        &toolchain,
        &env_signature,
    ));

    let mut invalid_schema = valid_entry.clone();
    invalid_schema.schema_version = E2E_CACHE_SCHEMA_VERSION + 1;
    assert!(!cache_entry_valid(
        &invalid_schema,
        &group,
        &cache_key,
        &toolchain,
        &env_signature,
    ));

    let mut invalid_fixture = valid_entry.clone();
    invalid_fixture
        .fixture_sources
        .iter_mut()
        .for_each(|item| item.hash.push_str("-changed"));
    assert!(!cache_entry_valid(
        &invalid_fixture,
        &group,
        &cache_key,
        &toolchain,
        &env_signature,
    ));

    let mut invalid_env = valid_entry.clone();
    invalid_env.env_signature = "different-env".to_string();
    assert!(!cache_entry_valid(
        &invalid_env,
        &group,
        &cache_key,
        &toolchain,
        &env_signature,
    ));

    let invalid_path = env::temp_dir()
        .join("sifr-e2e-missing-artifact")
        .join("no-file");
    let mut invalid_artifact = valid_entry.clone();
    invalid_artifact.artifact_path = invalid_path.to_string_lossy().to_string();
    assert!(!cache_entry_valid(
        &invalid_artifact,
        &group,
        &cache_key,
        &toolchain,
        &env_signature,
    ));
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

    let pruned = prune_cache_manifest(&root, manifest, now_unix_secs);
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

    let pruned = prune_cache_manifest(&root, manifest, now_unix_secs);
    assert_eq!(pruned.entries.len(), 1);
    assert!(pruned.entries.contains_key("fresh-key"));
    assert!(cache_group_path(&root, shared_group).is_dir());

    let _ = std::fs::remove_dir_all(root);
}
