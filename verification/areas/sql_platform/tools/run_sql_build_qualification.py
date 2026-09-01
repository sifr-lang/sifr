#!/usr/bin/env python3
"""Qualify linked native SQL artifacts and cross-target SQL type checks."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[4]
QUALIFICATION = (
    REPO_ROOT
    / "verification"
    / "areas"
    / "sql_platform"
    / "data"
    / "integrated_qualification.json"
)
NATIVE_PACKAGES = (
    "sifr_compiler_component",
    "sifr_sql_contract",
    "sifr_sql_mysql",
    "sifr_sql_mysql_runtime",
    "sifr_sql_mysql_tools",
    "sifr_sql_postgresql_runtime",
    "sifr_sql_postgresql_tools",
    "sifr_sql_runtime",
    "sifr_sql_sqlite_runtime",
    "sifr_sql_sqlite_tools",
    "sifr_sql_tool",
)
WASI_PACKAGES = ("sifr_sql_contract",)
LINKED_NATIVE_ARTIFACTS = (
    "sifr-sql-mysql",
    "sifr-sql-postgresql",
    "sifr-sql-sqlite",
)


def host_target() -> str:
    result = subprocess.run(
        ["rustc", "-vV"],
        cwd=REPO_ROOT,
        text=True,
        stdout=subprocess.PIPE,
        check=True,
    )
    for line in result.stdout.splitlines():
        if line.startswith("host: "):
            return line.removeprefix("host: ")
    raise SystemExit("rustc did not report a host target")


def build(
    target: str, target_dir: Path, *, link: bool
) -> tuple[tuple[tuple[Any, ...], ...], dict[str, str]]:
    packages = WASI_PACKAGES if target == "wasm32-wasip2" else NATIVE_PACKAGES
    command = [
        "cargo",
        "build" if link else "check",
        "--locked",
        "--offline",
        "--target",
        target,
        "--message-format=json-render-diagnostics",
    ]
    if link:
        command.append("--release")
    for package in packages:
        command.extend(("-p", package))
    environment = os.environ.copy()
    environment["CARGO_INCREMENTAL"] = "0" if link else "1"
    environment["CARGO_NET_OFFLINE"] = "true"
    environment["CARGO_TARGET_DIR"] = str(target_dir)
    environment["SOURCE_DATE_EPOCH"] = "1"
    if link:
        environment["CARGO_PROFILE_RELEASE_DEBUG"] = "0"
        environment["CARGO_PROFILE_RELEASE_STRIP"] = "symbols"
        environment["CARGO_ENCODED_RUSTFLAGS"] = "\x1f".join(
            reproducible_rust_flags(target, target_dir)
        )
    result = subprocess.run(
        command,
        cwd=REPO_ROOT,
        env=environment,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if result.returncode != 0:
        raise SystemExit(result.stderr.strip() or f"SQL build failed for {target}")
    artifacts = []
    linked_hashes: dict[str, str] = {}
    for line in result.stdout.splitlines():
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        if message.get("reason") != "compiler-artifact":
            continue
        artifact_target = message["target"]
        artifacts.append(
            (
                message["package_id"],
                artifact_target["name"],
                tuple(artifact_target["crate_types"]),
                tuple(message.get("features", [])),
                bool(message["profile"].get("test")),
            )
        )
        executable = message.get("executable")
        name = str(artifact_target["name"])
        if link and executable and name in LINKED_NATIVE_ARTIFACTS:
            linked_hashes[name] = sha256_file(Path(executable))
    if not artifacts:
        raise SystemExit(f"clean SQL build for {target} produced no artifact evidence")
    if link and set(linked_hashes) != set(LINKED_NATIVE_ARTIFACTS):
        missing = sorted(set(LINKED_NATIVE_ARTIFACTS).difference(linked_hashes))
        raise SystemExit(f"native SQL build did not link required artifacts: {', '.join(missing)}")
    return tuple(sorted(artifacts)), linked_hashes


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def reproducible_rust_flags(target: str, target_dir: Path) -> list[str]:
    flags = [
        f"--remap-path-prefix={REPO_ROOT}=.",
        f"--remap-path-prefix={target_dir}=target",
    ]
    if target.endswith("-apple-darwin"):
        flags.extend(("-C", "link-arg=-Wl,-no_uuid"))
    elif target.endswith("-unknown-linux-gnu"):
        flags.extend(("-C", "link-arg=-Wl,--build-id=none"))
    elif target.endswith("-pc-windows-msvc"):
        flags.extend(("-C", "link-arg=/Brepro"))
    return flags


def reset_clean_target(target_dir: Path) -> None:
    shutil.rmtree(target_dir)
    target_dir.mkdir()


def validate_linked_hashes(first: dict[str, str], second: dict[str, str]) -> None:
    if set(first) != set(LINKED_NATIVE_ARTIFACTS):
        raise SystemExit("first native build has an incomplete linked-artifact set")
    if set(second) != set(LINKED_NATIVE_ARTIFACTS):
        raise SystemExit("second native build has an incomplete linked-artifact set")
    if first != second:
        changed = sorted(name for name in first if first[name] != second[name])
        raise SystemExit(
            "clean native SQL artifact hashes are not reproducible: " + ", ".join(changed)
        )


def qualify_native(target: str, temp_root: Path) -> None:
    with tempfile.TemporaryDirectory(prefix="native-candidate-", dir=temp_root) as candidate_dir:
        target_dir = Path(candidate_dir)
        first_plan, first_hashes = build(target, target_dir, link=True)
        incremental_plan, incremental_hashes = build(target, target_dir, link=True)
        if incremental_plan != first_plan or incremental_hashes != first_hashes:
            raise SystemExit("incremental native SQL build differs from its clean linked build")
        reset_clean_target(target_dir)
        second_plan, second_hashes = build(target, target_dir, link=True)
    if second_plan != first_plan:
        raise SystemExit("independent clean native SQL build plan is not reproducible")
    validate_linked_hashes(first_hashes, second_hashes)
    rendered_hashes = ",".join(
        f"{name}:{first_hashes[name]}" for name in LINKED_NATIVE_ARTIFACTS
    )
    print(
        "SQL native build qualification ok: "
        f"target={target} linked={len(first_hashes)} hash=sha256 artifacts={rendered_hashes} "
        "modes=clean-twice,incremental,locked,offline,native-linked-reproducible"
    )


def qualify_cross_target(target: str, temp_root: Path) -> None:
    with tempfile.TemporaryDirectory(prefix="cross-check-", dir=temp_root) as target_dir:
        clean_plan, hashes = build(target, Path(target_dir), link=False)
        incremental_plan, incremental_hashes = build(target, Path(target_dir), link=False)
    if hashes or incremental_hashes:
        raise SystemExit("cross-target check unexpectedly claimed linked artifact hashes")
    if incremental_plan != clean_plan:
        raise SystemExit("incremental cross-target SQL check differs from its clean check")
    print(
        "SQL cross-target qualification ok: "
        f"target={target} artifacts={len(clean_plan)} mode=typecheck-only "
        "linked=false byte-reproducible=false reason=local-linker-unavailable"
    )


def self_test() -> None:
    first = {name: "a" * 64 for name in LINKED_NATIVE_ARTIFACTS}
    second = dict(first)
    second[LINKED_NATIVE_ARTIFACTS[0]] = "b" * 64
    try:
        validate_linked_hashes(first, second)
    except SystemExit:
        print("SQL build qualification self-test ok: changed linked hash rejected")
        return
    raise SystemExit("SQL build qualification self-test failed")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", default=host_target())
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return 0
    record = json.loads(QUALIFICATION.read_text(encoding="utf-8"))
    supported = set(record["cross_targets"])
    if args.target not in supported:
        raise SystemExit(f"target is not in SQL qualification: {args.target}")
    temp_root = REPO_ROOT / "target" / "verification" / "sql-platform-builds"
    temp_root.mkdir(parents=True, exist_ok=True)
    if args.target == host_target():
        qualify_native(args.target, temp_root)
    else:
        qualify_cross_target(args.target, temp_root)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
