"""Exact proc-macro/codegen scenario policy and mutation coverage."""

from __future__ import annotations

import shutil
import tempfile
import tomllib
from pathlib import Path
from typing import Any, Callable

ScenarioValidator = Callable[[list[str], str, Path, dict[str, Any]], int]

EXPECTED_WORKSPACE_DEPENDENCIES = {
    "prost": {"version": "=0.14.4", "default-features": True},
    "prost_build_upstream": {
        "package": "prost-build",
        "version": "=0.14.4",
        "default-features": True,
    },
    "prost_types": {
        "package": "prost-types",
        "version": "=0.14.4",
        "default-features": True,
    },
    "serde_derive_upstream": {
        "package": "serde_derive",
        "version": "=1.0.228",
        "default-features": True,
    },
}


def validate_proc_macro_scenario(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    cargo: dict[str, Any],
    dependencies: dict[str, Any],
    trust: dict[str, Any],
    example_dir: Path,
) -> None:
    workspace = cargo.get("workspace", {})
    if workspace.get("members") != ["rust/serde_derive", "rust/prost_build"]:
        failures.append(
            f"{fixture_id}: {raw_path}/Cargo.toml workspace members must equal "
            "the proc-macro and prost-build wrappers"
        )
    if workspace.get("dependencies") != EXPECTED_WORKSPACE_DEPENDENCIES:
        failures.append(
            f"{fixture_id}: {raw_path}/Cargo.toml workspace dependencies must "
            "exact-pin serde_derive and prost-build with their generation graph"
        )

    expected_dependencies = {
        "prost-build": {
            "package": "sifr-prost-build-probe",
            "path": "rust/prost_build",
        },
        "serde_derive": {
            "package": "sifr-serde-derive-probe",
            "path": "rust/serde_derive",
        },
    }
    if dependencies != expected_dependencies:
        failures.append(
            f"{fixture_id}: {raw_path}/Cargo.toml dependencies must equal "
            f"{expected_dependencies!r}"
        )

    _validate_serde_wrapper(
        failures,
        fixture_id,
        raw_path,
        example_dir / "rust/serde_derive/Cargo.toml",
    )
    _validate_prost_wrapper(
        failures,
        fixture_id,
        raw_path,
        example_dir / "rust/prost_build/Cargo.toml",
    )
    _require_exact_trust(
        failures,
        fixture_id,
        raw_path,
        trust,
        "rust-proc-macros",
        ["serde_derive"],
    )
    _require_exact_trust(
        failures,
        fixture_id,
        raw_path,
        trust,
        "rust-build-scripts",
        ["prost_build"],
    )
    _require_exact_trust(
        failures,
        fixture_id,
        raw_path,
        trust,
        "rust-no-panic",
        ["bridge.generated.decode"],
    )
    if trust.get("unsafe-rust-bridges"):
        failures.append(
            f"{fixture_id}: {raw_path}/sifr.toml must not grant unsafe bridge trust"
        )
    _validate_sources(failures, fixture_id, raw_path, example_dir)


def run_proc_macro_self_test(
    area_root: Path,
    validate_scenarios: ScenarioValidator,
) -> tuple[int, str | None]:
    source = area_root / "fixtures/proc_macro_trust"
    raw_examples = {"proc_macro_trust_package": "examples/proc_macro_trust_package"}
    cases = 0
    with tempfile.TemporaryDirectory(
        prefix="sifr-rust-proc-macro-scenario-self-test-"
    ) as raw_temp:
        fixture_dir = Path(raw_temp) / "proc_macro_trust"
        shutil.copytree(source, fixture_dir, ignore=shutil.ignore_patterns("target"))
        failures: list[str] = []
        validate_scenarios(
            failures,
            "proc_macro_trust",
            fixture_dir,
            raw_examples,
        )
        if failures:
            return cases, f"proc-macro baseline failed: {failures}"
        cases += 1

        mutation_cases = (
            (
                "serde pin drift",
                "examples/proc_macro_trust_package/Cargo.toml",
                'version = "=1.0.228"',
                'version = "1.0.228"',
                "workspace dependencies must exact-pin",
            ),
            (
                "prost pin drift",
                "examples/proc_macro_trust_package/Cargo.toml",
                'version = "=0.14.4"',
                'version = "0.14.4"',
                "workspace dependencies must exact-pin",
            ),
            (
                "proc-macro package drift",
                "examples/proc_macro_trust_package/rust/serde_derive/Cargo.toml",
                'name = "sifr-serde-derive-probe"',
                'name = "serde_derive"',
                "serde wrapper package must be sifr-serde-derive-probe",
            ),
            (
                "proc-macro target drift",
                "examples/proc_macro_trust_package/rust/serde_derive/Cargo.toml",
                "proc-macro = true",
                "proc-macro = false",
                "serde wrapper must declare proc-macro = true",
            ),
            (
                "prost build target drift",
                "examples/proc_macro_trust_package/rust/prost_build/Cargo.toml",
                'build = "build.rs"',
                'build = "other.rs"',
                "prost wrapper must declare build.rs",
            ),
            (
                "proc-macro trust drift",
                "examples/proc_macro_trust_package/sifr.toml",
                'rust-proc-macros = ["serde_derive"]',
                "rust-proc-macros = []",
                "trust.rust-proc-macros",
            ),
            (
                "build-script trust drift",
                "examples/proc_macro_trust_package/sifr.toml",
                'rust-build-scripts = ["prost_build"]',
                "rust-build-scripts = []",
                "trust.rust-build-scripts",
            ),
            (
                "no-panic trust drift",
                "examples/proc_macro_trust_package/sifr.toml",
                'rust-no-panic = ["bridge.generated.decode"]',
                "rust-no-panic = []",
                "trust.rust-no-panic",
            ),
            (
                "macro sentinel drift",
                "examples/proc_macro_trust_package/rust/serde_derive/src/lib.rs",
                "PROC_MACRO_EXECUTED",
                "OTHER_MACRO_SENTINEL",
                "must contain PROC_MACRO_EXECUTED",
            ),
            (
                "macro version evidence drift",
                "examples/proc_macro_trust_package/rust/serde_derive/src/lib.rs",
                "serde_derive=1.0.228;upstream=compiled;sifr_wrapper_macro=executed",
                "serde_derive=0.0.0;upstream=compiled;sifr_wrapper_macro=executed",
                "must contain two exact serde_derive version markers",
            ),
            (
                "prost execution drift",
                "examples/proc_macro_trust_package/rust/prost_build/build.rs",
                "compile_fds(descriptor)",
                "skip_fds(descriptor)",
                "must contain compile_fds(descriptor)",
            ),
            (
                "prost sentinel drift",
                "examples/proc_macro_trust_package/rust/prost_build/build.rs",
                "BUILD_SCRIPT_EXECUTED",
                "OTHER_BUILD_SENTINEL",
                "must contain BUILD_SCRIPT_EXECUTED",
            ),
            (
                "prost output location drift",
                "examples/proc_macro_trust_package/rust/prost_build/build.rs",
                'std::env::var("OUT_DIR")',
                'std::env::var("CARGO_TARGET_DIR")',
                "must keep generated output under OUT_DIR",
            ),
            (
                "bridge derive drift",
                "examples/proc_macro_trust_package/src/bridges/generated.rs",
                "#[derive(serde_derive::SifrGenerated)]",
                "#[derive(Clone)]",
                "must contain #[derive(serde_derive::SifrGenerated)]",
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
                "proc_macro_trust",
                fixture_dir,
                raw_examples,
            )
            path.write_text(original, encoding="utf-8")
            if not any(expected in failure for failure in failures):
                return cases, f"{name} did not report {expected!r}: {failures}"
            cases += 1
    return cases, None


def _validate_serde_wrapper(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    path: Path,
) -> None:
    manifest = _read_toml(failures, fixture_id, raw_path, path)
    if manifest is None:
        return
    if manifest.get("package", {}).get("name") != "sifr-serde-derive-probe":
        failures.append(
            f"{fixture_id}: {raw_path} serde wrapper package must be "
            "sifr-serde-derive-probe"
        )
    if manifest.get("lib") != {"path": "src/lib.rs", "proc-macro": True}:
        failures.append(
            f"{fixture_id}: {raw_path} serde wrapper must declare proc-macro = true"
        )
    if manifest.get("dependencies") != {
        "serde_derive_upstream": {"workspace": True}
    }:
        failures.append(
            f"{fixture_id}: {raw_path} serde wrapper must compile upstream serde_derive"
        )


def _validate_prost_wrapper(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    path: Path,
) -> None:
    manifest = _read_toml(failures, fixture_id, raw_path, path)
    if manifest is None:
        return
    package = manifest.get("package", {})
    if package.get("name") != "sifr-prost-build-probe":
        failures.append(
            f"{fixture_id}: {raw_path} prost wrapper package must be "
            "sifr-prost-build-probe"
        )
    if package.get("build") != "build.rs":
        failures.append(f"{fixture_id}: {raw_path} prost wrapper must declare build.rs")
    if manifest.get("dependencies") != {"prost": {"workspace": True}}:
        failures.append(
            f"{fixture_id}: {raw_path} prost wrapper must compile generated prost code"
        )
    if manifest.get("build-dependencies") != {
        "prost_build_upstream": {"workspace": True},
        "prost_types": {"workspace": True},
    }:
        failures.append(
            f"{fixture_id}: {raw_path} prost wrapper build dependencies must "
            "compile upstream prost-build and prost-types"
        )


def _validate_sources(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    example_dir: Path,
) -> None:
    requirements = {
        "rust/serde_derive/src/lib.rs": (
            "#[proc_macro_derive(SifrGenerated)]",
            "ARM_PROC_MACRO_SENTINEL",
            "PROC_MACRO_EXECUTED",
            "serde_derive=1.0.228;upstream=compiled;sifr_wrapper_macro=executed",
        ),
        "rust/prost_build/build.rs": (
            "prost_build_upstream::Config::new()",
            "compile_fds(descriptor)",
            'std::env::var("OUT_DIR")',
            "ARM_BUILD_SCRIPT_SENTINEL",
            "BUILD_SCRIPT_EXECUTED",
            "prost-build=0.14.4;message=sifr.probe.Probe",
        ),
        "rust/prost_build/src/lib.rs": (
            'include!(concat!(env!("OUT_DIR"), "/sifr.probe.rs"))',
            "sifr-prost-build-evidence.txt",
        ),
        "src/bridges/generated.rs": (
            "#[derive(serde_derive::SifrGenerated)]",
            "sifr_proc_macro_marker",
            "prost_build::generated_artifact",
        ),
    }
    for relative_path, tokens in requirements.items():
        path = example_dir / relative_path
        try:
            source = path.read_text(encoding="utf-8")
        except OSError as error:
            failures.append(
                f"{fixture_id}: {raw_path}/{relative_path} is unreadable: {error}"
            )
            continue
        for token in tokens:
            if token not in source:
                failures.append(
                    f"{fixture_id}: {raw_path} {relative_path} must contain {token}"
                )
        if (
            relative_path == "rust/serde_derive/src/lib.rs"
            and source.count(
                "serde_derive=1.0.228;upstream=compiled;sifr_wrapper_macro=executed"
            )
            != 2
        ):
            failures.append(
                f"{fixture_id}: {raw_path} {relative_path} must contain two "
                "exact serde_derive version markers"
            )
        if relative_path.endswith("build.rs"):
            if "OUT_DIR" not in source or "CARGO_TARGET_DIR" in source:
                failures.append(
                    f"{fixture_id}: {raw_path} {relative_path} must keep "
                    "generated output under OUT_DIR"
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


def _require_exact_trust(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    trust: Any,
    key: str,
    expected: list[str],
) -> None:
    actual = trust.get(key) if isinstance(trust, dict) else None
    if actual != expected:
        failures.append(
            f"{fixture_id}: {raw_path}/sifr.toml trust.{key} must equal "
            f"{expected!r}"
        )
