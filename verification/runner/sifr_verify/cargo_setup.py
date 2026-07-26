"""Resolve the canonical Cargo cache preparation for validation profiles."""

from __future__ import annotations

import os
import shlex
from typing import Any, Callable

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
    """Populate the exact lock graph before profile execution becomes offline."""
    command = cargo_setup_command(profile)
    setup_env = env.copy()
    setup_env.pop("CARGO_NET_OFFLINE", None)
    print(f"[sifr-profile-setup] command={' '.join(command)}")
    command_runner(command, env=setup_env)


def enable_offline_cargo(env: dict[str, str]) -> None:
    """Force profile execution to use the prepared Cargo cache."""
    env["CARGO_NET_OFFLINE"] = "true"
    os.environ["CARGO_NET_OFFLINE"] = "true"
