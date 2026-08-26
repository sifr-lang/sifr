from __future__ import annotations

from collections.abc import Callable
from typing import Any

from env import RunnerPaths
from example_packages import ExampleCase, build_examples_report, run_examples_self_tests

CALLBACK_CASES = {
    "cffi-current-thread": ExampleCase(
        case_id="cffi-current-thread",
        relative_source="cffi_callback/current_declaration_callback.sifr",
        stdout_marker="sifr-python-interop:callback:cffi-current=42",
        import_roots=("cffi", "threading"),
        native_roots=("cffi",),
    ),
    "cffi-foreign-thread": ExampleCase(
        case_id="cffi-foreign-thread",
        relative_source="cffi_callback/declaration_callback.sifr",
        stdout_marker="sifr-python-interop:callback:cffi=42",
        import_roots=("cffi", "threading"),
        native_roots=("cffi",),
    ),
    "kafka-foreign": ExampleCase(
        case_id="kafka-foreign",
        relative_source="kafka/declaration_callback.sifr",
        stdout_marker=(
            "sifr-python-interop:callback:kafka=version=3.0.11:"
            "schema=DescribeClusterRequest-v2:fields=3:ack=42"
        ),
        import_roots=("kafka", "threading"),
        native_roots=(),
    ),
    "asyncio-roundtrip": ExampleCase(
        case_id="asyncio-roundtrip",
        relative_source="callback/asyncio_roundtrip.sifr",
        stdout_marker="sifr-python-interop:callback:asyncio=42",
        import_roots=("asyncio", "gc", "types", "weakref"),
        native_roots=(),
    ),
    "callback-reconciliation": ExampleCase(
        case_id="callback-reconciliation",
        relative_source="callback/reconciliation.sifr",
        stdout_marker=(
            "sifr-python-interop:callback:reconciliation="
            "provisional-cancelled:enter-cancelled:enter-closed:"
            "call-closed:exit-skipped:python-handler-error:call-released:"
            "sifr-handler-error"
        ),
        import_roots=("asyncio", "gc", "types", "weakref"),
        native_roots=(),
    ),
    "sync-context-reconciliation": ExampleCase(
        case_id="sync-context-reconciliation",
        relative_source="callback/sync_context_reconciliation.sifr",
        stdout_marker=(
            "sifr-python-interop:callback:sync-context="
            "sync-closed:sync-handler-success:sync-call-closed:"
            "sync-exit-skipped:python-handler-error:sync-call-released:"
            "sifr-handler-error"
        ),
        import_roots=("asyncio", "gc", "types", "weakref"),
        native_roots=(),
    ),
    "pubsub-retained-async-close": ExampleCase(
        case_id="pubsub-retained-async-close",
        relative_source="pubsub/declaration_callback.sifr",
        stdout_marker="sifr-python-interop:callback:pubsub=42:close=drained",
        import_roots=("asyncio", "types"),
        native_roots=(),
    ),
}


def build_callback_examples_report(
    paths: RunnerPaths,
    example_runner: Callable[[RunnerPaths], list[dict[str, Any]]] | None = None,
) -> dict[str, Any]:
    return build_examples_report(
        paths,
        suite_name="callback",
        cases_by_id=CALLBACK_CASES,
        example_runner=example_runner,
    )


def run_callback_examples_self_tests(paths: RunnerPaths) -> None:
    run_examples_self_tests(
        paths,
        suite_name="callback",
        cases_by_id=CALLBACK_CASES,
    )
