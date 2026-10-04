"""Candidate-bound aggregate acceptance; selection alone never qualifies work."""
from __future__ import annotations

from datetime import datetime, timezone
import re

CONTEXT = "validation-required"
WORKFLOW = ".github/workflows/local-first-validation.yml"
PUBLISHER = ".github/workflows/validation-required.yml"
PLATFORMS = ("aarch64-apple-darwin", "aarch64-unknown-linux-gnu",
             "x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu")


def component_steps(name: str) -> set[str]:
    if not name.startswith('compiler-component-'): return set()
    steps = {'Prepare locked target dependencies', 'Run native compiler component qualification',
             'Run native SQL build qualification'}
    if name.endswith('x86_64-pc-windows-msvc'):
        steps.update({'Run Windows native executable selection',
                      'Run Windows generated Rust formatter and native build script contracts',
                      'Run coupled Windows driver storage and process contracts'})
    if name.endswith('x86_64-unknown-linux-gnu'):
        steps.add('Run Unix generated Rust formatter and native build script contracts')
    return steps


def expected_jobs(event: str, profile: str) -> set[str]:
    if event not in {"pull_request", "merge_group", "push"}:
        raise ValueError("event cannot qualify protected delivery")
    if profile not in {"create-pr", "merge"} or (event != "pull_request" and profile != "merge"):
        raise ValueError("incorrect protected profile")
    jobs = {"uv-toolchain-invariant", "validation-selection", "local-first-" + profile,
            "smoke-fuzz-property", "sql-build-wasm32-wasip2"}
    jobs.update("compiler-component-" + platform for platform in PLATFORMS)
    if event == "pull_request":
        jobs.add("deterministic-report-signature")
    return jobs


def utc(value: str) -> datetime:
    if not isinstance(value, str):
        raise ValueError('timestamp is unavailable')
    parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    if parsed.utcoffset() is None:
        raise ValueError("timestamp lacks UTC offset")
    return parsed.astimezone(timezone.utc)


def evaluate(run: dict, jobs: list[dict], *, candidate: str, profile: str,
             repository: str, workflow_id: int, workflow_matches: bool,
             now: datetime, reused_jobs: set[str] | None = None) -> list[str]:
    errors = []
    reused_jobs = reused_jobs or set()
    reusable = {'smoke-fuzz-property', 'sql-build-wasm32-wasip2'} | {'compiler-component-' + target for target in PLATFORMS}
    if reused_jobs and (run.get('event') != 'push' or not reused_jobs <= reusable):
        errors.append('reuse is restricted to exact-commit main correctness jobs')
        reused_jobs = set()
    if not re.fullmatch(r"[0-9a-f]{40}", candidate):
        errors.append("candidate must be a complete commit SHA")
    if (run.get("repository", {}).get("full_name") != repository
            or run.get("workflow_id") != workflow_id or run.get("path") != WORKFLOW
            or not workflow_matches):
        errors.append("untrusted workflow provenance")
    if run.get("status") != "completed" or run.get("conclusion") != "success":
        errors.append("producer run did not complete successfully")
    try:
        required = expected_jobs(run.get("event"), profile)
    except ValueError as error:
        errors.append(str(error))
        required = set()
    # PR runs name the branch head in GitHub metadata; the trusted publisher
    # independently resolves and verifies its actual synthetic merge candidate.
    if run.get("event") != "pull_request" and run.get("head_sha") != candidate:
        errors.append("run candidate differs from required commit")
    observed = {}
    for job in jobs:
        name = job.get("name")
        if name in observed:
            errors.append(f"duplicate job: {name}")
        observed[name] = job
    for name in sorted(required):
        job = observed.get(name)
        if not job:
            errors.append(f"missing mandatory job: {name}")
            continue
        expected_conclusion = "skipped" if name in reused_jobs and not name.startswith('compiler-component-') else "success"
        if job.get("status") != "completed" or job.get("conclusion") != expected_conclusion:
            errors.append(f"mandatory job did not pass: {name}")
        if job.get("run_id") != run.get("id") or job.get("run_attempt") != run.get("run_attempt"):
            errors.append(f"job comes from a different run/attempt: {name}")
        if name in reused_jobs:
            # This attempt did not execute the job. The caller independently
            # verifies the original producer's actual execution and freshness.
            if name.startswith('compiler-component-') and not any(
                    step.get('name') == 'Record exact-commit correctness reuse' and step.get('conclusion') == 'success'
                    for step in job.get('steps', [])):
                errors.append(f'missing current reuse marker: {name}')
            continue
        steps = {step.get('name'): step for step in job.get('steps', [])}
        for step in component_steps(name):
            if steps.get(step, {}).get('conclusion') != 'success':
                errors.append(f'mandatory component step did not pass: {name}: {step}')
        try:
            completed = utc(job["completed_at"])
            started = utc(job["started_at"])
            if started > completed or completed > now or (now - completed).total_seconds() > 86400:
                raise ValueError("invalid or stale completion")
        except (KeyError, ValueError, TypeError):
            errors.append(f"missing, invalid or stale job timestamp: {name}")
    return errors
