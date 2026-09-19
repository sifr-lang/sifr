"""Charge exact native example compilation to explicit setup, without runtime claims."""
from __future__ import annotations
import argparse
import importlib
import json
import time
from env import discover_paths, require_canonical_python
from example_packages import (
    prepare_example_package, certification_commands_for, _run_sifr_command,
    package_snapshot,
)

# Each row uses the same registry, package name and interpreter as execution.
EXAMPLES = {
    "callback-examples": ("callback_examples", "CALLBACK_CASES", "callback"),
    "dataframe-examples": ("dataframe_examples", "DATAFRAME_EXAMPLE_CASES", "dataframe"),
    "buffer-examples": ("buffer_examples", "BUFFER_EXAMPLE_CASES", "buffer"),
    "arrow-examples": ("arrow_examples", "ARROW_EXAMPLE_CASES", "arrow"),
    "dlpack-examples": ("dlpack_examples", "DLPACK_EXAMPLE_CASES", "dlpack"),
    "ml": ("ml_examples", "ML_EXAMPLE_CASES", "ml"),
    "libraries": ("library_examples", "LIBRARY_EXAMPLE_CASES", "library"),
    "async-declaration-examples": ("async_declaration_examples", "ASYNC_DECLARATION_CASES", "async-declaration"),
    "async-context-examples": ("async_context_examples", "ASYNC_CONTEXT_CASES", "async-context"),
}


def prepare(suites, paths=None):
    paths = paths or discover_paths()
    require_canonical_python(paths.area_root)
    for suite in suites:
        module_name, registry, package_name = EXAMPLES[suite]
        cases = getattr(importlib.import_module(module_name), registry)
        for case in cases.values():
            start = time.monotonic()
            package = prepare_example_package(paths, package_name, case)
            for command in [*certification_commands_for(case), ["build", "--release"]]:
                before = package_snapshot(package) if command[-1] == "--check" else None
                result = _run_sifr_command(paths, package, command)
                if result["exit_code"] != 0:
                    raise RuntimeError(json.dumps(result))
                if before is not None and before != package_snapshot(package):
                    raise RuntimeError("certification recheck mutated package inputs")
            print(json.dumps({"preparation": "python-example", "suite": suite,
                              "case": case.case_id, "elapsed_seconds": time.monotonic()-start,
                              "status": "built", "runtime_assertions": "not-executed"}), flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite", action="append", required=True, choices=EXAMPLES)
    prepare(parser.parse_args().suite)
