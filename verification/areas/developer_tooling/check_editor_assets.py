#!/usr/bin/env python3
"""Validate Phase 36 editor integration assets."""

from __future__ import annotations

import argparse
import json
import shutil
import sys
import tempfile
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[3]
ASSET_ROOT = REPO_ROOT / "editor_integrations"
FIXTURE_ROOT = REPO_ROOT / "verification" / "areas" / "performance" / "sifr_syntax_token_fixtures"

REQUIRED_FILES = [
    "README.md",
    "syntaxes/sifr.tmLanguage.json",
    "syntaxes/sifr-token-scope-map.json",
    "neovim/ftdetect/sifr.lua",
    "neovim/lsp/sifr.lua",
    "zed/extension.toml",
    "zed/languages/sifr/config.toml",
    "helix/languages.toml",
    "emacs/sifr-mode.el",
]

TARGET_FILES = {
    "neovim": ["neovim/ftdetect/sifr.lua", "neovim/lsp/sifr.lua"],
    "zed": ["zed/extension.toml", "zed/languages/sifr/config.toml"],
    "helix": ["helix/languages.toml"],
    "emacs": ["emacs/sifr-mode.el"],
}

FORBIDDEN_MARKERS = {
    "pyright": "editor assets must not fall back to Python tooling",
    "pylsp": "editor assets must not fall back to Python tooling",
    "python-lsp-server": "editor assets must not fall back to Python tooling",
    "ruff-lsp": "editor assets must not fall back to Ruff tooling",
    "ruff server": "editor assets must not fall back to Ruff tooling",
    "sifr fmt": "editor assets must not invoke the CLI formatter as the editor formatting provider",
    "sifr_python_parser": "editor assets must not call Sifr parser internals",
    "ruff_python_parser": "editor assets must not call parser internals",
    "type_check": "editor assets must not type-check",
    "lower_module": "editor assets must not lower HIR",
    "sifr_codegen": "editor assets must not call codegen",
}

REQUIRED_GRAMMAR_SCOPES = {
    "source.sifr",
    "comment.line.number-sign.sifr",
    "string.quoted.double.sifr",
    "constant.numeric.integer.sifr",
    "storage.type.function.sifr",
    "entity.name.function.sifr",
    "storage.type.class.sifr",
    "entity.name.type.class.sifr",
    "keyword.control.sifr",
    "keyword.control.import.sifr",
    "keyword.operator.word.sifr",
    "support.type.sifr",
    "variable.other.sifr",
    "keyword.operator.assignment.sifr",
    "keyword.operator.comparison.sifr",
    "keyword.operator.arithmetic.sifr",
    "punctuation.separator.return-type.sifr",
}


def read_text(root: Path, relative: str) -> str:
    return (root / relative).read_text(encoding="utf-8")


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def all_asset_files(root: Path) -> list[Path]:
    files: list[Path] = []
    for path in root.rglob("*"):
        if not path.is_file():
            continue
        parts = path.relative_to(root).parts
        if parts and parts[0] in {".git", "vscode"}:
            continue
        files.append(path)
    return sorted(files)


def validate_required_files(root: Path) -> list[str]:
    failures: list[str] = []
    for relative in REQUIRED_FILES:
        if not (root / relative).is_file():
            failures.append(f"missing editor asset: editor_integrations/{relative}")
    return failures


def validate_lsp_launch(root: Path) -> list[str]:
    failures: list[str] = []
    for target, relatives in TARGET_FILES.items():
        combined = "\n".join(read_text(root, relative) for relative in relatives)
        has_split_command = (
            '"sifr"' in combined
            and '"lsp"' in combined
            and '"--stdio"' in combined
        )
        has_shell_command = "sifr lsp --stdio" in combined
        if not (has_split_command or has_shell_command):
            failures.append(f"{target} does not launch sifr lsp --stdio")
        if ".sifr" not in combined and '"sifr"' not in combined and "'sifr'" not in combined:
            failures.append(f"{target} does not register Sifr filetype or .sifr files")
    return failures


def validate_no_fallbacks(root: Path) -> list[str]:
    failures: list[str] = []
    for path in all_asset_files(root):
        text = path.read_text(encoding="utf-8", errors="replace").lower()
        for marker, reason in FORBIDDEN_MARKERS.items():
            if marker in text:
                failures.append(f"{path.relative_to(REPO_ROOT)} contains forbidden marker {marker!r}: {reason}")
    return failures


def collect_scope_names(value: Any) -> set[str]:
    scopes: set[str] = set()
    if isinstance(value, dict):
        name = value.get("name")
        scope_name = value.get("scopeName")
        if isinstance(name, str):
            scopes.add(name)
        if isinstance(scope_name, str):
            scopes.add(scope_name)
        for child in value.values():
            scopes.update(collect_scope_names(child))
    elif isinstance(value, list):
        for item in value:
            scopes.update(collect_scope_names(item))
    return scopes


def fixture_token_kinds() -> set[str]:
    kinds: set[str] = set()
    for path in sorted(FIXTURE_ROOT.glob("*.json")):
        data = read_json(path)
        for kind in data.get("expected_token_kinds", []):
            if not isinstance(kind, str):
                raise SystemExit(f"{path.relative_to(REPO_ROOT)} has non-string token kind")
            kinds.add(kind)
    return kinds


def validate_syntax_assets(root: Path) -> list[str]:
    failures: list[str] = []
    grammar_path = root / "syntaxes" / "sifr.tmLanguage.json"
    scope_map_path = root / "syntaxes" / "sifr-token-scope-map.json"
    grammar = read_json(grammar_path)
    scope_map = read_json(scope_map_path)

    if grammar.get("scopeName") != "source.sifr":
        failures.append("TextMate grammar scopeName must be source.sifr")
    if "sifr" not in grammar.get("fileTypes", []):
        failures.append("TextMate grammar must register .sifr files")

    scopes = collect_scope_names(grammar)
    missing_required_scopes = REQUIRED_GRAMMAR_SCOPES - scopes
    if missing_required_scopes:
        failures.append(f"TextMate grammar missing required scopes: {sorted(missing_required_scopes)}")

    mapped = scope_map.get("token_scopes", {})
    ignored = set(scope_map.get("ignored_token_kinds", []))
    if not isinstance(mapped, dict):
        failures.append("syntax token scope map token_scopes must be an object")
        mapped = {}
    fixture_kinds = fixture_token_kinds()
    unmapped = fixture_kinds - set(mapped) - ignored
    if unmapped:
        failures.append(f"syntax fixtures contain unmapped token kinds: {sorted(unmapped)}")

    for token_kind, scope in mapped.items():
        if token_kind not in fixture_kinds:
            failures.append(f"syntax scope map contains token kind not covered by fixtures: {token_kind}")
        if not isinstance(scope, str) or not scope:
            failures.append(f"syntax scope map value for {token_kind} must be a non-empty scope")
        elif scope not in scopes and not any(existing.startswith(scope) for existing in scopes):
            failures.append(f"scope map for {token_kind} references missing grammar scope {scope}")

    return failures


def validate(root: Path = ASSET_ROOT) -> list[str]:
    failures = validate_required_files(root)
    if failures:
        return failures
    failures.extend(validate_lsp_launch(root))
    failures.extend(validate_no_fallbacks(root))
    failures.extend(validate_syntax_assets(root))
    return failures


def run_self_test() -> None:
    with tempfile.TemporaryDirectory(dir=REPO_ROOT / "target") as tmp:
        temp_root = Path(tmp) / "editor_integrations"
        ignore_vcs = shutil.ignore_patterns(".git")
        shutil.copytree(ASSET_ROOT, temp_root, ignore=ignore_vcs)

        neovim_lsp = temp_root / "neovim" / "lsp" / "sifr.lua"
        neovim_lsp.write_text(neovim_lsp.read_text(encoding="utf-8").replace('"--stdio"', '"--tcp"'), encoding="utf-8")
        failures = validate(temp_root)
        if not any("neovim" in failure and "sifr lsp --stdio" in failure for failure in failures):
            raise SystemExit("editor assets self-test failed: bad Neovim launch passed")

        shutil.rmtree(temp_root)
        shutil.copytree(ASSET_ROOT, temp_root, ignore=ignore_vcs)
        scope_map_path = temp_root / "syntaxes" / "sifr-token-scope-map.json"
        scope_map = read_json(scope_map_path)
        scope_map["token_scopes"].pop("String", None)
        scope_map_path.write_text(json.dumps(scope_map, indent=2, sort_keys=True), encoding="utf-8")
        failures = validate(temp_root)
        if not any("unmapped token kinds" in failure and "String" in failure for failure in failures):
            raise SystemExit("editor assets self-test failed: missing syntax token scope passed")

        shutil.rmtree(temp_root)
        shutil.copytree(ASSET_ROOT, temp_root, ignore=ignore_vcs)
        neovim_lsp = temp_root / "neovim" / "lsp" / "sifr.lua"
        neovim_lsp.write_text(
            neovim_lsp.read_text(encoding="utf-8") + "\n-- formatter fallback: sifr fmt\n",
            encoding="utf-8",
        )
        failures = validate(temp_root)
        if not any("sifr fmt" in failure for failure in failures):
            raise SystemExit("editor assets self-test failed: direct sifr fmt formatter fallback passed")

    print("editor assets self-test: PASS")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        run_self_test()
        return 0

    failures = validate()
    if failures:
        print("editor assets: FAIL", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1
    print("editor assets: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
