#!/usr/bin/env python3
"""Generate and validate architecture crate/profile inventories."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[3]
ARCHITECTURE = REPO_ROOT / "internal_docs" / "architecture.md"
PROFILE_ROOT = REPO_ROOT / "verification" / "profiles"
CRATE_BEGIN = "<!-- BEGIN GENERATED WORKSPACE CRATE MAP -->"
CRATE_END = "<!-- END GENERATED WORKSPACE CRATE MAP -->"
PROFILE_BEGIN = "<!-- BEGIN GENERATED VALIDATION PROFILE MAP -->"
PROFILE_END = "<!-- END GENERATED VALIDATION PROFILE MAP -->"
INLINE_CODE = re.compile(r"`([^`\n]+)`")
MARKDOWN_LINK = re.compile(r"\[[^]]*\]\(([^)]+)\)")
MACHINE_PATH = re.compile(
    r"(?:/(?:Users|home)/[^\s`)]+|/root/[^\s`)]+|[A-Za-z]:\\Users\\[^\s`)]+)"
)
NON_CRATE_SIFR_SYMBOLS = {
    "sifr_type_to_rust_field_type",
    "sifr_type_to_rust_type",
}
ARCHITECTURE_MUTATION_CASES = (
    "crate-map",
    "profile-map",
    "unknown-crate-path",
    "unknown-bare-crate",
    "missing-in-crate-path",
    "broken-link",
    "machine-path",
    "machine-home-path",
)


def cargo_metadata() -> dict[str, Any]:
    result = subprocess.run(
        ["cargo", "metadata", "--locked", "--no-deps", "--format-version", "1"],
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        detail = result.stderr.strip() or result.stdout.strip()
        raise SystemExit(f"cargo metadata failed: {detail}")
    payload = json.loads(result.stdout)
    if not isinstance(payload, dict):
        raise SystemExit("cargo metadata returned a non-object")
    return payload


def workspace_crates(metadata: dict[str, Any]) -> list[dict[str, Any]]:
    packages = metadata.get("packages", [])
    candidates: list[dict[str, Any]] = []
    for package in packages:
        if not isinstance(package, dict):
            continue
        manifest = Path(str(package.get("manifest_path", "")))
        try:
            relative = manifest.relative_to(REPO_ROOT)
        except ValueError:
            continue
        if len(relative.parts) != 3 or relative.parts[0] != "crates":
            continue
        candidates.append(package)
    names = {str(package["name"]) for package in candidates}
    metadata_paths = {
        Path(str(package["manifest_path"])).relative_to(REPO_ROOT).parent
        for package in candidates
    }
    disk_paths = {
        manifest.relative_to(REPO_ROOT).parent
        for manifest in (REPO_ROOT / "crates").glob("*/Cargo.toml")
    }
    if metadata_paths != disk_paths:
        missing = sorted(path.as_posix() for path in disk_paths - metadata_paths)
        extra = sorted(path.as_posix() for path in metadata_paths - disk_paths)
        raise SystemExit(
            f"workspace crate inventory mismatch: missing={missing}, unexpected={extra}"
        )
    rows: list[dict[str, Any]] = []
    for package in candidates:
        manifest = Path(str(package["manifest_path"])).relative_to(REPO_ROOT)
        dependencies = sorted(
            {
                str(dependency["name"])
                for dependency in package.get("dependencies", [])
                if isinstance(dependency, dict)
                and dependency.get("path") is not None
                and dependency.get("kind") is None
                and str(dependency.get("name")) in names
            }
        )
        rows.append(
            {
                "name": str(package["name"]),
                "path": manifest.parent.as_posix(),
                "dependencies": dependencies,
            }
        )
    return sorted(rows, key=lambda row: str(row["name"]))


def validation_profiles() -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for path in sorted(PROFILE_ROOT.glob("*.json")):
        payload = json.loads(path.read_text(encoding="utf-8"))
        selections = []
        for selection in payload.get("selected_areas", []):
            if not isinstance(selection, dict):
                continue
            suites = "+".join(str(suite) for suite in selection.get("suites", []))
            selections.append(f"{selection.get('area')}:{suites}")
        rows.append(
            {
                "name": str(payload.get("name", path.stem)),
                "path": path.relative_to(REPO_ROOT).as_posix(),
                "selections": selections,
            }
        )
    return rows


def render_crate_map(rows: list[dict[str, Any]]) -> str:
    lines = [
        CRATE_BEGIN,
        "| Crate | Workspace path | Direct normal first-party dependencies |",
        "| --- | --- | --- |",
    ]
    for row in rows:
        dependencies = ", ".join(f"`{name}`" for name in row["dependencies"]) or "—"
        lines.append(f"| `{row['name']}` | `{row['path']}` | {dependencies} |")
    lines.append(CRATE_END)
    return "\n".join(lines)


def render_profile_map(rows: list[dict[str, Any]]) -> str:
    lines = [
        PROFILE_BEGIN,
        "| Profile | Manifest | Selected area suites |",
        "| --- | --- | --- |",
    ]
    for row in rows:
        selections = (
            "<br>".join(f"`{selection}`" for selection in row["selections"]) or "—"
        )
        lines.append(f"| `{row['name']}` | `{row['path']}` | {selections} |")
    lines.append(PROFILE_END)
    return "\n".join(lines)


def replace_generated_section(text: str, begin: str, end: str, rendered: str) -> str:
    start = text.find(begin)
    finish = text.find(end)
    if start == -1 or finish == -1 or finish < start:
        raise ValueError(f"missing generated section markers: {begin} / {end}")
    finish += len(end)
    return text[:start] + rendered + text[finish:]


def expected_document(
    text: str, crates: list[dict[str, Any]], profiles: list[dict[str, Any]]
) -> str:
    updated = replace_generated_section(
        text, CRATE_BEGIN, CRATE_END, render_crate_map(crates)
    )
    return replace_generated_section(
        updated,
        PROFILE_BEGIN,
        PROFILE_END,
        render_profile_map(profiles),
    )


def validation_failures(
    text: str, crates: list[dict[str, Any]], profiles: list[dict[str, Any]]
) -> list[str]:
    failures: list[str] = []
    for label, begin, end, rendered in (
        ("crate-map", CRATE_BEGIN, CRATE_END, render_crate_map(crates)),
        ("profile-map", PROFILE_BEGIN, PROFILE_END, render_profile_map(profiles)),
    ):
        try:
            if replace_generated_section(text, begin, end, rendered) != text:
                failures.append(f"{label}-drift")
        except ValueError as error:
            failures.append(f"{label}-markers: {error}")
    crate_paths = {str(row["path"]) for row in crates}
    crate_names = {str(row["name"]) for row in crates}
    for code in INLINE_CODE.findall(text):
        if code.startswith("crates/"):
            root = "/".join(code.rstrip("/").split("/")[:2])
            if root not in crate_paths:
                failures.append(f"unknown-first-party-crate: {root}")
            elif not (REPO_ROOT / code).exists():
                failures.append(f"missing-first-party-path: {code}")
        elif (
            re.fullmatch(r"sifr_[a-z0-9_]+", code)
            and code not in crate_names
            and code not in NON_CRATE_SIFR_SYMBOLS
        ):
            failures.append(f"unknown-bare-first-party-crate: {code}")
    prose = re.sub(r"`[^`\n]*`", "", text)
    for target in MARKDOWN_LINK.findall(prose):
        target = target.split("#", 1)[0]
        if not target or "://" in target or target.startswith("mailto:"):
            continue
        resolved = (ARCHITECTURE.parent / target).resolve()
        if not resolved.exists():
            failures.append(f"broken-relative-link: {target}")
    for match in MACHINE_PATH.findall(text):
        failures.append(f"machine-local-path: {match}")
    return sorted(set(failures))


def run_self_test(crates: list[dict[str, Any]], profiles: list[dict[str, Any]]) -> None:
    source = ARCHITECTURE.read_text(encoding="utf-8")
    cases = {
        "crate-map": source.replace(CRATE_BEGIN, CRATE_BEGIN + "\ncorrupt", 1),
        "profile-map": source.replace(PROFILE_BEGIN, PROFILE_BEGIN + "\ncorrupt", 1),
        "unknown-crate-path": source + "\n`crates/sifr_missing/src/lib.rs`\n",
        "unknown-bare-crate": source + "\n`sifr_test_utils`\n",
        "missing-in-crate-path": source
        + "\n`crates/sifr_frontend/src/does_not_exist.rs`\n",
        "broken-link": source + "\n[missing](./missing-architecture-file.md)\n",
        "machine-path": source + "\n`/Users/example/work/sifr`\n",
        "machine-home-path": source + "\n`/home/example/work/sifr`\n",
    }
    if tuple(cases) != ARCHITECTURE_MUTATION_CASES:
        raise SystemExit("architecture guard mutation registration drifted")
    expected = {
        "crate-map": "crate-map-drift",
        "profile-map": "profile-map-drift",
        "unknown-crate-path": "unknown-first-party-crate",
        "unknown-bare-crate": "unknown-bare-first-party-crate",
        "missing-in-crate-path": "missing-first-party-path",
        "broken-link": "broken-relative-link",
        "machine-path": "machine-local-path",
        "machine-home-path": "machine-local-path",
    }
    for name, mutated in cases.items():
        failures = validation_failures(mutated, crates, profiles)
        if not any(failure.startswith(expected[name]) for failure in failures):
            raise SystemExit(
                f"architecture guard self-test did not reject {name}: {failures}"
            )
    print("architecture documentation guard self-test: PASS")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    crates = workspace_crates(cargo_metadata())
    profiles = validation_profiles()
    if args.self_test:
        run_self_test(crates, profiles)
        return 0
    source = ARCHITECTURE.read_text(encoding="utf-8")
    if args.write:
        ARCHITECTURE.write_text(
            expected_document(source, crates, profiles), encoding="utf-8"
        )
        print(f"updated {ARCHITECTURE.relative_to(REPO_ROOT)}")
        return 0
    failures = validation_failures(source, crates, profiles)
    if failures:
        print("architecture documentation guard: FAIL", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1
    run_self_test(crates, profiles)
    print("architecture documentation guard: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
