#!/usr/bin/env python3
"""Validate and execute runtime-platform support/evidence manifests."""

from __future__ import annotations

import argparse
import json
import locale
import os
import platform
import shutil
import signal
import socket
import subprocess
import sys
import tempfile
import time
import unicodedata
from pathlib import Path
from typing import Any, Callable

REPO_ROOT = Path(__file__).resolve().parents[4]
AREA_ROOT = Path(__file__).resolve().parents[1]
SUPPORTED_PLATFORMS = AREA_ROOT / "supported_platforms.json"
EVIDENCE_MANIFEST = AREA_ROOT / "platform_evidence_manifest.json"
DEFAULT_RESULT_JSON = REPO_ROOT / "target" / "verification" / "areas" / "runtime-platform-evidence-results.json"

HOST_STATUSES = {"supported", "host-limited", "unsupported"}
REQUIREMENTS = {"execute", "structured-skip", "not-required"}
TARGET_REQUIREMENTS = {"execute-on-matching-host", "structured-skip", "not-required"}
SUPPORTED_SUITES = {"platform-support-matrix", "platform-evidence"}
OS_NAMES = {"macos", "linux", "windows"}


class EvidenceFailure(Exception):
    """Raised when an executable evidence probe fails."""


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--suite",
        choices=sorted(SUPPORTED_SUITES),
        default="platform-evidence",
        help="Suite to execute.",
    )
    parser.add_argument("--self-test", action="store_true", help="Run fail-closed manifest mutation checks.")
    parser.add_argument("--json-out", default=str(DEFAULT_RESULT_JSON.relative_to(REPO_ROOT)))
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.self_test:
        run_self_test()
        return 0

    started = time.perf_counter()
    support = load_supported_platforms()
    evidence = load_evidence_manifest()
    host = current_host()
    host_row = host_support_row(support, host["triple"])
    if args.suite == "platform-support-matrix":
        variants = support_matrix_variants(support, evidence, host, host_row)
    else:
        variants = evidence_variants(evidence, host, host_row)
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    failures = sum(1 for variant in variants if variant["status"] == "fail")
    skips = sum(1 for variant in variants if variant["status"] == "skip")
    payload = {
        "schema_version": 1,
        "suite": args.suite,
        "host": host,
        "variants": variants,
        "summary": {
            "total_variants": len(variants),
            "total_failures": failures,
            "blocking_failures": failures,
            "non_blocking_failures": 0,
            "skipped": skips,
            "duration_ms": round(elapsed_ms, 3),
        },
    }
    result_path = REPO_ROOT / args.json_out
    result_path.parent.mkdir(parents=True, exist_ok=True)
    result_path.write_text(json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8")
    print(f"result_json={result_path.relative_to(REPO_ROOT)}", flush=True)
    if failures:
        print(
            f"runtime platform evidence failed: variants={len(variants)}, failures={failures}, "
            f"blocking_failures={failures}, skipped={skips}",
            file=sys.stderr,
            flush=True,
        )
        return 1
    print(
        f"runtime platform evidence ok: variants={len(variants)}, failures=0, "
        f"blocking_failures=0, non_blocking_failures=0, skipped={skips}",
        flush=True,
    )
    return 0


def load_supported_platforms() -> dict[str, Any]:
    payload = json.loads(SUPPORTED_PLATFORMS.read_text(encoding="utf-8"))
    validate_supported_platforms(payload)
    return payload


def load_evidence_manifest() -> dict[str, Any]:
    payload = json.loads(EVIDENCE_MANIFEST.read_text(encoding="utf-8"))
    validate_evidence_manifest(payload)
    return payload


def validate_supported_platforms(payload: object) -> None:
    if not isinstance(payload, dict):
        raise SystemExit("supported_platforms must be a JSON object")
    if payload.get("schema_version") != 1:
        raise SystemExit("supported_platforms schema_version must be 1")
    if payload.get("owner") != "runtime-platform":
        raise SystemExit("supported_platforms owner must be runtime-platform")
    policy = payload.get("policy")
    if not isinstance(policy, dict):
        raise SystemExit("supported_platforms policy must be an object")
    if policy.get("create_pr_merge_network") != "loopback-only":
        raise SystemExit("create-pr/merge network policy must be loopback-only")
    if policy.get("undeclared_host") != "fail":
        raise SystemExit("undeclared host policy must be fail")
    host_triples = payload.get("host_triples")
    if not isinstance(host_triples, list) or not host_triples:
        raise SystemExit("supported_platforms requires non-empty host_triples")
    seen_hosts: set[str] = set()
    for row in host_triples:
        validate_host_row(row, seen_hosts)
    target_triples = payload.get("target_triples")
    if not isinstance(target_triples, list) or not target_triples:
        raise SystemExit("supported_platforms requires non-empty target_triples")
    seen_targets: set[str] = set()
    for row in target_triples:
        validate_target_row(row, seen_targets)


def validate_host_row(row: object, seen: set[str]) -> None:
    if not isinstance(row, dict):
        raise SystemExit("host_triples entries must be objects")
    required_keys = {
        "allowed_skips",
        "arch",
        "evidence_suites",
        "merge_requirement",
        "nightly_requirement",
        "os",
        "status",
        "toolchain",
        "triple",
    }
    unknown = sorted(set(row).difference(required_keys))
    if unknown:
        raise SystemExit(f"host row has unknown field(s): {', '.join(unknown)}")
    triple = required_string(row, "triple")
    if triple in seen:
        raise SystemExit(f"duplicate host triple: {triple}")
    seen.add(triple)
    if required_string(row, "os") not in OS_NAMES:
        raise SystemExit(f"host row {triple} has unknown os")
    if required_string(row, "status") not in HOST_STATUSES:
        raise SystemExit(f"host row {triple} has unknown status")
    if required_string(row, "merge_requirement") not in REQUIREMENTS:
        raise SystemExit(f"host row {triple} has unknown merge requirement")
    if required_string(row, "nightly_requirement") not in REQUIREMENTS:
        raise SystemExit(f"host row {triple} has unknown nightly requirement")
    required_string(row, "arch")
    required_string(row, "toolchain")
    allowed_skips = row.get("allowed_skips")
    if not isinstance(allowed_skips, list) or not all(isinstance(item, str) and item for item in allowed_skips):
        raise SystemExit(f"host row {triple} allowed_skips must be a string list")
    evidence_suites = row.get("evidence_suites")
    if not isinstance(evidence_suites, list) or "platform-evidence" not in evidence_suites:
        raise SystemExit(f"host row {triple} must list platform-evidence")
    if row["status"] == "supported" and allowed_skips:
        raise SystemExit(f"supported host row {triple} must not carry allowed skips")
    if row["status"] == "supported" and row["merge_requirement"] != "execute":
        raise SystemExit(f"supported host row {triple} must execute in merge")
    if row["status"] == "host-limited" and row["merge_requirement"] != "structured-skip":
        raise SystemExit(f"host-limited row {triple} must structured-skip in merge")


def validate_target_row(row: object, seen: set[str]) -> None:
    if not isinstance(row, dict):
        raise SystemExit("target_triples entries must be objects")
    required_keys = {"merge_requirement", "nightly_requirement", "status", "triple"}
    unknown = sorted(set(row).difference(required_keys))
    if unknown:
        raise SystemExit(f"target row has unknown field(s): {', '.join(unknown)}")
    triple = required_string(row, "triple")
    if triple in seen:
        raise SystemExit(f"duplicate target triple: {triple}")
    seen.add(triple)
    if required_string(row, "merge_requirement") not in TARGET_REQUIREMENTS:
        raise SystemExit(f"target row {triple} has unknown merge requirement")
    if required_string(row, "nightly_requirement") not in TARGET_REQUIREMENTS:
        raise SystemExit(f"target row {triple} has unknown nightly requirement")
    required_string(row, "status")


def validate_evidence_manifest(payload: object) -> None:
    if not isinstance(payload, dict):
        raise SystemExit("platform evidence manifest must be a JSON object")
    if payload.get("schema_version") != 1:
        raise SystemExit("platform evidence manifest schema_version must be 1")
    network_policy = payload.get("network_policy")
    if not isinstance(network_policy, dict):
        raise SystemExit("platform evidence network_policy must be an object")
    if network_policy.get("create_pr_merge") != "loopback-only":
        raise SystemExit("platform evidence create-pr/merge network must be loopback-only")
    if network_policy.get("external_network") != "forbidden":
        raise SystemExit("platform evidence external network must be forbidden")
    cases = payload.get("cases")
    if not isinstance(cases, list) or not cases:
        raise SystemExit("platform evidence requires non-empty cases")
    seen: set[str] = set()
    for case in cases:
        validate_evidence_case(case, seen)
    required = {
        "filesystem-paths",
        "unicode-paths",
        "symlink-roundtrip",
        "file-permissions",
        "tempdir-cleanup",
        "line-endings",
        "subprocess-exit-code",
        "subprocess-stdio",
        "loopback-networking",
        "signals-process-control",
        "locale-unicode-assumptions",
        "install-distribution-smoke",
    }
    missing = sorted(required.difference(seen))
    if missing:
        raise SystemExit(f"platform evidence missing required case(s): {', '.join(missing)}")


def validate_evidence_case(case: object, seen: set[str]) -> None:
    if not isinstance(case, dict):
        raise SystemExit("platform evidence cases must be objects")
    allowed_keys = {
        "allowed_skip_statuses",
        "command",
        "concern",
        "evidence",
        "id",
        "network",
        "skip_reason",
        "supported_os",
        "timeout_seconds",
    }
    unknown = sorted(set(case).difference(allowed_keys))
    if unknown:
        raise SystemExit(f"platform evidence case has unknown field(s): {', '.join(unknown)}")
    case_id = required_string(case, "id")
    if case_id in seen:
        raise SystemExit(f"duplicate platform evidence case id: {case_id}")
    seen.add(case_id)
    command = required_string(case, "command")
    if command not in BUILTINS:
        raise SystemExit(f"unknown platform evidence command for {case_id}: {command}")
    required_string(case, "concern")
    timeout = case.get("timeout_seconds")
    if not isinstance(timeout, int) or timeout <= 0:
        raise SystemExit(f"platform evidence case {case_id} timeout_seconds must be positive")
    supported_os = case.get("supported_os")
    if not isinstance(supported_os, list) or not supported_os:
        raise SystemExit(f"platform evidence case {case_id} requires supported_os")
    unknown_os = sorted(set(str(item) for item in supported_os).difference(OS_NAMES))
    if unknown_os:
        raise SystemExit(f"platform evidence case {case_id} has unknown supported_os: {', '.join(unknown_os)}")
    evidence = case.get("evidence")
    if not isinstance(evidence, list) or not all(isinstance(item, str) and item for item in evidence):
        raise SystemExit(f"platform evidence case {case_id} requires evidence strings")
    allowed_skip_statuses = case.get("allowed_skip_statuses")
    if not isinstance(allowed_skip_statuses, list) or not all(isinstance(item, str) for item in allowed_skip_statuses):
        raise SystemExit(f"platform evidence case {case_id} allowed_skip_statuses must be a string list")
    unknown_statuses = sorted(set(allowed_skip_statuses).difference(HOST_STATUSES))
    if unknown_statuses:
        raise SystemExit(f"platform evidence case {case_id} has unknown allowed skip status")
    if allowed_skip_statuses and not case.get("skip_reason"):
        raise SystemExit(f"platform evidence case {case_id} needs skip_reason")
    if case.get("network") not in {None, "loopback-only"}:
        raise SystemExit(f"platform evidence case {case_id} has illegal network policy")


def support_matrix_variants(
    support: dict[str, Any],
    evidence: dict[str, Any],
    host: dict[str, str],
    host_row: dict[str, Any],
) -> list[dict[str, Any]]:
    variants = [
        pass_variant("supported-platforms-schema", ["json-validate", str(SUPPORTED_PLATFORMS)]),
        pass_variant(
            "current-host-declared",
            ["rustc", "-Vv"],
            {
                "host_triple": host["triple"],
                "host_status": str(host_row["status"]),
                "merge_requirement": str(host_row["merge_requirement"]),
            },
        ),
        pass_variant(
            "target-platforms-declared",
            ["json-validate", str(SUPPORTED_PLATFORMS)],
            {"target_count": len(support["target_triples"])},
        ),
        pass_variant(
            "merge-network-policy-loopback-only",
            ["json-validate", str(EVIDENCE_MANIFEST)],
            {"network_policy": evidence["network_policy"]["create_pr_merge"]},
        ),
    ]
    for variant in variants:
        print_case_timing("platform-support-matrix", str(variant["label"]), 0.0, str(variant["status"]))
    return variants


def evidence_variants(
    evidence: dict[str, Any],
    host: dict[str, str],
    host_row: dict[str, Any],
) -> list[dict[str, Any]]:
    variants = []
    for case in evidence["cases"]:
        variants.append(run_evidence_case(case, host, host_row))
    return variants


def run_evidence_case(case: dict[str, Any], host: dict[str, str], host_row: dict[str, Any]) -> dict[str, Any]:
    case_id = str(case["id"])
    command = str(case["command"])
    argv = [command]
    host_status = str(host_row["status"])
    if host["os"] not in set(case["supported_os"]):
        if host_status in set(case["allowed_skip_statuses"]):
            reason = str(case.get("skip_reason", f"host os {host['os']} is not supported for this case"))
            print(f"[platform-evidence] skip {case_id} reason={reason}", flush=True)
            print_case_timing("platform-evidence", case_id, 0.0, "skip")
            return skip_variant(case_id, argv, reason, host)
        return fail_variant(case_id, argv, [f"host os {host['os']} is not supported and skip is not allowed"], host)

    started = time.perf_counter()
    try:
        run_and_check_duration(BUILTINS[command], int(case["timeout_seconds"]))
        elapsed_ms = (time.perf_counter() - started) * 1000.0
        print(f"[platform-evidence] pass {case_id}", flush=True)
        print_case_timing("platform-evidence", case_id, elapsed_ms, "pass")
        return pass_variant(
            case_id,
            argv,
            {
                "host_triple": host["triple"],
                "host_os": host["os"],
                "concern": str(case["concern"]),
                "evidence": list(case["evidence"]),
            },
            elapsed_ms=elapsed_ms,
        )
    except EvidenceFailure as exc:
        elapsed_ms = (time.perf_counter() - started) * 1000.0
        print(f"[platform-evidence] fail {case_id} {exc}", file=sys.stderr, flush=True)
        print_case_timing("platform-evidence", case_id, elapsed_ms, "fail")
        return fail_variant(case_id, argv, [str(exc)], host, elapsed_ms=elapsed_ms)
    except subprocess.TimeoutExpired as exc:
        elapsed_ms = (time.perf_counter() - started) * 1000.0
        failure = f"subprocess timeout after {exc.timeout}s"
        print(f"[platform-evidence] fail {case_id} {failure}", file=sys.stderr, flush=True)
        print_case_timing("platform-evidence", case_id, elapsed_ms, "fail")
        return fail_variant(case_id, argv, [failure], host, elapsed_ms=elapsed_ms)
    except Exception as exc:  # noqa: BLE001 - platform probes report host errors as evidence failures.
        elapsed_ms = (time.perf_counter() - started) * 1000.0
        failure = f"{type(exc).__name__}: {exc}"
        print(f"[platform-evidence] fail {case_id} {failure}", file=sys.stderr, flush=True)
        print_case_timing("platform-evidence", case_id, elapsed_ms, "fail")
        return fail_variant(case_id, argv, [failure], host, elapsed_ms=elapsed_ms)


def run_and_check_duration(callback: Callable[[], None], timeout_seconds: int) -> None:
    started = time.monotonic()
    callback()
    elapsed = time.monotonic() - started
    if elapsed > timeout_seconds:
        raise EvidenceFailure(f"case exceeded timeout: {elapsed:.3f}s > {timeout_seconds}s")


def check_filesystem_paths() -> None:
    with tempfile.TemporaryDirectory() as temp:
        root = Path(temp)
        nested = root / "nested dir" / "child"
        nested.mkdir(parents=True)
        target = nested / "payload.bin"
        target.write_bytes(b"path-evidence")
        if not target.is_absolute():
            raise EvidenceFailure("temporary target is not absolute")
        if os.sep not in str(target):
            raise EvidenceFailure("native separator absent from absolute path")
        relative = target.relative_to(root)
        if (root / relative).read_bytes() != b"path-evidence":
            raise EvidenceFailure("relative path readback failed")


def check_unicode_paths() -> None:
    with tempfile.TemporaryDirectory() as temp:
        path = Path(temp) / "sifr-unicode-\u00e5-\u03bb-\U0001f642.txt"
        payload = "unicode-path-evidence".encode()
        path.write_bytes(payload)
        if path.read_bytes() != payload:
            raise EvidenceFailure("Unicode path payload mismatch")


def check_symlink_roundtrip() -> None:
    with tempfile.TemporaryDirectory() as temp:
        root = Path(temp)
        target = root / "target.txt"
        link = root / "link.txt"
        target.write_text("symlink-evidence", encoding="utf-8")
        link.symlink_to(target)
        if not link.is_symlink():
            raise EvidenceFailure("created path is not a symlink")
        if link.read_text(encoding="utf-8") != "symlink-evidence":
            raise EvidenceFailure("symlink readback failed")
        if link.resolve() != target.resolve():
            raise EvidenceFailure("symlink target resolution mismatch")


def check_file_permissions() -> None:
    with tempfile.TemporaryDirectory() as temp:
        path = Path(temp) / "permissions.txt"
        path.write_text("permissions", encoding="utf-8")
        path.chmod(0o600)
        observed = path.stat().st_mode & 0o777
        if observed != 0o600:
            raise EvidenceFailure(f"chmod mode mismatch: {oct(observed)}")


def check_tempdir_cleanup() -> None:
    marker: Path
    with tempfile.TemporaryDirectory() as temp:
        marker = Path(temp)
        (marker / "payload.txt").write_text("cleanup", encoding="utf-8")
        if not marker.exists():
            raise EvidenceFailure("tempdir was not created")
    if marker.exists():
        raise EvidenceFailure("tempdir still exists after cleanup")


def check_line_endings() -> None:
    with tempfile.TemporaryDirectory() as temp:
        path = Path(temp) / "line-endings.txt"
        payload = b"one\n two\r\nthree\r\n"
        path.write_bytes(payload)
        if path.read_bytes() != payload:
            raise EvidenceFailure("line ending bytes changed on binary readback")


def check_subprocess_exit_code() -> None:
    result = subprocess.run(
        [sys.executable, "-c", "import sys; print('platform-subprocess'); sys.exit(7)"],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if result.returncode != 7:
        raise EvidenceFailure(f"unexpected subprocess exit: {result.returncode}")
    if "platform-subprocess" not in result.stdout:
        raise EvidenceFailure("stdout capture missing")


def check_subprocess_stdio() -> None:
    script = "import sys; data=sys.stdin.buffer.read(); sys.stdout.buffer.write(data[::-1]); sys.stderr.write('stderr-ok')"
    result = subprocess.run(
        [sys.executable, "-c", script],
        input=b"abc123",
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if result.returncode != 0:
        raise EvidenceFailure(f"stdio subprocess failed: {result.returncode}")
    if result.stdout != b"321cba":
        raise EvidenceFailure("stdout byte payload mismatch")
    if b"stderr-ok" not in result.stderr:
        raise EvidenceFailure("stderr capture missing")


def check_loopback_networking() -> None:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as server:
        server.bind(("127.0.0.1", 0))
        server.listen(1)
        port = int(server.getsockname()[1])
        with socket.create_connection(("127.0.0.1", port), timeout=2.0) as client:
            conn, addr = server.accept()
            with conn:
                if addr[0] != "127.0.0.1":
                    raise EvidenceFailure(f"non-loopback peer: {addr[0]}")
                client.sendall(b"ping")
                if conn.recv(4) != b"ping":
                    raise EvidenceFailure("server did not receive loopback payload")
                conn.sendall(b"pong")
                if client.recv(4) != b"pong":
                    raise EvidenceFailure("client did not receive loopback payload")


def check_signals_process_control() -> None:
    if platform.system().lower() == "windows":
        raise EvidenceFailure("POSIX signal evidence is not available on Windows")
    proc = subprocess.Popen([sys.executable, "-c", "import time; time.sleep(30)"])
    try:
        os.kill(proc.pid, signal.SIGTERM)
        exit_code = proc.wait(timeout=5)
    finally:
        if proc.poll() is None:
            proc.kill()
            proc.wait(timeout=5)
    if exit_code not in {-signal.SIGTERM, 128 + signal.SIGTERM, 143}:
        raise EvidenceFailure(f"unexpected SIGTERM exit status: {exit_code}")


def check_locale_unicode_assumptions() -> None:
    encoding = locale.getpreferredencoding(False)
    if not encoding:
        raise EvidenceFailure("preferred encoding is empty")
    text = "Cafe\u0301"
    normalized = unicodedata.normalize("NFC", text)
    if normalized != "Caf\u00e9":
        raise EvidenceFailure("Unicode normalization mismatch")
    if normalized.encode("utf-8").decode("utf-8") != normalized:
        raise EvidenceFailure("explicit UTF-8 round trip failed")


def check_install_distribution_smoke() -> None:
    if shutil.which("cargo") is None:
        raise EvidenceFailure("cargo is not available")
    result = subprocess.run(
        ["cargo", "run", "--locked", "-q", "-p", "sifr", "--", "--help"],
        cwd=REPO_ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=60,
        check=False,
    )
    if result.returncode != 0:
        raise EvidenceFailure(f"sifr help failed: exit={result.returncode}")
    combined = result.stdout + result.stderr
    if "Usage" not in combined and "Commands" not in combined:
        raise EvidenceFailure("sifr help output did not render expected text")


BUILTINS: dict[str, Callable[[], None]] = {
    "builtin:filesystem-paths": check_filesystem_paths,
    "builtin:unicode-paths": check_unicode_paths,
    "builtin:symlink-roundtrip": check_symlink_roundtrip,
    "builtin:file-permissions": check_file_permissions,
    "builtin:tempdir-cleanup": check_tempdir_cleanup,
    "builtin:line-endings": check_line_endings,
    "builtin:subprocess-exit-code": check_subprocess_exit_code,
    "builtin:subprocess-stdio": check_subprocess_stdio,
    "builtin:loopback-networking": check_loopback_networking,
    "builtin:signals-process-control": check_signals_process_control,
    "builtin:locale-unicode-assumptions": check_locale_unicode_assumptions,
    "builtin:install-distribution-smoke": check_install_distribution_smoke,
}


def current_host() -> dict[str, str]:
    triple = current_rust_host_triple()
    system = platform.system().lower()
    os_name = {"darwin": "macos"}.get(system, system)
    return {
        "triple": triple,
        "os": os_name,
        "arch": platform.machine().lower(),
    }


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
    raise SystemExit("unable to determine rust host triple from rustc -Vv")


def host_support_row(support: dict[str, Any], triple: str) -> dict[str, Any]:
    for row in support["host_triples"]:
        if row["triple"] == triple:
            return row
    raise SystemExit(f"current host triple is not declared in supported_platforms.json: {triple}")


def required_string(payload: dict[str, Any], key: str) -> str:
    value = payload.get(key)
    if not isinstance(value, str) or not value:
        raise SystemExit(f"required string field missing: {key}")
    return value


def pass_variant(
    label: str,
    argv: list[str],
    metadata: dict[str, Any] | None = None,
    *,
    elapsed_ms: float = 0.0,
) -> dict[str, Any]:
    variant = {
        "label": label,
        "argv": argv,
        "status": "pass",
        "mismatches": [],
        "expected_exit_code": 0,
        "actual_exit_code": 0,
        "duration_ms": round(elapsed_ms, 3),
    }
    if metadata:
        variant.update(metadata)
    return variant


def skip_variant(label: str, argv: list[str], reason: str, host: dict[str, str]) -> dict[str, Any]:
    return {
        "label": label,
        "argv": argv,
        "status": "skip",
        "mismatches": [],
        "expected_exit_code": 0,
        "actual_exit_code": None,
        "duration_ms": 0.0,
        "host_triple": host["triple"],
        "skip_reason": reason,
    }


def fail_variant(
    label: str,
    argv: list[str],
    failures: list[str],
    host: dict[str, str],
    *,
    elapsed_ms: float = 0.0,
) -> dict[str, Any]:
    return {
        "label": label,
        "argv": argv,
        "status": "fail",
        "mismatches": failures,
        "expected_exit_code": 0,
        "actual_exit_code": 1,
        "duration_ms": round(elapsed_ms, 3),
        "host_triple": host["triple"],
    }


def print_case_timing(suite_name: str, label: str, elapsed_ms: float, status: str) -> None:
    print(
        f"[sifr-case-timing] bucket=runtime_platform case={timing_token(suite_name)}/{timing_token(label)} "
        f"elapsed_ms={int(elapsed_ms)} status={status}",
        flush=True,
    )


def timing_token(value: object) -> str:
    return "".join(char if char.isalnum() or char in "_.:/+-" else "_" for char in str(value))


def run_self_test() -> None:
    support = load_supported_platforms()
    evidence = load_evidence_manifest()
    mutations: list[tuple[str, Callable[[], None]]] = [
        ("supported host with skip", lambda: validate_supported_platforms(mutated_supported_host_skip(support))),
        ("external network allowed", lambda: validate_evidence_manifest(mutated_external_network(evidence))),
        ("missing required evidence", lambda: validate_evidence_manifest(mutated_missing_case(evidence))),
    ]
    for label, callback in mutations:
        try:
            callback()
        except SystemExit:
            print(f"[platform-evidence-self-test] pass {label}", flush=True)
        else:
            raise SystemExit(f"self-test mutation unexpectedly passed: {label}")


def clone_json(payload: dict[str, Any]) -> dict[str, Any]:
    return json.loads(json.dumps(payload))


def mutated_supported_host_skip(payload: dict[str, Any]) -> dict[str, Any]:
    mutated = clone_json(payload)
    mutated["host_triples"][0]["allowed_skips"] = ["unexpected skip"]
    return mutated


def mutated_external_network(payload: dict[str, Any]) -> dict[str, Any]:
    mutated = clone_json(payload)
    mutated["network_policy"]["create_pr_merge"] = "external"
    return mutated


def mutated_missing_case(payload: dict[str, Any]) -> dict[str, Any]:
    mutated = clone_json(payload)
    mutated["cases"] = [case for case in mutated["cases"] if case["id"] != "loopback-networking"]
    return mutated


if __name__ == "__main__":
    raise SystemExit(main())
