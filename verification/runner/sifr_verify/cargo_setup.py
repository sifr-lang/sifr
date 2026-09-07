"""Resolve the canonical Cargo cache preparation for validation profiles."""

from __future__ import annotations

import os
import shlex
import subprocess
import sys
from typing import Any, Callable

from .paths import REPO_ROOT

CANONICAL_SETUP_COMMAND = "cargo fetch --locked"


def cargo_setup_command(profile: dict[str, Any]) -> list[str]:
    """Return the one supported profile cache-setup command."""
    policy = profile.get("cargo_policy")
    if not isinstance(policy, dict):
        raise ValueError("profile cargo_policy must be an object")
    if policy.get("locked") is not True:
        raise ValueError("profile Cargo execution must be locked")
    if not isinstance(policy.get("offline"), bool):
        raise ValueError("profile Cargo offline policy must be a boolean")
    if policy.get("setup_command") != CANONICAL_SETUP_COMMAND:
        raise ValueError(
            f"profile cargo_policy.setup_command must be {CANONICAL_SETUP_COMMAND!r}"
        )
    return shlex.split(CANONICAL_SETUP_COMMAND)


def prepare_cargo_cache(
    profile: dict[str, Any],
    env: dict[str, str],
    command_runner: Callable[..., None],
) -> None:
    """Populate workspace and generated lock graphs before offline execution."""
    command = cargo_setup_command(profile)
    setup_env = env.copy()
    setup_env.pop("CARGO_NET_OFFLINE", None)
    print(f"[sifr-profile-setup] command={' '.join(command)}")
    command_runner(command, env=setup_env)
    if any(area["area"] == "generated_code_quality" for area in profile.get("selected_areas", [])):
        revision = subprocess.check_output(
            ["git", "rev-parse", "--verify", "HEAD^{commit}"], cwd=REPO_ROOT, text=True
        ).strip()
        # A source-identical later commit still names a different Cargo Git source.
        shared_root = REPO_ROOT / "target" / "sifr_generated_code_quality" / f"{profile['name']}.{revision}.shared"
        env["SIFR_GCQ_SHARED_ROOT"] = str(shared_root)
        setup_env["SIFR_GCQ_SHARED_ROOT"] = str(shared_root)
        command_runner(
            [sys.executable, "-m", "sifr_verify.generated_cargo_setup",
             "--profile", str(profile["name"]), "--revision", revision],
            env=setup_env,
        )


def enable_offline_cargo(env: dict[str, str]) -> None:
    """Force profile execution to use the prepared Cargo cache."""
    env["CARGO_NET_OFFLINE"] = "true"
    os.environ["CARGO_NET_OFFLINE"] = "true"
