#!/usr/bin/env python3
"""Validate registry coverage for active diagnostic codes."""

from __future__ import annotations

import pathlib
import re
import subprocess
import sys


ROOT = pathlib.Path(__file__).resolve().parents[4]
CODES_RS = ROOT / "crates" / "sifr_diagnostics" / "src" / "codes.rs"
CODE_RE = r"SIFR-[A-Z]+-\d{4}"
INCLUDE_RE = re.compile(r'^\s*include!\("([^"]+)"\);\s*$', re.MULTILINE)
LOCAL_MOD_RE = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z0-9_]+);\s*$",
    re.MULTILINE,
)


def git_ls_files(*patterns: str) -> list[pathlib.Path]:
    result = subprocess.run(
        ["git", "ls-files", *patterns],
        cwd=ROOT,
        check=True,
        stdout=subprocess.PIPE,
        text=True,
    )
    return [ROOT / line for line in result.stdout.splitlines() if line]


def strip_cfg_test_blocks(text: str) -> str:
    kept: list[str] = []
    pending_cfg_test = False
    skipping = False
    depth = 0

    for line in text.splitlines():
        stripped = line.strip()
        if skipping:
            depth += line.count("{") - line.count("}")
            if depth <= 0:
                skipping = False
            continue

        if stripped == "#[cfg(test)]":
            pending_cfg_test = True
            continue

        if pending_cfg_test:
            pending_cfg_test = False
            if "{" in line:
                skipping = True
                depth = line.count("{") - line.count("}")
                if depth <= 0:
                    skipping = False
                continue
            continue

        kept.append(line)

    return "\n".join(kept)


def local_module_path(parent: pathlib.Path, module_name: str) -> pathlib.Path | None:
    direct = parent / f"{module_name}.rs"
    if direct.exists():
        return direct
    nested = parent / module_name / "mod.rs"
    if nested.exists():
        return nested
    return None


def read_rust_with_local_sources(path: pathlib.Path, seen: set[pathlib.Path] | None = None) -> str:
    if seen is None:
        seen = set()
    resolved = path.resolve()
    if resolved in seen:
        return ""
    seen.add(resolved)

    text = path.read_text(encoding="utf-8")

    def expand(match: re.Match[str]) -> str:
        include_path = path.parent / match.group(1)
        if not include_path.exists():
            return match.group(0)
        return read_rust_with_local_sources(include_path, seen)

    def expand_mod(match: re.Match[str]) -> str:
        module_name = match.group(1)
        module_path = local_module_path(path.parent, module_name)
        if module_path is None and path.name != "mod.rs":
            module_path = local_module_path(path.parent / path.stem, module_name)
        if module_path is None:
            return match.group(0)
        return read_rust_with_local_sources(module_path, seen)

    return LOCAL_MOD_RE.sub(expand_mod, INCLUDE_RE.sub(expand, text))


def non_test_compiler_sources() -> list[pathlib.Path]:
    sources = []
    for path in git_ls_files("crates/**/*.rs"):
        rel = path.relative_to(ROOT)
        parts = rel.parts
        name = path.name
        if parts[:2] == ("crates", "sifr_diagnostics"):
            continue
        if "tests" in parts or name == "tests.rs" or name.endswith("_tests.rs"):
            continue
        sources.append(path)
    return sources


def parse_registry() -> tuple[dict[str, str], dict[str, str], dict[str, str]]:
    text = read_rust_with_local_sources(CODES_RS)
    constants = dict(
        re.findall(
            rf"pub const ([A-Z0-9_]+): Self\s*=\s*(?:\n\s*)?Self::new\(\"({CODE_RE})\"",
            text,
        )
    )
    code_to_constant = {code: name for name, code in constants.items()}

    active_block = re.search(
        r"ACTIVE_DIAGNOSTIC_CODES:\s*&\[DiagnosticCode\]\s*=\s*&\[(?P<body>.*?)\];",
        text,
        re.S,
    )
    if active_block is None:
        raise ValueError("could not find ACTIVE_DIAGNOSTIC_CODES")
    active_constants = {
        name
        for name in re.findall(r"DiagnosticCode::([A-Z0-9_]+)", active_block.group("body"))
    }

    active_code_to_constant = {
        code: name for name, code in constants.items() if name in active_constants
    }
    active_fixtures = {
        code: fixture
        for code, fixture in re.findall(
            rf'active_entry!\(\s*"({CODE_RE})".*?Severity::[A-Za-z]+\s*,\s*"([^"]+)"',
            text,
            re.S,
        )
    }
    return code_to_constant, active_code_to_constant, active_fixtures


def fixture_file_exists(fixture: str) -> bool:
    path_part = fixture.split("::", 1)[0]
    return (ROOT / path_part).exists()


def main() -> int:
    errors: list[str] = []
    code_to_constant, active_code_to_constant, active_fixtures = parse_registry()
    active_constants = set(active_code_to_constant.values())
    all_constants = set(code_to_constant.values())

    catch_all_code = "SIFR-TYPE-" + "0001"
    if catch_all_code in active_code_to_constant:
        errors.append(f"{catch_all_code} must not be active as a semantic catch-all")

    for code, fixture in sorted(active_fixtures.items()):
        if code not in active_code_to_constant:
            errors.append(f"{code} is active in the registry but missing from ACTIVE_DIAGNOSTIC_CODES")
        if not fixture_file_exists(fixture):
            errors.append(f"{code} representative fixture does not exist: {fixture}")
        docs_page = ROOT / "docs" / "errors" / f"{code}.md"
        if not docs_page.exists():
            errors.append(f"{code} active docs page is missing: {docs_page.relative_to(ROOT)}")

    uses_by_constant = {name: [] for name in active_constants}
    for path in non_test_compiler_sources():
        if not path.exists():
            continue
        rel = path.relative_to(ROOT)
        text = strip_cfg_test_blocks(read_rust_with_local_sources(path))
        for name in re.findall(r"DiagnosticCode::([A-Z0-9_]+)", text):
            if name not in all_constants:
                errors.append(f"{rel}: references unknown DiagnosticCode::{name}")
                continue
            if name not in active_constants:
                errors.append(f"{rel}: references non-active DiagnosticCode::{name}")
                continue
            uses_by_constant[name].append(str(rel))

    for code, name in sorted(active_code_to_constant.items()):
        if not uses_by_constant.get(name):
            errors.append(
                f"{code} ({name}) is active but has no non-test compiler-source DiagnosticCode::{name} use"
            )

    if errors:
        for error in errors:
            print(f"diagnostic code coverage: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
