#![expect(
    clippy::print_stderr,
    reason = "the E2E harness reports cache and batch execution status"
)]

use super::*;
pub(crate) fn build_batch_group(
    group: BatchGroup,
    config: &RunnerConfig,
    toolchain: &ToolchainInfo,
    env_signature: &str,
    manifest: &Arc<Mutex<CacheManifest>>,
) -> GroupBuildOutcome {
    build_batch_group_bounded(
        group,
        config,
        toolchain,
        env_signature,
        manifest,
        std::time::Duration::from_secs(30 * 60),
    )
}

pub(crate) fn build_batch_group_bounded(
    group: BatchGroup,
    config: &RunnerConfig,
    toolchain: &ToolchainInfo,
    env_signature: &str,
    manifest: &Arc<Mutex<CacheManifest>>,
    deadline: std::time::Duration,
) -> GroupBuildOutcome {
    let mut source_failure = false;
    let mut infrastructure_failure = true;
    let started = Instant::now();
    let group_root = config.cache.root.join("groups").join(&group.id);
    let mut build_error = None;
    let mut build_log: Option<PathBuf> = None;
    let mut artifact = None;
    let mut cache_hit = false;

    if let Err(err) = sifr_cache_storage::directory(&group_root.join("src")) {
        build_error = Some(format!("failed to create batch crate dir: {err}"));
    }

    let cache_key = cache_key_for_group(&group, toolchain, env_signature);
    if build_error.is_none() {
        if let Err(error) =
            native_cache::write_changed(&group_root.join("pending"), b"build/run in progress\n")
        {
            build_error = Some(format!("cannot mark pending group: {error}"));
        }
    }
    if build_error.is_none() {
        let cargo_toml = sifr_driver::generate_dependency_cargo_toml(
            &group.package_name,
            &group.dependency_plan,
        );
        let source_path = group_root.join("src").join("main.rs");
        let cargo_toml_path = group_root.join("Cargo.toml");

        if let Err(err) = native_cache::write_changed(&cargo_toml_path, cargo_toml.as_bytes()) {
            build_error = Some(format!("failed to write Cargo.toml: {err}"));
        } else if let Err(err) =
            native_cache::write_changed(&source_path, group.generated_main.as_bytes())
        {
            build_error = Some(format!("failed to write main.rs: {err}"));
        } else {
            let mut build_command = Command::new("cargo");
            build_command
                .args(sifr_driver::sysroot_cargo_config_args(
                    &group.dependency_plan,
                ))
                .args(["build", "--message-format=json", "-j"])
                .arg(config.cargo_build_jobs.to_string())
                .current_dir(&group_root);
            application_profile().configure(&mut build_command);
            build_command.env("CARGO_TARGET_DIR", config.cache.root.join("native-target"));
            let build_capture = run_capture_bounded(build_command, deadline);
            source_failure =
                batch_isolation::is_source_failure(&build_capture, &group.package_name);
            // Normal Cargo compile errors can be unclassified source/dependency
            // failures. Only an interrupted/failed launch or non-Cargo exit is a
            // known infrastructure stop; unclassified diagnostics do not stop
            // unrelated original groups.
            infrastructure_failure = !matches!(build_capture.exit_code, Some(0 | 101));
            let selected = native_cache::select_artifact(
                build_capture.status_ok,
                &build_capture.stdout,
                &group_root,
                &config.cache.root.join("native-target"),
                &group.package_name,
            )
            .and_then(|selected| {
                native_cache::record_retirement(
                    &group_root,
                    &config.cache.root.join("native-target"),
                    &sifr_driver::sysroot_cargo_config_args(&group.dependency_plan),
                    &selected.path,
                )
                .map_err(|error| error.to_string())?;
                Ok(selected)
            });
            if let Ok(selected) = selected.as_ref() {
                artifact = Some(selected.path.clone());
                cache_hit = selected.fresh;
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
                infrastructure_failure |= build_capture.status_ok;
                let _ =
                    native_cache::write_changed(&group_root.join("preserve"), b"failed build\n");
                let log_path = group_root.join(format!(
                    "build-{}.log",
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos()
                ));
                let mut diagnostic = String::new();
                let _ = std::fmt::Write::write_str(
                    &mut diagnostic,
                    &format!(
                        "Rust build failed for {} ({})\n\nSTDOUT:\n{}\n\nSTDERR:\n{}\n{}\n",
                        group.id,
                        group.fingerprint.hash(),
                        build_capture.stdout,
                        build_capture.stderr,
                        selected.unwrap_err()
                    ),
                );
                let _ = std::fmt::Write::write_str(&mut diagnostic, "Generated Rust:\n");
                let _ = std::fmt::Write::write_str(&mut diagnostic, &group.generated_main);
                if let Err(err) = std::fs::write(&log_path, diagnostic) {
                    infrastructure_failure = true;
                    source_failure = false;
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
        source_failure,
        infrastructure_failure,
        execution_deadline: None,
        group_root,
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
    // A shared target has one mutable publication namespace. Keep Cargo serial;
    // the explicit inner job budget controls dependency compilation parallelism.
    let mut outcomes = Vec::with_capacity(groups.len());
    let mut stopped = false;
    for group in groups {
        let outcome = if stopped {
            batch_isolation::blocked(group.clone(), config, "prior native infrastructure failure")
        } else {
            build_batch_group(
                group.clone(),
                config,
                toolchain,
                env_signature,
                &shared_manifest,
            )
        };
        stopped |= outcome.build_error.is_some() && outcome.infrastructure_failure;
        outcomes.push(outcome);
    }

    let next_manifest = shared_manifest
        .lock()
        .map_or_else(|_| manifest.clone(), |lock| lock.clone());
    if config.cache.enabled {
        write_cache_manifest(&config.cache.root, &next_manifest);
    }

    (outcomes, next_manifest)
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
                    group_outcome.group_root.display(),
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
                    group_outcome.group_root.display(),
                )),
            })
            .collect();
    };

    group
        .cases
        .iter()
        .map(|case| {
            let status =
                match fixture_execution::run_single_case_bounded(artifact, &case.fixture.name, &group_outcome.group_root,
                    group_outcome.execution_deadline.map_or(std::time::Duration::from_secs(30 * 60), |end| end.saturating_duration_since(Instant::now()))) {
                    Ok(()) => Ok(()),
                    Err(err) => Err(format!(
                        "FAIL [{}]: {}\n  group: {}\n  group fingerprint: {}\n  crate: {}\n  artifact: {}",
                        case.fixture.name,
                        err,
                        group.id,
                        group.fingerprint.hash(),
                        group_outcome.group_root.display(),
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
    application_profile().configure(&mut build_command);
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
    let binary_path = tmp_dir
        .join("target")
        .join(application_directory())
        .join(binary_name);
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
            if let Err(payload) = handle.join() {
                std::panic::resume_unwind(payload);
            }
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
    assert_eq!(
        ordered.len(),
        items.len(),
        "parallel work lost a selected outcome"
    );
    ordered
}

pub(crate) fn run_pass_suite(config: &RunnerConfig) -> PassReport {
    let fixtures = discover_fixtures(Path::new("tests/e2e/pass"));
    assert!(!fixtures.is_empty(), "No pass tests found");
    if env::var("SIFR_E2E_PROFILE").ok().as_deref() == Some("release") {
        assert_eq!(
            application_profile(),
            sifr_driver::ApplicationProfile::Release
        );
        let authority: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../verification/runner/application_profiles.json"
        ))
        .expect("application selection authority");
        let expected = authority["selections"][0]["fixtures"]
            .as_array()
            .expect("release inventory")
            .iter()
            .map(|v| v.as_str().expect("fixture name"))
            .collect::<Vec<_>>();
        let actual = fixtures
            .iter()
            .map(|fixture| {
                fixture
                    .path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .expect("fixture filename")
            })
            .collect::<Vec<_>>();
        assert_eq!(
            actual, expected,
            "release qualification must execute every declared run-pass fixture"
        );
    }

    let lease = match native_cache::NativeCache::acquire(&config.cache.root) {
        Ok(lease) => lease,
        Err(error) => {
            return PassReport {
                batch_failures: Vec::new(),
                cases: fixtures
                    .iter()
                    .map(|fixture| FixtureExecution {
                        name: fixture.name.clone(),
                        status: Err(format!("native cache unavailable: {error}")),
                    })
                    .collect(),
            };
        }
    };
    let mut owned_config = config.clone();
    owned_config.cache.root = lease.root.clone();
    let config = &owned_config;
    eprintln!(
        "[sifr-e2e] native workers: requested={} actual=1 cargo_jobs={}",
        config.rust_jobs, config.cargo_build_jobs
    );
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
    let initial_manifest = initial_cache_manifest(config);

    let build_started = Instant::now();
    let (original_builds, updated_manifest) = build_batch_suite(
        &groups,
        config,
        &toolchain,
        &env_signature,
        &initial_manifest,
    );
    let (build_outcomes, batch_failures) = batch_isolation::isolate_failed_batches(
        original_builds,
        config,
        &toolchain,
        &env_signature,
        &updated_manifest,
        std::time::Duration::from_secs(30 * 60),
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

    // The lease spans consumers too. Failed and interrupted evidence is never
    // retired by TTL pruning; dependencies live outside the group directory.
    for outcome in &build_outcomes {
        let root = config.cache.root.join("groups").join(&outcome.group.id);
        let succeeded = outcome.build_error.is_none()
            && outcome.group.cases.iter().all(|case| {
                all_cases
                    .iter()
                    .any(|result| result.name == case.fixture.name && result.status.is_ok())
            });
        if succeeded {
            let _ = std::fs::remove_file(root.join("pending"));
            let _ = native_cache::write_changed(&root.join("completed"), b"completed\n");
        } else {
            let _ = native_cache::write_changed(&root.join("preserve"), b"failed build/run\n");
        }
    }
    all_cases.sort_by(|left, right| left.name.cmp(&right.name));
    PassReport {
        cases: all_cases,
        batch_failures,
    }
}

pub(crate) fn initial_cache_manifest(config: &RunnerConfig) -> CacheManifest {
    if !config.cache.enabled {
        return CacheManifest {
            schema_version: E2E_CACHE_SCHEMA_VERSION,
            entries: BTreeMap::new(),
        };
    }
    let manifest = read_cache_manifest(&config.cache.root);
    let pruned = prune_cache_manifest(
        &config.cache.root,
        manifest,
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or_default(),
        |group_root| {
            for args in native_cache::retirement_commands(
                group_root,
                &config.cache.root.join("native-target"),
                &package_name_from_id(
                    group_root
                        .file_name()
                        .and_then(|name| name.to_str())
                        .ok_or("invalid group path")?,
                ),
            )
            .map_err(|e| e.to_string())?
            {
                let mut command = Command::new("cargo");
                command.args(args).current_dir(group_root);
                let capture = run_capture(command);
                if !capture.status_ok {
                    let log = group_root.join(format!(
                        "retirement-failure-{}.log",
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_nanos()
                    ));
                    let _ = std::fs::write(&log, format!("{}\n{}", capture.stdout, capture.stderr));
                    return Err(format!(
                        "package retirement failed; retained {}",
                        log.display()
                    ));
                }
            }
            Ok(())
        },
    );
    write_cache_manifest(&config.cache.root, &pruned);
    pruned
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
    let summary = report_summary(kind, report);
    deterministic_hash(&format!(
        "{kind}|{}|{}|{}",
        report.cases.len(),
        report.passed_count(),
        summary
    ))
}

pub(crate) fn assert_report(label: &str, report: &PassReport) {
    let summary = report_summary(label, report);
    assert!(summary.is_empty(), "{}", summary);
}

fn report_summary(label: &str, report: &PassReport) -> String {
    let mut summary = format_failures(label, &report.cases);
    let mut batches = report.batch_failures.clone();
    batches.sort();
    if !batches.is_empty() {
        summary.push_str(&format!(
            "\nOriginal native batch failures ({} unique fixtures passed, {} failed):\n{}",
            report.passed_count(),
            report.failed_count(),
            batches.join("\n")
        ));
    }
    summary
}
