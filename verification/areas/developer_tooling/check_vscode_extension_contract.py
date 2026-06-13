#!/usr/bin/env python3
"""Validate the main-repo VS Code extension contract and optional checkout."""

from __future__ import annotations

import argparse
import json
import os
import sys
import tempfile
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[3]
CONTRACT_PATH = REPO_ROOT / "verification" / "areas" / "developer_tooling" / "vscode_extension_contract.json"

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

REQUIRED_SETTINGS = {
    "sifr.lsp.path",
    "sifr.lsp.trace.server",
    "sifr.diagnostics.mode",
    "sifr.format.enable",
    "sifr.lint.enable",
}

FORBIDDEN_EXTENSION_TERMS = [
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


def extension_repo_path(contract: dict[str, Any]) -> Path | None:
    repo = contract["repository"]
    env_name = repo["location_env"]
    if env_value := os.environ.get(env_name):
        return Path(env_value)
    submodule = (REPO_ROOT / repo["submodule_checkout"]).resolve()
    if submodule.exists():
        return submodule
    sibling = (REPO_ROOT / repo["sibling_checkout"]).resolve()
    if sibling.exists():
        return sibling
    return None


def validate_contract(contract: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    extension = contract.get("extension", {})
    if extension.get("extension_id") != "sifr-lang.sifr-vscode":
        failures.append("extension id must be sifr-lang.sifr-vscode")
    if extension.get("language_id") != "sifr":
        failures.append("language id must be sifr")
    if ".sifr" not in extension.get("extensions", []):
        failures.append("extension must register .sifr")
    launch = contract.get("launch", {})
    if launch.get("default_command") != "sifr" or launch.get("default_args") != ["lsp", "--stdio"]:
        failures.append("extension launch must default to sifr lsp --stdio")
    commands = {item.get("command") for item in contract.get("commands", [])}
    missing_commands = REQUIRED_COMMANDS - commands
    if missing_commands:
        failures.append(f"missing required extension commands: {sorted(missing_commands)}")
    settings = {item.get("key") for item in contract.get("settings", [])}
    missing_settings = REQUIRED_SETTINGS - settings
    if missing_settings:
        failures.append(f"missing required extension settings: {sorted(missing_settings)}")
    return failures


def validate_package_json(contract: dict[str, Any], package_json: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    extension = contract["extension"]
    if package_json.get("name") != extension["name"]:
        failures.append("package.json name does not match contract")
    if package_json.get("publisher") != extension["publisher"]:
        failures.append("package.json publisher does not match contract")
    engines = package_json.get("engines", {})
    if engines.get("vscode") != extension["minimum_vscode_engine"]:
        failures.append("package.json VS Code engine does not match contract")

    contributes = package_json.get("contributes", {})
    languages = contributes.get("languages", [])
    if not any(language.get("id") == "sifr" and ".sifr" in language.get("extensions", []) for language in languages):
        failures.append("package.json must contribute language id sifr and extension .sifr")

    commands = {item.get("command") for item in contributes.get("commands", [])}
    missing_commands = REQUIRED_COMMANDS - commands
    if missing_commands:
        failures.append(f"package.json missing commands: {sorted(missing_commands)}")

    properties = contributes.get("configuration", {}).get("properties", {})
    missing_settings = REQUIRED_SETTINGS - set(properties)
    if missing_settings:
        failures.append(f"package.json missing settings: {sorted(missing_settings)}")

    scripts = set(package_json.get("scripts", {}))
    missing_scripts = set(contract.get("package_scripts", [])) - scripts
    if missing_scripts:
        failures.append(f"package.json missing scripts: {sorted(missing_scripts)}")

    text = json.dumps(package_json)
    for term in FORBIDDEN_EXTENSION_TERMS:
        if term in text:
            failures.append(f"package.json contains forbidden extension behavior marker: {term}")
    return failures


def validate_checkout(contract: dict[str, Any], require_extension_repo: bool) -> list[str]:
    failures: list[str] = []
    repo_path = extension_repo_path(contract)
    if repo_path is None:
        if require_extension_repo or contract.get("validation_stage") == "extension-active":
            failures.append(
                "VS Code extension repo missing; run scripts/clone_subrepos.sh, set SIFR_VSCODE_REPO, "
                "or checkout ../sifr-vscode"
            )
        return failures
    package_path = repo_path / "package.json"
    if not package_path.exists():
        failures.append(f"VS Code extension checkout has no package.json: {repo_path}")
        return failures
    failures.extend(validate_package_json(contract, read_json(package_path)))
    return failures


def validate(require_extension_repo: bool = False) -> list[str]:
    contract = read_json(CONTRACT_PATH)
    failures = validate_contract(contract)
    failures.extend(validate_checkout(contract, require_extension_repo))
    return failures


def run_self_test() -> None:
    contract = read_json(CONTRACT_PATH)
    package_json = {
        "name": "sifr-vscode",
        "publisher": "sifr-lang",
        "engines": {"vscode": "^1.90.0"},
        "contributes": {
            "languages": [{"id": "sifr", "extensions": [".sifr"]}],
            "commands": [{"command": command} for command in sorted(REQUIRED_COMMANDS - {"sifr.runCheck"})],
            "configuration": {"properties": {setting: {} for setting in REQUIRED_SETTINGS}},
        },
        "scripts": {script: "true" for script in contract["package_scripts"]},
    }
    with tempfile.TemporaryDirectory() as tmp:
        path = Path(tmp) / "package.json"
        path.write_text(json.dumps(package_json), encoding="utf-8")
        failures = validate_package_json(contract, read_json(path))
    if not any("sifr.runCheck" in failure for failure in failures):
        raise SystemExit("VS Code contract self-test failed: missing command passed")
    print("VS Code extension contract self-test: PASS")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--require-extension-repo", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        run_self_test()
        return 0

    failures = validate(require_extension_repo=args.require_extension_repo)
    if failures:
        print("VS Code extension contract: FAIL", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1
    print("VS Code extension contract: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
