from __future__ import annotations

from collections.abc import Callable
from typing import Any

from env import RunnerPaths
from example_packages import ExampleCase, build_examples_report, run_examples_self_tests


ASYNC_DECLARATION_CASES = {
    "httpx-client": ExampleCase(
        case_id="httpx-client",
        relative_source="async_declaration/httpx_client.sifr",
        stdout_marker=(
            "sifr-python-interop:async-declaration:status=207:message=async-ready:"
            "close=1:loop=shared:failure=covered:conversion=covered"
        ),
        import_roots=("asyncio", "fastapi", "httpx", "threading"),
        native_roots=(),
    ),
}


def build_async_declaration_examples_report(
    paths: RunnerPaths,
    example_runner: Callable[[RunnerPaths], list[dict[str, Any]]] | None = None,
) -> dict[str, Any]:
    return build_examples_report(
        paths,
        suite_name="async-declaration",
        cases_by_id=ASYNC_DECLARATION_CASES,
        example_runner=example_runner,
    )


def run_async_declaration_examples_self_tests(paths: RunnerPaths) -> None:
    run_examples_self_tests(
        paths,
        suite_name="async-declaration",
        cases_by_id=ASYNC_DECLARATION_CASES,
    )
