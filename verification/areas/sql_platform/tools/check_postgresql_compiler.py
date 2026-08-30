#!/usr/bin/env python3
"""Validate the PostgreSQL compiler source and evidence authority."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import re
import subprocess
import tomllib
from pathlib import Path
from typing import Any

from postgresql_component_inputs import guest_source_sha256

REPO_ROOT = Path(__file__).resolve().parents[4]
RECORD = REPO_ROOT / "verification/areas/sql_platform/data/postgresql_compiler_qualification.json"
ARTIFACT_MANIFEST = REPO_ROOT / "crates/sifr_sql_postgresql/component-artifacts.json"
HEX40 = re.compile(r"[0-9a-f]{40}")
HEX64 = re.compile(r"[0-9a-f]{64}")
MAJORS = list(range(13, 19))


class ContractError(ValueError):
    """The PostgreSQL compiler qualification is invalid."""


def validate(payload: Any) -> None:
    require(isinstance(payload, dict) and payload.get("schema_version") == 1, "schema_version must be 1")
    require(payload.get("component_crate") == "sifr_sql_postgresql", "component crate has drifted")
    require(
        payload.get("artifact_manifest") == "crates/sifr_sql_postgresql/component-artifacts.json",
        "artifact manifest authority has drifted",
    )
    require(payload.get("supported_server_majors") == MAJORS, "supported PostgreSQL majors have drifted")
    source_path = REPO_ROOT / str(payload.get("source_manifest"))
    sources = json.loads(source_path.read_text(encoding="utf-8"))
    require(sources.get("target") == "wasm32-wasip2", "component target has drifted")
    rows = sources.get("sources")
    require(isinstance(rows, list) and [row.get("server_major") for row in rows] == MAJORS, "source matrix is incomplete")
    baseline = tomllib.loads((REPO_ROOT / "verification/areas/sql_platform/dependency_baseline.toml").read_text(encoding="utf-8"))
    baseline_sources = {
        int(row["server_major"]): (row["tag"], row["commit"])
        for row in baseline["source"]
        if row["name"] == "libpg_query"
    }
    for row in rows:
        validate_source(row, baseline_sources)
    validate_artifacts(rows)
    validate_implementation(payload)
    validate_live_evidence(payload)


def validate_source(row: dict[str, Any], baseline: dict[int, tuple[str, str]]) -> None:
    major = int(row["server_major"])
    commit = str(row.get("commit"))
    checksum = str(row.get("source_content_sha256"))
    require(HEX40.fullmatch(commit) is not None, f"PostgreSQL {major} commit is invalid")
    require(HEX64.fullmatch(checksum) is not None, f"PostgreSQL {major} checksum is invalid")
    require(baseline.get(major) == (row.get("tag"), commit), f"PostgreSQL {major} baseline has drifted")
    source = (REPO_ROOT / "crates/sifr_sql_postgresql" / str(row["path"])).resolve()
    require(source.is_dir(), f"PostgreSQL {major} source is not initialized")
    head = git_output(source, ["rev-parse", "HEAD"]).decode().strip()
    require(head == commit, f"PostgreSQL {major} submodule commit has drifted")
    digest = hashlib.sha256()
    tracked = git_output(source, ["ls-files", "-z"]).split(b"\0")
    for raw_path in tracked:
        if not raw_path:
            continue
        digest.update(raw_path)
        digest.update(b"\0")
        digest.update(hashlib.sha256((source / raw_path.decode()).read_bytes()).digest())
    require(digest.hexdigest() == checksum, f"PostgreSQL {major} source content checksum has drifted")


def validate_artifacts(sources: list[dict[str, Any]]) -> None:
    manifest = json.loads(ARTIFACT_MANIFEST.read_text(encoding="utf-8"))
    require(manifest.get("schema_version") == 1, "artifact manifest schema has drifted")
    require(manifest.get("target") == "wasm32-wasip2", "artifact target has drifted")
    require(manifest.get("wit_world") == "embedded-language-provider", "WIT world has drifted")
    require(manifest.get("protocol_major") == 1, "component protocol has drifted")
    require(
        manifest.get("guest_source_sha256") == guest_source_sha256(REPO_ROOT),
        "component guest source checksum has drifted",
    )
    toolchain = manifest.get("toolchain")
    require(isinstance(toolchain, dict), "component toolchain is missing")
    require(toolchain.get("wasi_sdk") == "33.0", "wasi-sdk pin has drifted")
    require(toolchain.get("wasi_virt") == "0.2.0", "WASI-Virt pin has drifted")
    require(
        toolchain.get("wasi_virt_commit") == "448f6df8f688cee5d6995e96b1ffc31f9bf00742",
        "WASI-Virt commit has drifted",
    )
    require(
        toolchain.get("wasi_virt_source_sha256")
        == "47c1ca1cc80df330c93c4797f6748d5330c2804001bdcff0342c4001920d1d2e",
        "WASI-Virt source checksum has drifted",
    )
    require(toolchain.get("wit_bindgen") == "0.61.1", "wit-bindgen pin has drifted")
    require(
        HEX64.fullmatch(str(toolchain.get("wasi_sdk_asset_sha256"))) is not None,
        "wasi-sdk asset checksum is invalid",
    )
    virt = REPO_ROOT / "third_party/wasi-virt"
    require(virt.is_dir(), "WASI-Virt source is not initialized")
    require(
        git_output(virt, ["rev-parse", "HEAD"]).decode().strip()
        == toolchain["wasi_virt_commit"],
        "WASI-Virt source commit has drifted",
    )
    digest = hashlib.sha256()
    for raw_path in git_output(virt, ["ls-files", "-z"]).split(b"\0"):
        if raw_path:
            digest.update(raw_path)
            digest.update(b"\0")
            digest.update(hashlib.sha256((virt / raw_path.decode()).read_bytes()).digest())
    require(
        digest.hexdigest() == toolchain["wasi_virt_source_sha256"],
        "WASI-Virt source content has drifted",
    )
    rows = manifest.get("artifacts")
    require(isinstance(rows, list) and [row.get("server_major") for row in rows] == MAJORS, "artifact matrix is incomplete")
    source_by_major = {int(row["server_major"]): row for row in sources}
    for row in rows:
        major = int(row["server_major"])
        expected_path = f"components/postgresql-{major}.wasm"
        require(row.get("path") == expected_path, f"PostgreSQL {major} artifact path has drifted")
        artifact = REPO_ROOT / "crates/sifr_sql_postgresql" / expected_path
        payload = artifact.read_bytes()
        require(payload.startswith(b"\0asm\r\0\x01\0"), f"PostgreSQL {major} is not a WebAssembly component")
        require(len(payload) == row.get("size_bytes"), f"PostgreSQL {major} artifact size has drifted")
        require(hashlib.sha256(payload).hexdigest() == row.get("sha256"), f"PostgreSQL {major} artifact hash has drifted")
        source = source_by_major[major]
        require(row.get("parser_tag") == source.get("tag"), f"PostgreSQL {major} artifact tag has drifted")
        require(row.get("parser_commit") == source.get("commit"), f"PostgreSQL {major} artifact commit has drifted")


def validate_implementation(payload: dict[str, Any]) -> None:
    sources = {
        "analysis": read("crates/sifr_sql_postgresql/src/analysis.rs"),
        "analyzer": read("crates/sifr_sql_postgresql/src/analyzer.rs"),
        "catalog": read("crates/sifr_sql_postgresql/src/catalog.rs"),
        "component": read("crates/sifr_sql_postgresql/src/component.rs"),
        "cardinality": read("crates/sifr_sql_postgresql/src/cardinality_analysis.rs"),
        "expression_operators": read("crates/sifr_sql_postgresql/src/expression_operators.rs"),
        "ffi": read("crates/sifr_sql_postgresql/src/ffi.rs"),
        "parameters": read("crates/sifr_sql_postgresql/src/parameters.rs"),
        "raw_adapter": read("crates/sifr_sql_postgresql/src/raw_adapter.rs"),
        "raw_advanced": read("crates/sifr_sql_postgresql/src/raw_advanced.rs"),
        "writes": read("crates/sifr_sql_postgresql/src/writes.rs"),
        "capabilities": read("crates/sifr_package/src/sql_capabilities.rs"),
    }
    required = {
        "analysis": ["analyze_select", "expanded-select-star"],
        "cardinality": ["unique_predicate_cardinality", "apply_limit_and_offset"],
        "analyzer": ["impl<P: PostgresParser> DialectSemantics", "ProviderAnalysisError::Diagnostic"],
        "catalog": ["pub struct PostgresCatalog", "ddl_document", "SchemaObjectKind::MaterializedView"],
        "component": ["component_registration", "execute_embedded_request", "compute_plan_fingerprint"],
        "expression_operators": ["infer_function", "DatabaseType::Json", "DatabaseType::Range"],
        "ffi": ["pg_query_parse", "pg_query_free_parse_result", "pg_query_normalize"],
        "parameters": ["rewrite_parameter_slots", "copy_dollar_quote", "copy_block_comment"],
        "raw_adapter": ["CreateTableAsStmt", "RawAdapter", "PostgresParser"],
        "raw_advanced": ["common_tables", "window_specification", "create_range"],
        "writes": ["excluded", "ON CONFLICT", "write_analysis"],
        "capabilities": ["ResolvedPackageCapabilities", "PackageCapabilityResolver"],
    }
    for owner, tokens in required.items():
        require(all(token in sources[owner] for token in tokens), f"{owner} misses a required mechanism")
    require("unsafe extern \"C\"" in sources["ffi"], "FFI boundary is missing")
    for owner, text in sources.items():
        if owner != "ffi":
            require("unsafe {" not in text, f"unsafe code escaped the FFI boundary into {owner}")
    evidence = payload.get("evidence")
    require(isinstance(evidence, dict), "evidence map is missing")
    for path in evidence.values():
        require((REPO_ROOT / str(path)).is_file(), f"evidence path is missing: {path}")


def validate_live_evidence(payload: dict[str, Any]) -> None:
    evidence_path = REPO_ROOT / str(payload["evidence"]["live_server_matrix"])
    evidence = json.loads(evidence_path.read_text(encoding="utf-8"))
    require(evidence.get("schema_version") == 1, "live matrix schema has drifted")
    rows = evidence.get("servers")
    require(isinstance(rows, list) and [row.get("major") for row in rows] == MAJORS, "live server matrix is incomplete")
    for row in rows:
        major = int(row["major"])
        require(row.get("status") == "passed", f"PostgreSQL {major} live evidence did not pass")
        require(str(row.get("server_version_num", "")).startswith(str(major)), f"PostgreSQL {major} server version has drifted")
        require(re.fullmatch(r"sha256:[0-9a-f]{64}", str(row.get("image_digest"))) is not None, f"PostgreSQL {major} image digest is invalid")
        require(row.get("parameter_types") == "{bigint,text}", f"PostgreSQL {major} parameter evidence has drifted")
        require(row.get("result_types") == "bigint|text|text", f"PostgreSQL {major} result evidence has drifted")
        require(row.get("nullability") == "id:t|name:t|nickname:f", f"PostgreSQL {major} nullability evidence has drifted")
        require(row.get("write_result") == "1|second", f"PostgreSQL {major} write evidence has drifted")
        require(row.get("diagnostic_sqlstate") == "42703", f"PostgreSQL {major} diagnostic evidence has drifted")


def self_test(payload: dict[str, Any]) -> None:
    mutations: list[tuple[str, dict[str, Any]]] = []
    missing_major = copy.deepcopy(payload)
    missing_major["supported_server_majors"].pop()
    mutations.append(("server-major", missing_major))
    bad_target = copy.deepcopy(payload)
    bad_target["component_crate"] = "sifr_sql_contract"
    mutations.append(("component-owner", bad_target))
    missing_evidence = copy.deepcopy(payload)
    missing_evidence["evidence"]["architecture"] = "internal_docs/missing.md"
    mutations.append(("evidence", missing_evidence))
    for name, mutated in mutations:
        try:
            validate(mutated)
        except ContractError:
            continue
        raise ContractError(f"self-test mutation '{name}' was not detected")


def git_output(directory: Path, arguments: list[str]) -> bytes:
    result = subprocess.run(["git", "-C", str(directory), *arguments], check=False, capture_output=True)
    if result.returncode != 0:
        raise ContractError(result.stderr.decode(errors="replace").strip())
    return result.stdout


def read(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ContractError(message)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    payload = json.loads(RECORD.read_text(encoding="utf-8"))
    validate(payload)
    if args.self_test:
        self_test(payload)
    print("PostgreSQL compiler qualification ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
