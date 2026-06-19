from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

if __package__ in {None, ""}:
    sys.path.insert(0, str(Path(__file__).resolve().parent))

from env import discover_paths
from env_probe import run_env_probe
from import_matrix import PackageEntry, load_matrix
from report import write_report
from smoke_matrix import KNOWN_GATES, KNOWN_GROUPS, KNOWN_TIERS

MATRIX_FILES = (
    "tier1.toml",
    "tier2.toml",
    "tier3.toml",
    "tier4.toml",
    "native.toml",
    "async.toml",
    "data.toml",
    "cloud.toml",
    "brokers.toml",
)

REQUIRED_FIXTURES = (
    "simple_import",
    "primitive_conversion",
    "pydantic_models",
    "async_blocking",
    "async_http",
    "fastapi_app",
    "sqlalchemy_psycopg",
    "redis",
    "kafka",
    "pubsub",
    "aws_sqs",
    "aws_sns",
    "aws_sns_sqs_subscription",
    "pandas_arrow",
    "polars_arrow",
    "pyarrow_capsule",
    "numpy_buffer",
    "torch_dlpack",
    "tensorflow_dlpack",
    "cffi_callback",
    "cryptography_tls",
    "resource_cleanup",
    "env_probe",
)

REQUIRED_FIXTURE_FILES = (
    "simple_import/opaque_object_operations.json",
    "primitive_conversion/primitive_roundtrip.json",
    "async_blocking/async_blocking_contract.json",
    "resource_cleanup/context_manager_cleanup.json",
)

REQUIRED_SOURCE_FIXTURES = (
    "async_blocking/direct_python_call_rejected.sifr",
    "async_blocking/object_crossing_rejected.sifr",
    "async_blocking/offloaded_python_calls.sifr",
    "async_blocking/unclassified_offload_rejected.sifr",
    "primitive_conversion/primitive_roundtrip.sifr",
)


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run Sifr embedded Python interop verification.")
    parser.add_argument("--group", action="append", default=[], help="Verification group filter.")
    parser.add_argument("--tier", action="append", default=[], help="Package tier filter.")
    parser.add_argument("--gate", action="append", default=[], help="Certification gate filter.")
    parser.add_argument("--package", action="append", default=[], help="Package name filter.")
    parser.add_argument("--report", default="reports/latest.json", help="Report path under verification/python_interop.")
    parser.add_argument("--self-test", action="store_true", help="Run runner positive and negative self-tests.")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    paths = discover_paths()
    if args.self_test:
        run_self_tests(paths.area_root)
        print("python interop runner self-test ok")
        return 0

    selected_groups = validate_filters("group", args.group, KNOWN_GROUPS)
    selected_tiers = validate_filters("tier", args.tier, KNOWN_TIERS)
    selected_gates = validate_filters("gate", args.gate, KNOWN_GATES)
    matrices = load_matrices(paths.packages_root)
    validate_matrix_entries(matrices)
    validate_fixtures(paths.fixtures_root)
    validate_fixture_files(paths.fixtures_root)

    selected = select_entries(
        matrices,
        selected_groups,
        selected_tiers,
        selected_gates,
        set(args.package),
    )
    env_result = run_env_probe(paths.area_root) if "env" in selected_groups else None
    payload = {
        "schema_version": 1,
        "area": "python_interop",
        "status": "passed" if env_result else "scaffold",
        "groups": selected_groups or ["scaffold"],
        "tiers": selected_tiers,
        "gates": selected_gates or sorted({entry.gate for entry in selected if entry.gate is not None}),
        "packages": sorted({entry.name for entry in selected}),
        "matrix_files": list(MATRIX_FILES),
        "matrix_entries": len(matrices),
        "fixture_directories": list(REQUIRED_FIXTURES),
        "fixture_files": list(REQUIRED_FIXTURE_FILES),
        "source_fixtures": list(REQUIRED_SOURCE_FIXTURES),
        "summary": {
            "total_variants": max(1, len(selected)),
            "total_failures": 0,
            "blocking_failures": 0,
            "non_blocking_failures": 0,
        },
    }
    if env_result is not None:
        payload["env_probe"] = env_result
    report_path = paths.area_root / args.report
    write_report(report_path, payload)
    print(f"python interop scaffold ok: report={report_path.relative_to(paths.repo_root)}")
    return 0


def validate_filters(label: str, values: list[str], known: set[str]) -> list[str]:
    unknown = sorted(set(values).difference(known))
    if unknown:
        raise SystemExit(f"unknown python interop {label} filter(s): {', '.join(unknown)}")
    return sorted(set(values))


def load_matrices(packages_root: Path) -> list[PackageEntry]:
    entries: list[PackageEntry] = []
    for file_name in MATRIX_FILES:
        path = packages_root / file_name
        if not path.is_file():
            raise SystemExit(f"missing python interop matrix file: {path}")
        entries.extend(load_matrix(path))
    return entries


def validate_matrix_entries(entries: list[PackageEntry]) -> None:
    seen: set[tuple[str, str]] = set()
    for entry in entries:
        if entry.tier not in KNOWN_TIERS:
            raise SystemExit(f"unknown package tier for {entry.name}: {entry.tier}")
        unknown_groups = sorted(set(entry.groups).difference(KNOWN_GROUPS))
        if unknown_groups:
            raise SystemExit(f"unknown group(s) for {entry.name}: {', '.join(unknown_groups)}")
        if entry.gate is not None and entry.gate not in KNOWN_GATES:
            raise SystemExit(f"unknown certification gate for {entry.name}: {entry.gate}")
        if entry.tier == "tier1" and entry.gate is None:
            raise SystemExit(f"tier1 package {entry.name} must declare gate = \"tier1a\" or \"tier1b\"")
        if entry.tier != "tier1" and entry.gate is not None:
            raise SystemExit(f"non-tier1 package {entry.name} must not declare certification gate")
        key = (entry.name, entry.tier)
        if key in seen:
            raise SystemExit(f"duplicate package matrix entry: {entry.name} in {entry.tier}")
        seen.add(key)


def validate_fixtures(fixtures_root: Path) -> None:
    missing = [name for name in REQUIRED_FIXTURES if not (fixtures_root / name).is_dir()]
    if missing:
        raise SystemExit(f"missing python interop fixture directories: {', '.join(missing)}")


def validate_fixture_files(fixtures_root: Path) -> None:
    missing = [name for name in REQUIRED_FIXTURE_FILES if not (fixtures_root / name).is_file()]
    if missing:
        raise SystemExit(f"missing python interop fixture files: {', '.join(missing)}")
    for name in REQUIRED_FIXTURE_FILES:
        path = fixtures_root / name
        try:
            json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as error:
            raise SystemExit(f"invalid python interop fixture JSON {path}: {error}") from error
    missing_sources = [
        name for name in REQUIRED_SOURCE_FIXTURES if not (fixtures_root / name).is_file()
    ]
    if missing_sources:
        raise SystemExit(f"missing python interop source fixtures: {', '.join(missing_sources)}")


def select_entries(
    entries: list[PackageEntry],
    groups: list[str],
    tiers: list[str],
    gates: list[str],
    package_names: set[str],
) -> list[PackageEntry]:
    selected = [
        entry
        for entry in entries
        if (not groups or set(groups).intersection(entry.groups))
        and (not tiers or entry.tier in tiers)
        and (not gates or entry.gate in gates)
        and (not package_names or entry.name in package_names)
    ]
    if package_names:
        present = {entry.name for entry in selected}
        missing = sorted(package_names.difference(present))
        if missing:
            raise SystemExit(f"unknown package filter(s): {', '.join(missing)}")
    return selected


def run_self_tests(area_root: Path) -> None:
    entries = load_matrices(area_root / "packages")
    validate_matrix_entries(entries)
    validate_fixture_files(area_root / "fixtures")
    run_env_probe(area_root)
    try:
        validate_filters("group", ["not-a-group"], KNOWN_GROUPS)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test failed: unknown group was accepted")
    try:
        validate_filters("tier", ["tier0"], KNOWN_TIERS)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test failed: unknown tier was accepted")
    try:
        validate_filters("gate", ["tier99"], KNOWN_GATES)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test failed: unknown gate was accepted")
    try:
        validate_matrix_entries([PackageEntry("bad-tier1", "tier1", ("imports",))])
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test failed: tier1 package without gate was accepted")
    try:
        validate_matrix_entries([PackageEntry("bad-tier2", "tier2", ("imports",), "tier1a")])
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test failed: non-tier1 package with gate was accepted")


if __name__ == "__main__":
    raise SystemExit(main())
