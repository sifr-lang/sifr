#!/usr/bin/env python3
"""Guard the bottom-of-graph dependency contract for `sifr_source`."""

from __future__ import annotations

import pathlib
import sys
import tomllib


REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
SOURCE_CRATE = REPO_ROOT / "crates" / "sifr_source"
ALLOWED_DEPENDENCIES = {"ruff_text_size"}
FORBIDDEN_CRATES = {
    "sifr_diagnostics",
    "sifr_syntax",
    "sifr_frontend",
    "sifr_analysis",
    "sifr_lsp",
    "sifr_lint",
    "sifr_format",
    "sifr_package",
    "sifr_lowering",
    "sifr_type_system",
    "sifr_codegen",
    "sifr_driver",
    "sifr",
}


def main() -> int:
    manifest = tomllib.loads((SOURCE_CRATE / "Cargo.toml").read_text())
    dependencies = set(manifest.get("dependencies", {}))
    unexpected = sorted(dependencies - ALLOWED_DEPENDENCIES)
    if unexpected:
        print(
            "sifr_source has non-bottom dependency/dependencies: "
            + ", ".join(unexpected),
            file=sys.stderr,
        )
        return 1

    violations: list[str] = []
    for path in sorted((SOURCE_CRATE / "src").rglob("*.rs")):
        text = path.read_text()
        for crate in sorted(FORBIDDEN_CRATES):
            if f"use {crate}" in text or f"{crate}::" in text:
                violations.append(f"{path.relative_to(REPO_ROOT)} references {crate}")

    if violations:
        print("sifr_source depends upward in source code:", file=sys.stderr)
        for violation in violations:
            print(f"  - {violation}", file=sys.stderr)
        return 1

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
