"""Create immutable canonical evidence for a passing release profile."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path
from typing import Any

from . import reports
from .paths import REPO_ROOT
from .profiles import load_profile

GOVERNANCE_ROOT = REPO_ROOT / "verification" / "areas" / "distribution_release"
sys.path.insert(0, str(GOVERNANCE_ROOT))

from governance.common import (  # noqa: E402
    GovernanceError,
    canonical_json_bytes,
    load_json_strict,
    sha256_file,
    write_canonical_json,
)
from governance.release_report import (  # noqa: E402
    canonical_profile_digest,
    collect_submodules,
    validate_release_profile_report,
)

CRITICAL_RESULTS = {
    "rust_interop": "rust-interop-release-results.json",
    "developer_tooling": "developer-tooling-release-results.json",
    "documentation": "documentation-release-results.json",
    "distribution_release": "distribution-release-release-results.json",
}
AREA_TO_STEP = {
    "rust_interop": "area_rust_interop",
    "developer_tooling": "area_developer_tooling",
    "documentation": "area_documentation",
    "distribution_release": "area_distribution_release",
}


def prepare_release_report_output(path_text: str, *, profile_name: str) -> Path:
    if profile_name != "release":
        raise GovernanceError("--release-report-out is accepted only for the release profile")
    path = Path(path_text).expanduser()
    if not path.is_absolute():
        path = Path.cwd() / path
    path = path.resolve(strict=False)
    repo = REPO_ROOT.resolve()
    if path == repo or path.is_relative_to(repo):
        raise GovernanceError("--release-report-out must be outside the repository checkout")
    if path.exists():
        raise GovernanceError(f"release report output already exists: {path}")
    if path.parent.exists() and any(path.parent.iterdir()):
        raise GovernanceError(
            f"release report parent must be a fresh directory: {path.parent}"
        )
    ensure_clean_source()
    path.parent.mkdir(parents=True, exist_ok=True)
    clear_critical_result_files()
    return path


def ensure_clean_source() -> None:
    dirty = git_output("status", "--porcelain", "--untracked-files=all")
    if dirty:
        raise GovernanceError("release report requires a clean source tree")
    unresolved = git_output("diff", "--name-only", "--diff-filter=U")
    if unresolved:
        raise GovernanceError("release report source contains unresolved paths")
    collect_submodules(REPO_ROOT)


def clear_critical_result_files() -> None:
    result_root = REPO_ROOT / "target" / "verification" / "areas"
    for filename in CRITICAL_RESULTS.values():
        (result_root / filename).unlink(missing_ok=True)


def write_release_profile_report(
    output_path: Path,
    *,
    log_path: Path,
    status: int,
) -> None:
    if status != 0:
        raise GovernanceError("cannot write release evidence for a failing profile")
    result_root = REPO_ROOT / "target" / "verification" / "areas"
    canonicalize_custodied_results(result_root)
    profile = load_profile("release")
    profile_path = REPO_ROOT / "verification" / "profiles" / "release.json"
    profile_digest = canonical_profile_digest(profile_path)
    commit = git_output("rev-parse", "HEAD")
    payload = build_release_profile_payload(
        output_path=output_path,
        log_path=log_path,
        profile=profile,
        profile_digest=profile_digest,
        commit=commit,
        submodules=collect_submodules(REPO_ROOT),
        toolchain={
            "rustc": command_version("rustc", "--version"),
            "cargo": command_version("cargo", "--version"),
            "uv": command_version("uv", "--version"),
            "python": command_version(sys.executable, "--version"),
        },
        result_root=result_root,
        artifact_root=REPO_ROOT,
    )
    validate_release_profile_report(
        payload,
        source_root=REPO_ROOT,
        expected_profile_sha256=profile_digest,
        verify_artifacts=True,
    )
    write_canonical_json(output_path, payload, refuse_existing=True)
    validate_release_profile_report(
        json.loads(output_path.read_text(encoding="utf-8")),
        canonical_bytes=output_path.read_bytes(),
        source_root=REPO_ROOT,
        expected_profile_sha256=profile_digest,
        verify_artifacts=True,
    )
    print(f"release_profile_report={output_path}")


def canonicalize_custodied_results(result_root: Path) -> None:
    """Canonicalize exact result bytes that enter candidate evidence custody."""
    path = result_root / CRITICAL_RESULTS["rust_interop"]
    if not path.is_file() or path.is_symlink():
        raise GovernanceError(
            "release profile emitted no critical area result: rust_interop"
        )
    payload = load_json_strict(path)
    if not isinstance(payload, dict) or payload.get("area") != "rust_interop":
        raise GovernanceError(f"critical area result identity mismatch: {path}")
    path.write_bytes(canonical_json_bytes(payload))


def build_release_profile_payload(
    *,
    output_path: Path,
    log_path: Path,
    profile: dict[str, Any],
    profile_digest: str,
    commit: str,
    submodules: dict[str, str],
    toolchain: dict[str, str],
    result_root: Path,
    artifact_root: Path,
) -> dict[str, Any]:
    """Assemble report bytes from already captured, caller-pinned evidence."""
    step_payloads = build_steps(log_path)
    result_artifacts, suite_results = collect_critical_results(
        result_root=result_root,
        artifact_root=artifact_root,
    )
    by_step: dict[str, list[dict[str, Any]]] = {}
    for area, entries in suite_results.items():
        by_step.setdefault(AREA_TO_STEP[area], []).extend(entries)
    for step in step_payloads:
        step["suite_results"] = by_step.get(str(step["name"]), [])

    expanded_by_area: dict[str, set[str]] = {}
    for selection in profile["selected_areas"]:
        area = str(selection["area"])
        expanded_by_area.setdefault(area, set()).update(
            expand_suites(area, [str(suite) for suite in selection["suites"]])
        )
    expanded = [
        {"area": area, "suites": sorted(suites)}
        for area, suites in sorted(expanded_by_area.items())
    ]
    payload = {
        "schema_version": 2,
        "report_id": f"release-{commit[:12]}-{profile_digest[:12]}",
        "source": {
            "commit": commit,
            "clean": True,
            "unresolved": False,
            "submodules": submodules,
        },
        "profile": {
            "name": "release",
            "manifest_sha256": profile_digest,
            "expanded_selected_areas": expanded,
        },
        "command": [
            "scripts/run_all_tests.sh",
            "--profile",
            "release",
            "--release-report-out",
            str(output_path),
        ],
        "toolchain": toolchain,
        "overall_status": "pass",
        "steps": step_payloads,
        "result_artifacts": result_artifacts,
    }
    return payload


def build_steps(log_path: Path) -> list[dict[str, Any]]:
    parsed = reports.parse_log(log_path)
    raw_steps = parsed.get("lane_steps")
    if not isinstance(raw_steps, list) or not raw_steps:
        raise GovernanceError("release profile log contains no lane-step evidence")
    steps: list[dict[str, Any]] = []
    names: set[str] = set()
    for raw in raw_steps:
        if not isinstance(raw, dict):
            raise GovernanceError("release profile log contains invalid lane-step evidence")
        name = str(raw.get("name", ""))
        status = str(raw.get("status", ""))
        elapsed_ms = raw.get("elapsed_ms")
        if not name or name in names or status != "pass" or type(elapsed_ms) is not int:
            raise GovernanceError(f"invalid or duplicate release lane step: {raw}")
        names.add(name)
        steps.append(
            {
                "name": name,
                "status": "pass",
                "elapsed_ms": elapsed_ms,
                "suite_results": [],
            }
        )
    return steps


def collect_critical_results(
    *,
    result_root: Path | None = None,
    artifact_root: Path = REPO_ROOT,
) -> tuple[list[dict[str, str]], dict[str, list[dict[str, Any]]]]:
    if result_root is None:
        result_root = REPO_ROOT / "target" / "verification" / "areas"
    artifacts: list[dict[str, str]] = []
    suite_results: dict[str, list[dict[str, Any]]] = {}
    for area, filename in CRITICAL_RESULTS.items():
        path = result_root / filename
        if not path.is_file():
            raise GovernanceError(f"release profile emitted no critical area result: {area}")
        payload = json.loads(path.read_text(encoding="utf-8"))
        if payload.get("area") != area:
            raise GovernanceError(f"critical area result identity mismatch: {path}")
        digest = sha256_file(path)
        artifacts.append(
            {
                "path": str(path.relative_to(artifact_root)),
                "sha256": digest,
            }
        )
        entries: list[dict[str, Any]] = []
        for suite in payload.get("suites", []):
            suite_name = str(suite.get("name", ""))
            case_ids = case_ids_for_suite(suite)
            if area == "developer_tooling" and suite_name == "full":
                editor_cases = [case_id for case_id in case_ids if case_id.startswith("editor-release:")]
                if not editor_cases:
                    raise GovernanceError("developer_tooling:full omitted editor-release case evidence")
                non_editor_cases = [
                    case_id for case_id in case_ids if not case_id.startswith("editor-release:")
                ]
                entries.append(suite_evidence(area, suite_name, non_editor_cases, digest))
                entries.append(suite_evidence(area, "editor-release", editor_cases, digest))
            else:
                entries.append(suite_evidence(area, suite_name, case_ids, digest))
        suite_results[area] = entries
    return sorted(artifacts, key=lambda item: item["path"]), suite_results


def suite_evidence(
    area: str,
    suite: str,
    case_ids: list[str],
    digest: str,
) -> dict[str, Any]:
    if not suite or not case_ids:
        raise GovernanceError(f"{area} suite emitted no case evidence: {suite}")
    return {
        "area": area,
        "suite": suite,
        "status": "pass",
        "case_ids": case_ids,
        "result_artifact_sha256": digest,
    }


def case_ids_for_suite(suite: Any) -> list[str]:
    ids: list[str] = []
    for case in suite.get("cases", []):
        for variant in case.get("variants", []):
            label = variant.get("label")
            if isinstance(label, str) and label:
                ids.append(label)
    if len(ids) != len(set(ids)):
        raise GovernanceError(f"suite result contains duplicate case evidence: {ids}")
    return ids


def expand_suites(area: str, suites: list[str]) -> list[str]:
    expanded = list(suites)
    if area == "developer_tooling" and "full" in expanded:
        expanded.append("editor-release")
    return sorted(set(expanded))


def command_version(*command: str) -> str:
    result = subprocess.run(
        list(command),
        cwd=REPO_ROOT,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    if result.returncode != 0 or not result.stdout.strip():
        raise GovernanceError(f"could not resolve toolchain identity: {' '.join(command)}")
    return result.stdout.strip().splitlines()[0]


def git_output(*args: str) -> str:
    result = subprocess.run(
        ["git", *args],
        cwd=REPO_ROOT,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode != 0:
        raise GovernanceError(f"git {' '.join(args)} failed: {result.stderr.strip()}")
    return result.stdout.strip()
