"""Exact locked/offline Cargo scenario policy and mutation coverage."""

from __future__ import annotations

import shutil
import tempfile
import tomllib
from pathlib import Path
from typing import Any, Callable

ScenarioValidator = Callable[[list[str], str, Path, dict[str, Any]], int]

CARGO_LOCKED_SCENARIO_TOKENS = (
    'indexmap = { version = "=2.14.0", default-features = false }',
    "IndexMap::<String, u32>::new()",
    "--locked",
    "--offline",
    "--frozen",
)

EXPECTED_TRUST_TARGETS = [
    "locked_bridge.cached_hash",
    "locked_bridge.lockfile_generation",
]


def validate_cargo_locked_scenario(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    dependencies: dict[str, Any],
    trust: dict[str, Any],
    example_dir: Path,
) -> None:
    expected_root = {"path": "rust/locked_bridge"}
    if dependencies.get("locked_bridge") != expected_root:
        failures.append(
            f"{fixture_id}: {raw_path}/Cargo.toml dependency locked_bridge "
            f"must equal {expected_root!r}"
        )

    wrapper_path = example_dir / "rust/locked_bridge/Cargo.toml"
    wrapper = _read_toml(failures, fixture_id, raw_path, wrapper_path)
    indexmap = (
        wrapper.get("dependencies", {}).get("indexmap")
        if isinstance(wrapper, dict)
        else None
    )
    expected_indexmap = {"version": "=2.14.0", "default-features": False}
    if indexmap != expected_indexmap:
        failures.append(
            f"{fixture_id}: {raw_path}/rust/locked_bridge/Cargo.toml dependency "
            f"indexmap must equal {expected_indexmap!r}"
        )

    actual_trust = trust.get("rust-no-panic") if isinstance(trust, dict) else None
    if actual_trust != EXPECTED_TRUST_TARGETS:
        failures.append(
            f"{fixture_id}: {raw_path}/sifr.toml [trust].rust-no-panic must "
            f"equal {EXPECTED_TRUST_TARGETS!r}"
        )


def run_cargo_locked_self_test(
    area_root: Path,
    validate: ScenarioValidator,
) -> tuple[int, str | None]:
    source = area_root / "fixtures/cargo_locked_offline"
    raw_examples = {"locked_offline_cache": "examples/locked_offline_cache"}
    cases = 0
    with tempfile.TemporaryDirectory(prefix="sifr-rust-cargo-locked-self-test-") as raw_temp:
        fixture_dir = Path(raw_temp) / "cargo_locked_offline"
        shutil.copytree(source, fixture_dir, ignore=shutil.ignore_patterns("target"))

        failures: list[str] = []
        validate(failures, "cargo_locked_offline", fixture_dir, raw_examples)
        if failures:
            return cases, f"cargo_locked_offline baseline failed: {failures}"
        cases += 1

        mutation_cases = (
            (
                "registry version drift",
                "examples/locked_offline_cache/rust/locked_bridge/Cargo.toml",
                'version = "=2.14.0"',
                'version = "=2.13.0"',
                "indexmap must equal",
            ),
            (
                "default feature drift",
                "examples/locked_offline_cache/rust/locked_bridge/Cargo.toml",
                "default-features = false",
                "default-features = true",
                "indexmap must equal",
            ),
            (
                "registry feature drift",
                "examples/locked_offline_cache/rust/locked_bridge/Cargo.toml",
                "default-features = false }",
                'default-features = false, features = ["serde"] }',
                "indexmap must equal",
            ),
            (
                "lock checksum drift",
                "examples/locked_offline_cache/Cargo.lock",
                "d466e9454f08e4a911e14806c24e16fba1b4c121d1ea474396f396069cf949d9",
                "0466e9454f08e4a911e14806c24e16fba1b4c121d1ea474396f396069cf949d9",
                "checksum",
            ),
            (
                "trust drift",
                "examples/locked_offline_cache/sifr.toml",
                '"locked_bridge.lockfile_generation"',
                '"locked_bridge.other"',
                "rust-no-panic must equal",
            ),
        )
        for name, relative_path, before, after, expected in mutation_cases:
            path = fixture_dir / relative_path
            original = path.read_text(encoding="utf-8")
            if before not in original:
                return cases, f"{name} self-test setup token is missing"
            path.write_text(original.replace(before, after, 1), encoding="utf-8")
            failures = []
            validate(failures, "cargo_locked_offline", fixture_dir, raw_examples)
            path.write_text(original, encoding="utf-8")
            if not any(expected in failure for failure in failures):
                return cases, f"{name} did not report {expected!r}: {failures}"
            cases += 1

    return cases, None


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
