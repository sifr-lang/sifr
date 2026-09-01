"""Schema-first SQL platform verification area adapter."""

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
RESULT_JSON = REPO_ROOT / "target" / "verification" / "areas" / "sql-platform-results.json"
PLATFORM_CHECKER = AREA_ROOT / "tools" / "check_contracts.py"
BASELINE_RESOLVER = AREA_ROOT / "tools" / "resolve_dependency_baseline.py"
COMMANDS = {
    "sql-platform-contracts": [sys.executable, str(PLATFORM_CHECKER)],
    "sql-dependency-baseline": [sys.executable, str(BASELINE_RESOLVER), "--check"],
    "sql-platform-contract-mutations": [sys.executable, str(PLATFORM_CHECKER), "--self-test"],
    "sql-dependency-baseline-mutations": [sys.executable, str(BASELINE_RESOLVER), "--self-test"],
    "sql-integrated-qualification": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_integrated_qualification.py"),
    ],
    "sql-integrated-qualification-mutations": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_integrated_qualification.py"),
        "--self-test",
    ],
    "sql-build-qualification": [
        sys.executable,
        str(AREA_ROOT / "tools" / "run_sql_build_qualification.py"),
    ],
    "sql-build-qualification-mutations": [
        sys.executable,
        str(AREA_ROOT / "tools" / "run_sql_build_qualification.py"),
        "--self-test",
    ],
    "sql-component-qualification": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_component_qualification.py"),
    ],
    "sql-component-qualification-mutations": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_component_qualification.py"),
        "--self-test",
    ],
    "sql-schema-profile-qualification": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_schema_profiles.py"),
    ],
    "sql-schema-profile-qualification-mutations": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_schema_profiles.py"),
        "--self-test",
    ],
    "sql-schema-profile-rust-tests": [
        "cargo", "test", "--locked", "-p", "sifr_sql_contract", "--test", "schema_profiles",
    ],
    "sql-schema-profile-driver-tests": [
        "cargo", "test", "--locked", "-p", "sifr_driver", "sql_profiles_tests",
    ],
    "sql-common-qualification": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_common_sql.py"),
    ],
    "sql-common-qualification-mutations": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_common_sql.py"),
        "--self-test",
    ],
    "sql-common-contract-tests": [
        "cargo", "test", "--locked", "-p", "sifr_sql_contract", "--test", "common_sql_contracts",
    ],
    "sql-common-runtime-tests": [
        "cargo", "test", "--locked", "-p", "sifr_sql_runtime",
    ],
    "sql-query-substrate-qualification": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_query_substrate.py"),
    ],
    "sql-query-substrate-qualification-mutations": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_query_substrate.py"),
        "--self-test",
    ],
    "sql-query-contract-tests": [
        "cargo", "test", "--locked", "-p", "sifr_sql_contract", "--test", "query_fragments",
    ],
    "sql-query-runtime-tests": [
        "cargo", "test", "--locked", "-p", "sifr_sql_runtime", "--test", "query_substrate",
    ],
    "sql-query-frontend-tests": [
        "cargo", "test", "--locked", "-p", "sifr_frontend", "--test", "sql_queries",
    ],
    "sql-schema-polymorphism-qualification": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_schema_polymorphism.py"),
    ],
    "sql-schema-polymorphism-mutations": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_schema_polymorphism.py"),
        "--self-test",
    ],
    "sql-schema-polymorphism-contract-tests": [
        "cargo", "test", "--locked", "-p", "sifr_sql_contract", "--test",
        "schema_polymorphism",
    ],
    "sql-schema-polymorphism-frontend-tests": [
        "cargo", "test", "--locked", "-p", "sifr_frontend", "--test",
        "sql_schema_polymorphism",
    ],
    "sql-schema-polymorphism-package-tests": [
        "cargo", "test", "--locked", "-p", "sifr_package", "sql_profile_tests",
    ],
    "sql-schema-polymorphism-driver-tests": [
        "cargo", "test", "--locked", "-p", "sifr_driver", "sql_profiles_tests",
    ],
    "sql-schema-polymorphism-postgresql-tests": [
        "cargo", "test", "--locked", "-p", "sifr_sql_postgresql", "--test",
        "postgresql_compiler", "postgresql_normalizes_portable_requirement",
    ],
    "sql-postgresql-compiler-qualification": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_postgresql_compiler.py"),
    ],
    "sql-postgresql-compiler-qualification-mutations": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_postgresql_compiler.py"),
        "--self-test",
    ],
    "sql-postgresql-parser-matrix": [
        sys.executable,
        str(AREA_ROOT / "tools" / "run_postgresql_parser_matrix.py"),
    ],
    "sql-postgresql-semantic-completion": [
        "cargo", "test", "--locked", "-p", "sifr_sql_postgresql", "--test",
        "postgresql_compiler", "advanced_postgresql_semantics_are_owned_and_exact",
    ],
    "sql-postgresql-runtime-qualification": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_postgresql_runtime.py"),
    ],
    "sql-postgresql-runtime-qualification-mutations": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_postgresql_runtime.py"),
        "--self-test",
    ],
    "sql-postgresql-runtime-tests": [
        "cargo", "test", "--locked", "-p", "sifr_sql_postgresql_runtime",
        "--lib", "--test", "runtime_types",
    ],
    "sql-mysql-qualification": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_mysql.py"),
    ],
    "sql-mysql-qualification-mutations": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_mysql.py"),
        "--self-test",
    ],
    "sql-mysql-compiler-tests": [
        "cargo", "test", "--locked", "-p", "sifr_sql_mysql", "--test", "mysql_compiler",
    ],
    "sql-mysql-property-tests": [
        "cargo", "test", "--locked", "-p", "sifr_sql_mysql", "--test", "mysql_properties",
    ],
    "sql-mysql-runtime-tests": [
        "cargo", "test", "--locked", "-p", "sifr_sql_mysql_runtime", "--test", "runtime_types",
    ],
    "sql-mysql-migration-tests": [
        "cargo", "test", "--locked", "-p", "sifr_sql_mysql_tools", "--test", "migration_qualification",
    ],
    "sql-mysql-live-matrix": [
        sys.executable,
        str(AREA_ROOT / "tools" / "run_mysql_server_matrix.py"),
    ],
    "sql-sqlite-qualification": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_sqlite.py"),
    ],
    "sql-sqlite-qualification-mutations": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_sqlite.py"),
        "--self-test",
    ],
    "sql-sqlite-compiler-tests": [
        "cargo", "test", "--locked", "-p", "sifr_sql_sqlite",
    ],
    "sql-sqlite-runtime-tests": [
        "cargo", "test", "--locked", "-p", "sifr_sql_sqlite_runtime",
    ],
    "sql-sqlite-tool-tests": [
        "cargo", "test", "--locked", "-p", "sifr_sql_sqlite_tools",
    ],
    "sql-sqlite-library-matrix": [
        sys.executable,
        str(AREA_ROOT / "tools" / "run_sqlite_library_matrix.py"),
    ],
    "sql-incremental-editor-qualification": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_incremental_editor.py"),
    ],
    "sql-incremental-editor-mutations": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_incremental_editor.py"),
        "--self-test",
    ],
    "sql-incremental-editor-tests": [
        "cargo", "test", "--locked", "-p", "sifr_frontend", "-p", "sifr_analysis",
        "-p", "sifr_lsp",
    ],
    "sql-host-tool-qualification": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_host_tools.py"),
    ],
    "sql-host-tool-mutations": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_host_tools.py"),
        "--self-test",
    ],
    "sql-host-tool-package-tests": [
        "cargo", "test", "--locked", "-p", "sifr_package", "host_tool",
    ],
    "sql-host-tool-cli-tests": [
        "cargo", "test", "--locked", "-p", "sifr", "--test", "host_tool_cli",
    ],
    "sql-schema-tool-tests": [
        "cargo", "test", "--locked", "-p", "sifr_sql_tool",
        "-p", "sifr_sql_postgresql_tools", "-p", "sifr_sql_mysql_tools",
        "-p", "sifr_sql_sqlite_tools",
    ],
    "sql-migration-engine-qualification": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_migration_engine.py"),
    ],
    "sql-migration-engine-mutations": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_migration_engine.py"),
        "--self-test",
    ],
    "sql-migration-contract-tests": [
        "cargo", "test", "--locked", "-p", "sifr_sql_contract", "--test",
        "migration_contracts",
    ],
    "sql-migration-frontend-tests": [
        "cargo", "test", "--locked", "-p", "sifr_frontend", "--test",
        "sql_migrations",
    ],
    "sql-migration-runtime-tests": [
        "cargo", "test", "--locked", "-p", "sifr_sql_runtime", "--test",
        "migration_engine",
    ],
    "sql-migration-tool-tests": [
        "cargo", "test", "--locked", "-p", "sifr_sql_tool", "--test",
        "migration_artifacts",
    ],
    "sql-postgresql-live-schema-tools": [
        sys.executable,
        str(AREA_ROOT / "tools" / "run_postgresql_schema_tool_matrix.py"),
    ],
    "sql-postgresql-migration-qualification": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_postgresql_migrations.py"),
    ],
    "sql-postgresql-migration-mutations": [
        sys.executable,
        str(AREA_ROOT / "tools" / "check_postgresql_migrations.py"),
        "--self-test",
    ],
    "sql-postgresql-migration-tests": [
        "cargo", "test", "--locked", "-p", "sifr_sql_postgresql_tools",
        "--test", "migration_qualification",
    ],
    "sql-postgresql-live-migrations": [
        sys.executable,
        str(AREA_ROOT / "tools" / "run_postgresql_migration_matrix.py"),
    ],
    "sql-query-signature-and-fragment-semantics": [
        "cargo", "test", "--locked", "-p", "sifr_sql_contract", "--test",
        "semantic_completion",
    ],
    "sql-postgresql-live-differential": [
        sys.executable,
        str(AREA_ROOT / "tools" / "run_postgresql_server_matrix.py"),
    ],
    "sql-postgresql-live-runtime": [
        sys.executable,
        str(AREA_ROOT / "tools" / "run_postgresql_runtime_matrix.py"),
    ],
}


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite", action="append", default=[], help="Suite filter. This option can repeat.")
    parser.add_argument("--bless", action="store_true", help="Accepted for runner parity. This area rejects it.")
    parser.add_argument("--result-json", default=str(RESULT_JSON.relative_to(REPO_ROOT)))
    return parser.parse_args(argv)


def select_suites(manifest: dict[str, Any], requested: set[str]) -> list[dict[str, Any]]:
    suites = manifest.get("suites", [])
    selected = [suite for suite in suites if not requested or suite.get("name") in requested]
    missing = sorted(requested.difference(str(suite.get("name")) for suite in selected))
    if missing:
        raise SystemExit(f"unknown SQL platform suite filter(s): {', '.join(missing)}")
    if not selected:
        raise SystemExit("no SQL platform suites selected")
    return selected


def run_case(case: dict[str, Any]) -> dict[str, Any]:
    name = str(case["command"])
    command = COMMANDS.get(name)
    if command is None:
        raise SystemExit(f"unsupported SQL platform command: {name}")
    started = time.perf_counter()
    result = subprocess.run(command, cwd=REPO_ROOT, check=False)
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    status = "pass" if result.returncode == case["expect_exit_code"] else "fail"
    print(
        f"[sifr-case-timing] bucket=sql_platform case={case['id']} "
        f"elapsed_ms={int(elapsed_ms)} status={status}",
        flush=True,
    )
    return {
        "id": case["id"],
        "entry": case["entry"],
        "command": name,
        "variants": [
            {
                "label": name,
                "argv": command,
                "status": status,
                "mismatches": [] if status == "pass" else ["unexpected-exit"],
                "expected_exit_code": case["expect_exit_code"],
                "actual_exit_code": result.returncode,
                "duration_ms": round(elapsed_ms, 3),
            }
        ],
    }


def run_suite(suite: dict[str, Any]) -> dict[str, Any]:
    cases = [run_case(case) for case in suite["cases"]]
    failures = sum(variant["status"] != "pass" for case in cases for variant in case["variants"])
    variants = sum(len(case["variants"]) for case in cases)
    return {
        "name": suite["name"],
        "owner": "compiler/sql-platform",
        "blocking": True,
        "runner": "sql_platform",
        "cases": cases,
        "failed_cases": failures,
        "total_variants": variants,
        "total_failures": failures,
    }


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    if args.bless:
        raise SystemExit("sql_platform area does not support --bless")
    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    selected = select_suites(manifest, set(args.suite))
    print("Running schema-first SQL platform verification area", flush=True)
    results = [run_suite(suite) for suite in selected]
    variants = sum(int(result["total_variants"]) for result in results)
    failures = sum(int(result["total_failures"]) for result in results)
    payload = {
        "schema_version": 1,
        "area": "sql_platform",
        "bless": False,
        "manifest": str(MANIFEST_PATH.relative_to(REPO_ROOT)),
        "suites": results,
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
        print(f"SQL platform verification failed: failures={failures}", file=sys.stderr)
        return 1
    print(f"SQL platform verification ok: variants={variants}, failures=0", flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
