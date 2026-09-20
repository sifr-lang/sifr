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
SELECTOR = re.compile(
    r"\$\{\{ fromJSON\(github\.event_name == 'pull_request' && '(.*?)' \|\| '(.*?)'\) \}\}"
)


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
    match = SELECTOR.fullmatch(str(strategy["matrix"]["profile"]))
    if not match:
        errors.append("expected the event-only fromJSON profile selector")
    else:
        for event, value, expected in (
            ("pull_request", match[1], ["create-pr"]),
            ("push", match[2], ["create-pr", "merge", "release"]),
            ("workflow_dispatch", match[2], ["create-pr", "merge", "release"]),
        ):
            if json.loads(value) != expected:
                errors.append(f"{event}: incorrect profile coverage")
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
    return errors


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    document = parse_workflows({WORKFLOW: (root / WORKFLOW).read_text()})[WORKFLOW]
    errors = validate(document)
    if errors:
        raise SystemExit("\n".join(errors))
    # Regression: the original condition must fail before any runner work.
    invalid = copy.deepcopy(document)
    invalid["jobs"]["local-first-profiles"]["if"] = (
        "github.event_name != 'pull_request' || matrix.profile == 'create-pr'"
    )
    assert any("cannot reference matrix" in error for error in validate(invalid))
    # Coverage cannot be narrowed to make admission appear green.
    narrowed = copy.deepcopy(document)
    matrix = narrowed["jobs"]["local-first-profiles"]["strategy"]["matrix"]
    matrix["profile"] = matrix["profile"].replace(', "merge", "release"', '')
    assert any("incorrect profile coverage" in error for error in validate(narrowed))
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
    print("local-first admission and event/profile contracts passed (including regressions)")


if __name__ == "__main__":
    main()
