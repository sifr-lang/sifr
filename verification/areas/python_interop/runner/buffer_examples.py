from __future__ import annotations

from collections.abc import Callable
from typing import Any

from env import RunnerPaths
from example_packages import ExampleCase, build_examples_report, run_examples_self_tests

BUFFER_EXAMPLE_CASES = {
    "top-level": ExampleCase(
        case_id="top-level",
        relative_source="numpy_buffer/buffer_declaration_codegen_smoke.sifr",
        stdout_marker="sifr-python-interop:buffer:top-level=ok:resources=zero",
        import_roots=("builtins", "mmap"),
    ),
    "receiver": ExampleCase(
        case_id="receiver",
        relative_source="numpy_buffer/buffer_declaration_self.sifr",
        stdout_marker="sifr-python-interop:buffer:receiver=ok:resources=zero",
        import_roots=("builtins", "mmap"),
    ),
    "bridge": ExampleCase(
        case_id="bridge",
        relative_source="numpy_buffer/buffer_declaration_bridge.sifr",
        stdout_marker="sifr-python-interop:buffer:bridge=ok:resources=zero",
        import_roots=("builtins", "mmap"),
    ),
    "affine-aggregate": ExampleCase(
        case_id="affine-aggregate",
        relative_source="numpy_buffer/buffer_affine_aggregate_codegen.sifr",
        stdout_marker="sifr-python-interop:buffer:affine-aggregate=compiled",
        import_roots=("builtins", "mmap"),
    ),
}


def build_buffer_examples_report(
    paths: RunnerPaths,
    example_runner: Callable[[RunnerPaths], list[dict[str, Any]]] | None = None,
) -> dict[str, Any]:
    return build_examples_report(
        paths,
        suite_name="buffer",
        cases_by_id=BUFFER_EXAMPLE_CASES,
        example_runner=example_runner,
    )


def run_buffer_examples_self_tests(paths: RunnerPaths) -> None:
    run_examples_self_tests(
        paths,
        suite_name="buffer",
        cases_by_id=BUFFER_EXAMPLE_CASES,
    )
