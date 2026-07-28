"""Distribution release verification area adapter."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path
from typing import Any

from verification.areas.distribution_release.governance.common import (
    PRODUCTION_CREDENTIAL_NAMES,
)

REPO_ROOT = Path(__file__).resolve().parents[3]
AREA_ROOT = Path(__file__).resolve().parent
MANIFEST_PATH = AREA_ROOT / "manifest.json"
CASES_ROOT = AREA_ROOT / "cases"
RESULT_JSON = REPO_ROOT / "target" / "verification" / "areas" / "distribution-release-results.json"


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite", action="append", default=[], help="Suite filter; can repeat.")
    parser.add_argument("--bless", action="store_true", help="Accepted for area runner parity; unused.")
    parser.add_argument(
        "--result-json",
        default=str(RESULT_JSON.relative_to(REPO_ROOT)),
        help="Path for machine-readable distribution release result summary.",
    )
    parser.add_argument(
        "--hardening-summary",
        action="store_true",
        help="Emit a legacy verification summary line for direct area invocations.",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    if args.bless:
        raise SystemExit("distribution_release area does not support --bless")
    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    selected = select_suites(manifest, set(args.suite))

    print("Running distribution release verification area", flush=True)
    print(f"  manifest={MANIFEST_PATH.relative_to(REPO_ROOT)}", flush=True)
    print("  bless=no", flush=True)

    incident_suite_selected = any(
        str(suite["name"]) == "incident-governance" for suite in selected
    )
    epoch_suite_selected = any(
        str(suite["name"]) == "epoch-bootstrap" for suite in selected
    )
    drill_suite_selected = any(
        str(suite["name"]) == "protected-drill" for suite in selected
    )
    stable_prepare_selected = any(
        str(suite["name"]) == "stable-prepare" for suite in selected
    )
    stable_publish_primitives_selected = any(
        str(suite["name"]) == "stable-publish-primitives" for suite in selected
    )
    suite_results = [
        run_suite(
            suite,
            include_incident=not incident_suite_selected,
            include_epoch_bootstrap=not epoch_suite_selected,
            include_protected_drill=not drill_suite_selected,
            include_stable_prepare=not stable_prepare_selected,
            include_stable_publish_primitives=not stable_publish_primitives_selected,
        )
        for suite in selected
    ]
    total_variants = sum(int(result["total_variants"]) for result in suite_results)
    total_failures = sum(int(result["total_failures"]) for result in suite_results)
    payload = {
        "schema_version": 1,
        "area": "distribution_release",
        "bless": False,
        "manifest": str(MANIFEST_PATH.relative_to(REPO_ROOT)),
        "suites": suite_results,
        "summary": {
            "total_variants": total_variants,
            "total_failures": total_failures,
            "blocking_failures": total_failures,
            "non_blocking_failures": 0,
        },
    }
    result_path = REPO_ROOT / args.result_json
    result_path.parent.mkdir(parents=True, exist_ok=True)
    result_path.write_text(json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8")
    print(f"result_json={result_path.relative_to(REPO_ROOT)}", flush=True)

    if total_failures:
        print(
            f"verification failed: variants={total_variants}, failures={total_failures}, "
            f"blocking_failures={total_failures}, non_blocking_failures=0",
            file=sys.stderr,
            flush=True,
        )
        return 1
    prefix = "verification ok" if args.hardening_summary else "distribution release verification ok"
    print(
        f"{prefix}: variants={total_variants}, failures={total_failures}, "
        "blocking_failures=0, non_blocking_failures=0",
        flush=True,
    )
    return 0


def select_suites(manifest: dict[str, Any], requested: set[str]) -> list[dict[str, Any]]:
    suites = manifest.get("suites", [])
    selected = [suite for suite in suites if not requested or str(suite.get("name")) in requested]
    if requested:
        present = {str(suite.get("name")) for suite in selected}
        missing = sorted(requested.difference(present))
        if missing:
            raise SystemExit(f"unknown distribution_release suite filter(s): {', '.join(missing)}")
    if not selected:
        raise SystemExit("no distribution_release suites selected")
    return selected


def run_suite(
    suite: dict[str, Any],
    *,
    include_incident: bool = True,
    include_epoch_bootstrap: bool = True,
    include_protected_drill: bool = True,
    include_stable_prepare: bool = True,
    include_stable_publish_primitives: bool = True,
) -> dict[str, Any]:
    suite_name = str(suite["name"])
    case = validate_suite_case(suite)
    if suite_name == "evidence-custody":
        variants = [
            run_python_module(
                "governance.evidence_custody",
                "evidence-custody",
            )
        ]
    elif suite_name == "incident-governance":
        variants = [
            run_python_module(
                "governance.incident_recovery_selftest",
                "incident-recovery",
            )
        ]
    elif suite_name == "epoch-bootstrap":
        variants = [
            run_python_module(
                "governance.schema_bootstrap_selftest",
                "schema-v2-preview-epoch-bootstrap",
            )
        ]
    elif suite_name == "protected-drill":
        variants = [
            run_python_module(
                "governance.protected_drill_selftest",
                "protected-stable-release-drill",
                scrub_credentials=True,
            )
        ]
    elif suite_name == "stable-prepare":
        variants = [
            run_python_module(
                "governance.stable_prepare_selftest",
                "stable-publication-prepare",
            )
        ]
    elif suite_name == "stable-publish-primitives":
        variants = [
            run_python_module(
                "governance.stable_publication_primitives_selftest",
                "stable-publication-primitives",
            )
        ]
    elif suite_name == "qualification":
        variants = [
            run_python_module(
                "governance.qualification_selftest",
                "stable-qualification",
            )
        ]
    else:
        variants = [run_distribution_case(script) for script in distribution_case_scripts()]
        if suite_name == "full":
            variants.append(run_python_module("governance.selftest", "governance-contracts"))
            variants.append(run_python_module("governance.schema_epoch", "schema-epoch"))
            if include_epoch_bootstrap:
                variants.append(
                    run_python_module(
                        "governance.schema_bootstrap_selftest",
                        "schema-v2-preview-epoch-bootstrap",
                    )
                )
            if include_incident:
                variants.append(
                    run_python_module(
                        "governance.incident_recovery_selftest",
                        "incident-recovery",
                    )
                )
            if include_protected_drill:
                variants.append(
                    run_python_module(
                        "governance.protected_drill_selftest",
                        "protected-stable-release-drill",
                        scrub_credentials=True,
                    )
                )
            if include_stable_prepare:
                variants.append(
                    run_python_module(
                        "governance.stable_prepare_selftest",
                        "stable-publication-prepare",
                    )
                )
            if include_stable_publish_primitives:
                variants.append(
                    run_python_module(
                        "governance.stable_publication_primitives_selftest",
                        "stable-publication-primitives",
                    )
                )
    failures = sum(1 for variant in variants if variant["status"] == "fail")
    return {
        "name": suite_name,
        "owner": "release/distribution",
        "blocking": True,
        "runner": "distribution-release",
        "cases": [
            {
                "id": str(case["id"]),
                "entry": str(case["entry"]),
                "command": str(case["command"]),
                "variants": variants,
            }
        ],
        "failed_cases": failures,
        "total_variants": len(variants),
        "total_failures": failures,
    }


def validate_suite_case(suite: dict[str, Any]) -> dict[str, Any]:
    suite_name = str(suite["name"])
    if suite_name not in {
        "representative",
        "full",
        "qualification",
        "evidence-custody",
        "incident-governance",
        "epoch-bootstrap",
        "protected-drill",
        "stable-prepare",
        "stable-publish-primitives",
    }:
        raise SystemExit(f"unsupported distribution_release suite: {suite_name}")
    cases = suite.get("cases", [])
    if not isinstance(cases, list) or len(cases) != 1:
        raise SystemExit(f"distribution_release suite '{suite_name}' must contain exactly one case directory")
    case = cases[0]
    expected_command = {
        "evidence-custody": "distribution-evidence-custody",
        "incident-governance": "distribution-incident-recovery",
        "epoch-bootstrap": "distribution-schema-bootstrap",
        "protected-drill": "distribution-protected-drill",
        "stable-prepare": "distribution-stable-prepare",
        "stable-publish-primitives": "distribution-stable-publish-primitives",
        "qualification": "distribution-stable-qualification",
    }.get(suite_name, "distribution-case-directory")
    if str(case.get("command")) != expected_command:
        raise SystemExit(f"distribution_release suite '{suite_name}' must use {expected_command}")
    entry = REPO_ROOT / str(case.get("entry"))
    expected_entry = {
        "evidence-custody": AREA_ROOT / "governance" / "evidence_custody.py",
        "incident-governance": AREA_ROOT / "governance" / "incident_recovery_selftest.py",
        "epoch-bootstrap": AREA_ROOT / "governance" / "schema_bootstrap_selftest.py",
        "protected-drill": AREA_ROOT / "governance" / "protected_drill_selftest.py",
        "stable-prepare": AREA_ROOT / "governance" / "stable_prepare_selftest.py",
        "stable-publish-primitives": (
            AREA_ROOT / "governance" / "stable_publication_primitives_selftest.py"
        ),
        "qualification": AREA_ROOT / "governance" / "qualification_selftest.py",
    }.get(suite_name, CASES_ROOT)
    if entry != expected_entry or not entry.exists():
        raise SystemExit(f"distribution_release suite '{suite_name}' entry does not exist: {entry}")
    return case


def distribution_case_scripts() -> list[Path]:
    scripts = sorted(path for path in CASES_ROOT.glob("*.sh") if path.name != "common.sh")
    if not scripts:
        raise SystemExit("distribution_release has no executable case scripts")
    return scripts


def run_distribution_case(script: Path) -> dict[str, Any]:
    label = script.stem
    print(f"Running {script.relative_to(REPO_ROOT)}", flush=True)
    started = time.perf_counter()
    result = subprocess.run([str(script)], cwd=REPO_ROOT, check=False)
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    status = "pass" if result.returncode == 0 else "fail"
    print_case_timing(label, elapsed_ms, status)
    return {
        "label": label,
        "argv": [str(script.relative_to(REPO_ROOT))],
        "status": status,
        "mismatches": [] if status == "pass" else [f"exit={result.returncode} expected=0"],
        "expected_exit_code": 0,
        "actual_exit_code": result.returncode,
        "duration_ms": round(elapsed_ms, 3),
    }


def run_python_module(
    module: str,
    label: str,
    *,
    scrub_credentials: bool = False,
) -> dict[str, Any]:
    print(f"Running distribution governance module {module}", flush=True)
    started = time.perf_counter()
    qualified_module = f"verification.areas.distribution_release.{module}"
    environment = os.environ.copy()
    if scrub_credentials:
        for name in PRODUCTION_CREDENTIAL_NAMES:
            environment.pop(name, None)
    result = subprocess.run(
        [sys.executable, "-m", qualified_module],
        cwd=REPO_ROOT,
        env=environment,
        check=False,
    )
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    status = "pass" if result.returncode == 0 else "fail"
    print_case_timing(label, elapsed_ms, status)
    return {
        "label": label,
        "argv": [sys.executable, "-m", qualified_module],
        "status": status,
        "mismatches": [] if status == "pass" else [f"exit={result.returncode} expected=0"],
        "expected_exit_code": 0,
        "actual_exit_code": result.returncode,
        "duration_ms": round(elapsed_ms, 3),
    }


def print_case_timing(label: str, elapsed_ms: float, status: str) -> None:
    print(
        f"[sifr-case-timing] bucket=distribution_release case={timing_token(label)} "
        f"elapsed_ms={int(elapsed_ms)} status={status}",
        flush=True,
    )


def timing_token(value: object) -> str:
    return "".join(char if char.isalnum() or char in "_.:/+-" else "_" for char in str(value))


if __name__ == "__main__":
    raise SystemExit(main())
