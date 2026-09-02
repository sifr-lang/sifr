#!/usr/bin/env python3
"""Validate final, fail-closed SQL platform integration evidence."""

from __future__ import annotations

import argparse
import copy
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any, Callable

REPO_ROOT = Path(__file__).resolve().parents[4]
DATA_ROOT = REPO_ROOT / "verification" / "areas" / "sql_platform" / "data"
QUALIFICATION = DATA_ROOT / "integrated_qualification.json"
PROVIDER_ROOTS = {
    "sifr_sql_contract",
    "sifr_sql_mysql",
    "sifr_sql_mysql_runtime",
    "sifr_sql_mysql_tools",
    "sifr_sql_postgresql",
    "sifr_sql_postgresql_runtime",
    "sifr_sql_postgresql_tools",
    "sifr_sql_runtime",
    "sifr_sql_sqlite",
    "sifr_sql_sqlite_runtime",
    "sifr_sql_sqlite_tools",
    "sifr_sql_tool",
}
RUNTIME_ROOTS = [
    "crates/sifr_sql_runtime/src",
    "crates/sifr_sql_mysql_runtime/src",
    "crates/sifr_sql_postgresql_runtime/src",
    "crates/sifr_sql_sqlite_runtime/src",
]


class QualificationError(ValueError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise QualificationError(message)


def read_json(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    require(isinstance(value, dict), f"{path} must contain one JSON object")
    return value


def validate_record(record: dict[str, Any]) -> None:
    require(record.get("schema_version") == 1, "integrated qualification version drift")
    require(set(record.get("providers", [])) == {"mysql", "postgresql", "sqlite"}, "provider set is incomplete")
    require(
        set(record.get("build_modes", []))
        == {"clean", "cross-target", "incremental", "locked", "offline", "reproducible"},
        "build-mode qualification is incomplete",
    )
    require(
        set(record.get("security_contracts", []))
        == {
            "injection",
            "malformed-protocol",
            "malicious-metadata",
            "resource-exhaustion",
            "sandbox-escape",
            "secret-redaction",
            "unsafe-capability",
        },
        "security qualification is incomplete",
    )
    require(len(record.get("cross_targets", [])) == 5, "cross-target matrix is incomplete")
    validate_build_evidence(record)
    require(len(record.get("runtime_audits", [])) == 5, "runtime audit inventory is incomplete")
    runtime_evidence = record.get("runtime_audit_evidence")
    require(
        isinstance(runtime_evidence, dict)
        and set(runtime_evidence) == set(record["runtime_audits"]),
        "runtime audit evidence is incomplete",
    )
    participants = set(record.get("component_protocol_participants", []))
    require(
        participants
        == {"sifr_compiler_component", "sifr_sql_mysql", "sifr_sql_postgresql", "sifr_sql_sqlite"},
        "component protocol participant set is incomplete",
    )
    component_evidence = record.get("component_protocol_evidence")
    require(
        isinstance(component_evidence, dict) and set(component_evidence) == participants,
        "component protocol evidence is incomplete",
    )
    portable_evidence = record.get("portable_provider_evidence")
    require(
        isinstance(portable_evidence, dict)
        and set(portable_evidence) == {"mysql", "postgresql", "source-program", "sqlite"},
        "portable provider evidence is incomplete",
    )
    security_evidence = record.get("security_evidence")
    require(
        isinstance(security_evidence, dict)
        and set(security_evidence) == set(record["security_contracts"]),
        "security evidence is not bound to every security contract",
    )
    for value in [
        *component_evidence.values(),
        *portable_evidence.values(),
        *runtime_evidence.values(),
        *security_evidence.values(),
    ]:
        validate_named_test(str(value))
    for field in ("allocation_evidence", "documentation", "runnable_documentation_examples"):
        values = record.get(field)
        require(isinstance(values, list) and values, f"{field} is empty")
        for value in values:
            if "::" in str(value):
                validate_named_test(str(value))
            else:
                path = str(value)
                require((REPO_ROOT / path).is_file(), f"{field} path does not exist: {path}")


def validate_build_evidence(record: dict[str, Any]) -> None:
    evidence = record.get("build_evidence")
    require(isinstance(evidence, dict), "build evidence is absent")
    require(
        evidence
        == {
            "runner": "verification/areas/sql_platform/tools/run_sql_build_qualification.py",
            "suite": "build-qualification",
            "workflow": ".github/workflows/local-first-validation.yml",
        },
        "build evidence identity has drifted",
    )
    runner = (REPO_ROOT / evidence["runner"]).read_text(encoding="utf-8")
    for token in (
        "cargo",
        "check",
        "--locked",
        "--offline",
        "CARGO_INCREMENTAL",
        "TemporaryDirectory",
        "incremental_plan != clean_plan",
        "reproduced_plan != clean_plan",
    ):
        require(token in runner, f"SQL build runner omits executable mechanism: {token}")
    manifest = read_json(REPO_ROOT / "verification/areas/sql_platform/manifest.json")
    suites = {str(suite.get("name")): suite for suite in manifest.get("suites", [])}
    suite = suites.get(evidence["suite"])
    require(isinstance(suite, dict), "SQL build qualification suite is absent")
    require(
        suite.get("resource_classes") == ["default-local"],
        "SQL build qualification must use the schema-supported default-local resource class",
    )
    commands = {str(case.get("command")) for case in suite.get("cases", [])}
    require(commands == {"sql-build-qualification"}, "SQL build suite command has drifted")
    workflow = (REPO_ROOT / evidence["workflow"]).read_text(encoding="utf-8")
    for target in record["cross_targets"]:
        require(target in workflow, f"cross-target workflow omits {target}")
    require(
        "run_sql_build_qualification.py --target" in workflow,
        "cross-target workflow does not execute the SQL build runner",
    )
    for profile in ("create-pr", "merge", "nightly", "release"):
        profile_record = read_json(REPO_ROOT / "verification" / "profiles" / f"{profile}.json")
        sql_rows = [
            row
            for row in profile_record.get("selected_areas", [])
            if row.get("area") == "sql_platform"
        ]
        require(len(sql_rows) == 1, f"{profile} SQL profile entry is not unique")
        require(
            evidence["suite"] in sql_rows[0].get("suites", []),
            f"{profile} omits executable SQL build qualification",
        )


def validate_named_test(evidence: str) -> None:
    parts = evidence.split("::", 1)
    require(len(parts) == 2 and all(parts), f"evidence must name one exact test: {evidence}")
    path, test_name = parts
    source_path = REPO_ROOT / path
    require(source_path.is_file(), f"evidence path does not exist: {path}")
    source = source_path.read_text(encoding="utf-8")
    pattern = rf"(?m)^\s*(?:async\s+)?fn\s+{re.escape(test_name)}\s*\("
    require(re.search(pattern, source) is not None, f"evidence test does not exist: {evidence}")


def validate_final_states() -> None:
    capabilities = read_json(DATA_ROOT / "capability_matrix.json").get("capabilities", [])
    inventory = read_json(DATA_ROOT / "verification_inventory.json").get("invariants", [])
    for label, rows in (("capability", capabilities), ("invariant", inventory)):
        incomplete = [str(row.get("id")) for row in rows if row.get("status") != "complete"]
        require(not incomplete, f"incomplete {label} rows: {', '.join(incomplete)}")
        unowned = [
            str(row.get("id"))
            for row in rows
            if not str(row.get("platform_part", "")).strip()
            or (
                label == "invariant"
                and not str(row.get("evidence_owner", "")).strip()
            )
            or (
                label == "capability"
                and not isinstance(row.get("required_evidence"), list)
            )
            or (
                label == "capability"
                and not row.get("required_evidence")
            )
        ]
        require(not unowned, f"unowned {label} rows: {', '.join(unowned)}")
        forbidden = [
            str(row.get("id"))
            for row in rows
            if str(row.get("status", "")).lower()
            in {"pending", "unowned", "waived", "fallback", "deferred"}
        ]
        require(not forbidden, f"forbidden final {label} states: {', '.join(forbidden)}")


def cargo_metadata() -> dict[str, Any]:
    result = subprocess.run(
        ["cargo", "metadata", "--locked", "--offline", "--format-version", "1"],
        cwd=REPO_ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    require(result.returncode == 0, f"locked offline Cargo metadata failed: {result.stderr.strip()}")
    return json.loads(result.stdout)


def validate_dependency_reachability(record: dict[str, Any]) -> None:
    metadata = cargo_metadata()
    packages = {package["id"]: package for package in metadata["packages"]}
    workspace_names = {
        package["name"]: package["id"]
        for package in metadata["packages"]
        if package["id"] in set(metadata["workspace_members"])
    }
    missing = PROVIDER_ROOTS.difference(workspace_names)
    require(not missing, f"provider workspace roots are missing: {', '.join(sorted(missing))}")
    nodes = {node["id"]: node for node in metadata["resolve"]["nodes"]}
    pending = [workspace_names[name] for name in sorted(PROVIDER_ROOTS)]
    reachable: set[str] = set()
    while pending:
        identity = pending.pop()
        if identity in reachable:
            continue
        reachable.add(identity)
        pending.extend(dependency["pkg"] for dependency in nodes[identity].get("deps", []))
    names = {packages[identity]["name"] for identity in reachable}
    banned = set(record.get("banned_provider_dependencies", []))
    require(not names.intersection(banned), f"banned provider dependencies are reachable: {', '.join(sorted(names.intersection(banned)))}")


def validate_runtime_panic_scan() -> None:
    forbidden = (".unwrap(", ".expect(", "panic!(", "todo!(", "unimplemented!(")
    findings: list[str] = []
    for root in RUNTIME_ROOTS:
        for path in sorted((REPO_ROOT / root).rglob("*.rs")):
            production = path.read_text(encoding="utf-8").split("#[cfg(test)]", 1)[0]
            for line_number, line in enumerate(production.splitlines(), 1):
                if line.lstrip().startswith("///"):
                    continue
                if any(token in line for token in forbidden):
                    findings.append(f"{path.relative_to(REPO_ROOT)}:{line_number}")
    require(not findings, f"production SQL runtime panic tokens found: {', '.join(findings)}")


def validate_all(record: dict[str, Any] | None = None) -> None:
    selected = record or read_json(QUALIFICATION)
    validate_record(selected)
    validate_final_states()
    validate_dependency_reachability(selected)
    validate_runtime_panic_scan()


def self_test() -> None:
    record = read_json(QUALIFICATION)
    validate_all(record)
    mutations: list[tuple[str, Callable[[dict[str, Any]], None]]] = [
        ("missing-provider", lambda value: value["providers"].pop()),
        ("missing-security", lambda value: value["security_contracts"].pop()),
        ("missing-build-mode", lambda value: value["build_modes"].pop()),
        ("missing-build-evidence", lambda value: value.pop("build_evidence")),
        ("missing-evidence", lambda value: value["allocation_evidence"].clear()),
        ("missing-doc-example", lambda value: value["runnable_documentation_examples"].clear()),
        ("missing-component-evidence", lambda value: value["component_protocol_evidence"].pop("sifr_sql_mysql")),
        ("missing-portable-evidence", lambda value: value["portable_provider_evidence"].pop("sqlite")),
        ("missing-runtime-evidence", lambda value: value["runtime_audit_evidence"].pop("bounded-allocation")),
        ("wrong-security-evidence", lambda value: value["security_evidence"].__setitem__("injection", "missing.rs::missing")),
        ("reachable-banned", lambda value: value["banned_provider_dependencies"].append("serde")),
    ]
    accepted = []
    for label, mutate in mutations:
        candidate = copy.deepcopy(record)
        mutate(candidate)
        try:
            validate_all(candidate)
        except QualificationError:
            continue
        accepted.append(label)
    require(not accepted, f"integrated qualification mutations passed: {', '.join(accepted)}")
    print(f"SQL integrated qualification self-test ok: mutations={len(mutations)}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
    else:
        validate_all()
        print("SQL integrated qualification ok: providers=3 incomplete=0")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (QualificationError, OSError, ValueError, json.JSONDecodeError) as error:
        print(f"SQL integrated qualification error: {error}", file=sys.stderr)
        raise SystemExit(1) from error
