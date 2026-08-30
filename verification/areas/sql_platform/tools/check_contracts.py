#!/usr/bin/env python3
"""Validate the locked schema-first SQL platform contracts."""

from __future__ import annotations

import argparse
import copy
import json
import re
import sys
import tomllib
from pathlib import Path
from typing import Any, Callable

REPO_ROOT = Path(__file__).resolve().parents[4]
AREA_ROOT = REPO_ROOT / "verification" / "areas" / "sql_platform"
DATA_ROOT = AREA_ROOT / "data"
PHASE_PATH = REPO_ROOT / "plans" / "issues" / "active" / "ad-hoc-schema-first-sql-platform.md"
ARCHITECTURE_PATH = REPO_ROOT / "internal_docs" / "sql_architecture.md"
REPOSITORY_ARCHITECTURE_PATH = REPO_ROOT / "internal_docs" / "architecture.md"
ROADMAP_PATH = REPO_ROOT / "plans" / "roadmap.md"
BASELINE_PATH = AREA_ROOT / "dependency_baseline.toml"
ROOT_MANIFEST_PATH = REPO_ROOT / "Cargo.toml"
LOCK_MANIFEST_PATH = REPO_ROOT / "crates" / "sifr_sql_dependency_lock" / "Cargo.toml"
LOCKFILE_PATH = REPO_ROOT / "Cargo.lock"

MILESTONES = [
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
PROVIDERS = {"postgresql", "mysql", "sqlite"}
DOMAINS = {"grammar", "schema", "runtime", "tool", "migration", "editor"}
EVIDENCE_TYPES = {"positive", "negative", "mutation", "integration", "fuzz", "property", "performance"}
PROFILE_NAMES = {"create-pr", "merge", "nightly", "release"}
PROFILE_SUITES = {"contracts", "dependency-baseline", "mutation"}
REQUIRED_AUDITS = {"advisory", "license", "panic", "secret-redaction", "unsafe-code"}


class ContractError(ValueError):
    """A locked SQL platform contract is invalid."""


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ContractError(message)


def read_json(path: Path) -> dict[str, Any]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    require(isinstance(payload, dict), f"{path.relative_to(REPO_ROOT)} must contain a JSON object")
    return payload


def read_toml(path: Path) -> dict[str, Any]:
    with path.open("rb") as handle:
        return tomllib.load(handle)


def nonempty(value: object) -> bool:
    return isinstance(value, str) and bool(value.strip())


def unique_rows(rows: object, identity: str, label: str) -> list[dict[str, Any]]:
    require(isinstance(rows, list) and rows, f"{label} has no rows")
    typed = [row for row in rows if isinstance(row, dict)]
    require(len(typed) == len(rows), f"{label} contains a non-object row")
    values = [row.get(identity) for row in typed]
    require(all(nonempty(value) for value in values), f"{label} contains an empty {identity}")
    require(len(values) == len(set(values)), f"{label} contains a duplicate {identity}")
    return typed


def validate_phase(text: str) -> None:
    headers = list(re.finditer(r"^### Milestone (\d+): .+$", text, re.MULTILINE))
    require([int(item.group(1)) for item in headers] == list(range(19)), "phase milestone headers must be 0 through 18")
    found_ids: list[str] = []
    for index, header in enumerate(headers):
        end = headers[index + 1].start() if index + 1 < len(headers) else text.find("\n## Dependency sequence", header.start())
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
    require(found_ids == MILESTONES, "phase milestone IDs do not match the locked sequence")

    tables = re.findall(
        r"\| Milestone \| Status \|[^\n]+\n\|[^\n]+\n((?:\|[^\n]+\n){19})",
        text,
    )
    require(len(tables) >= 2, "phase must contain the milestone status table and progress ledger")
    parsed = []
    for table in (tables[0], tables[-1]):
        rows = []
        for line in table.splitlines():
            cells = [cell.strip() for cell in line.strip("|").split("|")]
            require(len(cells) >= 2 and cells[0].isdigit(), "phase contains an invalid milestone progress row")
            rows.append((int(cells[0]), cells[1].lower()))
        parsed.append(rows)
    require(parsed[0] == parsed[1], "milestone status table and progress ledger are not synchronized")

    required_links = (
        (REPOSITORY_ARCHITECTURE_PATH, "ad-hoc-schema-first-sql-platform.md"),
        (REPOSITORY_ARCHITECTURE_PATH, "sql_architecture.md"),
        (ROADMAP_PATH, "ad-hoc-schema-first-sql-platform.md"),
        (ROADMAP_PATH, "sql_architecture.md"),
    )
    for path, link in required_links:
        require(link in path.read_text(encoding="utf-8"), f"{path.relative_to(REPO_ROOT)} does not link {link}")


def validate_language_contracts(payload: dict[str, Any]) -> None:
    require(payload.get("schema_version") == 1, "language contract schema_version must be 1")
    rows = unique_rows(payload.get("contracts"), "id", "language contracts")
    expected = {
        "structural-records",
        "fixed-width-integers",
        "canonical-temporal-values",
        "network-address-values",
        "replay-safe-callbacks",
        "bounded-cancellation-cleanup",
        "diagnostic-registry",
    }
    require({row["id"] for row in rows} == expected, "language contract inventory is incomplete")
    for row in rows:
        require(row.get("status") == "accepted", f"language contract {row['id']} is not accepted")
        require(nonempty(row.get("implementation_owner")), f"language contract {row['id']} has no owner")
        document = REPO_ROOT / str(row.get("document", ""))
        require(document.is_file(), f"language contract {row['id']} document does not exist")
        anchor = row.get("anchor")
        require(nonempty(anchor) and anchor in document.read_text(encoding="utf-8"), f"language contract {row['id']} anchor is missing")


def validate_ownership(payload: dict[str, Any]) -> None:
    require(payload.get("schema_version") == 1, "ownership schema_version must be 1")
    rows = unique_rows(payload.get("surfaces"), "id", "ownership map")
    covered: set[str] = set()
    for row in rows:
        milestone = row.get("milestone")
        require(milestone in MILESTONES, f"ownership row {row['id']} has an invalid milestone")
        covered.add(str(milestone))
        require(nonempty(row.get("repository_owner")), f"ownership row {row['id']} has no repository owner")
        require(nonempty(row.get("acceptance")), f"ownership row {row['id']} has no acceptance mapping")
    require(covered == set(MILESTONES), "ownership map does not cover every milestone")


def validate_topology(payload: dict[str, Any]) -> set[str]:
    require(payload.get("schema_version") == 1, "artifact topology schema_version must be 1")
    rows = unique_rows(payload.get("artifacts"), "id", "artifact topology")
    by_id = {str(row["id"]): row for row in rows}
    allowed_roles = {
        "qualification-only",
        "compiler-host",
        "compiler-contract",
        "public-api",
        "runtime",
        "provider-component",
        "provider-runtime",
        "provider-tool",
    }
    forbidden_application_roles = {"qualification-only", "compiler-host", "compiler-contract", "provider-component", "provider-tool"}
    for row in rows:
        identity = str(row["id"])
        require(row.get("role") in allowed_roles, f"artifact {identity} has an invalid role")
        require(row.get("owner_milestone") in MILESTONES, f"artifact {identity} has an invalid milestone")
        require(nonempty(row.get("repository_owner")), f"artifact {identity} has no owner")
        require(isinstance(row.get("included_in_application"), bool), f"artifact {identity} has no application policy")
        dependencies = row.get("depends_on")
        require(isinstance(dependencies, list), f"artifact {identity} has invalid dependencies")
        require(len(dependencies) == len(set(dependencies)), f"artifact {identity} has duplicate dependencies")
        require(all(item in by_id for item in dependencies), f"artifact {identity} has an unknown dependency")
        if row.get("role") in forbidden_application_roles:
            require(row["included_in_application"] is False, f"artifact {identity} leaks into application output")

    visiting: set[str] = set()
    visited: set[str] = set()

    def walk(identity: str, application_root: bool) -> None:
        require(identity not in visiting, f"artifact topology contains a cycle at {identity}")
        if identity in visited and not application_root:
            return
        visiting.add(identity)
        row = by_id[identity]
        for dependency in row["depends_on"]:
            if application_root:
                require(by_id[dependency]["included_in_application"] is True, f"application artifact {identity} reaches host-only {dependency}")
            walk(str(dependency), application_root)
        visiting.remove(identity)
        visited.add(identity)

    for identity, row in by_id.items():
        walk(identity, bool(row["included_in_application"]))
    return set(by_id)


def validate_capabilities(payload: dict[str, Any]) -> None:
    require(payload.get("schema_version") == 1, "capability schema_version must be 1")
    require(set(payload.get("providers", [])) == PROVIDERS, "capability providers must be PostgreSQL, MySQL, and SQLite")
    require(set(payload.get("domains", [])) == DOMAINS, "capability domains are incomplete")
    rows = unique_rows(payload.get("capabilities"), "id", "capability matrix")
    expected = {(provider, domain) for provider in PROVIDERS for domain in DOMAINS}
    actual: set[tuple[str, str]] = set()
    for row in rows:
        identity = str(row["id"])
        provider = row.get("provider")
        domain = row.get("domain")
        require(identity == f"{provider}.{domain}", f"capability {identity} has inconsistent identity")
        require((provider, domain) in expected, f"capability {identity} is an unsupported provider claim")
        actual.add((str(provider), str(domain)))
        require(row.get("owner_milestone") in MILESTONES, f"capability {identity} has an invalid milestone")
        require(nonempty(row.get("acceptance")), f"capability {identity} has no acceptance mapping")
        require(isinstance(row.get("behaviors"), list) and row["behaviors"], f"capability {identity} has no behaviors")
        evidence = row.get("required_evidence")
        require(isinstance(evidence, list) and evidence, f"capability {identity} has an empty gate")
        require(set(evidence).issubset(EVIDENCE_TYPES), f"capability {identity} has an invalid evidence type")
        require(row.get("status") in {"planned", "active", "complete"}, f"capability {identity} has an invalid status")
    require(actual == expected, "capability matrix does not cover every provider and domain")


def validate_inventory(payload: dict[str, Any]) -> None:
    require(payload.get("schema_version") == 1, "verification inventory schema_version must be 1")
    require(set(payload.get("allowed_evidence_types", [])) == EVIDENCE_TYPES, "verification evidence vocabulary has drifted")
    rows = unique_rows(payload.get("invariants"), "id", "verification inventory")
    require(len(rows) == 30, "verification inventory must contain 12 permanent and 18 delivery invariants")
    actual = {(row.get("source"), row.get("source_index")) for row in rows}
    expected = {("sql-architecture-permanent-rule", index) for index in range(1, 13)}
    expected.update({("phase-locked-delivery-contract", index) for index in range(1, 19)})
    require(actual == expected, "verification inventory invariant indices are incomplete")
    for row in rows:
        identity = str(row["id"])
        require(row.get("owner_milestone") in MILESTONES, f"invariant {identity} has an invalid milestone")
        require(nonempty(row.get("acceptance")), f"invariant {identity} has no acceptance mapping")
        require(nonempty(row.get("evidence_owner")), f"invariant {identity} has no evidence owner")
        evidence = row.get("evidence_types")
        require(isinstance(evidence, list) and evidence, f"invariant {identity} has an empty gate")
        require(set(evidence).issubset(EVIDENCE_TYPES), f"invariant {identity} has an invalid evidence type")
        require(row.get("status") in {"planned", "active", "complete"}, f"invariant {identity} has an invalid status")
    prerequisites = unique_rows(payload.get("external_prerequisites"), "id", "external prerequisites")
    require(len(prerequisites) == 1, "verification inventory must name one async cleanup prerequisite")
    row = prerequisites[0]
    issue = REPO_ROOT / str(row.get("issue", ""))
    require(issue.is_file(), "async cleanup prerequisite issue does not exist")
    require(nonempty(row.get("owner")), "async cleanup prerequisite has no owner")
    require(row.get("required_before") == "sql_9_postgresql_runtime", "async cleanup prerequisite has the wrong deadline")
    require(nonempty(row.get("merge_evidence")), "async cleanup prerequisite has no merge evidence contract")
    require(isinstance(row.get("capabilities"), list) and len(row["capabilities"]) == 4, "async cleanup prerequisite is incomplete")


def validate_qualification(payload: dict[str, Any], baseline: dict[str, Any], artifacts: set[str]) -> None:
    require(payload.get("schema_version") == 1, "dependency qualification schema_version must be 1")
    for field in ("rust_version", "root_lock_package", "sqlite_amalgamation"):
        require(payload.get(field) == baseline.get(field), f"dependency qualification has {field} drift")
    sqlite_parts = [int(part) for part in str(baseline["sqlite_amalgamation"]).split(".")]
    require(len(sqlite_parts) == 3, "SQLite amalgamation must have three version components")
    sqlite_number = str(sqlite_parts[0] * 1_000_000 + sqlite_parts[1] * 1_000 + sqlite_parts[2])
    build_environment = payload.get("build_environment")
    require(isinstance(build_environment, dict), "dependency qualification has no build environment")
    require(
        build_environment.get("SYNTAQLITE_SQLITE_VERSION") == sqlite_number,
        "Syntaqlite numeric version differs from the SQLite amalgamation",
    )
    require(build_environment.get("SYNTAQLITE_CFLAG_*") == [], "Syntaqlite parser flags must match bundled SQLite defaults")
    require(set(payload.get("required_audits", [])) == REQUIRED_AUDITS, "dependency audit inventory is incomplete")
    constraints = unique_rows(payload.get("constraints"), "id", "dependency constraints")
    expected_constraints = {
        "syntaqlite-source-and-fork-readiness",
        "single-sqlite-link-identity",
        "shared-tls-ring",
    }
    require({row["id"] for row in constraints} == expected_constraints, "dependency constraints are incomplete")
    for row in constraints:
        require(row.get("owner_milestone") in MILESTONES, f"dependency constraint {row['id']} has an invalid milestone")
        require(nonempty(row.get("audit_owner")), f"dependency constraint {row['id']} has no audit owner")
        rules = row.get("rules")
        require(isinstance(rules, list) and rules, f"dependency constraint {row['id']} has no enforceable rules")
        require(all(nonempty(rule) for rule in rules), f"dependency constraint {row['id']} has an empty rule")
    rows = unique_rows(payload.get("dependencies"), "crate", "dependency qualification")
    baseline_rows = unique_rows(baseline.get("crate"), "name", "dependency baseline crates")
    by_name = {str(row["name"]): row for row in baseline_rows}
    require({row["crate"] for row in rows} == set(by_name), "qualification and baseline crate identities differ")
    for row in rows:
        name = str(row["crate"])
        source = by_name[name]
        for field in ("version", "checksum", "license"):
            require(row.get(field) == source.get(field), f"qualification for {name} has baseline {field} drift")
        require(row.get("default_features") is False, f"qualification for {name} enables default features")
        require(isinstance(row.get("features"), list), f"qualification for {name} has invalid features")
        require(isinstance(row.get("targets"), list) and row["targets"], f"qualification for {name} has no targets")
        require(isinstance(row.get("artifacts"), list) and row["artifacts"], f"qualification for {name} has no artifacts")
        require(set(row["artifacts"]).issubset(artifacts), f"qualification for {name} names an unknown artifact")
        require(row.get("owner_milestone") in MILESTONES, f"qualification for {name} has an invalid milestone")
        require(nonempty(row.get("audit_owner")), f"qualification for {name} has no audit owner")
    source_rows = unique_rows(payload.get("sources"), "name", "dependency source qualification")
    require(len(source_rows) == 1 and source_rows[0]["name"] == "libpg_query", "source qualification must own libpg_query")
    tags = sorted(str(row["tag"]) for row in baseline.get("source", []))
    require(sorted(source_rows[0].get("versions", [])) == tags, "libpg_query qualification tags have drifted")
    require(source_rows[0].get("license") == "BSD-3-Clause", "libpg_query qualification has the wrong license")
    require(set(source_rows[0].get("artifacts", [])).issubset(artifacts), "libpg_query qualification names an unknown artifact")


def validate_cargo(baseline: dict[str, Any], qualification: dict[str, Any]) -> None:
    root = read_toml(ROOT_MANIFEST_PATH)
    members = root.get("workspace", {}).get("members", [])
    require("crates/sifr_sql_dependency_lock" in members, "root workspace omits sifr_sql_dependency_lock")
    dependencies = root.get("workspace", {}).get("dependencies", {})
    lock_manifest = read_toml(LOCK_MANIFEST_PATH)
    lock_dependencies = lock_manifest.get("dependencies", {})
    qualified = {row["crate"]: row for row in qualification["dependencies"]}
    for row in baseline["crate"]:
        name = str(row["name"])
        root_dependency = dependencies.get(name)
        require(isinstance(root_dependency, dict), f"root workspace omits dependency {name}")
        require(root_dependency.get("version") == f"={row['version']}", f"root dependency {name} is not pinned exactly")
        require(root_dependency.get("default-features") is False, f"root dependency {name} does not disable default features")
        lock_dependency = lock_dependencies.get(name)
        require(isinstance(lock_dependency, dict) and lock_dependency.get("workspace") is True, f"lock package omits {name}")
        require(lock_dependency.get("optional") is True, f"lock package dependency {name} must be optional")
        expected_features = set(qualified[name]["features"])
        root_features = set(root_dependency.get("features", []))
        member_features = set(lock_dependency.get("features", []))
        require(root_features.union(member_features) == expected_features, f"Cargo features for {name} differ from qualification")

    metadata = lock_manifest.get("package", {}).get("metadata", {}).get("sifr", {})
    require(metadata.get("artifact-role") == "qualification-only", "lock package has the wrong artifact role")
    require(metadata.get("contract") == "verification/areas/sql_platform/data/artifact_topology.json", "lock package has no topology contract")
    all_feature = set(lock_manifest.get("features", {}).get("all", []))
    require(all_feature == {"postgresql", "mysql", "sqlite", "runtime"}, "lock package all feature is incomplete")

    lockfile = read_toml(LOCKFILE_PATH)
    packages = lockfile.get("package", [])
    for row in baseline["crate"]:
        matches = [item for item in packages if item.get("name") == row["name"] and item.get("version") == row["version"]]
        require(len(matches) == 1, f"Cargo.lock does not resolve {row['name']} {row['version']} exactly once")
        require(matches[0].get("checksum") == row["checksum"], f"Cargo.lock checksum differs for {row['name']}")


def validate_architecture_baseline(baseline: dict[str, Any]) -> None:
    text = ARCHITECTURE_PATH.read_text(encoding="utf-8")
    require(str(baseline.get("verified_at")) in text, "SQL architecture does not contain the baseline verification date")
    require(str(baseline.get("sqlite_amalgamation")) in text, "SQL architecture has SQLite amalgamation drift")
    for row in baseline.get("crate", []):
        require(f"`{row['version']}`" in text, f"SQL architecture omits {row['name']} {row['version']}")
    for row in baseline.get("source", []):
        require(f"`{row['tag']}`" in text, f"SQL architecture omits libpg_query {row['tag']}")


def validate_profiles(overrides: dict[str, Any] | None = None) -> None:
    for name in PROFILE_NAMES:
        payload = (overrides or {}).get(name)
        if payload is None:
            payload = read_json(REPO_ROOT / "verification" / "profiles" / f"{name}.json")
        selections = [row for row in payload.get("selected_areas", []) if row.get("area") == "sql_platform"]
        require(len(selections) == 1, f"profile {name} must select sql_platform exactly once")
        row = selections[0]
        require(set(row.get("suites", [])) == PROFILE_SUITES, f"profile {name} omits an SQL platform suite")
        require(set(row.get("resource_classes", [])) == {"default-local"}, f"profile {name} has invalid SQL resources")


def validate_all(overrides: dict[str, Any] | None = None) -> None:
    values = overrides or {}
    phase = values.get("phase", PHASE_PATH.read_text(encoding="utf-8"))
    baseline = values.get("baseline", read_toml(BASELINE_PATH))
    language = values.get("language", read_json(DATA_ROOT / "language_contracts.json"))
    ownership = values.get("ownership", read_json(DATA_ROOT / "ownership_map.json"))
    topology = values.get("topology", read_json(DATA_ROOT / "artifact_topology.json"))
    capabilities = values.get("capabilities", read_json(DATA_ROOT / "capability_matrix.json"))
    inventory = values.get("inventory", read_json(DATA_ROOT / "verification_inventory.json"))
    qualification = values.get("qualification", read_json(DATA_ROOT / "dependency_qualification.json"))
    validate_phase(phase)
    validate_language_contracts(language)
    validate_ownership(ownership)
    artifacts = validate_topology(topology)
    validate_capabilities(capabilities)
    validate_inventory(inventory)
    validate_qualification(qualification, baseline, artifacts)
    validate_cargo(baseline, qualification)
    validate_architecture_baseline(baseline)
    validate_profiles(values.get("profiles"))


def self_test() -> None:
    baseline = read_toml(BASELINE_PATH)
    validate_all({"baseline": baseline})
    payloads = {
        "phase": PHASE_PATH.read_text(encoding="utf-8"),
        "ownership": read_json(DATA_ROOT / "ownership_map.json"),
        "topology": read_json(DATA_ROOT / "artifact_topology.json"),
        "capabilities": read_json(DATA_ROOT / "capability_matrix.json"),
        "inventory": read_json(DATA_ROOT / "verification_inventory.json"),
        "qualification": read_json(DATA_ROOT / "dependency_qualification.json"),
        "profiles": {
            name: read_json(REPO_ROOT / "verification" / "profiles" / f"{name}.json")
            for name in PROFILE_NAMES
        },
    }
    mutations: list[tuple[str, str, Callable[[Any], None]]] = [
        ("missing-milestone-id", "phase", lambda value: None),
        ("missing-owner", "ownership", lambda value: value["surfaces"][0].__setitem__("repository_owner", "")),
        ("invalid-milestone", "ownership", lambda value: value["surfaces"][0].__setitem__("milestone", "sql_99")),
        ("duplicate-identity", "ownership", lambda value: value["surfaces"].append(copy.deepcopy(value["surfaces"][0]))),
        ("topology-leak", "topology", lambda value: value["artifacts"][0].__setitem__("included_in_application", True)),
        ("missing-capability", "capabilities", lambda value: value["capabilities"].pop()),
        ("unsupported-provider", "capabilities", lambda value: value["capabilities"][0].__setitem__("provider", "generic")),
        ("empty-gate", "inventory", lambda value: value["invariants"][0].__setitem__("evidence_types", [])),
        ("invalid-evidence", "inventory", lambda value: value["invariants"][0].__setitem__("evidence_types", ["snapshot"])),
        ("missing-qualification", "qualification", lambda value: value["dependencies"].pop()),
        ("missing-dependency-constraint", "qualification", lambda value: value["constraints"].pop()),
        (
            "missing-profile-suite",
            "profiles",
            lambda value: value["create-pr"]["selected_areas"][-1]["suites"].pop(),
        ),
    ]
    accepted: list[str] = []
    for label, key, mutate in mutations:
        candidate = copy.deepcopy(payloads[key])
        if label == "missing-milestone-id":
            candidate = candidate.replace("ID: `sql_0_contract_lock`", "ID: missing", 1)
        else:
            mutate(candidate)
        try:
            validate_all({key: candidate, "baseline": baseline})
        except ContractError:
            continue
        accepted.append(label)
    require(not accepted, f"contract mutations were accepted: {', '.join(accepted)}")
    print(f"SQL platform contract self-test ok: mutations={len(mutations)}")


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    if args.self_test:
        self_test()
        return 0
    validate_all()
    print("SQL platform contracts ok: milestones=19 providers=3 domains=6 invariants=30")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (ContractError, OSError, ValueError, json.JSONDecodeError, tomllib.TOMLDecodeError) as error:
        print(f"SQL platform contract error: {error}", file=sys.stderr)
        raise SystemExit(1) from error
