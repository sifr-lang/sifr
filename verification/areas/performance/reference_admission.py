#!/usr/bin/env python3
"""Admit a selected performance reference before expensive validation setup."""

from __future__ import annotations

import argparse
import json
import os
import sys
from datetime import date
from pathlib import Path
from typing import Any, Callable

from check_budgets import BudgetError, DEFAULT_WAIVERS, check_reference_policy
from check_trend_policy import DEFAULT_POLICY, TrendPolicyError, validate_trend_policy
from reference_host import reference_identity
from reference_profiles import (
    ReferenceProfileError,
    assert_comparable,
    load_profile,
    profile_digest,
    validate_manifest_binding,
)

REPO_ROOT = Path(__file__).resolve().parents[3]
MANIFEST = Path(__file__).resolve().parent / "data" / "benchmark_manifest.json"


def admit_reference(
    name: str,
    *,
    repo_root: Path = REPO_ROOT,
    manifest_path: Path = MANIFEST,
    policy_path: Path = DEFAULT_POLICY,
    waivers_path: Path = DEFAULT_WAIVERS,
    today: date | None = None,
    now_unix: int | None = None,
    identity_fn: Callable[[Path, Path, str], dict[str, Any]] = reference_identity,
) -> dict[str, Any]:
    if not name:
        raise ReferenceProfileError(
            "select SIFR_PERFORMANCE_REFERENCE before validation setup"
        )
    profile = load_profile(name)
    validate_manifest_binding(profile, manifest_path)
    manifest = json.loads(manifest_path.read_text())
    policy = json.loads(policy_path.read_text())
    waivers = json.loads(waivers_path.read_text())
    validate_trend_policy(
        manifest, profile["baseline"], policy,
        manifest_path=manifest_path, today=today, now_unix=now_unix,
    )
    check_reference_policy(manifest, profile, waivers)
    # The producer uses the selected reference's Cargo concurrency. Admit the
    # same execution configuration instead of the outer profile's worker count.
    expected_jobs = profile["identity"]["execution"]["cargo_jobs"]
    old_jobs = os.environ.get("CARGO_BUILD_JOBS")
    os.environ["CARGO_BUILD_JOBS"] = expected_jobs
    try:
        observed = identity_fn(repo_root, manifest_path, profile["identity"]["execution"]["control_mode"])
    finally:
        if old_jobs is None:
            os.environ.pop("CARGO_BUILD_JOBS", None)
        else:
            os.environ["CARGO_BUILD_JOBS"] = old_jobs
    assert_comparable(profile, observed)
    return profile


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reference-profile", default=os.environ.get("SIFR_PERFORMANCE_REFERENCE", ""))
    args = parser.parse_args()
    try:
        profile = admit_reference(args.reference_profile)
    except (ReferenceProfileError, TrendPolicyError, BudgetError, OSError, ValueError, KeyError) as error:
        print(f"performance qualification unavailable: {error}", file=sys.stderr)
        return 1
    print(f"performance reference admitted: {profile['name']} sha256={profile_digest(profile)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
