"""Explicit named reference selection and approved, atomic baseline capture."""

from __future__ import annotations

import copy
import hashlib
import json
import os
import re
import subprocess
from pathlib import Path
from typing import Any

from reference_host import comparison_mismatches

REFERENCE_ROOT = Path(__file__).resolve().parent / "data" / "references"


class ReferenceProfileError(Exception):
    pass


def validate_name(name: str) -> str:
    if not re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*", name):
        raise ReferenceProfileError("reference profile must be a lowercase hyphenated name")
    return name


def profile_path(name: str, root: Path = REFERENCE_ROOT) -> Path:
    return root / f"{validate_name(name)}.json"


def load_profile(name: str, root: Path = REFERENCE_ROOT) -> dict[str, Any]:
    path = profile_path(name, root)
    try:
        profile = json.loads(path.read_text())
    except (OSError, ValueError) as error:
        raise ReferenceProfileError(f"reference profile {name} is unavailable: {error}") from error
    if profile.get("schema_version") != 1 or profile.get("name") != name:
        raise ReferenceProfileError(f"invalid reference profile {name}")
    for key in ("identity", "baseline", "budgets"):
        if not isinstance(profile.get(key), dict):
            raise ReferenceProfileError(f"reference profile {name} is missing {key}")
    receipt = profile["baseline"].get("reference_capture", {})
    if not receipt.get("controlled_host") or receipt.get("approval_owner") != "compiler/performance":
        raise ReferenceProfileError("reference profile lacks approved controlled capture")
    return profile


def assert_comparable(profile: dict[str, Any], identity: dict[str, Any]) -> None:
    try:
        mismatches = comparison_mismatches(profile["identity"], identity)
    except (KeyError, TypeError) as error:
        raise ReferenceProfileError("reference identity is incomplete") from error
    if mismatches:
        raise ReferenceProfileError(
            f"reference profile {profile['name']} is not comparable: " + ", ".join(mismatches)
        )


def validate_result_profile(profile: dict[str, Any], report: dict[str, Any]) -> None:
    metadata = report.get("metadata", {})
    if metadata.get("reference_profile") != profile["name"]:
        raise ReferenceProfileError("result reference profile does not match selected profile")
    assert_comparable(profile, metadata.get("reference_identity", {}))
    if metadata.get("source_dirty") is not False:
        raise ReferenceProfileError("named qualification requires a clean producer worktree")
    baseline = profile["baseline"]
    if report.get("runner_version") != baseline.get("runner_version"):
        raise ReferenceProfileError("result and reference runner versions differ")
    if metadata.get("host_control", {}).get("status") != "controlled":
        raise ReferenceProfileError("named-profile qualification requires controlled-host evidence")
    if metadata.get("reference_profile_sha256") != profile_digest(profile):
        raise ReferenceProfileError("selected reference changed since benchmark production")


def profile_digest(profile: dict[str, Any]) -> str:
    return hashlib.sha256(
        json.dumps(profile, sort_keys=True, separators=(",", ":")).encode()
    ).hexdigest()


def derive_budgets(template: dict[str, Any], baseline: dict[str, Any]) -> dict[str, Any]:
    budgets = copy.deepcopy(template)
    results = {row["id"]: row for row in baseline["results"]}
    if set(results) != {row["benchmark_id"] for row in budgets["budgets"]}:
        raise ReferenceProfileError("reference capture must cover the complete budget corpus")
    for entry in budgets["budgets"]:
        result = results[entry["benchmark_id"]]
        metrics = result["metrics"]
        median, p95 = metrics["median_ms"], metrics["p95_ms"]
        rss = metrics["peak_rss_bytes"]
        if rss is None:
            raise ReferenceProfileError(f"{entry['benchmark_id']}: reference RSS is unavailable")
        thresholds = entry["thresholds"]
        policy = entry["policy"]
        if policy == "command-default":
            thresholds["median_ms"] = max(median * 1.10, median + 25)
            thresholds["p95_ms"] = max(p95 * 1.15, p95 + 50)
        elif policy == "frontend-query-edit-loop":
            thresholds["median_ms"] = max(median * 1.05, median + 2)
            thresholds["p95_ms"] = max(p95 * 1.10, p95 + 5)
            entry["cache"] = {"min_hits": result["cache"]["hits"]} if result["cache"]["hits"] else {}
        elif policy == "lsp-query":
            # Preserve the currently enforced editor-latency ceilings. A host
            # baseline capture does not authorize relaxing editor response SLOs.
            if entry["benchmark_id"] != "lsp-query-001-request-families":
                thresholds["median_ms"] = min(thresholds["median_ms"], max(median * 3, median + 5))
                thresholds["p95_ms"] = min(thresholds["p95_ms"], max(p95 * 4, p95 + 10))
        else:
            raise ReferenceProfileError(f"unknown budget derivation policy {policy}")
        thresholds["peak_rss_bytes"] = max(rss * 1.10, rss + 32 * 1024 * 1024)
        for key in ("median_ms", "p95_ms", "peak_rss_bytes"):
            thresholds[key] = round(thresholds[key], 3)
        entry["baseline"] = {
            key: metrics[key]
            for key in ("median_ms", "p95_ms", "peak_rss_bytes", "coefficient_variation")
        }
        entry["rationale"] = "Derived from this named reference using the shared regression rules."
    budgets["description"] = "Named-host regression budgets; existing editor latency ceilings and timeouts preserved."
    return budgets


def capture_profile(
    name: str, baseline: dict[str, Any], template: dict[str, Any],
    *, root: Path = REFERENCE_ROOT,
) -> Path:
    path = profile_path(name, root)
    if path.exists():
        raise ReferenceProfileError("reference profile already exists; choose an explicitly versioned name")
    identity = baseline.get("metadata", {}).get("reference_identity")
    if not isinstance(identity, dict):
        raise ReferenceProfileError("capture is missing measured reference identity")
    if identity["execution"]["cargo_jobs"] == "cargo-default":
        raise ReferenceProfileError("reference capture requires explicit CARGO_BUILD_JOBS")
    receipt = baseline.get("reference_capture", {})
    if receipt.get("approval_owner") != "compiler/performance" or not receipt.get("controlled_host"):
        raise ReferenceProfileError("capture requires approved controlled reference evidence")
    profile = {
        "schema_version": 1,
        "name": name,
        "identity": identity,
        "baseline": baseline,
        "budgets": derive_budgets(template, baseline),
    }
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(f".{path.name}.{os.getpid()}.tmp")
    temporary.write_text(json.dumps(profile, indent=2, sort_keys=True) + "\n")
    try:
        # Publish complete bytes without ever replacing an existing reference.
        os.link(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)
    return path


def validate_compiler_reference(repo_root: Path, commit: str) -> str:
    if not re.fullmatch(r"[0-9a-f]{40}", commit):
        raise ReferenceProfileError("named capture requires --reference-compiler-commit with a full merged SHA")
    merged = subprocess.run(
        ["git", "merge-base", "--is-ancestor", commit, "origin/main"],
        cwd=repo_root, capture_output=True, text=True, check=False, timeout=30,
    )
    if merged.returncode != 0:
        raise ReferenceProfileError("reference compiler must be an ancestor of origin/main")
    changed = subprocess.run(
        ["git", "diff", "--name-only", commit, "HEAD"],
        cwd=repo_root, capture_output=True, text=True, check=True, timeout=30,
    ).stdout.splitlines()
    forbidden = [
        path for path in changed
        if not path.startswith("verification/areas/performance/")
        and path != "internal_docs/performance_budgets.md"
    ]
    if forbidden:
        raise ReferenceProfileError(
            "reference worktree changes compiler inputs outside benchmark tooling: "
            + ", ".join(forbidden)
        )
    return commit
