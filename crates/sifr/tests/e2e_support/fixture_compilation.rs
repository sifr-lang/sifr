use super::*;
const POSTCARD_DEP: &str =
    "postcard = { version = \"1.1.3\", default-features = false, features = [\"use-std\"] }";
const COOKIE_DEP: &str = "cookie = { version = \"0.18.1\", default-features = false }";
const HTTP_DEP: &str = "http = \"1.4.1\"";
const PERCENT_ENCODING_DEP: &str = "percent-encoding = \"2.3.2\"";
const SERDE_DEP: &str = "serde = { version = \"1.0.228\", features = [\"derive\"] }";
const SERDE_JSON_DEP: &str =
    "serde_json = { version = \"1.0.149\", features = [\"preserve_order\"] }";
const TRACING_DEP: &str =
    "tracing = { version = \"0.1.44\", default-features = false, features = [\"std\"] }";
const URL_DEP: &str = "url = \"2.5.8\"";
pub(crate) fn failure_matches_expectation(
    failure: &CompiledFailure,
    expectation: &CompileFailureExpectation,
) -> bool {
    failure.code == expectation.code
        && match expectation.column {
            Some(column) => failure.column == Some(column),
            None => true,
        }
}

pub(crate) fn match_compile_failure_expectations(
    expectations: &[CompileFailureExpectation],
    failures: &[CompiledFailure],
) -> Result<(), CompileFailureExpectation> {
    let mut consumed = vec![false; failures.len()];
    for expectation in expectations {
        let Some(index) = failures.iter().enumerate().find_map(|(index, failure)| {
            (!consumed[index] && failure_matches_expectation(failure, expectation)).then_some(index)
        }) else {
            return Err(expectation.clone());
        };
        consumed[index] = true;
    }
    Ok(())
}

pub(crate) fn compile_source_with_metadata(
    source: &str,
) -> Result<(String, HashSet<String>, HashSet<String>), Vec<String>> {
    match sifr_driver::compile_with_metadata(source) {
        sifr_driver::CompileResultFull::Success {
            rust_source,
            used_stdlib_modules,
            required_features,
            ..
        } => Ok((
            rust_source,
            used_stdlib_modules,
            required_features
                .into_iter()
                .map(|feature| feature.id().to_string())
                .collect(),
        )),
        sifr_driver::CompileResultFull::Errors { errors } => {
            Err(errors.iter().map(|error| error.message.clone()).collect())
        }
    }
}

pub(crate) fn compile_source_with_metadata_and_stats(
    source: &str,
) -> Result<
    (
        String,
        HashSet<String>,
        HashSet<String>,
        sifr_driver::LoweringStats,
    ),
    Vec<String>,
> {
    match sifr_driver::compile_with_metadata(source) {
        sifr_driver::CompileResultFull::Success {
            rust_source,
            used_stdlib_modules,
            required_features,
            lowering_stats,
        } => Ok((
            rust_source,
            used_stdlib_modules,
            required_features
                .into_iter()
                .map(|feature| feature.id().to_string())
                .collect(),
            lowering_stats,
        )),
        sifr_driver::CompileResultFull::Errors { errors } => {
            Err(errors.iter().map(|error| error.message.clone()).collect())
        }
    }
}

pub(crate) fn parse_fixture_selection_manifest(raw: &str) -> Result<BTreeSet<String>, String> {
    let manifest: FixtureSelectionManifest =
        serde_json::from_str(raw).map_err(|err| format!("manifest parse failed: {err}"))?;
    if manifest.fixture_names.is_empty() {
        return Err("fixture manifest must contain at least one fixture name".to_string());
    }

    let mut selected = BTreeSet::new();
    for fixture_name in manifest.fixture_names {
        let trimmed = fixture_name.trim();
        if trimmed.is_empty() {
            return Err("fixture manifest contains an empty fixture name".to_string());
        }
        selected.insert(trimmed.to_string());
    }
    Ok(selected)
}

pub(crate) fn load_selected_fixture_names_from_env() -> Result<Option<BTreeSet<String>>, String> {
    let Some(raw_path) = env::var(E2E_FIXTURE_MANIFEST_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };
    let manifest_path = PathBuf::from(&raw_path);
    let raw = std::fs::read_to_string(&manifest_path).map_err(|err| {
        format!(
            "unable to read fixture manifest {}: {err}",
            manifest_path.display()
        )
    })?;
    parse_fixture_selection_manifest(&raw).map(Some)
}

pub(crate) fn filter_fixtures_by_selection(
    entries: Vec<FixtureCase>,
    selected_fixture_names: &BTreeSet<String>,
) -> Result<Vec<FixtureCase>, String> {
    let available_names = entries
        .iter()
        .map(|fixture| fixture.name.clone())
        .collect::<BTreeSet<_>>();
    let missing = selected_fixture_names
        .difference(&available_names)
        .cloned()
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(format!(
            "fixture manifest referenced unknown pass fixtures: {}",
            missing.join(", ")
        ));
    }

    let filtered = entries
        .into_iter()
        .filter(|fixture| selected_fixture_names.contains(&fixture.name))
        .collect::<Vec<_>>();
    if filtered.is_empty() {
        return Err("fixture selection produced an empty pass corpus".to_string());
    }
    Ok(filtered)
}

pub(crate) fn discover_fixtures(base_dir: &Path) -> Vec<FixtureCase> {
    if !base_dir.exists() {
        return Vec::new();
    }

    let mut entries = Vec::new();
    for entry in std::fs::read_dir(base_dir)
        .into_iter()
        .flat_map(|read| read.filter_map(Result::ok))
    {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "sifr") {
            match std::fs::read_to_string(&path) {
                Ok(source) => {
                    let name = path_to_name(&path);
                    let source_hash = deterministic_hash(&source);
                    entries.push(FixtureCase {
                        name,
                        path,
                        source: source.clone(),
                        source_hash,
                        expected_stdout: extract_expect_stdout(&source),
                        _expected_stderr: extract_expect_stderr(&source),
                    });
                }
                Err(err) => {
                    eprintln!("[sifr-e2e] skipping fixture read error: {}", err);
                }
            }
        }
    }

    entries.sort_by(|left, right| left.path.cmp(&right.path));
    let selected_fixture_names = load_selected_fixture_names_from_env()
        .unwrap_or_else(|err| panic!("invalid {E2E_FIXTURE_MANIFEST_ENV}: {err}"));
    if let Some(selected_fixture_names) = selected_fixture_names {
        return filter_fixtures_by_selection(entries, &selected_fixture_names)
            .unwrap_or_else(|err| panic!("invalid {E2E_FIXTURE_MANIFEST_ENV}: {err}"));
    }
    entries
}

pub(crate) fn build_rust_source_from_module(raw: &str, entry_fn: &str) -> Result<String, String> {
    let marker = "fn main(";
    let index = raw
        .find(marker)
        .ok_or_else(|| "generated fixture Rust source does not define fn main".to_string())?;
    let after = &raw[index + marker.len()..];
    let prefix = &raw[..index];
    Ok(format!("{prefix}fn {entry_fn}({after}"))
}

pub(crate) fn compile_fixture(fixture: &FixtureCase) -> Result<CompiledCase, String> {
    let started = Instant::now();
    let (rust_source, stdlib_modules, required_crates) =
        compile_source_with_metadata(&fixture.source)
            .map_err(|errors| format!("sifr compilation failed:\n  {}", errors.join("\n  ")))?;

    let stdlib_modules = infer_dependencies(
        &rust_source,
        &normalize_dependency_set(stdlib_modules),
        &BTreeSet::new(),
    )
    .0;

    let required_crates = infer_dependencies(
        &rust_source,
        &BTreeSet::new(),
        &normalize_dependency_set(required_crates),
    )
    .1;

    if !rust_source.contains("fn main(") {
        return Err("generated Rust has no main function".to_string());
    }

    Ok(CompiledCase {
        fixture: fixture.clone(),
        rust_source,
        stdlib_modules,
        required_crates,
        _compile_duration_ms: started.elapsed().as_millis(),
    })
}

pub(crate) fn compile_suite_parallel(
    fixtures: &[FixtureCase],
    workers: usize,
) -> Vec<(FixtureCase, Result<CompiledCase, String>)> {
    run_in_parallel(fixtures, workers, |fixture| {
        let result = compile_fixture(fixture);
        (fixture.clone(), result)
    })
}

pub(crate) fn generate_cargo_toml(
    stdlib_modules: &BTreeSet<String>,
    required_crates: &BTreeSet<String>,
    package_name: &str,
) -> String {
    let mut contents = format!(
        "[package]\nname = \"{package_name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n"
    );
    let mut deps = BTreeSet::new();
    for module_name in stdlib_modules {
        match module_name.as_str() {
            "sifr.json" | "sifr.collections" | "_sifr.json" | "_sifr.collections" => {
                deps.insert(SERDE_JSON_DEP.to_string());
                deps.insert(SERDE_DEP.to_string());
            }
            "sifr.time" | "_sifr.time" | "sifr.datetime" | "_sifr.datetime" => {
                deps.insert("chrono = \"0.4.44\"".to_string());
            }
            "sifr.random" | "_sifr.crypto" => {
                deps.insert("rand = \"0.10.1\"".to_string());
                deps.insert("rand_distr = \"0.6.0\"".to_string());
            }
            "sifr.uuid" | "_sifr.uuid" => {
                deps.insert("rand = \"0.10.1\"".to_string());
                deps.insert(
                    "uuid = { version = \"1.23.1\", features = [\"v3\", \"v5\"] }".to_string(),
                );
            }
            "sifr.re" | "_sifr.regex" => {
                deps.insert("regex = \"1.12.3\"".to_string());
            }
            "sifr.hash" | "sifr.hashlib" => {
                deps.insert("sha2 = \"0.11.0\"".to_string());
                deps.insert("md5 = \"0.8.0\"".to_string());
                deps.insert("sha1 = \"0.11.0\"".to_string());
                deps.insert("blake2 = \"0.10.6\"".to_string());
            }
            "sifr.encoding" | "_sifr.encoding" => {
                deps.insert("encoding_rs = \"0.8.35\"".to_string());
            }
            "sifr.unicode" | "_sifr.unicode" => {
                deps.insert("unicode_names2 = \"3.1.0\"".to_string());
                deps.insert("unicode-normalization = \"0.1.25\"".to_string());
                deps.insert("unicode-segmentation = \"1.13.3\"".to_string());
            }
            "sifr.i18n" | "_sifr.i18n" => {
                deps.insert("icu_collator = \"2.2.0\"".to_string());
                deps.insert("icu_datetime = \"2.2.0\"".to_string());
                deps.insert("icu_decimal = \"2.2.0\"".to_string());
                deps.insert("icu_locale = \"2.2.0\"".to_string());
                deps.insert("icu_plurals = \"2.2.0\"".to_string());
            }
            "sifr.base64" => {
                deps.insert("base64 = \"0.22.1\"".to_string());
            }
            "sifr.parallel" => {
                deps.insert("rayon = \"1.12.0\"".to_string());
            }
            "sifr.runtime" | "_sifr.runtime" => {
                deps.insert("metrics = \"0.24.6\"".to_string());
                deps.insert(TRACING_DEP.to_string());
            }
            "sifr.ipc" | "_sifr.ipc" => {
                deps.insert(POSTCARD_DEP.to_string());
                deps.insert(SERDE_DEP.to_string());
            }
            "sifr.tomllib" | "_sifr.toml" => {
                deps.insert(
                    "toml = { version = \"1.1.2\", features = [\"preserve_order\"] }".to_string(),
                );
            }
            "sifr.url" | "_sifr.url" => {
                deps.insert(URL_DEP.to_string());
                deps.insert(PERCENT_ENCODING_DEP.to_string());
            }
            "sifr.http" | "_sifr.http" => {
                deps.insert(HTTP_DEP.to_string());
                deps.insert(COOKIE_DEP.to_string());
            }
            "sifr.gzip" | "sifr.zipfile" | "_sifr.compress" => {
                deps.insert("flate2 = \"1.1.9\"".to_string());
                deps.insert("zip = \"8.6.0\"".to_string());
            }
            "_bigint" => {
                deps.insert("num-bigint = \"0.4.6\"".to_string());
                deps.insert("num-traits = \"0.2.19\"".to_string());
            }
            _ => {}
        }
    }

    if stdlib_modules.iter().any(|module| {
        matches!(
            module.as_str(),
            "sifr.encoding"
                | "_sifr.encoding"
                | "sifr.unicode"
                | "_sifr.unicode"
                | "sifr.i18n"
                | "_sifr.i18n"
                | "sifr.net"
                | "_sifr.net"
                | "sifr.tls"
                | "_sifr.tls"
        )
    }) {
        deps.insert(sifr_runtime_dependency_spec_for_modules(stdlib_modules));
    }

    for crate_name in required_crates {
        match crate_name.as_str() {
            "serde_json" => {
                deps.insert(SERDE_JSON_DEP.to_string());
                deps.insert(SERDE_DEP.to_string());
            }
            "postcard" | "ipc" => {
                deps.insert(POSTCARD_DEP.to_string());
                deps.insert(SERDE_DEP.to_string());
            }
            "chrono" => {
                deps.insert("chrono = \"0.4.44\"".to_string());
            }
            "rand" => {
                deps.insert("rand = \"0.10.1\"".to_string());
            }
            "rand_distr" => {
                deps.insert("rand_distr = \"0.6.0\"".to_string());
            }
            "regex" => {
                deps.insert("regex = \"1.12.3\"".to_string());
            }
            "sha2" => {
                deps.insert("sha2 = \"0.11.0\"".to_string());
            }
            "md5" => {
                deps.insert("md5 = \"0.8.0\"".to_string());
            }
            "sha1" => {
                deps.insert("sha1 = \"0.11.0\"".to_string());
            }
            "uuid" => {
                deps.insert(
                    "uuid = { version = \"1.23.1\", features = [\"v3\", \"v5\"] }".to_string(),
                );
            }
            "blake2" => {
                deps.insert("blake2 = \"0.10.6\"".to_string());
            }
            "base64" => {
                deps.insert("base64 = \"0.22.1\"".to_string());
            }
            "toml" => {
                deps.insert(
                    "toml = { version = \"1.1.2\", features = [\"preserve_order\"] }".to_string(),
                );
            }
            "url" => {
                deps.insert(URL_DEP.to_string());
            }
            "percent-encoding" | "percent_encoding" => {
                deps.insert(PERCENT_ENCODING_DEP.to_string());
            }
            "http" => {
                deps.insert(HTTP_DEP.to_string());
            }
            "cookie" => {
                deps.insert(COOKIE_DEP.to_string());
            }
            "flate2" => {
                deps.insert("flate2 = \"1.1.9\"".to_string());
            }
            "zip" => {
                deps.insert("zip = \"8.6.0\"".to_string());
            }
            "num-bigint" => {
                deps.insert("num-bigint = \"0.4.6\"".to_string());
            }
            "num-traits" => {
                deps.insert("num-traits = \"0.2.19\"".to_string());
            }
            "rust_decimal" => {
                deps.insert(
                    "rust_decimal = { version = \"1.41.0\", features = [\"maths\", \"serde-with-str\"] }".to_string(),
                );
            }
            "bigdecimal" => {
                deps.insert(
                    "bigdecimal = { version = \"0.4.10\", features = [\"serde\"] }".to_string(),
                );
            }
            "rayon" => {
                deps.insert("rayon = \"1.12.0\"".to_string());
            }
            "sifr_runtime" | "sifr-runtime" => {
                if !deps
                    .iter()
                    .any(|dependency| dependency.starts_with("sifr_runtime = "))
                {
                    deps.insert(sifr_runtime_dependency_spec_with_features(&[]));
                }
            }
            "tokio" => {
                deps.insert(tokio_dependency_spec());
            }
            "tokio-rustls" | "tokio_rustls" => {
                deps.insert("tokio-rustls = \"0.26.4\"".to_string());
            }
            "rustls" => {
                deps.insert("rustls = \"=0.23.35\"".to_string());
            }
            "rustls-pemfile" | "rustls_pemfile" => {
                deps.insert("rustls-pemfile = \"2.2.0\"".to_string());
            }
            "rustls-platform-verifier" | "rustls_platform_verifier" => {
                deps.insert(
                    "rustls-platform-verifier = { version = \"0.7.0\", default-features = false }"
                        .to_string(),
                );
            }
            "metrics" => {
                deps.insert("metrics = \"0.24.6\"".to_string());
            }
            "tracing" => {
                deps.insert(TRACING_DEP.to_string());
            }
            _ => {}
        }
    }

    if !deps.is_empty() {
        contents.push_str("[dependencies]\n");
        for dep in deps {
            contents.push_str(&dep);
            contents.push('\n');
        }
    }

    // Keep generated grouped crates outside of the parent workspace to avoid Cargo
    // interpreting them as non-members of the existing workspace.
    contents.push_str("\n[workspace]\n");

    contents
}

pub(crate) fn build_group_sources(group_cases: Vec<CompiledCase>) -> Result<BatchGroup, String> {
    let mut cases = group_cases;
    cases.sort_by(|left, right| left.fixture.name.cmp(&right.fixture.name));

    let fingerprint = cases
        .first()
        .map(|case| case.dependency_fingerprint())
        .unwrap_or_else(|| DependencyFingerprint {
            stdlib_modules: BTreeSet::new(),
            required_crates: BTreeSet::new(),
        });

    let mut case_modules = Vec::with_capacity(cases.len());
    let mut generated_modules = String::new();
    let mut case_signature = String::new();

    for (ordinal, case) in cases.iter().enumerate() {
        let module_name = case.fixture.module_name(ordinal);
        let entry_fn = fixture_entry_name(&module_name);
        let wrapper_fn = format!("{entry_fn}_batch_run");
        let rust_source = build_rust_source_from_module(&case.rust_source, &entry_fn)
            .map_err(|err| format!("fixture {}: {}", case.fixture.name, err))?;

        let _ = writeln!(generated_modules, "pub mod {module_name} {{");
        generated_modules.push_str(&rust_source);
        generated_modules.push('\n');
        let _ = writeln!(generated_modules, "pub fn {wrapper_fn}() {{");
        let _ = writeln!(
            generated_modules,
            "    super::__SifrBatchTermination::__sifr_finish({entry_fn}());"
        );
        generated_modules.push_str("}\n");
        generated_modules.push_str("}\n\n");

        let _ = write!(
            case_signature,
            "{}:{}",
            case.fixture.name, case.fixture.source_hash
        );
        case_signature.push('|');

        case_modules.push((case.fixture.name.clone(), module_name, wrapper_fn));
    }

    let _union_stdlib = cases
        .iter()
        .flat_map(|case| case.stdlib_modules.iter().cloned())
        .collect::<BTreeSet<_>>();
    let _union_crates = cases
        .iter()
        .flat_map(|case| case.required_crates.iter().cloned())
        .collect::<BTreeSet<_>>();

    let mut generated_main = String::new();
    generated_main.push_str("trait __SifrBatchTermination {\n");
    generated_main.push_str("    fn __sifr_finish(self);\n");
    generated_main.push_str("}\n\n");
    generated_main.push_str("impl __SifrBatchTermination for () {\n");
    generated_main.push_str("    fn __sifr_finish(self) {}\n");
    generated_main.push_str("}\n\n");
    generated_main
        .push_str("impl<E: std::fmt::Debug> __SifrBatchTermination for Result<(), E> {\n");
    generated_main.push_str("    fn __sifr_finish(self) {\n");
    generated_main.push_str("        if let Err(error) = self {\n");
    generated_main.push_str("            eprintln!(\"{:?}\", error);\n");
    generated_main.push_str("            std::process::exit(1);\n");
    generated_main.push_str("        }\n");
    generated_main.push_str("    }\n");
    generated_main.push_str("}\n\n");
    generated_main.push_str("fn usage() -> ! {\n");
    generated_main.push_str("    eprintln!(\"usage: --case <fixture_name>\");\n");
    generated_main.push_str("    std::process::exit(2);\n");
    generated_main.push_str("}\n\n");
    generated_main.push_str("fn main() {\n");
    generated_main.push_str("    let mut args = std::env::args().skip(1);\n");
    generated_main.push_str("    let flag = args.next();\n");
    generated_main.push_str("    let case = args.next();\n");
    generated_main.push_str(
        "    if flag.as_deref() != Some(\"--case\") || case.is_none() || args.next().is_some() {\n",
    );
    generated_main.push_str("        usage();\n");
    generated_main.push_str("    }\n");
    generated_main.push_str("    match case.as_deref().expect(\"case\") {\n");
    for module in &case_modules {
        let _ = writeln!(
            generated_main,
            "        \"{}\" => __SifrBatchTermination::__sifr_finish({}::{}()),",
            module.0, module.1, module.2
        );
    }
    generated_main.push_str("        other => {\n");
    generated_main.push_str("            eprintln!(\"--case must be one of: \");\n");
    generated_main.push_str("            eprintln!(\"  {}\", other);\n");
    generated_main.push_str("            std::process::exit(2);\n");
    generated_main.push_str("        }\n");
    generated_main.push_str("    }\n");
    generated_main.push_str("}\n");

    let generated_rust = format!("{}{}", generated_modules, generated_main);
    let generated_rust_hash = deterministic_hash(&generated_rust);
    let raw_group_signature = format!(
        "{}|{}|{}",
        fingerprint.signature(),
        case_signature,
        generated_rust_hash,
    );
    let id = deterministic_hash(&raw_group_signature);
    let package_name = package_name_from_id(&id);

    Ok(BatchGroup {
        id,
        fingerprint,
        cases,
        generated_main: generated_rust,
        generated_rust_hash,
        package_name,
    })
}

pub(crate) fn plan_batches(
    compiled_cases: Vec<CompiledCase>,
) -> (Vec<BatchGroup>, Vec<FixtureExecution>) {
    let mut buckets: BTreeMap<DependencyFingerprint, Vec<CompiledCase>> = BTreeMap::new();
    for case in compiled_cases {
        let fp = case.dependency_fingerprint();
        buckets.entry(fp).or_default().push(case);
    }

    let mut groups = Vec::with_capacity(buckets.len());
    let mut planning_failures = Vec::new();
    let max_group_fixtures = std::env::var("SIFR_E2E_MAX_GROUP_FIXTURES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(usize::MAX);
    for (_, cases) in buckets {
        let mut cases = cases;
        cases.sort_by(|left, right| left.fixture.name.cmp(&right.fixture.name));
        for chunk in cases.chunks(max_group_fixtures) {
            let chunk_cases = chunk.to_vec();
            let fixture_names = chunk_cases
                .iter()
                .map(|case| case.fixture.name.clone())
                .collect::<Vec<_>>();
            match build_group_sources(chunk_cases) {
                Ok(group) => groups.push(group),
                Err(err) => {
                    for fixture_name in fixture_names {
                        planning_failures.push(FixtureExecution {
                            name: fixture_name.clone(),
                            status: Err(format!(
                                "FAIL [{}]: failed to generate grouped crate source: {}",
                                fixture_name, err
                            )),
                        });
                    }
                }
            }
        }
    }

    groups.sort_by(|left, right| left.id.cmp(&right.id));
    planning_failures.sort_by(|left, right| left.name.cmp(&right.name));
    (groups, planning_failures)
}

pub(crate) fn cache_env_signature() -> String {
    let mut values = Vec::with_capacity(E2E_CACHE_ENV_ALLOWLIST.len());
    for key in &E2E_CACHE_ENV_ALLOWLIST {
        values.push(format!(
            "{key}={}",
            env::var(key).unwrap_or_else(|_| "<unset>".to_string())
        ));
    }
    deterministic_hash(&values.join("\0"))
}

pub(crate) fn read_cache_manifest(root: &Path) -> CacheManifest {
    let manifest_path = cache_manifest_path(root);
    let raw = std::fs::read_to_string(&manifest_path).ok();
    match raw {
        Some(raw) => serde_json::from_str::<CacheManifest>(&raw).unwrap_or_else(|err| {
            eprintln!("[sifr-e2e-cache] manifest parse failed: {err}; rebuilding cache manifest");
            CacheManifest {
                schema_version: E2E_CACHE_SCHEMA_VERSION,
                entries: BTreeMap::new(),
            }
        }),
        None => CacheManifest {
            schema_version: E2E_CACHE_SCHEMA_VERSION,
            entries: BTreeMap::new(),
        },
    }
}

pub(crate) fn write_cache_manifest(root: &Path, manifest: &CacheManifest) {
    if let Err(err) = std::fs::create_dir_all(root) {
        eprintln!("[sifr-e2e-cache] cannot create cache dir: {err}");
        return;
    }
    let manifest_path = cache_manifest_path(root);
    let content = match serde_json::to_string_pretty(manifest) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("[sifr-e2e-cache] cannot serialize manifest: {err}");
            return;
        }
    };

    if let Err(err) = std::fs::write(manifest_path, content) {
        eprintln!("[sifr-e2e-cache] cannot persist manifest: {err}");
    }
}

pub(crate) fn prune_cache_manifest(
    root: &Path,
    manifest: CacheManifest,
    now_unix_secs: u64,
) -> CacheManifest {
    let groups_root = cache_groups_root(root);
    let cutoff_unix_secs = now_unix_secs.saturating_sub(E2E_CACHE_TTL_SECS);
    let mut next_manifest = CacheManifest {
        schema_version: manifest.schema_version,
        entries: manifest
            .entries
            .into_iter()
            .filter(|(_, entry)| entry.built_at_unix_secs >= cutoff_unix_secs)
            .collect(),
    };

    next_manifest
        .entries
        .retain(|_, entry| cache_group_path(root, &entry.group_id).is_dir());

    let live_group_ids = next_manifest
        .entries
        .values()
        .map(|entry| entry.group_id.as_str())
        .collect::<HashSet<_>>();

    match std::fs::read_dir(&groups_root) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                let Ok(file_type) = entry.file_type() else {
                    continue;
                };
                if !file_type.is_dir() {
                    continue;
                }

                let Some(group_id) = path.file_name().and_then(|name| name.to_str()) else {
                    continue;
                };

                if live_group_ids.contains(group_id) {
                    continue;
                }

                if let Err(err) = std::fs::remove_dir_all(&path) {
                    eprintln!(
                        "[sifr-e2e-cache] cannot remove stale group dir {}: {err}",
                        path.display()
                    );
                }
            }
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => {
            eprintln!(
                "[sifr-e2e-cache] cannot list cache groups root {}: {err}",
                groups_root.display()
            );
        }
    }

    next_manifest
}

pub(crate) fn toolchain_info() -> ToolchainInfo {
    let rustc_version = command_with_capture("rustc", &["-V"], None).stdout;
    let rustc_vv = command_with_capture("rustc", &["-Vv"], None).stdout;
    let cargo_v = command_with_capture("cargo", &["-V"], None).stdout;

    let mut target = "unknown-target".to_string();
    for line in rustc_vv.lines() {
        if let Some((label, value)) = line.split_once(':') {
            if label == "host" {
                target = value.trim().to_string();
            }
        }
    }

    ToolchainInfo {
        rustc_v: rustc_version.trim().to_string(),
        rustc_vv: rustc_vv.trim().to_string(),
        cargo_v: cargo_v.trim().to_string(),
        target,
        os: env::consts::OS.to_string(),
        arch: env::consts::ARCH.to_string(),
    }
}

pub(crate) fn cache_key_for_group(
    group: &BatchGroup,
    toolchain: &ToolchainInfo,
    env_signature: &str,
) -> String {
    let fixture_signature = group
        .cases
        .iter()
        .map(|case| format!("{}:{}", case.fixture.name, case.fixture.source_hash))
        .collect::<Vec<_>>()
        .join("|");
    deterministic_hash(&format!(
        "schema={}\nfp={}\ngen={}\nsig={}\nfixture={}\n{}",
        E2E_CACHE_SCHEMA_VERSION,
        group.fingerprint.signature(),
        group.generated_rust_hash,
        toolchain.signature(),
        fixture_signature,
        env_signature,
    ))
}

pub(crate) fn cache_entry_valid(
    entry: &CacheEntry,
    group: &BatchGroup,
    cache_key: &str,
    toolchain: &ToolchainInfo,
    env_signature: &str,
) -> bool {
    if entry.schema_version != E2E_CACHE_SCHEMA_VERSION {
        return false;
    }
    if entry.cache_key != cache_key || entry.group_id != group.id {
        return false;
    }
    if entry.group_fingerprint != group.fingerprint.signature()
        || entry.group_rust_hash != group.generated_rust_hash
    {
        return false;
    }
    if entry.compiler_signature != toolchain.signature()
        || entry.rustc_v != toolchain.rustc_v
        || entry.rustc_vv != toolchain.rustc_vv
        || entry.cargo_v != toolchain.cargo_v
        || entry.target != toolchain.target
        || entry.os != toolchain.os
        || entry.arch != toolchain.arch
        || entry.env_signature != env_signature
    {
        return false;
    }

    if group.cases.len() != entry.fixture_sources.len() {
        return false;
    }

    for (case, saved) in group.cases.iter().zip(entry.fixture_sources.iter()) {
        if case.fixture.name != saved.fixture || case.fixture.source_hash != saved.hash {
            return false;
        }
    }

    Path::new(&entry.artifact_path).exists()
}
