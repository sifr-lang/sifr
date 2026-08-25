"""Manifest and harness policy for the opaque resource runtime scenario."""

from __future__ import annotations

import shutil
import tempfile
from pathlib import Path
from typing import Any, Callable

from _scenario_async_reqwest import (
    _require_dependency,
    _require_path_dependency,
    _require_trust,
)

ScenarioValidator = Callable[
    [list[str], str, Path, dict[str, Any]],
    int,
]

OPAQUE_RESOURCE_SCENARIO_TOKENS = (
    "resource_contract",
    "reqwest::Client",
    ".no_proxy()",
    "Connection::open",
    'savepoint_with_name("sifr; DROP TABLE evidence; --")',
    "redis::Client",
    "tokio_postgres::Config",
    'TcpListener::bind(("127.0.0.1", 0))',
    "OPERATION_TIMEOUT",
    "ACTIVE_TASKS.load(Ordering::SeqCst) != 0",
    "bridge.resources.aclose",
    "bridge.resources.close_observation",
    "bridge.resources.invalid_aliasing",
    "close=async_close,\n    borrow=exclusive",
    "impl Drop for TemporaryDatabase",
    "impl Drop for TrackedTask",
    "let activity = TaskActivity::new();",
    ".set_skip_set_lib_name()",
    "serve_redis_malformed",
    "PostgreSQL early-close shutdown",
    "catch_unwind_silently",
    "PoisonOnPanic::new(",
    "Rust bridge panicked",
)


def validate_opaque_resource_scenario(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    rust: dict[str, Any],
    dependencies: dict[str, Any],
    trust: dict[str, Any],
) -> None:
    if rust.get("bridges") != ["src/bridges"]:
        failures.append(
            f"{fixture_id}: {raw_path}/sifr.toml must declare "
            '[rust] bridges = ["src/bridges"]'
        )
    _require_path_dependency(
        failures,
        fixture_id,
        raw_path,
        dependencies,
        "sifr_runtime",
        "../../../../../../../crates/sifr_runtime",
    )
    for name, version, features, default_features in (
        ("redis", "=1.4.1", ["tokio-comp"], False),
        ("reqwest", "=0.13.4", ["rustls", "json"], False),
        ("rusqlite", "=0.40.2", ["bundled"], False),
        ("tokio", "=1.53.1", ["io-util", "net", "rt", "sync", "time"], None),
        ("tokio-postgres", "=0.7.18", ["runtime"], False),
    ):
        _require_dependency(
            failures,
            fixture_id,
            raw_path,
            dependencies,
            name,
            version,
            features,
            default_features=default_features,
        )
    _require_trust(
        failures,
        fixture_id,
        raw_path,
        trust,
        "rust-no-panic",
        [
            "bridge.resources.aclose",
            "bridge.resources.close_observation",
            "bridge.resources.contract",
            "bridge.resources.invalid_aliasing",
            "bridge.resources.open",
            "bridge.resources.run",
        ],
    )
    _require_trust(
        failures,
        fixture_id,
        raw_path,
        trust,
        "rust-build-scripts",
        ["aws-lc-sys", "libsqlite3-sys"],
    )
    _require_trust(
        failures,
        fixture_id,
        raw_path,
        trust,
        "native-links",
        ["aws_lc_0_44_0_crypto", "sqlite3"],
    )


def run_opaque_resource_self_test(
    area_root: Path,
    validate_scenarios: ScenarioValidator,
) -> tuple[int, str | None]:
    source = area_root / "fixtures/opaque_resource_matrix"
    raw_examples = {
        "resource_lifecycle_runtime": "examples/resource_lifecycle_runtime"
    }
    cases = 0
    with tempfile.TemporaryDirectory(
        prefix="sifr-rust-resource-scenario-self-test-"
    ) as raw_temp:
        fixture_dir = Path(raw_temp) / "opaque_resource_matrix"
        shutil.copytree(
            source,
            fixture_dir,
            ignore=shutil.ignore_patterns("target"),
        )
        baseline_failures: list[str] = []
        validate_scenarios(
            baseline_failures,
            "opaque_resource_matrix",
            fixture_dir,
            raw_examples,
        )
        if baseline_failures:
            return cases, f"opaque resource baseline failed: {baseline_failures}"
        cases += 1

        mutation_cases = (
            (
                "rusqlite pin drift",
                "examples/resource_lifecycle_runtime/Cargo.toml",
                'version = "=0.40.2"',
                'version = "0.40.2"',
                "must pin version =0.40.2",
            ),
            (
                "Rusqlite default feature drift",
                "examples/resource_lifecycle_runtime/Cargo.toml",
                'rusqlite = { version = "=0.40.2", default-features = false, features = ["bundled"] }',
                'rusqlite = { version = "=0.40.2", default-features = true, features = ["bundled"] }',
                "rusqlite must set default-features = false",
            ),
            (
                "SQLite savepoint identifier safety drift",
                "examples/resource_lifecycle_runtime/src/bridges/resources.rs",
                'savepoint_with_name("sifr; DROP TABLE evidence; --")',
                'savepoint_with_name("sifr")',
                "missing scenario token",
            ),
            (
                "Redis feature drift",
                "examples/resource_lifecycle_runtime/Cargo.toml",
                'features = ["tokio-comp"]',
                "features = []",
                "must declare features",
            ),
            (
                "SQLite build trust drift",
                "examples/resource_lifecycle_runtime/sifr.toml",
                'rust-build-scripts = ["aws-lc-sys", "libsqlite3-sys"]',
                'rust-build-scripts = ["aws-lc-sys"]',
                "trust.rust-build-scripts",
            ),
            (
                "SQLite native link drift",
                "examples/resource_lifecycle_runtime/sifr.toml",
                '"aws_lc_0_44_0_crypto", "sqlite3"',
                '"aws_lc_0_44_0_crypto"',
                "trust.native-links",
            ),
            (
                "opaque async close policy drift",
                "examples/resource_lifecycle_runtime/src/main.sifr",
                "close=async_close",
                "close=drop",
                "missing scenario token 'close=async_close,\\n    borrow=exclusive'",
            ),
            (
                "proxy bypass drift",
                "examples/resource_lifecycle_runtime/src/bridges/resources.rs",
                ".no_proxy()",
                ".proxy_defaults()",
                "missing scenario token '.no_proxy()'",
            ),
            (
                "Redis metadata handshake drift",
                "examples/resource_lifecycle_runtime/src/bridges/resources.rs",
                ".set_skip_set_lib_name()",
                ".set_default_set_lib_name()",
                "missing scenario token '.set_skip_set_lib_name()'",
            ),
            (
                "task cleanup drift",
                "examples/resource_lifecycle_runtime/src/bridges/resources.rs",
                "ACTIVE_TASKS.load(Ordering::SeqCst) != 0",
                "ACTIVE_TASKS.load(Ordering::SeqCst) == 0",
                "missing scenario token 'ACTIVE_TASKS.load(Ordering::SeqCst) != 0'",
            ),
            (
                "temporary database RAII drift",
                "examples/resource_lifecycle_runtime/src/bridges/resources.rs",
                "impl Drop for TemporaryDatabase",
                "impl TemporaryDatabase",
                "missing scenario token 'impl Drop for TemporaryDatabase'",
            ),
            (
                "task abort RAII drift",
                "examples/resource_lifecycle_runtime/src/bridges/resources.rs",
                "impl Drop for TrackedTask",
                "impl TrackedTaskDrop",
                "missing scenario token 'impl Drop for TrackedTask'",
            ),
            (
                "pre-spawn task accounting drift",
                "examples/resource_lifecycle_runtime/src/bridges/resources.rs",
                "let activity = TaskActivity::new();",
                "let activity = TaskActivity;",
                "missing scenario token 'let activity = TaskActivity::new();'",
            ),
            (
                "poison guard drift",
                "examples/resource_lifecycle_runtime/src/bridges/resources.rs",
                "PoisonOnPanic::new(",
                "manual_poison(",
                "missing scenario token 'PoisonOnPanic::new('",
            ),
            (
                "protocol negative drift",
                "examples/resource_lifecycle_runtime/src/bridges/protocols.rs",
                "PostgreSQL early-close shutdown",
                "PostgreSQL unbounded shutdown",
                "missing scenario token 'PostgreSQL early-close shutdown'",
            ),
        )
        for name, relative_path, before, after, expected in mutation_cases:
            path = fixture_dir / relative_path
            original = path.read_text(encoding="utf-8")
            if before not in original:
                return cases, f"{name} self-test setup token is missing"
            path.write_text(original.replace(before, after, 1), encoding="utf-8")
            failures: list[str] = []
            validate_scenarios(
                failures,
                "opaque_resource_matrix",
                fixture_dir,
                raw_examples,
            )
            path.write_text(original, encoding="utf-8")
            if not any(expected in failure for failure in failures):
                return cases, f"{name} did not report {expected!r}: {failures}"
            cases += 1

    return cases, None
