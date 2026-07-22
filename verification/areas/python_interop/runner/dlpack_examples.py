from __future__ import annotations

from collections.abc import Callable
from typing import Any

from env import RunnerPaths
from example_packages import ExampleCase, build_examples_report, run_examples_self_tests


DLPACK_EXAMPLE_CASES = {
    "torch-declaration": ExampleCase(
        case_id="torch-declaration",
        relative_source="torch_dlpack/dlpack_declaration_compiled.sifr",
        stdout_marker=(
            "sifr-python-interop:dlpack:torch:pointer=stable:device=cpu:"
            "one-shot=ok:resources=zero"
        ),
        import_roots=("torch",),
        dlpack_certifications=(("torch.Tensor", "python_certifications/dlpack_evidence.py"),),
    ),
    "tensorflow-declaration": ExampleCase(
        case_id="tensorflow-declaration",
        relative_source="tensorflow_dlpack/dlpack_declaration_compiled.sifr",
        stdout_marker=(
            "sifr-python-interop:dlpack:tensorflow:pointer=stable:device=cpu:"
            "bridge=versioned-call:resources=zero"
        ),
        import_roots=("ctypes", "tensorflow"),
        native_roots=("tensorflow",),
        bridge_files=("tensorflow_dlpack.py",),
        dlpack_certifications=((
            "bridge.tensorflow_dlpack.make",
            "python_certifications/dlpack_evidence.py",
        ),),
    ),
}


def build_dlpack_examples_report(
    paths: RunnerPaths,
    example_runner: Callable[[RunnerPaths], list[dict[str, Any]]] | None = None,
) -> dict[str, Any]:
    return build_examples_report(
        paths,
        suite_name="dlpack",
        cases_by_id=DLPACK_EXAMPLE_CASES,
        example_runner=example_runner,
    )


def run_dlpack_examples_self_tests(paths: RunnerPaths) -> None:
    run_examples_self_tests(
        paths,
        suite_name="dlpack",
        cases_by_id=DLPACK_EXAMPLE_CASES,
    )
