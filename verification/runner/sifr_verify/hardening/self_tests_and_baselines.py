from __future__ import annotations

import argparse
import re
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from typing import Any

from .core import (
    assert_self_test_failure,
    baseline_artifact_paths,
    baseline_case_metadata,
    baseline_variant_label,
    canonicalize_output,
    load_text,
    run_captured_command,
    run_variant,
    validate_unique_baseline_artifact_paths,
    write_text,
)
from .coverage_fuzz import classify_build_failure, output_tail

def run_self_tests() -> int:
    expected_build_classes = {
        classify_build_failure(127, "cargo not found", False): "missing tool",
        classify_build_failure(101, "offline mode prevented a download", False): "offline dependency",
        classify_build_failure(101, "rustc failed", False): "instrumented build",
        classify_build_failure(124, "", True): "build timeout",
    }
    if set(expected_build_classes) != {
        "missing-fuzz-tool",
        "offline-dependency-failure",
        "instrumented-build-failure",
        "instrumented-build-timeout",
    }:
        raise AssertionError(f"fuzz build failure classes collapsed: {expected_build_classes}")
    if output_tail("a" * 20_000).encode() != b"a" * (16 * 1024):
        raise AssertionError("fuzz output tail is not bounded")

    repo_root = Path("/tmp/sifr-verification-hardening-self-test").resolve()
    validate_unique_baseline_artifact_paths(
        suite_name="self-test",
        repo_root=repo_root,
        cases=[
            {
                "id": "a",
                "entry": "fixtures/a/main.sifr",
                "command": "check",
                "diagnostic_formats": ["human", "json"],
            },
            {
                "id": "b",
                "entry": "fixtures/b/main.sifr",
                "command": "check",
                "diagnostic_formats": ["human", "json"],
            },
        ],
    )
    assert_self_test_failure(
        "normalized duplicate baseline artifact paths",
        "fixtures/a/baselines/check-json.stdout.txt",
        lambda: validate_unique_baseline_artifact_paths(
            suite_name="self-test",
            repo_root=repo_root,
            cases=[
                {
                    "id": "canonical",
                    "entry": "fixtures/a/main.sifr",
                    "command": "check",
                    "diagnostic_formats": ["json"],
                },
                {
                    "id": "prefixed",
                    "entry": "./fixtures/a/main.sifr",
                    "command": "check",
                    "diagnostic_formats": ["json"],
                },
            ],
        ),
    )
    with tempfile.TemporaryDirectory(prefix="sifr-hardening-deadline-") as tmp:
        deadline_root = Path(tmp)
        marker = deadline_root / "child-survived"
        child = (
            "import pathlib,time; time.sleep(0.4); "
            f"pathlib.Path({str(marker)!r}).write_text('survived')"
        )
        parent = (
            "import subprocess,sys,time; "
            f"subprocess.Popen([sys.executable, '-c', {child!r}]); time.sleep(5)"
        )
        exit_code, _, stderr = run_captured_command(
            args=[sys.executable, "-c", parent],
            cwd=deadline_root,
            timeout_secs=0.1,
        )
        if exit_code != 124 or "timed out" not in stderr:
            raise AssertionError("hardening command deadline did not report a timeout")
        time.sleep(0.5)
        if marker.exists():
            raise AssertionError("hardening command deadline left a descendant running")
    assert_self_test_failure(
        "duplicate diagnostic formats",
        "lists diagnostic_format 'json' more than once",
        lambda: validate_unique_baseline_artifact_paths(
            suite_name="self-test",
            repo_root=repo_root,
            cases=[
                {
                    "id": "duplicate-format",
                    "entry": "fixtures/c/main.sifr",
                    "command": "check",
                    "diagnostic_formats": ["json", "json"],
                }
            ],
        ),
    )
    assert_self_test_failure(
        "absolute baseline entry",
        "entry must be repo-relative",
        lambda: validate_unique_baseline_artifact_paths(
            suite_name="self-test",
            repo_root=repo_root,
            cases=[
                {
                    "id": "absolute",
                    "entry": "/tmp/main.sifr",
                    "command": "check",
                    "diagnostic_formats": ["json"],
                }
            ],
        ),
    )
    assert_self_test_failure(
        "repo-relative baseline entry escape",
        "entry must stay under repo root",
        lambda: validate_unique_baseline_artifact_paths(
            suite_name="self-test",
            repo_root=repo_root,
            cases=[
                {
                    "id": "escape",
                    "entry": "../escape/main.sifr",
                    "command": "check",
                    "diagnostic_formats": ["json"],
                }
            ],
        ),
    )
    with tempfile.TemporaryDirectory(prefix="sifr-hardening-history-") as tmp:
        history_root = Path(tmp)
        subprocess.run(["git", "init", "-q"], cwd=history_root, check=True)
        subprocess.run(
            ["git", "config", "user.email", "self-test@example.invalid"],
            cwd=history_root,
            check=True,
        )
        subprocess.run(
            ["git", "config", "user.name", "Self Test"],
            cwd=history_root,
            check=True,
        )
        old_project_root = history_root / "verification/oss/projects/example"
        old_project_root.mkdir(parents=True)
        (old_project_root / "main.sifr").write_text("def main() -> None:\n    pass\n")
        subprocess.run(["git", "add", "."], cwd=history_root, check=True)
        subprocess.run(["git", "commit", "-qm", "add example"], cwd=history_root, check=True)
        initial_revision = latest_project_revision(
            history_root,
            "verification/oss/projects/example",
        )
        assert initial_revision is not None
        new_project_root = history_root / "verification/areas/ecosystem_compatibility/projects/example"
        new_project_root.parent.mkdir(parents=True)
        subprocess.run(
            [
                "git",
                "mv",
                "verification/oss/projects/example",
                "verification/areas/ecosystem_compatibility/projects/example",
            ],
            cwd=history_root,
            check=True,
        )
        subprocess.run(["git", "commit", "-qm", "move example"], cwd=history_root, check=True)
        history = project_revision_history(
            history_root,
            "verification/areas/ecosystem_compatibility/projects/example",
        )
        assert initial_revision in history
        assert history[0] != initial_revision
    print("verification hardening self-tests ok")
    return 0


def run_git(repo_root: Path, args: list[str]) -> subprocess.CompletedProcess[str] | None:
    try:
        return subprocess.run(
            ["git", *args],
            cwd=repo_root,
            text=True,
            capture_output=True,
            check=False,
        )
    except OSError:
        return None


def project_tracked_paths(repo_root: Path, project_root: str) -> list[str]:
    proc = run_git(repo_root, ["ls-files", "--", project_root])
    if proc is None or proc.returncode != 0:
        return []
    paths = [line.strip() for line in proc.stdout.splitlines() if line.strip()]
    if paths:
        return paths

    fallback = run_git(repo_root, ["cat-file", "-e", f"HEAD:{project_root}"])
    if fallback is None or fallback.returncode != 0:
        return []
    return [project_root]


def project_revision_history(repo_root: Path, project_root: str) -> list[str]:
    revisions: list[str] = []
    seen: set[str] = set()
    for tracked_path in project_tracked_paths(repo_root, project_root):
        proc = run_git(
            repo_root,
            ["log", "--follow", "--find-renames", "--format=%H", "--", tracked_path],
        )
        if proc is None or proc.returncode != 0:
            continue
        for revision in proc.stdout.splitlines():
            revision = revision.strip()
            if revision in seen or re.fullmatch(r"[0-9a-f]{40}", revision) is None:
                continue
            revisions.append(revision)
            seen.add(revision)
    return revisions


def latest_project_revision(repo_root: Path, project_root: str) -> str | None:
    history = project_revision_history(repo_root, project_root)
    if not history:
        return None
    return history[0]


def baseline_case_result(
    *,
    suite_name: str,
    case: dict[str, Any],
    args: argparse.Namespace,
    repo_root: Path,
    actual_root: Path,
) -> tuple[dict[str, Any], bool, int]:
    case_id, entry_path, command_name, formats = baseline_case_metadata(
        suite_name=suite_name,
        case=case,
        repo_root=repo_root,
    )
    expected_exit = case.get("expect_exit_code")

    if not isinstance(expected_exit, int):
        raise SystemExit(f"suite '{suite_name}' case '{case_id}' missing integer 'expect_exit_code'")
    if not entry_path.is_file():
        raise SystemExit(f"suite '{suite_name}' case '{case_id}' entry does not exist: {entry_path}")

    case_failed = False
    failed_variants = 0
    case_result = {
        "id": case_id,
        "entry": str(entry_path.relative_to(repo_root)),
        "command": command_name,
        "variants": [],
    }

    for diagnostic_format in formats:
        label = baseline_variant_label(command_name, diagnostic_format)
        exit_code, stdout, stderr, elapsed_ms, argv = run_variant(
            repo_root=repo_root,
            command_name=command_name,
            entry=entry_path,
            diagnostic_format=diagnostic_format,
        )
        stdout_norm = canonicalize_output(
            repo_root=repo_root,
            text=stdout,
            diagnostic_format=diagnostic_format,
            stream="stdout",
        )
        stderr_norm = canonicalize_output(
            repo_root=repo_root,
            text=stderr,
            diagnostic_format=diagnostic_format,
            stream="stderr",
        )

        stdout_file, stderr_file, exit_file = baseline_artifact_paths(entry_path, label)

        mismatches: list[str] = []

        if args.bless:
            write_text(stdout_file, stdout_norm)
            write_text(stderr_file, stderr_norm)
            write_text(exit_file, f"{exit_code}\n")
        else:
            missing_files = [path for path in (stdout_file, stderr_file, exit_file) if not path.is_file()]
            if missing_files:
                mismatches.append(
                    "missing-baseline:"
                    + ",".join(str(path.relative_to(repo_root)) for path in missing_files)
                )
            else:
                expected_stdout = load_text(stdout_file)
                expected_stderr = load_text(stderr_file)
                expected_exit_raw = load_text(exit_file).strip()
                if stdout_norm != expected_stdout:
                    mismatches.append("stdout")
                if stderr_norm != expected_stderr:
                    mismatches.append("stderr")
                if str(exit_code) != expected_exit_raw:
                    mismatches.append("exit-code")

        if exit_code != expected_exit:
            mismatches.append("unexpected-exit")

        status = "pass" if not mismatches else "fail"
        if mismatches:
            case_failed = True
            failed_variants += 1

            actual_case_dir = actual_root / suite_name / case_id
            write_text(actual_case_dir / f"{label}.stdout.txt", stdout_norm)
            write_text(actual_case_dir / f"{label}.stderr.txt", stderr_norm)
            write_text(actual_case_dir / f"{label}.exit-code.txt", f"{exit_code}\n")

        case_result["variants"].append(
            {
                "label": label,
                "diagnostic_format": diagnostic_format,
                "argv": argv,
                "status": status,
                "mismatches": mismatches,
                "expected_exit_code": expected_exit,
                "actual_exit_code": exit_code,
                "duration_ms": round(elapsed_ms, 3),
                "baseline_stdout": str(stdout_file.relative_to(repo_root)),
                "baseline_stderr": str(stderr_file.relative_to(repo_root)),
                "baseline_exit_code": str(exit_file.relative_to(repo_root)),
            }
        )

    return case_result, case_failed, failed_variants


def run_baseline_suite(
    *,
    suite: dict[str, Any],
    args: argparse.Namespace,
    repo_root: Path,
    actual_root: Path,
) -> dict[str, Any]:
    suite_name = suite["name"]
    cases = suite.get("cases", [])
    if not isinstance(cases, list) or not cases:
        raise SystemExit(f"suite '{suite_name}' has no cases")
    validate_unique_baseline_artifact_paths(
        suite_name=suite_name,
        cases=cases,
        repo_root=repo_root,
    )
    print(f"  suite={suite_name} owner={suite.get('owner', 'unknown')} cases={len(cases)}")

    result = {
        "name": suite_name,
        "owner": suite.get("owner", "unknown"),
        "blocking": bool(suite.get("blocking", False)),
        "runner": "baseline",
        "cases": [],
        "failed_cases": 0,
        "total_variants": 0,
        "total_failures": 0,
    }

    for case in cases:
        case_result, case_failed, failed_variants = baseline_case_result(
            suite_name=suite_name,
            case=case,
            args=args,
            repo_root=repo_root,
            actual_root=actual_root,
        )
        result["cases"].append(case_result)
        result["total_variants"] += len(case_result["variants"])
        result["total_failures"] += failed_variants
        if case_failed:
            result["failed_cases"] += 1

    return result
