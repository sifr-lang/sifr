"""Exact CLI/tooling ecosystem scenario policy and mutation coverage."""

from __future__ import annotations

import shutil
import tempfile
import tomllib
from pathlib import Path
from typing import Any, Callable

ScenarioValidator = Callable[[list[str], str, Path, dict[str, Any]], int]

EXPECTED_WORKSPACE_DEPENDENCIES = {
    "anyhow": {"version": "=1.0.102", "default-features": True},
    "clap": {"version": "=4.6.1", "default-features": True},
    "tracing": {"version": "=0.1.44", "default-features": True},
    "tracing-subscriber": {
        "version": "=0.3.23",
        "default-features": True,
        "features": ["env-filter"],
    },
}

EXPECTED_ROOT_DEPENDENCIES = {
    "anyhow": {"workspace": True},
    "anyhow_surface": {
        "package": "sifr-anyhow-surface-probe",
        "path": "rust/anyhow_surface",
    },
    "clap": {"workspace": True},
    "sifr_runtime": {"path": "../../../../../../../crates/sifr_runtime"},
    "tracing": {"workspace": True},
    "tracing-subscriber": {"workspace": True},
}


def validate_cli_scenario(
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
    if workspace.get("members") != ["rust/anyhow_surface"]:
        failures.append(
            f"{fixture_id}: {raw_path}/Cargo.toml workspace members must equal "
            "the direct-anyhow surface wrapper"
        )
    if workspace.get("dependencies") != EXPECTED_WORKSPACE_DEPENDENCIES:
        failures.append(
            f"{fixture_id}: {raw_path}/Cargo.toml workspace dependencies must "
            "exact-pin anyhow, clap, tracing, and tracing-subscriber env-filter"
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

    expected_trust = {
        "rust-build-scripts": ["anyhow"],
        "rust-no-panic": ["anyhow_surface.direct_error"],
    }
    if trust != expected_trust:
        failures.append(
            f"{fixture_id}: {raw_path}/sifr.toml trust must equal "
            f"{expected_trust!r}"
        )

    wrapper_path = example_dir / "rust/anyhow_surface/Cargo.toml"
    wrapper = _read_toml(failures, fixture_id, raw_path, wrapper_path)
    expected_wrapper = {
        "package": {
            "name": "sifr-anyhow-surface-probe",
            "version": "0.1.0",
            "edition": "2021",
        },
        "lib": {"path": "src/lib.rs"},
        "dependencies": {"anyhow": {"workspace": True}},
    }
    if wrapper != expected_wrapper:
        failures.append(
            f"{fixture_id}: {raw_path} anyhow surface wrapper must expose the "
            "exact workspace anyhow dependency"
        )

    _require_source_tokens(
        failures,
        fixture_id,
        raw_path,
        example_dir / "src/bridges/cli.rs",
        (
            "clap::Command::new",
            "tracing_subscriber::EnvFilter::try_new",
            'target: "sifr_cli_probe"',
            'target: "sifr_cli_noise"',
            'trace.contains("cli bridge event")',
            '!trace.contains("excluded bridge event")',
            ".context(\"clap parse failed\")",
            "CliErrorBridge",
            "RustPanicErrorBridge",
            "anyhow=1.0.102;adapter=CliError",
        ),
    )
    _require_source_tokens(
        failures,
        fixture_id,
        raw_path,
        example_dir / "rust/anyhow_surface/src/lib.rs",
        (
            "-> anyhow::Error",
            'context("direct anyhow surface")',
        ),
    )


def run_cli_self_test(
    area_root: Path,
    validate_scenarios: ScenarioValidator,
) -> tuple[int, str | None]:
    source = area_root / "fixtures/ecosystem_cli_certification"
    raw_examples = {"cli_feature_package": "examples/cli_feature_package"}
    cases = 0
    with tempfile.TemporaryDirectory(
        prefix="sifr-rust-cli-scenario-self-test-"
    ) as raw_temp:
        fixture_dir = Path(raw_temp) / "ecosystem_cli_certification"
        shutil.copytree(source, fixture_dir, ignore=shutil.ignore_patterns("target"))
        failures: list[str] = []
        validate_scenarios(
            failures,
            "ecosystem_cli_certification",
            fixture_dir,
            raw_examples,
        )
        if failures:
            return cases, f"CLI ecosystem baseline failed: {failures}"
        cases += 1

        mutation_cases = (
            (
                "anyhow pin drift",
                "examples/cli_feature_package/Cargo.toml",
                'version = "=1.0.102"',
                'version = "1.0.102"',
                "workspace dependencies must exact-pin",
            ),
            (
                "clap pin drift",
                "examples/cli_feature_package/Cargo.toml",
                'version = "=4.6.1"',
                'version = "4.6.1"',
                "workspace dependencies must exact-pin",
            ),
            (
                "tracing pin drift",
                "examples/cli_feature_package/Cargo.toml",
                'version = "=0.1.44"',
                'version = "0.1.44"',
                "workspace dependencies must exact-pin",
            ),
            (
                "subscriber pin drift",
                "examples/cli_feature_package/Cargo.toml",
                'version = "=0.3.23"',
                'version = "0.3.23"',
                "workspace dependencies must exact-pin",
            ),
            (
                "env-filter feature drift",
                "examples/cli_feature_package/Cargo.toml",
                'features = ["env-filter"]',
                "features = []",
                "workspace dependencies must exact-pin",
            ),
            (
                "wrapper membership drift",
                "examples/cli_feature_package/Cargo.toml",
                'members = ["rust/anyhow_surface"]',
                "members = []",
                "workspace members must equal",
            ),
            (
                "wrapper path drift",
                "examples/cli_feature_package/Cargo.toml",
                'path = "rust/anyhow_surface"',
                'path = "rust/other_surface"',
                "dependencies must equal",
            ),
            (
                "direct binding policy drift",
                "examples/cli_feature_package/sifr.toml",
                "direct-crate-bindings = true",
                "direct-crate-bindings = false",
                "must enable [rust] direct-crate-bindings",
            ),
            (
                "build trust drift",
                "examples/cli_feature_package/sifr.toml",
                'rust-build-scripts = ["anyhow"]',
                "rust-build-scripts = []",
                "trust must equal",
            ),
            (
                "direct surface trust drift",
                "examples/cli_feature_package/sifr.toml",
                'rust-no-panic = ["anyhow_surface.direct_error"]',
                "rust-no-panic = []",
                "trust must equal",
            ),
            (
                "bridge path drift",
                "examples/cli_feature_package/sifr.toml",
                'bridges = ["src/bridges"]',
                "bridges = []",
                '[rust] bridges = ["src/bridges"]',
            ),
            (
                "clap execution drift",
                "examples/cli_feature_package/src/bridges/cli.rs",
                "clap::Command::new",
                "clap::Command::missing",
                "must contain clap::Command::new",
            ),
            (
                "env-filter execution drift",
                "examples/cli_feature_package/src/bridges/cli.rs",
                "tracing_subscriber::EnvFilter::try_new",
                "tracing_subscriber::EnvFilter::missing",
                "must contain tracing_subscriber::EnvFilter::try_new",
            ),
            (
                "tracing event drift",
                "examples/cli_feature_package/src/bridges/cli.rs",
                'trace.contains("cli bridge event")',
                'trace.contains("other event")',
                'must contain trace.contains("cli bridge event")',
            ),
            (
                "tracing excluded emission drift",
                "examples/cli_feature_package/src/bridges/cli.rs",
                'target: "sifr_cli_noise"',
                'target: "sifr_cli_probe"',
                'must contain target: "sifr_cli_noise"',
            ),
            (
                "tracing exclusion drift",
                "examples/cli_feature_package/src/bridges/cli.rs",
                '!trace.contains("excluded bridge event")',
                'trace.contains("excluded bridge event")',
                'must contain !trace.contains("excluded bridge event")',
            ),
            (
                "anyhow context drift",
                "examples/cli_feature_package/src/bridges/cli.rs",
                '.context("clap parse failed")',
                '.context("other failure")',
                'must contain .context("clap parse failed")',
            ),
            (
                "direct anyhow type drift",
                "examples/cli_feature_package/rust/anyhow_surface/src/lib.rs",
                "-> anyhow::Error",
                "-> String",
                "must contain -> anyhow::Error",
            ),
        )
        for name, relative_path, before, after, expected in mutation_cases:
            path = fixture_dir / relative_path
            original = path.read_text(encoding="utf-8")
            if before not in original:
                return cases, f"{name} self-test setup token is missing"
            path.write_text(original.replace(before, after, 1), encoding="utf-8")
            failures = []
            validate_scenarios(
                failures,
                "ecosystem_cli_certification",
                fixture_dir,
                raw_examples,
            )
            path.write_text(original, encoding="utf-8")
            if not any(expected in failure for failure in failures):
                return cases, f"{name} did not report {expected!r}: {failures}"
            cases += 1
    return cases, None


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
