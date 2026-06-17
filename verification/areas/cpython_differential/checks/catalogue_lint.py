"""Lint the CPython differential policy and hand-seeded catalogue."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[4]
POLICY = REPO_ROOT / "verification" / "policy" / "cpython_differential.md"
HAND_SEEDED = (
    REPO_ROOT
    / "verification"
    / "areas"
    / "cpython_differential"
    / "data"
    / "hand_seeded_manifest.json"
)
GENERATED_SEEDS = (
    REPO_ROOT
    / "verification"
    / "areas"
    / "cpython_differential"
    / "data"
    / "generated_seed_manifest.json"
)
MINIMIZED_FAILURES = (
    REPO_ROOT
    / "verification"
    / "areas"
    / "cpython_differential"
    / "data"
    / "minimized_failures.json"
)
REQUIRED_TABLES = {
    "Table 1. Supported Constructs",
    "Table 2. Excluded Divergences",
    "Table 3. Exit-Code-Stable Programs",
    "Table 4. Exclusion Id References",
}
REQUIRED_EXCLUSION_IDS = {
    "D001_RESULT_OPTION_ERRORS",
    "D002_OWNERSHIP_BORROW",
    "D003_FIXED_WIDTH_OVERFLOW",
    "D004_DEFAULT_ARGUMENT_EVALUATION",
    "D005_DIVISION_FLOOR",
    "D006_DICT_ORDER_MUTATION",
    "D007_UNICODE_ENCODING",
    "D008_ASYNC_RUNTIME",
    "D009_STATIC_NARROWING_REJECTION",
    "D010_FLOAT_PRECISION",
    "D011_REPR_FORMATTING",
    "D012_EXCEPTION_MESSAGES",
}
EXCLUSION_ID_RE = re.compile(r"^D\d{3}_[A-Z0-9_]+$")
GENERATED_SUITES = {"generated_minimized_seeds", "generated_broader"}
ERROR_PRESENCE = {"no-error", "compile-error", "runtime-error"}


def main() -> int:
    failures = lint()
    if failures:
        for failure in failures:
            print(f"cpython differential catalogue error: {failure}", file=sys.stderr)
        return 1
    print("cpython differential catalogue ok")
    return 0


def lint() -> list[str]:
    try:
        policy_text = POLICY.read_text(encoding="utf-8")
    except OSError as error:
        return [f"failed to read {POLICY.relative_to(REPO_ROOT)}: {error}"]
    try:
        manifest = json.loads(HAND_SEEDED.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        return [f"failed to read {HAND_SEEDED.relative_to(REPO_ROOT)}: {error}"]
    try:
        generated = json.loads(GENERATED_SEEDS.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        return [f"failed to read {GENERATED_SEEDS.relative_to(REPO_ROOT)}: {error}"]
    try:
        minimized = json.loads(MINIMIZED_FAILURES.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        return [f"failed to read {MINIMIZED_FAILURES.relative_to(REPO_ROOT)}: {error}"]

    tables = parse_tables(policy_text)
    failures: list[str] = []
    missing_tables = sorted(REQUIRED_TABLES - tables.keys())
    if missing_tables:
        failures.append(f"policy missing required table(s): {', '.join(missing_tables)}")

    excluded_rows = tables.get("Table 2. Excluded Divergences", [])
    declared_ids = ids_from_rows(excluded_rows, "Exclusion id", failures, "excluded divergences")
    reference_rows = tables.get("Table 4. Exclusion Id References", [])
    referenced_ids = ids_from_rows(reference_rows, "Exclusion id", failures, "exclusion references")
    supported_rows = tables.get("Table 1. Supported Constructs", [])
    exit_rows = tables.get("Table 3. Exit-Code-Stable Programs", [])

    failures.extend(validate_exclusion_ids(declared_ids, referenced_ids))
    supported_constructs = supported_construct_ids(supported_rows, failures)
    failures.extend(validate_manifest(manifest, declared_ids, supported_constructs, exit_rows))
    failures.extend(validate_generated_manifest(generated, minimized, declared_ids))
    if "requires-python" not in policy_text:
        failures.append("policy must state that CPython follows verification/pyproject.toml requires-python")
    if "exactly one JSON line" not in policy_text:
        failures.append("policy must state the exactly-one-JSON-line serializer rules")
    return failures


def validate_generated_manifest(
    generated: dict[str, Any],
    minimized: dict[str, Any],
    declared_ids: list[str],
) -> list[str]:
    failures: list[str] = []
    declared = set(declared_ids)
    if generated.get("schema_version") != 1:
        failures.append("generated seed manifest schema_version must be 1")
    if generated.get("generator_rules_version") != 1:
        failures.append("generated seed manifest generator_rules_version must be 1")
    release = generated.get("release_binary")
    if not isinstance(release, dict):
        failures.append("generated seed manifest release_binary must be an object")
    else:
        if release.get("build_command") != ["cargo", "build", "--release", "-p", "sifr"]:
            failures.append("generated suite must build the release Sifr CLI once with cargo build --release -p sifr")
        if release.get("path") != "target/release/sifr":
            failures.append("generated suite release binary path must be target/release/sifr")
        if not release.get("source_digest_inputs"):
            failures.append("generated suite must declare source_digest_inputs")
    promotion = generated.get("merge_promotion_policy")
    if not isinstance(promotion, dict) or promotion.get("requires_consecutive_green_runs") != 20:
        failures.append("generated suite merge promotion policy must require 20 consecutive green runs")
    required = generated.get("required_coverage")
    if not isinstance(required, list) or not all(isinstance(item, str) for item in required):
        failures.append("generated seed manifest required_coverage must be a string list")
        required = []
    suites = generated.get("suites")
    if not isinstance(suites, dict) or set(suites) != GENERATED_SUITES:
        failures.append("generated seed manifest suites must exactly define generated_minimized_seeds and generated_broader")
        return failures
    covered: set[str] = set()
    seen_cases: list[str] = []
    seen_seeds: list[str] = []
    for suite_name, suite in suites.items():
        validate_generated_suite(str(suite_name), suite, declared, covered, seen_cases, seen_seeds, failures)
    missing_coverage = sorted(set(required) - covered)
    if missing_coverage:
        failures.append(f"generated seed coverage missing required category/categories: {', '.join(missing_coverage)}")
    duplicate_cases = duplicates(seen_cases)
    if duplicate_cases:
        failures.append(f"duplicate generated case id(s): {', '.join(duplicate_cases)}")
    duplicate_seeds = duplicates(seen_seeds)
    if duplicate_seeds:
        failures.append(f"duplicate generated seed(s): {', '.join(duplicate_seeds)}")
    if minimized.get("schema_version") != 1:
        failures.append("minimized failures ledger schema_version must be 1")
    if not isinstance(minimized.get("entries"), list):
        failures.append("minimized failures ledger entries must be a list")
    return failures


def validate_generated_suite(
    suite_name: str,
    suite: Any,
    declared_ids: set[str],
    covered: set[str],
    seen_cases: list[str],
    seen_seeds: list[str],
    failures: list[str],
) -> None:
    if not isinstance(suite, dict):
        failures.append(f"{suite_name} generated suite must be an object")
        return
    per_timeout = suite.get("per_program_timeout_seconds")
    overall_timeout = suite.get("overall_timeout_seconds")
    if not isinstance(per_timeout, int) or per_timeout <= 0:
        failures.append(f"{suite_name} per_program_timeout_seconds must be a positive integer")
    if not isinstance(overall_timeout, int) or overall_timeout <= 0:
        failures.append(f"{suite_name} overall_timeout_seconds must be a positive integer")
    cases = suite.get("cases")
    if not isinstance(cases, list) or not cases:
        failures.append(f"{suite_name} generated suite cases must be a non-empty list")
        return
    if isinstance(per_timeout, int) and isinstance(overall_timeout, int) and overall_timeout < per_timeout:
        failures.append(f"{suite_name} overall timeout must be at least one per-program timeout")
    for case in cases:
        validate_generated_case(suite_name, case, declared_ids, covered, seen_cases, seen_seeds, failures)


def validate_generated_case(
    suite_name: str,
    case: Any,
    declared_ids: set[str],
    covered: set[str],
    seen_cases: list[str],
    seen_seeds: list[str],
    failures: list[str],
) -> None:
    if not isinstance(case, dict):
        failures.append(f"{suite_name} generated case must be an object")
        return
    case_id = case.get("id")
    if not isinstance(case_id, str) or not case_id:
        failures.append(f"{suite_name} generated case id must be a non-empty string")
    else:
        seen_cases.append(case_id)
    seed = case.get("seed")
    if not isinstance(seed, int):
        failures.append(f"{case_id} generated seed must be an integer")
    else:
        seen_seeds.append(str(seed))
    if case.get("expected_exit_bucket") not in {"0", "non-zero"}:
        failures.append(f"{case_id} expected_exit_bucket must be 0 or non-zero")
    if case.get("expected_error_presence") not in ERROR_PRESENCE:
        failures.append(f"{case_id} expected_error_presence must be one of {', '.join(sorted(ERROR_PRESENCE))}")
    covers = case.get("covers")
    if not isinstance(covers, list) or not all(isinstance(item, str) for item in covers):
        failures.append(f"{case_id} covers must be a string list")
    else:
        covered.update(covers)
    exclusions = case.get("forbidden_exclusions")
    if not isinstance(exclusions, list) or set(exclusions) != declared_ids:
        failures.append(f"{case_id} forbidden_exclusions must exactly match policy ids")
    shape = case.get("shape")
    if shape not in {"arith_branch", "string_choice", "list_tuple_loop", "dict_sorted"}:
        failures.append(f"{case_id} generated shape is not supported by the generator")


def parse_tables(text: str) -> dict[str, list[dict[str, str]]]:
    tables: dict[str, list[dict[str, str]]] = {}
    current_heading: str | None = None
    table_lines: list[str] = []
    for line in text.splitlines():
        if line.startswith("## "):
            if current_heading is not None:
                tables[current_heading] = parse_table_lines(table_lines)
            current_heading = line.removeprefix("## ").strip()
            table_lines = []
            continue
        if current_heading is not None and line.startswith("|"):
            table_lines.append(line)
    if current_heading is not None:
        tables[current_heading] = parse_table_lines(table_lines)
    return {heading: rows for heading, rows in tables.items() if rows}


def parse_table_lines(lines: list[str]) -> list[dict[str, str]]:
    if len(lines) < 2:
        return []
    headers = split_row(lines[0])
    rows: list[dict[str, str]] = []
    for line in lines[2:]:
        cells = split_row(line)
        if len(cells) != len(headers):
            continue
        rows.append(dict(zip(headers, cells, strict=True)))
    return rows


def split_row(line: str) -> list[str]:
    return [cell.strip() for cell in line.strip().strip("|").split("|")]


def ids_from_rows(
    rows: list[dict[str, str]],
    field: str,
    failures: list[str],
    label: str,
) -> list[str]:
    ids: list[str] = []
    for index, row in enumerate(rows):
        raw_id = row.get(field, "").strip("` ")
        if not raw_id:
            failures.append(f"{label} row {index + 1} missing {field}")
            continue
        ids.append(raw_id)
    return ids


def validate_exclusion_ids(declared_ids: list[str], referenced_ids: list[str]) -> list[str]:
    failures: list[str] = []
    duplicate_declared = duplicates(declared_ids)
    if duplicate_declared:
        failures.append(f"duplicate excluded divergence ids: {', '.join(duplicate_declared)}")
    invalid = sorted({item for item in declared_ids if EXCLUSION_ID_RE.fullmatch(item) is None})
    if invalid:
        failures.append(f"invalid excluded divergence id format: {', '.join(invalid)}")
    declared = set(declared_ids)
    missing_required = sorted(REQUIRED_EXCLUSION_IDS - declared)
    if missing_required:
        failures.append(f"excluded divergence table missing required id(s): {', '.join(missing_required)}")
    unknown_references = sorted(set(referenced_ids) - declared)
    if unknown_references:
        failures.append(f"exclusion reference table mentions unknown id(s): {', '.join(unknown_references)}")
    unreferenced = sorted(declared - set(referenced_ids))
    if unreferenced:
        failures.append(f"excluded divergence id(s) lack references: {', '.join(unreferenced)}")
    return failures


def validate_manifest(
    manifest: dict[str, Any],
    declared_ids: list[str],
    supported_constructs: set[int],
    exit_rows: list[dict[str, str]],
) -> list[str]:
    failures: list[str] = []
    if manifest.get("schema_version") != 1:
        failures.append("hand-seeded manifest schema_version must be 1")
    declared = set(declared_ids)
    forbidden = manifest.get("forbidden_exclusions")
    if not isinstance(forbidden, list) or set(forbidden) != declared:
        failures.append("hand-seeded manifest forbidden_exclusions must exactly match policy ids")
    cases = manifest.get("cases")
    if not isinstance(cases, list) or not cases:
        failures.append("hand-seeded manifest cases must be a non-empty list")
        return failures
    case_ids = [case.get("id") for case in cases if isinstance(case, dict)]
    duplicate_cases = duplicates([item for item in case_ids if isinstance(item, str)])
    if duplicate_cases:
        failures.append(f"duplicate hand-seeded case id(s): {', '.join(duplicate_cases)}")
    exit_codes_by_program = {
        row.get("Program id", "").strip("` "): parse_exit_codes(row.get("Allowed exit codes", ""))
        for row in exit_rows
    }
    for case in cases:
        if not isinstance(case, dict):
            failures.append("hand-seeded manifest case must be an object")
            continue
        case_id = case.get("id")
        if not isinstance(case_id, str) or not case_id:
            failures.append("hand-seeded manifest case id must be a non-empty string")
            continue
        validate_case(case, case_id, declared, supported_constructs, exit_codes_by_program, failures)
    missing_exit_rows = sorted(set(case_ids) - set(exit_codes_by_program))
    if missing_exit_rows:
        failures.append(f"exit-code-stable table missing case id(s): {', '.join(missing_exit_rows)}")
    orphan_exit_rows = sorted(set(exit_codes_by_program) - set(case_ids))
    if orphan_exit_rows:
        failures.append(f"exit-code-stable table has unknown case id(s): {', '.join(orphan_exit_rows)}")
    return failures


def validate_case(
    case: dict[str, Any],
    case_id: str,
    declared_ids: set[str],
    supported_constructs: set[int],
    exit_codes_by_program: dict[str, list[int]],
    failures: list[str],
) -> None:
    for field in ("python", "sifr"):
        path = case.get(field)
        if not isinstance(path, str) or not (REPO_ROOT / path).is_file():
            failures.append(f"{case_id} {field} path is missing or does not exist")
    allowed = case.get("allowed_exit_codes")
    if not isinstance(allowed, list) or not allowed or not all(isinstance(item, int) for item in allowed):
        failures.append(f"{case_id} allowed_exit_codes must be a non-empty integer list")
    elif exit_codes_by_program.get(case_id) != allowed:
        failures.append(f"{case_id} allowed_exit_codes do not match policy exit-code table")
    constructs = case.get("supported_constructs")
    if not isinstance(constructs, list) or not constructs or not all(isinstance(item, int) for item in constructs):
        failures.append(f"{case_id} supported_constructs must be a non-empty integer list")
    else:
        unknown_constructs = sorted(set(constructs) - supported_constructs)
        if unknown_constructs:
            failures.append(
                f"{case_id} references unknown supported construct(s): "
                f"{', '.join(str(item) for item in unknown_constructs)}"
            )
    exclusions = case.get("excluded_divergences")
    if not isinstance(exclusions, list) or not all(isinstance(item, str) for item in exclusions):
        failures.append(f"{case_id} excluded_divergences must be a string list")
        return
    unknown = sorted(set(exclusions) - declared_ids)
    if unknown:
        failures.append(f"{case_id} references unknown exclusion id(s): {', '.join(unknown)}")


def parse_exit_codes(raw: str) -> list[int]:
    return [int(item) for item in re.findall(r"\d+", raw)]


def supported_construct_ids(rows: list[dict[str, str]], failures: list[str]) -> set[int]:
    ids: list[int] = []
    for index, row in enumerate(rows):
        raw_id = row.get("No.", "").strip("` ")
        try:
            ids.append(int(raw_id))
        except ValueError:
            failures.append(f"supported constructs row {index + 1} has invalid No. value {raw_id!r}")
    duplicate_ids = duplicates([str(item) for item in ids])
    if duplicate_ids:
        failures.append(f"duplicate supported construct id(s): {', '.join(duplicate_ids)}")
    return set(ids)


def duplicates(values: list[str]) -> list[str]:
    seen: set[str] = set()
    repeated: set[str] = set()
    for value in values:
        if value in seen:
            repeated.add(value)
        seen.add(value)
    return sorted(repeated)


if __name__ == "__main__":
    raise SystemExit(main())
