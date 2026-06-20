from __future__ import annotations

import shutil
import subprocess
import time
from collections.abc import Callable
from pathlib import Path
from typing import Any

from env import RunnerPaths

DATAFRAME_EXAMPLE_SOURCES = {
    "numpy": "numpy_buffer/numpy_full_example.sifr",
    "pandas": "pandas_arrow/pandas_full_example.sifr",
    "polars": "polars_arrow/polars_full_example.sifr",
}

DATAFRAME_EXAMPLE_MARKERS = {
    "numpy": "sifr-python-interop:numpy:sum=20:values=2,4,6,8",
    "pandas": "sifr-python-interop:pandas:double-total=20:values=2,3,5",
    "polars": "sifr-python-interop:polars:sum=10:first-city=oslo",
}

DATAFRAME_IMPORT_ROOTS = {
    "numpy": ("numpy",),
    "pandas": ("numpy", "pandas"),
    "polars": ("polars",),
}

EXAMPLE_TIMEOUT_SECONDS = 600


def build_dataframe_examples_report(
    paths: RunnerPaths,
    example_runner: Callable[[RunnerPaths], list[dict[str, Any]]] | None = None,
) -> dict[str, Any]:
    source_checks = validate_dataframe_source_presence(paths)
    missing_sources = [check for check in source_checks if check["status"] != "pass"]
    if missing_sources:
        return _report(
            status="examples-failed",
            source_checks=source_checks,
            cases=[],
            failures=len(missing_sources),
        )

    runner = example_runner or run_dataframe_cases
    cases = runner(paths)
    failures = sum(1 for case in cases if case["status"] != "example-passed")
    return _report(
        status="examples-failed" if failures else "examples-passed",
        source_checks=source_checks,
        cases=cases,
        failures=failures,
    )


def run_dataframe_examples_self_tests(paths: RunnerPaths) -> None:
    skipped_payload = build_dataframe_examples_report(
        paths,
        example_runner=lambda _paths: [
            {
                "id": case_id,
                "status": "example-passed",
                "sifr_source": source,
                "elapsed_ms": 0,
            }
            for case_id, source in DATAFRAME_EXAMPLE_SOURCES.items()
        ],
    )
    if skipped_payload["status"] != "examples-passed":
        raise SystemExit("dataframe examples self-test expected examples-passed")
    case_ids = {case["id"] for case in skipped_payload["cases"]}
    if case_ids != set(DATAFRAME_EXAMPLE_SOURCES):
        raise SystemExit(f"dataframe examples self-test case drift: {sorted(case_ids)}")
    source_ids = {check["id"] for check in skipped_payload["source_checks"]}
    if source_ids != set(DATAFRAME_EXAMPLE_SOURCES):
        raise SystemExit(f"dataframe examples self-test source drift: {sorted(source_ids)}")
    marker_ids = set(DATAFRAME_EXAMPLE_MARKERS)
    if marker_ids != set(DATAFRAME_EXAMPLE_SOURCES):
        raise SystemExit(f"dataframe examples self-test marker drift: {sorted(marker_ids)}")
    roots_ids = set(DATAFRAME_IMPORT_ROOTS)
    if roots_ids != set(DATAFRAME_EXAMPLE_SOURCES):
        raise SystemExit(f"dataframe examples self-test trust-root drift: {sorted(roots_ids)}")

    failed_payload = build_dataframe_examples_report(
        paths,
        example_runner=lambda _paths: [
            {
                "id": "numpy",
                "status": "example-failed",
                "sifr_source": DATAFRAME_EXAMPLE_SOURCES["numpy"],
                "error": "synthetic example failure",
            }
        ],
    )
    if failed_payload["status"] != "examples-failed":
        raise SystemExit("dataframe examples self-test expected examples-failed")
    if failed_payload["summary"]["total_failures"] != 1:
        raise SystemExit("dataframe examples self-test expected one failure")


def validate_dataframe_source_presence(paths: RunnerPaths) -> list[dict[str, Any]]:
    checks: list[dict[str, Any]] = []
    for case_id, relative_source in DATAFRAME_EXAMPLE_SOURCES.items():
        source_path = paths.fixtures_root / relative_source
        if not source_path.is_file():
            checks.append(
                {
                    "id": case_id,
                    "status": "fail",
                    "sifr_source": relative_source,
                    "reason": "missing source fixture",
                }
            )
            continue
        checks.append(
            {
                "id": case_id,
                "status": "pass",
                "sifr_source": relative_source,
                "check": "source-present",
            }
        )
    return checks


def run_dataframe_cases(paths: RunnerPaths) -> list[dict[str, Any]]:
    cases: list[dict[str, Any]] = []
    for case_id, relative_source in DATAFRAME_EXAMPLE_SOURCES.items():
        package_root = prepare_dataframe_example_package(paths, case_id, relative_source)
        cases.append(_run_case(paths, package_root, case_id, relative_source))
    return cases


def prepare_dataframe_example_package(paths: RunnerPaths, case_id: str, relative_source: str) -> Path:
    package_root = (
        paths.repo_root
        / "target"
        / "verification"
        / "areas"
        / "python_interop"
        / "dataframe_examples_package"
        / case_id
    )
    if package_root.exists():
        shutil.rmtree(package_root)
    source_root = package_root / "src"
    source_root.mkdir(parents=True, exist_ok=True)
    source_path = paths.fixtures_root / relative_source
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
                f'name = "sifr-python-interop-dataframe-examples-{case_id}"',
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
    roots = ", ".join(f'"{root}"' for root in DATAFRAME_IMPORT_ROOTS[case_id])
    (package_root / "sifr.toml").write_text(
        "\n".join(
            [
                "[package]",
                'name = "python_interop_dataframe_examples"',
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
            "python interop dataframe examples require the area uv environment; "
            "run through `uv run --project verification/areas/python_interop --locked ...`"
        )
    venv_link.symlink_to(area_venv, target_is_directory=True)
    return package_root


def _run_case(
    paths: RunnerPaths,
    package_root: Path,
    case_id: str,
    relative_source: str,
) -> dict[str, Any]:
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
            "id": case_id,
            "status": "example-failed",
            "sifr_source": relative_source,
            "elapsed_ms": elapsed_ms,
            "reason": "timeout",
            "timeout_seconds": EXAMPLE_TIMEOUT_SECONDS,
            "stdout": (error.stdout or "")[-4000:],
            "stderr": (error.stderr or "")[-4000:],
        }
    elapsed_ms = round((time.perf_counter() - started) * 1000.0)
    expected_marker = DATAFRAME_EXAMPLE_MARKERS[case_id]
    marker_observed = expected_marker in proc.stdout
    status = "example-passed" if proc.returncode == 0 and marker_observed else "example-failed"
    case: dict[str, Any] = {
        "id": case_id,
        "status": status,
        "sifr_source": relative_source,
        "elapsed_ms": elapsed_ms,
        "stdout_marker": expected_marker,
        "stdout_marker_observed": marker_observed,
        "trusted_import_roots": list(DATAFRAME_IMPORT_ROOTS[case_id]),
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
    status: str,
    source_checks: list[dict[str, Any]],
    cases: list[dict[str, Any]],
    failures: int,
) -> dict[str, Any]:
    return {
        "schema_version": 1,
        "area": "python_interop",
        "status": status,
        "result_statuses": ["example-passed", "example-failed"],
        "dependencies": sorted({root for roots in DATAFRAME_IMPORT_ROOTS.values() for root in roots}),
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
