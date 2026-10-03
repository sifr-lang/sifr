#!/usr/bin/env python3
"""Guard local-first admission and preserve the intended event/profile coverage.

This deliberately checks the small event selector contract, not arbitrary GitHub
expressions. GitHub remains the authority for actual workflow admission.
"""

from __future__ import annotations

import copy
import json
import re
from pathlib import Path

from check_uv_toolchain import parse_workflows

WORKFLOW = ".github/workflows/local-first-validation.yml"
SELECTOR = "${{ fromJSON(needs.validation-selection.outputs.profiles) }}"
CANDIDATE = "${{ github.event.pull_request.merge_commit_sha || github.event.merge_group.head_sha || github.sha }}"


def validate(document: dict) -> list[str]:
    errors = []
    jobs = document["jobs"]
    for name, job in jobs.items():
        # Job-level if is evaluated before the strategy expands the matrix.
        if re.search(r"\bmatrix(?:\.|\[)", str(job.get("if", ""))):
            errors.append(f"{name}: job-level if cannot reference matrix")
    profile = jobs["local-first-profiles"]
    if "if" in profile:
        errors.append("profile coverage must be selected in the matrix, not a job condition")
    strategy = profile["strategy"]
    if strategy.get("fail-fast") is not True:
        errors.append("profile fail-fast must remain enabled")
    if strategy["matrix"]["profile"] != SELECTOR or profile.get("needs") != "validation-selection":
        errors.append("profiles must use the complete candidate-bound selector")
    if document.get("permissions") != {"contents": "read"}:
        errors.append("candidate validation must have read-only permissions")
    events = document.get("on", document.get("true", {}))
    if not {"pull_request", "merge_group", "push", "schedule", "workflow_dispatch"}.issubset(events):
        errors.append("required delivery and scheduled events must remain declared")
    if events.get("merge_group", {}).get("types") != ["checks_requested"]:
        errors.append("merge_group must validate its admitted queue candidate")
    for name, job in jobs.items():
        for step in job.get("steps", []):
            if step.get("uses", "").startswith("actions/checkout@"):
                options = step.get("with", {})
                if options.get("persist-credentials") is not False or options.get("ref") != CANDIDATE:
                    errors.append(f"{name}: checkout must bind the actual candidate without credentials")
    select = jobs.get("validation-selection", {})
    if not any(step.get("run") == "uv run --project verification --locked python scripts/select_ci_validation.py"
               and "if" not in step and not step.get("continue-on-error") for step in select.get("steps", [])):
        errors.append("selection must execute the canonical commit-bound selector")
    if not any(step.get("run") == 'bash scripts/run_all_tests.sh --profile "${{ matrix.profile }}"'
               and "if" not in step and not step.get("continue-on-error", False)
               for step in profile["steps"]):
        errors.append("every selected profile must execute the authoritative runner")
    for name, preparation in (
        ("smoke-fuzz-property", "uv run --project verification --locked python -m sifr_verify.ci_smoke_setup"),
        ("compiler-component-targets", 'cargo fetch --locked'),
        ("sql-wasi-build", "cargo fetch --locked"),
    ):
        steps = jobs[name]["steps"]
        commands = [step.get("run") for step in steps]
        if preparation not in commands or commands.index(preparation) >= len(steps) - 1:
            errors.append(f"{name}: locked preparation must precede assertions")
        elif any(key in steps[commands.index(preparation)] for key in ("if", "continue-on-error")):
            errors.append(f"{name}: preparation must be unconditional and blocking")
    hardening = jobs.get("fuzz-hardening", {})
    if hardening.get("if") != "github.event_name == 'schedule'" or hardening.get("strategy", {}).get("matrix", {}).get("target") != [
            "parser", "lowering", "ownership", "diagnostics", "project_graph"]:
        errors.append("nightly must retain all five explicit fuzz targets")
    if not any(step.get("if") == "always()" and step.get("uses", "").startswith("actions/upload-artifact@")
               for step in hardening.get("steps", [])):
        errors.append("nightly must preserve failed/incomplete fuzz evidence")
    return errors


def validate_publisher(document: dict) -> list[str]:
    errors = []
    event = document.get("on", document.get("true", {}))
    if event != {"workflow_run": {"workflows": ["local-first-validation"], "types": ["completed"]}}:
        errors.append("publisher must use the trusted workflow_run definition")
    jobs = document.get("jobs", {})
    if set(jobs) != {"publish"} or document.get("permissions") != {"contents": "read"}:
        errors.append("publisher must isolate write permissions to its trusted job")
    job = jobs.get("publish", {})
    if job.get("permissions") != {"contents": "read", "actions": "read"}:
        errors.append("publisher Actions token must remain read-only")
    if not any(step.get("id") == "app-token" and step.get("uses") ==
               "actions/create-github-app-token@bcd2ba49218906704ab6c1aa796996da409d3eb1" and
               step.get("with", {}).get("app-id") == "${{ vars.VALIDATION_CHECK_APP_ID }}" and
               step.get("with", {}).get("private-key") == "${{ secrets.VALIDATION_CHECK_APP_PRIVATE_KEY }}" and
               step.get("with", {}).get("permission-checks") == "write" for step in job.get("steps", [])):
        errors.append("publisher requires a separate pinned protected-check integration")
    for step in job.get("steps", []):
        if step.get("uses", "").startswith("actions/checkout@"):
            if step.get("with", {}).get("ref") != "${{ github.sha }}" or step.get("with", {}).get("persist-credentials") is not False:
                errors.append("publisher must never checkout candidate source")
    if not any(step.get("run") == "uv run --project verification --locked python scripts/publish_validation_aggregate.py"
               and "if" not in step and not step.get("continue-on-error") for step in job.get("steps", [])):
        errors.append("publisher must independently reconcile producer facts")
    return errors


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    document = parse_workflows({WORKFLOW: (root / WORKFLOW).read_text()})[WORKFLOW]
    publisher_path = ".github/workflows/validation-required.yml"
    publisher = parse_workflows({publisher_path: (root / publisher_path).read_text()})[publisher_path]
    errors = validate(document) + validate_publisher(publisher)
    if errors:
        raise SystemExit("\n".join(errors))
    # Regression: the original condition must fail before any runner work.
    invalid = copy.deepcopy(document)
    invalid["jobs"]["local-first-profiles"]["if"] = (
        "github.event_name != 'pull_request' || matrix.profile == 'create-pr'"
    )
    assert any("cannot reference matrix" in error for error in validate(invalid))
    # A static or bypassed selector cannot make coverage appear green.
    narrowed = copy.deepcopy(document)
    narrowed["jobs"]["local-first-profiles"]["strategy"]["matrix"]["profile"] = ["create-pr"]
    assert any("candidate-bound selector" in error for error in validate(narrowed))
    wrong_checkout = copy.deepcopy(document)
    wrong_checkout["jobs"]["local-first-profiles"]["steps"][0]["with"]["ref"] = "main"
    assert any("actual candidate" in error for error in validate(wrong_checkout))
    skipped = copy.deepcopy(document)
    for step in skipped["jobs"]["local-first-profiles"]["steps"]:
        if step.get("name") == "Run local-first profile":
            step["if"] = "false"
    assert any("must execute" in error for error in validate(skipped))
    unprepared = copy.deepcopy(document)
    unprepared["jobs"]["sql-wasi-build"]["steps"] = [
        step for step in unprepared["jobs"]["sql-wasi-build"]["steps"]
        if step.get("run") != "cargo fetch --locked"
    ]
    assert any("preparation must precede" in error for error in validate(unprepared))
    untrusted = copy.deepcopy(publisher)
    untrusted["jobs"]["publish"]["steps"][0]["with"]["ref"] = CANDIDATE
    assert any("never checkout candidate" in error for error in validate_publisher(untrusted))
    print("local-first admission and event/profile contracts passed (including regressions)")


if __name__ == "__main__":
    main()
