#!/usr/bin/env python3
"""Reject forbidden production dependencies in editor tooling paths."""

from __future__ import annotations

import argparse
import sys
import tempfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]

SCAN_PREFIXES = [
    Path("Cargo.toml"),
    Path("crates"),
    Path("editors"),
    Path("vscode"),
    Path("packages"),
]

FORBIDDEN_PATTERNS = {
    "ty_python_semantic": "Python semantic authority is forbidden in Sifr tooling",
    "ty_project": "Python project semantics are forbidden in Sifr tooling",
    "ruff_server": "Ruff Server semantic behavior is forbidden as Sifr tooling authority",
    "pyright": "Pyright fallback is forbidden",
    "pylsp": "Python language-server fallback is forbidden",
    "python-language-server": "Python language-server fallback is forbidden",
}


def should_scan(path: Path) -> bool:
    if not path.is_file():
        return False
    if "target" in path.parts or ".git" in path.parts:
        return False
    if path.suffix not in {".rs", ".toml", ".json", ".ts", ".js", ".mjs", ".cjs"}:
        return False
    rel = path.relative_to(REPO_ROOT)
    return any(rel == prefix or rel.is_relative_to(prefix) for prefix in SCAN_PREFIXES)


def candidate_files() -> list[Path]:
    files: list[Path] = []
    for prefix in SCAN_PREFIXES:
        root = REPO_ROOT / prefix
        if root.is_file():
            files.append(root)
        elif root.exists():
            files.extend(path for path in root.rglob("*") if should_scan(path))
    return sorted(set(files))


def violations(paths: list[Path]) -> list[str]:
    failures: list[str] = []
    for path in paths:
        text = path.read_text(encoding="utf-8", errors="replace")
        for pattern, reason in FORBIDDEN_PATTERNS.items():
            if pattern in text:
                failures.append(f"{path.relative_to(REPO_ROOT)} contains {pattern!r}: {reason}")
                break
    return failures


def run_self_test() -> None:
    with tempfile.TemporaryDirectory(dir=REPO_ROOT / "target") as tmp:
        seed = Path(tmp) / "seed.rs"
        seed.write_text("use ty_python_semantic::lint;\n", encoding="utf-8")
        found = violations([seed])
    if not found:
        raise SystemExit("tooling dependency boundary self-test failed: seeded dependency passed")
    print("tooling dependency boundary self-test: PASS")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        run_self_test()
        return 0

    found = violations(candidate_files())
    if found:
        print("tooling dependency boundary: FAIL", file=sys.stderr)
        for failure in found:
            print(f"  - {failure}", file=sys.stderr)
        return 1
    print("tooling dependency boundary: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
