#!/usr/bin/env python3
"""Verify the checked-in diagnostics JSON Schema matches the Rust model."""

from __future__ import annotations

import pathlib
import subprocess
import sys


ROOT = pathlib.Path(__file__).resolve().parents[4]
SCHEMA_PATH = ROOT / "docs" / "schemas" / "diagnostics.schema.json"
GENERATOR_COMMAND = [
    "cargo", "run", "--locked", "-q", "-p", "sifr_diagnostics",
    "--bin", "gen-diagnostic-schema",
]


def main() -> int:
    generated = subprocess.run(
        GENERATOR_COMMAND,
        cwd=ROOT,
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if generated.returncode != 0:
        sys.stderr.write(
            "schema sync: failed to invoke generator: "
            "cargo run --locked -q -p sifr_diagnostics --bin gen-diagnostic-schema\n"
        )
        sys.stderr.write(generated.stderr)
        return generated.returncode

    expected = generated.stdout
    actual = SCHEMA_PATH.read_text(encoding="utf-8") if SCHEMA_PATH.exists() else ""
    if actual != expected:
        sys.stderr.write(
            "docs/schemas/diagnostics.schema.json is out of sync. "
            "Run `cargo run --locked -q -p sifr_diagnostics --bin gen-diagnostic-schema "
            "> docs/schemas/diagnostics.schema.json`.\n"
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
