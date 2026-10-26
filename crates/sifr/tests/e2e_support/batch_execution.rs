#![expect(
    clippy::print_stderr,
    reason = "the E2E harness reports cache and batch execution status"
)]

use super::*;
pub(crate) fn build_batch_group(
    lease: &native_cache::NativeCache,
    group: BatchGroup,
    config: &RunnerConfig,
    toolchain: &ToolchainInfo,
    env_signature: &str,
    manifest: &Arc<Mutex<CacheManifest>>,
) -> GroupBuildOutcome {
    build_batch_group_bounded(
        lease,
        group,
        config,
        toolchain,
        env_signature,
        manifest,
        std::time::Duration::from_secs(30 * 60),
    )
}

pub(crate) fn build_batch_group_bounded(
    lease: &native_cache::NativeCache,
    group: BatchGroup,
    config: &RunnerConfig,
    toolchain: &ToolchainInfo,
    env_signature: &str,
    manifest: &Arc<Mutex<CacheManifest>>,
    deadline: std::time::Duration,
) -> GroupBuildOutcome {
    let span = telemetry::Span::start(
        "native_attempt",
        None,
        serde_json::json!({
        "group":group.id,"selected":group.cases.iter().map(|c|c.fixture.name.as_str()).collect::<Vec<_>>(),
        "cargo_jobs":config.cargo_build_jobs,"safety_deadline_ms":deadline.as_millis()}),
    );
    let mut units = serde_json::Value::Null;
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
        } else if let Err(error) = native_resolution::prepare(
            lease,
            &group,
            &group_root,
            deadline.saturating_sub(started.elapsed()),
        ) {
            infrastructure_failure = error.infrastructure;
            build_error = Some(error.message);
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
            let build_capture = run_leased_capture(
                build_command,
                lease,
                deadline.saturating_sub(started.elapsed()),
            );
            units = telemetry::cargo_units(&build_capture.stdout);
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

    span.finish(telemetry::status(build_error.is_none()), serde_json::json!({
        "group":group.id,"artifact_fresh":cache_hit,"compiler_units":units,
        "cause":if build_error.is_none() {"success"} else if source_failure {"correctness"}
            else if infrastructure_failure {"infrastructure_or_safety; see process cause"} else {"unclassified_native_failure"}}));
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
    lease: &native_cache::NativeCache,
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
                lease,
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

pub(crate) fn run_batch_outcomes(
    lease: &native_cache::NativeCache,
    group_outcome: &GroupBuildOutcome,
) -> Vec<FixtureExecution> {
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
                match fixture_execution::run_cached_case(lease, artifact, &case.fixture.name,
                    &group_outcome.group_root, group_outcome.execution_deadline.map_or(
                        std::time::Duration::from_secs(30 * 60),
                        |end| end.saturating_duration_since(Instant::now()))) {
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
    lease: &native_cache::NativeCache,
    build_outcomes: &[GroupBuildOutcome],
    config: &RunnerConfig,
) -> (Vec<FixtureExecution>, Vec<GroupRunOutcome>) {
    let mut outputs = Vec::new();
    let mut run_outcomes = Vec::with_capacity(build_outcomes.len());

    let per_group = run_in_parallel(build_outcomes, config.run_jobs, |group| {
        let started = Instant::now();
        let results = run_batch_outcomes(lease, group);
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

pub(crate) fn initial_cache_manifest(
    lease: &native_cache::NativeCache,
    config: &RunnerConfig,
    groups: &[BatchGroup],
) -> CacheManifest {
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
        &groups.iter().map(|group| group.id.clone()).collect(),
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
                let capture =
                    run_leased_capture(command, lease, std::time::Duration::from_secs(30 * 60));
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

fn run_leased_capture(
    mut command: Command,
    lease: &native_cache::NativeCache,
    deadline: std::time::Duration,
) -> CommandCapture {
    if let Err(error) = lease.bind_cargo_command(&mut command) {
        return CommandCapture {
            stdout: String::new(),
            stderr: format!("cannot bind native cache lease: {error}"),
            status_ok: false,
            exit_code: None,
        };
    }
    run_capture_bounded(command, deadline)
}
