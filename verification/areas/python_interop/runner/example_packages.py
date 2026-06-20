from __future__ import annotations

import shutil
import subprocess
import time
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from env import RunnerPaths

EXAMPLE_TIMEOUT_SECONDS = 600


@dataclass(frozen=True)
class ExampleCase:
    case_id: str
    relative_source: str
    stdout_marker: str
    import_roots: tuple[str, ...]


def build_examples_report(
    paths: RunnerPaths,
    *,
    suite_name: str,
    cases_by_id: dict[str, ExampleCase],
    example_runner: Callable[[RunnerPaths], list[dict[str, Any]]] | None = None,
) -> dict[str, Any]:
    source_checks = validate_source_presence(paths, cases_by_id)
    missing_sources = [check for check in source_checks if check["status"] != "pass"]
    if missing_sources:
        return _report(
            suite_name=suite_name,
            status="examples-failed",
            source_checks=source_checks,
            cases=[],
            failures=len(missing_sources),
            cases_by_id=cases_by_id,
        )

    runner = example_runner or (
        lambda runner_paths: run_example_cases(runner_paths, suite_name, cases_by_id)
    )
    case_results = runner(paths)
    failures = sum(1 for case in case_results if case["status"] != "example-passed")
    return _report(
        suite_name=suite_name,
        status="examples-failed" if failures else "examples-passed",
        source_checks=source_checks,
        cases=case_results,
        failures=failures,
        cases_by_id=cases_by_id,
    )


def run_examples_self_tests(
    paths: RunnerPaths,
    *,
    suite_name: str,
    cases_by_id: dict[str, ExampleCase],
) -> None:
    skipped_payload = build_examples_report(
        paths,
        suite_name=suite_name,
        cases_by_id=cases_by_id,
        example_runner=lambda _paths: [
            {
                "id": case.case_id,
                "status": "example-passed",
                "sifr_source": case.relative_source,
                "elapsed_ms": 0,
            }
            for case in cases_by_id.values()
        ],
    )
    if skipped_payload["status"] != "examples-passed":
        raise SystemExit(f"{suite_name} examples self-test expected examples-passed")
    case_ids = {case["id"] for case in skipped_payload["cases"]}
    if case_ids != set(cases_by_id):
        raise SystemExit(f"{suite_name} examples self-test case drift: {sorted(case_ids)}")
    source_ids = {check["id"] for check in skipped_payload["source_checks"]}
    if source_ids != set(cases_by_id):
        raise SystemExit(f"{suite_name} examples self-test source drift: {sorted(source_ids)}")
    marker_ids = {case.case_id for case in cases_by_id.values() if case.stdout_marker}
    if marker_ids != set(cases_by_id):
        raise SystemExit(f"{suite_name} examples self-test marker drift: {sorted(marker_ids)}")
    roots_ids = {case.case_id for case in cases_by_id.values() if case.import_roots}
    if roots_ids != set(cases_by_id):
        raise SystemExit(f"{suite_name} examples self-test trust-root drift: {sorted(roots_ids)}")

    first_case = next(iter(cases_by_id.values()))
    failed_payload = build_examples_report(
        paths,
        suite_name=suite_name,
        cases_by_id=cases_by_id,
        example_runner=lambda _paths: [
            {
                "id": first_case.case_id,
                "status": "example-failed",
                "sifr_source": first_case.relative_source,
                "error": "synthetic example failure",
            }
        ],
    )
    if failed_payload["status"] != "examples-failed":
        raise SystemExit(f"{suite_name} examples self-test expected examples-failed")
    if failed_payload["summary"]["total_failures"] != 1:
        raise SystemExit(f"{suite_name} examples self-test expected one failure")


def validate_source_presence(
    paths: RunnerPaths,
    cases_by_id: dict[str, ExampleCase],
) -> list[dict[str, Any]]:
    checks: list[dict[str, Any]] = []
    for case in cases_by_id.values():
        source_path = paths.fixtures_root / case.relative_source
        if not source_path.is_file():
            checks.append(
                {
                    "id": case.case_id,
                    "status": "fail",
                    "sifr_source": case.relative_source,
                    "reason": "missing source fixture",
                }
            )
            continue
        checks.append(
            {
                "id": case.case_id,
                "status": "pass",
                "sifr_source": case.relative_source,
                "check": "source-present",
            }
        )
    return checks


def run_example_cases(
    paths: RunnerPaths,
    suite_name: str,
    cases_by_id: dict[str, ExampleCase],
) -> list[dict[str, Any]]:
    cases: list[dict[str, Any]] = []
    for case in cases_by_id.values():
        package_root = prepare_example_package(paths, suite_name, case)
        cases.append(_run_case(paths, package_root, case))
    return cases


def prepare_example_package(paths: RunnerPaths, suite_name: str, case: ExampleCase) -> Path:
    package_root = (
        paths.repo_root
        / "target"
        / "verification"
        / "areas"
        / "python_interop"
        / f"{suite_name}_examples_package"
        / case.case_id
    )
    if package_root.exists():
        shutil.rmtree(package_root)
    source_root = package_root / "src"
    source_root.mkdir(parents=True, exist_ok=True)
    source_path = paths.fixtures_root / case.relative_source
    if source_path.is_file():
        shutil.copy2(source_path, source_root / "main.sifr")
    (source_root / "lib.rs").write_text(
        "// Cargo package marker required for metadata discovery; runnable Sifr source is src/main.sifr.\n",
        encoding="utf-8",
    )
    (package_root / "Cargo.toml").write_text(
        "\n".join(
            [
                "[package]",
                f'name = "sifr-python-interop-{suite_name}-examples-{case.case_id}"',
                'version = "0.1.0"',
                'edition = "2024"',
                "",
                "[package.metadata.sifr]",
                'manifest = "sifr.toml"',
                "",
                "[workspace]",
                "",
            ]
        ),
        encoding="utf-8",
    )
    roots = ", ".join(f'"{root}"' for root in case.import_roots)
    (package_root / "sifr.toml").write_text(
        "\n".join(
            [
                "[package]",
                f'name = "python_interop_{suite_name}_examples"',
                'edition = "2026"',
                'sifr-version = ">=0.3,<0.4"',
                "",
                "[source]",
                'root = "src"',
                "",
                "[python]",
                'venv = ".venv"',
                f"allow-imports = [{roots}]",
                "",
                "[trust]",
                f"python = [{roots}]",
                f"python-native = [{roots}]",
                "",
            ]
        ),
        encoding="utf-8",
    )
    venv_link = package_root / ".venv"
    area_venv = paths.area_root / ".venv"
    if not area_venv.exists():
        raise SystemExit(
            f"python interop {suite_name} examples require the area uv environment; "
            "run through `uv run --project verification/areas/python_interop --locked ...`"
        )
    venv_link.symlink_to(area_venv, target_is_directory=True)
    return package_root


def _run_case(paths: RunnerPaths, package_root: Path, case_config: ExampleCase) -> dict[str, Any]:
    started = time.perf_counter()
    try:
        proc = subprocess.run(
            [
                "cargo",
                "run",
                "-q",
                "-p",
                "sifr",
                "--manifest-path",
                str(paths.repo_root / "Cargo.toml"),
                "--",
                "run",
            ],
            cwd=package_root,
            text=True,
            capture_output=True,
            check=False,
            timeout=EXAMPLE_TIMEOUT_SECONDS,
        )
    except subprocess.TimeoutExpired as error:
        elapsed_ms = round((time.perf_counter() - started) * 1000.0)
        return {
            "id": case_config.case_id,
            "status": "example-failed",
            "sifr_source": case_config.relative_source,
            "elapsed_ms": elapsed_ms,
            "reason": "timeout",
            "timeout_seconds": EXAMPLE_TIMEOUT_SECONDS,
            "stdout": (error.stdout or "")[-4000:],
            "stderr": (error.stderr or "")[-4000:],
        }
    elapsed_ms = round((time.perf_counter() - started) * 1000.0)
    marker_observed = case_config.stdout_marker in proc.stdout
    status = "example-passed" if proc.returncode == 0 and marker_observed else "example-failed"
    case: dict[str, Any] = {
        "id": case_config.case_id,
        "status": status,
        "sifr_source": case_config.relative_source,
        "elapsed_ms": elapsed_ms,
        "stdout_marker": case_config.stdout_marker,
        "stdout_marker_observed": marker_observed,
        "trusted_import_roots": list(case_config.import_roots),
        "stdout": proc.stdout[-4000:],
    }
    if status != "example-passed":
        case["stderr"] = proc.stderr[-4000:]
        case["exit_code"] = proc.returncode
        if proc.returncode == 0 and not marker_observed:
            case["reason"] = "missing stdout marker"
    return case


def _report(
    *,
    suite_name: str,
    status: str,
    source_checks: list[dict[str, Any]],
    cases: list[dict[str, Any]],
    failures: int,
    cases_by_id: dict[str, ExampleCase],
) -> dict[str, Any]:
    return {
        "schema_version": 1,
        "area": "python_interop",
        "suite": f"{suite_name}-examples",
        "status": status,
        "result_statuses": ["example-passed", "example-failed"],
        "dependencies": sorted({root for case in cases_by_id.values() for root in case.import_roots}),
        "source_checks": source_checks,
        "cases": cases,
        "summary": {
            "total_variants": len(source_checks) + len(cases),
            "total_failures": failures,
            "blocking_failures": failures,
            "non_blocking_failures": 0,
            "skipped": 0,
        },
    }
