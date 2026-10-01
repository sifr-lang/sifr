#!/usr/bin/env python3
"""Qualify linked native SQL tools and explicitly non-linking cross-target checks."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shlex
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[4]
QUALIFICATION = REPO_ROOT / "verification/areas/sql_platform/data/integrated_qualification.json"
TOOLS = {
    "sifr-sql-mysql": "sifr_sql_mysql_tools",
    "sifr-sql-postgresql": "sifr_sql_postgresql_tools",
    "sifr-sql-sqlite": "sifr_sql_sqlite_tools",
}
WASI_PACKAGES = ("sifr_sql_contract",)
CROSS_PACKAGES = (
    "sifr_compiler_component", "sifr_sql_contract", "sifr_sql_mysql",
    "sifr_sql_mysql_runtime", "sifr_sql_mysql_tools", "sifr_sql_postgresql_runtime",
    "sifr_sql_postgresql_tools", "sifr_sql_runtime", "sifr_sql_sqlite_runtime",
    "sifr_sql_sqlite_tools", "sifr_sql_tool",
)


class BuildError(ValueError):
    """A selected artifact or build comparison violated the qualification contract."""


def host_target() -> str:
    result = subprocess.run(["rustc", "-vV"], cwd=REPO_ROOT, text=True,
                            capture_output=True, check=True)
    for line in result.stdout.splitlines():
        if line.startswith("host: "):
            return line.removeprefix("host: ")
    raise BuildError("rustc did not report a host target")


def cargo_metadata() -> dict[str, str]:
    result = subprocess.run(
        ["cargo", "metadata", "--locked", "--offline", "--no-deps", "--format-version", "1"],
        cwd=REPO_ROOT, text=True, capture_output=True, check=True,
    )
    packages = json.loads(result.stdout)["packages"]
    identities = {package["name"]: package["id"] for package in packages}
    if not set(TOOLS.values()) <= identities.keys():
        raise BuildError("SQL tool package is missing from locked workspace metadata")
    return identities


def cargo_run(command: list[str], target_dir: Path,
              settings: dict[str, str] | None = None) -> list[dict[str, Any]]:
    environment = os.environ.copy()
    environment.update(CARGO_INCREMENTAL="1", CARGO_NET_OFFLINE="true",
                       CARGO_TARGET_DIR=str(target_dir))
    environment.update(settings or {})
    result = subprocess.run(command, cwd=REPO_ROOT, env=environment, text=True,
                            capture_output=True, check=False)
    if result.returncode:
        raise BuildError(result.stderr.strip() or f"Cargo exited {result.returncode}")
    messages = []
    for line in result.stdout.splitlines():
        try:
            messages.append(json.loads(line))
        except json.JSONDecodeError:
            continue
    return messages


def selected_executables(
    messages: list[dict[str, Any]], target_dir: Path, target: str,
    identities: dict[str, str],
) -> dict[str, str]:
    """Select exactly the requested bin outputs from Cargo, then hash their bytes."""
    selected: dict[str, str] = {}
    owned = (target_dir / target / "debug").resolve()
    for message in messages:
        if message.get("reason") != "compiler-artifact":
            continue
        artifact_target = message.get("target", {})
        name = artifact_target.get("name")
        if name not in TOOLS or "bin" not in artifact_target.get("kind", []):
            continue
        if name in selected:
            raise BuildError(f"duplicate SQL executable artifact: {name}")
        if message.get("package_id") != identities[TOOLS[name]]:
            raise BuildError(f"SQL executable has wrong package identity: {name}")
        executable = message.get("executable")
        if not isinstance(executable, str):
            raise BuildError(f"SQL artifact is not a linked executable: {name}")
        path = Path(executable)
        if path.resolve().parent != owned:
            raise BuildError(f"SQL executable is outside the selected target/profile: {name}")
        if not path.is_file() or not os.access(path, os.X_OK):
            raise BuildError(f"SQL executable is absent or non-executable: {name}")
        selected[name] = hashlib.sha256(path.read_bytes()).hexdigest()
    if set(selected) != set(TOOLS):
        raise BuildError(f"SQL executable set differs: expected={sorted(TOOLS)} actual={sorted(selected)}")
    return selected


def compare_hashes(first: dict[str, str], next_build: dict[str, str], mode: str) -> None:
    if set(first) != set(TOOLS) or set(next_build) != set(TOOLS):
        raise BuildError(f"{mode} SQL executable set differs")
    differing = [name for name in TOOLS if first[name] != next_build[name]]
    if differing:
        raise BuildError(f"{mode} SQL executable bytes differ: {', '.join(differing)}")


def native_settings(first: Path, second: Path) -> dict[str, str]:
    """Hold both path maps constant, including the product-identity inputs."""
    encoded = os.environ.get("CARGO_ENCODED_RUSTFLAGS")
    flags = encoded.split("\x1f") if encoded else shlex.split(os.environ.get("RUSTFLAGS", ""))
    flags.extend(f"--remap-path-prefix={path}=/sifr-sql-build" for path in (first, second))
    return {"CARGO_ENCODED_RUSTFLAGS": "\x1f".join(flags), "CARGO_PROFILE_DEV_DEBUG": "0"}


def build_native(target: str, target_dir: Path, identities: dict[str, str],
                 settings: dict[str, str]) -> dict[str, str]:
    command = ["cargo", "build", "--locked", "--offline", "--target", target,
               "--profile", "dev", "--message-format=json-render-diagnostics"]
    for name, package in TOOLS.items():
        command.extend(("-p", package, "--bin", name))
    return selected_executables(cargo_run(command, target_dir, settings), target_dir, target, identities)


def check_cross_target(target: str, target_dir: Path) -> int:
    packages = WASI_PACKAGES if target == "wasm32-wasip2" else CROSS_PACKAGES
    command = ["cargo", "check", "--locked", "--offline", "--target", target,
               "--message-format=json-render-diagnostics"]
    for package in packages:
        command.extend(("-p", package))
    messages = cargo_run(command, target_dir)
    artifacts = sum(message.get("reason") == "compiler-artifact" for message in messages)
    if not artifacts:
        raise BuildError(f"cross-target check produced no artifact evidence: {target}")
    return artifacts


def self_test() -> None:
    with tempfile.TemporaryDirectory(prefix="sql-build-self-test-") as root:
        target_dir = Path(root)
        target = "x86_64-unknown-linux-gnu"
        owned = target_dir / target / "debug"
        owned.mkdir(parents=True)
        identities = {package: f"pkg:{package}" for package in TOOLS.values()}
        def message(name: str, executable: Path | None, *, package: str | None = None) -> dict[str, Any]:
            return {"reason": "compiler-artifact", "target": {"name": name, "kind": ["bin"]},
                    "package_id": package or identities[TOOLS[name]],
                    "executable": str(executable) if executable else None}
        messages = []
        for name in TOOLS:
            path = owned / name
            path.write_bytes(name.encode())
            path.chmod(0o755)
            messages.append(message(name, path))
        expected = selected_executables(messages, target_dir, target, identities)
        compare_hashes(expected, expected, "unchanged")
        mutations = []
        def rejects(label: str, action: Any) -> None:
            try:
                action()
            except BuildError:
                mutations.append(label)
                return
            raise BuildError(f"self-test accepted {label}")
        (owned / "sifr-sql-mysql").write_bytes(b"changed linked executable bytes")
        changed = selected_executables(messages, target_dir, target, identities)
        rejects("changed-bytes", lambda: compare_hashes(expected, changed, "independent"))
        rejects("missing", lambda: selected_executables(messages[:-1], target_dir, target, identities))
        rejects("duplicate", lambda: selected_executables(messages + messages[:1], target_dir, target, identities))
        rejects("wrong-package", lambda: selected_executables(
            [message("sifr-sql-mysql", owned / "sifr-sql-mysql", package="other"), *messages[1:]],
            target_dir, target, identities))
        rejects("wrong-target", lambda: selected_executables(messages, target_dir, "aarch64-unknown-linux-gnu", identities))
        rejects("non-executable-artifact", lambda: selected_executables(
            [message("sifr-sql-mysql", None), *messages[1:]], target_dir, target, identities))
        outside = target_dir / "release"
        outside.write_bytes(b"other")
        outside.chmod(0o755)
        rejects("wrong-profile", lambda: selected_executables(
            [message("sifr-sql-mysql", outside), *messages[1:]], target_dir, target, identities))
        if len(mutations) != 7:
            raise BuildError("SQL build mutation coverage is incomplete")
    print(f"SQL build qualification self-test ok: mutations={len(mutations)}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return 0
    target = args.target or host_target()
    record = json.loads(QUALIFICATION.read_text(encoding="utf-8"))
    if target not in record["cross_targets"]:
        raise BuildError(f"target is not in SQL qualification: {target}")
    if shutil.disk_usage(REPO_ROOT).free < 8 * 1024**3:
        raise BuildError("less than 8 GiB free before clean SQL build qualification")
    parent = REPO_ROOT / "target/verification/sql-platform-builds"
    parent.mkdir(parents=True, exist_ok=True)
    context = {"target": target, "host": host_target(), "profile": "dev",
               "lock_sha256": hashlib.sha256((REPO_ROOT / "Cargo.lock").read_bytes()).hexdigest(),
               "rustc": subprocess.run(["rustc", "-vV"], text=True, capture_output=True,
                                        check=True).stdout.strip(),
               "cargo_incremental": "1", "cargo_net_offline": "true"}
    with tempfile.TemporaryDirectory(prefix="candidate-", dir=parent) as candidate_dir:
        first = Path(candidate_dir) / "a"
        second = Path(candidate_dir) / "b"
        if target == context["host"]:
            identities = cargo_metadata()
            settings = native_settings(first, second)
            print("SQL native qualification: clean A", file=sys.stderr, flush=True)
            clean = build_native(target, first, identities, settings)
            print("SQL native qualification: unchanged rebuild A", file=sys.stderr, flush=True)
            reused = build_native(target, first, identities, settings)
            compare_hashes(clean, reused, "unchanged rebuild")
            shutil.rmtree(first)
            print("SQL native qualification: independent clean B", file=sys.stderr, flush=True)
            independent = build_native(target, second, identities, settings)
            compare_hashes(clean, independent, "independent clean rebuild")
            context.update(claim="linked-native-target-directories", executables=clean,
                           native_settings=settings,
                           modes=["clean-a", "reused-a", "clean-b", "locked", "offline"])
        else:
            artifacts = check_cross_target(target, first)
            context.update(claim="cross-target-check-only", artifacts=artifacts,
                           modes=["check", "locked", "offline"])
    print(json.dumps(context, sort_keys=True))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (BuildError, OSError, subprocess.CalledProcessError, json.JSONDecodeError) as error:
        print(f"SQL build qualification error: {error}", file=sys.stderr)
        raise SystemExit(1) from error
