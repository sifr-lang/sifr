"""Manifest and harness policy for crate-backed zero-copy runtime evidence."""

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

ZERO_COPY_SCENARIO_TOKENS = (
    "@rust.zero_copy(owner=data, view=bridge.zero_copy.CrateBackedView)",
    "lifetime=owner, mutability=immutable, send=True, sync=True",
    "Bytes::from(data)",
    "owner.slice(..)",
    "drop(owner)",
    "MmapMut::map_anon",
    ".make_read_only()",
    "bytemuck::try_cast_slice",
    "Packet::ref_from_bytes",
    "impl Drop for CrateBackedView",
    "ACTIVE_VIEWS.fetch_sub",
    "RELEASED_VIEWS.fetch_add",
    "mutation=exclusive+sealed;send-sync=type-probed",
)


def reject_unsafe_rust(
    failures: list[str],
    fixture_id: str,
    rust_sources: list[Path],
    example_dir: Path,
) -> None:
    for source in rust_sources:
        for line_number, line in enumerate(
            source.read_text(encoding="utf-8").splitlines(), start=1
        ):
            if "unsafe" in line and not line.lstrip().startswith("//"):
                failures.append(
                    f"{fixture_id}: {source.relative_to(example_dir)}:"
                    f"{line_number} must use only safe Rust"
                )


def validate_zero_copy_scenario(
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
    for name, version, features in (
        ("bytemuck", "=1.25.2", None),
        ("bytes", "=1.11.1", None),
        ("memmap2", "=0.9.11", None),
        ("zerocopy", "=0.8.48", ["derive"]),
    ):
        if features is None:
            entry = dependencies.get(name)
            if not isinstance(entry, dict) or entry.get("version") != version:
                failures.append(
                    f"{fixture_id}: {raw_path}/Cargo.toml dependency {name} "
                    f"must pin version {version}"
                )
            elif entry.get("features"):
                failures.append(
                    f"{fixture_id}: {raw_path}/Cargo.toml dependency {name} "
                    "must not enable extra features"
                )
            continue
        _require_dependency(
            failures,
            fixture_id,
            raw_path,
            dependencies,
            name,
            version,
            features,
        )
    _require_trust(
        failures,
        fixture_id,
        raw_path,
        trust,
        "rust-no-panic",
        [
            "bridge.zero_copy.close",
            "bridge.zero_copy.create",
            "bridge.zero_copy.observe",
            "bridge.zero_copy.release_observation",
        ],
    )


def run_zero_copy_self_test(
    area_root: Path,
    validate_scenarios: ScenarioValidator,
) -> tuple[int, str | None]:
    source = area_root / "fixtures/zero_copy_runtime_matrix"
    raw_examples = {
        "crate_backed_view_runtime": "examples/crate_backed_view_runtime"
    }
    cases = 0
    with tempfile.TemporaryDirectory(
        prefix="sifr-rust-zero-copy-scenario-self-test-"
    ) as raw_temp:
        fixture_dir = Path(raw_temp) / "zero_copy_runtime_matrix"
        shutil.copytree(
            source,
            fixture_dir,
            ignore=shutil.ignore_patterns("target"),
        )
        baseline_failures: list[str] = []
        validate_scenarios(
            baseline_failures,
            "zero_copy_runtime_matrix",
            fixture_dir,
            raw_examples,
        )
        if baseline_failures:
            return cases, f"zero-copy baseline failed: {baseline_failures}"
        cases += 1

        mutation_cases = (
            (
                "bytes pin drift",
                "examples/crate_backed_view_runtime/Cargo.toml",
                'version = "=1.11.1"',
                'version = "1.11.1"',
                "must pin version =1.11.1",
            ),
            (
                "zerocopy derive drift",
                "examples/crate_backed_view_runtime/Cargo.toml",
                'features = ["derive"]',
                "features = []",
                "must declare features",
            ),
            (
                "zero-copy trust drift",
                "examples/crate_backed_view_runtime/sifr.toml",
                '  "bridge.zero_copy.observe",',
                '  "bridge.zero_copy.unobserved",',
                "trust.rust-no-panic",
            ),
            (
                "bytes owner retention drift",
                "examples/crate_backed_view_runtime/src/bridges/zero_copy.rs",
                "owner.slice(..)",
                "Bytes::copy_from_slice(owner.as_ref())",
                "missing scenario token 'owner.slice(..)'",
            ),
            (
                "mmap sealing drift",
                "examples/crate_backed_view_runtime/src/bridges/zero_copy.rs",
                ".make_read_only()",
                ".make_exec()",
                "missing scenario token '.make_read_only()'",
            ),
            (
                "release tracking drift",
                "examples/crate_backed_view_runtime/src/bridges/zero_copy.rs",
                "RELEASED_VIEWS.fetch_add",
                "RELEASED_VIEWS.fetch_sub",
                "missing scenario token 'RELEASED_VIEWS.fetch_add'",
            ),
            (
                "active-view tracking drift",
                "examples/crate_backed_view_runtime/src/bridges/zero_copy.rs",
                "ACTIVE_VIEWS.fetch_sub",
                "ACTIVE_VIEWS.fetch_add",
                "missing scenario token 'ACTIVE_VIEWS.fetch_sub'",
            ),
            (
                "bytemuck view drift",
                "examples/crate_backed_view_runtime/src/bridges/zero_copy.rs",
                "bytemuck::try_cast_slice",
                "bytemuck::pod_read_unaligned",
                "missing scenario token 'bytemuck::try_cast_slice'",
            ),
            (
                "zerocopy parse drift",
                "examples/crate_backed_view_runtime/src/bridges/zero_copy.rs",
                "Packet::ref_from_bytes",
                "Packet::read_from_bytes",
                "missing scenario token 'Packet::ref_from_bytes'",
            ),
            (
                "zero-copy decorator drift",
                "examples/crate_backed_view_runtime/src/main.sifr",
                "@rust.zero_copy(owner=data, view=bridge.zero_copy.CrateBackedView)",
                "@rust.zero_copy(owner=data, view=bridge.zero_copy.OtherView)",
                "missing scenario token "
                "'@rust.zero_copy(owner=data, "
                "view=bridge.zero_copy.CrateBackedView)'",
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
                "zero_copy_runtime_matrix",
                fixture_dir,
                raw_examples,
            )
            path.write_text(original, encoding="utf-8")
            if not any(expected in failure for failure in failures):
                return cases, f"{name} did not report {expected!r}: {failures}"
            cases += 1

        bridge_path = (
            fixture_dir
            / "examples/crate_backed_view_runtime/src/bridges/zero_copy.rs"
        )
        original = bridge_path.read_text(encoding="utf-8")
        bridge_path.write_text(
            original.replace(
                "pub fn create(mut data: Vec<u8>)",
                "pub unsafe fn create(mut data: Vec<u8>)",
                1,
            ),
            encoding="utf-8",
        )
        failures = []
        validate_scenarios(
            failures,
            "zero_copy_runtime_matrix",
            fixture_dir,
            raw_examples,
        )
        if not any("must use only safe Rust" in failure for failure in failures):
            return cases, f"unsafe bridge was accepted: {failures}"
        cases += 1

    return cases, None
