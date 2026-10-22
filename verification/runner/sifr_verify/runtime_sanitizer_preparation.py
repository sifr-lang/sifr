"""Prepare selected instrumented Cargo graphs without executing assertions."""
from __future__ import annotations

import argparse
import importlib.util
import json

from .paths import REPO_ROOT
from .profile_commands import run_command


SANITIZER_SUITES = frozenset({"sanitizer-smoke", "sanitizer-full"})


def load_adapter():
    path = REPO_ROOT / "verification/areas/runtime_platform/runner.py"
    spec = importlib.util.spec_from_file_location("_runtime_sanitizer_preparation", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load sanitizer authority: {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def preparation_command(command: list[str]) -> list[str]:
    """Retain Cargo selection; remove only run arguments or disable test execution."""
    if not command or command[0] != "cargo":
        raise ValueError("instrumented preparation requires Cargo")
    action = 2 if len(command) > 1 and command[1].startswith("+") else 1
    boundary = command.index("--") if "--" in command else len(command)
    prepared = command[:boundary]
    if len(prepared) <= action:
        raise ValueError("instrumented preparation requires a Cargo action")
    if prepared[action] == "run":
        prepared[action] = "build"
    elif prepared[action] == "test":
        prepared.insert(action + 1, "--no-run")
    else:
        raise ValueError(f"unsupported instrumented preparation action: {prepared[action]}")
    return prepared


def prepare(suites: set[str], *, run=run_command, adapter=None) -> None:
    if not suites or not suites <= SANITIZER_SUITES:
        raise ValueError("select a nonempty set of declared sanitizer suites")
    adapter = load_adapter() if adapter is None else adapter
    manifest = adapter.load_sanitizer_manifest()
    host = adapter.current_rust_host_triple()
    for case in manifest["cases"]:
        if not suites.intersection(case["suites"]):
            continue
        # Miri/model cases retain their distinct tool protocol. This preparer
        # owns the explicit ASan/LSan/TSan Cargo configurations only.
        if case["tool"] not in {"asan", "lsan", "tsan"}:
            continue
        adapter.validate_sanitizer_inputs(case)
        reasons = adapter.sanitizer_skip_reasons(case, host)
        if reasons:
            print(json.dumps({"sanitizer_preparation": case["id"],
                              "status": "unavailable", "reasons": reasons}), flush=True)
            # Execution still reports the original skip; preparation never
            # fabricates an assertion pass for an unavailable graph.
            continue
        command = preparation_command(adapter.sanitizer_command(case, host))
        env = adapter.sanitizer_environment(case, host)
        print(json.dumps({"sanitizer_preparation": case["id"],
                          "status": "start", "command": command}), flush=True)
        run(command, env=env)
        print(json.dumps({"sanitizer_preparation": case["id"],
                          "status": "prepared", "assertions_executed": False}), flush=True)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite", action="append", required=True,
                        choices=sorted(SANITIZER_SUITES))
    prepare(set(parser.parse_args().suite))


if __name__ == "__main__":
    main()
