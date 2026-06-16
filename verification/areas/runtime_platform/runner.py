"""Runtime platform verification area adapter."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[3]
AREA_ROOT = Path(__file__).resolve().parent
MANIFEST_PATH = AREA_ROOT / "manifest.json"
GOLDEN_MANIFEST = AREA_ROOT / "golden" / "manifest.json"
PLATFORM_CONTRACT = AREA_ROOT / "platform_contract.json"
SANITIZER_MANIFEST = AREA_ROOT / "sanitizer_manifest.json"
RESULT_JSON = REPO_ROOT / "target" / "verification" / "areas" / "runtime-platform-results.json"


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite", action="append", default=[], help="Suite filter; can repeat.")
    parser.add_argument("--bless", action="store_true", help="Accepted for area runner parity; unused.")
    parser.add_argument(
        "--result-json",
        default=str(RESULT_JSON.relative_to(REPO_ROOT)),
        help="Path for machine-readable runtime platform result summary.",
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
        raise SystemExit("runtime_platform area does not support --bless")
    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    selected = select_suites(manifest, set(args.suite))

    print("Running runtime platform verification area", flush=True)
    print(f"  manifest={MANIFEST_PATH.relative_to(REPO_ROOT)}", flush=True)
    print("  bless=no", flush=True)

    suite_results = [run_suite(suite) for suite in selected]
    total_variants = sum(int(result["total_variants"]) for result in suite_results)
    total_failures = sum(int(result["total_failures"]) for result in suite_results)
    total_skips = sum(int(result["total_skips"]) for result in suite_results)
    payload = {
        "schema_version": 1,
        "area": "runtime_platform",
        "bless": False,
        "manifest": str(MANIFEST_PATH.relative_to(REPO_ROOT)),
        "suites": suite_results,
        "summary": {
            "total_variants": total_variants,
            "total_failures": total_failures,
            "blocking_failures": total_failures,
            "non_blocking_failures": 0,
            "skipped": total_skips,
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
    prefix = "verification ok" if args.hardening_summary else "runtime platform verification ok"
    print(
        f"{prefix}: variants={total_variants}, failures={total_failures}, "
        f"blocking_failures=0, non_blocking_failures=0, skipped={total_skips}",
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
            raise SystemExit(f"unknown runtime_platform suite filter(s): {', '.join(missing)}")
    if not selected:
        raise SystemExit("no runtime_platform suites selected")
    return selected


def run_suite(suite: dict[str, Any]) -> dict[str, Any]:
    suite_name = str(suite["name"])
    if suite_name == "platform-golden":
        variants = run_platform_golden()
    elif suite_name == "platform-contract":
        variants = [run_contract_variant()]
    elif suite_name in {"sanitizer-smoke", "sanitizer-full"}:
        variants = run_sanitizer_suite(suite_name)
    else:
        raise SystemExit(f"unsupported runtime_platform suite: {suite_name}")
    failures = sum(1 for variant in variants if variant["status"] == "fail")
    skips = sum(1 for variant in variants if variant["status"] == "skip")
    case = suite["cases"][0]
    return {
        "name": suite_name,
        "owner": "runtime/platform",
        "blocking": True,
        "runner": "runtime-platform",
        "cases": [
            {
                "id": str(case["id"]),
                "entry": str(case["entry"]),
                "command": str(case["command"]),
                "variants": variants,
            }
        ],
        "failed_cases": 1 if failures else 0,
        "total_variants": len(variants),
        "total_failures": failures,
        "total_skips": skips,
    }


def run_contract_variant() -> dict[str, Any]:
    started = time.perf_counter()
    status = "pass"
    failures: list[str] = []
    try:
        payload = json.loads(PLATFORM_CONTRACT.read_text(encoding="utf-8"))
        if not isinstance(payload, dict):
            failures.append("platform contract must be a JSON object")
    except Exception as exc:  # noqa: BLE001 - validation result captures parse failure.
        failures.append(str(exc))
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    if failures:
        status = "fail"
        for failure in failures:
            print(f"[platform-contract] fail {failure}", file=sys.stderr, flush=True)
    print_case_timing("platform-contract", "platform-contract", elapsed_ms, status)
    return {
        "label": "platform-contract",
        "argv": ["json-validate", str(PLATFORM_CONTRACT.relative_to(REPO_ROOT))],
        "status": status,
        "mismatches": failures,
        "expected_exit_code": 0,
        "actual_exit_code": 0 if status == "pass" else 1,
        "duration_ms": round(elapsed_ms, 3),
    }


def run_platform_golden() -> list[dict[str, Any]]:
    manifest = json.loads(GOLDEN_MANIFEST.read_text(encoding="utf-8"))
    closed = {item.strip() for item in os.environ.get("SIFR_PLATFORM_CLOSED_MILESTONES", "").split(",") if item.strip()}
    variants = []
    passed = 0
    skipped = 0
    for entry in manifest.get("entries", []):
        variant = run_platform_entry(entry, closed)
        variants.append(variant)
        if variant["status"] == "skip":
            skipped += 1
        elif variant["status"] == "pass":
            passed += 1
    print(f"[platform-golden] summary pass={passed} skip={skipped}", flush=True)
    return variants


def run_platform_entry(entry: dict[str, Any], closed: set[str]) -> dict[str, Any]:
    program = str(entry["program"])
    missing = [milestone for milestone in entry.get("blocked_until", []) if milestone not in closed]
    if missing:
        print(f"[platform-golden] skip {program} blocked_until={','.join(missing)}", flush=True)
        return {
            "label": program,
            "argv": [str(entry.get("command", ""))],
            "status": "skip",
            "mismatches": [],
            "expected_exit_code": int(entry.get("expected_exit", 0)),
            "actual_exit_code": None,
            "duration_ms": 0.0,
            "blocked_until": missing,
        }

    started = time.perf_counter()
    result = subprocess.run(
        str(entry["command"]),
        cwd=REPO_ROOT,
        shell=True,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    combined = result.stdout + result.stderr
    expected_exit = int(entry.get("expected_exit", 0))
    failures = []
    if result.returncode != expected_exit:
        failures.append(f"exit={result.returncode} expected={expected_exit}")
    for needle in entry.get("expected_stdout_contains", []):
        if needle not in result.stdout:
            failures.append(f"missing stdout: {needle}")
    for needle in entry.get("expected_diagnostic_contains", []):
        if needle not in combined:
            failures.append(f"missing diagnostic: {needle}")
    status = "fail" if failures else "pass"
    if failures:
        print(f"[platform-golden] fail {program} {'; '.join(failures)}", file=sys.stderr, flush=True)
        print(combined, file=sys.stderr, flush=True)
    else:
        print(f"[platform-golden] pass {program}", flush=True)
    print_case_timing("platform-golden", program, elapsed_ms, status)
    return {
        "label": program,
        "argv": [str(entry["command"])],
        "status": status,
        "mismatches": failures,
        "expected_exit_code": expected_exit,
        "actual_exit_code": result.returncode,
        "duration_ms": round(elapsed_ms, 3),
    }


def run_sanitizer_suite(suite_name: str) -> list[dict[str, Any]]:
    manifest = load_sanitizer_manifest()
    host_triple = current_rust_host_triple()
    variants = []
    passed = 0
    skipped = 0
    for case in manifest["cases"]:
        if suite_name not in case["suites"]:
            continue
        variant = run_sanitizer_case(suite_name, case, host_triple)
        variants.append(variant)
        if variant["status"] == "skip":
            skipped += 1
        elif variant["status"] == "pass":
            passed += 1
    if not variants:
        raise SystemExit(f"sanitizer manifest has no cases for suite: {suite_name}")
    print(f"[{suite_name}] summary pass={passed} skip={skipped}", flush=True)
    return variants


def load_sanitizer_manifest() -> dict[str, Any]:
    try:
        payload = json.loads(SANITIZER_MANIFEST.read_text(encoding="utf-8"))
    except Exception as exc:  # noqa: BLE001 - surfaced as area failure.
        raise SystemExit(f"failed to read sanitizer manifest: {exc}") from exc
    if not isinstance(payload, dict):
        raise SystemExit("sanitizer manifest must be a JSON object")
    if payload.get("schema_version") != 1:
        raise SystemExit("sanitizer manifest schema_version must be 1")
    cases = payload.get("cases")
    if not isinstance(cases, list) or not cases:
        raise SystemExit("sanitizer manifest requires non-empty cases")
    seen: set[str] = set()
    for case in cases:
        validate_sanitizer_case(case, seen)
    return payload


def validate_sanitizer_case(case: object, seen: set[str]) -> None:
    if not isinstance(case, dict):
        raise SystemExit("sanitizer case must be a JSON object")
    allowed_keys = {
        "always_skip",
        "command",
        "env",
        "finding_promotion",
        "id",
        "required_rustup_components",
        "required_rustup_toolchains",
        "required_tools",
        "scope",
        "skip_reason",
        "suites",
        "supported_host_triples",
        "timeout_seconds",
        "tool",
    }
    unknown_keys = sorted(set(case).difference(allowed_keys))
    if unknown_keys:
        raise SystemExit(f"sanitizer case has unknown field(s): {', '.join(unknown_keys)}")
    case_id = required_string(case, "id")
    if case_id in seen:
        raise SystemExit(f"duplicate sanitizer case id: {case_id}")
    seen.add(case_id)
    suites = case.get("suites")
    if not isinstance(suites, list) or not suites:
        raise SystemExit(f"sanitizer case {case_id} requires non-empty suites")
    unknown = sorted(set(str(suite) for suite in suites).difference({"sanitizer-smoke", "sanitizer-full"}))
    if unknown:
        raise SystemExit(f"sanitizer case {case_id} has unknown suites: {', '.join(unknown)}")
    required_string(case, "scope")
    required_string(case, "tool")
    required_string(case, "skip_reason")
    command = case.get("command")
    if not isinstance(command, list) or not command or not all(isinstance(arg, str) for arg in command):
        raise SystemExit(f"sanitizer case {case_id} requires a non-empty string command list")
    for key in ("supported_host_triples", "required_tools", "required_rustup_toolchains"):
        value = case.get(key, [])
        if not isinstance(value, list) or not all(isinstance(item, str) for item in value):
            raise SystemExit(f"sanitizer case {case_id} field {key} must be a string list")
    env = case.get("env", {})
    if not isinstance(env, dict) or not all(isinstance(key, str) and isinstance(value, str) for key, value in env.items()):
        raise SystemExit(f"sanitizer case {case_id} env must be a string map")
    timeout = case.get("timeout_seconds")
    if not isinstance(timeout, int) or timeout < 0:
        raise SystemExit(f"sanitizer case {case_id} timeout_seconds must be a non-negative integer")
    always_skip = case.get("always_skip", False)
    if not isinstance(always_skip, bool):
        raise SystemExit(f"sanitizer case {case_id} always_skip must be a boolean")
    components = case.get("required_rustup_components", [])
    if not isinstance(components, list):
        raise SystemExit(f"sanitizer case {case_id} required_rustup_components must be a list")
    for component in components:
        if not isinstance(component, dict):
            raise SystemExit(f"sanitizer case {case_id} component entries must be objects")
        required_string(component, "toolchain")
        required_string(component, "component")


def required_string(payload: dict[str, Any], key: str) -> str:
    value = payload.get(key)
    if not isinstance(value, str) or not value:
        raise SystemExit(f"required string field missing: {key}")
    return value


def run_sanitizer_case(suite_name: str, case: dict[str, Any], host_triple: str) -> dict[str, Any]:
    case_id = str(case["id"])
    command = list(case["command"])
    started = time.perf_counter()
    skip_reasons = sanitizer_skip_reasons(case, host_triple)
    if skip_reasons:
        elapsed_ms = (time.perf_counter() - started) * 1000.0
        reason = "; ".join(skip_reasons)
        print(f"[{suite_name}] skip {case_id} reason={reason}", flush=True)
        print_case_timing(suite_name, case_id, elapsed_ms, "skip")
        return {
            "label": case_id,
            "argv": command,
            "status": "skip",
            "mismatches": [],
            "expected_exit_code": 0,
            "actual_exit_code": None,
            "duration_ms": round(elapsed_ms, 3),
            "host_triple": host_triple,
            "skip_reason": reason,
            "tool": str(case["tool"]),
            "scope": str(case["scope"]),
            "finding_promotion": str(case.get("finding_promotion", "")),
        }

    timeout_seconds = int(case["timeout_seconds"])
    env = os.environ.copy()
    env.update({str(key): str(value) for key, value in case.get("env", {}).items()})
    env.setdefault("CARGO_NET_OFFLINE", "true")
    try:
        result = subprocess.run(
            command,
            cwd=REPO_ROOT,
            env=env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=timeout_seconds,
            check=False,
        )
        elapsed_ms = (time.perf_counter() - started) * 1000.0
        failures = [] if result.returncode == 0 else [f"exit={result.returncode} expected=0"]
        status = "fail" if failures else "pass"
        if failures:
            print(f"[{suite_name}] fail {case_id} {'; '.join(failures)}", file=sys.stderr, flush=True)
            print(result.stdout + result.stderr, file=sys.stderr, flush=True)
        else:
            print(f"[{suite_name}] pass {case_id}", flush=True)
    except subprocess.TimeoutExpired as exc:
        elapsed_ms = (time.perf_counter() - started) * 1000.0
        failures = [f"timeout after {timeout_seconds}s"]
        status = "fail"
        print(f"[{suite_name}] fail {case_id} timeout after {timeout_seconds}s", file=sys.stderr, flush=True)
        if exc.stdout:
            print(exc.stdout, file=sys.stderr, flush=True)
        if exc.stderr:
            print(exc.stderr, file=sys.stderr, flush=True)
    print_case_timing(suite_name, case_id, elapsed_ms, status)
    return {
        "label": case_id,
        "argv": command,
        "status": status,
        "mismatches": failures,
        "expected_exit_code": 0,
        "actual_exit_code": 0 if status == "pass" else 1,
        "duration_ms": round(elapsed_ms, 3),
        "host_triple": host_triple,
        "tool": str(case["tool"]),
        "scope": str(case["scope"]),
        "finding_promotion": str(case.get("finding_promotion", "")),
    }


def sanitizer_skip_reasons(case: dict[str, Any], host_triple: str) -> list[str]:
    reasons = []
    if bool(case.get("always_skip", False)):
        reasons.append(str(case["skip_reason"]))
    supported_hosts = set(str(item) for item in case.get("supported_host_triples", []))
    if "*" not in supported_hosts and host_triple not in supported_hosts:
        reasons.append(f"host {host_triple} is not in supported_host_triples")
    missing_tools = [tool for tool in case.get("required_tools", []) if shutil.which(str(tool)) is None]
    if missing_tools:
        reasons.append("missing required tool(s): " + ", ".join(sorted(missing_tools)))
    missing_toolchains = [
        toolchain for toolchain in case.get("required_rustup_toolchains", []) if not rustup_toolchain_available(str(toolchain))
    ]
    if missing_toolchains:
        reasons.append("missing rustup toolchain(s): " + ", ".join(sorted(missing_toolchains)))
    missing_components = [
        f"{component['toolchain']}:{component['component']}"
        for component in case.get("required_rustup_components", [])
        if not rustup_component_available(str(component["toolchain"]), str(component["component"]))
    ]
    if missing_components:
        reasons.append("missing rustup component(s): " + ", ".join(sorted(missing_components)))
    return reasons


def rustup_toolchain_available(toolchain: str) -> bool:
    if shutil.which("rustup") is None:
        return False
    result = subprocess.run(
        ["rustup", "toolchain", "list"],
        cwd=REPO_ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    if result.returncode != 0:
        return False
    for line in result.stdout.splitlines():
        normalized = line.split()[0]
        if normalized == toolchain or normalized.startswith(f"{toolchain}-"):
            return True
    return False


def rustup_component_available(toolchain: str, component: str) -> bool:
    if shutil.which("rustup") is None or not rustup_toolchain_available(toolchain):
        return False
    result = subprocess.run(
        ["rustup", "component", "list", "--toolchain", toolchain],
        cwd=REPO_ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    if result.returncode != 0:
        return False
    return any(line.startswith(component) and "(installed)" in line for line in result.stdout.splitlines())


def current_rust_host_triple() -> str:
    result = subprocess.run(
        ["rustc", "-Vv"],
        cwd=REPO_ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    for line in result.stdout.splitlines():
        if line.startswith("host: "):
            return line.removeprefix("host: ").strip()
    return "unknown-host"


def print_case_timing(suite_name: str, label: str, elapsed_ms: float, status: str) -> None:
    print(
        f"[sifr-case-timing] bucket=runtime_platform case={timing_token(suite_name)}/{timing_token(label)} "
        f"elapsed_ms={int(elapsed_ms)} status={status}",
        flush=True,
    )


def timing_token(value: object) -> str:
    return "".join(char if char.isalnum() or char in "_.:/+-" else "_" for char in str(value))


if __name__ == "__main__":
    raise SystemExit(main())
