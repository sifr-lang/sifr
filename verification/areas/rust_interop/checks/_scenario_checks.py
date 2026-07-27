"""Scenario-package validation for the Rust interop fixture matrix."""

from __future__ import annotations

import shutil
import tempfile
import tomllib
from pathlib import Path
from typing import Any

from _binding_helpers import contains_empty_pass_body as _contains_empty_pass_body
from _binding_helpers import rust_bound_declarations as _rust_bound_declarations
from _binding_helpers import verifier_binds_call as _verifier_binds_call

REQUIRED_SCENARIO_EXAMPLES = {
    "bridge_type_matrix": {
        "bridge_type_roundtrip": {
            "tokens": (
                "serde_json_roundtrip",
                "bytes_roundtrip",
                "indexmap_roundtrip",
                "nested_indexmap_roundtrip",
                "indexmap_list_roundtrip",
                "thiserror",
            ),
        },
    },
    "bridge_version_mismatch": {
        "bridge_version_package": {
            "tokens": ("bridge-version = 1", "version_bridge"),
        },
    },
    "callbacks_call_scoped": {
        "call_scoped_callback_runtime": {
            "tokens": (
                "CallScopedCallbackBridge",
                "bridge.callbacks.visit",
                "Rust bridge panicked",
            ),
        },
    },
    "cargo_locked_offline": {
        "locked_offline_cache": {
            "tokens": ("locked_bridge", "Cargo.lock", "--locked", "--offline", "--frozen"),
        },
    },
    "local_bridge_blake3": {
        "local_blake3_bridge": {
            "tokens": ("bridge.blake3.hash_bytes", "src/bridges", "blake3"),
        },
    },
    "panic_abort_profile": {
        "abort_profile_package": {
            "tokens": ("rust-panic-abort", "panic = \"abort\"", "legacy_backend"),
        },
    },
    "panic_boundary_wrapper_emission": {
        "panic_wrapper_runtime": {
            "tokens": (
                "RustPanicErrorBridge",
                "mapper_panics",
                "--locked",
                "--offline",
                "--frozen",
            ),
        },
    },
    "proc_macro_trust": {
        "proc_macro_trust_package": {
            "tokens": ("rust-proc-macros", "rust-build-scripts", "serde_derive", "prost-build"),
        },
    },
    "same_workspace_crate": {
        "workspace_hash_crate": {
            "tokens": ("workspace_hash", "path = \"rust/workspace_hash\"", "members = ["),
        },
    },
    "shared_bridge_crate": {
        "shared_hash_bridge": {
            "tokens": ("sifr_shared_hash_bridge", "digest_hex", "crate::__sifr_bridge"),
        },
    },
    "native_build_script": {
        "native_trust_package": {
            "tokens": ("rust-build-scripts", "native-links", "zstd", "bindgen", "cxx"),
        },
    },
    "ecosystem_backend_certification": {
        "backend_feature_package": {
            "tokens": ("runtime-tokio-rustls", "postgres", "macros", "tower-http"),
        },
    },
    "ecosystem_cli_certification": {
        "cli_feature_package": {
            "tokens": ("env-filter", "tracing-subscriber", "clap"),
        },
    },
}

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
        shutil.copytree(source, fixture_dir)

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
        shutil.copytree(source, fixture_dir)

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

    sifr_sources = sorted(example_dir.rglob("*.sifr"))
    cargo_manifests = sorted(example_dir.rglob("Cargo.toml"))
    rust_sources = sorted(example_dir.rglob("*.rs"))
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
    for source in sifr_sources:
        text = source.read_text(encoding="utf-8")
        raw_source_path = source.relative_to(example_dir).as_posix()
        _validate_scenario_sifr_source(failures, fixture_id, example, f"{raw_path}/{raw_source_path}", text)


def _read_scenario_text(
    readme_path: Path,
    sifr_config_path: Path,
    sifr_sources: list[Path],
    cargo_manifests: list[Path],
    rust_sources: list[Path],
) -> str:
    paths = [readme_path, sifr_config_path, *sifr_sources, *cargo_manifests, *rust_sources]
    return "\n".join(path.read_text(encoding="utf-8") for path in paths if path.is_file())


def _validate_scenario_sifr_source(
    failures: list[str],
    fixture_id: str,
    example: str,
    raw_path: str,
    text: str,
) -> None:
    if len(text.strip().splitlines()) < 10:
        failures.append(f"{fixture_id}: {raw_path} must contain a full scenario source")
    for header in ("# execution-kind:", "# expected-result:"):
        if header not in text:
            failures.append(f"{fixture_id}: {raw_path} missing {header} header")
    if _contains_empty_pass_body(text):
        failures.append(f"{fixture_id}: {raw_path} must not use empty placeholder class bodies")
    if not any(line.lstrip().startswith("@rust") for line in text.splitlines()):
        failures.append(f"{fixture_id}: {raw_path} must exercise a Rust interop declaration")
    bound_declarations = _rust_bound_declarations(text)
    if not bound_declarations:
        failures.append(f"{fixture_id}: {raw_path} must include Rust-decorated binding declarations")

    verifier_markers = (f"def verify_{example}(", f"async def verify_{example}(")
    verifier_start = min((text.find(marker) for marker in verifier_markers if marker in text), default=-1)
    if verifier_start < 0:
        failures.append(f"{fixture_id}: {raw_path} must include verify_{example}")
        return

    verifier_body = text[verifier_start:]
    for name, return_type in bound_declarations:
        if f"{name}(" not in verifier_body and f".{name}(" not in verifier_body:
            failures.append(f"{fixture_id}: {raw_path} verifier must call {name}")
        if return_type != "None" and not _verifier_binds_call(verifier_body, name):
            failures.append(f"{fixture_id}: {raw_path} verifier must bind {name} result before returning")


def _reject_generated_bridge_imports(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    rust_sources: list[Path],
) -> None:
    for source in rust_sources:
        for line_number, line in enumerate(source.read_text(encoding="utf-8").splitlines(), start=1):
            stripped = line.strip()
            if "crate::__sifr_bridge" in stripped and not stripped.startswith("//"):
                relative = source.as_posix().split(f"{raw_path}/", maxsplit=1)[-1]
                failures.append(f"{fixture_id}: {relative}:{line_number} must not reference crate::__sifr_bridge")


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
    root_lock = _read_toml(
        failures,
        fixture_id,
        "repository root",
        REPO_ROOT / "Cargo.lock",
    )
    if isinstance(scenario_lock, dict) and isinstance(root_lock, dict):
        _require_root_lock_subset(
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
    elif fixture_id == "same_workspace_crate":
        _require_path_dependency(failures, fixture_id, raw_path, dependencies, "workspace_hash", "rust/workspace_hash")
        _require_member(failures, fixture_id, raw_path, workspace_members, "rust/workspace_hash")
        _require_trust_targets(failures, fixture_id, raw_path, trust, "rust-no-panic", ["workspace_hash.hash", "workspace_hash.hash_pair"])
    elif fixture_id == "shared_bridge_crate":
        _require_path_dependency(failures, fixture_id, raw_path, dependencies, "sifr_shared_hash_bridge", "rust/sifr_shared_hash_bridge")
        _require_trust_targets(failures, fixture_id, raw_path, trust, "rust-no-panic", ["sifr_shared_hash_bridge.digest", "sifr_shared_hash_bridge.digest_hex"])
    elif fixture_id == "cargo_locked_offline":
        _require_path_dependency(failures, fixture_id, raw_path, dependencies, "locked_bridge", "rust/locked_bridge")
        if not (example_dir / "Cargo.lock").is_file():
            failures.append(f"{fixture_id}: {raw_path}/Cargo.lock is required")
        _read_toml(failures, fixture_id, raw_path, example_dir / "Cargo.lock")
        _require_trust_targets(failures, fixture_id, raw_path, trust, "rust-no-panic", ["locked_bridge.cached_hash", "locked_bridge.lockfile_generation"])
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
        _require_path_dependency(failures, fixture_id, raw_path, dependencies, "serde_derive", "rust/serde_derive")
        _require_path_dependency(failures, fixture_id, raw_path, dependencies, "prost-build", "rust/prost_build")
        _require_proc_macro_lib(failures, fixture_id, raw_path, example_dir, "rust/serde_derive")
        _require_build_script(failures, fixture_id, raw_path, example_dir, "rust/prost_build")
        _require_trust_targets(failures, fixture_id, raw_path, trust, "rust-proc-macros", ["serde_derive"])
        _require_trust_targets(failures, fixture_id, raw_path, trust, "rust-build-scripts", ["prost-build"])
    elif fixture_id == "native_build_script":
        for dependency in ("cc", "bindgen", "cxx", "zstd"):
            _require_path_dependency(failures, fixture_id, raw_path, dependencies, dependency, f"rust/{dependency}")
            _require_build_script(failures, fixture_id, raw_path, example_dir, f"rust/{dependency}")
        _require_native_links(failures, fixture_id, raw_path, example_dir, "rust/zstd", "zstd")
        _require_trust_targets(failures, fixture_id, raw_path, trust, "rust-build-scripts", ["cc", "bindgen", "cxx", "zstd"])
        _require_trust_targets(failures, fixture_id, raw_path, trust, "native-links", ["zstd"])
    elif fixture_id == "ecosystem_backend_certification":
        _require_path_dependency(failures, fixture_id, raw_path, dependencies, "axum", "rust/axum")
        _require_path_dependency(failures, fixture_id, raw_path, dependencies, "tower-http", "rust/tower_http")
        _require_path_dependency(failures, fixture_id, raw_path, dependencies, "sqlx", "rust/sqlx")
        _require_dependency_features(failures, fixture_id, raw_path, dependencies, "sqlx", ["runtime-tokio-rustls", "postgres", "macros"], default_features=False)
    elif fixture_id == "ecosystem_cli_certification":
        for dependency in ("anyhow", "clap", "tracing"):
            _require_path_dependency(failures, fixture_id, raw_path, dependencies, dependency, f"rust/{dependency}")
        _require_path_dependency(failures, fixture_id, raw_path, dependencies, "tracing-subscriber", "rust/tracing_subscriber")
        _require_dependency_features(failures, fixture_id, raw_path, dependencies, "tracing-subscriber", ["env-filter"])


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


def _require_root_lock_subset(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    scenario_lock: dict[str, Any],
    root_lock: dict[str, Any],
) -> None:
    root_packages = {
        (str(package.get("name")), str(package.get("version")))
        for package in root_lock.get("package", [])
        if isinstance(package, dict) and package.get("source")
    }
    for package in scenario_lock.get("package", []):
        if not isinstance(package, dict) or not package.get("source"):
            continue
        identity = (str(package.get("name")), str(package.get("version")))
        if identity not in root_packages:
            failures.append(
                f"{fixture_id}: {raw_path}/Cargo.lock package "
                f"{identity[0]} {identity[1]} is not present in root Cargo.lock"
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


def _require_proc_macro_lib(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    example_dir: Path,
    crate_path: str,
) -> None:
    manifest = _read_toml(failures, fixture_id, raw_path, example_dir / crate_path / "Cargo.toml")
    if not isinstance(manifest, dict) or manifest.get("lib", {}).get("proc-macro") is not True:
        failures.append(f"{fixture_id}: {raw_path}/{crate_path}/Cargo.toml must declare [lib] proc-macro = true")


def _require_build_script(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    example_dir: Path,
    crate_path: str,
) -> None:
    manifest = _read_toml(failures, fixture_id, raw_path, example_dir / crate_path / "Cargo.toml")
    if not isinstance(manifest, dict) or manifest.get("package", {}).get("build") != "build.rs":
        failures.append(f"{fixture_id}: {raw_path}/{crate_path}/Cargo.toml must declare package build = \"build.rs\"")
    if not (example_dir / crate_path / "build.rs").is_file():
        failures.append(f"{fixture_id}: {raw_path}/{crate_path}/build.rs is required")


def _require_native_links(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    example_dir: Path,
    crate_path: str,
    expected_links: str,
) -> None:
    manifest = _read_toml(failures, fixture_id, raw_path, example_dir / crate_path / "Cargo.toml")
    if not isinstance(manifest, dict) or manifest.get("package", {}).get("links") != expected_links:
        failures.append(f"{fixture_id}: {raw_path}/{crate_path}/Cargo.toml must declare links = {expected_links!r}")


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
