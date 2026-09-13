#!/usr/bin/env python3
"""Check the editor-owned Node/npm selectors and the active command runtime."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import re
import subprocess
import sys

REPO_ROOT = Path(__file__).resolve().parents[1]
EXACT_VERSION = r"[0-9]+\.[0-9]+\.[0-9]+"


class ToolchainError(ValueError):
    """An actionable checkout, selector, or runtime error."""


def read_selectors(extension: Path) -> tuple[str, str]:
    required = (".node-version", "package.json", "package-lock.json")
    missing = [name for name in required if not (extension / name).is_file()]
    if missing:
        raise ToolchainError(
            f"Node toolchain checkout incomplete at {extension}: missing {', '.join(missing)}; "
            "run git submodule update --init --recursive -- editor_integrations "
            "from the Sifr source root"
        )
    try:
        node = (extension / ".node-version").read_text(encoding="utf-8").strip()
        package = json.loads((extension / "package.json").read_text(encoding="utf-8"))
        lock = json.loads((extension / "package-lock.json").read_text(encoding="utf-8"))
        manager = package.get("packageManager", "")
        match = re.fullmatch(rf"npm@({EXACT_VERSION})", manager)
        if re.fullmatch(EXACT_VERSION, node) is None or match is None:
            raise ToolchainError(".node-version and packageManager must select exact Node/npm releases")
        npm = match.group(1)
        engines = {"node": node, "npm": npm, "vscode": package.get("engines", {}).get("vscode")}
        if package.get("engines") != engines:
            raise ToolchainError("extension engines must agree with .node-version and packageManager")
        expected = {
            "runtime": {"name": "node", "version": node, "onFail": "error"},
            "packageManager": {"name": "npm", "version": npm, "onFail": "error"},
        }
        if package.get("devEngines") != expected:
            raise ToolchainError("npm devEngines must reject Node or npm selector drift")
        if lock.get("lockfileVersion") != 3:
            raise ToolchainError("extension lock must use npm lockfile version 3")
        if lock.get("packages", {}).get("", {}).get("engines") != engines:
            raise ToolchainError("extension lock root engines drifted from package.json")
    except (OSError, TypeError, AttributeError, json.JSONDecodeError) as error:
        raise ToolchainError(f"cannot read Node toolchain metadata at {extension}: {error}") from error
    return node, npm


def check_runtime(extension: Path, node: str, npm: str) -> None:
    for command, expected in (("node", f"v{node}"), ("npm", npm)):
        try:
            result = subprocess.run(
                [command, "--version"], cwd=extension, capture_output=True, text=True, check=False,
            )
        except OSError as error:
            raise ToolchainError(
                f"required {command} executable unavailable: {error}; "
                f"select Node {node}, provision npm {npm} with {extension}/scripts/setup-npm.sh "
                "<private-prefix>, and prepend its printed bin directory to PATH"
            ) from error
        actual = result.stdout.strip()
        if result.returncode != 0 or actual != expected:
            raise ToolchainError(
                f"Node toolchain mismatch: {command} expected {expected}, "
                f"found {actual or '<no version>'} (exit {result.returncode}); "
                f"select Node {node}, provision npm {npm} with {extension}/scripts/setup-npm.sh "
                "<private-prefix>, and prepend its printed bin directory to PATH"
            )


def validate(extension: Path) -> tuple[str, str]:
    node, npm = read_selectors(extension)
    check_runtime(extension, node, npm)
    return node, npm


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--extension", type=Path, default=REPO_ROOT / "editor_integrations/vscode")
    args = parser.parse_args()
    try:
        node, npm = validate(args.extension.resolve())
    except ToolchainError as error:
        print(str(error), file=sys.stderr)
        return 1
    print(f"Node toolchain: PASS (Node {node}, npm {npm})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
