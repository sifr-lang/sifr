from __future__ import annotations

from collections.abc import Callable
from typing import Any

from env import RunnerPaths
from example_packages import ExampleCase, build_examples_report, run_examples_self_tests

ML_EXAMPLE_CASES = {
    "torch": ExampleCase(
        case_id="torch",
        relative_source="torch_dlpack/torch_full_example.sifr",
        stdout_marker="sifr-python-interop:torch:sum=42.0:shape=2x3:dtype=float32",
        import_roots=("torch",),
    ),
    "scikit-learn": ExampleCase(
        case_id="scikit-learn",
        relative_source="sklearn/sklearn_full_example.sifr",
        stdout_marker="sifr-python-interop:sklearn:predictions=0,1:classes=0,1",
        import_roots=("numpy", "sklearn"),
    ),
}


def build_ml_examples_report(
    paths: RunnerPaths,
    example_runner: Callable[[RunnerPaths], list[dict[str, Any]]] | None = None,
) -> dict[str, Any]:
    return build_examples_report(
        paths,
        suite_name="ml",
        cases_by_id=ML_EXAMPLE_CASES,
        example_runner=example_runner,
    )


def run_ml_examples_self_tests(paths: RunnerPaths) -> None:
    run_examples_self_tests(
        paths,
        suite_name="ml",
        cases_by_id=ML_EXAMPLE_CASES,
    )
