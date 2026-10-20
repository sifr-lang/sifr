"""Pinned Python environment ownership for selected verification consumers."""
from .area_cargo_setup import sql_preparation_commands
from .paths import REPO_ROOT
from .profiles import crate_test_mode, crate_test_suites_for_mode


def python_consumers(profile):
    consumers = set()
    for area in profile.get("selected_areas", []):
        if area["area"] == "python_interop":
            consumers.add("area_python_interop")
        elif area["area"] == "sql_platform":
            commands = sql_preparation_commands(area["suites"])
            if any(any(flag in ("-p", "--package") and package == "sifr_lsp"
                       for flag, package in zip(command, command[1:]))
                   for command in commands):
                consumers.add("area_sql_platform")
    mode = crate_test_mode(profile)
    if mode is not None:
        suites = crate_test_suites_for_mode(profile, mode)
        if any(suite["package"] in {"sifr_driver", "sifr_lsp"}
               and not (suite["status"] == "red-blocker" and not suite["executed_in_merge"])
               for suite in suites):
            consumers.add(f"toolchain_cargo_test_sifr_{mode}")
    return consumers


def prepare_python_environment(env, run):
    area = REPO_ROOT / "verification/areas/python_interop"
    active = env.copy()
    active.pop("VIRTUAL_ENV", None)
    # Rust fixtures consume this project-owned environment directly. An ambient
    # uv override must not populate a different environment and leave them broken.
    active["UV_PROJECT_ENVIRONMENT"] = str(area / ".venv")
    run(["uv", "run", "--project", str(area), "--locked", "python",
         str(area / "runner/prepare_environment.py")], env=active)
