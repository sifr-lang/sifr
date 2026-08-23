#!/usr/bin/env python3
"""Ensure retained compiler-native stdlib glue is explicitly allowlisted."""

from __future__ import annotations

import json
import re
import sys
import tempfile
import tomllib
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[1]
ALLOWLIST_PATH = REPO_ROOT / "internal_docs" / "stdlib_retained_compiler_intrinsics.toml"
REGISTRY_ROOT = REPO_ROOT / "crates" / "sifr_codegen" / "src" / "intrinsics" / "registry"
REGISTRY_DISPATCH_PATH = (
    REPO_ROOT / "crates" / "sifr_codegen" / "src" / "intrinsics" / "registry.rs"
)
HIR_NODES_PATH = REPO_ROOT / "crates" / "sifr_ir" / "src" / "hir_nodes.rs"
LOWERING_ROOT = REPO_ROOT / "crates" / "sifr_lowering" / "src"
PREAMBLE_ROOT = REPO_ROOT / "crates" / "sifr_codegen" / "src" / "preamble"
CODEGEN_ROOT = REPO_ROOT / "crates" / "sifr_codegen" / "src"
STDLIB_SOURCE_ROOT = REPO_ROOT / "stdlib"
STDLIB_FEATURES_PATH = REPO_ROOT / "crates" / "sifr_stdlib_manifest" / "src" / "features.rs"
DEPENDENCY_PLAN_RS_PATH = (
    REPO_ROOT
    / "crates"
    / "sifr_stdlib_manifest"
    / "src"
    / "features"
    / "dependency_plan.rs"
)
DELETED_OWNERSHIP_REGISTRY = REPO_ROOT / "internal_docs" / "stdlib_native_surface_ownership.toml"
ARCH_DOC_PATH = REPO_ROOT / "internal_docs" / "sifr_sysroot_and_stdlib_architecture.md"

TYPED_INTRINSIC_PAIR_RE = re.compile(
    r'Self::([A-Za-z0-9_]+)\s*=>\s*"([A-Za-z0-9_]+)"'
)
DISPATCH_INTRINSIC_VARIANT_RE = re.compile(r"CompilerIntrinsicId::([A-Za-z0-9_]+)\s*=>")
SOURCE_INTRINSIC_RE = re.compile(r"@compiler_intrinsic\(([A-Za-z0-9_]+)\)")
STDLIB_FEATURE_PAIR_RE = re.compile(
    r'Self::([A-Za-z0-9_]+)\s*=>\s*"([A-Za-z0-9_-]+)"'
)
DEPENDENCY_FEATURE_RE = re.compile(
    r'StdlibFeature::([A-Za-z0-9_]+)\s*=>\s*(?:\{\s*)?&\[\s*"([A-Za-z0-9_-]+)\s+=',
    re.DOTALL,
)
CODEGEN_FEATURE_RE = re.compile(r"StdlibFeature::([A-Za-z0-9_]+)")
LOWERER_MATCH_INTRINSIC_RE = re.compile(r'"([A-Za-z0-9_]+)"\s*(?=\||=>)')
PREFIX_INTRINSIC_RE = re.compile(r'starts_with\("([A-Za-z0-9_]+)"\)')
GENERATED_DEPENDENCY_PACKAGE_RE = re.compile(r'"([A-Za-z0-9_-]+)\s+=')
DIRECT_RUNTIME_ROOT_RE = re.compile(r"\bsifr_runtime::([A-Za-z_][A-Za-z0-9_]*)")
EXPECTED_PREFIX_DISPATCHERS: set[str] = set()
PREFIX_DISPATCH_LOWERERS: tuple[Path, ...] = ()
STALE_ARCH_PHRASES = (
    "complete surface-by-surface ownership decision remains the TOML registry",
    "validated against the compiler intrinsic registry; compiler intrinsic metadata remains the current signature owner",
    "old handwritten intrinsic registry is removed or reduced",
)
DELETED_COLLECTION_RESIDUES = (
    "counter_from_list",
    "counter_get",
    "counter_increment",
    "counter_items",
    "counter_keys",
    "counter_most_common",
    "counter_total",
    "counter_values",
    "_defaultdict_new_impl",
    "_defaultdict_get_impl",
    "_defaultdict_set_impl",
)
DELETED_COLLECTION_RESIDUE_ROOTS = (
    REPO_ROOT / "crates" / "sifr_ir" / "src" / "hir_nodes.rs",
    REGISTRY_ROOT,
    REPO_ROOT / "crates" / "sifr_stdlib" / "src" / "collections.rs",
    REPO_ROOT / "stdlib" / "_sifr" / "collections.sifr",
    REPO_ROOT / "stdlib" / "sifr" / "collections.sifr",
)
DELETED_FALLBACK_PATHS = (
    REPO_ROOT / "crates" / "sifr_retained_intrinsics",
    REPO_ROOT / "stdlib" / "_sifr" / "io.sifr",
    REPO_ROOT / "stdlib" / "_sifr" / "test.sifr",
)
DELETED_FALLBACK_TOKENS = (
    "sifr_retained_intrinsics",
    "fallback_signature_modules",
    "resolve_retained_fallback",
    "re_export_intrinsic_fallbacks",
    "get_intrinsic_module",
)
DELETED_FALLBACK_SCAN_ROOTS = (
    REPO_ROOT / "Cargo.toml",
    REPO_ROOT / "Cargo.lock",
    REPO_ROOT / "crates",
    REPO_ROOT / "internal_docs" / "stdlib_retained_compiler_intrinsics.toml",
    REPO_ROOT / "internal_docs" / "architecture.md",
    ARCH_DOC_PATH,
    REPO_ROOT / "scripts" / "check_source_crate_dependency_direction.py",
    REPO_ROOT / "scripts" / "check_stdlib_manifest_schema.py",
    REPO_ROOT / "verification" / "profiles",
    REPO_ROOT
    / "verification"
    / "areas"
    / "coverage_matrix"
    / "data"
    / "cargo_metadata_classification.json",
    REPO_ROOT
    / "verification"
    / "areas"
    / "generated_code_quality"
    / "generated_code_quality.py",
)
FALLBACK_SCAN_SUFFIXES = {".json", ".lock", ".py", ".rs", ".toml"}


def main() -> int:
    observed = _observed_surface()
    allowlist = tomllib.loads(ALLOWLIST_PATH.read_text(encoding="utf-8"))
    failures = _deleted_collection_residue_failures()
    failures.extend(_deleted_fallback_architecture_failures())
    if failures:
        print("stdlib native intrinsic allowlist guard: FAIL")
        for failure in failures:
            print(f"- {failure}")
        return 1
    return _run(observed, allowlist)


def _deleted_collection_residue_failures() -> list[str]:
    failures = []
    for root in DELETED_COLLECTION_RESIDUE_ROOTS:
        if not root.exists():
            continue
        paths = root.rglob("*.rs") if root.is_dir() else (root,)
        for path in paths:
            text = path.read_text(encoding="utf-8")
            for residue in DELETED_COLLECTION_RESIDUES:
                if residue in text:
                    failures.append(
                        f"deleted collections residue {residue!r} remains in "
                        f"{path.relative_to(REPO_ROOT)}"
                    )
    return failures


def _deleted_fallback_architecture_failures(
    deleted_paths: tuple[Path, ...] = DELETED_FALLBACK_PATHS,
    scan_roots: tuple[Path, ...] = DELETED_FALLBACK_SCAN_ROOTS,
) -> list[str]:
    failures = []
    for path in deleted_paths:
        if path.exists():
            failures.append(
                f"deleted fallback architecture path remains: {path.relative_to(REPO_ROOT)}"
            )
    for root in scan_roots:
        paths = root.rglob("*") if root.is_dir() else (root,)
        for path in paths:
            if (
                not path.is_file()
                or path.is_symlink()
                or path.suffix not in FALLBACK_SCAN_SUFFIXES
            ):
                continue
            try:
                text = path.read_text(encoding="utf-8")
            except UnicodeDecodeError:
                continue
            for token in DELETED_FALLBACK_TOKENS:
                if token in text:
                    failures.append(
                        f"deleted fallback architecture token {token!r} remains in "
                        f"{path.relative_to(REPO_ROOT)}"
                    )
    return failures


def _run(observed: dict[str, set[str]], allowlist: dict[str, Any]) -> int:
    failures = _validate(observed, allowlist)
    if failures:
        print("stdlib native intrinsic allowlist guard: FAIL")
        for failure in failures:
            print(f"- {failure}")
        return 1

    print(
        "stdlib native intrinsic allowlist guard: PASS "
        f"(exact_intrinsics={len(observed['exact_intrinsics'])}, "
        f"registry_files={len(observed['registry_files'])}, "
        f"preamble_files={len(observed['preamble_files'])}, "
        f"lowering_files={len(observed['lowering_files'])}, "
        f"codegen_files={len(observed['codegen_files'])}, "
        "retained_direct_dependency_packages="
        f"{len(observed['retained_direct_dependency_packages'])}, "
        f"direct_runtime_roots={len(observed['direct_runtime_roots'])})"
    )
    return 0


def _observed_surface() -> dict[str, set[str]]:
    registry_text = REGISTRY_DISPATCH_PATH.read_text(encoding="utf-8")
    typed_pairs = dict(
        TYPED_INTRINSIC_PAIR_RE.findall(HIR_NODES_PATH.read_text(encoding="utf-8"))
    )
    exact_intrinsics = set(typed_pairs.values())
    for lowerer_path in PREFIX_DISPATCH_LOWERERS:
        exact_intrinsics.update(
            LOWERER_MATCH_INTRINSIC_RE.findall(lowerer_path.read_text(encoding="utf-8"))
        )
    return {
        "exact_intrinsics": exact_intrinsics,
        "dispatch_intrinsics": {
            typed_pairs[variant]
            for variant in DISPATCH_INTRINSIC_VARIANT_RE.findall(registry_text)
            if variant in typed_pairs
        },
        "unmapped_dispatch_variants": {
            variant
            for variant in DISPATCH_INTRINSIC_VARIANT_RE.findall(registry_text)
            if variant not in typed_pairs
        },
        "source_declared_intrinsics": _source_declared_intrinsics(),
        "prefix_dispatchers": set(PREFIX_INTRINSIC_RE.findall(registry_text)),
        "registry_files": {
            path.relative_to(REGISTRY_ROOT).as_posix()
            for path in REGISTRY_ROOT.rglob("*.rs")
        },
        "preamble_files": {
            path.relative_to(PREAMBLE_ROOT).as_posix()
            for path in PREAMBLE_ROOT.rglob("*.rs")
        },
        "lowering_files": _compiler_intrinsic_files(LOWERING_ROOT),
        "codegen_files": _compiler_intrinsic_files(CODEGEN_ROOT),
        "retained_direct_dependency_packages": {
            package
            for package in GENERATED_DEPENDENCY_PACKAGE_RE.findall(
                _non_test_dependency_plan_text()
            )
        },
        "retained_direct_dependency_features": _retained_dependency_feature_mappings(),
        "orphan_retained_dependency_features": _orphan_retained_dependency_features(),
        "direct_runtime_roots": _direct_runtime_roots(),
    }


def _source_declared_intrinsics() -> set[str]:
    declared: set[str] = set()
    for path in STDLIB_SOURCE_ROOT.rglob("*.sifr"):
        declared.update(SOURCE_INTRINSIC_RE.findall(path.read_text(encoding="utf-8")))
    return declared


def _compiler_intrinsic_files(root: Path) -> set[str]:
    files = set()
    for path in root.rglob("*.rs"):
        relative = path.relative_to(root).as_posix()
        if _is_test_source(relative):
            continue
        if "CompilerIntrinsicId" in path.read_text(encoding="utf-8"):
            files.add(path.relative_to(REPO_ROOT).as_posix())
    return files


def _feature_variant_ids() -> dict[str, str]:
    return dict(STDLIB_FEATURE_PAIR_RE.findall(STDLIB_FEATURES_PATH.read_text(encoding="utf-8")))


def _retained_dependency_pairs() -> set[tuple[str, str]]:
    return set(DEPENDENCY_FEATURE_RE.findall(_non_test_dependency_plan_text()))


def _retained_dependency_feature_mappings() -> set[str]:
    feature_ids = _feature_variant_ids()
    return {
        f"{package}={feature_ids[variant]}"
        for variant, package in _retained_dependency_pairs()
        if variant in feature_ids
    }


def _orphan_retained_dependency_features() -> set[str]:
    live_variants: set[str] = set()
    for path in CODEGEN_ROOT.rglob("*.rs"):
        relative = path.relative_to(CODEGEN_ROOT).as_posix()
        if _is_test_source(relative):
            continue
        live_variants.update(CODEGEN_FEATURE_RE.findall(path.read_text(encoding="utf-8")))
    return {
        variant
        for variant, _package in _retained_dependency_pairs()
        if variant not in live_variants
    }


def _non_test_dependency_plan_text() -> str:
    text = DEPENDENCY_PLAN_RS_PATH.read_text(encoding="utf-8")
    test_marker = "\n#[cfg(test)]"
    return text.split(test_marker, 1)[0]


def _validate(observed: dict[str, set[str]], allowlist: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    allowed = {
        "exact_intrinsics": set[str](),
        "source_declared_intrinsics": set[str](),
        "registry_files": set[str](),
        "preamble_files": set[str](),
        "lowering_files": set[str](),
        "codegen_files": set[str](),
        "retained_direct_dependency_packages": set[str](),
        "retained_direct_dependency_features": set[str](),
        "direct_runtime_roots": set[str](),
    }
    owners: dict[tuple[str, str], str] = {}
    unique_owner_keys = {
        "exact_intrinsics",
        "source_declared_intrinsics",
        "registry_files",
        "preamble_files",
        "lowering_files",
        "codegen_files",
        "retained_direct_dependency_features",
        "direct_runtime_roots",
    }

    surfaces = allowlist.get("surface", [])
    if not isinstance(surfaces, list) or not surfaces:
        return ["allowlist must contain at least one [[surface]] entry"]

    failures.extend(
        _permanent_file_failures(
            DELETED_OWNERSHIP_REGISTRY.exists(),
            ARCH_DOC_PATH.read_text(encoding="utf-8"),
        )
    )

    for index, surface in enumerate(surfaces):
        if not isinstance(surface, dict):
            failures.append(f"surface entry {index}: must be a table")
            continue

        surface_id = _required_text(failures, surface, "id", f"surface entry {index}")
        reason = _required_text(
            failures,
            surface,
            "reason",
            surface_id or f"surface entry {index}",
        )
        if not surface_id:
            continue

        has_items = False
        for key in allowed:
            values = _string_list(failures, surface, key, surface_id)
            if values:
                has_items = True
            for value in values:
                owner_key = (key, value)
                previous_owner = owners.get(owner_key)
                if previous_owner is not None:
                    if key in unique_owner_keys:
                        failures.append(
                            f"{key} entry {value!r} is duplicated by {surface_id} "
                            f"and {previous_owner}"
                        )
                        continue
                owners[owner_key] = surface_id
                allowed[key].add(value)

        state = surface.get("state")
        if state != "retained-by-design":
            failures.append(f"{surface_id}: state must be retained-by-design")
        if has_items and not reason:
            failures.append(f"{surface_id}: reason is required for retained compiler-native glue")
        if not has_items:
            failures.append(f"{surface_id}: allowlist entry has no retained files or intrinsics")

    prefix_dispatchers = observed.get("prefix_dispatchers", set())
    unexpected_prefixes = sorted(prefix_dispatchers - EXPECTED_PREFIX_DISPATCHERS)
    stale_prefixes = sorted(EXPECTED_PREFIX_DISPATCHERS - prefix_dispatchers)
    if unexpected_prefixes:
        failures.append(
            "registry.rs contains untracked prefix dispatchers: "
            + ", ".join(unexpected_prefixes)
        )
    if stale_prefixes:
        failures.append(
            "expected prefix dispatchers are missing from registry.rs: "
            + ", ".join(stale_prefixes)
        )

    _compare_sets(
        failures,
        "typed dispatch intrinsics",
        observed.get("dispatch_intrinsics", set()),
        observed.get("exact_intrinsics", set()),
    )
    unmapped_dispatch_variants = sorted(
        observed.get("unmapped_dispatch_variants", set())
    )
    if unmapped_dispatch_variants:
        failures.append(
            "dispatch variants missing typed declaration names: "
            + ", ".join(unmapped_dispatch_variants)
        )
    orphan_dependency_features = sorted(
        observed.get("orphan_retained_dependency_features", set())
    )
    if orphan_dependency_features:
        failures.append(
            "retained direct dependency features without live codegen requirements: "
            + ", ".join(orphan_dependency_features)
        )

    for key, observed_values in observed.items():
        if key not in allowed:
            continue
        _compare_sets(failures, key, observed_values, allowed[key])

    return failures


def _permanent_file_failures(
    deleted_ownership_registry_exists: bool,
    arch_doc_text: str,
) -> list[str]:
    failures: list[str] = []
    if deleted_ownership_registry_exists:
        failures.append(
            "internal_docs/stdlib_native_surface_ownership.toml must remain deleted"
        )
    for phrase in STALE_ARCH_PHRASES:
        if phrase in arch_doc_text:
            failures.append(f"architecture doc contains stale deleted-registry phrase: {phrase!r}")
    return failures


def _direct_runtime_roots() -> set[str]:
    roots: set[str] = set()
    for path in CODEGEN_ROOT.rglob("*.rs"):
        relative = path.relative_to(CODEGEN_ROOT).as_posix()
        if _is_test_source(relative):
            continue
        text = path.read_text(encoding="utf-8")
        roots.update(
            f"sifr_runtime::{root}" for root in DIRECT_RUNTIME_ROOT_RE.findall(text)
        )
    return roots


def _is_test_source(relative_path: str) -> bool:
    name = relative_path.rsplit("/", 1)[-1]
    return name.endswith("_tests.rs") or "/tests/" in f"/{relative_path}/"


def _required_text(
    failures: list[str],
    table: dict[str, Any],
    key: str,
    context: str,
) -> str:
    value = table.get(key)
    if not isinstance(value, str) or not value.strip():
        failures.append(f"{context}: {key} must be a non-empty string")
        return ""
    return value.strip()


def _string_list(
    failures: list[str],
    table: dict[str, Any],
    key: str,
    context: str,
) -> list[str]:
    value = table.get(key, [])
    if not isinstance(value, list):
        failures.append(f"{context}: {key} must be a list")
        return []

    parsed: list[str] = []
    for item in value:
        if not isinstance(item, str) or not item.strip():
            failures.append(f"{context}: {key} entries must be non-empty strings")
            continue
        parsed.append(item.strip())
    return parsed


def _compare_sets(
    failures: list[str],
    key: str,
    observed_values: set[str],
    allowed_values: set[str],
) -> None:
    missing = sorted(observed_values - allowed_values)
    stale = sorted(allowed_values - observed_values)
    if missing:
        failures.append(f"{key} missing allowlist entries: {', '.join(missing)}")
    if stale:
        failures.append(f"{key} has stale allowlist entries: {', '.join(stale)}")


def _self_test() -> int:
    dependency_plan_fixture = r"""
        StdlibFeature::Alpha => &["alpha-dep = \"=1.0.0\""],
        StdlibFeature::Beta => &[
            "beta-dep = \"=1.0.0\"",
        ],
        StdlibFeature::Gamma => {
            &[
                "gamma-dep = \"=1.0.0\"",
            ]
        }
    """
    dependency_pairs = set(DEPENDENCY_FEATURE_RE.findall(dependency_plan_fixture))
    expected_dependency_pairs = {
        ("Alpha", "alpha-dep"),
        ("Beta", "beta-dep"),
        ("Gamma", "gamma-dep"),
    }
    if dependency_pairs != expected_dependency_pairs:
        print(
            "self-test retained dependency parser rejected rustfmt layouts",
            file=sys.stderr,
        )
        return 1

    observed = {
        "exact_intrinsics": {"alpha"},
        "dispatch_intrinsics": {"alpha"},
        "unmapped_dispatch_variants": set(),
        "source_declared_intrinsics": {"alpha"},
        "prefix_dispatchers": EXPECTED_PREFIX_DISPATCHERS,
        "registry_files": {"alpha.rs"},
        "preamble_files": {"runtime.rs"},
        "lowering_files": {"crates/sifr_lowering/src/alpha.rs"},
        "codegen_files": {"crates/sifr_codegen/src/alpha.rs"},
        "retained_direct_dependency_packages": {"alpha-dep"},
        "retained_direct_dependency_features": {"alpha-dep=alpha-feature"},
        "orphan_retained_dependency_features": set(),
        "direct_runtime_roots": {"sifr_runtime::alpha"},
    }
    allowlist = {
        "surface": [
            {
                "id": "_sifr.alpha",
                "state": "retained-by-design",
                "reason": "language-owned test fixture",
                "declaration_files": ["stdlib/_sifr/alpha.sifr"],
                "exact_intrinsics": ["alpha"],
                "source_declared_intrinsics": ["alpha"],
                "registry_files": ["alpha.rs"],
                "preamble_files": ["runtime.rs"],
                "lowering_files": ["crates/sifr_lowering/src/alpha.rs"],
                "codegen_files": ["crates/sifr_codegen/src/alpha.rs"],
                "retained_direct_dependency_packages": ["alpha-dep"],
                "retained_direct_dependency_features": ["alpha-dep=alpha-feature"],
                "direct_runtime_roots": ["sifr_runtime::alpha"],
            }
        ]
    }
    if _validate(observed, allowlist):
        print("self-test seed should pass", file=sys.stderr)
        return 1

    missing = json.loads(json.dumps(allowlist))
    missing["surface"][0]["exact_intrinsics"] = []
    if not any(
        "exact_intrinsics missing allowlist entries: alpha" in failure
        for failure in _validate(observed, missing)
    ):
        print("self-test missing exact intrinsic was not rejected", file=sys.stderr)
        return 1

    stale = json.loads(json.dumps(allowlist))
    stale["surface"][0]["registry_files"].append("stale.rs")
    if not any(
        "registry_files has stale allowlist entries: stale.rs" in failure
        for failure in _validate(observed, stale)
    ):
        print("self-test stale registry file was not rejected", file=sys.stderr)
        return 1

    missing_source_declaration = json.loads(json.dumps(allowlist))
    missing_source_declaration["surface"][0]["source_declared_intrinsics"] = []
    if not any(
        "source_declared_intrinsics missing allowlist entries: alpha" in failure
        for failure in _validate(observed, missing_source_declaration)
    ):
        print("self-test missing source intrinsic declaration was not rejected", file=sys.stderr)
        return 1

    missing_lowering_owner = json.loads(json.dumps(allowlist))
    missing_lowering_owner["surface"][0]["lowering_files"] = []
    if not any(
        "lowering_files missing allowlist entries" in failure
        for failure in _validate(observed, missing_lowering_owner)
    ):
        print("self-test missing lowering ownership was not rejected", file=sys.stderr)
        return 1

    missing_codegen_owner = json.loads(json.dumps(allowlist))
    missing_codegen_owner["surface"][0]["codegen_files"] = []
    if not any(
        "codegen_files missing allowlist entries" in failure
        for failure in _validate(observed, missing_codegen_owner)
    ):
        print("self-test missing codegen ownership was not rejected", file=sys.stderr)
        return 1

    missing_dispatch = {key: set(value) for key, value in observed.items()}
    missing_dispatch["dispatch_intrinsics"] = set()
    if not any(
        "typed dispatch intrinsics has stale allowlist entries: alpha" in failure
        for failure in _validate(missing_dispatch, allowlist)
    ):
        print("self-test missing typed dispatch implementation was not rejected", file=sys.stderr)
        return 1

    missing_dependency = json.loads(json.dumps(allowlist))
    missing_dependency["surface"][0]["retained_direct_dependency_packages"] = []
    if not any(
        "retained_direct_dependency_packages missing allowlist entries: alpha-dep" in failure
        for failure in _validate(observed, missing_dependency)
    ):
        print("self-test missing retained direct dependency was not rejected", file=sys.stderr)
        return 1

    orphan_dependency = {key: set(value) for key, value in observed.items()}
    orphan_dependency["orphan_retained_dependency_features"] = {"AlphaFeature"}
    if not any(
        "without live codegen requirements: AlphaFeature" in failure
        for failure in _validate(orphan_dependency, allowlist)
    ):
        print("self-test orphan retained dependency feature was not rejected", file=sys.stderr)
        return 1

    missing_runtime_root = json.loads(json.dumps(allowlist))
    missing_runtime_root["surface"][0]["direct_runtime_roots"] = []
    if not any(
        "direct_runtime_roots missing allowlist entries: sifr_runtime::alpha" in failure
        for failure in _validate(observed, missing_runtime_root)
    ):
        print("self-test missing direct runtime root was not rejected", file=sys.stderr)
        return 1

    duplicate = json.loads(json.dumps(allowlist))
    duplicate["surface"].append(
        {
            "id": "duplicate-owner",
            "reason": "duplicate test fixture",
            "exact_intrinsics": ["alpha"],
        }
    )
    if not any("is duplicated" in failure for failure in _validate(observed, duplicate)):
        print("self-test duplicate allowlist entry was not rejected", file=sys.stderr)
        return 1

    new_prefix_observed = json.loads(
        json.dumps({key: sorted(value) for key, value in observed.items()})
    )
    new_prefix_observed["prefix_dispatchers"].append("s3_")
    new_prefix_observed = {key: set(value) for key, value in new_prefix_observed.items()}
    if not any(
        "untracked prefix dispatchers: s3_" in failure
        for failure in _validate(new_prefix_observed, allowlist)
    ):
        print("self-test untracked prefix dispatcher was not rejected", file=sys.stderr)
        return 1

    missing_reason = json.loads(json.dumps(allowlist))
    missing_reason["surface"][0]["reason"] = ""
    if not any(
        "reason must be a non-empty string" in failure
        for failure in _validate(observed, missing_reason)
    ):
        print("self-test missing reason was not rejected", file=sys.stderr)
        return 1

    bad_state = json.loads(json.dumps(allowlist))
    bad_state["surface"][0]["state"] = "closing"
    if not any(
        "_sifr.alpha: state must be retained-by-design" in failure
        for failure in _validate(observed, bad_state)
    ):
        print("self-test bad state was not rejected", file=sys.stderr)
        return 1

    design_without_observed_items = json.loads(json.dumps(allowlist))
    for key in (
        "exact_intrinsics",
        "source_declared_intrinsics",
        "registry_files",
        "preamble_files",
        "lowering_files",
        "codegen_files",
        "retained_direct_dependency_packages",
        "retained_direct_dependency_features",
        "direct_runtime_roots",
    ):
        design_without_observed_items["surface"][0].pop(key, None)
    design_without_observed_items["surface"][0]["declaration_files"] = [
        "stdlib/_sifr/alpha.sifr"
    ]
    observed_without_surface = {
        key: set(value) for key, value in observed.items()
    }
    for key in (
        "exact_intrinsics",
        "source_declared_intrinsics",
        "registry_files",
        "preamble_files",
        "lowering_files",
        "codegen_files",
        "retained_direct_dependency_packages",
        "retained_direct_dependency_features",
        "direct_runtime_roots",
    ):
        observed_without_surface[key] = set()
    if not any(
        "_sifr.alpha: allowlist entry has no retained files or intrinsics" in failure
        for failure in _validate(observed_without_surface, design_without_observed_items)
    ):
        print("self-test metadata-only retained-by-design row was not rejected", file=sys.stderr)
        return 1

    stale_phrase = STALE_ARCH_PHRASES[0]
    if not any(
        "stale deleted-registry phrase" in failure
        for failure in _permanent_file_failures(False, stale_phrase)
    ):
        print("self-test stale architecture phrase was not rejected", file=sys.stderr)
        return 1

    if _permanent_file_failures(True, "") != [
        "internal_docs/stdlib_native_surface_ownership.toml must remain deleted"
    ]:
        print("self-test restored ownership registry was not rejected", file=sys.stderr)
        return 1

    self_test_root = REPO_ROOT / "target"
    self_test_root.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(dir=self_test_root) as tmp:
        fixture_root = Path(tmp)
        restored_path = fixture_root / "deleted-crate"
        restored_path.mkdir()
        token_paths = []
        for index, token in enumerate(DELETED_FALLBACK_TOKENS):
            token_path = fixture_root / f"deleted-{index}.toml"
            token_path.write_text(f"value = {token!r}\n", encoding="utf-8")
            token_paths.append(token_path)
        fallback_failures = _deleted_fallback_architecture_failures(
            (restored_path,), tuple(token_paths)
        )
    if not any("deleted fallback architecture path remains" in failure for failure in fallback_failures):
        print("self-test restored fallback path was not rejected", file=sys.stderr)
        return 1
    for token in DELETED_FALLBACK_TOKENS:
        if not any(token in failure for failure in fallback_failures):
            print(f"self-test restored fallback token {token!r} was not rejected", file=sys.stderr)
            return 1

    print("stdlib native intrinsic allowlist guard self-test: PASS")
    return 0


if __name__ == "__main__":
    if sys.argv[1:] == ["--self-test"]:
        raise SystemExit(_self_test())
    raise SystemExit(main())
