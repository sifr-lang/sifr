from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

if __package__ in {None, ""}:
    sys.path.insert(0, str(Path(__file__).resolve().parent))

from async_declaration_examples import (
    build_async_declaration_examples_report,
    run_async_declaration_examples_self_tests,
)
from async_context_examples import (
    build_async_context_examples_report,
    run_async_context_examples_self_tests,
)
from arrow_evidence import validate_arrow_declaration_evidence
from arrow_examples import build_arrow_examples_report, run_arrow_examples_self_tests
from buffer_examples import (
    BUFFER_EXAMPLE_CASES,
    build_buffer_examples_report,
    run_buffer_examples_self_tests,
)
from buffer_evidence import BUFFER_MATRIX_SPECS
from callback_examples import build_callback_examples_report, run_callback_examples_self_tests
from certification_matrix import build_certification_report, validate_certification_policy
from dataframe_examples import build_dataframe_examples_report, run_dataframe_examples_self_tests
from declaration_capabilities import (
    load_and_validate_capabilities,
    run_declaration_capability_self_tests,
)
from env import discover_paths
from env_probe import run_env_probe
from import_matrix import PackageEntry, load_matrix
from library_examples import build_library_examples_report, run_library_examples_self_tests
from live_examples import build_live_examples_report, run_live_examples_self_tests
from live_policy import build_live_policy_report, run_live_policy_self_tests
from ml_examples import build_ml_examples_report, run_ml_examples_self_tests
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
    "declaration_sync",
    "async_declaration",
    "async_context",
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
    "sklearn",
    "cffi_callback",
    "cryptography_tls",
    "resource_cleanup",
    "sqlite_context",
    "package_bridge_archive",
    "env_probe",
)

REQUIRED_FIXTURE_FILES = (
    "declaration_sync/sync_declaration_contract.json",
    "async_declaration/async_declaration_evidence.json",
    "async_context/async_context_evidence.json",
    "simple_import/opaque_object_operations.json",
    "primitive_conversion/primitive_roundtrip.json",
    "async_blocking/async_blocking_contract.json",
    "async_http/async_http_contract.json",
    "aws_sns/aws_sns_contract.json",
    "aws_sns_sqs_subscription/aws_sns_sqs_subscription_contract.json",
    "aws_sqs/aws_sqs_contract.json",
    "cryptography_tls/cryptography_tls_contract.json",
    "fastapi_app/fastapi_app_contract.json",
    "kafka/kafka_contract.json",
    "numpy_buffer/py_buffer_contract.json",
    "numpy_buffer/buffer_declaration_evidence.json",
    "pandas_arrow/pandas_arrow_contract.json",
    "polars_arrow/polars_arrow_contract.json",
    "pubsub/pubsub_contract.json",
    "pyarrow_capsule/arrow_capsule_contract.json",
    "pyarrow_capsule/arrow_declaration_evidence.json",
    "pydantic_models/pydantic_models_contract.json",
    "redis/redis_contract.json",
    "torch_dlpack/dlpack_tensor_contract.json",
    "cffi_callback/callback_contract.json",
    "sqlalchemy_psycopg/sqlalchemy_psycopg_contract.json",
    "tensorflow_dlpack/tensorflow_dlpack_contract.json",
    "resource_cleanup/context_manager_cleanup.json",
    "sqlite_context/sync_context_evidence.json",
    "package_bridge_archive/package_bridge_evidence.json",
)

REQUIRED_SOURCE_FIXTURES = (
    "declaration_sync/complete_call_shapes.sifr",
    "declaration_sync/pure_and_native.sifr",
    "async_declaration/httpx_client.sifr",
    "async_context/aiosqlite_session.sifr",
    "async_blocking/direct_python_call_rejected.sifr",
    "async_blocking/object_crossing_rejected.sifr",
    "async_blocking/offloaded_python_calls.sifr",
    "async_blocking/unclassified_offload_rejected.sifr",
    "simple_import/biip_schwifty_full_example.sifr",
    "primitive_conversion/primitive_roundtrip.sifr",
    "numpy_buffer/py_buffer_readonly_failure.sifr",
    "numpy_buffer/py_buffer_memoryview.sifr",
    "numpy_buffer/numpy_full_example.sifr",
    "numpy_buffer/py_buffer_roundtrip.sifr",
    "numpy_buffer/buffer_declaration_codegen_smoke.sifr",
    "numpy_buffer/buffer_declaration_self.sifr",
    "numpy_buffer/buffer_declaration_bridge.sifr",
    "numpy_buffer/buffer_affine_aggregate_codegen.sifr",
    "numpy_buffer/buffer_declaration_numpy.sifr",
    "numpy_buffer/buffer_comparison_rejected.sifr",
    "pandas_arrow/pandas_full_example.sifr",
    "polars_arrow/polars_full_example.sifr",
    "pyarrow_capsule/arrow_capsule_copy_possible.sifr",
    "pyarrow_capsule/arrow_capsule_roundtrip.sifr",
    "pyarrow_capsule/arrow_capsule_zero_copy.sifr",
    "pyarrow_capsule/arrow_declaration_compiled.sifr",
    "pyarrow_capsule/python_certifications/arrow_evidence.py",
    "pyarrow_capsule/pyarrow_full_example.sifr",
    "fastapi_app/fastapi_pydantic_full_example.sifr",
    "cryptography_tls/cryptography_cffi_full_example.sifr",
    "aws_sqs/boto3_botocore_full_example.sifr",
    "redis/redis_fakeredis_full_example.sifr",
    "sqlalchemy_psycopg/sqlalchemy_psycopg_full_example.sifr",
    "torch_dlpack/dlpack_tensor_device_failure.sifr",
    "torch_dlpack/dlpack_tensor_roundtrip.sifr",
    "torch_dlpack/torch_full_example.sifr",
    "sklearn/sklearn_full_example.sifr",
    "cffi_callback/callback_roundtrip.sifr",
    "resource_cleanup/context_manager_body_failure.sifr",
    "resource_cleanup/context_manager_failure.sifr",
    "resource_cleanup/context_manager_success.sifr",
    "resource_cleanup/resource_diagnostics.sifr",
    "sqlite_context/context_codegen_smoke.sifr",
    "package_bridge_archive/main.sifr",
    "redis/redis_live_roundtrip.sifr",
    "sqlalchemy_psycopg/postgres_live_roundtrip.sifr",
    "kafka/kafka_live_roundtrip.sifr",
    "pubsub/pubsub_live_callback_roundtrip.sifr",
    "aws_sns/sns_live_callback_roundtrip.sifr",
    "aws_sqs/sqs_live_callback_roundtrip.sifr",
)


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run Sifr embedded Python interop verification.")
    parser.add_argument("--group", action="append", default=[], help="Verification group filter.")
    parser.add_argument("--tier", action="append", default=[], help="Package tier filter.")
    parser.add_argument("--gate", action="append", default=[], help="Certification gate filter.")
    parser.add_argument("--package", action="append", default=[], help="Package name filter.")
    parser.add_argument(
        "--report",
        default="../../../target/verification/areas/python_interop/latest.json",
        help="Report path relative to verification/areas/python_interop.",
    )
    parser.add_argument("--self-test", action="store_true", help="Run runner positive and negative self-tests.")
    parser.add_argument("--live-policy", action="store_true", help="Validate live container-runtime policy.")
    parser.add_argument("--live-examples", action="store_true", help="Run testcontainers-backed live examples.")
    parser.add_argument("--dataframe-examples", action="store_true", help="Run full NumPy/pandas/Polars Sifr examples.")
    parser.add_argument(
        "--buffer-examples",
        action="store_true",
        help="Run compiled declaration-first Python buffer examples.",
    )
    parser.add_argument(
        "--arrow-examples",
        action="store_true",
        help="Run compiled declaration-first Arrow C Data Interface examples.",
    )
    parser.add_argument("--ml-examples", action="store_true", help="Run full torch/scikit-learn Sifr examples.")
    parser.add_argument("--library-examples", action="store_true", help="Run full library-family Sifr examples.")
    parser.add_argument(
        "--async-declaration-examples",
        action="store_true",
        help="Run compiled typed async Python declaration examples.",
    )
    parser.add_argument(
        "--async-context-examples",
        action="store_true",
        help="Run compiled typed async Python context-manager examples.",
    )
    parser.add_argument(
        "--callback-examples",
        action="store_true",
        help="Run compiled typed Python callback examples.",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    paths = discover_paths()
    if args.self_test:
        run_self_tests(paths.area_root)
        run_live_policy_self_tests(paths)
        run_live_examples_self_tests(paths)
        run_dataframe_examples_self_tests(paths)
        run_buffer_examples_self_tests(paths)
        run_arrow_examples_self_tests(paths)
        run_ml_examples_self_tests(paths)
        run_library_examples_self_tests(paths)
        run_async_declaration_examples_self_tests(paths)
        run_async_context_examples_self_tests(paths)
        run_callback_examples_self_tests(paths)
        print("python interop runner self-test ok")
        return 0
    if args.live_policy:
        payload = build_live_policy_report(paths)
        report_path = (paths.area_root / args.report).resolve()
        write_report(report_path, payload)
        print(f"python interop live-policy ok: report={report_path.relative_to(paths.repo_root)}")
        return 0
    if args.live_examples:
        payload = build_live_examples_report(paths)
        report_path = (paths.area_root / args.report).resolve()
        write_report(report_path, payload)
        print(
            "python interop live-examples "
            f"{payload['status']} ok: report={report_path.relative_to(paths.repo_root)}"
        )
        return 1 if payload["status"] == "live-failed" else 0
    if args.dataframe_examples:
        payload = build_dataframe_examples_report(paths)
        report_path = (paths.area_root / args.report).resolve()
        write_report(report_path, payload)
        print(
            "python interop dataframe-examples "
            f"{payload['status']} ok: report={report_path.relative_to(paths.repo_root)}"
        )
        return 1 if payload["status"] == "examples-failed" else 0
    if args.buffer_examples:
        payload = build_buffer_examples_report(paths)
        report_path = (paths.area_root / args.report).resolve()
        write_report(report_path, payload)
        print(
            "python interop buffer-examples "
            f"{payload['status']} ok: report={report_path.relative_to(paths.repo_root)}"
        )
        return 1 if payload["status"] == "examples-failed" else 0
    if args.arrow_examples:
        payload = build_arrow_examples_report(paths)
        report_path = (paths.area_root / args.report).resolve()
        write_report(report_path, payload)
        print(
            "python interop arrow-examples "
            f"{payload['status']} ok: report={report_path.relative_to(paths.repo_root)}"
        )
        return 1 if payload["status"] == "examples-failed" else 0
    if args.ml_examples:
        payload = build_ml_examples_report(paths)
        report_path = (paths.area_root / args.report).resolve()
        write_report(report_path, payload)
        print(
            "python interop ml-examples "
            f"{payload['status']} ok: report={report_path.relative_to(paths.repo_root)}"
        )
        return 1 if payload["status"] == "examples-failed" else 0
    if args.library_examples:
        payload = build_library_examples_report(paths)
        report_path = (paths.area_root / args.report).resolve()
        write_report(report_path, payload)
        print(
            "python interop library-examples "
            f"{payload['status']} ok: report={report_path.relative_to(paths.repo_root)}"
        )
        return 1 if payload["status"] == "examples-failed" else 0
    if args.async_declaration_examples:
        payload = build_async_declaration_examples_report(paths)
        report_path = (paths.area_root / args.report).resolve()
        write_report(report_path, payload)
        print(
            "python interop async-declaration-examples "
            f"{payload['status']} ok: report={report_path.relative_to(paths.repo_root)}"
        )
        return 1 if payload["status"] == "examples-failed" else 0
    if args.async_context_examples:
        payload = build_async_context_examples_report(paths)
        report_path = (paths.area_root / args.report).resolve()
        write_report(report_path, payload)
        print(
            "python interop async-context-examples "
            f"{payload['status']} ok: report={report_path.relative_to(paths.repo_root)}"
        )
        return 1 if payload["status"] == "examples-failed" else 0
    if args.callback_examples:
        payload = build_callback_examples_report(paths)
        report_path = (paths.area_root / args.report).resolve()
        write_report(report_path, payload)
        print(
            "python interop callback-examples "
            f"{payload['status']} ok: report={report_path.relative_to(paths.repo_root)}"
        )
        return 1 if payload["status"] == "examples-failed" else 0

    selected_groups = validate_filters("group", args.group, KNOWN_GROUPS)
    selected_tiers = validate_filters("tier", args.tier, KNOWN_TIERS)
    selected_gates = validate_filters("gate", args.gate, KNOWN_GATES)
    matrices = load_matrices(paths.packages_root)
    validate_matrix_entries(matrices)
    validate_certification_policy(matrices)
    declaration_capabilities = load_and_validate_capabilities(paths.area_root, paths.repo_root)
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
    certification = build_certification_report(selected)
    skipped = certification["host_dependent_skips"]
    payload = {
        "schema_version": 1,
        "area": "python_interop",
        "status": report_status(
            env_result,
            selected_groups,
            selected_tiers,
            selected_gates,
            bool(args.package),
        ),
        "groups": selected_groups or ["scaffold"],
        "tiers": selected_tiers,
        "gates": selected_gates or sorted({entry.gate for entry in selected if entry.gate is not None}),
        "packages": sorted({entry.name for entry in selected}),
        "matrix_files": list(MATRIX_FILES),
        "matrix_entries": len(matrices),
        "declaration_capabilities": {
            "file": "declaration_capabilities.json",
            "rows": len(declaration_capabilities["capabilities"]),
            "active": sum(
                row["implementation_status"] == "active"
                for row in declaration_capabilities["capabilities"]
            ),
            "reserved": sum(
                row["implementation_status"] == "reserved"
                for row in declaration_capabilities["capabilities"]
            ),
        },
        "fixture_directories": list(REQUIRED_FIXTURES),
        "fixture_files": list(REQUIRED_FIXTURE_FILES),
        "source_fixtures": list(REQUIRED_SOURCE_FIXTURES),
        "summary": {
            "total_variants": max(1, len(selected)),
            "total_failures": 0,
            "blocking_failures": 0,
            "non_blocking_failures": 0,
            "skipped": skipped,
        },
        "package_certification": certification,
    }
    if env_result is not None:
        payload["env_probe"] = env_result
    report_path = (paths.area_root / args.report).resolve()
    write_report(report_path, payload)
    print(
        f"python interop {payload['status']} ok: report={report_path.relative_to(paths.repo_root)}"
    )
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
            payload = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as error:
            raise SystemExit(f"invalid python interop fixture JSON {path}: {error}") from error
        if name == "sqlite_context/sync_context_evidence.json":
            validate_sync_context_evidence(payload)
        if name == "package_bridge_archive/package_bridge_evidence.json":
            validate_package_bridge_evidence(payload)
        if name == "async_declaration/async_declaration_evidence.json":
            validate_async_declaration_evidence(payload)
        if name == "async_context/async_context_evidence.json":
            validate_async_context_evidence(payload)
        if name == "numpy_buffer/buffer_declaration_evidence.json":
            validate_buffer_declaration_evidence(payload, fixtures_root)
        if name == "pyarrow_capsule/arrow_declaration_evidence.json":
            validate_arrow_declaration_evidence(payload, fixtures_root)
    missing_sources = [
        name for name in REQUIRED_SOURCE_FIXTURES if not (fixtures_root / name).is_file()
    ]
    if missing_sources:
        raise SystemExit(f"missing python interop source fixtures: {', '.join(missing_sources)}")


def validate_sync_context_evidence(payload: object) -> None:
    if not isinstance(payload, dict) or payload.get("capability") != "sync-context":
        raise SystemExit("sync context evidence must identify the sync-context capability")
    required_causes = {
        "normal",
        "return",
        "break",
        "continue",
        "originating Python exception",
        "ordinary Sifr error",
        "timeout, cancellation, or runtime fault",
    }
    outcomes = payload.get("outcome_matrix")
    observed_causes = {
        item.get("cause") for item in outcomes if isinstance(item, dict)
    } if isinstance(outcomes, list) else set()
    missing_causes = sorted(required_causes - observed_causes)
    if missing_causes:
        raise SystemExit(f"sync context evidence is missing causes: {', '.join(missing_causes)}")
    for matrix_name, minimum in (("cleanup_matrix", 3), ("negative_matrix", 3)):
        matrix = payload.get(matrix_name)
        if not isinstance(matrix, list) or len(matrix) < minimum:
            raise SystemExit(f"sync context evidence requires at least {minimum} {matrix_name} rows")
        if any(not isinstance(item, dict) or not item.get("evidence") for item in matrix):
            raise SystemExit(f"sync context evidence {matrix_name} rows require evidence owners")
    live = payload.get("live_example")
    if not isinstance(live, dict) or live.get("stdout_marker") != (
        "sifr-python-interop:sqlite-context:total=71"
    ):
        raise SystemExit("sync context live evidence must lock the SQLite transaction marker")


def validate_package_bridge_evidence(payload: object) -> None:
    if not isinstance(payload, dict) or payload.get("capability") != "package-bridge":
        raise SystemExit("package bridge evidence must identify the package-bridge capability")
    for matrix_name, minimum in (("positive", 3), ("negative", 3), ("cleanup", 3)):
        matrix = payload.get(matrix_name)
        if not isinstance(matrix, list) or len(matrix) < minimum:
            raise SystemExit(
                f"package bridge evidence requires at least {minimum} {matrix_name} rows"
            )
        if any(not isinstance(item, str) or not item for item in matrix):
            raise SystemExit(f"package bridge evidence {matrix_name} rows require owners")
    live = payload.get("live")
    if not isinstance(live, dict) or live.get("stdout_marker") != (
        "sifr-python-interop:package-bridge:gtin=7032069804988:format=13:check=8"
    ):
        raise SystemExit("package bridge live evidence must lock the biip archive marker")


def validate_async_declaration_evidence(payload: object) -> None:
    if not isinstance(payload, dict) or payload.get("capability") != "coroutine-declaration":
        raise SystemExit("async declaration evidence must identify coroutine-declaration")
    for matrix_name, minimum in (
        ("positive", 3),
        ("negative", 3),
        ("cleanup", 3),
        ("cancellation", 3),
    ):
        matrix = payload.get(matrix_name)
        if not isinstance(matrix, list) or len(matrix) < minimum:
            raise SystemExit(
                f"async declaration evidence requires at least {minimum} {matrix_name} rows"
            )
        if any(not isinstance(item, str) or not item for item in matrix):
            raise SystemExit(f"async declaration {matrix_name} rows require evidence owners")
    live = payload.get("live")
    expected = (
        "sifr-python-interop:async-declaration:status=207:message=async-ready:"
        "close=1:loop=shared:failure=covered:conversion=covered"
    )
    if not isinstance(live, dict) or live.get("stdout_marker") != expected:
        raise SystemExit("async declaration live evidence must lock the httpx client marker")


def validate_async_context_evidence(payload: object) -> None:
    if not isinstance(payload, dict) or payload.get("capability") != "async-context":
        raise SystemExit("async context evidence must identify async-context")
    for matrix_name, minimum in (
        ("positive", 3),
        ("negative", 3),
        ("cleanup", 3),
        ("cancellation", 3),
    ):
        matrix = payload.get(matrix_name)
        if not isinstance(matrix, list) or len(matrix) < minimum:
            raise SystemExit(
                f"async context evidence requires at least {minimum} {matrix_name} rows"
            )
        if any(not isinstance(item, str) or not item for item in matrix):
            raise SystemExit(f"async context {matrix_name} rows require evidence owners")
    live = payload.get("live")
    expected = (
        "sifr-python-interop:async-context:value=sqlite-ready:enter=7:exit=7:"
        "close=7:loop=shared:suppression=covered:sifr=unsuppressed:"
        "cancellation=ordered:nested=lifo:exit-failure=covered"
    )
    if not isinstance(live, dict) or live.get("stdout_marker") != expected:
        raise SystemExit("async context live evidence must lock the aiosqlite marker")


def validate_buffer_declaration_evidence(payload: object, fixtures_root: Path) -> None:
    if not isinstance(payload, dict):
        raise SystemExit("buffer evidence must identify buffer-protocol-declaration")
    required_keys = {
        "schema_version",
        "capability",
        "surface",
        "positive",
        "negative",
        "cleanup",
        "cancellation",
        "live",
        "profiles",
    }
    if set(payload) != required_keys:
        raise SystemExit("buffer evidence top-level schema drift")
    if payload.get("schema_version") != 1:
        raise SystemExit("buffer evidence schema_version must be 1")
    if payload.get("capability") != "buffer-protocol-declaration":
        raise SystemExit("buffer evidence must identify buffer-protocol-declaration")
    if payload.get("surface") != "@python.buffer -> Result[python.Buffer[T], PythonError]":
        raise SystemExit("buffer evidence surface drift")
    repo_root = fixtures_root.parents[3]
    for matrix_name, expected_rows in BUFFER_MATRIX_SPECS.items():
        matrix = payload.get(matrix_name)
        if not isinstance(matrix, list) or len(matrix) != len(expected_rows):
            raise SystemExit(f"buffer evidence requires exact {matrix_name} rows")
        ids = [item.get("id") for item in matrix if isinstance(item, dict)]
        if len(ids) != len(matrix) or len(set(ids)) != len(ids) or set(ids) != set(expected_rows):
            raise SystemExit(f"buffer evidence {matrix_name} row id drift")
        for item in matrix:
            if set(item) != {"id", "layer", "evidence", "owners", "covers"}:
                raise SystemExit(f"buffer evidence {matrix_name} row schema drift")
            expected_layer, expected_evidence, expected_owners, expected_coverage = expected_rows[
                item["id"]
            ]
            if item.get("layer") != expected_layer:
                raise SystemExit(f"buffer evidence layer drift: {item['id']}")
            if item.get("evidence") != expected_evidence:
                raise SystemExit(f"buffer evidence description drift: {item['id']}")
            covers = item.get("covers")
            if not isinstance(covers, list) or len(covers) != len(set(covers)) or set(covers) != expected_coverage:
                raise SystemExit(f"buffer evidence coverage drift: {item['id']}")
            owners = item.get("owners")
            if (
                not isinstance(owners, list)
                or len(owners) != len(set(owners))
                or set(owners) != expected_owners
            ):
                raise SystemExit(f"buffer evidence owner drift: {item['id']}")
            for owner in owners:
                if not isinstance(owner, str) or not owner:
                    raise SystemExit(f"buffer evidence owner is invalid: {item['id']}")
                relative_path, separator, symbol = owner.partition("::")
                owner_path = repo_root / relative_path
                if not owner_path.is_file():
                    raise SystemExit(f"buffer evidence owner is missing: {owner}")
                if separator and symbol not in owner_path.read_text(encoding="utf-8"):
                    raise SystemExit(f"buffer evidence owner symbol is missing: {owner}")
    cancellation = payload.get("cancellation")
    if cancellation != {
        "status": "not-applicable",
        "reason": "buffer acquisition and access are synchronous blocking boundaries",
    }:
        raise SystemExit("buffer evidence must record synchronous cancellation as not applicable")
    live = payload.get("live")
    required_live_ids = {
        "import-root-bytearray",
        "receiver-mmap",
        "package-bridge",
        "affine-aggregate",
        "numpy-ndarray",
    }
    if not isinstance(live, list) or len(live) != len(required_live_ids):
        raise SystemExit("buffer live evidence requires exactly five rows")
    if any(not isinstance(item, dict) or set(item) != {"id", "source", "stdout_marker"} for item in live):
        raise SystemExit("buffer live evidence row schema drift")
    observed_live_ids = {item.get("id") for item in live}
    if observed_live_ids != required_live_ids:
        missing = sorted(required_live_ids - observed_live_ids)
        extra = sorted(observed_live_ids - required_live_ids)
        raise SystemExit(f"buffer live evidence drift: missing={missing}, extra={extra}")
    registered_cases = {
        (case.relative_source.removeprefix("numpy_buffer/"), case.stdout_marker)
        for case in BUFFER_EXAMPLE_CASES.values()
    }
    observed_cases = set()
    for item in live:
        source = item.get("source")
        marker = item.get("stdout_marker")
        source_path = fixtures_root / "numpy_buffer" / source if isinstance(source, str) else None
        if source_path is None or not source_path.is_file():
            raise SystemExit(f"buffer live evidence source is missing: {source}")
        if not isinstance(marker, str) or not marker:
            raise SystemExit(f"buffer live evidence marker is missing: {source}")
        if marker not in source_path.read_text(encoding="utf-8"):
            raise SystemExit(f"buffer live evidence marker is absent from source: {source}")
        observed_cases.add((source, marker))
    if observed_cases != registered_cases:
        raise SystemExit("buffer live evidence must match the executable case registry")
    profiles = payload.get("profiles")
    required_profiles = ["create-pr", "merge", "nightly", "release"]
    if profiles != required_profiles:
        raise SystemExit("buffer evidence must remain blocking in every delivery profile")
    manifest_payload = json.loads(
        (repo_root / "verification" / "areas" / "python_interop" / "manifest.json").read_text(
            encoding="utf-8"
        )
    )
    required_suites = {
        "buffer-examples": "python-interop-buffer-examples",
        "buffer-cpython311": "python-interop-buffer-cpython311",
    }
    for suite_name, expected_command in required_suites.items():
        manifest_suites = [
            suite for suite in manifest_payload["suites"] if suite["name"] == suite_name
        ]
        if len(manifest_suites) != 1 or manifest_suites[0].get("kind") != "adapter":
            raise SystemExit(f"{suite_name} manifest ownership drift")
        manifest_cases = manifest_suites[0].get("cases")
        if (
            not isinstance(manifest_cases, list)
            or len(manifest_cases) != 1
            or manifest_cases[0].get("command") != expected_command
        ):
            raise SystemExit(f"{suite_name} manifest command drift")
    for profile in required_profiles:
        profile_payload = json.loads(
            (repo_root / "verification" / "profiles" / f"{profile}.json").read_text(
                encoding="utf-8"
            )
        )
        python_areas = [
            area for area in profile_payload["selected_areas"] if area["area"] == "python_interop"
        ]
        if len(python_areas) != 1:
            raise SystemExit(f"python interop profile ownership drift in {profile}")
        python_suites = python_areas[0]["suites"]
        missing_suites = set(required_suites).difference(python_suites)
        if missing_suites:
            raise SystemExit(
                f"buffer evidence suites are not blocking in {profile}: {sorted(missing_suites)}"
            )


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


def report_status(
    env_result: dict[str, object] | None,
    groups: list[str],
    tiers: list[str],
    gates: list[str],
    has_package_filter: bool,
) -> str:
    if env_result is not None:
        return "passed"
    if tiers or gates or has_package_filter or (groups and groups != ["scaffold"]):
        return "matrix-passed"
    return "scaffold"


def run_self_tests(area_root: Path) -> None:
    entries = load_matrices(area_root / "packages")
    validate_matrix_entries(entries)
    validate_certification_policy(entries)
    repo_root = area_root.parents[2]
    run_declaration_capability_self_tests(area_root, repo_root)
    validate_fixture_files(area_root / "fixtures")
    evidence = json.loads(
        (area_root / "fixtures/sqlite_context/sync_context_evidence.json").read_text(
            encoding="utf-8"
        )
    )
    evidence["outcome_matrix"] = [
        item for item in evidence["outcome_matrix"] if item["cause"] != "normal"
    ]
    try:
        validate_sync_context_evidence(evidence)
    except SystemExit as error:
        if "missing causes: normal" not in str(error):
            raise
    else:
        raise SystemExit("sync context evidence self-test accepted a missing normal outcome")
    buffer_evidence = json.loads(
        (area_root / "fixtures/numpy_buffer/buffer_declaration_evidence.json").read_text(
            encoding="utf-8"
        )
    )
    buffer_mutations = []
    invalid_schema = json.loads(json.dumps(buffer_evidence))
    invalid_schema["schema_version"] = 999
    buffer_mutations.append((invalid_schema, "schema_version"))
    duplicate_matrix = json.loads(json.dumps(buffer_evidence))
    duplicate_matrix["positive"][1]["id"] = duplicate_matrix["positive"][0]["id"]
    buffer_mutations.append((duplicate_matrix, "row id drift"))
    missing_owner = json.loads(json.dumps(buffer_evidence))
    missing_owner["cleanup"][0]["owners"][0] = "crates/missing.rs::missing_test"
    buffer_mutations.append((missing_owner, "owner drift"))
    unrelated_owner = json.loads(json.dumps(buffer_evidence))
    unrelated_owner["positive"][0]["owners"] = ["README.md"]
    buffer_mutations.append((unrelated_owner, "owner drift"))
    fabricated_evidence = json.loads(json.dumps(buffer_evidence))
    fabricated_evidence["positive"][0]["evidence"] = "fabricated but nonempty"
    buffer_mutations.append((fabricated_evidence, "description drift"))
    missing_reason = json.loads(json.dumps(buffer_evidence))
    del missing_reason["cancellation"]["reason"]
    buffer_mutations.append((missing_reason, "synchronous cancellation"))
    duplicate_live = json.loads(json.dumps(buffer_evidence))
    duplicate_live["live"].append(duplicate_live["live"][0])
    buffer_mutations.append((duplicate_live, "exactly five rows"))
    missing_profile = json.loads(json.dumps(buffer_evidence))
    missing_profile["profiles"].remove("release")
    buffer_mutations.append((missing_profile, "every delivery profile"))
    for mutated, expected_error in buffer_mutations:
        try:
            validate_buffer_declaration_evidence(mutated, area_root / "fixtures")
        except SystemExit as error:
            if expected_error not in str(error):
                raise
        else:
            raise SystemExit(f"buffer evidence self-test accepted mutation: {expected_error}")
    arrow_evidence = json.loads(
        (area_root / "fixtures/pyarrow_capsule/arrow_declaration_evidence.json").read_text(
            encoding="utf-8"
        )
    )
    arrow_mutations = []
    invalid_arrow_schema = json.loads(json.dumps(arrow_evidence))
    invalid_arrow_schema["schema_version"] = 1
    arrow_mutations.append((invalid_arrow_schema, "schema_version"))
    missing_arrow_owner = json.loads(json.dumps(arrow_evidence))
    missing_arrow_owner["positive"][0]["owners"][0] = "crates/missing.rs"
    arrow_mutations.append((missing_arrow_owner, "owner is missing"))
    drifted_arrow_target = json.loads(json.dumps(arrow_evidence))
    drifted_arrow_target["live"][0]["targets"].remove("pyarrow.array")
    arrow_mutations.append((drifted_arrow_target, "target drift"))
    missing_arrow_profile = json.loads(json.dumps(arrow_evidence))
    missing_arrow_profile["profiles"].remove("release")
    arrow_mutations.append((missing_arrow_profile, "every delivery profile"))
    for mutated, expected_error in arrow_mutations:
        try:
            validate_arrow_declaration_evidence(mutated, area_root / "fixtures")
        except SystemExit as error:
            if expected_error not in str(error):
                raise
        else:
            raise SystemExit(f"Arrow evidence self-test accepted mutation: {expected_error}")
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
    try:
        validate_certification_policy([PackageEntry("bad-tier4", "tier4", ("imports",))])
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test failed: tier4 package without skip was accepted")
    payload = build_certification_report([
        PackageEntry(name="requests", tier="tier1", groups=("imports",), gate="tier1a"),
        PackageEntry(
            name="watchdog",
            tier="tier4",
            groups=("imports",),
            host_dependent=True,
            skip_reason="requires host filesystem event backend",
        ),
    ])
    if payload["certified_packages"] != 1 or payload["host_dependent_skips"] != 1:
        raise SystemExit("negative self-test failed: package status counts drifted")


if __name__ == "__main__":
    raise SystemExit(main())
