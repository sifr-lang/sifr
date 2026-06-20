from __future__ import annotations

from collections.abc import Callable
from typing import Any

from env import RunnerPaths
from example_packages import ExampleCase, build_examples_report, run_examples_self_tests

DATAFRAME_EXAMPLE_CASES = {
    "numpy": ExampleCase(
        case_id="numpy",
        relative_source="numpy_buffer/numpy_full_example.sifr",
        stdout_marker="sifr-python-interop:numpy:sum=20:values=2,4,6,8",
        import_roots=("numpy",),
    ),
    "pandas": ExampleCase(
        case_id="pandas",
        relative_source="pandas_arrow/pandas_full_example.sifr",
        stdout_marker="sifr-python-interop:pandas:double-total=20:values=2,3,5",
        import_roots=("numpy", "pandas"),
    ),
    "polars": ExampleCase(
        case_id="polars",
        relative_source="polars_arrow/polars_full_example.sifr",
        stdout_marker="sifr-python-interop:polars:sum=10:first-city=oslo",
        import_roots=("polars",),
    ),
}


def build_dataframe_examples_report(
    paths: RunnerPaths,
    example_runner: Callable[[RunnerPaths], list[dict[str, Any]]] | None = None,
) -> dict[str, Any]:
    return build_examples_report(
        paths,
        suite_name="dataframe",
        cases_by_id=DATAFRAME_EXAMPLE_CASES,
        example_runner=example_runner,
    )


def run_dataframe_examples_self_tests(paths: RunnerPaths) -> None:
    run_examples_self_tests(
        paths,
        suite_name="dataframe",
        cases_by_id=DATAFRAME_EXAMPLE_CASES,
    )
