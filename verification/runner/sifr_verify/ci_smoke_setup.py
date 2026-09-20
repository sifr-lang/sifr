"""Prepare the standalone CI smoke lane before its offline, timed assertions."""
from __future__ import annotations

import os
from .cargo_setup import prepare_cargo_cache
from .profile_commands import run_command


def prepare(env, run):
    # Reuse profile acquisition and selected-area setup without selecting a broad
    # profile or changing any smoke assertion, filter, feature or deadline.
    profile = {
        "name": "ci-smoke",
        "cargo_policy": {"locked": True, "offline": True,
                         "setup_command": "cargo fetch --locked"},
        "selected_areas": [{"area": "fuzz_property",
                            "suites": ["cargo-smoke", "property", "fuzz-smoke"]}],
    }
    prepare_cargo_cache(profile, env, run)
    offline = env | {"CARGO_NET_OFFLINE": "true"}
    run(["cargo", "test", "--locked", "--offline", "--no-run",
         "-p", "sifr", "--test", "e2e"], env=offline)
    run(["cargo", "build", "--locked", "--offline", "-p", "sifr"], env=offline)


if __name__ == "__main__":
    prepare(os.environ.copy(), run_command)
