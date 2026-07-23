from __future__ import annotations

from collections.abc import Callable
from typing import Any

from env import RunnerPaths
from example_packages import ExampleCase, build_examples_report, run_examples_self_tests


ASYNC_CONTEXT_CASES = {
    "aiosqlite-session": ExampleCase(
        case_id="aiosqlite-session",
        relative_source="async_context/aiosqlite_session.sifr",
        stdout_marker=(
            "sifr-python-interop:async-context:value=sqlite-ready:enter=8:exit=8:"
            "close=8:loop=shared:suppression=covered:sifr=unsuppressed:"
            "python-error=unsuppressed:"
            "cancellation=ordered:nested=lifo:exit-failure=covered"
        ),
        import_roots=("aiosqlite", "asyncio", "contextlib", "sqlite3", "threading"),
        native_roots=(),
    ),
}


def build_async_context_examples_report(
    paths: RunnerPaths,
    example_runner: Callable[[RunnerPaths], list[dict[str, Any]]] | None = None,
) -> dict[str, Any]:
    return build_examples_report(
        paths,
        suite_name="async-context",
        cases_by_id=ASYNC_CONTEXT_CASES,
        example_runner=example_runner,
    )


def run_async_context_examples_self_tests(paths: RunnerPaths) -> None:
    run_examples_self_tests(
        paths,
        suite_name="async-context",
        cases_by_id=ASYNC_CONTEXT_CASES,
    )
