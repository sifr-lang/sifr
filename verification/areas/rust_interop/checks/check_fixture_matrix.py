"""Validate the Rust interop fixture matrix inventory."""

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
    "panic_boundary_wrapper_emission",
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
    for fixture_id in sorted(REQUIRED_FIXTURES & discovered_dirs):
        if not (FIXTURES_ROOT / fixture_id / "README.md").is_file():
            failures.append(f"{fixture_id}: fixture README.md is required for evidence notes")
        if not (FIXTURES_ROOT / fixture_id / "fixture.json").is_file():
            failures.append(f"{fixture_id}: fixture.json is required for evidence files")

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
        _validate_fixture_files(failures, fixture)

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


def _validate_fixture_files(failures: list[str], fixture: dict[str, Any]) -> None:
    fixture_id = str(fixture.get("id"))
    fixture_dir = FIXTURES_ROOT / fixture_id
    manifest_path = fixture_dir / "fixture.json"
    if not manifest_path.is_file():
        return
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        failures.append(f"{fixture_id}: fixture.json is not valid JSON: {error}")
        return
    if not isinstance(manifest, dict):
        failures.append(f"{fixture_id}: fixture.json must be an object")
        return

    for field in ("id", "capability", "tier", "execution_kind", "required_crates"):
        if manifest.get(field) != fixture.get(field):
            failures.append(f"{fixture_id}: fixture.json {field} must match fixture matrix")
    if manifest.get("features", {}) != fixture.get("features", {}):
        failures.append(f"{fixture_id}: fixture.json features must match fixture matrix")
    if manifest.get("schema_version") != 1:
        failures.append(f"{fixture_id}: fixture.json schema_version must be 1")
    if manifest.get("diagnostic_family") not in REQUIRED_DIAGNOSTICS:
        failures.append(f"{fixture_id}: fixture.json diagnostic_family must be a reserved SIFR-RUST code")

    evidence = manifest.get("evidence")
    if not isinstance(evidence, dict):
        failures.append(f"{fixture_id}: fixture.json evidence must be an object")
        return
    _validate_fixture_evidence_file(
        failures,
        fixture_id,
        fixture_dir,
        evidence.get("positive"),
        fixture.get("positive_evidence"),
        fixture.get("execution_kind"),
        "positive",
    )
    _validate_fixture_evidence_file(
        failures,
        fixture_id,
        fixture_dir,
        evidence.get("negative"),
        fixture.get("negative_evidence"),
        fixture.get("execution_kind"),
        "negative",
    )


def _validate_fixture_evidence_file(
    failures: list[str],
    fixture_id: str,
    fixture_dir: Path,
    manifest_evidence: Any,
    matrix_evidence: Any,
    execution_kind: Any,
    side: str,
) -> None:
    if not isinstance(manifest_evidence, dict):
        failures.append(f"{fixture_id}: fixture.json evidence.{side} must be an object")
        return
    if not isinstance(matrix_evidence, dict):
        return
    for field in ("id", "status"):
        if manifest_evidence.get(field) != matrix_evidence.get(field):
            failures.append(f"{fixture_id}: fixture.json evidence.{side}.{field} must match fixture matrix")

    raw_path = manifest_evidence.get("path")
    if not isinstance(raw_path, str) or not raw_path:
        failures.append(f"{fixture_id}: fixture.json evidence.{side}.path is required")
        return
    raw_source_path = Path(raw_path)
    if raw_source_path.is_absolute() or ".." in raw_source_path.parts:
        failures.append(f"{fixture_id}: evidence.{side}.path must stay inside the fixture directory")
        return
    expected_path = Path(side) / f"{matrix_evidence.get('id')}.sifr"
    if raw_source_path != expected_path:
        failures.append(f"{fixture_id}: evidence.{side}.path must be {expected_path.as_posix()}")
        return
    source_path = fixture_dir / raw_path
    try:
        source_path.relative_to(fixture_dir)
    except ValueError:
        failures.append(f"{fixture_id}: evidence.{side}.path must stay inside the fixture directory")
        return
    if source_path.suffix != ".sifr":
        failures.append(f"{fixture_id}: evidence.{side}.path must point to a .sifr file")
    if not source_path.is_file():
        failures.append(f"{fixture_id}: missing evidence source {raw_path}")
        return

    text = source_path.read_text(encoding="utf-8")
    if len(text.strip().splitlines()) < 5:
        failures.append(f"{fixture_id}: {raw_path} must contain a concrete fixture, not an empty stub")
    required_headers = (
        f"# fixture: {fixture_id}",
        f"# evidence: {side}/{matrix_evidence.get('id')}",
        f"# evidence-status: {matrix_evidence.get('status')}",
    )
    for header in required_headers:
        if header not in text:
            failures.append(f"{fixture_id}: {raw_path} missing header {header!r}")
    if not any(line.lstrip().startswith("@rust") for line in text.splitlines()):
        failures.append(f"{fixture_id}: {raw_path} must exercise a Rust interop declaration")

    expected_result = manifest_evidence.get("expected_result")
    if not isinstance(expected_result, str) or not expected_result:
        failures.append(f"{fixture_id}: evidence.{side}.expected_result is required")
        return
    expected_headers = (
        f"# execution-kind: {execution_kind}",
        f"# expected-result: {expected_result}",
    )
    if expected_headers[0] not in text:
        failures.append(f"{fixture_id}: {raw_path} missing execution-kind header")
    if expected_headers[1] not in text:
        failures.append(f"{fixture_id}: {raw_path} missing expected-result header")
    status = matrix_evidence.get("status")
    if expected_result.startswith("future-owned") and status == "passing":
        failures.append(f"{fixture_id}: passing {side} evidence cannot be marked future-owned")
    if status != "passing" and not expected_result.startswith("future-owned"):
        failures.append(f"{fixture_id}: non-passing {side} evidence must be marked future-owned")

    expected_diagnostic = manifest_evidence.get("expected_diagnostic")
    if expected_result in {"diagnostic", "future-owned-diagnostic"}:
        if expected_diagnostic not in REQUIRED_DIAGNOSTICS:
            failures.append(f"{fixture_id}: evidence.{side}.expected_diagnostic must be a reserved SIFR-RUST code")
        elif f"# expected-diagnostic: {expected_diagnostic}" not in text:
            failures.append(f"{fixture_id}: {raw_path} missing expected diagnostic marker {expected_diagnostic}")


if __name__ == "__main__":
    raise SystemExit(main())
