"""Validate current compiler artifacts against their actual producer receipts."""

from __future__ import annotations

import copy
import json
from pathlib import Path

from component_provenance import FAMILIES, ROOT, SERIES, require_digest, validate_provenance
from wasi_sdk_inputs import sha256


def validate_manifest(manifest: dict, family: str, root: Path = ROOT) -> None:
    expected_schema = 1 if family == "words" else 2
    if manifest.get("schema_version") != expected_schema:
        raise ValueError("component artifact manifest schema has drifted")
    validate_provenance(manifest.get("provenance"), family, root)
    directory = root / FAMILIES[family]
    if family == "words":
        if manifest.get("target") != "wasm32-unknown-unknown" or manifest.get("artifact") != "words_component.wasm":
            raise ValueError("words artifact target or path differs")
        rows = [dict(manifest, path=manifest["artifact"])]
    else:
        if (manifest.get("target"), manifest.get("wit_world"), manifest.get("protocol_major")) != (
            "wasm32-wasip2", "embedded-language-provider", 1
        ):
            raise ValueError("SQL component boundary differs")
        rows = manifest.get("artifacts")
        key = "server_major" if family == "postgresql" else "series"
        if not isinstance(rows, list) or [row.get(key) for row in rows] != SERIES[family]:
            raise ValueError("component supported major coverage differs")
        for row in rows:
            if row.get("path") != f"components/{family}-{row[key]}.wasm":
                raise ValueError("component artifact path differs")
    for row in rows:
        artifact = directory / row["path"]
        if not artifact.is_file() or row.get("sha256") != sha256(artifact) or row.get("size_bytes") != artifact.stat().st_size:
            raise ValueError("component artifact digest or size differs")
        with artifact.open("rb") as stream:
            if stream.read(8) != b"\0asm\r\0\x01\0":
                raise ValueError("artifact is not a WebAssembly component")
        require_digest(row.get("core_sha256"))


def self_test() -> int:
    """Reject meaningful stale-source, producer and major-coverage mutations."""
    count = 0
    for family, relative in FAMILIES.items():
        manifest = json.loads((ROOT / relative / "component-artifacts.json").read_text())
        mutations = []
        missing_source = copy.deepcopy(manifest)
        missing_source["provenance"]["source_inputs"].pop(next(iter(missing_source["provenance"]["source_inputs"])))
        mutations.append(missing_source)
        wrong_rust = copy.deepcopy(manifest)
        wrong_rust["provenance"]["rust"]["rustc_version"] = "rustc 1.97.0 (old)"
        mutations.append(wrong_rust)
        missing_tool = copy.deepcopy(manifest)
        missing_tool["provenance"]["tools"] = {}
        mutations.append(missing_tool)
        if family != "words":
            missing_major = copy.deepcopy(manifest)
            missing_major["artifacts"].pop()
            mutations.append(missing_major)
            if family != "mysql":
                for field in ("archive_sha256", "compiler_sha256", "sysroot_sha256"):
                    invalid_sdk = copy.deepcopy(manifest)
                    invalid_sdk["provenance"]["tools"]["sdk"][field] = "missing"
                    mutations.append(invalid_sdk)
        for mutation in mutations:
            try:
                validate_manifest(mutation, family)
            except ValueError:
                count += 1
            else:
                raise ValueError(f"{family} producer mutation was accepted")
    return count
