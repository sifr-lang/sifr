#!/usr/bin/env python3
"""Publish a protected check using GitHub facts from a trusted workflow-run job.

Never checkout or execute candidate code here. The invoking workflow checks out
its protected default-branch implementation with only this job able to write checks.
"""
from __future__ import annotations

from datetime import datetime, timezone
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import urllib.request

from validation_aggregate_policy import CONTEXT, WORKFLOW, evaluate
from validation_candidate_artifact import candidate_identity

ROOT = Path(__file__).resolve().parents[1]


def api(path: str, body: dict | None = None):
    request = urllib.request.Request(
        os.environ.get("GITHUB_API_URL", "https://api.github.com") + path,
        data=None if body is None else json.dumps(body).encode(),
        headers={"Authorization": "Bearer " + os.environ["CHECK_TOKEN" if body is not None else "GH_TOKEN"],
                 "Accept": "application/vnd.github+json", "Content-Type": "application/json",
                 "X-GitHub-Api-Version": "2022-11-28"},
        method="GET" if body is None else "POST",
    )
    with urllib.request.urlopen(request, timeout=30) as response:
        return json.load(response)


def sha(value: str) -> str:
    if not isinstance(value, str) or not re.fullmatch(r"[0-9a-f]{40}", value):
        raise ValueError("GitHub candidate identity is unavailable")
    return value


def fetch(commit: str) -> None:
    subprocess.run(["git", "fetch", "--no-tags", "origin", sha(commit)], cwd=ROOT,
                   check=True, stdout=subprocess.DEVNULL, timeout=120)


def main() -> int:
    # The future branch rule pins this separate integration. An Actions token
    # or a same-named Actions job cannot satisfy that rule.
    app_id = int(os.environ["CHECK_APP_ID"])
    if app_id <= 0 or app_id == 15368:
        raise ValueError("protected checks require a dedicated GitHub App identity")
    repository = os.environ["GITHUB_REPOSITORY"]
    run_id = int(os.environ["VALIDATION_RUN_ID"])
    prefix = "/repos/" + repository
    run = api(f"{prefix}/actions/runs/{run_id}")
    workflow = api(f"{prefix}/actions/workflows/local-first-validation.yml")
    candidate = sha(run["head_sha"])
    check_head = candidate
    errors = []
    profile = "merge"
    if run["event"] == "pull_request":
        snapshots = run.get("pull_requests", [])
        if len(snapshots) != 1:
            raise ValueError("producer must identify one exact pull request")
        snapshot = snapshots[0]
        current = api(f"{prefix}/pulls/{int(snapshot['number'])}")
        candidate = sha(current["merge_commit_sha"])
        check_head = sha(current["head"]["sha"])
        if (current["head"]["sha"] != run["head_sha"] or
                current["base"]["sha"] != snapshot["base"]["sha"] or current["state"] != "open"):
            errors.append("PR source/base changed or PR closed after producer admission")
        base = sha(current["base"]["sha"])
        fetch(base)
        fetch(candidate)
        # Import only the checked-out trusted default-branch selector. Git reads
        # the untrusted commit objects without ever running their source/hooks.
        sys.path.insert(0, str(ROOT / "verification/runner"))
        from sifr_verify.change_selection import selection
        profile = selection(ROOT, base, candidate)["profile"]
    identity = candidate_identity(api, prefix, run)
    if sha(identity["candidate_sha"]) != candidate:
        errors.append("executed checkout artifact differs from current required candidate")
    trusted = sha(os.environ["TRUSTED_WORKFLOW_SHA"])
    observed_workflow = api(f"{prefix}/contents/{WORKFLOW}?ref={candidate}")
    trusted_workflow = api(f"{prefix}/contents/{WORKFLOW}?ref={trusted}")
    matches = observed_workflow.get("sha") == trusted_workflow.get("sha")
    jobs = []
    page = 1
    while True:
        response = api(f"{prefix}/actions/runs/{run_id}/attempts/{int(run['run_attempt'])}/jobs?per_page=100&page={page}")
        jobs.extend(response["jobs"])
        if len(jobs) >= response["total_count"]:
            if len(jobs) != response["total_count"]:
                raise ValueError("job pagination did not preserve complete inventory")
            break
        if not response["jobs"] or page >= 100:
            raise ValueError("job inventory is incomplete")
        page += 1
    errors.extend(evaluate(run, jobs, candidate=candidate, profile=profile,
                           repository=repository, workflow_id=workflow["id"],
                           workflow_matches=matches, now=datetime.now(timezone.utc)))
    # Missing publication itself leaves a required context absent and blocks.
    conclusion = "failure" if errors else "success"
    summary = ("\n".join(errors) if errors else
               "Every mandatory current-attempt job passed for this exact candidate.")
    summary += f"\nTested merge/source candidate: {candidate}\nProtected check head: {check_head}"
    published = api(prefix + "/check-runs", {"name": CONTEXT, "head_sha": check_head, "status": "completed",
        "conclusion": conclusion, "details_url": run["html_url"],
        "output": {"title": "Protected validation " + conclusion, "summary": summary},
        "external_id": f"validation:{run_id}:{run['run_attempt']}:{candidate}"})
    if published.get("app", {}).get("id") != app_id or published.get("app", {}).get("slug") == "github-actions":
        raise ValueError("published check did not use the dedicated pinned integration")
    print(f"{CONTEXT}: {conclusion} check_head={check_head} tested_candidate={candidate} producer={run_id}")
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
