"""Manifest policy for the hermetic async reqwest scenario."""

from __future__ import annotations

import shutil
import tempfile
from pathlib import Path
from typing import Any, Callable

ScenarioValidator = Callable[
    [list[str], str, Path, dict[str, Any]],
    int,
]


def validate_async_reqwest_scenario(
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
    _require_dependency(
        failures,
        fixture_id,
        raw_path,
        dependencies,
        "reqwest",
        "=0.12.28",
        ["rustls-tls", "json"],
        default_features=False,
    )
    _require_dependency(
        failures,
        fixture_id,
        raw_path,
        dependencies,
        "tokio",
        "=1.52.3",
        ["io-util", "net", "rt", "sync", "time"],
    )
    _require_trust(
        failures,
        fixture_id,
        raw_path,
        trust,
        "rust-no-panic",
        ["bridge.http.request_roundtrip", "bridge.http.runtime_snapshot"],
    )
    _require_trust(
        failures,
        fixture_id,
        raw_path,
        trust,
        "rust-build-scripts",
        ["ring"],
    )
    _require_trust(
        failures,
        fixture_id,
        raw_path,
        trust,
        "native-links",
        ["ring_core_0_17_14_", "ring_core_0_17_14__test"],
    )


def run_async_reqwest_self_test(
    area_root: Path,
    validate_scenarios: ScenarioValidator,
) -> tuple[int, str | None]:
    source = area_root / "fixtures/async_runtime_reqwest"
    raw_examples = {"reqwest_loopback_runtime": "examples/reqwest_loopback_runtime"}
    cases = 0
    with tempfile.TemporaryDirectory(prefix="sifr-rust-async-scenario-self-test-") as raw_temp:
        fixture_dir = Path(raw_temp) / "async_runtime_reqwest"
        shutil.copytree(source, fixture_dir)
        baseline_failures: list[str] = []
        validate_scenarios(
            baseline_failures,
            "async_runtime_reqwest",
            fixture_dir,
            raw_examples,
        )
        if baseline_failures:
            return cases, f"async reqwest baseline failed: {baseline_failures}"
        cases += 1

        mutation_cases = (
            (
                "reqwest pin drift",
                "examples/reqwest_loopback_runtime/Cargo.toml",
                'version = "=0.12.28"',
                'version = "0.12.28"',
                "must pin version =0.12.28",
            ),
            (
                "tokio feature drift",
                "examples/reqwest_loopback_runtime/Cargo.toml",
                '"sync", "time"',
                '"sync"',
                "must declare features",
            ),
            (
                "build-script trust drift",
                "examples/reqwest_loopback_runtime/sifr.toml",
                'rust-build-scripts = ["ring"]',
                "rust-build-scripts = []",
                "trust.rust-build-scripts",
            ),
            (
                "native-link trust drift",
                "examples/reqwest_loopback_runtime/sifr.toml",
                'native-links = ["ring_core_0_17_14_", "ring_core_0_17_14__test"]',
                "native-links = []",
                "trust.native-links",
            ),
            (
                "proxy bypass drift",
                "examples/reqwest_loopback_runtime/src/bridges/http.rs",
                ".no_proxy()",
                ".proxy_defaults()",
                "missing scenario token '.no_proxy()'",
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
                "async_runtime_reqwest",
                fixture_dir,
                raw_examples,
            )
            path.write_text(original, encoding="utf-8")
            if not any(expected in failure for failure in failures):
                return cases, f"{name} did not report {expected!r}: {failures}"
            cases += 1

    return cases, None


def _require_path_dependency(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    dependencies: dict[str, Any],
    name: str,
    path: str,
) -> None:
    entry = dependencies.get(name)
    if not isinstance(entry, dict) or entry.get("path") != path:
        failures.append(
            f"{fixture_id}: {raw_path}/Cargo.toml dependency {name} "
            f"must use path {path!r}"
        )


def _require_dependency(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    dependencies: dict[str, Any],
    name: str,
    version: str,
    features: list[str],
    *,
    default_features: bool | None = None,
) -> None:
    entry = dependencies.get(name)
    if not isinstance(entry, dict) or entry.get("version") != version:
        failures.append(
            f"{fixture_id}: {raw_path}/Cargo.toml dependency {name} "
            f"must pin version {version}"
        )
        return
    if entry.get("features") != features:
        failures.append(
            f"{fixture_id}: {raw_path}/Cargo.toml dependency {name} "
            f"must declare features {features!r}"
        )
    if default_features is not None and entry.get("default-features") is not default_features:
        failures.append(
            f"{fixture_id}: {raw_path}/Cargo.toml dependency {name} "
            f"must set default-features = {str(default_features).lower()}"
        )


def _require_trust(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    trust: dict[str, Any],
    key: str,
    expected: list[str],
) -> None:
    if trust.get(key) != expected:
        failures.append(
            f"{fixture_id}: {raw_path}/sifr.toml trust.{key} "
            f"must equal {expected!r}"
        )
