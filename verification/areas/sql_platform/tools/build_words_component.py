#!/usr/bin/env python3
"""Regenerate the closed synchronous words component and its producer receipt."""

from __future__ import annotations

import json
import os
import subprocess
import tempfile
from pathlib import Path

from component_provenance import ROOT, WORDS, guest_environment, provenance, rust_identity, target_directory
from wasi_sdk_inputs import sha256


def main() -> int:
    crate = ROOT / WORDS
    manifest = crate / "Cargo.toml"
    rust = rust_identity("wasm32-unknown-unknown")
    target = target_directory(manifest)
    environment = guest_environment(os.environ.copy())
    subprocess.run([
        "cargo", "build", "--manifest-path", str(manifest), "--target",
        "wasm32-unknown-unknown", "--release", "--locked", "--offline", "--lib",
    ], cwd=ROOT, env=environment, check=True)
    subprocess.run([
        "cargo", "build", "--manifest-path", str(manifest), "--features", "componentize",
        "--bin", "componentize", "--release", "--locked", "--offline",
    ], cwd=ROOT, check=True)
    encoder = target / "release" / ("componentize.exe" if os.name == "nt" else "componentize")
    core = target / "wasm32-unknown-unknown/release/sifr_component_fixture_words.wasm"
    with tempfile.TemporaryDirectory(prefix="words-component-", dir=target) as raw_stage:
        stage = Path(raw_stage)
        output = stage / "words_component.wasm"
        subprocess.run([str(encoder), str(core), str(output)], cwd=ROOT, check=True)
        record = {
            "schema_version": 1, "target": "wasm32-unknown-unknown",
            "artifact": "words_component.wasm", "sha256": sha256(output),
            "size_bytes": output.stat().st_size, "core_sha256": sha256(core),
            "provenance": provenance("words", rust, wit_bindgen="0.61.1", wit_component="0.258.0",
                                     componentizer_sha256=sha256(encoder)),
        }
        receipt = stage / "component-artifacts.json"
        receipt.write_text(json.dumps(record, indent=2, sort_keys=True) + "\n")
        output.replace(crate / output.name)
        receipt.replace(crate / receipt.name)
        qualification = ROOT / "verification/areas/sql_platform/data/compiler_component_qualification.json"
        data = json.loads(qualification.read_text())
        data["fixture"]["artifact_sha256"] = record["sha256"]
        data["fixture"]["build_tooling"] = {
            "rust_target": "wasm32-unknown-unknown", "wit_bindgen": "0.61.1", "wit_component": "0.258.0"
        }
        qualification.write_text(json.dumps(data, indent=2) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
