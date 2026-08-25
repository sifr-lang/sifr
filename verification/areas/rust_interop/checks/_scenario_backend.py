"""Exact backend ecosystem scenario policy and mutation coverage."""

from __future__ import annotations

import hashlib
import json
import shutil
import tempfile
import tomllib
from pathlib import Path
from typing import Any, Callable

ScenarioValidator = Callable[[list[str], str, Path, dict[str, Any]], int]

QUERY = "SELECT 13::INT4 AS value"
QUERY_HASH = hashlib.sha256(QUERY.encode("utf-8")).hexdigest()
QUERY_FILE = f"query-{QUERY_HASH}.json"

EXPECTED_WORKSPACE_DEPENDENCIES = {
    "axum": {
        "version": "=0.8.9",
        "default-features": False,
        "features": ["http1", "tokio"],
    },
    "sqlx": {
        "version": "=0.9.0",
        "default-features": False,
        "features": [
            "runtime-tokio",
            "tls-rustls-ring-webpki",
            "postgres",
            "macros",
        ],
    },
    "tower-http": {
        "version": "=0.7.0",
        "default-features": False,
        "features": ["set-header"],
    },
    "tokio": {
        "version": "=1.53.1",
        "default-features": False,
        "features": ["io-util", "net", "rt", "sync", "time"],
    },
}

EXPECTED_ROOT_DEPENDENCIES = {
    "axum": {"workspace": True},
    "sifr_runtime": {"path": "../../../../../../../crates/sifr_runtime"},
    "sqlx": {"workspace": True},
    "tokio": {"workspace": True},
    "tower-http": {"workspace": True},
}

EXPECTED_TRUST = {
    "rust-proc-macros": ["sqlx-macros"],
    "rust-build-scripts": ["ring"],
    "native-links": ["ring_core_0_17_14_", "ring_core_0_17_14__test"],
}


def validate_backend_scenario(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    cargo: dict[str, Any],
    dependencies: dict[str, Any],
    rust: dict[str, Any],
    trust: dict[str, Any],
    example_dir: Path,
) -> None:
    workspace = cargo.get("workspace", {})
    if workspace.get("members", []) != []:
        failures.append(
            f"{fixture_id}: {raw_path}/Cargo.toml must not retain shadow workspace members"
        )
    if workspace.get("dependencies") != EXPECTED_WORKSPACE_DEPENDENCIES:
        failures.append(
            f"{fixture_id}: {raw_path}/Cargo.toml workspace dependencies must "
            "exact-pin the backend graph and frozen features"
        )
    if dependencies != EXPECTED_ROOT_DEPENDENCIES:
        failures.append(
            f"{fixture_id}: {raw_path}/Cargo.toml dependencies must equal "
            f"{EXPECTED_ROOT_DEPENDENCIES!r}"
        )
    if rust.get("bridges") != ["src/bridges"]:
        failures.append(
            f"{fixture_id}: {raw_path}/sifr.toml must declare "
            '[rust] bridges = ["src/bridges"]'
        )
    if trust != EXPECTED_TRUST:
        failures.append(
            f"{fixture_id}: {raw_path}/sifr.toml trust must equal {EXPECTED_TRUST!r}"
        )

    _validate_cargo_environment(failures, fixture_id, raw_path, example_dir)
    _validate_query_metadata(failures, fixture_id, raw_path, example_dir)
    _require_source_tokens(
        failures,
        fixture_id,
        raw_path,
        example_dir / "src/bridges/backend.rs",
        (
            'const QUERY: &str = "SELECT 13::INT4 AS value"',
            f'const QUERY_HASH: &str = "{QUERY_HASH}"',
            'TcpListener::bind("127.0.0.1:0")',
            "SetResponseHeaderLayer::if_not_present",
            'HeaderValue::from_static("tower-http-0.7.0")',
            'sqlx::query!("SELECT 13::INT4 AS value")',
            "query.sql() != QUERY",
            "with_graceful_shutdown",
            "timeout(Duration::from_secs(2), server)",
            "shutdown=clean",
        ),
    )


def run_backend_self_test(
    area_root: Path,
    validate_scenarios: ScenarioValidator,
) -> tuple[int, str | None]:
    source = area_root / "fixtures/ecosystem_backend_certification"
    raw_examples = {"backend_feature_package": "examples/backend_feature_package"}
    cases = 0
    with tempfile.TemporaryDirectory(
        prefix="sifr-rust-backend-scenario-self-test-"
    ) as raw_temp:
        fixture_dir = Path(raw_temp) / "ecosystem_backend_certification"
        shutil.copytree(source, fixture_dir, ignore=shutil.ignore_patterns("target"))
        failures: list[str] = []
        validate_scenarios(
            failures,
            "ecosystem_backend_certification",
            fixture_dir,
            raw_examples,
        )
        if failures:
            return cases, f"backend ecosystem baseline failed: {failures}"
        cases += 1

        mutations = (
            ("axum pin", "Cargo.toml", '"=0.8.9"', '"0.8.9"', "exact-pin"),
            ("sqlx pin", "Cargo.toml", '"=0.9.0"', '"0.9.0"', "exact-pin"),
            ("tower pin", "Cargo.toml", '"=0.7.0"', '"0.7.0"', "exact-pin"),
            ("tokio pin", "Cargo.toml", '"=1.53.1"', '"1.53.1"', "exact-pin"),
            (
                "inactive SQLx driver lock identity",
                "Cargo.lock",
                "488e99c397a62007e4229aec669a179816339afc6d2620ca6fa420dbee2e982c",
                "0000000000000000000000000000000000000000000000000000000000000000",
                "no longer contains allowed fixture-only package",
            ),
            (
                "SQLx TLS provider feature",
                "Cargo.toml",
                '"runtime-tokio", "tls-rustls-ring-webpki", "postgres", "macros"',
                '"runtime-tokio", "tls-rustls", "postgres", "macros"',
                "exact-pin",
            ),
            (
                "tower feature",
                "Cargo.toml",
                'features = ["set-header"]',
                "features = []",
                "exact-pin",
            ),
            (
                "shadow member",
                "Cargo.toml",
                'resolver = "3"',
                'members = ["rust/sqlx"]\nresolver = "3"',
                "must not retain shadow",
            ),
            (
                "direct binding",
                "sifr.toml",
                "direct-crate-bindings = true",
                "direct-crate-bindings = false",
                "must enable [rust] direct-crate-bindings",
            ),
            (
                "proc macro trust",
                "sifr.toml",
                'rust-proc-macros = ["sqlx-macros"]',
                "rust-proc-macros = []",
                "trust must equal",
            ),
            (
                "native trust",
                "sifr.toml",
                '"ring_core_0_17_14__test"',
                '"other_link"',
                "trust must equal",
            ),
            (
                "Cargo network",
                ".cargo/config.toml",
                "offline = true",
                "offline = false",
                "must disable Cargo network access",
            ),
            (
                "fixture SQLx environment",
                ".cargo/config.toml",
                "offline = true",
                'offline = true\n\n[env]\nSQLX_OFFLINE = { value = "true", force = true }',
                "must leave SQLx environment policy to Sifr",
            ),
            (
                "loopback bind",
                "src/bridges/backend.rs",
                'TcpListener::bind("127.0.0.1:0")',
                'TcpListener::bind("0.0.0.0:8080")',
                'must contain TcpListener::bind("127.0.0.1:0")',
            ),
            (
                "middleware",
                "src/bridges/backend.rs",
                "SetResponseHeaderLayer::if_not_present",
                "SetResponseHeaderLayer::missing",
                "must contain SetResponseHeaderLayer::if_not_present",
            ),
            (
                "SQLx macro",
                "src/bridges/backend.rs",
                'sqlx::query!("SELECT 13::INT4 AS value")',
                'sqlx::query("SELECT 13::INT4 AS value")',
                'must contain sqlx::query!("SELECT 13::INT4 AS value")',
            ),
            (
                "shutdown",
                "src/bridges/backend.rs",
                "with_graceful_shutdown",
                "without_graceful_shutdown",
                "must contain with_graceful_shutdown",
            ),
        )
        scenario_dir = fixture_dir / "examples/backend_feature_package"
        for name, relative_path, before, after, expected in mutations:
            path = scenario_dir / relative_path
            error = _run_mutation(
                path,
                before,
                after,
                expected,
                fixture_dir,
                raw_examples,
                validate_scenarios,
            )
            if error is not None:
                return cases, f"{name} {error}"
            cases += 1

        query_path = scenario_dir / ".sqlx" / QUERY_FILE
        for name, before, after, expected in (
            ("metadata query", QUERY, "SELECT 12::INT4 AS value", "query must equal"),
            ("metadata hash", QUERY_HASH, "0" * 64, "hash must equal"),
            (
                "metadata describe shape",
                '"describe": {',
                '"describe": null,\n  "ignored_describe": {',
                "metadata describe must be an object",
            ),
        ):
            error = _run_mutation(
                query_path,
                before,
                after,
                expected,
                fixture_dir,
                raw_examples,
                validate_scenarios,
            )
            if error is not None:
                return cases, f"{name} {error}"
            cases += 1
    return cases, None


def _validate_cargo_environment(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    example_dir: Path,
) -> None:
    config = _read_toml(
        failures, fixture_id, raw_path, example_dir / ".cargo/config.toml"
    )
    if config is None:
        return
    if config.get("net") != {"offline": True}:
        failures.append(
            f"{fixture_id}: {raw_path}/.cargo/config.toml must disable Cargo network access"
        )
    if "env" in config:
        failures.append(
            f"{fixture_id}: {raw_path}/.cargo/config.toml must leave SQLx environment policy to Sifr"
        )


def _validate_query_metadata(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    example_dir: Path,
) -> None:
    sqlx_dir = example_dir / ".sqlx"
    files = sorted(sqlx_dir.glob("query-*.json")) if sqlx_dir.is_dir() else []
    if [path.name for path in files] != [QUERY_FILE]:
        failures.append(
            f"{fixture_id}: {raw_path}/.sqlx must contain only {QUERY_FILE}"
        )
        return
    try:
        data = json.loads(files[0].read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        failures.append(
            f"{fixture_id}: {raw_path}/.sqlx/{QUERY_FILE} is invalid: {error}"
        )
        return
    if data.get("query") != QUERY:
        failures.append(f"{fixture_id}: {raw_path}/.sqlx query must equal {QUERY!r}")
    if data.get("hash") != QUERY_HASH:
        failures.append(
            f"{fixture_id}: {raw_path}/.sqlx hash must equal SHA-256 {QUERY_HASH}"
        )
    describe = data.get("describe")
    if not isinstance(describe, dict):
        failures.append(
            f"{fixture_id}: {raw_path}/.sqlx metadata describe must be an object"
        )
        return
    if data.get("db_name") != "PostgreSQL" or describe.get("nullable") != [None]:
        failures.append(
            f"{fixture_id}: {raw_path}/.sqlx metadata must describe the PostgreSQL query"
        )


def _run_mutation(
    path: Path,
    before: str,
    after: str,
    expected: str,
    fixture_dir: Path,
    raw_examples: dict[str, str],
    validate_scenarios: ScenarioValidator,
) -> str | None:
    original = path.read_text(encoding="utf-8")
    if before not in original:
        return "self-test setup token is missing"
    path.write_text(original.replace(before, after, 1), encoding="utf-8")
    failures: list[str] = []
    validate_scenarios(
        failures,
        "ecosystem_backend_certification",
        fixture_dir,
        raw_examples,
    )
    path.write_text(original, encoding="utf-8")
    if not any(expected in failure for failure in failures):
        return f"did not report {expected!r}: {failures}"
    return None


def _require_source_tokens(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    path: Path,
    tokens: tuple[str, ...],
) -> None:
    try:
        source = path.read_text(encoding="utf-8")
    except OSError as error:
        failures.append(f"{fixture_id}: {raw_path}/{path.name} is unreadable: {error}")
        return
    for token in tokens:
        if token not in source:
            failures.append(
                f"{fixture_id}: {raw_path}/{path.name} must contain {token}"
            )


def _read_toml(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    path: Path,
) -> dict[str, Any] | None:
    try:
        return tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        failures.append(f"{fixture_id}: {raw_path}/{path.name} is invalid: {error}")
        return None
