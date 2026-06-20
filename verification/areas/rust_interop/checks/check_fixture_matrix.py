"""Validate the Rust interop fixture matrix scaffold."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[4]
AREA_ROOT = REPO_ROOT / "verification" / "areas" / "rust_interop"
MATRIX_PATH = AREA_ROOT / "data" / "rust_interop_fixture_matrix.json"
FIXTURES_ROOT = AREA_ROOT / "fixtures"

REQUIRED_FIXTURES = {
    "advanced_data_matrix",
    "arrow_record_batch",
    "async_ecosystem_matrix",
    "async_runtime_reqwest",
    "blocking_diagnostics",
    "bridge_type_matrix",
    "bridge_version_mismatch",
    "callback_subscription_matrix",
    "callbacks_call_scoped",
    "callbacks_threadsafe",
    "cargo_locked_offline",
    "close_after_use",
    "direct_crate_crc32",
    "direct_crate_matrix",
    "direct_crate_negative_type",
    "dotted_path_resolution",
    "ecosystem_backend_certification",
    "ecosystem_cli_certification",
    "local_bridge_blake3",
    "native_build_script",
    "opaque_handle_tokenizer",
    "opaque_resource_matrix",
    "panic_abort_profile",
    "panic_boundary",
    "proc_macro_trust",
    "same_workspace_crate",
    "shared_bridge_crate",
    "tensor_dlpack_bridge",
    "zero_copy_bytes",
    "zero_copy_view_matrix",
}

REQUIRED_DIAGNOSTICS = {
    "SIFR-RUST-ASYNC-0001",
    "SIFR-RUST-CARGO-0001",
    "SIFR-RUST-CB-0001",
    "SIFR-RUST-CONFIG-0001",
    "SIFR-RUST-HANDLE-0001",
    "SIFR-RUST-PANIC-0001",
    "SIFR-RUST-RESOLVE-0001",
    "SIFR-RUST-TRUST-0001",
    "SIFR-RUST-TYPE-0001",
    "SIFR-RUST-ZC-0001",
}

REQUIRED_CRATES = {
    "anyhow",
    "arrow",
    "axum",
    "bindgen",
    "blake3",
    "bytemuck",
    "bytes",
    "candle",
    "cc",
    "clap",
    "crc32fast",
    "cxx",
    "datafusion",
    "flate2",
    "futures",
    "http",
    "http-body",
    "indexmap",
    "memmap2",
    "ndarray",
    "notify",
    "polars",
    "prost-build",
    "rayon",
    "redis",
    "regex",
    "reqwest",
    "rusqlite",
    "serde",
    "serde_derive",
    "serde_json",
    "sha2",
    "sqlx",
    "thiserror",
    "tokio",
    "tokio-postgres",
    "tokio-tungstenite",
    "tower",
    "tower-http",
    "tracing",
    "tracing-subscriber",
    "uuid",
    "zerocopy",
    "zstd",
}

VALID_EVIDENCE_STATUS = {"planned", "probe-only", "runtime-observed", "passing", "failing"}

EXPECTED_FEATURE_POLICIES = {
    "candle": {"backend": "cpu-only"},
    "flate2": {"default_features": False, "features": ["rust_backend"]},
    "prost-build": {"generated_output": "deterministic"},
    "redis": {"default_features": False, "features": ["tokio-comp"]},
    "reqwest": {"default_features": False, "features": ["rustls-tls", "json"]},
    "rusqlite": {"features": ["bundled"]},
    "sqlx": {
        "default_features": False,
        "features": ["runtime-tokio-rustls", "postgres", "macros"],
    },
    "tokio-postgres": {"default_features": False, "features": ["runtime"]},
    "tokio-tungstenite": {"default_features": False},
    "tracing-subscriber": {"features": ["env-filter"]},
}


def main() -> int:
    matrix = json.loads(MATRIX_PATH.read_text(encoding="utf-8"))
    failures: list[str] = []

    if matrix.get("schema_version") != 1:
        failures.append("matrix schema_version must be 1")
    if matrix.get("phase") != "39_rust_interop":
        failures.append("matrix phase must be 39_rust_interop")
    if matrix.get("bridge_version") != 1:
        failures.append("matrix bridge_version must be 1")

    diagnostics = set(matrix.get("diagnostic_families", {}))
    failures.extend(
        f"missing diagnostic family reservation: {code}"
        for code in sorted(REQUIRED_DIAGNOSTICS.difference(diagnostics))
    )
    unexpected_diagnostics = diagnostics.difference(REQUIRED_DIAGNOSTICS)
    failures.extend(
        f"unexpected diagnostic family reservation: {code}"
        for code in sorted(unexpected_diagnostics)
    )

    fixtures = matrix.get("fixtures", [])
    if not isinstance(fixtures, list):
        failures.append("fixtures must be a list")
        fixtures = []
    fixture_ids = [str(fixture.get("id")) for fixture in fixtures if isinstance(fixture, dict)]
    fixture_id_set = set(fixture_ids)
    if len(fixture_ids) != len(fixture_id_set):
        failures.append("fixture ids must be unique")
    failures.extend(f"missing fixture matrix entry: {item}" for item in sorted(REQUIRED_FIXTURES - fixture_id_set))
    failures.extend(f"unexpected fixture matrix entry: {item}" for item in sorted(fixture_id_set - REQUIRED_FIXTURES))

    discovered_dirs = {path.name for path in FIXTURES_ROOT.iterdir() if path.is_dir()}
    failures.extend(f"missing fixture directory: {item}" for item in sorted(REQUIRED_FIXTURES - discovered_dirs))
    failures.extend(f"unexpected fixture directory: {item}" for item in sorted(discovered_dirs - REQUIRED_FIXTURES))

    covered_crates: set[str] = set()
    for fixture in fixtures:
        if not isinstance(fixture, dict):
            failures.append("fixture entries must be objects")
            continue
        fixture_id = str(fixture.get("id"))
        tier = fixture.get("tier")
        if tier not in {0, 1, 2, 3, 4}:
            failures.append(f"{fixture_id}: tier must be 0..4")
        if not fixture.get("capability"):
            failures.append(f"{fixture_id}: capability is required")
        if fixture.get("execution_kind") not in {"compiler-diagnostic", "contract-only", "cargo-probe", "runtime-observed"}:
            failures.append(f"{fixture_id}: invalid execution_kind")
        crates = fixture.get("required_crates", [])
        if not isinstance(crates, list):
            failures.append(f"{fixture_id}: required_crates must be a list")
            crates = []
        covered_crates.update(str(crate) for crate in crates)
        _validate_feature_policies(failures, fixture_id, fixture.get("features"), crates)
        _validate_evidence(failures, fixture_id, fixture.get("positive_evidence"), "positive_evidence")
        _validate_evidence(failures, fixture_id, fixture.get("negative_evidence"), "negative_evidence")

    failures.extend(f"required crate lacks fixture coverage: {crate}" for crate in sorted(REQUIRED_CRATES - covered_crates))

    if failures:
        for failure in failures:
            print(f"rust interop fixture matrix error: {failure}", file=sys.stderr)
        return 1
    print(
        "rust interop fixture matrix ok: "
        f"fixtures={len(fixture_id_set)} diagnostics={len(diagnostics)} crates={len(covered_crates)}"
    )
    return 0


def _validate_evidence(failures: list[str], fixture_id: str, value: Any, field: str) -> None:
    if not isinstance(value, dict):
        failures.append(f"{fixture_id}: {field} must be an object")
        return
    if not value.get("id"):
        failures.append(f"{fixture_id}: {field}.id is required")
    status = value.get("status")
    if status not in VALID_EVIDENCE_STATUS:
        failures.append(f"{fixture_id}: {field}.status is invalid")


def _validate_feature_policies(
    failures: list[str],
    fixture_id: str,
    raw_features: Any,
    crates: list[Any],
) -> None:
    required_pins = {
        str(crate): EXPECTED_FEATURE_POLICIES[str(crate)]
        for crate in crates
        if str(crate) in EXPECTED_FEATURE_POLICIES
    }
    if not required_pins:
        return
    if not isinstance(raw_features, dict):
        failures.append(f"{fixture_id}: missing features block for feature-sensitive crates")
        return
    for crate, expected in sorted(required_pins.items()):
        actual = raw_features.get(crate)
        if actual != expected:
            failures.append(
                f"{fixture_id}: feature policy for {crate} must be {expected!r}, got {actual!r}"
            )


if __name__ == "__main__":
    raise SystemExit(main())
