from __future__ import annotations

from collections.abc import Callable
from typing import Any

from env import RunnerPaths
from example_packages import ExampleCase, build_examples_report, run_examples_self_tests

ARROW_EXAMPLE_CASES = {
    "pyarrow-pandas-polars": ExampleCase(
        case_id="pyarrow-pandas-polars",
        relative_source="pyarrow_capsule/arrow_declaration_compiled.sifr",
        stdout_marker=(
            "sifr-python-interop:arrow:pyarrow=array:schema=parameter:pandas=stream:"
            "polars=stream:owned-transfer=len3:method-transfer=len3:rollback=ok:resources=zero"
        ),
        import_roots=("builtins", "pandas", "polars", "pyarrow"),
        native_roots=("pandas", "polars", "pyarrow"),
        copy_bridges=False,
        arrow_certifications=(
            ("pandas.DataFrame", "python_certifications/arrow_evidence.py"),
            ("polars.Series", "python_certifications/arrow_evidence.py"),
            ("pyarrow.Array", "python_certifications/arrow_evidence.py"),
            ("pyarrow.array", "python_certifications/arrow_evidence.py"),
            ("pyarrow.int64", "python_certifications/arrow_evidence.py"),
        ),
        explicit_requirements=False,
    ),
}


def build_arrow_examples_report(
    paths: RunnerPaths,
    example_runner: Callable[[RunnerPaths], list[dict[str, Any]]] | None = None,
) -> dict[str, Any]:
    return build_examples_report(
        paths,
        suite_name="arrow",
        cases_by_id=ARROW_EXAMPLE_CASES,
        example_runner=example_runner,
    )


def run_arrow_examples_self_tests(paths: RunnerPaths) -> None:
    run_examples_self_tests(
        paths,
        suite_name="arrow",
        cases_by_id=ARROW_EXAMPLE_CASES,
    )
    case = ARROW_EXAMPLE_CASES["pyarrow-pandas-polars"]
    if len(case.arrow_certifications) != 5:
        raise SystemExit("Arrow example certification matrix drift")
