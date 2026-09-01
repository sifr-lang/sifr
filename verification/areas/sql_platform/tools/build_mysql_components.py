#!/usr/bin/env python3
"""Build and record the pure-Rust MySQL compiler components."""

from __future__ import annotations

import hashlib
import json
import subprocess
from pathlib import Path

from wasi_virt_inputs import (
    WASI_VIRT_COMMIT,
    WASI_VIRT_SOURCE_SHA256,
    WASI_VIRT_VERSION,
    validate_wasi_virt,
)

ROOT = Path(__file__).resolve().parents[4]
CRATE = ROOT / "crates/sifr_sql_mysql"
OUTPUT = ROOT / "target/wasm32-wasip2/release/sifr_sql_mysql.wasm"
WASI_VIRT = ROOT / "third_party/wasi-virt/target/release/wasi-virt"
SERIES = ["8.4", "9.7", "26.7"]


def main() -> int:
    validate_wasi_virt(ROOT / "third_party/wasi-virt")
    if not WASI_VIRT.is_file():
        raise SystemExit("build the pinned third_party/wasi-virt tool before MySQL components")
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
        subprocess.run(
            [str(WASI_VIRT), str(OUTPUT), "-o", str(destination)],
            cwd=ROOT,
            check=True,
        )
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
        "wasi_virtualization": {
            "name": "wasi-virt",
            "version": WASI_VIRT_VERSION,
            "commit": WASI_VIRT_COMMIT,
            "source_content_sha256": WASI_VIRT_SOURCE_SHA256,
        },
        "artifacts": artifacts,
    }
    (CRATE / "component-artifacts.json").write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
