//! Separate raw programs sharing Cargo dependency artifacts, never passing results.
use super::*;
use std::time::Duration;

static PUBLICATION: Mutex<()> = Mutex::new(());

pub(super) struct DirectOutcome {
    pub(super) capture: CommandCapture,
    pub(super) build: GroupBuildOutcome,
}

pub(super) fn direct_group(source: &str, name: &str, plan: SysrootDependencyPlan) -> BatchGroup {
    let source_hash = deterministic_hash(source);
    let fingerprint = DependencyFingerprint {
        dependency_inputs: plan.dependency_input_fingerprint(),
        resolved_plan: plan.cache_fingerprint.clone(),
    };
    let manifest = sifr_driver::generate_dependency_cargo_toml("sifr_direct_identity", &plan);
    let id = deterministic_hash(
        &serde_json::json!([
            "standalone-native-v1",
            name,
            source,
            fingerprint.dependency_inputs,
            manifest,
            sifr_driver::sysroot_cargo_config_args(&plan),
        ])
        .to_string(),
    );
    // This is a native build description, not a request for batch source wrapping.
    BatchGroup {
        package_name: package_name_from_id(&id),
        id,
        fingerprint,
        dependency_plan: plan,
        cases: Vec::new(),
        generated_main: source.into(),
        generated_rust_hash: source_hash,
    }
}

pub(super) fn execute(
    group: BatchGroup,
    requested: &RunnerConfig,
    deadline: Duration,
) -> Result<DirectOutcome, String> {
    let start = Instant::now();
    // Libtest can run the two standalone consumers concurrently. Keep the lease
    // through execution too, so no Cargo publication can replace a live program.
    let _publication = loop {
        match PUBLICATION.try_lock() {
            Ok(guard) => break guard,
            Err(std::sync::TryLockError::Poisoned(_)) => {
                return Err("standalone publication lock is poisoned".into());
            }
            Err(std::sync::TryLockError::WouldBlock) if start.elapsed() < deadline => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(_) => return Err("standalone publication deadline exhausted".into()),
        }
    };
    if start.elapsed() >= deadline {
        return Err("standalone publication deadline exhausted".into());
    }
    let lease = native_cache::NativeCache::acquire_with_wait(
        &requested.cache.root.join("standalone"),
        Duration::from_secs(1).min(deadline.saturating_sub(start.elapsed())),
    )
    .map_err(|error| error.to_string())?;
    let mut config = requested.clone();
    config.cache.root = lease.root.clone();
    let manifest = initial_cache_manifest(&lease, &config, std::slice::from_ref(&group));
    let manifest = Arc::new(Mutex::new(manifest));
    let build = build_batch_group_bounded(
        &lease,
        group,
        &config,
        &toolchain_info(),
        &cache_env_signature(),
        &manifest,
        deadline.saturating_sub(start.elapsed()),
    );
    if config.cache.enabled {
        let saved = manifest.lock().map_err(|error| error.to_string())?;
        write_cache_manifest(&config.cache.root, &saved);
    }
    if let Some(error) = &build.build_error {
        // Standalone callers inspect the original Rust diagnostics as well as
        // failure status. Keep the retained log and return its contents too.
        return Err(match &build.build_log_path {
            Some(path) => match std::fs::read_to_string(path) {
                Ok(log) => format!("{error}\n{log}"),
                Err(read_error) => format!("{error}\ncannot read retained build log: {read_error}"),
            },
            None => error.clone(),
        });
    }
    let artifact = build
        .artifact_path
        .as_ref()
        .ok_or("missing standalone Cargo artifact")?;
    let work = build.group_root.join("work");
    sifr_cache_storage::directory(&work).map_err(|error| error.to_string())?;
    let mut command = Command::new(artifact);
    command.current_dir(work);
    lease
        .bind_command(&mut command)
        .map_err(|error| error.to_string())?;
    let remaining = deadline.saturating_sub(start.elapsed());
    if remaining.is_zero() {
        return Err("standalone execution deadline exhausted".into());
    }
    let capture = run_capture_bounded(command, remaining);
    if !capture.status_ok {
        let log = build.group_root.join(format!(
            "execution-{}.json",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        native_cache::write_changed(
            &log,
            serde_json::json!({
                "exit_code":capture.exit_code, "stdout":capture.stdout, "stderr":capture.stderr,
            })
            .to_string()
            .as_bytes(),
        )
        .map_err(|error| error.to_string())?;
        native_cache::write_changed(
            &build.group_root.join("preserve"),
            b"nonzero or interrupted standalone execution\n",
        )
        .map_err(|error| error.to_string())?;
    }
    if capture.exit_code.is_none() {
        return Err(format!(
            "standalone execution did not complete: {}",
            capture.stderr
        ));
    }
    // Nonzero exits are returned to the original runtime-failure assertions.
    native_cache::write_changed(
        &build.group_root.join("completed"),
        b"completed standalone execution\n",
    )
    .map_err(|error| error.to_string())?;
    std::fs::remove_file(build.group_root.join("pending")).map_err(|error| error.to_string())?;
    Ok(DirectOutcome { capture, build })
}

pub(crate) fn build_and_run_capture_with_deps(
    rust_source: &str,
    test_name: &str,
    stdlib_modules: &HashSet<String>,
    required_features: &HashSet<StdlibFeature>,
    interop: &sifr_driver::InteropBuildPlan,
) -> Result<(String, String, bool), String> {
    let plan = sifr_driver::try_generate_standalone_dependency_plan(
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
    let mut config = runner_config();
    // Cargo tests execute from the crate directory, whose identity producer watches
    // the source tree. Keep relative generated-output roots at checkout scope.
    if !config.cache.root.is_absolute() {
        config.cache.root = sifr_cache_storage::owner_scope()
            .map_err(|error| error.to_string())?
            .join(&config.cache.root);
    }
    let result = execute(
        direct_group(rust_source, test_name, plan),
        &config,
        Duration::from_secs(30 * 60),
    )?;
    Ok((
        result.capture.stdout,
        result.capture.stderr,
        result.capture.status_ok,
    ))
}
