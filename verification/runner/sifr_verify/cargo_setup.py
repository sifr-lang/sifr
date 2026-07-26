"""Resolve the canonical Cargo cache preparation for validation profiles."""

from __future__ import annotations

import shlex
from typing import Any

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
