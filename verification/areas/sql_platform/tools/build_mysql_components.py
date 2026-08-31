#!/usr/bin/env python3
"""Build and record the pure-Rust MySQL compiler components."""

from __future__ import annotations

import hashlib
import json
import shutil
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
CRATE = ROOT / "crates/sifr_sql_mysql"
OUTPUT = ROOT / "target/wasm32-wasip2/release/sifr_sql_mysql.wasm"
SERIES = ["8.4", "9.7", "26.7"]


def main() -> int:
    subprocess.run(
        ["cargo", "build", "--locked", "--release", "--target", "wasm32-wasip2", "-p", "sifr_sql_mysql"],
        cwd=ROOT,
        check=True,
    )
    component_dir = CRATE / "components"
    component_dir.mkdir(exist_ok=True)
    artifacts = []
    for series in SERIES:
        destination = component_dir / f"mysql-{series}.wasm"
        shutil.copyfile(OUTPUT, destination)
        payload = destination.read_bytes()
        artifacts.append(
            {
                "series": series,
                "path": str(destination.relative_to(CRATE)),
                "sha256": hashlib.sha256(payload).hexdigest(),
                "size_bytes": len(payload),
            }
        )
    manifest = {
        "schema_version": 1,
        "target": "wasm32-wasip2",
        "wit_world": "embedded-language-provider",
        "protocol_major": 1,
        "parser_generator": {"name": "lalrpop", "version": "0.23.1"},
        "artifacts": artifacts,
    }
    (CRATE / "component-artifacts.json").write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
