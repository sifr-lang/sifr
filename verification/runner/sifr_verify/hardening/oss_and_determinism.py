from __future__ import annotations

import hashlib
import json
import subprocess
import time
from pathlib import Path
from typing import Any

from .core import (
    LOCAL_PINNED_REVISION_PATTERN,
    load_index,
    normalize_string,
    required_missing,
    run_variant,
)
from .fixedbugs_and_crashes import contains_internal_panic
from .self_tests_and_baselines import project_revision_history

ALLOWED_SPDX_LICENSES = {"MIT"}


def run_oss_suite(
    *,
    suite: dict[str, Any],
    repo_root: Path,
    runner_name: str,
) -> dict[str, Any]:
    suite_name = suite["name"]
    index_raw = suite.get("index")
    if not isinstance(index_raw, str):
        raise SystemExit(f"suite '{suite_name}' missing string 'index'")
    index_path = repo_root / index_raw
    entries = load_index(index_path)
    if not entries:
        raise SystemExit(f"suite '{suite_name}' has empty index: {index_path}")
    print(f"  suite={suite_name} owner={suite.get('owner', 'unknown')} entries={len(entries)}")

    result = {
        "name": suite_name,
        "owner": suite.get("owner", "unknown"),
        "blocking": bool(suite.get("blocking", False)),
        "runner": runner_name,
        "index": str(index_path.relative_to(repo_root)),
        "cases": [],
        "failed_cases": 0,
        "total_variants": 0,
        "total_failures": 0,
    }

    allowed_classifications = {"pass", "known-failure", "investigate"}
    pinned_revision_cache: dict[str, list[str]] = {}

    for entry in entries:
        case_id = str(entry.get("id", "<missing-id>"))
        case_result = {
            "id": case_id,
            "project_root": entry.get("project_root"),
            "pinned_revision": entry.get("pinned_revision"),
            "source_checksum_sha256": entry.get("source_checksum_sha256"),
            "license": entry.get("license"),
            "expected_result_classification": entry.get("expected_result_classification"),
            "variants": [],
        }

        mismatches = required_missing(
            entry,
            (
                "id",
                "project_root",
                "pinned_revision",
                "source_checksum_sha256",
                "license",
                "owner",
                "rationale",
                "expected_result_classification",
            ),
        )
        classification = entry.get("expected_result_classification")
        if classification not in allowed_classifications:
            mismatches.append("expected_result_classification")

        project_root_raw = entry.get("project_root")
        pinned_revision_raw = entry.get("pinned_revision")
        commands = entry.get("commands")
        project_root = repo_root / str(project_root_raw) if isinstance(project_root_raw, str) else None
        if project_root is None or not project_root.is_dir():
            mismatches.append("project_root")
        expected_source_checksum = entry.get("source_checksum_sha256")
        if not is_sha256_hex(expected_source_checksum):
            mismatches.append("source_checksum_sha256")
        if entry.get("license") not in ALLOWED_SPDX_LICENSES:
            mismatches.append("license")
        if not isinstance(commands, list) or not commands:
            mismatches.append("commands")

        case_failed = False
        if mismatches:
            result["total_variants"] += 1
            result["total_failures"] += 1
            result["failed_cases"] += 1
            case_result["variants"].append(
                {
                    "label": "metadata",
                    "status": "fail",
                    "mismatches": sorted(set(mismatches)),
                }
            )
            result["cases"].append(case_result)
            continue

        assert project_root is not None
        assert isinstance(project_root_raw, str)
        assert isinstance(pinned_revision_raw, str)
        pinned_match = LOCAL_PINNED_REVISION_PATTERN.fullmatch(pinned_revision_raw)
        if pinned_match is None:
            result["total_variants"] += 1
            result["total_failures"] += 1
            result["failed_cases"] += 1
            case_result["variants"].append(
                {
                    "label": "pinned-revision",
                    "status": "fail",
                    "mismatches": ["pinned_revision_format"],
                }
            )
            result["cases"].append(case_result)
            continue
        expected_sha = pinned_match.group(1)
        revision_history = pinned_revision_cache.get(project_root_raw)
        if project_root_raw not in pinned_revision_cache:
            revision_history = project_revision_history(repo_root, project_root_raw)
            pinned_revision_cache[project_root_raw] = revision_history
        if not revision_history:
            result["total_variants"] += 1
            result["total_failures"] += 1
            result["failed_cases"] += 1
            case_result["variants"].append(
                {
                    "label": "pinned-revision",
                    "status": "fail",
                    "mismatches": ["pinned_revision_unresolvable"],
                }
            )
            result["cases"].append(case_result)
            continue
        latest_sha = revision_history[0]
        matched_sha = next((sha for sha in revision_history if sha.startswith(expected_sha)), None)
        if matched_sha is None:
            result["total_variants"] += 1
            result["total_failures"] += 1
            result["failed_cases"] += 1
            case_result["variants"].append(
                {
                    "label": "pinned-revision",
                    "status": "fail",
                    "mismatches": ["pinned_revision_mismatch"],
                    "expected_pinned_revision": pinned_revision_raw,
                    "latest_project_revision": f"local-main@{latest_sha[:len(expected_sha)]}",
                }
            )
            result["cases"].append(case_result)
            continue
        result["total_variants"] += 1
        case_result["variants"].append(
            {
                "label": "pinned-revision",
                "status": "pass",
                "mismatches": [],
                "expected_pinned_revision": pinned_revision_raw,
                "latest_project_revision": f"local-main@{latest_sha[:len(expected_sha)]}",
                "matched_project_revision": f"local-main@{matched_sha[:len(expected_sha)]}",
            }
        )

        actual_source_checksum = project_source_checksum(repo_root, project_root_raw)
        result["total_variants"] += 1
        if actual_source_checksum != expected_source_checksum:
            result["total_failures"] += 1
            result["failed_cases"] += 1
            case_result["variants"].append(
                {
                    "label": "source-checksum",
                    "status": "fail",
                    "mismatches": ["source_checksum_mismatch"],
                    "expected_source_checksum_sha256": expected_source_checksum,
                    "actual_source_checksum_sha256": actual_source_checksum,
                }
            )
            result["cases"].append(case_result)
            continue
        case_result["variants"].append(
            {
                "label": "source-checksum",
                "status": "pass",
                "mismatches": [],
                "expected_source_checksum_sha256": expected_source_checksum,
                "actual_source_checksum_sha256": actual_source_checksum,
            }
        )

        for idx, command_meta in enumerate(commands, start=1):
            if not isinstance(command_meta, dict):
                result["total_variants"] += 1
                result["total_failures"] += 1
                case_failed = True
                case_result["variants"].append(
                    {
                        "label": f"command-{idx}",
                        "status": "fail",
                        "mismatches": ["command-metadata"],
                    }
                )
                continue

            command_name = command_meta.get("command")
            entrypoint_raw = command_meta.get("entrypoint")
            expected_exit = command_meta.get("expect_exit_code")
            timeout_secs = command_meta.get("timeout_secs")

            command_mismatches: list[str] = []
            if command_name not in {"check", "run", "build", "test"}:
                command_mismatches.append("command")
            if not isinstance(entrypoint_raw, str) or not entrypoint_raw:
                command_mismatches.append("entrypoint")
            if not isinstance(expected_exit, int):
                command_mismatches.append("expect_exit_code")
            if not isinstance(timeout_secs, int) or timeout_secs < 1:
                command_mismatches.append("timeout_secs")

            entrypoint_path = project_root / str(entrypoint_raw) if isinstance(entrypoint_raw, str) else None
            if entrypoint_path is None or not entrypoint_path.is_file():
                command_mismatches.append("entrypoint")

            if command_mismatches:
                result["total_variants"] += 1
                result["total_failures"] += 1
                case_failed = True
                case_result["variants"].append(
                    {
                        "label": f"command-{idx}",
                        "status": "fail",
                        "mismatches": sorted(set(command_mismatches)),
                    }
                )
                continue

            assert entrypoint_path is not None
            assert isinstance(timeout_secs, int)
            exit_code, stdout, stderr, elapsed_ms, argv = run_variant(
                repo_root=repo_root,
                command_name=str(command_name),
                entry=entrypoint_path,
                diagnostic_format=None,
                timeout_secs=timeout_secs,
            )
            stdout_norm = normalize_string(stdout, repo_root)
            stderr_norm = normalize_string(stderr, repo_root)

            variant_mismatches: list[str] = []
            if exit_code != expected_exit:
                variant_mismatches.append("unexpected-exit")
            if contains_internal_panic(stdout_norm + stderr_norm):
                variant_mismatches.append("panic-signal")

            result["total_variants"] += 1
            status = "pass" if not variant_mismatches else "fail"
            if variant_mismatches:
                case_failed = True
                result["total_failures"] += 1

            case_result["variants"].append(
                {
                    "label": f"{command_name}-{idx}",
                    "status": status,
                    "mismatches": variant_mismatches,
                    "argv": argv,
                    "expected_exit_code": expected_exit,
                    "actual_exit_code": exit_code,
                    "duration_ms": round(elapsed_ms, 3),
                    "timeout_secs": timeout_secs,
                }
            )

        if case_failed:
            result["failed_cases"] += 1
        result["cases"].append(case_result)

    return result


def is_sha256_hex(value: object) -> bool:
    return isinstance(value, str) and len(value) == 64 and all(char in "0123456789abcdef" for char in value)


def project_source_checksum(repo_root: Path, project_root: str) -> str:
    proc = subprocess.run(
        ["git", "ls-files", "--", project_root],
        cwd=repo_root,
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        return ""
    digest = hashlib.sha256()
    tracked_paths = sorted(line.strip() for line in proc.stdout.splitlines() if line.strip())
    for relative in tracked_paths:
        path = repo_root / relative
        if not path.is_file():
            return ""
        digest.update(Path(relative).relative_to(project_root).as_posix().encode("utf-8"))
        digest.update(b"\0")
        digest.update(path.read_bytes())
        digest.update(b"\0")
    return digest.hexdigest()


def run_external_command(
    *,
    repo_root: Path,
    argv: list[str],
    timeout_secs: int | None,
) -> tuple[int, str, str, float]:
    started = time.perf_counter()
    try:
        proc = subprocess.run(
            argv,
            cwd=repo_root,
            text=True,
            capture_output=True,
            check=False,
            timeout=timeout_secs,
        )
        exit_code = proc.returncode
        stdout = proc.stdout
        stderr = proc.stderr
    except subprocess.TimeoutExpired as timeout_error:
        exit_code = 124
        stdout = timeout_error.stdout or ""
        stderr = (timeout_error.stderr or "") + f"\ncommand timed out after {timeout_secs} seconds"
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    return exit_code, stdout, stderr, elapsed_ms


def substitute_profile_tokens(command: list[str], profile: str) -> list[str]:
    return [profile if token == "<PROFILE>" else token for token in command]


def run_determinism_scale_suite(
    *,
    suite: dict[str, Any],
    repo_root: Path,
    profile: str,
) -> dict[str, Any]:
    suite_name = suite["name"]
    index_raw = suite.get("index")
    if not isinstance(index_raw, str):
        raise SystemExit(f"suite '{suite_name}' missing string 'index'")
    index_path = repo_root / index_raw
    entries = load_index(index_path)
    if not entries:
        raise SystemExit(f"suite '{suite_name}' has empty index: {index_path}")
    print(f"  suite={suite_name} owner={suite.get('owner', 'unknown')} entries={len(entries)}")

    result = {
        "name": suite_name,
        "owner": suite.get("owner", "unknown"),
        "blocking": bool(suite.get("blocking", False)),
        "runner": "determinism-scale",
        "index": str(index_path.relative_to(repo_root)),
        "cases": [],
        "failed_cases": 0,
        "total_variants": 0,
        "total_failures": 0,
    }

    for entry in entries:
        case_id = str(entry.get("id", "<missing-id>"))
        description = entry.get("description")
        command_raw = entry.get("command")
        expected_exit = entry.get("expect_exit_code")
        timeout_secs = entry.get("timeout_secs")
        mismatches = []

        if not isinstance(description, str) or not description:
            mismatches.append("description")
        if not isinstance(command_raw, list) or not command_raw or not all(
            isinstance(token, str) and token for token in command_raw
        ):
            mismatches.append("command")
        if not isinstance(expected_exit, int):
            mismatches.append("expect_exit_code")
        if not isinstance(timeout_secs, int) or timeout_secs < 1:
            mismatches.append("timeout_secs")

        case_result = {
            "id": case_id,
            "description": description,
            "variants": [],
        }
        case_failed = False

        if mismatches:
            result["total_variants"] += 1
            result["total_failures"] += 1
            result["failed_cases"] += 1
            case_result["variants"].append(
                {
                    "label": "metadata",
                    "status": "fail",
                    "mismatches": sorted(set(mismatches)),
                }
            )
            result["cases"].append(case_result)
            continue

        assert isinstance(command_raw, list)
        assert isinstance(timeout_secs, int)
        command = substitute_profile_tokens(command_raw, profile)
        exit_code, stdout, stderr, elapsed_ms = run_external_command(
            repo_root=repo_root,
            argv=command,
            timeout_secs=timeout_secs,
        )
        stdout_norm = normalize_string(stdout, repo_root)
        stderr_norm = normalize_string(stderr, repo_root)
        variant_mismatches: list[str] = []
        if exit_code != expected_exit:
            variant_mismatches.append("unexpected-exit")
        if contains_internal_panic(stdout_norm + stderr_norm):
            variant_mismatches.append("panic-signal")

        status = "pass" if not variant_mismatches else "fail"
        result["total_variants"] += 1
        if variant_mismatches:
            case_failed = True
            result["total_failures"] += 1
        case_result["variants"].append(
            {
                "label": "command",
                "status": status,
                "mismatches": variant_mismatches,
                "argv": command,
                "expected_exit_code": expected_exit,
                "actual_exit_code": exit_code,
                "duration_ms": round(elapsed_ms, 3),
                "timeout_secs": timeout_secs,
            }
        )

        result["cases"].append(case_result)
        if case_failed:
            result["failed_cases"] += 1

    return result


def deterministic_suite_shard(name: str, shard_total: int) -> int:
    if shard_total <= 1:
        return 0
    digest = hashlib.sha256(name.encode("utf-8")).hexdigest()
    return int(digest[:16], 16) % shard_total


def load_quarantine_metadata(path: Path, suites: list[dict[str, Any]]) -> list[dict[str, Any]]:
    if not path.is_file():
        raise SystemExit(f"quarantine file not found: {path}")
    payload = json.loads(path.read_text(encoding="utf-8"))
    entries = payload.get("entries", [])
    if not isinstance(entries, list):
        raise SystemExit(f"invalid quarantine file '{path}': 'entries' must be a list")
    suite_names = {suite.get("name") for suite in suites}
    validated: list[dict[str, Any]] = []
    for entry in entries:
        if not isinstance(entry, dict):
            raise SystemExit(f"invalid quarantine entry in '{path}': expected object")
        missing = required_missing(
            entry,
            (
                "suite",
                "case_id",
                "reason",
                "owner",
                "added_on",
                "reenable_criteria",
            ),
        )
        if missing:
            raise SystemExit(
                f"invalid quarantine entry in '{path}': missing fields {', '.join(sorted(set(missing)))}"
            )
        if entry.get("suite") not in suite_names:
            raise SystemExit(
                f"invalid quarantine entry in '{path}': unknown suite '{entry.get('suite')}'"
            )
        validated.append(entry)
    return validated


def failed_case_ids(suite_result: dict[str, Any]) -> set[str]:
    failed: set[str] = set()
    for case in suite_result.get("cases", []):
        if not isinstance(case, dict):
            continue
        case_id = case.get("id")
        variants = case.get("variants", [])
        if isinstance(case_id, str) and isinstance(variants, list):
            if any(isinstance(variant, dict) and variant.get("status") == "fail" for variant in variants):
                failed.add(case_id)
    return failed
