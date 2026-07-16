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
        import_roots=("builtins",),
        native_roots=(),
        copy_bridges=False,
    ),
    "receiver": ExampleCase(
        case_id="receiver",
        relative_source="numpy_buffer/buffer_declaration_self.sifr",
        stdout_marker="sifr-python-interop:buffer:receiver=ok:resources=zero",
        import_roots=("mmap",),
        native_roots=("mmap",),
        bridge_files=("buffer_owner.py",),
    ),
    "bridge": ExampleCase(
        case_id="bridge",
        relative_source="numpy_buffer/buffer_declaration_bridge.sifr",
        stdout_marker="sifr-python-interop:buffer:bridge=ok:resources=zero",
        import_roots=("ctypes",),
        native_roots=("ctypes",),
        bridge_files=("buffer_bytes.py",),
    ),
    "affine-aggregate": ExampleCase(
        case_id="affine-aggregate",
        relative_source="numpy_buffer/buffer_affine_aggregate_codegen.sifr",
        stdout_marker="sifr-python-interop:buffer:affine-aggregate=ok:resources=zero",
        import_roots=("ctypes",),
        native_roots=("ctypes",),
        bridge_files=("buffer_bytes.py",),
    ),
    "numpy": ExampleCase(
        case_id="numpy",
        relative_source="numpy_buffer/buffer_declaration_numpy.sifr",
        stdout_marker="sifr-python-interop:buffer:numpy=int64:write=42:identity=shared:resources=zero",
        import_roots=("numpy",),
        native_roots=("numpy",),
        bridge_files=("numpy_buffer.py",),
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
    observed_roots = {
        case_id: (case.import_roots, case.native_roots, case.copy_bridges, case.bridge_files)
        for case_id, case in BUFFER_EXAMPLE_CASES.items()
    }
    expected_roots = {
        "top-level": (("builtins",), (), False, None),
        "receiver": (("mmap",), ("mmap",), True, ("buffer_owner.py",)),
        "bridge": (("ctypes",), ("ctypes",), True, ("buffer_bytes.py",)),
        "affine-aggregate": (("ctypes",), ("ctypes",), True, ("buffer_bytes.py",)),
        "numpy": (("numpy",), ("numpy",), True, ("numpy_buffer.py",)),
    }
    if observed_roots != expected_roots:
        raise SystemExit(f"buffer example trust-root drift: {observed_roots}")
