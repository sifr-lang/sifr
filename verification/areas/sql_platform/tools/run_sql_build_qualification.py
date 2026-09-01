#!/usr/bin/env python3
"""Execute clean, incremental, reproducible, locked, offline SQL builds."""

from __future__ import annotations

import argparse
import json
import os
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


def build(target: str, target_dir: Path) -> tuple[tuple[Any, ...], ...]:
    packages = WASI_PACKAGES if target == "wasm32-wasip2" else NATIVE_PACKAGES
    command = [
        "cargo",
        "check",
        "--locked",
        "--offline",
        "--target",
        target,
        "--message-format=json-render-diagnostics",
    ]
    for package in packages:
        command.extend(("-p", package))
    environment = os.environ.copy()
    environment["CARGO_INCREMENTAL"] = "1"
    environment["CARGO_NET_OFFLINE"] = "true"
    environment["CARGO_TARGET_DIR"] = str(target_dir)
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
    if not artifacts:
        raise SystemExit(f"clean SQL build for {target} produced no artifact evidence")
    return tuple(sorted(artifacts))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", default=host_target())
    args = parser.parse_args()
    record = json.loads(QUALIFICATION.read_text(encoding="utf-8"))
    supported = set(record["cross_targets"])
    if args.target not in supported:
        raise SystemExit(f"target is not in SQL qualification: {args.target}")
    temp_root = REPO_ROOT / "target" / "verification" / "sql-platform-builds"
    temp_root.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="candidate-a-", dir=temp_root) as first_dir:
        first = Path(first_dir)
        clean_plan = build(args.target, first)
        incremental_plan = build(args.target, first)
        if incremental_plan != clean_plan:
            raise SystemExit("incremental SQL build plan differs from its clean build")
    with tempfile.TemporaryDirectory(prefix="candidate-b-", dir=temp_root) as second_dir:
        reproduced_plan = build(args.target, Path(second_dir))
    if reproduced_plan != clean_plan:
        raise SystemExit("independent clean SQL build plan is not reproducible")
    print(
        "SQL build qualification ok: "
        f"target={args.target} artifacts={len(clean_plan)} modes=clean,incremental,locked,offline,reproducible"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
