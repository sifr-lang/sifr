#![expect(
    clippy::print_stderr,
    reason = "E2E timing and qualification reporting"
)]
use super::*;
pub(crate) fn run_pass_suite(config: &RunnerConfig) -> PassReport {
    let suite = telemetry::Span::start(
        "suite",
        None,
        serde_json::json!({
        "sifr_jobs":config.sifr_jobs,"native_workers":1,"cargo_jobs":config.cargo_build_jobs,
        "run_jobs":config.run_jobs,"native_safety_deadline_seconds":1800}),
    );
    let stage = |name, details| telemetry::Span::start(name, Some(&suite.id), details);
    let discovery = stage("discovery", serde_json::json!({}));
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

    discovery.finish("pass", serde_json::json!({"selected":fixtures.len()}));
    let lease_stage = stage(
        "cache_lease",
        serde_json::json!({"selected":fixtures.len()}),
    );
    let lease = match native_cache::NativeCache::acquire(&config.cache.root) {
        Ok(lease) => lease,
        Err(error) => {
            lease_stage.finish(
                "fail",
                serde_json::json!({"cause":"infrastructure","error":error.to_string()}),
            );
            suite.finish("fail", serde_json::json!({"selected":fixtures.len(),"blocked":fixtures.len(),"cause":"infrastructure"}));
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
    lease_stage.finish("pass", serde_json::json!({}));
    let mut owned_config = config.clone();
    owned_config.cache.root = lease.root.clone();
    let config = &owned_config;
    eprintln!(
        "[sifr-e2e] native workers: requested={} actual=1 cargo_jobs={}",
        config.rust_jobs, config.cargo_build_jobs
    );
    let compile_stage = stage(
        "source_compile",
        serde_json::json!({"selected":fixtures.len(),"workers":config.sifr_jobs}),
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

    compile_stage.finish(telemetry::status(compiled_failures.is_empty()), serde_json::json!({
        "selected":fixtures.len(),"passed":compiled_cases.len(),"failed":compiled_failures.len(),"cause":"compiler_diagnostic_if_failed"}));
    let planning_stage = stage(
        "planning",
        serde_json::json!({"selected":compiled_cases.len()}),
    );
    let plan_started = Instant::now();
    let (groups, planning_failures) = plan_batches(compiled_cases);
    let plan_ms = plan_started.elapsed().as_millis();

    planning_stage.finish(
        telemetry::status(planning_failures.is_empty()),
        serde_json::json!({"groups":groups.len(),"failed":planning_failures.len()}),
    );
    let setup_stage = stage("native_setup", serde_json::json!({}));
    let toolchain = toolchain_info();
    let env_signature = cache_env_signature();
    setup_stage.finish("pass", serde_json::json!({}));
    let cache_stage = stage("cache_prepare", serde_json::json!({}));
    let initial_manifest = initial_cache_manifest(config);

    cache_stage.finish(
        "pass",
        serde_json::json!({"entries":initial_manifest.entries.len()}),
    );
    let build_started = Instant::now();
    let native_stage = stage(
        "native_build",
        serde_json::json!({"selected_groups":groups.len(),"native_workers":1,"cargo_jobs":config.cargo_build_jobs}),
    );
    let (original_builds, updated_manifest) = build_batch_suite(
        &groups,
        config,
        &toolchain,
        &env_signature,
        &initial_manifest,
    );
    let original_build_ms: u128 = original_builds.iter().map(|o| o.build_ms).sum();
    let replaced_build_ms: u128 = original_builds
        .iter()
        .filter(|o| o.source_failure && o.group.cases.len() > 1)
        .map(|o| o.build_ms)
        .sum();
    native_stage.finish(
        telemetry::status(original_builds.iter().all(|o| o.build_error.is_none())),
        serde_json::json!({"selected_groups":groups.len(),"attempt_work_ms":original_build_ms}),
    );
    let isolation_stage = stage(
        "native_isolation",
        serde_json::json!({"safety_budget_seconds":1800}),
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
    isolation_stage.finish(telemetry::status(batch_failures.is_empty()), serde_json::json!({"original_failed_batches":batch_failures.len(),"execution_groups":build_outcomes.len()}));
    // Include discarded failed originals as well as individual diagnostic work.
    let observed_build_ms: u128 = replaced_build_ms
        + build_outcomes
            .iter()
            .map(|outcome| outcome.build_ms)
            .sum::<u128>();
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
    let runtime_stage = stage(
        "runtime",
        serde_json::json!({"groups":build_outcomes.len(),"workers":config.run_jobs}),
    );
    let run_started = Instant::now();
    let (run_cases, run_outcomes) = run_batch_suite(&build_outcomes, config);
    let runtime_selected = run_cases.len();
    let runtime_failed = run_cases.iter().filter(|o| o.status.is_err()).count();
    let runtime_blocked: usize = build_outcomes
        .iter()
        .filter(|o| o.build_error.is_some())
        .map(|o| o.group.cases.len())
        .sum();
    all_cases.extend(run_cases);
    all_cases.extend(compiled_failures);
    all_cases.extend(planning_failures);
    let run_ms = run_started.elapsed().as_millis();
    runtime_stage.finish(telemetry::status(runtime_failed == 0), serde_json::json!({"selected":runtime_selected,"failed_or_blocked":runtime_failed,"blocked":runtime_blocked,"note":"per-case worker times overlap"}));
    let reporting_stage = stage("reporting_and_retention", serde_json::json!({}));

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
    reporting_stage.finish("pass", serde_json::json!({"selected":all_cases.len()}));
    suite.finish(telemetry::status(batch_failures.is_empty() && all_cases.iter().all(|o|o.status.is_ok())), serde_json::json!({
        "selected":fixtures.len(),"outcomes":all_cases.len(),"passed":all_cases.iter().filter(|o|o.status.is_ok()).count(),
        "failed":all_cases.iter().filter(|o|o.status.is_err()).count(),"original_batch_failures":batch_failures.len(),
        "native_attempt_work_ms":observed_build_ms}));
    PassReport {
        cases: all_cases,
        batch_failures,
    }
}
