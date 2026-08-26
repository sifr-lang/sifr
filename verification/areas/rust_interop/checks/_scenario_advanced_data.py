"""Manifest and runtime policy for crate-backed advanced-data evidence."""

from __future__ import annotations

import shutil
import tempfile
from pathlib import Path
from typing import Any, Callable

ScenarioValidator = Callable[
    [list[str], str, Path, dict[str, Any]],
    int,
]

ADVANCED_DATA_SCENARIO_TOKENS = (
    "@rust.zero_copy(owner=input, view=sifr_arrow_bridge.record_batch.RecordBatchView)",
    "data=arrow_record_batch, schema=sifr_arrow_bridge.schema.RecordBatch",
    "data=tensor, dtype=f64, rank=2, shape=[2, 3]",
    "data=dlpack, dtype=f64, rank=2, shape=[2, 3]",
    "ownership=transfer, protocol=sifr_tensor_bridge.dlpack.Capsule",
    "Float64Array::from(input)",
    "array.values().as_ptr() as usize != input_pointer",
    "let polars_values = array.values().to_vec();",
    "MemTable::try_new",
    '.register_table("input"',
    'fill_nan(&ScalarValue::from(0.0), &["value"])',
    'map_err(display_error)?',
    "DataFrame::new",
    '.is_sorted(&["value".into()], &[false], &[false])',
    "Array2::from_shape_vec",
    "Tensor::from_vec",
    "storage_and_layout",
    ".into_inner()",
    ".take()",
    "impl Drop for OwnerGuard",
    "RELEASED_OWNERS.fetch_add",
    'assert arrow_before_close == "arrow-released=0;active=1"',
    'assert tensor_before_close == "tensor-released=0;active=1"',
    '"blake3_avx512_assembly"',
    '"blake3_neon"',
    '"blake3_sse2_sse41_avx2_assembly"',
)

EXPECTED_NO_PANIC_TARGETS = [
    "sifr_arrow_bridge.record_batch.close",
    "sifr_arrow_bridge.record_batch.create",
    "sifr_arrow_bridge.record_batch.observe",
    "sifr_arrow_bridge.record_batch.release_observation",
    "sifr_tensor_bridge.dlpack.close",
    "sifr_tensor_bridge.dlpack.observe",
    "sifr_tensor_bridge.dlpack.transfer",
    "sifr_tensor_bridge.tensor.close",
    "sifr_tensor_bridge.tensor.create",
    "sifr_tensor_bridge.tensor.observe",
    "sifr_tensor_bridge.tensor.release_observation",
]

EXPECTED_NATIVE_LINKS = [
    "blake3_avx512_assembly",
    "blake3_neon",
    "blake3_sse2_sse41_avx2_assembly",
    "lzma",
    "onig",
    "psm_s",
    "zstd",
]


def validate_advanced_data_scenario(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    cargo: dict[str, Any],
    dependencies: dict[str, Any],
    trust: dict[str, Any],
    example_dir: Path,
) -> None:
    workspace = cargo.get("workspace", {})
    workspace_dependencies = workspace.get("dependencies", {})
    members = workspace.get("members", [])
    if not isinstance(members, list) or set(members) != {
        "rust/sifr_arrow_bridge",
        "rust/sifr_tensor_bridge",
    }:
        failures.append(
            f"{fixture_id}: {raw_path}/Cargo.toml workspace members must be "
            "the two shared advanced-data bridges"
        )

    _require_dependency_entry(
        failures,
        fixture_id,
        raw_path,
        workspace_dependencies,
        "arrow",
        {"version": "=59.2.0", "default-features": True},
    )
    _require_dependency_entry(
        failures,
        fixture_id,
        raw_path,
        workspace_dependencies,
        "datafusion",
        {"version": "=55.0.0", "default-features": True},
    )
    _require_dependency_entry(
        failures,
        fixture_id,
        raw_path,
        workspace_dependencies,
        "polars",
        {"version": "=0.55.2", "default-features": True},
    )
    _require_dependency_entry(
        failures,
        fixture_id,
        raw_path,
        workspace_dependencies,
        "ndarray",
        {"version": "=0.17.2", "default-features": True},
    )
    _require_dependency_entry(
        failures,
        fixture_id,
        raw_path,
        workspace_dependencies,
        "candle",
        {
            "package": "candle-core",
            "version": "=0.11.0",
            "default-features": False,
        },
    )
    _require_dependency_entry(
        failures,
        fixture_id,
        raw_path,
        workspace_dependencies,
        "sifr_runtime",
        {"path": "../../../../../../../crates/sifr_runtime"},
    )
    for name, path in (
        ("sifr_arrow_bridge", "rust/sifr_arrow_bridge"),
        ("sifr_tensor_bridge", "rust/sifr_tensor_bridge"),
    ):
        _require_dependency_entry(
            failures,
            fixture_id,
            raw_path,
            dependencies,
            name,
            {"path": path},
        )
    _require_dependency_entry(
        failures,
        fixture_id,
        raw_path,
        dependencies,
        "sifr_runtime",
        {"workspace": True},
    )

    _require_shared_bridge_manifest(
        failures,
        fixture_id,
        raw_path,
        example_dir / "rust/sifr_arrow_bridge/Cargo.toml",
        "sifr_arrow_bridge",
        ["arrow", "datafusion", "polars", "sifr_runtime"],
    )
    _require_shared_bridge_manifest(
        failures,
        fixture_id,
        raw_path,
        example_dir / "rust/sifr_tensor_bridge/Cargo.toml",
        "sifr_tensor_bridge",
        ["candle", "ndarray", "sifr_runtime"],
    )
    _require_exact_trust(
        failures,
        fixture_id,
        raw_path,
        trust,
        "rust-no-panic",
        EXPECTED_NO_PANIC_TARGETS,
    )
    _require_exact_trust(
        failures,
        fixture_id,
        raw_path,
        trust,
        "native-links",
        EXPECTED_NATIVE_LINKS,
    )


def run_advanced_data_self_test(
    area_root: Path,
    validate_scenarios: ScenarioValidator,
) -> tuple[int, str | None]:
    source = area_root / "fixtures/advanced_data_runtime_matrix"
    raw_examples = {"advanced_data_runtime": "examples/advanced_data_runtime"}
    cases = 0
    with tempfile.TemporaryDirectory(
        prefix="sifr-rust-advanced-data-scenario-self-test-"
    ) as raw_temp:
        fixture_dir = Path(raw_temp) / "advanced_data_runtime_matrix"
        shutil.copytree(source, fixture_dir, ignore=shutil.ignore_patterns("target"))
        baseline_failures: list[str] = []
        validate_scenarios(
            baseline_failures,
            "advanced_data_runtime_matrix",
            fixture_dir,
            raw_examples,
        )
        if baseline_failures:
            return cases, f"advanced-data baseline failed: {baseline_failures}"
        cases += 1

        mutation_cases = (
            (
                "Arrow pin drift",
                "examples/advanced_data_runtime/Cargo.toml",
                'arrow = { version = "=59.2.0", default-features = true }',
                'arrow = { version = "59.2.0", default-features = true }',
                "workspace dependency arrow",
            ),
            (
                "Candle backend drift",
                "examples/advanced_data_runtime/Cargo.toml",
                'default-features = false }',
                'default-features = true }',
                "workspace dependency candle",
            ),
            (
                "native-link trust drift",
                "examples/advanced_data_runtime/sifr.toml",
                '"psm_s",',
                "",
                "trust.native-links",
            ),
            (
                "Arrow allocation identity drift",
                "examples/advanced_data_runtime/rust/sifr_arrow_bridge/src/record_batch.rs",
                "array.values().as_ptr() as usize != input_pointer",
                "array.len() != input_pointer",
                "missing scenario token 'array.values().as_ptr() as usize != input_pointer'",
            ),
            (
                "DataFusion registration drift",
                "examples/advanced_data_runtime/rust/sifr_arrow_bridge/src/record_batch.rs",
                '.register_table("input"',
                '.deregister_table("input"',
                "missing scenario token '.register_table(\"input\"'",
            ),
            (
                "DataFusion 55 NaN-fill planning drift",
                "examples/advanced_data_runtime/rust/sifr_arrow_bridge/src/record_batch.rs",
                'fill_nan(&ScalarValue::from(0.0), &["value"])',
                'fill_null(&ScalarValue::from(0.0), &["value"])',
                "missing scenario token 'fill_nan(&ScalarValue::from(0.0), "
                '&["value"])\'',
            ),
            (
                "Polars crossed-data derivation drift",
                "examples/advanced_data_runtime/rust/sifr_arrow_bridge/src/record_batch.rs",
                "let polars_values = array.values().to_vec();",
                "let polars_values = vec![1.0_f64; array.len()];",
                "missing scenario token 'let polars_values = array.values().to_vec();'",
            ),
            (
                "Polars 0.55 dataframe sortedness drift",
                "examples/advanced_data_runtime/rust/sifr_arrow_bridge/src/record_batch.rs",
                '.is_sorted(&["value".into()], &[false], &[false])',
                '.is_unique(&["value".into()], &[false], &[false])',
                "missing scenario token '.is_sorted(&[\"value\".into()], "
                '&[false], &[false])\'',
            ),
            (
                "ndarray allocation identity drift",
                "examples/advanced_data_runtime/rust/sifr_tensor_bridge/src/tensor.rs",
                "Array2::from_shape_vec",
                "Array2::from_shape_fn",
                "missing scenario token 'Array2::from_shape_vec'",
            ),
            (
                "Candle owned allocation drift",
                "examples/advanced_data_runtime/rust/sifr_tensor_bridge/src/tensor.rs",
                "Tensor::from_vec",
                "Tensor::from_slice",
                "missing scenario token 'Tensor::from_vec'",
            ),
            (
                "DLPack ownership transfer drift",
                "examples/advanced_data_runtime/rust/sifr_tensor_bridge/src/dlpack.rs",
                ".take()",
                "owner.as_ref()",
                "missing scenario token '.take()'",
            ),
            (
                "owner cleanup drift",
                "examples/advanced_data_runtime/rust/sifr_tensor_bridge/src/tensor.rs",
                "RELEASED_OWNERS.fetch_add",
                "RELEASED_OWNERS.fetch_sub",
                "missing scenario token 'RELEASED_OWNERS.fetch_add'",
            ),
            (
                "pre-close owner observation drift",
                "examples/advanced_data_runtime/src/main.sifr",
                'assert tensor_before_close == "tensor-released=0;active=1"',
                'assert tensor_before_close == "tensor-released=1;active=0"',
                "missing scenario token 'assert tensor_before_close == "
                '"tensor-released=0;active=1"\'',
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
                "advanced_data_runtime_matrix",
                fixture_dir,
                raw_examples,
            )
            path.write_text(original, encoding="utf-8")
            if not any(expected in failure for failure in failures):
                return cases, f"{name} did not report {expected!r}: {failures}"
            cases += 1

        rust_path = (
            fixture_dir
            / "examples/advanced_data_runtime/rust/sifr_tensor_bridge/src/dlpack.rs"
        )
        original = rust_path.read_text(encoding="utf-8")
        rust_path.write_text(
            original.replace("pub fn transfer(", "pub unsafe fn transfer(", 1),
            encoding="utf-8",
        )
        failures = []
        validate_scenarios(
            failures,
            "advanced_data_runtime_matrix",
            fixture_dir,
            raw_examples,
        )
        if not any("must use only safe Rust" in failure for failure in failures):
            return cases, f"unsafe advanced-data bridge was accepted: {failures}"
        cases += 1

    return cases, None


def _require_dependency_entry(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    dependencies: Any,
    name: str,
    expected: dict[str, Any],
) -> None:
    actual = dependencies.get(name) if isinstance(dependencies, dict) else None
    if not isinstance(actual, dict) or any(
        actual.get(key) != value for key, value in expected.items()
    ):
        failures.append(
            f"{fixture_id}: {raw_path}/Cargo.toml workspace dependency "
            f"{name} must declare {expected!r}"
        )


def _require_shared_bridge_manifest(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    path: Path,
    package_name: str,
    expected_dependencies: list[str],
) -> None:
    import tomllib

    try:
        cargo = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        failures.append(f"{fixture_id}: {raw_path}/{path.name} is invalid: {error}")
        return
    if cargo.get("package", {}).get("name") != package_name:
        failures.append(
            f"{fixture_id}: {raw_path}/{path.parent.name}/Cargo.toml "
            f"must declare package {package_name}"
        )
    dependencies = cargo.get("dependencies", {})
    for dependency in expected_dependencies:
        if dependencies.get(dependency) != {"workspace": True}:
            failures.append(
                f"{fixture_id}: {raw_path}/{path.parent.name}/Cargo.toml "
                f"dependency {dependency} must use workspace = true"
            )


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
            f"{fixture_id}: {raw_path}/sifr.toml trust.{key} must equal {expected!r}"
        )
