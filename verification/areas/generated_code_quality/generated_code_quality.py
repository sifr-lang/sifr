#!/usr/bin/env python3
"""Generated-code quality gates."""

from __future__ import annotations

import argparse
import dataclasses
import functools
import hashlib
import json
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path
from typing import Any, Iterable

REPO_ROOT = Path(__file__).resolve().parents[3]
GCQ_ROOT = REPO_ROOT / "verification" / "areas" / "generated_code_quality"
MANIFEST = GCQ_ROOT / "data" / "corpus_manifest.json"
QUALITY_DEBT = GCQ_ROOT / "data" / "generated_quality_debt.json"
TARGET_ROOT = REPO_ROOT / "target" / "sifr_generated_code_quality"
EVIDENCE_ROOT = TARGET_ROOT / "evidence"
sys.path.insert(0, str(REPO_ROOT / "verification" / "areas" / "common"))

from sifr_binary import resolve_sifr_binary  # noqa: E402
from inventory_gates import (  # noqa: E402
    gate_freshness as run_freshness_gate,
    gate_inventory as run_inventory_gate,
)
from quality_policy import (  # noqa: E402
    STRICT_CLIPPY_ARGS,
    assert_negative_pattern,
    compare_exact_debt,
    compact_clippy_summary,
    debt_selection_id,
    load_debt,
    merge_output_signatures,
    merge_signature_summaries,
    output_signature,
    parse_clippy_diagnostics,
    scan_files,
    validate_debt_owners,
    validate_clippy_lint_owners,
    violation_summary,
)
from source_quality_checks import (
    assert_negative_clippy,
    assert_negative_determinism,
    assert_negative_rustfmt,
    compare_bytes,
    gate_intrinsic_panic_lint as run_intrinsic_panic_lint_gate,
    run_strict_clippy,
)

# Inputs that change generated Rust or its compile environment. Manifest metadata
# remains part of per-entry selection, not producer cache invalidation.
PRODUCER_FINGERPRINT_CRATES = (
    "sifr", "sifr_codegen", "sifr_driver", "sifr_frontend", "sifr_ipc",
    "sifr_lowering", "sifr_package", "sifr_runtime", "sifr_stdlib",
    "sifr_stdlib_imports", "sifr_stdlib_manifest", "sifr_syntax",
)
PRODUCER_FINGERPRINT_INPUTS = [
    "Cargo.lock",
    "Cargo.toml",
    "stdlib",
    *[
        f"crates/{crate}/{path}"
        for crate in PRODUCER_FINGERPRINT_CRATES
        for path in ("Cargo.toml", "src")
    ],
]
PRODUCER_FINGERPRINT_EXTENSIONS = {".lock", ".py", ".rs", ".sifr", ".toml"}
PRODUCER_CACHE_SCHEMA = b"generated-code-quality-materialization-v2\0"

POSITIVE_GROUPS = {
    "concurrency-runtime-readiness",
    "demos-required",
    "e2e-pass-representative",
    "multi-module-projects",
    "stdlib-flows",
}
NEGATIVE_GROUP = "negative-seeds"
REQUIRED_GROUP_COUNTS = {
    "concurrency-runtime-readiness": 7,
    "e2e-pass-representative": 50,
    "stdlib-flows": 10,
    "multi-module-projects": 5,
    "demos-required": 6,
    "negative-seeds": 5,
}
REQUIRED_DEMOS = {
    "demos/codegen_output/main.sifr",
    "demos/codegen_structural_passes/main.sifr",
    "demos/cargo_manifest/main.sifr",
    "demos/dependency_manifest/main.sifr",
    "demos/additional_modules/main.sifr",
}
ASYNC_DEMOS = {
    "demos/async_generator_comprehension_demo/main.sifr",
    "demos/blocking_offload_demo/main.sifr",
}
CONCURRENCY_READINESS_DEMOS = {
    "demos/async_subprocess_pipeline_demo/main.sifr",
    "demos/blocking_offload_demo/main.sifr",
    "demos/cancellation_cleanup_demo/main.sifr",
    "demos/parallel_map_demo/main.sifr",
    "demos/structured_concurrency_demo/main.sifr",
    "demos/structured_shutdown_demo/main.sifr",
    "demos/sync_channel_demo/main.sifr",
}
@dataclasses.dataclass(frozen=True)
class Entry:
    id: str
    group: str
    source_path: str
    expected_command: str
    evidence_category: str

    @property
    def absolute_source(self) -> Path:
        return REPO_ROOT / self.source_path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "mode",
        choices=(
            "companions",
            "corpus",
            "inventory",
            "panic-scan",
            "rustfmt",
            "clippy",
            "determinism",
            "demos",
            "freshness",
            "intrinsic-panic-lint",
        ),
    )
    parser.add_argument(
        "--manifest",
        default=str(MANIFEST),
        help="Generated-code quality manifest path.",
    )
    parser.add_argument(
        "--group",
        action="append",
        default=[],
        help="Limit positive entries to one or more groups.",
    )
    parser.add_argument(
        "--keep-success",
        action="store_true",
        default=os.environ.get("SIFR_GCQ_KEEP_SUCCESS") == "1",
        help="Keep successful transient generated projects.",
    )
    return parser.parse_args()


def load_manifest(path: Path) -> list[Entry]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict) or payload.get("version") != 1:
        raise SystemExit(f"invalid generated-code quality manifest header: {path}")
    raw_entries = payload.get("entries")
    if not isinstance(raw_entries, list):
        raise SystemExit("invalid generated-code quality manifest: entries must be a list")

    entries: list[Entry] = []
    ids: list[str] = []
    for raw in raw_entries:
        if not isinstance(raw, dict):
            raise SystemExit("invalid generated-code quality manifest: entry must be an object")
        missing = {
            key
            for key in ("id", "group", "source_path", "expected_command", "evidence_category")
            if not isinstance(raw.get(key), str) or not raw.get(key)
        }
        if missing:
            raise SystemExit(f"invalid manifest entry {raw!r}: missing {sorted(missing)}")
        entry = Entry(
            id=raw["id"],
            group=raw["group"],
            source_path=raw["source_path"],
            expected_command=raw["expected_command"],
            evidence_category=raw["evidence_category"],
        )
        entries.append(entry)
        ids.append(entry.id)

    if ids != sorted(ids):
        raise SystemExit("manifest entries must be sorted lexicographically by stable id")
    if len(ids) != len(set(ids)):
        raise SystemExit("manifest entry ids must be unique")

    counts: dict[str, int] = {}
    for entry in entries:
        if entry.group not in POSITIVE_GROUPS and entry.group != NEGATIVE_GROUP:
            raise SystemExit(f"{entry.id}: unsupported group {entry.group!r}")
        if not entry.absolute_source.is_file():
            raise SystemExit(f"{entry.id}: source path does not exist: {entry.source_path}")
        counts[entry.group] = counts.get(entry.group, 0) + 1

    for group, minimum in REQUIRED_GROUP_COUNTS.items():
        actual = counts.get(group, 0)
        if actual < minimum:
            raise SystemExit(f"manifest group {group!r} has {actual} entries, need >= {minimum}")

    demo_paths = {entry.source_path for entry in entries if entry.group == "demos-required"}
    missing_demos = REQUIRED_DEMOS - demo_paths
    if missing_demos:
        raise SystemExit(f"missing required generated-code demos: {sorted(missing_demos)}")
    if not demo_paths.intersection(ASYNC_DEMOS):
        raise SystemExit("manifest must include one supported async/concurrency demo")
    concurrency_demo_paths = {entry.source_path for entry in entries if entry.group == "concurrency-runtime-readiness"}
    missing_concurrency_demos = CONCURRENCY_READINESS_DEMOS - concurrency_demo_paths
    if missing_concurrency_demos:
        raise SystemExit(f"missing concurrency generated-code demos: {sorted(missing_concurrency_demos)}")

    return entries


def selected_positive_entries(entries: list[Entry], groups: Iterable[str]) -> list[Entry]:
    explicit_ids = explicit_entry_ids()
    if explicit_ids:
        if os.environ.get("SIFR_GCQ_MAX_ENTRIES"):
            raise SystemExit("SIFR_GCQ_ENTRY_IDS cannot be combined with SIFR_GCQ_MAX_ENTRIES")
        if list(groups):
            raise SystemExit("SIFR_GCQ_ENTRY_IDS cannot be combined with --group filters")
        entries_by_id = {entry.id: entry for entry in entries}
        missing = [entry_id for entry_id in explicit_ids if entry_id not in entries_by_id]
        if missing:
            raise SystemExit(f"unknown SIFR_GCQ_ENTRY_IDS entries: {missing}")
        selected = [entries_by_id[entry_id] for entry_id in explicit_ids]
        non_positive = [entry.id for entry in selected if entry.group not in POSITIVE_GROUPS]
        if non_positive:
            raise SystemExit(f"SIFR_GCQ_ENTRY_IDS must select positive entries: {non_positive}")
        return selected

    selected_groups = set(groups)
    limit_raw = os.environ.get("SIFR_GCQ_MAX_ENTRIES")
    limit = int(limit_raw) if limit_raw else 0
    selected = [
        entry
        for entry in entries
        if entry.group in POSITIVE_GROUPS and (not selected_groups or entry.group in selected_groups)
    ]
    if limit > 0:
        selected = selected[:limit]
    if not selected:
        raise SystemExit("no positive manifest entries selected")
    return selected


def explicit_entry_ids() -> list[str]:
    raw = os.environ.get("SIFR_GCQ_ENTRY_IDS", "")
    if not raw.strip():
        return []
    ids = [entry_id.strip() for entry_id in raw.split(",")]
    if any(not entry_id for entry_id in ids):
        raise SystemExit("SIFR_GCQ_ENTRY_IDS contains an empty entry id")
    duplicates = sorted({entry_id for entry_id in ids if ids.count(entry_id) > 1})
    if duplicates:
        raise SystemExit(f"SIFR_GCQ_ENTRY_IDS contains duplicate entries: {duplicates}")
    return ids


def selection_id(entries: list[Entry]) -> str:
    return debt_selection_id(entry.id for entry in entries)


def run_id(mode: str) -> str:
    return f"{mode}-{int(time.time())}-{os.getpid()}"


def command_env() -> dict[str, str]:
    return os.environ.copy()


def shared_artifact_root() -> Path | None:
    raw = os.environ.get("SIFR_GCQ_SHARED_ROOT")
    if not raw:
        return None
    return (REPO_ROOT / raw).resolve() if not Path(raw).is_absolute() else Path(raw)


def producer_fingerprint_files() -> list[Path]:
    files: list[Path] = []
    for relative in PRODUCER_FINGERPRINT_INPUTS:
        path = REPO_ROOT / relative
        if path.is_file():
            files.append(path)
        elif path.is_dir():
            files.extend(
                candidate
                for candidate in path.rglob("*")
                if candidate.is_file() and candidate.suffix in PRODUCER_FINGERPRINT_EXTENSIONS
            )
    return sorted(files)


@functools.cache
def producer_fingerprint() -> str:
    digest = hashlib.sha256()
    digest.update(PRODUCER_CACHE_SCHEMA)
    for path in producer_fingerprint_files():
        digest.update(path.relative_to(REPO_ROOT).as_posix().encode("utf-8"))
        digest.update(b"\0")
        digest.update(path.read_bytes())
        digest.update(b"\0")
    return digest.hexdigest()[:16]


def entry_cache_key(entry: Entry) -> str:
    digest = hashlib.sha256()
    digest.update(producer_fingerprint().encode("utf-8"))
    digest.update(b"\0")
    digest.update(entry.id.encode("utf-8"))
    digest.update(b"\0")
    digest.update(entry.source_path.encode("utf-8"))
    digest.update(b"\0")
    digest.update(entry.absolute_source.read_bytes())
    return digest.hexdigest()[:16]


def run_command(
    args: list[str],
    *,
    cwd: Path = REPO_ROOT,
    check: bool = True,
    cargo_target_dir: Path | None = None,
) -> subprocess.CompletedProcess[str]:
    env = command_env()
    shared_root = shared_artifact_root()
    if cargo_target_dir is not None:
        env["CARGO_TARGET_DIR"] = str(cargo_target_dir)
    elif shared_root is not None:
        env["CARGO_TARGET_DIR"] = str(shared_root / "cargo-target")
    result = subprocess.run(
        args,
        cwd=cwd,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if check and result.returncode != 0:
        command = " ".join(args)
        raise RuntimeError(
            f"command failed ({result.returncode}): {command}\n"
            f"--- stdout ---\n{result.stdout}\n--- stderr ---\n{result.stderr}"
        )
    return result


@functools.cache
def sifr_binary() -> str:
    default = REPO_ROOT / "target" / "debug" / "sifr"
    return str(resolve_sifr_binary(REPO_ROOT, explicit_env_var="SIFR_GCQ_BIN", default_binary=default))


def materialize_entry(entry: Entry, run_root: Path) -> Path:
    run_root.mkdir(parents=True, exist_ok=True)
    shared_root = shared_artifact_root()
    cache_key = entry_cache_key(entry)
    entry_root = (
        shared_root / "entries" / f"{entry.id}-{cache_key}"
        if shared_root is not None
        else run_root / entry.id
    )
    entry_root.mkdir(parents=True, exist_ok=True)
    crate_root = entry_root / "sifr_output"
    if (crate_root / "Cargo.toml").is_file():
        print(
            f"[sifr-artifact-cache] namespace=generated-code-quality key={cache_key} "
            f"cache_hit=true workspace={crate_root}"
        )
    else:
        print(
            f"[sifr-artifact-cache] namespace=generated-code-quality key={cache_key} "
            f"cache_hit=false workspace={crate_root} miss_reason=not_materialized"
        )
        run_command(
            [
                sifr_binary(),
                "build",
                entry.source_path,
                "-o",
                str(entry_root),
                "--materialize-only",
            ]
        )
    cargo_toml = crate_root / "Cargo.toml"
    if not cargo_toml.is_file():
        raise RuntimeError(f"{entry.id}: generated crate missing Cargo.toml at {cargo_toml}")
    return crate_root


def rust_files(crate_root: Path) -> list[Path]:
    return sorted((crate_root / "src").rglob("*.rs"))


def emit_source(entry: Entry) -> bytes:
    result = run_command(
        [sifr_binary(), "emit", entry.source_path],
    )
    return result.stdout.encode("utf-8")


def record_evidence(mode: str, run: str, records: list[dict[str, Any]]) -> Path:
    EVIDENCE_ROOT.mkdir(parents=True, exist_ok=True)
    evidence = {
        "mode": mode,
        "run_id": run,
        "records": records,
    }
    path = EVIDENCE_ROOT / f"{run}.json"
    path.write_text(json.dumps(evidence, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return path


def record_for_entry(entry: Entry, crate_root: Path | None, status: str) -> dict[str, Any]:
    record: dict[str, Any] = {
        "id": entry.id,
        "group": entry.group,
        "source_path": entry.source_path,
        "evidence_category": entry.evidence_category,
        "status": status,
    }
    if crate_root is not None:
        files = rust_files(crate_root)
        digest = hashlib.sha256()
        for path in files:
            digest.update(path.relative_to(crate_root).as_posix().encode("utf-8"))
            digest.update(b"\0")
            digest.update(path.read_bytes())
            digest.update(b"\0")
        record["crate_root"] = str(crate_root.relative_to(REPO_ROOT))
        record["rust_file_count"] = len(files)
        record["source_sha256"] = digest.hexdigest()
    return record


def timed_case(bucket: str, case_id: str, action: Any) -> Any:
    started = time.perf_counter()
    status = "pass"
    try:
        return action()
    except Exception:
        status = "fail"
        raise
    finally:
        elapsed_ms = int((time.perf_counter() - started) * 1000.0)
        print(
            f"[sifr-case-timing] bucket={bucket} case={case_id} "
            f"elapsed_ms={elapsed_ms} status={status}"
        )


def gate_inventory(_entries: list[Entry], _args: argparse.Namespace) -> None:
    run_inventory_gate(timed_case, record_evidence, run_id)


def gate_freshness(_entries: list[Entry], _args: argparse.Namespace) -> None:
    run_freshness_gate(timed_case, record_evidence, run_id, run_command, sifr_binary)


def gate_corpus(entries: list[Entry], args: argparse.Namespace) -> None:
    run = run_id("corpus")
    run_root = TARGET_ROOT / run
    records = []
    try:
        for entry in selected_positive_entries(entries, args.group):
            def check_entry() -> Path:
                crate_root_inner = materialize_entry(entry, run_root)
                run_command(["cargo", "check", "--locked", "--manifest-path", str(crate_root_inner / "Cargo.toml")])
                return crate_root_inner

            crate_root = timed_case("generated_code_quality", f"corpus/{entry.id}", check_entry)
            records.append(record_for_entry(entry, crate_root, "passed"))
        evidence = record_evidence("corpus", run, records)
        print(f"generated-code corpus passed; evidence={evidence.relative_to(REPO_ROOT)}")
    except Exception:
        print(f"generated-code corpus failed; preserved={run_root}", file=sys.stderr)
        raise
    else:
        if not args.keep_success:
            shutil.rmtree(run_root, ignore_errors=True)


def gate_panic_scan(entries: list[Entry], args: argparse.Namespace) -> None:
    run = run_id("panic-scan")
    run_root = TARGET_ROOT / run
    records = []
    try:
        negative_seeds = {
            "unwrap": "forbidden_unwrap.rs",
            "expect": "forbidden_expect.rs",
            "panic": "forbidden_panic.rs",
            "todo": "forbidden_todo.rs",
            "unimplemented": "forbidden_unimplemented.rs",
            "unsafe": "forbidden_unsafe.rs",
            "unreachable": "forbidden_unreachable.rs",
            "process-abort": "forbidden_abort.rs",
            "process-exit": "forbidden_exit.rs",
            "direct-index": "forbidden_index.rs",
            "signed-to-usize": "forbidden_signed_to_usize.rs",
            "allow-attribute": "forbidden_allow.rs",
        }
        for policy_id, filename in negative_seeds.items():
            timed_case(
                "generated_code_quality",
                f"panic-scan/negative-{policy_id}",
                lambda policy_id=policy_id, filename=filename: assert_negative_pattern(
                    GCQ_ROOT / "negative_seeds" / filename,
                    policy_id,
                ),
            )
        debt = load_debt(QUALITY_DEBT)
        validate_debt_owners(debt)
        selected = selected_positive_entries(entries, args.group)
        summaries = []
        for entry in selected:
            def scan_entry() -> Path:
                crate_root_inner = materialize_entry(entry, run_root)
                actual = violation_summary(
                    scan_files(rust_files(crate_root_inner), crate_root_inner)
                )
                summaries.append((entry.id, actual))
                return crate_root_inner

            crate_root = timed_case("generated_code_quality", f"panic-scan/{entry.id}", scan_entry)
            records.append(record_for_entry(entry, crate_root, "passed"))
        compare_exact_debt(
            category="safety",
            entry_id=selection_id(selected),
            actual=merge_signature_summaries(summaries),
            debt=debt,
        )
        evidence = record_evidence("panic-scan", run, records)
        print(f"generated-code panic scan passed; evidence={evidence.relative_to(REPO_ROOT)}")
    except Exception:
        print(f"generated-code panic scan failed; preserved={run_root}", file=sys.stderr)
        raise
    else:
        if not args.keep_success:
            shutil.rmtree(run_root, ignore_errors=True)


def gate_rustfmt(entries: list[Entry], args: argparse.Namespace) -> None:
    run = run_id("rustfmt")
    run_root = TARGET_ROOT / run
    records = []
    try:
        timed_case(
            "generated_code_quality",
            "rustfmt/negative-format-violation",
            lambda: assert_negative_rustfmt(
                GCQ_ROOT / "negative_seeds" / "format_violation.rs",
                run_root,
                run_command,
            ),
        )
        debt = load_debt(QUALITY_DEBT)
        validate_debt_owners(debt)
        selected = selected_positive_entries(entries, args.group)
        signatures = []
        for entry in selected:
            def format_entry() -> Path:
                crate_root_inner = materialize_entry(entry, run_root)
                result = run_command(
                    [
                        "cargo",
                        "fmt",
                        "--manifest-path",
                        str(crate_root_inner / "Cargo.toml"),
                        "--",
                        "--check",
                    ],
                    check=False,
                )
                actual = (
                    None
                    if result.returncode == 0
                    else output_signature(result.stdout + result.stderr, crate_root_inner)
                )
                if actual is not None:
                    signatures.append((entry.id, actual))
                return crate_root_inner

            crate_root = timed_case("generated_code_quality", f"rustfmt/{entry.id}", format_entry)
            records.append(record_for_entry(entry, crate_root, "passed"))
        compare_exact_debt(
            category="rustfmt",
            entry_id=selection_id(selected),
            actual=merge_output_signatures(signatures),
            debt=debt,
        )
        evidence = record_evidence("rustfmt", run, records)
        print(f"generated-code rustfmt passed; evidence={evidence.relative_to(REPO_ROOT)}")
    except Exception:
        print(f"generated-code rustfmt failed; preserved={run_root}", file=sys.stderr)
        raise
    else:
        if not args.keep_success:
            shutil.rmtree(run_root, ignore_errors=True)


def gate_clippy(entries: list[Entry], args: argparse.Namespace) -> None:
    run = run_id("clippy")
    run_root = TARGET_ROOT / run
    cargo_target_dir = run_root / "cargo-target"
    records = []
    try:
        negative_seeds = {
            "clippy::arithmetic_side_effects": "forbidden_arithmetic.rs",
            "clippy::cast_sign_loss": "forbidden_allocation_width.rs",
            "clippy::needless_return": "clippy_warning.rs",
        }
        for lint, filename in negative_seeds.items():
            timed_case(
                "generated_code_quality",
                f"clippy/negative-{lint.removeprefix('clippy::')}",
                lambda lint=lint, filename=filename: assert_negative_clippy(
                    GCQ_ROOT / "negative_seeds" / filename,
                    run_root,
                    lint,
                    run_command,
                    STRICT_CLIPPY_ARGS,
                    parse_clippy_diagnostics,
                    cargo_target_dir,
                ),
            )
        debt = load_debt(QUALITY_DEBT)
        validate_debt_owners(debt)
        selected = selected_positive_entries(entries, args.group)
        summaries = []
        for entry in selected:
            def clippy_entry() -> Path:
                crate_root_inner = materialize_entry(entry, run_root)
                result = run_strict_clippy(
                    crate_root_inner,
                    run_command,
                    STRICT_CLIPPY_ARGS,
                    cargo_target_dir,
                )
                actual = parse_clippy_diagnostics(result.stdout, crate_root_inner)
                if result.returncode != 0 and not actual:
                    raise RuntimeError(
                        f"{entry.id}: clippy failed without classifiable diagnostics\n"
                        f"{result.stderr}"
                    )
                summaries.append((entry.id, actual))
                return crate_root_inner

            crate_root = timed_case("generated_code_quality", f"clippy/{entry.id}", clippy_entry)
            records.append(record_for_entry(entry, crate_root, "passed"))
        merged = merge_signature_summaries(summaries)
        lint_counts = ", ".join(
            f"{lint}={signature['count']}" for lint, signature in merged.items()
        )
        print(f"generated-code clippy diagnostics: {lint_counts or 'none'}")
        summary_path = run_root / "clippy-summary.json"
        summary_path.write_text(
            json.dumps(
                {"selection_id": selection_id(selected), "summary": merged},
                indent=2,
                sort_keys=True,
            ),
            encoding="utf-8",
        )
        print(f"generated-code clippy summary={summary_path.relative_to(REPO_ROOT)}")
        full_selection = not args.group and not explicit_entry_ids() and not os.environ.get(
            "SIFR_GCQ_MAX_ENTRIES"
        )
        validate_clippy_lint_owners(merged, debt, require_exact=full_selection)
        compare_exact_debt(
            category="clippy",
            entry_id=selection_id(selected),
            actual=merged,
            debt=debt,
        )
        evidence = record_evidence("clippy", run, records)
        print(f"generated-code clippy passed; evidence={evidence.relative_to(REPO_ROOT)}")
    except Exception:
        print(f"generated-code clippy failed; preserved={run_root}", file=sys.stderr)
        raise
    else:
        if not args.keep_success:
            shutil.rmtree(run_root, ignore_errors=True)


def gate_determinism(entries: list[Entry], args: argparse.Namespace) -> None:
    run = run_id("determinism")
    run_root = TARGET_ROOT / run
    run_root.mkdir(parents=True, exist_ok=True)
    records = []
    try:
        timed_case(
            "generated_code_quality",
            "determinism/negative-byte-drift",
            lambda: assert_negative_determinism(
                GCQ_ROOT / "negative_seeds" / "determinism_a.rs",
                GCQ_ROOT / "negative_seeds" / "determinism_b.rs",
            ),
        )
        for entry in selected_positive_entries(entries, args.group):
            def deterministic_entry() -> bytes:
                first_inner = emit_source(entry)
                second = emit_source(entry)
                if not compare_bytes(first_inner, second):
                    first_path = run_root / f"{entry.id}.first.rs"
                    second_path = run_root / f"{entry.id}.second.rs"
                    first_path.write_bytes(first_inner)
                    second_path.write_bytes(second)
                    raise RuntimeError(f"{entry.id}: repeated emission was not byte-stable")
                return first_inner

            first = timed_case("generated_code_quality", f"determinism/{entry.id}", deterministic_entry)
            digest = hashlib.sha256(first).hexdigest()
            records.append(
                {
                    "id": entry.id,
                    "group": entry.group,
                    "source_path": entry.source_path,
                    "evidence_category": entry.evidence_category,
                    "status": "passed",
                    "emit_sha256": digest,
                }
            )
        evidence = record_evidence("determinism", run, records)
        print(f"generated-code determinism passed; evidence={evidence.relative_to(REPO_ROOT)}")
    except Exception:
        print(f"generated-code determinism failed; preserved={run_root}", file=sys.stderr)
        raise
    else:
        if not args.keep_success:
            shutil.rmtree(run_root, ignore_errors=True)


def gate_demos(entries: list[Entry], args: argparse.Namespace) -> None:
    args.group = ["demos-required"]
    gate_corpus(entries, args)


def authoritative_companion_entries() -> list[tuple[Path, Entry]]:
    entries = []
    for emitted in sorted((REPO_ROOT / "demos").glob("**/emitted.rs")):
        source = emitted.with_name("main.sifr")
        if not source.is_file():
            raise RuntimeError(
                f"authoritative companion has no source: {emitted.relative_to(REPO_ROOT)}"
            )
        relative_source = source.relative_to(REPO_ROOT).as_posix()
        digest = hashlib.sha256(relative_source.encode("utf-8")).hexdigest()[:12]
        entries.append(
            (
                emitted,
                Entry(
                    id=f"companion-{digest}",
                    group="authoritative-demo-companions",
                    source_path=relative_source,
                    expected_command="build",
                    evidence_category="authoritative-companion-quality",
                ),
            )
        )
    return entries


def gate_companions(_entries: list[Entry], args: argparse.Namespace) -> None:
    run = run_id("companions")
    run_root = TARGET_ROOT / run
    cargo_target_dir = run_root / "cargo-target"
    records = []
    summaries = []
    debt = load_debt(QUALITY_DEBT)
    companions = authoritative_companion_entries()
    try:
        for emitted, entry in companions:
            def check_companion() -> tuple[Path, dict[str, dict[str, Any]]]:
                run_command(
                    [
                        "rustfmt",
                        "--edition",
                        "2024",
                        "--check",
                        "--config",
                        "skip_children=true",
                        str(emitted),
                    ]
                )
                crate_root_inner = materialize_entry(entry, run_root)
                result = run_strict_clippy(
                    crate_root_inner,
                    run_command,
                    STRICT_CLIPPY_ARGS,
                    cargo_target_dir,
                )
                diagnostics = parse_clippy_diagnostics(result.stdout, crate_root_inner)
                if result.returncode != 0 and not diagnostics:
                    raise RuntimeError(
                        f"{entry.source_path}: companion Clippy failed without "
                        "classifiable diagnostics\n"
                        f"{result.stderr}"
                    )
                return crate_root_inner, diagnostics

            crate_root, diagnostics = timed_case(
                "generated_code_quality",
                f"companions/{entry.id}",
                check_companion,
            )
            summaries.append((entry.id, diagnostics))
            record = record_for_entry(entry, crate_root, "passed")
            record["clippy_summary"] = diagnostics
            records.append(record)
            if not args.keep_success:
                shutil.rmtree(crate_root.parent, ignore_errors=True)
        merged = merge_signature_summaries(summaries)
        companion_selection = selection_id([entry for _, entry in companions])
        summary_path = run_root / "companion-summary.json"
        summary_path.write_text(
            json.dumps(
                {"selection_id": companion_selection, "summary": merged},
                indent=2,
                sort_keys=True,
            ),
            encoding="utf-8",
        )
        evidence = record_evidence("companions", run, records)
        print(
            "authoritative demo companion summary="
            f"{summary_path.relative_to(REPO_ROOT)}"
        )
        validate_clippy_lint_owners(merged, debt)
        compare_exact_debt(
            category="clippy",
            entry_id=companion_selection,
            actual=merged,
            debt=debt,
        )
        governed = compact_clippy_summary(merged)
        print(
            "authoritative demo companion quality passed; "
            f"governed_clippy={governed or 'none'}; "
            f"evidence={evidence.relative_to(REPO_ROOT)}"
        )
    except Exception:
        print(
            f"authoritative demo companion quality failed; preserved={run_root}",
            file=sys.stderr,
        )
        raise
    else:
        if not args.keep_success:
            shutil.rmtree(run_root, ignore_errors=True)


def main() -> None:
    args = parse_args()
    entries = load_manifest(Path(args.manifest))
    try:
        if args.mode == "companions":
            gate_companions(entries, args)
        elif args.mode == "corpus":
            gate_corpus(entries, args)
        elif args.mode == "inventory":
            gate_inventory(entries, args)
        elif args.mode == "panic-scan":
            gate_panic_scan(entries, args)
        elif args.mode == "rustfmt":
            gate_rustfmt(entries, args)
        elif args.mode == "clippy":
            gate_clippy(entries, args)
        elif args.mode == "determinism":
            gate_determinism(entries, args)
        elif args.mode == "demos":
            gate_demos(entries, args)
        elif args.mode == "freshness":
            gate_freshness(entries, args)
        elif args.mode == "intrinsic-panic-lint":
            run_intrinsic_panic_lint_gate(REPO_ROOT, timed_case)
        else:
            raise SystemExit(f"unsupported mode {args.mode}")
    except Exception as error:
        print(str(error), file=sys.stderr)
        raise SystemExit(1) from error


if __name__ == "__main__":
    main()
