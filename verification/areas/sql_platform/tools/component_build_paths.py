"""Remap build-machine paths before immutable guest bytes are published."""

from __future__ import annotations

import os
import shlex
from pathlib import Path

PATH_POLICY = {
    "repository": "/sifr",
    "cargo_home": "/cargo-home",
    "rustup_home": "/rustup-home",
    "target": "/cargo-target",
    "sdk": "/wasi-sdk",
}


def build_path_mappings(root: Path, environment: dict[str, str]) -> list[tuple[str, str]]:
    paths = {
        "repository": root,
        "cargo_home": Path(environment.get("CARGO_HOME") or Path.home() / ".cargo"),
        "rustup_home": Path(environment.get("RUSTUP_HOME") or Path.home() / ".rustup"),
        "target": Path(environment.get("CARGO_TARGET_DIR") or root / "target"),
    }
    if environment.get("WASI_SDK_PATH"):
        paths["sdk"] = Path(environment["WASI_SDK_PATH"])
    mappings: dict[str, str] = {}
    for name, path in paths.items():
        path = path.expanduser()
        if not path.is_absolute():
            path = root / path
        # Both invocation spelling and canonical paths can enter compiler output.
        for source in (os.path.abspath(path), str(path.resolve())):
            destination = PATH_POLICY[name]
            if source in mappings and mappings[source] != destination:
                raise ValueError("component build roots must have distinct identities")
            mappings[source] = destination
    # Both compilers use the last matching rule; nested roots take precedence.
    return sorted(mappings.items(), key=lambda pair: (len(pair[0]), pair[0]))


def remap_environment(root: Path, environment: dict[str, str]) -> dict[str, str]:
    result = environment.copy()
    mappings = build_path_mappings(root, result)
    result.pop("RUSTFLAGS", None)
    result["CARGO_ENCODED_RUSTFLAGS"] = "\x1f".join(
        f"--remap-path-prefix={source}={destination}" for source, destination in mappings
    )
    if result.get("WASI_SYSROOT"):
        # SDK selection has already rejected ambient C overrides. cc's supported
        # shell parser preserves paths containing spaces on every producer host.
        flags = [f"--sysroot={result['WASI_SYSROOT']}"]
        flags.extend(f"-ffile-prefix-map={source}={destination}" for source, destination in mappings)
        result["CC_SHELL_ESCAPED_FLAGS"] = "1"
        result["CFLAGS_wasm32_wasip2"] = shlex.join(flags)
    return result


def validate_build_paths(artifact: Path, root: Path, environment: dict[str, str]) -> None:
    data = artifact.read_bytes()
    for source, _ in build_path_mappings(root, environment):
        # Windows tools may emit either separator spelling.
        if any(value.encode() in data for value in {source, source.replace("\\", "/")}):
            raise ValueError(f"component contains a producer path: {artifact.name}: {source}")
