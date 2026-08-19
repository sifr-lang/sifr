use super::*;
use sifr_stdlib_manifest::{feature_for_codegen_requirement, features_for_stdlib_module};

const FORMER_MODULE_RULES: &[(&str, &[StdlibFeature])] = &[
    (
        "_bigint",
        &[StdlibFeature::NumBigint, StdlibFeature::NumTraits],
    ),
    ("_sifr.fs", &[StdlibFeature::Fs]),
    (
        "_sifr.net",
        &[
            StdlibFeature::SifrRuntime,
            StdlibFeature::Tokio,
            StdlibFeature::Tracing,
        ],
    ),
    (
        "_sifr.tls",
        &[
            StdlibFeature::SifrRuntime,
            StdlibFeature::Tokio,
            StdlibFeature::TokioRustls,
            StdlibFeature::Rustls,
            StdlibFeature::RustlsPemfile,
            StdlibFeature::RustlsPlatformVerifier,
            StdlibFeature::Tracing,
        ],
    ),
    ("_sifr.http", &[StdlibFeature::Http]),
    ("_sifr.signal", &[]),
];

const FORMER_DIRECT_RULES: &[(&str, StdlibFeature)] = &[
    ("regex", StdlibFeature::Regex),
    ("rand", StdlibFeature::Rand),
    ("rand_distr", StdlibFeature::RandDistr),
    ("chrono", StdlibFeature::Chrono),
    ("md5", StdlibFeature::Md5),
    ("uuid", StdlibFeature::Uuid),
    ("toml", StdlibFeature::Toml),
    ("flate2", StdlibFeature::Flate2),
    ("zip", StdlibFeature::Zip),
    ("base64", StdlibFeature::Base64),
    ("sha1", StdlibFeature::Sha1),
    ("sha2", StdlibFeature::Sha2),
    ("blake2", StdlibFeature::Blake2),
    ("rust_decimal", StdlibFeature::RustDecimal),
    ("bigdecimal", StdlibFeature::BigDecimal),
    ("tracing", StdlibFeature::Tracing),
    ("metrics", StdlibFeature::Metrics),
    ("postcard", StdlibFeature::Ipc),
    ("url", StdlibFeature::Url),
    ("percent-encoding", StdlibFeature::PercentEncoding),
    ("http", StdlibFeature::Http),
    ("bytes", StdlibFeature::Bytes),
    ("h2", StdlibFeature::H2),
    ("http-body", StdlibFeature::HttpBody),
    ("http-body-util", StdlibFeature::HttpBodyUtil),
    ("hyper", StdlibFeature::Hyper),
    ("hyper-util", StdlibFeature::HyperUtil),
    ("tower-service", StdlibFeature::TowerService),
    ("cookie", StdlibFeature::Cookie),
];

fn case_with_plan(
    name: &str,
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
        fixture: FixtureCase {
            name: name.to_string(),
            path: PathBuf::from(format!("{name}.sifr")),
            source: String::new(),
            source_hash: deterministic_hash(name),
            _expected_stderr: Vec::new(),
        },
        rust_source: rust_source.to_string(),
        used_stdlib_modules,
        required_features,
        dependency_plan,
        _compile_duration_ms: 0,
    }
}

#[test]
fn former_module_inference_inventory_is_owned_by_typed_production_metadata() {
    for (module, expected_features) in FORMER_MODULE_RULES {
        assert_eq!(
            features_for_stdlib_module(module),
            *expected_features,
            "{module}"
        );
        let (modules, features) = if *module == "_bigint" {
            (
                HashSet::new(),
                expected_features.iter().copied().collect::<HashSet<_>>(),
            )
        } else {
            (HashSet::from([(*module).to_string()]), HashSet::new())
        };
        let plan = sifr_driver::try_generate_standalone_dependency_plan(
            &modules,
            &features,
            &sifr_driver::InteropBuildPlan::default(),
        )
        .expect("module dependency plan should resolve");
        if *module == "_bigint" {
            assert_eq!(plan.required_features.len(), expected_features.len());
            for feature in *expected_features {
                assert!(plan.required_features.contains(feature), "{feature:?}");
            }
        } else {
            assert!(plan.stdlib_modules.contains(*module), "{module}");
        }
        assert!(
            !plan.cargo_dependency_lines().is_empty(),
            "former module rule {module} must resolve to production Cargo inputs"
        );
    }
}

#[test]
fn former_direct_inference_inventory_has_typed_requirement_identity() {
    for (requirement, expected_feature) in FORMER_DIRECT_RULES {
        assert_eq!(
            feature_for_codegen_requirement(requirement),
            Some(*expected_feature),
            "{requirement}"
        );
        let features = HashSet::from([*expected_feature]);
        let plan = sifr_driver::try_generate_standalone_dependency_plan(
            &HashSet::new(),
            &features,
            &sifr_driver::InteropBuildPlan::default(),
        )
        .expect("feature dependency plan should resolve");
        assert!(
            plan.required_features.contains(expected_feature),
            "{requirement}"
        );
    }
}

#[test]
fn runtime_crate_inference_rule_is_a_typed_production_feature() {
    assert_eq!(
        feature_for_codegen_requirement("sifr_runtime"),
        Some(StdlibFeature::SifrRuntime)
    );
    let plan = sifr_driver::try_generate_standalone_dependency_plan(
        &HashSet::new(),
        &HashSet::from([StdlibFeature::SifrRuntime]),
        &sifr_driver::InteropBuildPlan::default(),
    )
    .expect("runtime dependency plan should resolve");
    assert!(plan
        .cargo_dependency_lines()
        .iter()
        .any(|line| line.starts_with("sifr_runtime = ")));
}

#[test]
fn compiled_fixture_plan_matches_production_build_report() {
    let path = Path::new("tests/e2e/pass/runtime_diagnostics_tracing.sifr");
    let source = std::fs::read_to_string(path).expect("runtime fixture should exist");
    let fixture = FixtureCase {
        name: "runtime_diagnostics_tracing".to_string(),
        path: path.to_path_buf(),
        source: source.clone(),
        source_hash: deterministic_hash(&source),
        _expected_stderr: extract_expect_stderr(&source),
    };
    let compiled = compile_fixture(&fixture).expect("fixture should compile");
    let output_dir = env::temp_dir().join("sifr_e2e_production_plan_parity");
    let report = sifr_driver::build_single_file_report(&source, path, &output_dir).unwrap_or_else(
        |errors| {
            panic!(
                "production build should succeed: {}",
                errors
                    .iter()
                    .map(|error| error.message.as_str())
                    .collect::<Vec<_>>()
                    .join("; ")
            )
        },
    );

    assert_eq!(
        compiled.dependency_plan.dependency_input_fingerprint(),
        report.sysroot().dependency_inputs()
    );
    assert_eq!(
        compiled.dependency_plan.cache_fingerprint,
        report.sysroot().dependency_fingerprint()
    );
    assert_eq!(
        compiled.dependency_plan.sysroot_root,
        report.sysroot().root()
    );
    assert_eq!(
        compiled.dependency_plan.toolchain_id,
        report.sysroot().toolchain_id()
    );
    assert_eq!(
        compiled.dependency_plan.sysroot_content_sha256,
        report.sysroot().content_sha256()
    );
}

#[test]
fn missing_metadata_is_not_repaired_from_generated_rust() {
    let rust_source = "use num_bigint::BigInt;\nfn main() { let _ = BigInt::from(1); }\n";
    let error = build_and_run_capture_with_deps(
        rust_source,
        "missing_dependency_metadata",
        &HashSet::new(),
        &HashSet::new(),
        &sifr_driver::InteropBuildPlan::default(),
    )
    .expect_err("missing compiler metadata must remain a Rust build failure");

    assert!(error.contains("Rust compilation failed"));
    assert!(error.contains("num_bigint"));
}

#[test]
fn fixtures_with_different_production_plans_cannot_repair_each_other() {
    let missing = case_with_plan(
        "missing_metadata",
        "use num_bigint::BigInt;\nfn main() { let _ = BigInt::from(1); }",
        HashSet::new(),
    );
    let declared = case_with_plan(
        "declared_metadata",
        "fn main() {}",
        HashSet::from([StdlibFeature::NumBigint]),
    );

    let error = build_group_sources(vec![missing.clone(), declared.clone()])
        .expect_err("non-identical plans must not be unioned");
    assert!(error.contains("non-identical production dependency plans"));

    let (groups, failures) = plan_batches(vec![missing, declared]);
    assert!(failures.is_empty());
    assert_eq!(groups.len(), 2);
    assert!(groups.iter().all(|group| group.cases.len() == 1));
}

#[test]
fn cache_identity_changes_with_the_production_dependency_plan() {
    let empty = case_with_plan("same_fixture", "fn main() {}", HashSet::new());
    let bigint = case_with_plan(
        "same_fixture",
        "fn main() {}",
        HashSet::from([StdlibFeature::NumBigint]),
    );

    assert_ne!(
        empty.dependency_fingerprint(),
        bigint.dependency_fingerprint()
    );
    let empty_group = build_group_sources(vec![empty]).expect("empty-plan group should build");
    let bigint_group = build_group_sources(vec![bigint]).expect("bigint-plan group should build");
    let toolchain = toolchain_info();
    let environment = cache_env_signature();
    assert_ne!(
        cache_key_for_group(&empty_group, &toolchain, &environment),
        cache_key_for_group(&bigint_group, &toolchain, &environment)
    );
}
