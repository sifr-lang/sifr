from __future__ import annotations

import json
import random
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
                "command",
                "diagnostic_format",
                "note",
            ),
        )

        entry_path_raw = entry.get("entry")
        command_name = entry.get("command")
        diagnostic_format = entry.get("diagnostic_format")
        expected_exit = entry.get("expect_exit_code")
        repeat_runs = entry.get("repeat_runs", 2)
        assert_no_panic = bool(entry.get("assert_no_panic", True))

        if command_name not in {"check", "run", "build", "test"}:
            mismatches.append("command")
        if not isinstance(expected_exit, int):
            mismatches.append("expect_exit_code")
        if not isinstance(repeat_runs, int) or repeat_runs < 2:
            mismatches.append("repeat_runs")
        entry_path = repo_root / str(entry_path_raw) if isinstance(entry_path_raw, str) else None
        if entry_path is None or not entry_path.is_file():
            mismatches.append("entry")
        if not isinstance(diagnostic_format, str) or not diagnostic_format:
            mismatches.append("diagnostic_format")

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


def run_fuzz_smoke_suite(
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
        "runner": "fuzz-smoke",
        "index": str(index_path.relative_to(repo_root)),
        "cases": [],
        "failed_cases": 0,
        "total_variants": 0,
        "total_failures": 0,
    }

    required = (
        "id",
        "command",
        "diagnostic_format",
        "note",
    )
    mismatches = required_missing(payload, required)
    seed_files = payload.get("seed_files")
    iterations = payload.get("iterations")
    random_seed = payload.get("random_seed")
    min_unique = payload.get("min_unique_cases")
    allow_exit_codes = payload.get("allow_exit_codes")
    assert_no_panic = bool(payload.get("assert_no_panic", True))

    if not isinstance(seed_files, list) or not seed_files:
        mismatches.append("seed_files")
    if not isinstance(iterations, int) or iterations < 1:
        mismatches.append("iterations")
    if not isinstance(random_seed, int):
        mismatches.append("random_seed")
    if not isinstance(min_unique, int) or min_unique < 1:
        mismatches.append("min_unique_cases")
    if not isinstance(allow_exit_codes, list) or not all(
        isinstance(code, int) for code in allow_exit_codes
    ):
        mismatches.append("allow_exit_codes")

    sources: list[str] = []
    if isinstance(seed_files, list):
        for seed in seed_files:
            if not isinstance(seed, str):
                mismatches.append("seed_file_path")
                continue
            seed_path = repo_root / seed
            if not seed_path.is_file():
                mismatches.append(f"seed_missing:{seed}")
                continue
            sources.append(seed_path.read_text(encoding="utf-8"))

    case_result = {
        "id": payload.get("id", "fuzz-smoke"),
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

    assert isinstance(iterations, int)
    assert isinstance(random_seed, int)
    assert isinstance(min_unique, int)
    assert isinstance(allow_exit_codes, list)

    generated: list[str] = []
    for idx, source in enumerate(sources):
        generated.extend(
            deterministic_mutations(
                seed_source=source,
                iterations=max(1, iterations // max(1, len(sources))),
                random_seed=random_seed + (idx * 17),
            )
        )

    if len(generated) < iterations:
        while len(generated) < iterations:
            generated.extend(
                deterministic_mutations(
                    seed_source=sources[len(generated) % len(sources)],
                    iterations=1,
                    random_seed=random_seed + len(generated),
                )
            )
    generated = generated[:iterations]

    unique_hashes: set[str] = set()
    case_failed = False
    for i, snippet in enumerate(generated, start=1):
        snippet_hash = hashlib.sha256(snippet.encode("utf-8")).hexdigest()[:16]
        unique_hashes.add(snippet_hash)
        tmp_file = repo_root / "target/verification/tmp" / f"fuzz_smoke_{i:03d}_{snippet_hash}.sifr"
        write_text(tmp_file, snippet)

        exit_code, stdout, stderr, elapsed_ms, argv = run_variant(
            repo_root=repo_root,
            command_name=str(payload["command"]),
            entry=tmp_file,
            diagnostic_format=str(payload["diagnostic_format"]),
        )
        stdout_norm = canonicalize_output(
            repo_root=repo_root,
            text=stdout,
            diagnostic_format=str(payload["diagnostic_format"]),
            stream="stdout",
        )
        stderr_norm = canonicalize_output(
            repo_root=repo_root,
            text=stderr,
            diagnostic_format=str(payload["diagnostic_format"]),
            stream="stderr",
        )

        run_mismatches: list[str] = []
        if exit_code not in allow_exit_codes:
            run_mismatches.append("unexpected-exit")
        if assert_no_panic and contains_internal_panic(stdout_norm + stderr_norm):
            run_mismatches.append("panic-signal")

        status = "pass" if not run_mismatches else "fail"
        result["total_variants"] += 1
        if run_mismatches:
            case_failed = True
            result["total_failures"] += 1
        else:
            tmp_file.unlink(missing_ok=True)

        variant_result = {
            "label": f"fuzz-{i:03d}",
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

    uniqueness_mismatch: list[str] = []
    if len(unique_hashes) < min_unique:
        uniqueness_mismatch.append("insufficient-unique-cases")
        case_failed = True
        result["total_failures"] += 1

    result["total_variants"] += 1
    case_result["variants"].append(
        {
            "label": "uniqueness",
            "status": "pass" if not uniqueness_mismatch else "fail",
            "mismatches": uniqueness_mismatch,
            "unique_cases": len(unique_hashes),
            "required_min_unique_cases": min_unique,
        }
    )

    if case_failed:
        result["failed_cases"] += 1
    result["cases"].append(case_result)
    return result


