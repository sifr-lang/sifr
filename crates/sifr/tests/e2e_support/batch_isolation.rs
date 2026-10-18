//! One diagnostic pass; never turns an original failed batch into a passing gate.
use super::*;
use std::time::Duration;

pub(super) fn is_source_failure(capture: &CommandCapture, package: &str) -> bool {
    if capture.status_ok || capture.exit_code != Some(101) {
        return false;
    }
    let mut found = false;
    for line in capture.stdout.lines() {
        let Ok(row) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if row["reason"] != "compiler-message" || row["message"]["level"] != "error" {
            continue;
        }
        // Only typed errors in this generated binary are eligible. Dependency,
        // toolchain, linker and unclassified failures stop diagnostic fan-out.
        let code = row["message"]["code"]["code"].as_str().unwrap_or("");
        if row["target"]["name"] != package
            || !code.starts_with('E')
            || matches!(code, "E0460" | "E0463" | "E0786")
        {
            return false;
        }
        found = true;
    }
    found
}

pub(super) fn blocked(group: BatchGroup, config: &RunnerConfig, reason: &str) -> GroupBuildOutcome {
    telemetry::emit(
        serde_json::json!({"schema":1,"event":"blocked","scope":"native_attempt",
        "group":group.id,"selected":group.cases.iter().map(|c|c.fixture.name.as_str()).collect::<Vec<_>>(),
        "status":"blocked","cause":reason,"attempted":false}),
    );
    GroupBuildOutcome {
        group_root: config.cache.root.join("groups").join(&group.id),
        group,
        artifact_path: None,
        build_log_path: None,
        build_error: Some(format!("native work blocked: {reason}")),
        source_failure: false,
        infrastructure_failure: true,
        execution_deadline: None,
        build_ms: 0,
        cache_hit: false,
    }
}

pub(super) fn isolate_failed_batches(
    original: Vec<GroupBuildOutcome>,
    config: &RunnerConfig,
    toolchain: &ToolchainInfo,
    env_signature: &str,
    manifest: &CacheManifest,
    budget: Duration,
) -> (Vec<GroupBuildOutcome>, Vec<String>) {
    let deadline = Instant::now() + budget;
    let shared = Arc::new(Mutex::new(manifest.clone()));
    let mut stop = original
        .iter()
        .any(|row| row.build_error.is_some() && !row.source_failure);
    let mut attempts = Vec::new();
    let mut failures = Vec::new();
    for outcome in original {
        if outcome.build_error.is_none()
            || !outcome.source_failure
            || outcome.group.cases.len() <= 1
        {
            attempts.push(outcome);
            continue;
        }
        failures.push(format!("{}: original native batch failed; {} members; individual diagnosis does not waive failure",
            outcome.group.id, outcome.group.cases.len()));
        // The original directory/log/preserve marker remains untouched.
        let mut cases = outcome.group.cases.clone();
        cases.sort_by(|a, b| a.fixture.name.cmp(&b.fixture.name));
        for case in cases {
            let remaining = deadline.saturating_duration_since(Instant::now());
            let group = match build_group_sources(vec![case.clone()]) {
                Ok(group) => group,
                Err(error) => {
                    let mut group = outcome.group.clone();
                    group.cases = vec![case];
                    attempts.push(blocked(
                        group,
                        config,
                        &format!("individual planning failed: {error}"),
                    ));
                    continue;
                }
            };
            let mut result = if stop || remaining.is_zero() {
                blocked(
                    group,
                    config,
                    "diagnostic budget exhausted or infrastructure unavailable",
                )
            } else {
                build_batch_group_bounded(
                    group,
                    config,
                    toolchain,
                    env_signature,
                    &shared,
                    remaining,
                )
            };
            stop |= result.build_error.is_some() && !result.source_failure;
            result.execution_deadline = Some(deadline);
            attempts.push(result);
        }
    }
    if config.cache.enabled {
        let next = shared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        write_cache_manifest(&config.cache.root, &next);
    }
    (attempts, failures)
}
