"""Prepare exact selected area graphs without running their assertions."""
from __future__ import annotations
import importlib.util
import json
import sys
from .paths import REPO_ROOT


def sql_preparation_commands(suites):
    path = REPO_ROOT / "verification/areas/sql_platform/runner.py"
    spec = importlib.util.spec_from_file_location("_sql_preparation_runner", path)
    if spec is None or spec.loader is None:
        raise AssertionError(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    manifest = json.loads(module.MANIFEST_PATH.read_text())
    commands = []
    for suite in module.select_suites(manifest, set(suites)):
        for case in suite["cases"]:
            command = module.COMMANDS[case["command"]]
            if command[:2] == ["cargo", "test"]:
                # Test filters do not change Cargo's compilation graph. Keep
                # every argument so feature/target selection cannot drift.
                prepared = [*command[:2], "--no-run", *command[2:]]
                if prepared not in commands:
                    commands.append(prepared)
    return commands


def prepare_area_graphs(profile, env, run):
    for area in profile.get("selected_areas", []):
        name, suites = area["area"], area["suites"]
        if name == "sql_platform":
            if "host-tools" in suites:
                run([sys.executable, str(REPO_ROOT / "verification/areas/sql_platform/tools/check_compiler_execution.py"),
                     "--prepare-only"], env=env)
            for command in sql_preparation_commands(suites):
                run(command, env=env)
        if name == "fuzz_property" and "fuzz-smoke" in suites:
            run(["cargo", "build", "--locked", "--offline", "-p", "sifr_driver",
                 "--bin", "diagnostic_rendering_harness"], env=env)
        if name == "runtime_platform":
            from .runtime_sanitizer_preparation import SANITIZER_SUITES
            selected = sorted(set(suites).intersection(SANITIZER_SUITES))
            if selected:
                command = [sys.executable, "-m", "sifr_verify.runtime_sanitizer_preparation"]
                for suite in selected:
                    command.extend(["--suite", suite])
                run(command, env=env)
