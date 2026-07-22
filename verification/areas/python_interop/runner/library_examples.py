from __future__ import annotations

from collections.abc import Callable
from typing import Any

from env import RunnerPaths
from example_packages import ExampleCase, build_examples_report, run_examples_self_tests

LIBRARY_EXAMPLE_CASES = {
    "biip-schwifty": ExampleCase(
        case_id="biip-schwifty",
        relative_source="simple_import/biip_schwifty_full_example.sifr",
        stdout_marker="sifr-python-interop:biip-schwifty:gtin=7032069804988:bic=DEUTDEFF",
        import_roots=("biip", "builtins", "schwifty"),
        native_roots=(),
    ),
    "pyarrow": ExampleCase(
        case_id="pyarrow",
        relative_source="pyarrow_capsule/pyarrow_full_example.sifr",
        stdout_marker="sifr-python-interop:pyarrow:sum=10:kind=array:producer=pyarrow.lib",
        import_roots=("pyarrow",),
        native_roots=("pyarrow",),
        bridge_files=("pyarrow_example.py",),
    ),
    "fastapi-pydantic": ExampleCase(
        case_id="fastapi-pydantic",
        relative_source="fastapi_app/fastapi_pydantic_full_example.sifr",
        stdout_marker="sifr-python-interop:fastapi-pydantic:value=42:title=Sifr API:status=201",
        import_roots=("builtins", "fastapi", "pydantic", "pydantic_core", "starlette"),
        native_roots=("pydantic_core",),
        bridge_files=("fastapi_pydantic_example.py",),
    ),
    "cryptography-cffi": ExampleCase(
        case_id="cryptography-cffi",
        relative_source="cryptography_tls/cryptography_cffi_full_example.sifr",
        stdout_marker="sifr-python-interop:cryptography-cffi:roundtrip=sifr-secret:certifi=ok",
        import_roots=("certifi", "cffi", "cryptography"),
        native_roots=("cffi", "cryptography"),
        bridge_files=("cryptography_cffi_example.py",),
    ),
    "boto3-botocore": ExampleCase(
        case_id="boto3-botocore",
        relative_source="aws_sqs/boto3_botocore_full_example.sifr",
        stdout_marker="sifr-python-interop:boto3-botocore:queue=https://sqs.us-east-1.amazonaws.com/123456789012/sifr-queue",
        import_roots=("boto3", "botocore"),
        native_roots=(),
        bridge_files=("boto3_botocore_example.py",),
    ),
    "redis-fakeredis": ExampleCase(
        case_id="redis-fakeredis",
        relative_source="redis/redis_fakeredis_full_example.sifr",
        stdout_marker="sifr-python-interop:redis-fakeredis:value=ready:reply=PONG",
        import_roots=("fakeredis", "hiredis", "redis"),
        native_roots=("hiredis",),
        bridge_files=("redis_fakeredis_example.py",),
    ),
    "sqlalchemy-psycopg": ExampleCase(
        case_id="sqlalchemy-psycopg",
        relative_source="sqlalchemy_psycopg/sqlalchemy_psycopg_full_example.sifr",
        stdout_marker="sifr-python-interop:sqlalchemy-psycopg:scalar=42:dialect=sqlite:conninfo=ok",
        import_roots=("alembic", "psycopg", "sqlalchemy"),
        native_roots=("psycopg", "sqlalchemy"),
        bridge_files=("sqlalchemy_psycopg_example.py",),
    ),
    "sqlite-context": ExampleCase(
        case_id="sqlite-context",
        relative_source="sqlite_context/context_codegen_smoke.sifr",
        stdout_marker="sifr-python-interop:sqlite-context:total=71",
        import_roots=("sqlite3",),
        native_roots=("sqlite3",),
    ),
}


def build_library_examples_report(
    paths: RunnerPaths,
    example_runner: Callable[[RunnerPaths], list[dict[str, Any]]] | None = None,
) -> dict[str, Any]:
    return build_examples_report(
        paths,
        suite_name="library",
        cases_by_id=LIBRARY_EXAMPLE_CASES,
        example_runner=example_runner,
    )


def run_library_examples_self_tests(paths: RunnerPaths) -> None:
    run_examples_self_tests(
        paths,
        suite_name="library",
        cases_by_id=LIBRARY_EXAMPLE_CASES,
    )
