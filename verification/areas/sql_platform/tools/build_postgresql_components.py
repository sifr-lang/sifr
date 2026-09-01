#!/usr/bin/env python3
"""Build the six pinned PostgreSQL compiler components."""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
from pathlib import Path

from postgresql_component_inputs import guest_source_sha256
from wasi_virt_inputs import (
    WASI_VIRT_COMMIT,
    WASI_VIRT_SOURCE_SHA256,
    WASI_VIRT_VERSION,
    validate_wasi_virt,
)

REPO_ROOT = Path(__file__).resolve().parents[4]
CRATE = REPO_ROOT / "crates/sifr_sql_postgresql"
OUTPUT = CRATE / "components"
MAJORS = range(13, 19)
ARTIFACT_MANIFEST = CRATE / "component-artifacts.json"
WASI_VIRT = REPO_ROOT / "third_party/wasi-virt"
WASI_SDK_ASSET_SHA256 = "85c997a2665ead91673b5bb88b7d0df3fc8900df3bfa244f720d478187bbdc78"


def main() -> int:
    sdk = os.environ.get("WASI_SDK_PATH")
    if not sdk:
        raise SystemExit("WASI_SDK_PATH must point to the pinned wasi-sdk 33 installation")
    compiler = Path(sdk) / "bin/clang"
    sysroot = Path(sdk) / "share/wasi-sysroot"
    if not compiler.is_file() or not sysroot.is_dir():
        raise SystemExit("WASI_SDK_PATH is not a complete wasi-sdk installation")
    version_file = Path(sdk) / "VERSION"
    if not version_file.is_file() or not version_file.read_text(encoding="utf-8").startswith("33.0"):
        raise SystemExit("PostgreSQL components require the pinned wasi-sdk 33.0 release")
    validate_wasi_virt(WASI_VIRT)
    virt_manifest = WASI_VIRT / "Cargo.toml"
    subprocess.run(
        [
            "cargo",
            "build",
            "--locked",
            "--release",
            "--no-default-features",
            "--manifest-path",
            str(virt_manifest),
        ],
        cwd=REPO_ROOT,
        check=True,
    )
    virtualizer = WASI_VIRT / "target/release/wasi-virt"

    OUTPUT.mkdir(parents=True, exist_ok=True)
    environment = dict(os.environ)
    environment["WASI_SDK_PATH"] = sdk
    sources = json.loads((CRATE / "component-sources.json").read_text(encoding="utf-8"))
    source_by_major = {row["server_major"]: row for row in sources["sources"]}
    artifacts = []
    for major in MAJORS:
        environment["SIFR_POSTGRESQL_MAJOR"] = str(major)
        subprocess.run(
            [
                "cargo",
                "build",
                "--locked",
                "--release",
                "--package",
                "sifr_sql_postgresql",
                "--target",
                "wasm32-wasip2",
            ],
            cwd=REPO_ROOT,
            env=environment,
            check=True,
        )
        source = REPO_ROOT / "target/wasm32-wasip2/release/sifr_sql_postgresql.wasm"
        destination = OUTPUT / f"postgresql-{major}.wasm"
        subprocess.run(
            [str(virtualizer), str(source), "-o", str(destination)],
            cwd=REPO_ROOT,
            check=True,
        )
        payload = destination.read_bytes()
        parser = source_by_major[major]
        artifacts.append(
            {
                "server_major": major,
                "path": f"components/postgresql-{major}.wasm",
                "sha256": hashlib.sha256(payload).hexdigest(),
                "size_bytes": len(payload),
                "parser_tag": parser["tag"],
                "parser_commit": parser["commit"],
            }
        )
    manifest = {
        "schema_version": 1,
        "target": "wasm32-wasip2",
        "wit_world": "embedded-language-provider",
        "protocol_major": 1,
        "guest_source_sha256": guest_source_sha256(REPO_ROOT),
        "toolchain": {
            "wasi_sdk": "33.0",
            "wasi_sdk_asset_sha256": WASI_SDK_ASSET_SHA256,
            "wasi_virt": WASI_VIRT_VERSION,
            "wasi_virt_commit": WASI_VIRT_COMMIT,
            "wasi_virt_source_sha256": WASI_VIRT_SOURCE_SHA256,
            "wit_bindgen": "0.61.1",
        },
        "artifacts": artifacts,
    }
    ARTIFACT_MANIFEST.write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
