"""Canonical development compiler preparation for sysroot certification."""

from __future__ import annotations

import os
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "common"))
from sifr_binary import resolve_sifr_binary  # noqa: E402


def prepare_source_sifr(root: Path, environment: dict[str, str]) -> Path:
    # Cargo remains the freshness authority and selects the actual executable,
    # including caller target/flags. Do not invent a second source build graph.
    env = dict(environment, CARGO_NET_OFFLINE="true")
    print("[sifr-profile-setup] sysroot-source-compiler=canonical-cargo-selection "
          f"target={env.get('CARGO_TARGET_DIR', 'Cargo configuration')}", flush=True)
    return resolve_sifr_binary(root, env=env)


if __name__ == "__main__":
    prepare_source_sifr(Path(__file__).resolve().parents[3], os.environ.copy())
