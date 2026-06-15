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
    if "requires-python" not in policy_text:
        failures.append("policy must state that CPython follows verification/pyproject.toml requires-python")
    if "exactly one JSON line" not in policy_text:
        failures.append("policy must state the exactly-one-JSON-line serializer contract")
    return failures


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
