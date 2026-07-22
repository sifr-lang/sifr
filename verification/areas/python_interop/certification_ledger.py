"""Bind declaration capability claims to fresh compiled-example reports."""

from __future__ import annotations

import hashlib
import json
from collections.abc import Callable
from pathlib import Path
from typing import Any

ReportLoader = Callable[[Path], tuple[bytes, object]]


def build_compiled_certification(
    matrix: dict[str, Any],
    repo_root: Path,
    suite_results: list[dict[str, Any]],
    *,
    report_loader: ReportLoader | None = None,
) -> dict[str, Any]:
    """Return current-run certification without promoting unselected suites."""

    loader = report_loader or _load_report
    suites = {str(result["name"]): result for result in suite_results}
    compiled_rows = [row for row in matrix["capabilities"] if row.get("compiled_evidence")]
    required_suites = {
        str(entry["suite"])
        for row in compiled_rows
        for entry in row["compiled_evidence"]
    }
    required_reports_by_suite = {
        suite: {
            str(entry["report"])
            for row in compiled_rows
            for entry in row["compiled_evidence"]
            if str(entry["suite"]) == suite
        }
        for suite in required_suites
    }
    _validate_suite_report_invocations(required_reports_by_suite, suites, repo_root)
    report_cache: dict[str, tuple[bytes, object]] = {}
    records: list[dict[str, Any]] = []

    for row in compiled_rows:
        entries = row["compiled_evidence"]
        row_suites = {str(entry["suite"]) for entry in entries}
        selected_suites = row_suites.intersection(suites)
        if not selected_suites:
            records.append(
                {
                    "capability_id": row["id"],
                    "status": "not-selected",
                    "evidence": [],
                }
            )
            continue
        if selected_suites != row_suites:
            missing = sorted(row_suites.difference(selected_suites))
            raise SystemExit(
                f"partial compiled certification selection for {row['id']}: missing {missing}"
            )
        failed_suites = sorted(
            suite
            for suite in row_suites
            if int(suites[suite].get("total_failures", 1)) != 0
        )
        if failed_suites:
            records.append(
                {
                    "capability_id": row["id"],
                    "status": "failed",
                    "failed_suites": failed_suites,
                    "evidence": [],
                }
            )
            continue

        evidence = [
            _validate_entry(entry, repo_root, loader, report_cache) for entry in entries
        ]
        records.append(
            {
                "capability_id": row["id"],
                "status": "passing",
                "evidence": evidence,
            }
        )

    passing = sum(record["status"] == "passing" for record in records)
    failed = sum(record["status"] == "failed" for record in records)
    selected_required_suites = required_suites.intersection(suites)
    if failed:
        status = "failed"
    elif selected_required_suites == required_suites:
        status = "complete"
    elif selected_required_suites:
        status = "partial"
    else:
        status = "not-selected"
    return {
        "schema_version": 1,
        "status": status,
        "matrix": "verification/areas/python_interop/declaration_capabilities.json",
        "matrix_schema_version": matrix["schema_version"],
        "required_suites": sorted(required_suites),
        "selected_required_suites": sorted(selected_required_suites),
        "summary": {
            "capabilities": len(records),
            "passing": passing,
            "failed": failed,
            "not_selected": sum(record["status"] == "not-selected" for record in records),
            "compiled_evidence": sum(len(record["evidence"]) for record in records),
            "resource_zero_evidence": sum(
                item["resource_state"] == "zero"
                for record in records
                for item in record["evidence"]
            ),
        },
        "capabilities": records,
    }


def _validate_suite_report_invocations(
    required_reports_by_suite: dict[str, set[str]],
    suites: dict[str, dict[str, Any]],
    repo_root: Path,
) -> None:
    area_root = repo_root / "verification" / "areas" / "python_interop"
    for suite_name in sorted(required_reports_by_suite.keys() & suites.keys()):
        expected_reports = required_reports_by_suite[suite_name]
        if len(expected_reports) != 1:
            raise SystemExit(
                f"compiled certification suite {suite_name} must own exactly one report"
            )
        expected_path = (repo_root / next(iter(expected_reports))).resolve()
        observed_paths: set[Path] = set()
        cases = suites[suite_name].get("cases")
        if not isinstance(cases, list):
            raise SystemExit(f"compiled certification invocation drift for {suite_name}")
        for case in cases:
            variants = case.get("variants") if isinstance(case, dict) else None
            if not isinstance(variants, list):
                raise SystemExit(f"compiled certification invocation drift for {suite_name}")
            for variant in variants:
                argv = variant.get("argv") if isinstance(variant, dict) else None
                if not isinstance(argv, list) or not all(isinstance(arg, str) for arg in argv):
                    raise SystemExit(f"compiled certification invocation drift for {suite_name}")
                report_indexes = [index for index, arg in enumerate(argv) if arg == "--report"]
                for report_index in report_indexes:
                    value_index = report_index + 1
                    if value_index >= len(argv):
                        raise SystemExit(
                            f"compiled certification invocation drift for {suite_name}"
                        )
                    report_argument = Path(argv[value_index])
                    resolved = (
                        report_argument.resolve()
                        if report_argument.is_absolute()
                        else (area_root / report_argument).resolve()
                    )
                    observed_paths.add(resolved)
        if observed_paths != {expected_path}:
            raise SystemExit(f"compiled certification invocation drift for {suite_name}")


def _validate_entry(
    entry: dict[str, Any],
    repo_root: Path,
    loader: ReportLoader,
    cache: dict[str, tuple[bytes, object]],
) -> dict[str, Any]:
    report_name = str(entry["report"])
    report_path = repo_root / report_name
    if report_name not in cache:
        cache[report_name] = loader(report_path)
    raw, payload = cache[report_name]
    if not isinstance(payload, dict) or payload.get("schema_version") != 1:
        raise SystemExit(f"compiled certification report schema drift: {report_name}")
    if payload.get("status") != "examples-passed":
        raise SystemExit(f"compiled certification report did not pass: {report_name}")
    summary = payload.get("summary")
    if (
        not isinstance(summary, dict)
        or summary.get("total_failures") != 0
        or summary.get("blocking_failures") != 0
        or summary.get("skipped") != 0
    ):
        raise SystemExit(f"compiled certification report summary drift: {report_name}")

    cases = payload.get("cases")
    matches = (
        [case for case in cases if isinstance(case, dict) and case.get("id") == entry["case"]]
        if isinstance(cases, list)
        else []
    )
    if len(matches) != 1:
        raise SystemExit(
            f"compiled certification case {entry['case']} must appear exactly once in {report_name}"
        )
    case = matches[0]
    marker = entry["stdout_marker"]
    if (
        case.get("status") != "example-passed"
        or case.get("execution_model") != "compiled-sifr-declaration"
        or case.get("sifr_source") != entry["sifr_source"]
        or case.get("stdout_marker") != marker
        or case.get("stdout_marker_observed") is not True
        or marker not in str(case.get("stdout", ""))
    ):
        raise SystemExit(f"compiled certification evidence drift for {entry['id']}")
    source_path = repo_root / "verification/areas/python_interop/fixtures" / entry["sifr_source"]
    if not source_path.is_file():
        raise SystemExit(f"compiled certification source is missing for {entry['id']}")

    commands = case.get("certification_commands")
    minimum_commands = entry["minimum_certification_commands"]
    if (
        not isinstance(commands, list)
        or len(commands) < minimum_commands
        or any(not isinstance(command, dict) or command.get("exit_code") != 0 for command in commands)
    ):
        raise SystemExit(f"compiled certification command evidence drift for {entry['id']}")
    import_roots = _validated_roots(case, "trusted_import_roots", entry["id"])
    native_roots = _validated_roots(case, "trusted_native_roots", entry["id"])
    resource_state = "zero" if entry["requires_resource_zero"] else "checked"
    if resource_state == "zero" and not marker.endswith(":resources=zero"):
        raise SystemExit(f"compiled certification resource evidence drift for {entry['id']}")
    return {
        "id": entry["id"],
        "suite": entry["suite"],
        "report": report_name,
        "report_sha256": hashlib.sha256(raw).hexdigest(),
        "case": entry["case"],
        "sifr_source": entry["sifr_source"],
        "execution_model": "compiled-sifr-declaration",
        "stdout_marker": marker,
        "resource_state": resource_state,
        "certification_commands": len(commands),
        "trusted_import_roots": import_roots,
        "trusted_native_roots": native_roots,
    }


def _validated_roots(case: dict[str, Any], key: str, evidence_id: str) -> list[str]:
    roots = case.get(key)
    if (
        not isinstance(roots, list)
        or any(not isinstance(root, str) or not root for root in roots)
        or len(roots) != len(set(roots))
    ):
        raise SystemExit(f"compiled certification {key} drift for {evidence_id}")
    return roots


def _load_report(path: Path) -> tuple[bytes, object]:
    if not path.is_file():
        raise SystemExit(f"missing fresh compiled certification report: {path}")
    raw = path.read_bytes()
    try:
        return raw, json.loads(raw)
    except json.JSONDecodeError as error:
        raise SystemExit(f"invalid compiled certification report JSON: {path}: {error}") from error


def run_compiled_certification_self_tests(matrix: dict[str, Any], repo_root: Path) -> None:
    suites = sorted(
        {
            entry["suite"]
            for row in matrix["capabilities"]
            for entry in row.get("compiled_evidence", [])
        }
    )
    reports_by_suite = {
        suite: next(
            entry["report"]
            for row in matrix["capabilities"]
            for entry in row.get("compiled_evidence", [])
            if entry["suite"] == suite
        )
        for suite in suites
    }
    suite_results = [
        {
            "name": suite,
            "total_failures": 0,
            "cases": [
                {
                    "variants": [
                        {
                            "argv": [
                                "python",
                                "runner.py",
                                "--report",
                                str(repo_root / reports_by_suite[suite]),
                            ]
                        }
                    ]
                }
            ],
        }
        for suite in suites
    ]
    reports = _synthetic_reports(matrix)

    def load(path: Path) -> tuple[bytes, object]:
        relative = str(path.relative_to(repo_root))
        payload = reports[relative]
        raw = json.dumps(payload, sort_keys=True).encode("utf-8")
        return raw, payload

    complete = build_compiled_certification(
        matrix,
        repo_root,
        suite_results,
        report_loader=load,
    )
    if complete["status"] != "complete" or complete["summary"]["failed"] != 0:
        raise SystemExit("compiled certification self-test expected complete evidence")

    rebound_suite = json.loads(json.dumps(suite_results))
    rebound_suite[0]["cases"][0]["variants"][0]["argv"] = ["python", "runner.py"]
    _expect_certification_rejection(
        matrix,
        repo_root,
        rebound_suite,
        reports,
        "invocation drift",
    )

    python_runner = json.loads(json.dumps(reports))
    next(iter(python_runner.values()))["cases"][0]["execution_model"] = "python-runner"
    _expect_certification_rejection(
        matrix,
        repo_root,
        suite_results,
        python_runner,
        "evidence drift",
    )

    skipped = json.loads(json.dumps(reports))
    next(iter(skipped.values()))["summary"]["skipped"] = 1
    _expect_certification_rejection(
        matrix,
        repo_root,
        suite_results,
        skipped,
        "summary drift",
    )

    duplicate = json.loads(json.dumps(reports))
    first_report = next(iter(duplicate.values()))
    first_report["cases"].append(json.loads(json.dumps(first_report["cases"][0])))
    _expect_certification_rejection(
        matrix,
        repo_root,
        suite_results,
        duplicate,
        "exactly once",
    )

    hidden_marker = json.loads(json.dumps(reports))
    next(iter(hidden_marker.values()))["cases"][0]["stdout_marker_observed"] = False
    _expect_certification_rejection(
        matrix,
        repo_root,
        suite_results,
        hidden_marker,
        "evidence drift",
    )

    failed_command = json.loads(json.dumps(reports))
    command_case = next(
        case
        for report in failed_command.values()
        for case in report["cases"]
        if case["certification_commands"]
    )
    command_case["certification_commands"][0]["exit_code"] = 1
    _expect_certification_rejection(
        matrix,
        repo_root,
        suite_results,
        failed_command,
        "command evidence drift",
    )


def _expect_certification_rejection(
    matrix: dict[str, Any],
    repo_root: Path,
    suite_results: list[dict[str, Any]],
    reports: dict[str, dict[str, Any]],
    expected: str,
) -> None:
    def load(path: Path) -> tuple[bytes, object]:
        relative = str(path.relative_to(repo_root))
        payload = reports[relative]
        return json.dumps(payload).encode("utf-8"), payload

    try:
        build_compiled_certification(
            matrix,
            repo_root,
            suite_results,
            report_loader=load,
        )
    except SystemExit as error:
        if expected not in str(error):
            raise SystemExit(
                f"compiled certification negative self-test expected {expected!r}, got {error!r}"
            ) from error
    else:
        raise SystemExit(
            f"compiled certification self-test accepted invalid evidence ({expected})"
        )


def _synthetic_reports(matrix: dict[str, Any]) -> dict[str, dict[str, Any]]:
    reports: dict[str, dict[str, Any]] = {}
    for row in matrix["capabilities"]:
        for entry in row.get("compiled_evidence", []):
            report = reports.setdefault(
                entry["report"],
                {
                    "schema_version": 1,
                    "status": "examples-passed",
                    "summary": {
                        "total_failures": 0,
                        "blocking_failures": 0,
                        "non_blocking_failures": 0,
                        "skipped": 0,
                        "total_variants": 1,
                    },
                    "cases": [],
                },
            )
            report["cases"].append(
                {
                    "id": entry["case"],
                    "status": "example-passed",
                    "execution_model": "compiled-sifr-declaration",
                    "sifr_source": entry["sifr_source"],
                    "stdout": entry["stdout_marker"] + "\n",
                    "stdout_marker": entry["stdout_marker"],
                    "stdout_marker_observed": True,
                    "certification_commands": [
                        {"exit_code": 0}
                        for _ in range(entry["minimum_certification_commands"])
                    ],
                    "trusted_import_roots": ["synthetic"],
                    "trusted_native_roots": [],
                }
            )
    return reports
