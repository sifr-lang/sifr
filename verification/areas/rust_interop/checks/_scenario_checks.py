"""Scenario-package validation for the Rust interop fixture matrix."""

from __future__ import annotations

import shutil
import tempfile
import tomllib
from pathlib import Path
from typing import Any

from _scenario_async_reqwest import (
    run_async_reqwest_self_test,
    validate_async_reqwest_scenario,
)
from _scenario_advanced_data import (
    run_advanced_data_self_test,
    validate_advanced_data_scenario,
)
from _scenario_callback_subscriptions import (
    run_callback_subscription_self_test,
    validate_callback_subscription_scenario,
)
from _scenario_backend import run_backend_self_test, validate_backend_scenario
from _scenario_cli import run_cli_self_test, validate_cli_scenario
from _scenario_cargo_locked import (
    run_cargo_locked_self_test,
    validate_cargo_locked_scenario,
)
from _scenario_lock_checks import read_root_lock, require_root_lock_subset
from _scenario_native_build import (
    run_native_build_self_test,
    validate_native_build_scenario,
)
from _scenario_opaque_resources import (
    run_opaque_resource_self_test,
    validate_opaque_resource_scenario,
)
from _scenario_proc_macro import (
    run_proc_macro_self_test,
    validate_proc_macro_scenario,
)
from _scenario_registry import REQUIRED_SCENARIO_EXAMPLES
from _scenario_source_checks import (
    read_scenario_text as _read_scenario_text,
    reject_generated_bridge_imports as _reject_generated_bridge_imports,
    validate_scenario_sifr_source as _validate_scenario_sifr_source,
)
from _scenario_zero_copy import (
    reject_unsafe_rust,
    run_zero_copy_self_test,
    validate_zero_copy_scenario,
)

AREA_ROOT = Path(__file__).resolve().parents[1]
REPO_ROOT = AREA_ROOT.parents[2]


def validate_scenario_examples(
    failures: list[str],
    fixture_id: str,
    fixture_dir: Path,
    raw_examples: Any,
) -> int:
    expected = REQUIRED_SCENARIO_EXAMPLES.get(fixture_id, {})
    if not expected:
        if raw_examples not in ({}, None):
            failures.append(f"{fixture_id}: scenario_examples are not expected for this fixture")
        return 0
    if not isinstance(raw_examples, dict):
        failures.append(f"{fixture_id}: fixture.json scenario_examples must cover every required scenario")
        return 0

    actual_examples = {str(example) for example in raw_examples}
    for example in sorted(set(expected) - actual_examples):
        failures.append(f"{fixture_id}: missing scenario example {example}")
    for example in sorted(actual_examples - set(expected)):
        failures.append(f"{fixture_id}: unexpected scenario example {example}")

    valid_examples = 0
    for example in sorted(set(expected) & actual_examples):
        raw_path = raw_examples.get(example)
        if not isinstance(raw_path, str) or not raw_path:
            failures.append(f"{fixture_id}: scenario_examples.{example} path is required")
            continue
        raw_example_path = Path(raw_path)
        expected_path = Path("examples") / example
        if raw_example_path.is_absolute() or ".." in raw_example_path.parts:
            failures.append(f"{fixture_id}: scenario_examples.{example} must stay inside the fixture directory")
            continue
        if raw_example_path != expected_path:
            failures.append(f"{fixture_id}: scenario_examples.{example} must be {expected_path.as_posix()}")
            continue

        example_dir = fixture_dir / raw_example_path
        if not example_dir.is_dir():
            failures.append(f"{fixture_id}: missing scenario example directory {raw_path}")
            continue
        _validate_scenario_example_dir(
            failures,
            fixture_id,
            example,
            raw_path,
            example_dir,
            tuple(str(token) for token in expected[example].get("tokens", ())),
        )
        valid_examples += 1
    _validate_negative_overlays(failures, fixture_id, fixture_dir)
    return valid_examples


def run_self_test() -> tuple[int, str | None]:
    source = AREA_ROOT / "fixtures/bridge_type_matrix"
    raw_examples = {"bridge_type_roundtrip": "examples/bridge_type_roundtrip"}
    cases = 0
    with tempfile.TemporaryDirectory(prefix="sifr-rust-interop-scenario-self-test-") as raw_temp:
        fixture_dir = Path(raw_temp) / "bridge_type_matrix"
        shutil.copytree(
            source,
            fixture_dir,
            ignore=shutil.ignore_patterns("target"),
        )

        baseline_failures: list[str] = []
        validate_scenario_examples(
            baseline_failures,
            "bridge_type_matrix",
            fixture_dir,
            raw_examples,
        )
        if baseline_failures:
            return cases, f"bridge_type_matrix baseline failed: {baseline_failures}"
        cases += 1

        mutation_cases = (
            (
                "dependency pin drift",
                "examples/bridge_type_roundtrip/Cargo.toml",
                '=1.11.1"',
                '1.11.1"',
                "must pin =1.11.1",
            ),
            (
                "serde feature drift",
                "examples/bridge_type_roundtrip/Cargo.toml",
                'features = ["derive"]',
                "features = []",
                "dependency serde features",
            ),
            (
                "bridge path drift",
                "examples/bridge_type_roundtrip/sifr.toml",
                'bridges = ["src/bridges"]',
                "bridges = []",
                '[rust] bridges = ["src/bridges"]',
            ),
            (
                "bridge trust drift",
                "examples/bridge_type_roundtrip/sifr.toml",
                'unsafe-rust-bridges = ["src/bridges/types.rs"]',
                "unsafe-rust-bridges = []",
                "[trust].unsafe-rust-bridges missing src/bridges/types.rs",
            ),
            (
                "root lock drift",
                "examples/bridge_type_roundtrip/Cargo.lock",
                'version = "2.8.0"',
                'version = "2.8.3"',
                "not present in root Cargo.lock",
            ),
        )
        for name, relative_path, before, after, expected in mutation_cases:
            path = fixture_dir / relative_path
            original = path.read_text(encoding="utf-8")
            if before not in original:
                return cases, f"{name} self-test setup token is missing"
            path.write_text(original.replace(before, after, 1), encoding="utf-8")
            failures: list[str] = []
            validate_scenario_examples(
                failures,
                "bridge_type_matrix",
                fixture_dir,
                raw_examples,
            )
            path.write_text(original, encoding="utf-8")
            if not any(expected in failure for failure in failures):
                return cases, f"{name} did not report {expected!r}: {failures}"
            cases += 1

        lock_path = fixture_dir / "examples/bridge_type_roundtrip/Cargo.lock"
        lock_contents = lock_path.read_bytes()
        lock_path.unlink()
        lock_failures: list[str] = []
        validate_scenario_examples(
            lock_failures,
            "bridge_type_matrix",
            fixture_dir,
            raw_examples,
        )
        lock_path.write_bytes(lock_contents)
        if not any("Cargo.lock is required" in failure for failure in lock_failures):
            return cases, f"missing lockfile was accepted: {lock_failures}"
        cases += 1

    source = AREA_ROOT / "fixtures/panic_boundary_wrapper_emission"
    raw_examples = {"panic_wrapper_runtime": "examples/panic_wrapper_runtime"}
    with tempfile.TemporaryDirectory(prefix="sifr-rust-panic-scenario-self-test-") as raw_temp:
        fixture_dir = Path(raw_temp) / "panic_boundary_wrapper_emission"
        shutil.copytree(
            source,
            fixture_dir,
            ignore=shutil.ignore_patterns("target"),
        )

        baseline_failures = []
        validate_scenario_examples(
            baseline_failures,
            "panic_boundary_wrapper_emission",
            fixture_dir,
            raw_examples,
        )
        if baseline_failures:
            return cases, f"panic wrapper baseline failed: {baseline_failures}"
        cases += 1

        mutation_cases = (
            (
                "panic wrapper unnecessary trust",
                "examples/panic_wrapper_runtime/sifr.toml",
                'bridges = ["src/bridges"]',
                'bridges = ["src/bridges"]\n\n[trust]\nunsafe-rust-bridges = ["src/bridges/wrapper.rs"]',
                "must not grant unsafe-rust-bridges",
            ),
            (
                "panic wrapper runtime path drift",
                "examples/panic_wrapper_runtime/Cargo.toml",
                'path = "../../../../../../../crates/sifr_runtime"',
                'path = "../missing-runtime"',
                "must declare sifr_runtime path",
            ),
        )
        for name, relative_path, before, after, expected in mutation_cases:
            path = fixture_dir / relative_path
            original = path.read_text(encoding="utf-8")
            if before not in original:
                return cases, f"{name} self-test setup token is missing"
            path.write_text(original.replace(before, after, 1), encoding="utf-8")
            failures = []
            validate_scenario_examples(
                failures,
                "panic_boundary_wrapper_emission",
                fixture_dir,
                raw_examples,
            )
            path.write_text(original, encoding="utf-8")
            if not any(expected in failure for failure in failures):
                return cases, f"{name} did not report {expected!r}: {failures}"
            cases += 1

    async_cases, async_error = run_async_reqwest_self_test(
        AREA_ROOT, validate_scenario_examples
    )
    cases += async_cases
    if async_error is not None:
        return cases, async_error

    resource_cases, resource_error = run_opaque_resource_self_test(
        AREA_ROOT, validate_scenario_examples
    )
    cases += resource_cases
    if resource_error is not None:
        return cases, resource_error

    callback_cases, callback_error = run_callback_subscription_self_test(
        AREA_ROOT, validate_scenario_examples
    )
    cases += callback_cases
    if callback_error is not None:
        return cases, callback_error

    zero_copy_cases, zero_copy_error = run_zero_copy_self_test(
        AREA_ROOT, validate_scenario_examples
    )
    cases += zero_copy_cases
    if zero_copy_error is not None:
        return cases, zero_copy_error

    advanced_data_cases, advanced_data_error = run_advanced_data_self_test(
        AREA_ROOT, validate_scenario_examples
    )
    cases += advanced_data_cases
    if advanced_data_error is not None:
        return cases, advanced_data_error

    native_build_cases, native_build_error = run_native_build_self_test(
        AREA_ROOT, validate_scenario_examples
    )
    cases += native_build_cases
    if native_build_error is not None:
        return cases, native_build_error

    proc_macro_cases, proc_macro_error = run_proc_macro_self_test(
        AREA_ROOT, validate_scenario_examples
    )
    cases += proc_macro_cases
    if proc_macro_error is not None:
        return cases, proc_macro_error

    cargo_locked_cases, cargo_locked_error = run_cargo_locked_self_test(
        AREA_ROOT, validate_scenario_examples
    )
    cases += cargo_locked_cases
    if cargo_locked_error is not None:
        return cases, cargo_locked_error

    cli_cases, cli_error = run_cli_self_test(AREA_ROOT, validate_scenario_examples)
    cases += cli_cases
    if cli_error is not None:
        return cases, cli_error

    backend_cases, backend_error = run_backend_self_test(
        AREA_ROOT, validate_scenario_examples
    )
    cases += backend_cases
    if backend_error is not None:
        return cases, backend_error

    return cases, None


def _validate_scenario_example_dir(
    failures: list[str],
    fixture_id: str,
    example: str,
    raw_path: str,
    example_dir: Path,
    required_tokens: tuple[str, ...],
) -> None:
    readme_path = example_dir / "README.md"
    sifr_config_path = example_dir / "sifr.toml"
    cargo_manifest_path = example_dir / "Cargo.toml"
    if not readme_path.is_file():
        failures.append(f"{fixture_id}: {raw_path}/README.md is required")
    if not sifr_config_path.is_file():
        failures.append(f"{fixture_id}: {raw_path}/sifr.toml is required")
    if not cargo_manifest_path.is_file():
        failures.append(f"{fixture_id}: {raw_path}/Cargo.toml is required")

    sifr_sources = _scenario_files(example_dir, "*.sifr")
    cargo_manifests = _scenario_files(example_dir, "Cargo.toml")
    rust_sources = _scenario_files(example_dir, "*.rs")
    if not sifr_sources:
        failures.append(f"{fixture_id}: {raw_path} must include a Sifr source file")
    if not cargo_manifests:
        failures.append(f"{fixture_id}: {raw_path} must include a Cargo.toml")
    if not rust_sources:
        failures.append(f"{fixture_id}: {raw_path} must include Rust source")

    _validate_scenario_manifests(failures, fixture_id, raw_path, example_dir)
    combined_text = _read_scenario_text(readme_path, sifr_config_path, sifr_sources, cargo_manifests, rust_sources)
    for header in (f"# fixture: {fixture_id}", f"# scenario-example: {example}"):
        if header not in combined_text:
            failures.append(f"{fixture_id}: {raw_path} missing header {header!r}")
    for token in required_tokens:
        if token not in combined_text:
            failures.append(f"{fixture_id}: {raw_path} missing scenario token {token!r}")
    if fixture_id == "shared_bridge_crate":
        _reject_generated_bridge_imports(failures, fixture_id, raw_path, rust_sources)
    if fixture_id in {
        "advanced_data_runtime_matrix",
        "ecosystem_backend_certification",
        "ecosystem_cli_certification",
        "native_build_script",
        "proc_macro_trust",
        "zero_copy_runtime_matrix",
    }:
        reject_unsafe_rust(failures, fixture_id, rust_sources, example_dir)
    for source in sifr_sources:
        text = source.read_text(encoding="utf-8")
        raw_source_path = source.relative_to(example_dir).as_posix()
        _validate_scenario_sifr_source(failures, fixture_id, example, f"{raw_path}/{raw_source_path}", text)


def _scenario_files(example_dir: Path, pattern: str) -> list[Path]:
    return sorted(
        path
        for path in example_dir.rglob(pattern)
        if "target" not in path.relative_to(example_dir).parts
    )


def _validate_scenario_manifests(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    example_dir: Path,
) -> None:
    sifr = _read_toml(failures, fixture_id, raw_path, example_dir / "sifr.toml")
    cargo = _read_toml(failures, fixture_id, raw_path, example_dir / "Cargo.toml")
    if not isinstance(sifr, dict) or not isinstance(cargo, dict):
        return

    package = sifr.get("package", {})
    rust = sifr.get("rust", {})
    trust = sifr.get("trust", {})
    if not isinstance(package, dict) or not package.get("name"):
        failures.append(f"{fixture_id}: {raw_path}/sifr.toml must declare [package] name")
    if not isinstance(package, dict) or not package.get("edition"):
        failures.append(f"{fixture_id}: {raw_path}/sifr.toml must declare [package] edition")
    sifr_version = package.get("sifr-version") if isinstance(package, dict) else None
    if not isinstance(sifr_version, str) or ("0.3" not in sifr_version and sifr_version != "*"):
        failures.append(f"{fixture_id}: {raw_path}/sifr.toml must declare package.sifr-version for 0.3")
    if not isinstance(rust, dict) or rust.get("bridge-version") != 1:
        failures.append(f"{fixture_id}: {raw_path}/sifr.toml must declare [rust] bridge-version = 1")
    if not isinstance(rust, dict) or rust.get("direct-crate-bindings") is not True:
        failures.append(f"{fixture_id}: {raw_path}/sifr.toml must enable [rust] direct-crate-bindings")
    if cargo.get("package", {}).get("metadata", {}).get("sifr", {}).get("manifest") != "sifr.toml":
        failures.append(f"{fixture_id}: {raw_path}/Cargo.toml must declare package.metadata.sifr.manifest")

    dependencies = cargo.get("dependencies", {})
    workspace_members = cargo.get("workspace", {}).get("members", [])
    scenario_lock_path = example_dir / "Cargo.lock"
    if not scenario_lock_path.is_file():
        failures.append(f"{fixture_id}: {raw_path}/Cargo.lock is required")
    scenario_lock = _read_toml(
        failures,
        fixture_id,
        raw_path,
        scenario_lock_path,
    )
    root_lock = read_root_lock(failures, fixture_id, REPO_ROOT / "Cargo.lock")
    if isinstance(scenario_lock, dict) and isinstance(root_lock, dict):
        require_root_lock_subset(
            failures,
            fixture_id,
            raw_path,
            scenario_lock,
            root_lock,
        )
    if fixture_id == "bridge_type_matrix":
        if rust.get("bridges") != ["src/bridges"]:
            failures.append(
                f"{fixture_id}: {raw_path}/sifr.toml must declare "
                '[rust] bridges = ["src/bridges"]'
            )
        for dependency, version in (
            ("bytes", "=1.11.1"),
            ("indexmap", "=2.14.0"),
            ("serde", "=1.0.228"),
            ("serde_json", "=1.0.149"),
            ("thiserror", "=2.0.18"),
        ):
            _require_exact_dependency(
                failures,
                fixture_id,
                raw_path,
                dependencies,
                dependency,
                version,
            )
        _require_dependency_features(
            failures,
            fixture_id,
            raw_path,
            dependencies,
            "serde",
            ["derive"],
        )
        _require_trust_targets(
            failures,
            fixture_id,
            raw_path,
            trust,
            "unsafe-rust-bridges",
            ["src/bridges/types.rs"],
        )
    elif fixture_id == "panic_boundary_wrapper_emission":
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
        unsafe_bridges = trust.get("unsafe-rust-bridges", []) if isinstance(trust, dict) else []
        if unsafe_bridges:
            failures.append(
                f"{fixture_id}: {raw_path}/sifr.toml must not grant "
                "unsafe-rust-bridges for the safe wrapper scenario"
            )
    elif fixture_id == "async_runtime_reqwest":
        validate_async_reqwest_scenario(
            failures, fixture_id, raw_path, rust, dependencies, trust
        )
    elif fixture_id == "opaque_resource_matrix":
        validate_opaque_resource_scenario(
            failures, fixture_id, raw_path, rust, dependencies, trust
        )
    elif fixture_id == "callback_subscription_ecosystem":
        validate_callback_subscription_scenario(
            failures, fixture_id, raw_path, rust, dependencies, trust
        )
    elif fixture_id == "zero_copy_runtime_matrix":
        validate_zero_copy_scenario(
            failures, fixture_id, raw_path, rust, dependencies, trust
        )
    elif fixture_id == "advanced_data_runtime_matrix":
        validate_advanced_data_scenario(
            failures,
            fixture_id,
            raw_path,
            cargo,
            dependencies,
            trust,
            example_dir,
        )
    elif fixture_id == "same_workspace_crate":
        _require_path_dependency(failures, fixture_id, raw_path, dependencies, "workspace_hash", "rust/workspace_hash")
        _require_member(failures, fixture_id, raw_path, workspace_members, "rust/workspace_hash")
        _require_trust_targets(failures, fixture_id, raw_path, trust, "rust-no-panic", ["workspace_hash.hash", "workspace_hash.hash_pair"])
    elif fixture_id == "shared_bridge_crate":
        _require_path_dependency(failures, fixture_id, raw_path, dependencies, "sifr_shared_hash_bridge", "rust/sifr_shared_hash_bridge")
        _require_trust_targets(failures, fixture_id, raw_path, trust, "rust-no-panic", ["sifr_shared_hash_bridge.digest", "sifr_shared_hash_bridge.digest_hex"])
    elif fixture_id == "cargo_locked_offline":
        validate_cargo_locked_scenario(
            failures,
            fixture_id,
            raw_path,
            dependencies,
            trust,
            example_dir,
        )
    elif fixture_id == "bridge_version_mismatch":
        _require_path_dependency(failures, fixture_id, raw_path, dependencies, "version_bridge", "rust/version_bridge")
        _require_trust_targets(failures, fixture_id, raw_path, trust, "rust-no-panic", ["version_bridge.accepted", "version_bridge.schema"])
    elif fixture_id == "panic_abort_profile":
        _require_path_dependency(failures, fixture_id, raw_path, dependencies, "legacy_backend", "rust/legacy_backend")
        if cargo.get("profile", {}).get("release", {}).get("panic") != "abort":
            failures.append(f"{fixture_id}: {raw_path}/Cargo.toml must declare [profile.release] panic = \"abort\"")
        _require_trust_targets(failures, fixture_id, raw_path, trust, "rust-panic-abort", ["legacy_backend.run", "legacy_backend.run_checked"])
    elif fixture_id == "local_bridge_blake3":
        if rust.get("bridges") != ["src/bridges"]:
            failures.append(f"{fixture_id}: {raw_path}/sifr.toml must declare [rust] bridges = [\"src/bridges\"]")
        _require_path_dependency(failures, fixture_id, raw_path, dependencies, "blake3", "rust/blake3_backend")
        _require_trust_targets(failures, fixture_id, raw_path, trust, "unsafe-rust-bridges", ["src/bridges/blake3.rs"])
    elif fixture_id == "proc_macro_trust":
        validate_proc_macro_scenario(
            failures,
            fixture_id,
            raw_path,
            cargo,
            dependencies,
            trust,
            example_dir,
        )
    elif fixture_id == "native_build_script":
        validate_native_build_scenario(
            failures,
            fixture_id,
            raw_path,
            cargo,
            dependencies,
            trust,
            example_dir,
        )
    elif fixture_id == "ecosystem_backend_certification":
        validate_backend_scenario(
            failures,
            fixture_id,
            raw_path,
            cargo,
            dependencies,
            rust,
            trust,
            example_dir,
        )
    elif fixture_id == "ecosystem_cli_certification":
        validate_cli_scenario(
            failures,
            fixture_id,
            raw_path,
            cargo,
            dependencies,
            rust,
            trust,
            example_dir,
        )


def _validate_negative_overlays(
    failures: list[str],
    fixture_id: str,
    fixture_dir: Path,
) -> None:
    if fixture_id == "same_workspace_crate":
        _validate_same_workspace_negative_overlay(failures, fixture_id, fixture_dir)
    elif fixture_id == "shared_bridge_crate":
        _validate_shared_bridge_negative_overlay(failures, fixture_id, fixture_dir)


def _validate_same_workspace_negative_overlay(
    failures: list[str],
    fixture_id: str,
    fixture_dir: Path,
) -> None:
    canonical_root = fixture_dir / "examples/workspace_hash_crate"
    negative_root = fixture_dir / "negative"
    canonical = _read_toml(
        failures,
        fixture_id,
        "examples/workspace_hash_crate",
        canonical_root / "Cargo.toml",
    )
    negative = _read_toml(
        failures,
        fixture_id,
        "negative",
        negative_root / "Cargo.toml",
    )
    if isinstance(canonical, dict) and isinstance(negative, dict):
        for section in ("workspace", "package"):
            if negative.get(section) != canonical.get(section):
                failures.append(
                    f"{fixture_id}: negative/Cargo.toml {section} must match "
                    "the canonical scenario"
                )
        if negative.get("dependencies") not in ({}, None):
            failures.append(
                f"{fixture_id}: negative/Cargo.toml must omit the workspace_hash dependency"
            )

    canonical_lock = _read_toml(
        failures,
        fixture_id,
        "examples/workspace_hash_crate",
        canonical_root / "Cargo.lock",
    )
    negative_lock = _read_toml(
        failures,
        fixture_id,
        "negative",
        negative_root / "Cargo.lock",
    )
    if isinstance(canonical_lock, dict) and isinstance(negative_lock, dict):
        canonical_names = {
            package.get("name")
            for package in canonical_lock.get("package", [])
            if isinstance(package, dict)
        }
        negative_names = {
            package.get("name")
            for package in negative_lock.get("package", [])
            if isinstance(package, dict)
        }
        if negative_names != canonical_names:
            failures.append(
                f"{fixture_id}: negative/Cargo.lock package set must match "
                "the canonical scenario"
            )


def _validate_shared_bridge_negative_overlay(
    failures: list[str],
    fixture_id: str,
    fixture_dir: Path,
) -> None:
    canonical = _read_toml(
        failures,
        fixture_id,
        "examples/shared_hash_bridge",
        fixture_dir / "examples/shared_hash_bridge/sifr.toml",
    )
    negative = _read_toml(
        failures,
        fixture_id,
        "negative",
        fixture_dir / "negative/sifr.toml",
    )
    if isinstance(canonical, dict) and isinstance(negative, dict):
        for section in ("package", "source", "rust"):
            if negative.get(section) != canonical.get(section):
                failures.append(
                    f"{fixture_id}: negative/sifr.toml {section} must match "
                    "the canonical scenario"
                )
        expected_trust = ["sifr_shared_hash_bridge.generated_private_type"]
        if negative.get("trust", {}).get("rust-no-panic") != expected_trust:
            failures.append(
                f"{fixture_id}: negative/sifr.toml must trust only "
                "sifr_shared_hash_bridge.generated_private_type"
            )

    negative_rust = fixture_dir / "negative/shared_bridge_lib.rs"
    if not negative_rust.is_file():
        failures.append(f"{fixture_id}: negative/shared_bridge_lib.rs is required")
    elif "use crate::__sifr_bridge::" not in negative_rust.read_text(encoding="utf-8"):
        failures.append(
            f"{fixture_id}: negative/shared_bridge_lib.rs must exercise "
            "the package-generated import rejection"
        )


def _read_toml(failures: list[str], fixture_id: str, raw_path: str, path: Path) -> dict[str, Any] | None:
    if not path.is_file():
        return None
    try:
        return tomllib.loads(path.read_text(encoding="utf-8"))
    except tomllib.TOMLDecodeError as error:
        failures.append(f"{fixture_id}: {raw_path}/{path.name} is not valid TOML: {error}")
        return None


def _require_path_dependency(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    dependencies: Any,
    dependency: str,
    expected_path: str,
) -> None:
    actual = dependencies.get(dependency) if isinstance(dependencies, dict) else None
    if not isinstance(actual, dict) or actual.get("path") != expected_path:
        failures.append(f"{fixture_id}: {raw_path}/Cargo.toml must declare {dependency} path {expected_path}")


def _require_exact_dependency(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    dependencies: Any,
    dependency: str,
    expected_version: str,
) -> None:
    actual = dependencies.get(dependency) if isinstance(dependencies, dict) else None
    version = actual.get("version") if isinstance(actual, dict) else actual
    if version != expected_version:
        failures.append(
            f"{fixture_id}: {raw_path}/Cargo.toml dependency {dependency} "
            f"must pin {expected_version}"
        )


def _require_member(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    members: Any,
    expected_member: str,
) -> None:
    if not isinstance(members, list) or expected_member not in members:
        failures.append(f"{fixture_id}: {raw_path}/Cargo.toml workspace must include {expected_member}")


def _require_dependency_features(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    dependencies: Any,
    dependency: str,
    expected_features: list[str],
    default_features: bool | None = None,
) -> None:
    actual = dependencies.get(dependency) if isinstance(dependencies, dict) else None
    features = actual.get("features") if isinstance(actual, dict) else None
    if not isinstance(features, list) or set(features) != set(expected_features):
        failures.append(f"{fixture_id}: {raw_path}/Cargo.toml dependency {dependency} features must be {expected_features!r}")
    if default_features is not None and isinstance(actual, dict) and actual.get("default-features") is not default_features:
        failures.append(f"{fixture_id}: {raw_path}/Cargo.toml dependency {dependency} default-features mismatch")


def _require_trust_targets(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    trust: Any,
    key: str,
    expected_targets: list[str],
) -> None:
    targets = trust.get(key) if isinstance(trust, dict) else None
    missing = [target for target in expected_targets if not isinstance(targets, list) or target not in targets]
    for target in missing:
        failures.append(f"{fixture_id}: {raw_path}/sifr.toml [trust].{key} missing {target}")
