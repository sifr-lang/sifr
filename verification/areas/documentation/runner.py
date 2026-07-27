"""Documentation verification area adapter."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[3]
AREA_ROOT = Path(__file__).resolve().parent
MANIFEST_PATH = AREA_ROOT / "manifest.json"
RESULT_JSON = REPO_ROOT / "target" / "verification" / "areas" / "documentation-results.json"
SUITE_COMMANDS = {
    "structure": [
        sys.executable,
        str(AREA_ROOT / "check_structure.py"),
    ],
    "ga-release": [
        sys.executable,
        str(AREA_ROOT / "check_ga_release_docs.py"),
    ],
}


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite", action="append", default=[])
    parser.add_argument("--bless", action="store_true")
    parser.add_argument("--result-json", default=str(RESULT_JSON.relative_to(REPO_ROOT)))
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    if args.bless:
        raise SystemExit("documentation area does not support --bless")
    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    selected = select_suites(manifest, set(args.suite))
    print("Running documentation verification area", flush=True)
    suite_results = [run_suite(suite) for suite in selected]
    failures = sum(int(result["total_failures"]) for result in suite_results)
    variants = sum(int(result["total_variants"]) for result in suite_results)
    payload = {
        "schema_version": 1,
        "area": "documentation",
        "bless": False,
        "manifest": str(MANIFEST_PATH.relative_to(REPO_ROOT)),
        "suites": suite_results,
        "summary": {
            "total_variants": variants,
            "total_failures": failures,
            "blocking_failures": failures,
            "non_blocking_failures": 0,
        },
    }
    result_path = REPO_ROOT / args.result_json
    result_path.parent.mkdir(parents=True, exist_ok=True)
    result_path.write_text(json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8")
    print(f"result_json={result_path.relative_to(REPO_ROOT)}", flush=True)
    if failures:
        print(f"documentation verification failed: failures={failures}", file=sys.stderr)
        return 1
    print(f"documentation verification ok: variants={variants}, failures=0")
    return 0


def select_suites(manifest: dict[str, Any], requested: set[str]) -> list[dict[str, Any]]:
    suites = manifest.get("suites", [])
    selected = [suite for suite in suites if not requested or suite.get("name") in requested]
    missing = sorted(requested.difference(str(suite.get("name")) for suite in selected))
    if missing:
        raise SystemExit(f"unknown documentation suite filter(s): {', '.join(missing)}")
    if not selected:
        raise SystemExit("no documentation suites selected")
    return selected


def run_suite(suite: dict[str, Any]) -> dict[str, Any]:
    name = str(suite["name"])
    command = SUITE_COMMANDS.get(name)
    if command is None:
        raise SystemExit(f"unsupported documentation suite: {name}")
    started = time.perf_counter()
    result = subprocess.run(command, cwd=REPO_ROOT, check=False)
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    status = "pass" if result.returncode == 0 else "fail"
    print(
        f"[sifr-case-timing] bucket=documentation case={name} "
        f"elapsed_ms={int(elapsed_ms)} status={status}",
        flush=True,
    )
    case = suite["cases"][0]
    return {
        "name": name,
        "owner": "documentation",
        "blocking": True,
        "runner": "documentation",
        "cases": [
            {
                "id": case["id"],
                "entry": case["entry"],
                "command": case["command"],
                "variants": [
                    {
                        "label": name,
                        "argv": command,
                        "status": status,
                        "mismatches": [] if status == "pass" else ["unexpected-exit"],
                        "expected_exit_code": 0,
                        "actual_exit_code": result.returncode,
                        "duration_ms": round(elapsed_ms, 3),
                    }
                ],
            }
        ],
        "failed_cases": 0 if status == "pass" else 1,
        "total_variants": 1,
        "total_failures": 0 if status == "pass" else 1,
    }


if __name__ == "__main__":
    raise SystemExit(main())
