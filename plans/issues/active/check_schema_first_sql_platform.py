#!/usr/bin/env python3
"""Validate the schema-first SQL delivery record."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
ISSUE_PATH = Path(__file__).with_name("ad-hoc-schema-first-sql-platform.md")
ROADMAP_PATH = REPO_ROOT / "plans" / "roadmap.md"
EXPECTED_IDS = [
    "sql_0_contract_lock",
    "sql_1_template_strings",
    "sql_2_structural_records",
    "sql_3_compiler_components",
    "sql_4_schema_profiles",
    "sql_5_common_contracts",
    "sql_6_queries_fragments",
    "sql_7_postgresql_compiler",
    "sql_8_postgresql_semantics",
    "sql_9_postgresql_runtime",
    "sql_10_incremental_editor",
    "sql_11_host_tools",
    "sql_12_schema_tools",
    "sql_13_migration_engine",
    "sql_14_postgresql_migrations",
    "sql_15_schema_polymorphism",
    "sql_16_mysql_provider",
    "sql_17_sqlite_provider",
    "sql_18_closure",
]


class RecordError(ValueError):
    """The SQL delivery record is invalid."""


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RecordError(message)


def validate_record(text: str) -> None:
    headers = list(re.finditer(r"^### Milestone (\d+): .+$", text, re.MULTILINE))
    require(
        [int(item.group(1)) for item in headers] == list(range(19)),
        "milestone headers must be 0 through 18",
    )
    found_ids: list[str] = []
    for index, header in enumerate(headers):
        if index + 1 < len(headers):
            end = headers[index + 1].start()
        else:
            end = text.find("\n## Dependency sequence", header.start())
        chunk = text[header.start() : end]
        identifier = re.search(r"^ID: `([^`]+)`$", chunk, re.MULTILINE)
        require(identifier is not None, f"milestone {index} has no ID")
        found_ids.append(identifier.group(1))
        require(re.search(r"^Purpose: .+", chunk, re.MULTILINE) is not None, f"milestone {index} has no purpose")
        require("\nOwned scope:\n" in chunk, f"milestone {index} has no owned scope")
        require("\nAcceptance criteria:\n" in chunk, f"milestone {index} has no acceptance list")
        require("- [ ] " in chunk, f"milestone {index} has an empty acceptance list")
        validation_label = "Closure validation:" if index == 18 else "Focused validation:"
        require(f"\n{validation_label}\n" in chunk, f"milestone {index} has no {validation_label.lower()}")
    require(found_ids == EXPECTED_IDS, "milestone IDs do not match the locked sequence")

    tables = re.findall(
        r"\| Milestone \| Status \|[^\n]+\n\|[^\n]+\n((?:\|[^\n]+\n){19})",
        text,
    )
    require(len(tables) >= 2, "the status table and progress ledger are required")
    parsed: list[list[tuple[int, str]]] = []
    for table in (tables[0], tables[-1]):
        rows: list[tuple[int, str]] = []
        for line in table.splitlines():
            cells = [cell.strip() for cell in line.strip("|").split("|")]
            require(len(cells) >= 2 and cells[0].isdigit(), "a progress row is invalid")
            rows.append((int(cells[0]), cells[1].lower()))
        parsed.append(rows)
    require(parsed[0] == parsed[1], "the status table and progress ledger differ")

    roadmap = ROADMAP_PATH.read_text(encoding="utf-8")
    require(ISSUE_PATH.name in roadmap, "the roadmap does not link the SQL delivery record")
    require("sql_architecture.md" in roadmap, "the roadmap does not link the SQL architecture")


def self_test(text: str) -> None:
    mutations = {
        "missing-id": text.replace("ID: `sql_0_contract_lock`", "ID: missing", 1),
        "missing-purpose": text.replace("Purpose: Lock every", "Intent: Lock every", 1),
        "missing-owned-scope": text.replace("\nOwned scope:\n", "\nOwnership:\n", 1),
        "missing-acceptance": text.replace("\nAcceptance criteria:\n", "\nChecks:\n", 1),
        "missing-validation": text.replace("\nFocused validation:\n", "\nValidation:\n", 1),
        "missing-progress-row": text.replace("| 0 | completed |", "| zero | completed |", 1),
    }
    accepted: list[str] = []
    for label, candidate in mutations.items():
        try:
            validate_record(candidate)
        except RecordError:
            continue
        accepted.append(label)
    require(not accepted, f"record mutations were accepted: {', '.join(accepted)}")
    print(f"SQL delivery record self-test ok: mutations={len(mutations)}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    text = ISSUE_PATH.read_text(encoding="utf-8")
    validate_record(text)
    if args.self_test:
        self_test(text)
    else:
        print("SQL delivery record ok: milestones=19 synchronized_tables=2")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, RecordError) as error:
        print(f"SQL delivery record error: {error}", file=sys.stderr)
        raise SystemExit(1) from error
