#!/usr/bin/env python3

from __future__ import annotations

import json
import subprocess
import time
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parent.parent
AUDIT_LEETCODE_DIR = REPO_ROOT / "audits" / "leetcode" / "src"
REQUIRED_TOPICS = (
    "arrays",
    "strings",
    "hash_map",
    "dp",
    "graphs",
    "trees",
    "backtracking",
    "math",
    "heap_priority_queue",
    "two_pointers_sliding_window",
)
VALID_DIFFICULTIES = ("easy", "medium", "hard")
VALID_SCOPE_CLASSIFICATIONS = (
    "in_scope",
    "blocked_feature",
    "out_of_scope_external_dep",
)
VALID_ORACLE_MODES = ("embedded_asserts", "no_oracle")


def repo_relative(path: Path) -> str:
    return path.relative_to(REPO_ROOT).as_posix()


def load_json(path: Path) -> Any:
    return json.loads(path.read_text())


def write_json(path: Path, payload: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")


def fixture_paths() -> list[Path]:
    return sorted(AUDIT_LEETCODE_DIR.glob("*.sifr"))


def detect_oracle_mode(source_text: str) -> str:
    if "assert " in source_text:
        return "embedded_asserts"
    if "def main():" in source_text:
        return "no_oracle"
    raise ValueError("unsupported LeetCode fixture oracle shape")


def build_inventory(seed_case_ids: set[str]) -> list[dict[str, Any]]:
    inventory: list[dict[str, Any]] = []
    for path in fixture_paths():
        stem = path.stem
        problem_id = stem[:4]
        python_source = path.with_suffix(".py")
        oracle_mode = detect_oracle_mode(path.read_text())
        inventory.append(
            {
                "id": problem_id,
                "fixture_slug": stem,
                "sifr_path": repo_relative(path),
                "python_source_path": (
                    repo_relative(python_source) if python_source.exists() else None
                ),
                "oracle_mode": oracle_mode,
                "seed_member": problem_id in seed_case_ids,
            }
        )
    return inventory


def inventory_summary(inventory: list[dict[str, Any]]) -> dict[str, Any]:
    oracle_counts: dict[str, int] = {mode: 0 for mode in VALID_ORACLE_MODES}
    for item in inventory:
        oracle_counts[item["oracle_mode"]] += 1
    return {
        "total_fixtures": len(inventory),
        "oracle_mode_counts": oracle_counts,
        "python_source_pairs": sum(1 for item in inventory if item["python_source_path"]),
        "seed_members": sum(1 for item in inventory if item["seed_member"]),
    }


def validate_seed_manifest(seed_manifest: list[dict[str, Any]]) -> dict[str, Any]:
    if len(seed_manifest) < 50:
        raise ValueError("phase-31 seed corpus must contain at least 50 problems")

    ids = [entry["id"] for entry in seed_manifest]
    if len(ids) != len(set(ids)):
        raise ValueError("phase-31 seed corpus contains duplicate problem ids")

    topic_counts = {topic: 0 for topic in REQUIRED_TOPICS}
    difficulty_counts = {difficulty: 0 for difficulty in VALID_DIFFICULTIES}
    oracle_counts = {mode: 0 for mode in VALID_ORACLE_MODES}

    for entry in seed_manifest:
        if entry["primary_topic"] not in REQUIRED_TOPICS:
            raise ValueError(f"unsupported primary topic: {entry['primary_topic']}")
        if entry["difficulty"] not in VALID_DIFFICULTIES:
            raise ValueError(f"unsupported difficulty: {entry['difficulty']}")
        if entry["scope_classification"] not in VALID_SCOPE_CLASSIFICATIONS:
            raise ValueError(
                f"unsupported scope classification: {entry['scope_classification']}"
            )
        oracle_mode = entry["oracle"]["mode"]
        if oracle_mode not in VALID_ORACLE_MODES:
            raise ValueError(f"unsupported oracle mode: {oracle_mode}")
        fixture_path = REPO_ROOT / entry["sifr_path"]
        if not fixture_path.exists():
            raise ValueError(f"missing fixture: {entry['sifr_path']}")
        detected_oracle = detect_oracle_mode(fixture_path.read_text())
        if detected_oracle != oracle_mode:
            raise ValueError(
                f"oracle mode mismatch for {entry['sifr_path']}: "
                f"{oracle_mode} != {detected_oracle}"
            )

        topic_counts[entry["primary_topic"]] += 1
        difficulty_counts[entry["difficulty"]] += 1
        oracle_counts[oracle_mode] += 1

    missing_topics = [topic for topic, count in topic_counts.items() if count == 0]
    if missing_topics:
        raise ValueError(f"seed corpus is missing required topics: {missing_topics}")

    if any(count == 0 for count in difficulty_counts.values()):
        raise ValueError("seed corpus must include easy, medium, and hard problems")

    return {
        "seed_problem_count": len(seed_manifest),
        "topic_counts": topic_counts,
        "difficulty_counts": difficulty_counts,
        "oracle_mode_counts": oracle_counts,
    }


def load_seed_manifest(path: Path) -> list[dict[str, Any]]:
    payload = load_json(path)
    if not isinstance(payload, dict) or "cases" not in payload:
        raise ValueError("expected manifest JSON object with a top-level 'cases' field")
    cases = payload["cases"]
    if not isinstance(cases, list):
        raise ValueError("manifest 'cases' field must be a list")
    return cases


def truncate_output(text: str, limit: int = 4000) -> str:
    stripped = text.strip()
    if len(stripped) <= limit:
        return stripped
    return stripped[:limit] + "...<truncated>"


def execute_stage(
    command_prefix: list[str], stage: str, fixture_path: str, timeout_seconds: int
) -> dict[str, Any]:
    started_at = time.perf_counter()
    command = [*command_prefix, stage, fixture_path]
    try:
        completed = subprocess.run(
            command,
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
            timeout=timeout_seconds,
            check=False,
        )
    except subprocess.TimeoutExpired as exc:
        duration_ms = int((time.perf_counter() - started_at) * 1000)
        return {
            "stage": stage,
            "command": command,
            "duration_ms": duration_ms,
            "timed_out": True,
            "returncode": None,
            "stdout": truncate_output(exc.stdout or ""),
            "stderr": truncate_output(exc.stderr or ""),
        }

    duration_ms = int((time.perf_counter() - started_at) * 1000)
    return {
        "stage": stage,
        "command": command,
        "duration_ms": duration_ms,
        "timed_out": False,
        "returncode": completed.returncode,
        "stdout": truncate_output(completed.stdout),
        "stderr": truncate_output(completed.stderr),
    }


def run_case(
    case: dict[str, Any], command_prefix: list[str], default_timeout_seconds: int
) -> dict[str, Any]:
    fixture_path = case["sifr_path"]
    timeout_seconds = int(case.get("timeout_seconds", default_timeout_seconds))
    check_result = execute_stage(command_prefix, "check", fixture_path, timeout_seconds)
    result: dict[str, Any] = {
        "id": case["id"],
        "fixture_slug": case["fixture_slug"],
        "primary_topic": case["primary_topic"],
        "difficulty": case["difficulty"],
        "scope_classification": case["scope_classification"],
        "oracle_mode": case["oracle"]["mode"],
        "timeout_seconds": timeout_seconds,
        "stages": [check_result],
    }
    if check_result["timed_out"]:
        result["status"] = "TIMEOUT"
        result["failure_stage"] = "check"
        return result
    if check_result["returncode"] != 0:
        result["status"] = "CHECK_ERROR"
        result["failure_stage"] = "check"
        return result

    run_result = execute_stage(command_prefix, "run", fixture_path, timeout_seconds)
    result["stages"].append(run_result)
    if run_result["timed_out"]:
        result["status"] = "TIMEOUT"
        result["failure_stage"] = "run"
        return result
    if run_result["returncode"] != 0:
        result["status"] = "RUN_ERROR"
        result["failure_stage"] = "run"
        return result

    result["failure_stage"] = None
    result["status"] = (
        "PASS" if case["oracle"]["mode"] == "embedded_asserts" else "NO_ORACLE"
    )
    return result


def build_runner_summary(results: list[dict[str, Any]]) -> dict[str, Any]:
    status_counts: dict[str, int] = {}
    topic_counts: dict[str, int] = {topic: 0 for topic in REQUIRED_TOPICS}
    difficulty_counts: dict[str, int] = {difficulty: 0 for difficulty in VALID_DIFFICULTIES}
    scope_counts: dict[str, int] = {
        scope: 0 for scope in VALID_SCOPE_CLASSIFICATIONS
    }

    for result in results:
        status_counts[result["status"]] = status_counts.get(result["status"], 0) + 1
        topic_counts[result["primary_topic"]] += 1
        difficulty_counts[result["difficulty"]] += 1
        scope_counts[result["scope_classification"]] += 1

    return {
        "case_count": len(results),
        "status_counts": status_counts,
        "topic_counts": topic_counts,
        "difficulty_counts": difficulty_counts,
        "scope_counts": scope_counts,
    }
