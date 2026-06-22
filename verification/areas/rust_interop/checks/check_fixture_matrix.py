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
    package_example_count = 0
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
        package_example_count += _validate_fixture_files(failures, fixture, crates)

    failures.extend(f"required crate lacks fixture coverage: {crate}" for crate in sorted(REQUIRED_CRATES - covered_crates))

    if failures:
        for failure in failures:
            print(f"rust interop fixture matrix error: {failure}", file=sys.stderr)
        return 1
    print(
        "rust interop fixture matrix ok: "
        f"fixtures={len(fixture_id_set)} diagnostics={len(diagnostics)} "
        f"crates={len(covered_crates)} package_examples={package_example_count}"
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


def _validate_fixture_files(failures: list[str], fixture: dict[str, Any], crates: list[Any]) -> int:
    fixture_id = str(fixture.get("id"))
    fixture_dir = FIXTURES_ROOT / fixture_id
    manifest_path = fixture_dir / "fixture.json"
    if not manifest_path.is_file():
        return 0
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        failures.append(f"{fixture_id}: fixture.json is not valid JSON: {error}")
        return 0
    if not isinstance(manifest, dict):
        failures.append(f"{fixture_id}: fixture.json must be an object")
        return 0

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
        return 0
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
    return _validate_package_examples(
        failures,
        fixture_id,
        fixture_dir,
        manifest.get("package_examples"),
        crates,
        fixture.get("execution_kind"),
    )


def _validate_package_examples(
    failures: list[str],
    fixture_id: str,
    fixture_dir: Path,
    raw_examples: Any,
    crates: list[Any],
    execution_kind: Any,
) -> int:
    expected_crates = {str(crate) for crate in crates}
    if not expected_crates:
        if raw_examples not in ({}, None):
            failures.append(f"{fixture_id}: package_examples must be empty when required_crates is empty")
        return 0
    if not isinstance(raw_examples, dict):
        failures.append(f"{fixture_id}: fixture.json package_examples must cover every required crate")
        return 0

    actual_crates = {str(crate) for crate in raw_examples}
    for crate in sorted(expected_crates - actual_crates):
        failures.append(f"{fixture_id}: missing package example for crate {crate}")
    for crate in sorted(actual_crates - expected_crates):
        failures.append(f"{fixture_id}: unexpected package example for crate {crate}")

    valid_examples = 0
    for crate in sorted(expected_crates & actual_crates):
        raw_path = raw_examples.get(crate)
        if not isinstance(raw_path, str) or not raw_path:
            failures.append(f"{fixture_id}: package_examples.{crate} path is required")
            continue
        raw_source_path = Path(raw_path)
        expected_path = Path("examples") / f"{crate}.sifr"
        if raw_source_path.is_absolute() or ".." in raw_source_path.parts:
            failures.append(f"{fixture_id}: package_examples.{crate} must stay inside the fixture directory")
            continue
        if raw_source_path != expected_path:
            failures.append(f"{fixture_id}: package_examples.{crate} must be {expected_path.as_posix()}")
            continue

        source_path = fixture_dir / raw_source_path
        if not source_path.is_file():
            failures.append(f"{fixture_id}: missing package example source {raw_path}")
            continue
        text = source_path.read_text(encoding="utf-8")
        _validate_package_example_text(failures, fixture_id, raw_path, text, crate, execution_kind)
        valid_examples += 1
    return valid_examples


def _validate_package_example_text(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    text: str,
    crate: str,
    execution_kind: Any,
) -> None:
    if len(text.strip().splitlines()) < 10:
        failures.append(f"{fixture_id}: {raw_path} must contain a full package example")
    required_headers = (
        f"# fixture: {fixture_id}",
        f"# package-example: {crate}",
        f"# required-crate: {crate}",
        f"# execution-kind: {execution_kind}",
        "# expected-result: package-example",
    )
    for header in required_headers:
        if header not in text:
            failures.append(f"{fixture_id}: {raw_path} missing header {header!r}")
    if _contains_empty_pass_body(text):
        failures.append(f"{fixture_id}: {raw_path} must not use empty placeholder class bodies")
    crate_token = crate.replace("-", "_")
    bound_functions = _rust_bound_function_names(text, crate_token)
    if not bound_functions:
        failures.append(f"{fixture_id}: {raw_path} must declare a Rust binding for crate {crate}")
        return

    verifier_marker = f"def verify_{crate_token}_package("
    async_verifier_marker = f"async def verify_{crate_token}_package("
    if verifier_marker not in text and async_verifier_marker not in text:
        failures.append(f"{fixture_id}: {raw_path} must include verify_{crate_token}_package")
        return
    verifier_start = min(
        index for index in (text.find(verifier_marker), text.find(async_verifier_marker)) if index >= 0
    )
    verifier_body = text[verifier_start:]
    for bound_function in bound_functions:
        if f"{bound_function}(" not in verifier_body:
            failures.append(f"{fixture_id}: {raw_path} verifier must call {bound_function}")
        if not _verifier_binds_call(verifier_body, bound_function):
            failures.append(f"{fixture_id}: {raw_path} verifier must bind {bound_function} result before returning")


def _rust_bound_function_names(text: str, crate_token: str) -> list[str]:
    names: list[str] = []
    lines = text.splitlines()
    for index, line in enumerate(lines):
        stripped = line.lstrip()
        binding_prefix = f"@rust({crate_token}"
        if not stripped.startswith(binding_prefix):
            continue
        if not _has_crate_token_boundary(stripped, len(binding_prefix)):
            continue
        for following in lines[index + 1 :]:
            following_stripped = following.lstrip()
            if following_stripped.startswith("@"):
                continue
            name = _decorated_function_name(following_stripped)
            if name is not None and name not in names:
                names.append(name)
            break
    return names


def _verifier_binds_call(verifier_body: str, bound_function: str) -> bool:
    for line in verifier_body.splitlines():
        for before_call in _bound_call_prefixes(line, bound_function):
            if "=" in before_call and not before_call.lstrip().startswith("return "):
                return True
    return False


def _bound_call_prefixes(line: str, bound_function: str) -> list[str]:
    prefixes: list[str] = []
    marker = f"{bound_function}("
    start = 0
    while True:
        index = line.find(marker, start)
        if index < 0:
            break
        if index == 0 or not _is_identifier_or_path_char(line[index - 1]):
            prefixes.append(line[:index])
        start = index + len(marker)
    method_marker = f".{bound_function}("
    start = 0
    while True:
        index = line.find(method_marker, start)
        if index < 0:
            break
        prefixes.append(line[: index + 1])
        start = index + len(method_marker)
    return prefixes


def _is_identifier_or_path_char(char: str) -> bool:
    return char.isalnum() or char in {"_", "."}


def _has_crate_token_boundary(text: str, index: int) -> bool:
    if index >= len(text):
        return True
    next_char = text[index]
    return not (next_char.isalnum() or next_char == "_")


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
    _validate_evidence_example_text(failures, fixture_id, raw_path, text, str(matrix_evidence.get("id")))

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


def _validate_evidence_example_text(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    text: str,
    evidence_id: str,
) -> None:
    if len(text.strip().splitlines()) < 9:
        failures.append(f"{fixture_id}: {raw_path} must include a binding and concrete verifier call site")
    if _contains_empty_pass_body(text):
        failures.append(f"{fixture_id}: {raw_path} must not use empty placeholder class bodies")

    verifier_markers = (
        f"def verify_{evidence_id}(",
        f"async def verify_{evidence_id}(",
    )
    verifier_start = min((text.find(marker) for marker in verifier_markers if marker in text), default=-1)
    if verifier_start < 0:
        failures.append(f"{fixture_id}: {raw_path} must include verify_{evidence_id}")
        return

    verifier_body = text[verifier_start:]
    bound_declarations = _rust_bound_declarations(text)
    if not bound_declarations:
        failures.append(f"{fixture_id}: {raw_path} must include a Rust-decorated binding declaration")
    for name, return_type in bound_declarations:
        if f"{name}(" not in verifier_body and f".{name}(" not in verifier_body:
            failures.append(f"{fixture_id}: {raw_path} verifier must call {name}")
        if return_type != "None" and not _verifier_binds_call(verifier_body, name):
            failures.append(f"{fixture_id}: {raw_path} verifier must bind {name} result before returning")


def _rust_bound_declaration_names(text: str) -> list[str]:
    return [name for name, _return_type in _rust_bound_declarations(text)]


def _rust_bound_declarations(text: str) -> list[tuple[str, str]]:
    names: list[str] = []
    declarations: list[tuple[str, str]] = []
    decorators: list[str] = []
    for line in text.splitlines():
        stripped = line.lstrip()
        if stripped.startswith("@"):
            decorators.append(stripped)
            continue
        if _is_rust_decorated_binding(stripped, decorators):
            name = _decorated_function_name(stripped)
            if name is not None and name not in names:
                names.append(name)
                declarations.append((name, _decorated_function_return_type(stripped)))
        if stripped and not stripped.startswith("@"):
            decorators = []
    return declarations


def _is_rust_decorated_binding(stripped: str, decorators: list[str]) -> bool:
    return (
        any(decorator.startswith("@rust") for decorator in decorators)
        and (stripped.startswith("def ") or stripped.startswith("async def "))
        and stripped.endswith(": ...")
    )


def _decorated_function_name(stripped: str) -> str | None:
    if stripped.startswith("async def "):
        stripped = stripped.removeprefix("async ")
    if not stripped.startswith("def "):
        return None
    return stripped.removeprefix("def ").split("(", maxsplit=1)[0].strip()


def _decorated_function_return_type(stripped: str) -> str:
    return stripped.split("->", maxsplit=1)[1].rsplit(":", maxsplit=1)[0].strip()


def _contains_empty_pass_body(text: str) -> bool:
    return any(line.strip() == "pass" for line in text.splitlines())


if __name__ == "__main__":
    raise SystemExit(main())
