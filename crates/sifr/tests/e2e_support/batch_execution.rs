#![expect(
    clippy::print_stderr,
    reason = "the E2E harness reports cache and batch execution status"
)]

use super::*;
pub(crate) fn build_group_binary_path(group_root: &Path, package_name: &str) -> PathBuf {
    let debug_dir = group_root.join("target").join("debug");
    if cfg!(target_os = "windows") {
        debug_dir.join(format!("{package_name}.exe"))
    } else {
        debug_dir.join(package_name)
    }
}

pub(crate) fn build_batch_group(
    group: BatchGroup,
    config: &RunnerConfig,
    toolchain: &ToolchainInfo,
    env_signature: &str,
    manifest: &Arc<Mutex<CacheManifest>>,
) -> GroupBuildOutcome {
    let started = Instant::now();
    let group_root = config.cache.root.join("groups").join(&group.id);
    let mut build_error = None;
    let mut build_log = None;
    let mut artifact = None;
    let mut cache_hit = false;

    if let Err(err) = std::fs::create_dir_all(group_root.join("src")) {
        build_error = Some(format!("failed to create batch crate dir: {err}"));
    }

    let cache_key = cache_key_for_group(&group, toolchain, env_signature);
    let cached_entry = if config.cache.enabled {
        manifest
            .lock()
            .ok()
            .and_then(|stored| stored.entries.get(&cache_key).cloned())
            .filter(|entry| cache_entry_valid(entry, &group, &cache_key, toolchain, env_signature))
    } else {
        None
    };

    if let Some(entry) = cached_entry {
        artifact = Some(PathBuf::from(entry.artifact_path));
        build_log = entry.build_log_path.map(PathBuf::from);
        cache_hit = true;
        return GroupBuildOutcome {
            group,
            artifact_path: artifact,
            build_log_path: build_log,
            build_error: None,
            build_ms: started.elapsed().as_millis(),
            cache_hit,
        };
    }

    if build_error.is_none() {
        let cargo_toml = sifr_driver::generate_dependency_cargo_toml(
            &group.package_name,
            &group.dependency_plan,
        );
        let source_path = group_root.join("src").join("main.rs");
        let cargo_toml_path = group_root.join("Cargo.toml");

        if let Err(err) = std::fs::write(&cargo_toml_path, cargo_toml) {
            build_error = Some(format!("failed to write Cargo.toml: {err}"));
        } else if let Err(err) = std::fs::write(&source_path, &group.generated_main) {
            build_error = Some(format!("failed to write main.rs: {err}"));
        } else {
            let mut build_command = Command::new("cargo");
            build_command
                .args(sifr_driver::sysroot_cargo_config_args(
                    &group.dependency_plan,
                ))
                .args(["build", "--quiet", "-j"])
                .arg(config.cargo_build_jobs.to_string())
                .current_dir(&group_root);
            // Batch crates are cached by their own `target/` artifact paths.
            // An inherited outer CARGO_TARGET_DIR moves binaries away from the
            // recorded cache location and makes the run phase miss them.
            build_command.env_remove("CARGO_TARGET_DIR");
            let build_capture = run_capture(build_command);
            if build_capture.status_ok {
                artifact = Some(build_group_binary_path(&group_root, &group.package_name));
                if let Some(path) = &artifact {
                    if config.cache.enabled {
                        let built_at_unix_secs = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map(|dur| dur.as_secs())
                            .unwrap_or(0);
                        let entry = CacheEntry {
                            schema_version: E2E_CACHE_SCHEMA_VERSION,
                            cache_key: cache_key.clone(),
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
                            artifact_path: path.to_string_lossy().to_string(),
                            build_log_path: build_log
                                .as_ref()
                                .map(|path| path.to_string_lossy().to_string()),
                            built_at_unix_secs,
                        };
                        if let Ok(mut manifest_lock) = manifest.lock() {
                            manifest_lock.entries.insert(cache_key.clone(), entry);
                        }
                    }
                }
            } else {
                let log_path = group_root.join("build.log");
                let mut diagnostic = String::new();
                let _ = std::fmt::Write::write_str(
                    &mut diagnostic,
                    &format!(
                        "Rust build failed for {} ({})\n\nSTDOUT:\n{}\n\nSTDERR:\n{}\n",
                        group.id,
                        group.fingerprint.hash(),
                        build_capture.stdout,
                        build_capture.stderr
                    ),
                );
                let _ = std::fmt::Write::write_str(&mut diagnostic, "Generated Rust:\n");
                let _ = std::fmt::Write::write_str(&mut diagnostic, &group.generated_main);
                if let Err(err) = std::fs::write(&log_path, diagnostic) {
                    eprintln!("[sifr-e2e-cache] failed to write build log: {err}");
                }
                build_error = Some(format!(
                    "Rust compilation failed. Check build log: {}",
                    log_path.display()
                ));
                build_log = Some(log_path);
            }
        }
    }

    GroupBuildOutcome {
        group,
        artifact_path: artifact,
        build_log_path: build_log,
        build_error,
        build_ms: started.elapsed().as_millis(),
        cache_hit,
    }
}

pub(crate) fn build_batch_suite(
    groups: &[BatchGroup],
    config: &RunnerConfig,
    toolchain: &ToolchainInfo,
    env_signature: &str,
    manifest: &CacheManifest,
) -> (Vec<GroupBuildOutcome>, CacheManifest) {
    let cache_root = manifest.entries.len();
    let _ = cache_root;
    let shared_manifest = Arc::new(Mutex::new(manifest.clone()));
    let outcomes = run_in_parallel(groups, config.rust_jobs, |group| {
        build_batch_group(
            group.clone(),
            config,
            toolchain,
            env_signature,
            &shared_manifest,
        )
    });

    let next_manifest = shared_manifest
        .lock()
        .map_or_else(|_| manifest.clone(), |lock| lock.clone());
    if config.cache.enabled {
        write_cache_manifest(&config.cache.root, &next_manifest);
    }

    (outcomes, next_manifest)
}

pub(crate) fn run_single_case(artifact_path: &Path, fixture_name: &str) -> Result<(), String> {
    let args = ["--case", fixture_name];
    let run_capture = command_with_capture(
        artifact_path.to_str().unwrap_or("sifr_batch_binary"),
        &args,
        None,
    );
    if !run_capture.status_ok {
        return Err(format!("binary exited with error:\n{}", run_capture.stderr));
    }

    Ok(())
}

pub(crate) fn run_batch_outcomes(group_outcome: &GroupBuildOutcome) -> Vec<FixtureExecution> {
    let group = &group_outcome.group;
    let fixture_names = group
        .cases
        .iter()
        .map(|case| case.fixture.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    if let Some(error) = &group_outcome.build_error {
        return group
            .cases
            .iter()
            .map(|case| FixtureExecution {
                name: case.fixture.name.clone(),
                status: Err(format!(
                    "FAIL [{}]: {}\n  group: {}\n  group fixture list: [{}]\n  group fingerprint: {}\n  crate: {}\n  build log: {}",
                    case.fixture.name,
                    error,
                    group.id,
                    fixture_names,
                    group.fingerprint.hash(),
                    config_cache_root().join("groups").join(&group.id).display(),
                    group_outcome
                        .build_log_path
                        .as_ref()
                        .map(|path| path.display().to_string())
                        .unwrap_or_else(|| "<none>".to_string())
                )),
            })
            .collect();
    }

    let Some(artifact) = &group_outcome.artifact_path else {
        let message = group_outcome
            .build_error
            .as_deref()
            .unwrap_or("missing batch artifact");
        return group
            .cases
            .iter()
            .map(|case| FixtureExecution {
                name: case.fixture.name.clone(),
                status: Err(format!(
                    "FAIL [{}]: {}\n  group: {}\n  group fingerprint: {}\n  crate: {}",
                    case.fixture.name,
                    message,
                    group.id,
                    group.fingerprint.hash(),
                    config_cache_root().join("groups").join(&group.id).display(),
                )),
            })
            .collect();
    };

    group
        .cases
        .iter()
        .map(|case| {
            let status =
                match run_single_case(artifact, &case.fixture.name) {
                    Ok(()) => Ok(()),
                    Err(err) => Err(format!(
                        "FAIL [{}]: {}\n  group: {}\n  group fingerprint: {}\n  crate: {}\n  artifact: {}",
                        case.fixture.name,
                        err,
                        group.id,
                        group.fingerprint.hash(),
                        config_cache_root().join("groups").join(&group.id).display(),
                        artifact.display(),
                    )),
                };
            FixtureExecution {
                name: case.fixture.name.clone(),
                status,
            }
        })
        .collect()
}

pub(crate) fn run_batch_suite(
    build_outcomes: &[GroupBuildOutcome],
    config: &RunnerConfig,
) -> (Vec<FixtureExecution>, Vec<GroupRunOutcome>) {
    let mut outputs = Vec::new();
    let mut run_outcomes = Vec::with_capacity(build_outcomes.len());

    let per_group = run_in_parallel(build_outcomes, config.run_jobs, |group| {
        let started = Instant::now();
        let results = run_batch_outcomes(group);
        (
            GroupRunOutcome {
                group_id: group.group.id.clone(),
                fixture_count: group.group.cases.len(),
                cache_hit: group.cache_hit,
                elapsed_ms: started.elapsed().as_millis(),
            },
            results,
        )
    });

    for (outcome, results) in per_group {
        outputs.extend(results);
        run_outcomes.push(outcome);
    }

    outputs.sort_by(|left, right| left.name.cmp(&right.name));
    (outputs, run_outcomes)
}

pub(crate) fn build_and_run_capture_with_deps(
    rust_source: &str,
    test_name: &str,
    stdlib_modules: &HashSet<String>,
    required_features: &HashSet<StdlibFeature>,
    interop: &sifr_driver::InteropBuildPlan,
) -> Result<(String, String, bool), String> {
    let dependency_plan = sifr_driver::try_generate_standalone_dependency_plan(
        stdlib_modules,
        required_features,
        interop,
    )
    .map_err(|error| {
        format!(
            "failed to resolve production dependency plan: {}",
            error.boundary_message()
        )
    })?;
    let project_identity = deterministic_hash(&format!(
        "{}\n{}",
        dependency_plan.dependency_input_fingerprint(),
        dependency_plan.cache_fingerprint
    ));
    let tmp_dir = env::temp_dir()
        .join("sifr_e2e_tests")
        .join(test_name)
        .join(project_identity);
    let src_dir = tmp_dir.join("src");
    std::fs::create_dir_all(&src_dir).map_err(|err| format!("failed to create dir: {err}"))?;

    let cargo_toml = sifr_driver::generate_dependency_cargo_toml("sifr_output", &dependency_plan);
    std::fs::write(tmp_dir.join("Cargo.toml"), cargo_toml)
        .map_err(|err| format!("failed to write Cargo.toml: {err}"))?;
    std::fs::write(src_dir.join("main.rs"), rust_source)
        .map_err(|err| format!("failed to write main.rs: {err}"))?;

    let mut build_command = Command::new("cargo");
    build_command
        .args(sifr_driver::sysroot_cargo_config_args(&dependency_plan))
        .args(["build", "--quiet"])
        .current_dir(&tmp_dir);
    build_command.env_remove("CARGO_TARGET_DIR");
    let build_capture = run_capture(build_command);
    if !build_capture.status_ok {
        return Err(format!(
            "Rust compilation failed.\n\nGenerated Rust:\n{}\n\nrustc errors:\n{}",
            rust_source, build_capture.stderr
        ));
    }

    let binary_name = if cfg!(target_os = "windows") {
        "sifr_output.exe"
    } else {
        "sifr_output"
    };
    let binary_path = tmp_dir.join("target").join("debug").join(binary_name);
    let run_capture = command_with_capture(
        binary_path.to_str().unwrap_or("sifr_output"),
        &[],
        Some(&tmp_dir),
    );
    Ok((
        run_capture.stdout,
        run_capture.stderr,
        run_capture.status_ok,
    ))
}

pub(crate) fn build_and_run_with_deps(
    rust_source: &str,
    test_name: &str,
    stdlib_modules: &HashSet<String>,
    required_features: &HashSet<StdlibFeature>,
    interop: &sifr_driver::InteropBuildPlan,
) -> Result<String, String> {
    match build_and_run_capture_with_deps(
        rust_source,
        test_name,
        stdlib_modules,
        required_features,
        interop,
    ) {
        Ok((stdout, stderr, status_ok)) => {
            if status_ok {
                Ok(stdout)
            } else {
                Err(format!("binary exited with error:\n{stderr}"))
            }
        }
        Err(err) => Err(err),
    }
}

pub(crate) fn run_in_parallel<T, R, F>(items: &[T], workers: usize, worker: F) -> Vec<R>
where
    T: Sync,
    R: Send,
    F: Fn(&T) -> R + Send + Sync,
{
    if items.is_empty() {
        return Vec::new();
    }
    let workers = workers.max(1).min(items.len());
    let results: Arc<Mutex<Vec<Option<R>>>> = Arc::new(Mutex::new(
        (0..items.len()).map(|_| None).collect::<Vec<_>>(),
    ));
    let index = Arc::new(Mutex::new(0usize));
    let worker = Arc::new(worker);

    thread::scope(|scope| {
        let mut handles = Vec::with_capacity(workers);
        for _ in 0..workers {
            let worker = Arc::clone(&worker);
            let index = Arc::clone(&index);
            let results = Arc::clone(&results);

            let handle = scope.spawn(move || {
                loop {
                    let item_index = {
                        let mut cursor = index
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        let next = *cursor;
                        *cursor += 1;
                        next
                    };

                    if item_index >= items.len() {
                        break;
                    }

                    let result = worker(&items[item_index]);
                    let locked_results = results.lock();
                    if let Ok(mut output) = locked_results {
                        output[item_index] = Some(result);
                    }
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.join();
        }
    });

    let mut ordered = Vec::with_capacity(items.len());
    let mut output = results
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    for slot in output.iter_mut() {
        if let Some(value) = slot.take() {
            ordered.push(value);
        }
    }
    ordered
}

pub(crate) fn run_pass_suite(config: &RunnerConfig) -> PassReport {
    let fixtures = discover_fixtures(Path::new("tests/e2e/pass"));
    assert!(!fixtures.is_empty(), "No pass tests found");

    let compile_started = Instant::now();
    let compiled_results = compile_suite_parallel(&fixtures, config.sifr_jobs);
    let compile_ms = compile_started.elapsed().as_millis();

    let mut compiled_failures = Vec::new();
    let mut compiled_cases = Vec::new();
    for (fixture, result) in compiled_results {
        match result {
            Ok(compiled) => compiled_cases.push(compiled),
            Err(message) => compiled_failures.push(FixtureExecution {
                name: fixture.name.clone(),
                status: Err(format!("FAIL [{}]: {}", fixture.name, message)),
            }),
        }
    }

    let plan_started = Instant::now();
    let (groups, planning_failures) = plan_batches(compiled_cases);
    let plan_ms = plan_started.elapsed().as_millis();

    let toolchain = toolchain_info();
    let env_signature = cache_env_signature();
    let initial_manifest = if config.cache.enabled {
        if let Err(err) = std::fs::create_dir_all(&config.cache.root) {
            eprintln!("[sifr-e2e-cache] cannot create cache root: {err}");
        }
        let manifest = read_cache_manifest(&config.cache.root);
        let pruned_manifest = prune_cache_manifest(
            &config.cache.root,
            manifest,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_secs())
                .unwrap_or_default(),
        );
        write_cache_manifest(&config.cache.root, &pruned_manifest);
        pruned_manifest
    } else {
        CacheManifest {
            schema_version: E2E_CACHE_SCHEMA_VERSION,
            entries: BTreeMap::new(),
        }
    };

    let build_started = Instant::now();
    let (build_outcomes, _updated_manifest) = build_batch_suite(
        &groups,
        config,
        &toolchain,
        &env_signature,
        &initial_manifest,
    );
    let build_ms = build_started.elapsed().as_millis();
    let observed_build_ms: u128 = build_outcomes.iter().map(|outcome| outcome.build_ms).sum();
    let cache_hits = build_outcomes
        .iter()
        .filter(|outcome| outcome.cache_hit)
        .count();
    let mut group_sizes = build_outcomes
        .iter()
        .map(|outcome| outcome.group.cases.len())
        .collect::<Vec<_>>();
    group_sizes.sort_unstable();
    let group_count = group_sizes.len();
    let largest_group_fixtures = group_sizes.iter().copied().max().unwrap_or(0);
    let median_group_fixtures = if group_sizes.is_empty() {
        0
    } else {
        group_sizes[group_sizes.len() / 2]
    };

    let mut all_cases = Vec::new();
    let run_started = Instant::now();
    let (run_cases, run_outcomes) = run_batch_suite(&build_outcomes, config);
    all_cases.extend(run_cases);
    all_cases.extend(compiled_failures);
    all_cases.extend(planning_failures);
    let run_ms = run_started.elapsed().as_millis();

    let mut build_timing = build_outcomes
        .iter()
        .map(|outcome| {
            (
                outcome.group.id.clone(),
                outcome.build_ms,
                outcome.group.cases.len(),
                outcome.cache_hit,
            )
        })
        .collect::<Vec<_>>();
    build_timing.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    let top_build = build_timing.iter().take(3).cloned().collect::<Vec<_>>();

    let mut run_timing = run_outcomes
        .into_iter()
        .map(|outcome| {
            (
                outcome.group_id,
                outcome.elapsed_ms,
                outcome.fixture_count,
                outcome.cache_hit,
            )
        })
        .collect::<Vec<_>>();
    run_timing.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    let top_run = run_timing.iter().take(3).cloned().collect::<Vec<_>>();

    let summarize_groups = |groups: &[(String, u128, usize, bool)]| {
        if groups.is_empty() {
            return String::new();
        }

        groups
            .iter()
            .map(|(id, ms, count, cache_hit)| {
                format!("  - {id} ({count} fixtures, {ms}ms, cache_hit={cache_hit})")
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    eprintln!(
        "[sifr-e2e] timing: compile={compile_ms}ms plan={plan_ms}ms build={build_ms}ms build-sum={observed_build_ms}ms run={run_ms}ms cache_hits={cache_hits}/{group_count}"
    );
    eprintln!(
        "[sifr-e2e] group_stats: groups={group_count} largest_group_fixtures={largest_group_fixtures} median_group_fixtures={median_group_fixtures}"
    );
    eprintln!(
        "[sifr-e2e] slowest build groups:\n{}",
        summarize_groups(&top_build)
    );
    eprintln!(
        "[sifr-e2e] slowest run groups:\n{}",
        summarize_groups(&top_run)
    );

    all_cases.sort_by(|left, right| left.name.cmp(&right.name));
    PassReport { cases: all_cases }
}

pub(crate) fn config_cache_root() -> PathBuf {
    Path::new(E2E_CACHE_DIR).to_path_buf()
}

pub(crate) fn failure_group(reason: &str) -> &'static str {
    if reason.contains("sifr compilation failed") {
        "compile"
    } else if reason.contains("failed to generate grouped crate source") {
        "planning"
    } else if reason.contains("Rust compilation failed")
        || reason.contains("build log:")
        || reason.contains("missing batch artifact")
    {
        "build"
    } else if reason.contains("stdout mismatch") || reason.contains("binary exited with error") {
        "run"
    } else {
        "other"
    }
}

pub(crate) fn indent_multiline(text: &str, indent: &str) -> String {
    text.lines()
        .map(|line| format!("{indent}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn format_failures(kind: &str, cases: &[FixtureExecution]) -> String {
    let mut failures = cases
        .iter()
        .filter_map(|case| {
            case.status.as_ref().err().map(|reason| {
                (
                    case.name.clone(),
                    failure_group(reason).to_string(),
                    reason.clone(),
                )
            })
        })
        .collect::<Vec<_>>();

    if failures.is_empty() {
        return String::new();
    }

    failures.sort_by(|left, right| {
        left.1
            .cmp(&right.1)
            .then_with(|| left.0.cmp(&right.0))
            .then_with(|| left.2.cmp(&right.2))
    });

    let mut grouped: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for (name, group, reason) in failures {
        grouped.entry(group).or_default().push((name, reason));
    }

    let passed = cases.iter().filter(|case| case.status.is_ok()).count();
    let failed = cases.len().saturating_sub(passed);
    let mut sections = Vec::new();
    for (group, entries) in grouped {
        let mut rows = Vec::new();
        for (name, reason) in entries {
            rows.push(format!("- [{}]\n{}", name, indent_multiline(&reason, "  ")));
        }
        sections.push(format!(
            "[{group}] {} failure(s)\n{}",
            rows.len(),
            rows.join("\n")
        ));
    }

    format!(
        "{kind} E2E pass failures ({passed} passed, {failed} failed)\n\n{}",
        sections.join("\n\n")
    )
}

pub(crate) fn report_signature(kind: &str, report: &PassReport) -> String {
    let summary = format_failures(kind, &report.cases);
    deterministic_hash(&format!(
        "{kind}|{}|{}|{}",
        report.cases.len(),
        report.passed_count(),
        summary
    ))
}

pub(crate) fn assert_report(label: &str, report: &PassReport) {
    let summary = format_failures(label, &report.cases);
    assert!(summary.is_empty(), "{}", summary);
}
