"""The exact source compiler graph used by sysroot boundary certification."""

from __future__ import annotations

import os
import subprocess
from pathlib import Path


def source_build_configuration(root: Path, environment: dict[str, str]):
    target = root.resolve() / "target" / "sysroot_release" / "source-cargo-target"
    env = environment.copy()
    env["CARGO_TARGET_DIR"] = str(target)
    env["CARGO_NET_OFFLINE"] = "true"
    command = ["cargo", "build", "--locked", "--offline", "-p", "sifr"]
    return command, env, target / "debug" / "sifr"


def prepare_source_sifr(root: Path, environment: dict[str, str], command_runner=subprocess.run):
    command, env, binary = source_build_configuration(root, environment)
    print(f"[sifr-profile-setup] sysroot-source-build={' '.join(command)} "
          f"target={env['CARGO_TARGET_DIR']}", flush=True)
    command_runner(command, cwd=root, env=env, check=True)
    if not binary.is_file():
        raise RuntimeError(f"source-tree compiler was not produced: {binary}")
    return binary


if __name__ == "__main__":
    prepare_source_sifr(Path(__file__).resolve().parents[3], os.environ.copy())
