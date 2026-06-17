#!/usr/bin/env python3
"""Validate the editor tooling VS Code extension build, tests, and package."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[3]
RULES_PATH = REPO_ROOT / "verification" / "areas" / "developer_tooling" / "vscode_extension_rules.json"

REQUIRED_SCRIPTS = ["lint", "typecheck", "test", "test:extension", "package"]
REQUIRED_COMMANDS = {
    "sifr.restartLanguageServer",
    "sifr.showLanguageServerLogs",
    "sifr.locateBinary",
    "sifr.runCheck",
    "sifr.runTests",
    "sifr.runLint",
    "sifr.checkFormat",
    "sifr.formatDocument",
    "sifr.showGeneratedRust",
    "sifr.explainDiagnostic",
}
FORBIDDEN_MARKERS = [
    "pyright",
    "pylsp",
    "ruff server",
    "ruffServer",
    "tyServer",
    "parseSifr",
    "typeCheckSifr",
    "formatSifrInExtension",
    "lintSifrInExtension",
    "generateRustInExtension",
]


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def extension_repo_path(rules: dict[str, Any]) -> Path | None:
    repo = rules["repository"]
    if env_value := os.environ.get(repo["location_env"]):
        return Path(env_value)
    submodule = (REPO_ROOT / repo["submodule_checkout"]).resolve()
    if submodule.exists():
        return submodule
    sibling = (REPO_ROOT / repo["sibling_checkout"]).resolve()
    if sibling.exists():
        return sibling
    return None


def validate_package_json(repo_path: Path, package_json: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    scripts = package_json.get("scripts", {})
    for script in REQUIRED_SCRIPTS:
        if script not in scripts:
            failures.append(f"package.json missing script: {script}")
    contributes = package_json.get("contributes", {})
    commands = {item.get("command") for item in contributes.get("commands", [])}
    missing_commands = REQUIRED_COMMANDS - commands
    if missing_commands:
        failures.append(f"package.json missing commands: {sorted(missing_commands)}")
    languages = contributes.get("languages", [])
    if not any(language.get("id") == "sifr" and ".sifr" in language.get("extensions", []) for language in languages):
        failures.append("package.json must contribute language id sifr and .sifr extension")
    grammars = contributes.get("grammars", [])
    if not any(grammar.get("scopeName") == "source.sifr" for grammar in grammars):
        failures.append("package.json must contribute source.sifr grammar")
    icon = package_json.get("icon")
    if not isinstance(icon, str) or not (repo_path / icon).is_file():
        failures.append("package.json must reference a checked-in extension icon")
    for metadata in ["displayName", "description", "categories", "keywords", "repository", "license"]:
        if metadata not in package_json:
            failures.append(f"package.json missing publication metadata: {metadata}")
    text = json.dumps(package_json)
    for marker in FORBIDDEN_MARKERS:
        if marker in text:
            failures.append(f"package.json contains forbidden marker: {marker}")
    if not (repo_path / "syntaxes" / "sifr.tmLanguage.json").is_file():
        failures.append("extension missing syntaxes/sifr.tmLanguage.json")
    if not (repo_path / "language-configuration" / "sifr.configuration.json").is_file():
        failures.append("extension missing language-configuration/sifr.configuration.json")
    return failures


def expected_package_output(package_json: dict[str, Any]) -> Path:
    name = package_json.get("name")
    version = package_json.get("version")
    if not isinstance(name, str) or not isinstance(version, str):
        return Path("dist") / "<invalid-package-name-or-version>.vsix"
    return Path("dist") / f"{name}-{version}.vsix"


def run_command(repo_path: Path, command: list[str]) -> str | None:
    result = subprocess.run(
        command,
        cwd=repo_path,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        return f"{' '.join(command)} failed with exit {result.returncode}\n{result.stdout}"
    return None


def validate(require_commands: bool = True) -> list[str]:
    rules = read_json(RULES_PATH)
    repo_path = extension_repo_path(rules)
    if repo_path is None:
        return [
            "VS Code extension repo missing; run scripts/clone_subrepos.sh, set SIFR_VSCODE_REPO, "
            "or checkout ../sifr-vscode"
        ]
    package_path = repo_path / "package.json"
    if not package_path.exists():
        return [f"VS Code extension checkout has no package.json: {repo_path}"]

    package_json = read_json(package_path)
    failures = validate_package_json(repo_path, package_json)
    if failures or not require_commands:
        return failures

    if not (repo_path / "node_modules").exists():
        failure = run_command(repo_path, ["npm", "ci"])
        if failure:
            failures.append(failure)
            return failures

    for script in REQUIRED_SCRIPTS:
        failure = run_command(repo_path, ["npm", "run", script])
        if failure:
            failures.append(failure)
            return failures

    required_output = expected_package_output(package_json)
    if not (repo_path / required_output).is_file():
        failures.append(f"package script did not produce {required_output}")
    return failures


def run_self_test() -> None:
    package_json = {
        "scripts": {script: "true" for script in REQUIRED_SCRIPTS if script != "package"},
        "contributes": {
            "languages": [{"id": "sifr", "extensions": [".sifr"]}],
            "grammars": [{"scopeName": "source.sifr", "path": "./syntaxes/sifr.tmLanguage.json"}],
        },
    }
    with tempfile.TemporaryDirectory() as tmp:
        repo_path = Path(tmp)
        (repo_path / "syntaxes").mkdir()
        (repo_path / "language-configuration").mkdir()
        (repo_path / "syntaxes" / "sifr.tmLanguage.json").write_text("{}", encoding="utf-8")
        (repo_path / "language-configuration" / "sifr.configuration.json").write_text("{}", encoding="utf-8")
        failures = validate_package_json(repo_path, package_json)
    if not any("package" in failure for failure in failures):
        raise SystemExit("VS Code extension package self-test failed: missing package script passed")
    print("VS Code extension package self-test: PASS")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--metadata-only", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        run_self_test()
        return 0

    failures = validate(require_commands=not args.metadata_only)
    if failures:
        print("VS Code extension package: FAIL", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1
    print("VS Code extension package: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
