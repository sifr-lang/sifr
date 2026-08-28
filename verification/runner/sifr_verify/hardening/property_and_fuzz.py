from __future__ import annotations

import hashlib
import json
import os
import random
import subprocess
import time
from pathlib import Path
from typing import Any

from .core import (
    FUNCTION_SIGNATURE_PATTERN,
    INTEGER_LITERAL_PATTERN,
    STRING_LITERAL_PATTERN,
    canonicalize_output,
    load_index,
    required_missing,
    run_variant,
    write_text,
)
from .fixedbugs_and_crashes import contains_internal_panic

MUTATION_SMOKE_MANIFEST = Path("verification/areas/fuzz_property/mutation_smoke_manifest.json")
REQUIRED_TARGET_IDS = {
    "parse_check_entrypoint",
    "hir_type_ownership_entrypoint",
    "codegen_entrypoint",
    "diagnostic_renderer_entrypoint",
    "package_project_manifest_entrypoint",
}
ALLOWED_PROGRAM_CLASSES = {
    "valid-only",
    "invalid-only",
    "mixed-valid-invalid",
    "structured-diagnostics",
}


def run_property_suite(
    *,
    suite: dict[str, Any],
    repo_root: Path,
) -> dict[str, Any]:
    suite_name = suite["name"]
    index_raw = suite.get("index")
    if not isinstance(index_raw, str):
        raise SystemExit(f"suite '{suite_name}' missing string 'index'")
    index_path = repo_root / index_raw
    entries = load_index(index_path)
    if not entries:
        raise SystemExit(f"suite '{suite_name}' has empty index: {index_path}")
    print(f"  suite={suite_name} owner={suite.get('owner', 'unknown')} entries={len(entries)}")
    known_targets = load_known_targets(repo_root)

    result = {
        "name": suite_name,
        "owner": suite.get("owner", "unknown"),
        "blocking": bool(suite.get("blocking", False)),
        "runner": "property",
        "index": str(index_path.relative_to(repo_root)),
        "cases": [],
        "failed_cases": 0,
        "total_variants": 0,
        "total_failures": 0,
    }

    for entry in entries:
        case_id = str(entry.get("id", "<missing-id>"))
        case_result = {
            "id": case_id,
            "entry": entry.get("entry"),
            "command": entry.get("command"),
            "variants": [],
        }
        case_failed = False
        mismatches = required_missing(
            entry,
            (
                "id",
                "entry",
                "program_class",
                "command",
                "diagnostic_format",
                "note",
            ),
        )
        mismatches.extend(validate_property_target_rules(entry, known_targets))

        entry_path_raw = entry.get("entry")
        command_name = entry.get("command")
        diagnostic_format = entry.get("diagnostic_format")
        expected_exit = entry.get("expect_exit_code")
        repeat_runs = entry.get("repeat_runs", 2)
        assert_no_panic = bool(entry.get("assert_no_panic", True))

        if command_name not in {"check", "run", "build", "emit", "test", "cargo-test"}:
            mismatches.append("command")
        if not isinstance(expected_exit, int):
            mismatches.append("expect_exit_code")
        minimum_runs = 1 if command_name == "cargo-test" else 2
        if not isinstance(repeat_runs, int) or repeat_runs < minimum_runs:
            mismatches.append("repeat_runs")
        if command_name == "cargo-test" and repeat_runs != 1:
            mismatches.append("cargo-test-repeat-contract")
        entry_path = repo_root / str(entry_path_raw) if isinstance(entry_path_raw, str) else None
        if entry_path is None or not entry_path.is_file():
            mismatches.append("entry")
        if not isinstance(diagnostic_format, str) or not diagnostic_format:
            mismatches.append("diagnostic_format")
        if command_name == "cargo-test":
            if not isinstance(entry.get("cargo_package"), str) or not entry["cargo_package"]:
                mismatches.append("cargo_package")
            if not isinstance(entry.get("test_filter"), str) or not entry["test_filter"]:
                mismatches.append("test_filter")

        if mismatches:
            result["total_variants"] += 1
            result["total_failures"] += 1
            result["failed_cases"] += 1
            case_result["variants"].append(
                {
                    "label": "metadata",
                    "status": "fail",
                    "mismatches": sorted(set(mismatches)),
                }
            )
            result["cases"].append(case_result)
            continue

        assert entry_path is not None
        if command_name == "cargo-test":
            cargo_result = run_cargo_property(entry=entry, repo_root=repo_root)
            case_result["variants"].append(cargo_result)
            result["total_variants"] += 1
            if cargo_result["status"] != "pass":
                result["total_failures"] += 1
                result["failed_cases"] += 1
            result["cases"].append(case_result)
            continue

        outputs: list[tuple[int, str, str]] = []
        for run_index in range(repeat_runs):
            exit_code, stdout, stderr, elapsed_ms, argv = run_variant(
                repo_root=repo_root,
                command_name=str(command_name),
                entry=entry_path,
                diagnostic_format=str(diagnostic_format),
            )
            stdout_norm = canonicalize_output(
                repo_root=repo_root,
                text=stdout,
                diagnostic_format=str(diagnostic_format),
                stream="stdout",
            )
            stderr_norm = canonicalize_output(
                repo_root=repo_root,
                text=stderr,
                diagnostic_format=str(diagnostic_format),
                stream="stderr",
            )
            run_mismatches: list[str] = []
            if exit_code != expected_exit:
                run_mismatches.append("unexpected-exit")
            if assert_no_panic and contains_internal_panic(stdout_norm + stderr_norm):
                run_mismatches.append("panic-signal")
            status = "pass" if not run_mismatches else "fail"
            result["total_variants"] += 1
            if run_mismatches:
                case_failed = True
                result["total_failures"] += 1
            case_result["variants"].append(
                {
                    "label": f"run-{run_index + 1}",
                    "diagnostic_format": diagnostic_format,
                    "argv": argv,
                    "status": status,
                    "mismatches": run_mismatches,
                    "expected_exit_code": expected_exit,
                    "actual_exit_code": exit_code,
                    "duration_ms": round(elapsed_ms, 3),
                }
            )
            outputs.append((exit_code, stdout_norm, stderr_norm))

        if len(outputs) >= 2:
            baseline = outputs[0]
            for idx, current in enumerate(outputs[1:], start=2):
                compare_mismatches: list[str] = []
                if current[0] != baseline[0]:
                    compare_mismatches.append("exit-code-drift")
                if current[1] != baseline[1]:
                    compare_mismatches.append("stdout-drift")
                if current[2] != baseline[2]:
                    compare_mismatches.append("stderr-drift")
                result["total_variants"] += 1
                status = "pass" if not compare_mismatches else "fail"
                if compare_mismatches:
                    case_failed = True
                    result["total_failures"] += 1
                case_result["variants"].append(
                    {
                        "label": f"determinism-1-vs-{idx}",
                        "status": status,
                        "mismatches": compare_mismatches,
                    }
                )

        if case_failed:
            result["failed_cases"] += 1
        result["cases"].append(case_result)

    return result


def run_cargo_property(*, entry: dict[str, Any], repo_root: Path) -> dict[str, Any]:
    argv = [
        "cargo",
        "test",
        "--locked",
        "-p",
        str(entry["cargo_package"]),
        str(entry["test_filter"]),
        "--",
        "--nocapture",
    ]
    started = time.perf_counter()
    proc = subprocess.run(
        argv,
        cwd=repo_root,
        env={**os.environ, "CARGO_NET_OFFLINE": "true"},
        text=True,
        capture_output=True,
        check=False,
    )
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    mismatches: list[str] = []
    if proc.returncode != int(entry["expect_exit_code"]):
        mismatches.append("unexpected-exit")
    if bool(entry.get("assert_no_panic", True)) and contains_internal_panic(
        proc.stdout + proc.stderr
    ):
        mismatches.append("panic-signal")
    return {
        "label": "cargo-test",
        "status": "pass" if not mismatches else "fail",
        "mismatches": mismatches,
        "expected_exit_code": int(entry["expect_exit_code"]),
        "actual_exit_code": proc.returncode,
        "duration_ms": round(elapsed_ms, 3),
        "argv": argv,
    }


def deterministic_mutations(seed_source: str, iterations: int, random_seed: int) -> list[str]:
    rng = random.Random(random_seed)
    lines = seed_source.splitlines()
    corpus: list[str] = []
    for _ in range(iterations):
        if not lines:
            lines = ["print(\"seed\")"]
        candidate = list(lines)
        op = rng.randint(0, 8)
        if op == 0:
            insert_line = rng.choice(
                [
                    "x: int = 1",
                    "y: int = x + 1",
                    "if x > 0:",
                    "    print(str(x))",
                    "from missing_mutation_module import bad",
                    "value: int = \"bad\"",
                ]
            )
            idx = rng.randint(0, len(candidate))
            candidate.insert(idx, insert_line)
        elif op == 1 and candidate:
            idx = rng.randrange(len(candidate))
            candidate[idx] = candidate[idx] + " # fuzz"
        elif op == 2 and len(candidate) > 1:
            idx = rng.randrange(len(candidate))
            del candidate[idx]
        elif op == 3 and candidate:
            idx = rng.randrange(len(candidate))
            candidate[idx] = candidate[idx].replace("main", "main_mut")
        elif op == 4 and candidate:
            idx = rng.randrange(len(candidate))
            candidate[idx] = candidate[idx].replace("int", "str", 1)
        elif op == 5:
            import_line = rng.choice(
                [
                    "from typing import Callable",
                    "from missing_mutation_module import bad",
                    "from helper import value",
                ]
            )
            idx = rng.randint(0, len(candidate))
            candidate.insert(idx, import_line)
        elif op == 6 and candidate:
            idx = rng.randrange(len(candidate))
            line = candidate[idx]
            if STRING_LITERAL_PATTERN.search(line):
                candidate[idx] = STRING_LITERAL_PATTERN.sub('"mutated"', line, count=1)
            else:
                candidate.insert(rng.randint(0, len(candidate)), 'label: str = "mutated"')
        elif op == 7 and candidate:
            idx = rng.randrange(len(candidate))
            line = candidate[idx]
            if INTEGER_LITERAL_PATTERN.search(line):
                replacement = str(rng.choice([0, 1, 2, 7, 42, 99, 1000]))
                candidate[idx] = INTEGER_LITERAL_PATTERN.sub(replacement, line, count=1)
            else:
                candidate.insert(rng.randint(0, len(candidate)), "counter: int = 42")
        elif op == 8:
            signature = rng.choice(
                [
                    "def fuzz_helper(value: int) -> int:",
                    "def fuzz_helper(value: str) -> str:",
                ]
            )
            if candidate:
                signature_indices = [
                    idx for idx, line in enumerate(candidate) if FUNCTION_SIGNATURE_PATTERN.search(line)
                ]
                if signature_indices:
                    idx = rng.choice(signature_indices)
                    replacement = signature.replace("fuzz_helper", f"fuzz_helper_{rng.randint(0, 9)}")
                    candidate[idx] = replacement
                else:
                    insert_at = rng.randint(0, len(candidate))
                    body = (
                        "    return value + 1"
                        if "-> int" in signature
                        else '    return value + "_mut"'
                    )
                    candidate[insert_at:insert_at] = [signature, body]
            else:
                candidate.extend([signature, "    return value"])
        corpus.append("\n".join(candidate).strip() + "\n")
    return corpus


def run_mutation_smoke_suite(
    *,
    suite: dict[str, Any],
    repo_root: Path,
) -> dict[str, Any]:
    suite_name = suite["name"]
    index_raw = suite.get("index")
    if not isinstance(index_raw, str):
        raise SystemExit(f"suite '{suite_name}' missing string 'index'")
    index_path = repo_root / index_raw
    if not index_path.is_file():
        raise SystemExit(f"suite '{suite_name}' index not found: {index_path}")
    payload = json.loads(index_path.read_text(encoding="utf-8"))

    print(f"  suite={suite_name} owner={suite.get('owner', 'unknown')} manifest=1")

    result = {
        "name": suite_name,
        "owner": suite.get("owner", "unknown"),
        "blocking": bool(suite.get("blocking", False)),
        "runner": "mutation-smoke",
        "index": str(index_path.relative_to(repo_root)),
        "cases": [],
        "failed_cases": 0,
        "total_variants": 0,
        "total_failures": 0,
    }

    required = ("id", "note")
    mismatches = required_missing(payload, required)
    mismatches.extend(validate_fuzz_target_rules(payload, repo_root))

    case_result = {
        "id": payload.get("id", "mutation-smoke"),
        "variants": [],
    }

    if mismatches:
        result["total_variants"] += 1
        result["total_failures"] += 1
        result["failed_cases"] += 1
        case_result["variants"].append(
            {
                "label": "metadata",
                "status": "fail",
                "mismatches": sorted(set(mismatches)),
            }
        )
        result["cases"].append(case_result)
        return result

    for target in payload["targets"]:
        target_result = run_mutation_target_smoke(target=target, repo_root=repo_root)
        result["total_variants"] += int(target_result["total_variants"])
        result["total_failures"] += int(target_result["total_failures"])
        if int(target_result["total_failures"]) > 0:
            result["failed_cases"] += 1
        result["cases"].append(target_result["case"])
    return result


def run_mutation_target_smoke(*, target: dict[str, Any], repo_root: Path) -> dict[str, Any]:
    coverage_mode = str(target["coverage_mode"])
    if str(target["input_grammar"]) == "sifr-source" and coverage_mode == "deterministic-smoke":
        return run_source_mutation_target(target=target, repo_root=repo_root)
    if str(target["input_grammar"]) == "sifr-source" and coverage_mode == "property-smoke":
        return run_source_seed_target(target=target, repo_root=repo_root)
    if coverage_mode == "diagnostic-rules-smoke":
        return run_reproduction_command_target(target=target, repo_root=repo_root)
    return target_metadata_failure(target=target, mismatch=f"unsupported-coverage-mode:{coverage_mode}")


def run_source_mutation_target(*, target: dict[str, Any], repo_root: Path) -> dict[str, Any]:
    sources = load_seed_sources(target=target, repo_root=repo_root)
    iterations = int(target["iterations"])
    random_seed = int(target["random_seed"])
    generated: list[tuple[str, str]] = []
    per_seed = max(1, iterations // max(1, len(sources)))
    for idx, (seed_path, source) in enumerate(sources):
        for snippet in deterministic_mutations(
            seed_source=source,
            iterations=per_seed,
            random_seed=random_seed + (idx * 17),
        ):
            generated.append((seed_path, snippet))

    while len(generated) < iterations:
        seed_path, source = sources[len(generated) % len(sources)]
        generated.append(
            (
                seed_path,
                deterministic_mutations(
                    seed_source=source,
                    iterations=1,
                    random_seed=random_seed + len(generated),
                )[0],
            )
        )
    generated = generated[:iterations]
    return run_source_snippets(target=target, repo_root=repo_root, snippets=generated, label_prefix="fuzz")


def run_source_seed_target(*, target: dict[str, Any], repo_root: Path) -> dict[str, Any]:
    snippets = load_seed_sources(target=target, repo_root=repo_root)
    return run_source_snippets(target=target, repo_root=repo_root, snippets=snippets, label_prefix="seed")


def run_source_snippets(
    *,
    target: dict[str, Any],
    repo_root: Path,
    snippets: list[tuple[str, str]],
    label_prefix: str,
) -> dict[str, Any]:
    target_id = str(target["id"])
    case_result = target_case(target)
    total_variants = 0
    total_failures = 0
    unique_hashes: set[str] = set()
    allow_exit_codes = target["allow_exit_codes"]
    assert isinstance(allow_exit_codes, list)
    timeout_seconds = int(target["timeout_seconds"])
    tmp_dir = repo_root / "target/verification/tmp"
    for stale_path in tmp_dir.glob(f"{target_id}_*.sifr"):
        stale_path.unlink(missing_ok=True)
    baseline_output: tuple[int, str, str] | None = None
    baseline_snippet: tuple[str, str] | None = None
    baseline_path: Path | None = None

    for i, (seed_path, snippet) in enumerate(snippets, start=1):
        snippet_hash = hashlib.sha256(snippet.encode("utf-8")).hexdigest()[:16]
        unique_hashes.add(snippet_hash)
        tmp_file = tmp_dir / f"{target_id}_{i:03d}_{snippet_hash}.sifr"
        write_text(tmp_file, snippet)

        exit_code, stdout, stderr, elapsed_ms, argv = run_variant(
            repo_root=repo_root,
            command_name=str(target["command"]),
            entry=tmp_file,
            diagnostic_format=str(target["diagnostic_format"]),
            timeout_secs=timeout_seconds,
        )
        stdout_norm = canonicalize_output(
            repo_root=repo_root,
            text=stdout,
            diagnostic_format=str(target["diagnostic_format"]),
            stream="stdout",
        )
        stderr_norm = canonicalize_output(
            repo_root=repo_root,
            text=stderr,
            diagnostic_format=str(target["diagnostic_format"]),
            stream="stderr",
        )

        run_mismatches: list[str] = []
        if exit_code not in allow_exit_codes:
            run_mismatches.append("unexpected-exit")
        if bool(target.get("assert_no_panic", True)) and contains_internal_panic(stdout_norm + stderr_norm):
            run_mismatches.append("panic-signal")
        if baseline_output is None and label_prefix == "fuzz" and not run_mismatches:
            baseline_output = (exit_code, stdout_norm, stderr_norm)
            baseline_snippet = (seed_path, snippet)
            baseline_path = tmp_file

        status = "pass" if not run_mismatches else "fail"
        total_variants += 1
        if run_mismatches:
            total_failures += 1
        else:
            tmp_file.unlink(missing_ok=True)

        variant_result = {
            "label": f"{label_prefix}-{i:03d}",
            "target_id": target_id,
            "seed": seed_path,
            "status": status,
            "mismatches": run_mismatches,
            "source_hash": snippet_hash,
            "actual_exit_code": exit_code,
            "duration_ms": round(elapsed_ms, 3),
            "argv": argv,
        }
        if run_mismatches:
            variant_result["source_path"] = str(tmp_file.relative_to(repo_root))
        case_result["variants"].append(variant_result)

    if baseline_output is not None and baseline_snippet is not None and baseline_path is not None:
        seed_path, snippet = baseline_snippet
        determinism_hash = hashlib.sha256(snippet.encode("utf-8")).hexdigest()[:16]
        tmp_file = baseline_path
        write_text(tmp_file, snippet)
        exit_code, stdout, stderr, elapsed_ms, argv = run_variant(
            repo_root=repo_root,
            command_name=str(target["command"]),
            entry=tmp_file,
            diagnostic_format=str(target["diagnostic_format"]),
            timeout_secs=timeout_seconds,
        )
        current_output = (
            exit_code,
            canonicalize_output(
                repo_root=repo_root,
                text=stdout,
                diagnostic_format=str(target["diagnostic_format"]),
                stream="stdout",
            ),
            canonicalize_output(
                repo_root=repo_root,
                text=stderr,
                diagnostic_format=str(target["diagnostic_format"]),
                stream="stderr",
            ),
        )
        determinism_mismatches: list[str] = []
        if current_output[0] != baseline_output[0]:
            determinism_mismatches.append("exit-code-drift")
        if current_output[1] != baseline_output[1]:
            determinism_mismatches.append("stdout-drift")
        if current_output[2] != baseline_output[2]:
            determinism_mismatches.append("stderr-drift")
        total_variants += 1
        if determinism_mismatches:
            total_failures += 1
        else:
            tmp_file.unlink(missing_ok=True)
        case_result["variants"].append(
            {
                "label": "determinism-rerun",
                "target_id": target_id,
                "seed": seed_path,
                "status": "pass" if not determinism_mismatches else "fail",
                "mismatches": determinism_mismatches,
                "source_hash": determinism_hash,
                "actual_exit_code": exit_code,
                "duration_ms": round(elapsed_ms, 3),
                "argv": argv,
            }
        )

    uniqueness_mismatch: list[str] = []
    min_unique = int(target["min_unique_cases"])
    if len(unique_hashes) < min_unique:
        uniqueness_mismatch.append("insufficient-unique-cases")
        total_failures += 1
    total_variants += 1
    case_result["variants"].append(
        {
            "label": "uniqueness",
            "target_id": target_id,
            "status": "pass" if not uniqueness_mismatch else "fail",
            "mismatches": uniqueness_mismatch,
            "unique_cases": len(unique_hashes),
            "required_min_unique_cases": min_unique,
        }
    )
    return {"case": case_result, "total_variants": total_variants, "total_failures": total_failures}


def run_reproduction_command_target(*, target: dict[str, Any], repo_root: Path) -> dict[str, Any]:
    target_id = str(target["id"])
    case_result = target_case(target)
    argv = list(target["reproduction_command"])
    timeout_seconds = int(target["timeout_seconds"])
    started = time.perf_counter()
    try:
        proc = subprocess.run(
            argv,
            cwd=repo_root,
            env={**os.environ, "CARGO_NET_OFFLINE": "true"},
            text=True,
            capture_output=True,
            check=False,
            timeout=timeout_seconds,
        )
        exit_code = proc.returncode
        stdout = proc.stdout
        stderr = proc.stderr
    except subprocess.TimeoutExpired as timeout_error:
        exit_code = 124
        stdout = timeout_error.stdout or ""
        stderr = (timeout_error.stderr or "") + f"\ncommand timed out after {timeout_seconds} seconds"
    elapsed_ms = (time.perf_counter() - started) * 1000.0

    stdout_norm = canonicalize_output(
        repo_root=repo_root,
        text=stdout,
        diagnostic_format=None,
        stream="stdout",
    )
    stderr_norm = canonicalize_output(
        repo_root=repo_root,
        text=stderr,
        diagnostic_format=None,
        stream="stderr",
    )
    mismatches: list[str] = []
    if exit_code == 124:
        mismatches.append("timeout")
    elif exit_code != int(target["expect_exit_code"]):
        mismatches.append("unexpected-exit")
    if bool(target.get("assert_no_panic", True)) and contains_internal_panic(stdout_norm + stderr_norm):
        mismatches.append("panic-signal")

    case_result["variants"].append(
        {
            "label": "reproduction-command",
            "target_id": target_id,
            "status": "pass" if not mismatches else "fail",
            "mismatches": mismatches,
            "expected_exit_code": int(target["expect_exit_code"]),
            "actual_exit_code": exit_code,
            "duration_ms": round(elapsed_ms, 3),
            "argv": argv,
        }
    )
    return {
        "case": case_result,
        "total_variants": 1,
        "total_failures": 1 if mismatches else 0,
    }


def target_case(target: dict[str, Any]) -> dict[str, Any]:
    return {
        "id": str(target["id"]),
        "entrypoint": target.get("entrypoint"),
        "input_grammar": target.get("input_grammar"),
        "coverage_mode": target.get("coverage_mode"),
        "program_class": target.get("program_class"),
        "reproduction_command": target.get("reproduction_command"),
        "minimization_command": target.get("minimization_command"),
        "variants": [],
    }


def target_metadata_failure(*, target: dict[str, Any], mismatch: str) -> dict[str, Any]:
    case_result = target_case(target)
    case_result["variants"].append(
        {
            "label": "metadata",
            "target_id": target.get("id"),
            "status": "fail",
            "mismatches": [mismatch],
        }
    )
    return {"case": case_result, "total_variants": 1, "total_failures": 1}


def load_seed_sources(*, target: dict[str, Any], repo_root: Path) -> list[tuple[str, str]]:
    sources: list[tuple[str, str]] = []
    for seed in target["seed_files"]:
        assert isinstance(seed, str)
        seed_path = repo_root / seed
        sources.append((seed, seed_path.read_text(encoding="utf-8")))
    return sources


def load_known_targets(repo_root: Path) -> dict[str, dict[str, Any]]:
    manifest_path = repo_root / MUTATION_SMOKE_MANIFEST
    if not manifest_path.is_file():
        raise SystemExit(f"fuzz target rules manifest missing: {manifest_path}")
    payload = json.loads(manifest_path.read_text(encoding="utf-8"))
    mismatches = validate_fuzz_target_rules(payload, repo_root)
    if mismatches:
        raise SystemExit(
            "fuzz target rules invalid: " + ", ".join(sorted(set(mismatches)))
        )
    targets = payload.get("targets", [])
    return {str(target["id"]): target for target in targets}


def validate_property_target_rules(entry: dict[str, Any], known_targets: dict[str, dict[str, Any]]) -> list[str]:
    mismatches: list[str] = []
    target_ids = entry.get("target_ids")
    if not isinstance(target_ids, list) or not target_ids:
        mismatches.append("target_ids")
    else:
        for target_id in target_ids:
            if not isinstance(target_id, str) or target_id not in known_targets:
                mismatches.append(f"target_ids:{target_id}")
                continue
            if not program_class_is_compatible(
                property_class=entry.get("program_class"),
                target_class=known_targets[target_id].get("program_class"),
            ):
                mismatches.append(f"program_class:{target_id}")
    program_class = entry.get("program_class")
    if program_class not in {"valid-only", "invalid-only"}:
        mismatches.append("program_class")
    if not command_list(entry.get("reproduction_command")):
        mismatches.append("reproduction_command")
    return mismatches


def validate_fuzz_target_rules(payload: dict[str, Any], repo_root: Path) -> list[str]:
    mismatches: list[str] = []
    if payload.get("target_rules_version") != 1:
        mismatches.append("target_rules_version")
    required_target_ids = payload.get("required_target_ids")
    if not isinstance(required_target_ids, list) or set(required_target_ids) != REQUIRED_TARGET_IDS:
        mismatches.append("required_target_ids")
    elif len(required_target_ids) != len(set(required_target_ids)):
        mismatches.append("required_target_ids.duplicate")
    targets = payload.get("targets")
    if not isinstance(targets, list) or not targets:
        mismatches.append("targets")
        return mismatches

    seen: set[str] = set()
    for target in targets:
        if not isinstance(target, dict):
            mismatches.append("target")
            continue
        target_id = target.get("id")
        if not isinstance(target_id, str) or target_id not in REQUIRED_TARGET_IDS:
            mismatches.append(f"target.id:{target_id}")
        elif target_id in seen:
            mismatches.append(f"target.duplicate:{target_id}")
        else:
            seen.add(target_id)
        for field in ("entrypoint", "input_grammar", "coverage_mode", "finding_promotion"):
            if not isinstance(target.get(field), str) or not target[field]:
                mismatches.append(f"{target_id}.{field}")
        if target.get("program_class") not in ALLOWED_PROGRAM_CLASSES:
            mismatches.append(f"{target_id}.program_class")
        corpus_dir = target.get("corpus_dir")
        if not isinstance(corpus_dir, str) or not (repo_root / corpus_dir).is_dir():
            mismatches.append(f"{target_id}.corpus_dir")
        seed_files = target.get("seed_files")
        if not isinstance(seed_files, list) or not seed_files:
            mismatches.append(f"{target_id}.seed_files")
        else:
            for seed in seed_files:
                if not isinstance(seed, str) or not (repo_root / seed).is_file():
                    mismatches.append(f"{target_id}.seed_file:{seed}")
        if not command_list(target.get("reproduction_command")):
            mismatches.append(f"{target_id}.reproduction_command")
        minimization_command = target.get("minimization_command")
        if not command_list(minimization_command):
            mismatches.append(f"{target_id}.minimization_command")
        else:
            mismatches.extend(validate_command_paths(target_id, minimization_command, repo_root))
        mismatches.extend(validate_target_execution_fields(target_id, target))
    missing_targets = REQUIRED_TARGET_IDS.difference(seen)
    for target_id in sorted(missing_targets):
        mismatches.append(f"target.missing:{target_id}")
    return mismatches


def validate_target_execution_fields(target_id: object, target: dict[str, Any]) -> list[str]:
    mismatches: list[str] = []
    coverage_mode = target.get("coverage_mode")
    input_grammar = target.get("input_grammar")
    if not isinstance(target.get("timeout_seconds"), int) or int(target["timeout_seconds"]) < 1:
        mismatches.append(f"{target_id}.timeout_seconds")
    if input_grammar == "sifr-source" and coverage_mode in {"deterministic-smoke", "property-smoke"}:
        if target.get("command") not in {"check", "run", "build", "test"}:
            mismatches.append(f"{target_id}.command")
        if not isinstance(target.get("diagnostic_format"), str) or not target["diagnostic_format"]:
            mismatches.append(f"{target_id}.diagnostic_format")
        if not isinstance(target.get("random_seed"), int):
            mismatches.append(f"{target_id}.random_seed")
        if not isinstance(target.get("min_unique_cases"), int) or int(target["min_unique_cases"]) < 1:
            mismatches.append(f"{target_id}.min_unique_cases")
        allow_exit_codes = target.get("allow_exit_codes")
        if not isinstance(allow_exit_codes, list) or not all(
            isinstance(code, int) for code in allow_exit_codes
        ):
            mismatches.append(f"{target_id}.allow_exit_codes")
        if not isinstance(target.get("assert_no_panic"), bool):
            mismatches.append(f"{target_id}.assert_no_panic")
        if coverage_mode == "deterministic-smoke":
            if not isinstance(target.get("iterations"), int) or int(target["iterations"]) < 1:
                mismatches.append(f"{target_id}.iterations")
        elif "iterations" in target:
            mismatches.append(f"{target_id}.iterations.unused")
    elif coverage_mode == "diagnostic-rules-smoke":
        if not isinstance(target.get("expect_exit_code"), int):
            mismatches.append(f"{target_id}.expect_exit_code")
        if not isinstance(target.get("assert_no_panic"), bool):
            mismatches.append(f"{target_id}.assert_no_panic")
        reproduction_command = target.get("reproduction_command")
        seed_files = target.get("seed_files")
        if isinstance(reproduction_command, list):
            if "--target" not in reproduction_command or target_id not in reproduction_command:
                mismatches.append(f"{target_id}.reproduction_command.target")
            if isinstance(seed_files, list):
                for seed in seed_files:
                    if seed not in reproduction_command:
                        mismatches.append(f"{target_id}.reproduction_command.seed:{seed}")
    else:
        mismatches.append(f"{target_id}.coverage_mode")
    return mismatches


def program_class_is_compatible(*, property_class: object, target_class: object) -> bool:
    if property_class == "valid-only":
        return target_class in {"valid-only", "mixed-valid-invalid"}
    if property_class == "invalid-only":
        return target_class in {"invalid-only", "mixed-valid-invalid"}
    return False


def command_list(value: object) -> bool:
    return isinstance(value, list) and bool(value) and all(
        isinstance(part, str) and bool(part) for part in value
    )


def validate_command_paths(target_id: object, command: object, repo_root: Path) -> list[str]:
    mismatches: list[str] = []
    if not isinstance(command, list):
        return mismatches
    for part in command:
        if isinstance(part, str) and part.endswith(".py") and not (repo_root / part).is_file():
            mismatches.append(f"{target_id}.command_path:{part}")
    return mismatches
