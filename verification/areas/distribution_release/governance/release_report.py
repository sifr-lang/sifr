"""Canonical release-profile report construction and validation."""

from __future__ import annotations

import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any

from .common import (
    GovernanceError,
    canonical_json_bytes,
    fail,
    require_array,
    require_commit,
    require_exact_keys,
    require_nonempty_string,
    require_object,
    require_positive_int,
    require_schema_v2,
    require_sha256,
    sha256_file,
)

REQUIRED_RELEASE_STEPS = {
    "rust_interop_checks",
    "developer_tooling_checks",
    "documentation_checks",
    "distribution_validation",
}
REQUIRED_SUITES = {
    "rust_interop": {
        "matrix",
        "tiers",
        "compatibility-matrix",
        "stale-drafts",
        "stable-candidate",
    },
    "developer_tooling": {"full", "editor-release"},
    "documentation": {"structure"},
    "distribution_release": {"full", "evidence-custody"},
}


def validate_release_profile_report(
    payload: Any,
    *,
    canonical_bytes: bytes | None = None,
    source_root: Path | None = None,
    expected_profile_sha256: str | None = None,
    verify_artifacts: bool = False,
) -> dict[str, Any]:
    report = require_object(payload, "$")
    require_exact_keys(
        report,
        required={
            "schema_version",
            "report_id",
            "source",
            "profile",
            "command",
            "toolchain",
            "overall_status",
            "steps",
            "result_artifacts",
        },
        location="$",
    )
    require_schema_v2(report)
    require_nonempty_string(report["report_id"], "$.report_id")
    validate_source(report["source"], source_root=source_root)
    validate_profile(report["profile"], expected_profile_sha256=expected_profile_sha256)
    command = require_array(report["command"], "$.command")
    if not command:
        fail("$.command", "must contain the exact invocation")
    for index, item in enumerate(command):
        require_nonempty_string(item, f"$.command[{index}]")
    validate_toolchain(report["toolchain"])
    if report["overall_status"] != "pass":
        fail("$.overall_status", "release evidence must pass")
    validate_steps(report["steps"])
    validate_result_artifacts(
        report["result_artifacts"],
        source_root=source_root,
        verify_artifacts=verify_artifacts,
    )
    if canonical_bytes is not None and canonical_bytes != canonical_json_bytes(report):
        fail("$", "release profile report is not canonical JSON")
    return report


def validate_source(payload: Any, *, source_root: Path | None) -> None:
    source = require_object(payload, "$.source")
    require_exact_keys(
        source,
        required={"commit", "clean", "unresolved", "submodules"},
        location="$.source",
    )
    commit = require_commit(source["commit"], "$.source.commit")
    if source["clean"] is not True or source["unresolved"] is not False:
        fail("$.source", "must identify a clean, resolved source tree")
    submodules = require_object(source["submodules"], "$.source.submodules")
    for path, sha in submodules.items():
        require_nonempty_string(path, "$.source.submodules key")
        require_commit(sha, f"$.source.submodules.{path}")
    if source_root is not None:
        actual_commit = git_output(source_root, "rev-parse", "HEAD")
        if actual_commit != commit:
            fail("$.source.commit", f"does not match checkout commit {actual_commit}")
        if git_output(source_root, "status", "--porcelain", "--untracked-files=all"):
            fail("$.source.clean", "checkout is dirty")
        if git_output(source_root, "diff", "--name-only", "--diff-filter=U"):
            fail("$.source.unresolved", "checkout has unresolved paths")
        if collect_submodules(source_root) != submodules:
            fail("$.source.submodules", "does not match recursive checkout submodules")


def validate_profile(payload: Any, *, expected_profile_sha256: str | None) -> None:
    profile = require_object(payload, "$.profile")
    require_exact_keys(
        profile,
        required={"name", "manifest_sha256", "expanded_selected_areas"},
        location="$.profile",
    )
    if profile["name"] != "release":
        fail("$.profile.name", "must be release")
    digest = require_sha256(profile["manifest_sha256"], "$.profile.manifest_sha256")
    if expected_profile_sha256 is not None and digest != expected_profile_sha256:
        fail("$.profile.manifest_sha256", "does not match the known release profile")
    selections = require_array(profile["expanded_selected_areas"], "$.profile.expanded_selected_areas")
    actual: dict[str, set[str]] = {}
    for index, value in enumerate(selections):
        location = f"$.profile.expanded_selected_areas[{index}]"
        selection = require_object(value, location)
        require_exact_keys(selection, required={"area", "suites"}, location=location)
        area = require_nonempty_string(selection["area"], f"{location}.area")
        if area in actual:
            fail(f"{location}.area", "duplicate area selection")
        suites = require_array(selection["suites"], f"{location}.suites")
        if not suites:
            fail(f"{location}.suites", "must not be empty")
        actual[area] = {
            require_nonempty_string(suite, f"{location}.suites") for suite in suites
        }
        if len(actual[area]) != len(suites):
            fail(f"{location}.suites", "contains duplicate suite selections")
    for area, required in REQUIRED_SUITES.items():
        missing = sorted(required.difference(actual.get(area, set())))
        if missing:
            fail(
                "$.profile.expanded_selected_areas",
                f"missing required {area} suite(s): {', '.join(missing)}",
            )


def validate_toolchain(payload: Any) -> None:
    toolchain = require_object(payload, "$.toolchain")
    require_exact_keys(
        toolchain,
        required={"rustc", "cargo", "uv", "python"},
        location="$.toolchain",
    )
    for name, value in toolchain.items():
        require_nonempty_string(value, f"$.toolchain.{name}")


def validate_steps(payload: Any) -> None:
    steps = require_array(payload, "$.steps")
    if not steps:
        fail("$.steps", "must contain executed steps")
    names: set[str] = set()
    observed_suites: dict[str, set[str]] = {}
    for index, value in enumerate(steps):
        location = f"$.steps[{index}]"
        step = require_object(value, location)
        require_exact_keys(
            step,
            required={"name", "status", "elapsed_ms", "suite_results"},
            location=location,
        )
        name = require_nonempty_string(step["name"], f"{location}.name")
        if name in names:
            fail(f"{location}.name", "duplicate step result")
        names.add(name)
        if step["status"] != "pass":
            fail(f"{location}.status", "must be pass")
        if type(step["elapsed_ms"]) is not int or step["elapsed_ms"] < 0:
            fail(f"{location}.elapsed_ms", "must be a non-negative integer")
        suites = require_array(step["suite_results"], f"{location}.suite_results")
        for suite_index, suite_value in enumerate(suites):
            suite_location = f"{location}.suite_results[{suite_index}]"
            suite = require_object(suite_value, suite_location)
            require_exact_keys(
                suite,
                required={"area", "suite", "status", "case_ids", "result_artifact_sha256"},
                location=suite_location,
            )
            area = require_nonempty_string(suite["area"], f"{suite_location}.area")
            suite_name = require_nonempty_string(suite["suite"], f"{suite_location}.suite")
            if suite["status"] != "pass":
                fail(f"{suite_location}.status", "must be pass")
            case_ids = require_array(suite["case_ids"], f"{suite_location}.case_ids")
            if not case_ids:
                fail(f"{suite_location}.case_ids", "must contain executed case evidence")
            observed_case_ids = {
                require_nonempty_string(case_id, f"{suite_location}.case_ids")
                for case_id in case_ids
            }
            if len(observed_case_ids) != len(case_ids):
                fail(f"{suite_location}.case_ids", "contains duplicate case evidence")
            if area == "developer_tooling" and suite_name == "editor-release":
                if any(not case_id.startswith("editor-release:") for case_id in observed_case_ids):
                    fail(f"{suite_location}.case_ids", "contains non-editor-release evidence")
            require_sha256(
                suite["result_artifact_sha256"],
                f"{suite_location}.result_artifact_sha256",
            )
            if suite_name in observed_suites.setdefault(area, set()):
                fail(suite_location, f"duplicate suite evidence for {area}:{suite_name}")
            observed_suites[area].add(suite_name)
    missing_steps = sorted(REQUIRED_RELEASE_STEPS.difference(names))
    if missing_steps:
        fail("$.steps", f"missing required step(s): {', '.join(missing_steps)}")
    for area, required in REQUIRED_SUITES.items():
        missing = sorted(required.difference(observed_suites.get(area, set())))
        if missing:
            fail("$.steps", f"missing executed {area} suite(s): {', '.join(missing)}")


def validate_result_artifacts(
    payload: Any,
    *,
    source_root: Path | None,
    verify_artifacts: bool,
) -> None:
    artifacts = require_array(payload, "$.result_artifacts")
    if not artifacts:
        fail("$.result_artifacts", "must contain result artifact digests")
    paths: set[str] = set()
    for index, value in enumerate(artifacts):
        location = f"$.result_artifacts[{index}]"
        artifact = require_object(value, location)
        require_exact_keys(artifact, required={"path", "sha256"}, location=location)
        path_text = require_nonempty_string(artifact["path"], f"{location}.path")
        path = Path(path_text)
        if path.is_absolute() or ".." in path.parts:
            fail(f"{location}.path", "must be repository-relative")
        if path_text in paths:
            fail(f"{location}.path", "duplicate artifact")
        paths.add(path_text)
        expected = require_sha256(artifact["sha256"], f"{location}.sha256")
        if verify_artifacts:
            if source_root is None:
                fail(location, "source_root is required to verify artifacts")
            artifact_path = source_root / path
            if not artifact_path.is_file() or sha256_file(artifact_path) != expected:
                fail(f"{location}.sha256", "does not match the result artifact")


def collect_submodules(source_root: Path) -> dict[str, str]:
    output = git_output(source_root, "submodule", "status", "--recursive")
    result: dict[str, str] = {}
    for line in output.splitlines():
        if not line:
            continue
        if line[0] in {"-", "+", "U"}:
            fail("$.source.submodules", f"submodule is not clean/resolved: {line}")
        parts = line[1:].split()
        if len(parts) < 2:
            fail("$.source.submodules", f"invalid submodule status: {line}")
        result[parts[1]] = parts[0]
    return dict(sorted(result.items()))


def git_output(source_root: Path, *args: str) -> str:
    result = subprocess.run(
        ["git", *args],
        cwd=source_root,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode != 0:
        raise GovernanceError(f"git {' '.join(args)} failed: {result.stderr.strip()}")
    return result.stdout.strip()


def canonical_profile_digest(profile_path: Path) -> str:
    try:
        payload = json.loads(profile_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise GovernanceError(f"{profile_path}: invalid release profile: {exc}") from exc
    return hashlib.sha256(canonical_json_bytes(payload)).hexdigest()
