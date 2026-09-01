#!/usr/bin/env python3
"""Build and record the pinned SQLite compiler component."""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
from pathlib import Path

from wasi_virt_inputs import (
    WASI_VIRT_COMMIT,
    WASI_VIRT_SOURCE_SHA256,
    WASI_VIRT_VERSION,
    validate_wasi_virt,
)

ROOT = Path(__file__).resolve().parents[4]
CRATE = ROOT / "crates/sifr_sql_sqlite"
OUTPUT = ROOT / "target/wasm32-wasip2/release/sifr_sql_sqlite.wasm"
WASI_VIRT = ROOT / "third_party/wasi-virt/target/release/wasi-virt"


def main() -> int:
    validate_wasi_virt(ROOT / "third_party/wasi-virt")
    if not WASI_VIRT.is_file():
        raise SystemExit("build the pinned third_party/wasi-virt tool before SQLite components")
    environment = os.environ.copy()
    sdk = environment.get("WASI_SDK_PATH")
    sdk_candidates = [
        Path(sdk) if sdk else None,
        Path("/opt/wasi-sdk-33.0"),
        Path("/usr/local/wasi-sdk-33.0"),
    ]
    resolved_sdk = next(
        (
            path
            for path in sdk_candidates
            if path is not None
            and (path / "bin/clang").is_file()
            and (path / "share/wasi-sysroot").is_dir()
        ),
        None,
    )
    sysroot = environment.get("WASI_SYSROOT")
    candidates = [
        resolved_sdk / "share/wasi-sysroot" if resolved_sdk else None,
        Path(sysroot) if sysroot else None,
        Path("/opt/homebrew/opt/wasi-libc/share/wasi-sysroot"),
        Path("/usr/local/opt/wasi-libc/share/wasi-sysroot"),
    ]
    resolved = next((path for path in candidates if path is not None and path.is_dir()), None)
    if resolved is None:
        raise SystemExit(
            "SQLite component build requires WASI libc; set WASI_SYSROOT or install wasi-libc"
        )
    environment["WASI_SYSROOT"] = str(resolved)
    compiler = environment.get("CC_wasm32_wasip2")
    compiler_candidates = [
        resolved_sdk / "bin/clang" if resolved_sdk else None,
        Path(compiler) if compiler else None,
        Path("/opt/homebrew/opt/llvm/bin/clang"),
        Path("/usr/local/opt/llvm/bin/clang"),
    ]
    resolved_compiler = next(
        (path for path in compiler_candidates if path is not None and path.is_file()),
        None,
    )
    if resolved_compiler is None:
        raise SystemExit(
            "SQLite component build requires WASI SDK 33 or upstream Clang with the "
            "wasm32-wasip2 target; set WASI_SDK_PATH or CC_wasm32_wasip2"
        )
    environment["CC_wasm32_wasip2"] = str(resolved_compiler)
    subprocess.run(
        ["cargo", "build", "--locked", "--release", "--target", "wasm32-wasip2", "-p", "sifr_sql_sqlite"],
        cwd=ROOT,
        env=environment,
        check=True,
    )
    destination = CRATE / "components/sqlite-3.53.2.wasm"
    destination.parent.mkdir(exist_ok=True)
    subprocess.run(
        [str(WASI_VIRT), str(OUTPUT), "-o", str(destination)],
        cwd=ROOT,
        check=True,
    )
    payload = destination.read_bytes()
    manifest = {
        "schema_version": 1,
        "target": "wasm32-wasip2",
        "wit_world": "embedded-language-provider",
        "protocol_major": 1,
        "wasi_virtualization": {
            "name": "wasi-virt",
            "version": WASI_VIRT_VERSION,
            "commit": WASI_VIRT_COMMIT,
            "source_content_sha256": WASI_VIRT_SOURCE_SHA256,
        },
        "parser": {
            "name": "syntaqlite",
            "version": "0.9.0",
            "sqlite_version": "3.53.2",
            "sqlite_version_number": 3053002,
            "compile_flags": [],
        },
        "artifacts": [{
            "series": "3.53.2",
            "path": "components/sqlite-3.53.2.wasm",
            "sha256": hashlib.sha256(payload).hexdigest(),
            "size_bytes": len(payload),
        }],
    }
    (CRATE / "component-artifacts.json").write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
