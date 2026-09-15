"""Locked, workspace-excluded package graphs consumed by offline crate tests."""

from __future__ import annotations

import hashlib
from pathlib import Path
from typing import Any, Callable

from .paths import REPO_ROOT
from .profiles import crate_test_mode, crate_test_suites_for_mode

# These are complete Cargo package fixtures, not intentionally unresolvable
# manifest negatives. Their owning crate tests require successful offline
# metadata before checking the intended Sifr-language diagnostic.
OFFLINE_CRATE_TEST_FIXTURES = {
    "sifr_driver": (
        "verification/areas/core_language/fixtures/static_class_adapter/negative/"
        "non_string_leaf_rejected/Cargo.toml",
    ),
}


def locked_fixture_manifests(profile: dict[str, Any]) -> list[Path]:
    mode = crate_test_mode(profile)
    if mode is None:
        return []
    packages = {suite["package"] for suite in crate_test_suites_for_mode(profile, mode)}
    return sorted({
        REPO_ROOT / manifest
        for package in packages
        for manifest in OFFLINE_CRATE_TEST_FIXTURES.get(package, ())
    })


def fixture_graph_hashes(manifest: Path) -> dict[str, str]:
    return {
        path.name: hashlib.sha256(path.read_bytes()).hexdigest()
        for path in (manifest, manifest.with_name("Cargo.lock"))
    }


def prepare_locked_fixture_caches(
    profile: dict[str, Any],
    env: dict[str, str],
    command_runner: Callable[..., None],
) -> list[dict[str, str]]:
    records = []
    for manifest in locked_fixture_manifests(profile):
        before = fixture_graph_hashes(manifest)
        command_runner(
            ["cargo", "fetch", "--locked", "--manifest-path", str(manifest)],
            env=env,
        )
        if fixture_graph_hashes(manifest) != before:
            raise ValueError(f"locked fixture preparation changed manifest or lock: {manifest}")
        records.append({"manifest": str(manifest.relative_to(REPO_ROOT)), **before})
        print(f"[sifr-profile-setup] fixture={manifest.relative_to(REPO_ROOT)} status=pass", flush=True)
    return records
