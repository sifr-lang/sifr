from __future__ import annotations

import shutil
import subprocess
import time
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from env import RunnerPaths, cargo_env_for_repo_manifest

EXAMPLE_TIMEOUT_SECONDS = 600


@dataclass(frozen=True)
class ExampleCase:
    case_id: str
    relative_source: str
    stdout_marker: str
    import_roots: tuple[str, ...]
    native_roots: tuple[str, ...] | None = None
    copy_bridges: bool = True
    bridge_files: tuple[str, ...] | None = None
    arrow_certifications: tuple[tuple[str, str], ...] = ()
    explicit_requirements: bool = True


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
    observed_case_ids = [case.get("id") for case in case_results]
    if len(observed_case_ids) != len(set(observed_case_ids)) or set(observed_case_ids) != set(
        cases_by_id
    ):
        failures = max(1, failures)
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
    invalid_roots = {
        case.case_id
        for case in cases_by_id.values()
        if any(not root for root in case.import_roots)
        or len(case.import_roots) != len(set(case.import_roots))
        or (
            case.native_roots is not None
            and (
                any(not root for root in case.native_roots)
                or len(case.native_roots) != len(set(case.native_roots))
            )
        )
        or (
            case.bridge_files is not None
            and (
                any(not bridge_file for bridge_file in case.bridge_files)
                or len(case.bridge_files) != len(set(case.bridge_files))
            )
        )
        or any(not target or not fixture for target, fixture in case.arrow_certifications)
        or len({target for target, _fixture in case.arrow_certifications})
        != len(case.arrow_certifications)
    }
    if invalid_roots:
        raise SystemExit(
            f"{suite_name} examples self-test invalid trust roots: {sorted(invalid_roots)}"
        )

    empty_payload = build_examples_report(
        paths,
        suite_name=suite_name,
        cases_by_id=cases_by_id,
        example_runner=lambda _paths: [],
    )
    if empty_payload["status"] != "examples-failed":
        raise SystemExit(f"{suite_name} examples self-test accepted an empty result set")

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
        if case.bridge_files is not None:
            bridge_root = source_path.parent / "python_bridges"
            missing_bridges = [
                bridge_file
                for bridge_file in case.bridge_files
                if not (bridge_root / bridge_file).is_file()
            ]
            if missing_bridges:
                checks.append(
                    {
                        "id": case.case_id,
                        "status": "fail",
                        "sifr_source": case.relative_source,
                        "reason": f"missing bridge files: {missing_bridges}",
                    }
                )
                continue
        missing_certification_fixtures = [
            fixture
            for _target, fixture in case.arrow_certifications
            if not (source_path.parent / fixture).is_file()
        ]
        if missing_certification_fixtures:
            checks.append(
                {
                    "id": case.case_id,
                    "status": "fail",
                    "sifr_source": case.relative_source,
                    "reason": (
                        "missing Arrow certification fixtures: "
                        f"{missing_certification_fixtures}"
                    ),
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
    bridge_source = source_path.parent / "python_bridges"
    if case.copy_bridges and bridge_source.is_dir():
        if case.bridge_files is None:
            shutil.copytree(bridge_source, source_root / "python_bridges")
        else:
            bridge_target = source_root / "python_bridges"
            bridge_target.mkdir()
            for bridge_file in case.bridge_files:
                shutil.copy2(bridge_source / bridge_file, bridge_target / bridge_file)
    for _target, fixture in case.arrow_certifications:
        fixture_source = source_path.parent / fixture
        fixture_target = package_root / fixture
        fixture_target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(fixture_source, fixture_target)
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
    native_roots = case.native_roots if case.native_roots is not None else case.import_roots
    native_roots_toml = ", ".join(f'"{root}"' for root in native_roots)
    manifest_lines = [
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
        'pyproject = "pyproject.toml"',
        'lock = "uv.lock"',
    ]
    if case.explicit_requirements:
        manifest_lines.append(f"requires-imports = [{roots}]")
    manifest_lines.extend(
        [
            "",
            "[trust]",
            f"python = [{roots}]",
            f"python-native = [{native_roots_toml}]",
            "",
        ]
    )
    (package_root / "sifr.toml").write_text(
        "\n".join(manifest_lines),
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
    (package_root / "pyproject.toml").symlink_to(paths.area_root / "pyproject.toml")
    (package_root / "uv.lock").symlink_to(paths.area_root / "uv.lock")
    return package_root


def _run_case(paths: RunnerPaths, package_root: Path, case_config: ExampleCase) -> dict[str, Any]:
    started = time.perf_counter()
    certification_commands = [
        ["python", "certify", "arrow", target, "--fixture", fixture]
        for target, fixture in case_config.arrow_certifications
    ]
    if certification_commands:
        certification_commands.append(["python", "certify", "--check"])
    certification_results: list[dict[str, Any]] = []
    for arguments in certification_commands:
        result = _run_sifr_command(paths, package_root, arguments)
        certification_results.append(result)
        if result["exit_code"] != 0:
            return {
                "id": case_config.case_id,
                "status": "example-failed",
                "sifr_source": case_config.relative_source,
                "elapsed_ms": round((time.perf_counter() - started) * 1000.0),
                "reason": "Arrow certification failed",
                "certification_commands": certification_results,
            }
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
            env=cargo_env_for_repo_manifest(paths.repo_root),
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
        "trusted_native_roots": list(case_config.native_roots if case_config.native_roots is not None else case_config.import_roots),
        "certification_commands": certification_results,
        "stdout": proc.stdout[-4000:],
    }
    if status != "example-passed":
        case["stderr"] = proc.stderr[-4000:]
        case["exit_code"] = proc.returncode
        if proc.returncode == 0 and not marker_observed:
            case["reason"] = "missing stdout marker"
    return case


def _run_sifr_command(
    paths: RunnerPaths,
    package_root: Path,
    arguments: list[str],
) -> dict[str, Any]:
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
            *arguments,
        ],
        cwd=package_root,
        env=cargo_env_for_repo_manifest(paths.repo_root),
        text=True,
        capture_output=True,
        check=False,
        timeout=EXAMPLE_TIMEOUT_SECONDS,
    )
    return {
        "arguments": arguments,
        "exit_code": proc.returncode,
        "stdout": proc.stdout[-4000:],
        "stderr": proc.stderr[-4000:],
    }


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
