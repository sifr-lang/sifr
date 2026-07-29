"""Plan-digest sensitivity checks for stable qualification fixtures."""

from __future__ import annotations

import hashlib
import json
import shutil
import subprocess
import tempfile
from collections.abc import Callable
from pathlib import Path
from typing import Any

from .qualification_fixture import build_evidence_bundle, create_fixture_source


def run_plan_digest_sensitivity_test(
    run_planner: Callable[[dict[str, Any], Path], bytes],
) -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-plan-sensitivity-") as directory:
        root = Path(directory)
        baseline_root = root / "baseline"
        source_root = create_fixture_source(baseline_root)
        result_root = source_root / "target" / "verification" / "sensitivity-results"
        bundle = build_evidence_bundle(
            source_root=source_root,
            evidence_root=baseline_root / "evidence",
            result_root=result_root,
        )
        baseline = run_planner(bundle, baseline_root / "plan.json")
        baseline_plan = json.loads(baseline)
        baseline_digest = hashlib.sha256(baseline).hexdigest()
        fresh_control_root = root / "fresh-control"
        fresh_control_source = create_fixture_source(
            fresh_control_root,
            variant="nochange",
        )
        if (
            subprocess.run(
                ["git", "rev-parse", "HEAD"],
                cwd=fresh_control_source,
                check=True,
                text=True,
                stdout=subprocess.PIPE,
            ).stdout.strip()
            != bundle["source_ref"]
        ):
            raise AssertionError("identical fixture sources have unstable commit ids")
        fresh_control_bundle = build_evidence_bundle(
            source_root=fresh_control_source,
            evidence_root=fresh_control_root / "evidence",
            result_root=(
                fresh_control_source / "target" / "verification" / "sensitivity-results"
            ),
            variant="nochange",
        )
        fresh_control = run_planner(
            fresh_control_bundle,
            fresh_control_root / "plan.json",
        )
        if hashlib.sha256(fresh_control).hexdigest() != baseline_digest:
            raise AssertionError("identical fresh fixture changed the plan digest")
        for variant in (
            "nochange",
            "target-artifact",
            "sysroot",
            "vsix",
        ):
            variant_root = root / variant
            shutil.rmtree(result_root, ignore_errors=True)
            variant_bundle = build_evidence_bundle(
                source_root=source_root,
                evidence_root=variant_root / "evidence",
                result_root=result_root,
                variant=variant,
            )
            changed = run_planner(variant_bundle, variant_root / "plan.json")
            changed_digest = hashlib.sha256(changed).hexdigest()
            if variant == "nochange" and changed_digest != baseline_digest:
                raise AssertionError("no-op planner inputs changed the plan digest")
            if variant != "nochange" and changed_digest == baseline_digest:
                raise AssertionError(
                    f"planner input variant {variant} did not change the plan digest"
                )
        for variant in ("source", "submodule", "lock", "rust-claims"):
            variant_root = root / variant
            variant_source = create_fixture_source(variant_root, variant=variant)
            variant_bundle = build_evidence_bundle(
                source_root=variant_source,
                evidence_root=variant_root / "evidence",
                result_root=(
                    variant_source / "target" / "verification" / "fixture-results"
                ),
                variant=variant,
            )
            changed = run_planner(variant_bundle, variant_root / "plan.json")
            if hashlib.sha256(changed).hexdigest() == baseline_digest:
                raise AssertionError(
                    f"planner input variant {variant} did not change the plan digest"
                )
            if variant == "rust-claims":
                changed_plan = json.loads(changed)
                baseline_rust = baseline_plan["rust_interop"]
                changed_rust = changed_plan["rust_interop"]
                if (
                    changed_rust["stable_support_claims_sha256"]
                    == baseline_rust["stable_support_claims_sha256"]
                    or changed_rust["advertised_claim_ids"]
                    == baseline_rust["advertised_claim_ids"]
                ):
                    raise AssertionError(
                        "Rust-claim source change did not alter its plan bindings"
                    )
