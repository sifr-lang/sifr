#!/usr/bin/env python3
"""Ensure retained compiler-native stdlib glue is explicitly allowlisted."""

from __future__ import annotations

import json
import re
import sys
import tomllib
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[1]
ALLOWLIST_PATH = REPO_ROOT / "internal_docs" / "stdlib_retained_compiler_intrinsics.toml"
REGISTRY_ROOT = REPO_ROOT / "crates" / "sifr_codegen" / "src" / "intrinsics" / "registry"
REGISTRY_DISPATCH_PATH = (
    REPO_ROOT / "crates" / "sifr_codegen" / "src" / "intrinsics" / "registry.rs"
)
PREAMBLE_ROOT = REPO_ROOT / "crates" / "sifr_codegen" / "src" / "preamble"
RETAINED_INTRINSICS_LIB_PATH = REPO_ROOT / "crates" / "sifr_retained_intrinsics" / "src" / "lib.rs"
CODEGEN_ROOT = REPO_ROOT / "crates" / "sifr_codegen" / "src"
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

EXACT_INTRINSIC_RE = re.compile(r'"([A-Za-z0-9_]+)"\s*(?=\||=>)')
LOWERER_MATCH_INTRINSIC_RE = re.compile(r'"([A-Za-z0-9_]+)"\s*(?=\||=>)')
PREFIX_INTRINSIC_RE = re.compile(r'starts_with\("([A-Za-z0-9_]+)"\)')
RETAINED_SIGNATURE_MODULE_RE = re.compile(r'"(_sifr\.[A-Za-z0-9_]+)"\s*=>\s*Some\(')
GENERATED_DEPENDENCY_PACKAGE_RE = re.compile(r'"([A-Za-z0-9_-]+)\s+=')
DIRECT_RUNTIME_ROOT_RE = re.compile(r"\bsifr_runtime::([A-Za-z_][A-Za-z0-9_]*)")
EXPECTED_PREFIX_DISPATCHERS: set[str] = set()
PREFIX_DISPATCH_LOWERERS: tuple[Path, ...] = ()
STALE_ARCH_PHRASES = (
    "complete surface-by-surface ownership decision remains the TOML registry",
    "validated against the compiler intrinsic registry; compiler intrinsic metadata remains the current signature owner",
    "old handwritten intrinsic registry is removed or reduced",
)


def main() -> int:
    observed = _observed_surface()
    allowlist = tomllib.loads(ALLOWLIST_PATH.read_text(encoding="utf-8"))
    return _run(observed, allowlist)


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
        f"fallback_signature_modules={len(observed['fallback_signature_modules'])}, "
        "retained_direct_dependency_packages="
        f"{len(observed['retained_direct_dependency_packages'])}, "
        f"direct_runtime_roots={len(observed['direct_runtime_roots'])})"
    )
    return 0


def _observed_surface() -> dict[str, set[str]]:
    registry_text = REGISTRY_DISPATCH_PATH.read_text(encoding="utf-8")
    exact_intrinsics = set(EXACT_INTRINSIC_RE.findall(registry_text))
    for lowerer_path in PREFIX_DISPATCH_LOWERERS:
        exact_intrinsics.update(
            LOWERER_MATCH_INTRINSIC_RE.findall(lowerer_path.read_text(encoding="utf-8"))
        )
    return {
        "exact_intrinsics": exact_intrinsics,
        "prefix_dispatchers": set(PREFIX_INTRINSIC_RE.findall(registry_text)),
        "registry_files": {
            path.relative_to(REGISTRY_ROOT).as_posix()
            for path in REGISTRY_ROOT.rglob("*.rs")
        },
        "preamble_files": {
            path.relative_to(PREAMBLE_ROOT).as_posix()
            for path in PREAMBLE_ROOT.rglob("*.rs")
        },
        "fallback_signature_modules": set(
            RETAINED_SIGNATURE_MODULE_RE.findall(
                RETAINED_INTRINSICS_LIB_PATH.read_text(encoding="utf-8")
            )
        ),
        "retained_direct_dependency_packages": {
            package
            for package in GENERATED_DEPENDENCY_PACKAGE_RE.findall(
                _non_test_dependency_plan_text()
            )
        },
        "direct_runtime_roots": _direct_runtime_roots(),
    }


def _non_test_dependency_plan_text() -> str:
    text = DEPENDENCY_PLAN_RS_PATH.read_text(encoding="utf-8")
    test_marker = "\n#[cfg(test)]"
    return text.split(test_marker, 1)[0]


def _validate(observed: dict[str, set[str]], allowlist: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    allowed = {
        "exact_intrinsics": set[str](),
        "registry_files": set[str](),
        "preamble_files": set[str](),
        "fallback_signature_modules": set[str](),
        "retained_direct_dependency_packages": set[str](),
        "direct_runtime_roots": set[str](),
    }
    owners: dict[tuple[str, str], str] = {}
    unique_owner_keys = {
        "exact_intrinsics",
        "registry_files",
        "preamble_files",
        "fallback_signature_modules",
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
    observed = {
        "exact_intrinsics": {"alpha"},
        "prefix_dispatchers": EXPECTED_PREFIX_DISPATCHERS,
        "registry_files": {"alpha.rs"},
        "preamble_files": {"runtime.rs"},
        "fallback_signature_modules": {"_sifr.alpha"},
        "retained_direct_dependency_packages": {"alpha-dep"},
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
                "registry_files": ["alpha.rs"],
                "preamble_files": ["runtime.rs"],
                "fallback_signature_modules": ["_sifr.alpha"],
                "retained_direct_dependency_packages": ["alpha-dep"],
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

    missing_dependency = json.loads(json.dumps(allowlist))
    missing_dependency["surface"][0]["retained_direct_dependency_packages"] = []
    if not any(
        "retained_direct_dependency_packages missing allowlist entries: alpha-dep" in failure
        for failure in _validate(observed, missing_dependency)
    ):
        print("self-test missing retained direct dependency was not rejected", file=sys.stderr)
        return 1

    missing_runtime_root = json.loads(json.dumps(allowlist))
    missing_runtime_root["surface"][0]["direct_runtime_roots"] = []
    if not any(
        "direct_runtime_roots missing allowlist entries: sifr_runtime::alpha" in failure
        for failure in _validate(observed, missing_runtime_root)
    ):
        print("self-test missing direct runtime root was not rejected", file=sys.stderr)
        return 1

    missing_fallback_module = json.loads(json.dumps(allowlist))
    missing_fallback_module["surface"][0]["fallback_signature_modules"] = []
    if not any(
        "fallback_signature_modules missing allowlist entries: _sifr.alpha" in failure
        for failure in _validate(observed, missing_fallback_module)
    ):
        print("self-test missing fallback signature module was not rejected", file=sys.stderr)
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
        "registry_files",
        "preamble_files",
        "fallback_signature_modules",
        "retained_direct_dependency_packages",
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
        "registry_files",
        "preamble_files",
        "fallback_signature_modules",
        "retained_direct_dependency_packages",
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

    print("stdlib native intrinsic allowlist guard self-test: PASS")
    return 0


if __name__ == "__main__":
    if sys.argv[1:] == ["--self-test"]:
        raise SystemExit(_self_test())
    raise SystemExit(main())
