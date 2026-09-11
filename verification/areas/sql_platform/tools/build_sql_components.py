"""Build SQL components and record the actual source and tool identities."""

from __future__ import annotations

import json
import os
import subprocess
import tempfile
from pathlib import Path

from component_provenance import FAMILIES, ROOT, SERIES, guest_environment, provenance, rust_identity, target_directory, toml
from wasi_sdk_inputs import sdk_environment, sha256
from wasi_virt_inputs import WASI_VIRT_COMMIT, WASI_VIRT_SOURCE_SHA256, WASI_VIRT_VERSION, validate_wasi_virt


def build(family: str) -> int:
    crate = ROOT / FAMILIES[family]
    environment = os.environ.copy()
    sdk = None
    if family != "mysql":
        environment, sdk = sdk_environment()
    environment = guest_environment(environment)
    rust = rust_identity("wasm32-wasip2")
    virt = ROOT / "third_party/wasi-virt"
    validate_wasi_virt(virt)
    subprocess.run([
        "cargo", "build", "--locked", "--offline", "--release", "--no-default-features",
        "--manifest-path", str(virt / "Cargo.toml"),
    ], cwd=ROOT, check=True)
    virtualizer = target_directory(virt / "Cargo.toml") / "release" / ("wasi-virt.exe" if os.name == "nt" else "wasi-virt")
    virt_identity = {"version": WASI_VIRT_VERSION, "commit": WASI_VIRT_COMMIT,
                     "source_sha256": WASI_VIRT_SOURCE_SHA256,
                     "lock_sha256": sha256(virt / "Cargo.lock"),
                     "binary_sha256": sha256(virtualizer)}
    target = target_directory(ROOT / "Cargo.toml")
    target.mkdir(parents=True, exist_ok=True)
    core = target / f"wasm32-wasip2/release/sifr_sql_{family}.wasm"
    parsers = {}
    if family == "postgresql":
        sources = json.loads((crate / "component-sources.json").read_text())
        parsers = {row["server_major"]: row for row in sources["sources"]}
        # Actual checked-out parser bytes must match their pinned source receipt.
        from check_postgresql_compiler import validate_source
        baseline = toml(ROOT / "verification/areas/sql_platform/dependency_baseline.toml")
        expected = {int(row["server_major"]): (row["tag"], row["commit"])
                    for row in baseline["source"] if row["name"] == "libpg_query"}
        for row in sources["sources"]:
            validate_source(row, expected)
    artifacts = []
    manifest = {"schema_version": 2, "target": "wasm32-wasip2",
                "wit_world": "embedded-language-provider", "protocol_major": 1}
    with tempfile.TemporaryDirectory(prefix=f"{family}-components-", dir=target) as raw_stage:
        stage = Path(raw_stage)
        for index, series in enumerate(SERIES[family]):
            if family == "postgresql":
                environment["SIFR_POSTGRESQL_MAJOR"] = str(series)
            if family == "postgresql" or index == 0:
                subprocess.run([
                    "cargo", "build", "--locked", "--offline", "--release", "--target",
                    "wasm32-wasip2", "--package", f"sifr_sql_{family}",
                ], cwd=ROOT, env=environment, check=True)
            filename = f"{family}-{series}.wasm"
            output = stage / filename
            subprocess.run([str(virtualizer), str(core), "-o", str(output)], cwd=ROOT, check=True)
            artifact = {"path": f"components/{filename}", "sha256": sha256(output),
                        "size_bytes": output.stat().st_size, "core_sha256": sha256(core)}
            if family == "postgresql":
                parser = parsers[series]
                artifact.update(server_major=series, parser_tag=parser["tag"], parser_commit=parser["commit"],
                                parser_source_sha256=parser["source_content_sha256"])
            else:
                artifact["series"] = series
            artifacts.append(artifact)
        tools = {"wit_bindgen": "0.61.1", "virtualizer": virt_identity}
        if sdk is not None:
            tools["sdk"] = sdk
        manifest["provenance"] = provenance(family, rust, **tools)
        manifest["artifacts"] = artifacts
        if family == "mysql":
            manifest["parser_generator"] = {"name": "lalrpop", "version": "0.23.1"}
        elif family == "sqlite":
            manifest["parser"] = {"name": "syntaqlite", "version": "0.9.0", "sqlite_version": "3.53.2",
                                  "sqlite_version_number": 3053002, "compile_flags": []}
        (crate / "components").mkdir(exist_ok=True)
        for artifact in artifacts:
            destination = crate / artifact["path"]
            (stage / destination.name).replace(destination)
        receipt = stage / "component-artifacts.json"
        receipt.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
        receipt.replace(crate / "component-artifacts.json")
    return 0
