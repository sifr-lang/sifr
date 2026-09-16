"""DX.1 lane and prospective budget-policy failure injection."""
from copy import deepcopy
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch
import json
import subprocess

import compiler_lanes
from benchmark_manifest import BenchmarkError
from reference_host import comparison_mismatches
from reference_profiles import load_profile


def rejected(call, label):
    try:
        call()
    except (ValueError, BenchmarkError):
        return
    raise AssertionError(f"{label} was incorrectly accepted")


def run_self_test():
    root = Path(__file__).resolve().parent
    contract = json.loads((root / "data/dx_contract.json").read_text())
    budgets = json.loads((root / "data/budgets.json").read_text())
    assert {r["budget_id"] for r in contract["legacy_budget_mapping"]} == {r["budget_id"] for r in budgets["budgets"]}
    assert contract["historical_budgets_sha256"] == compiler_lanes.digest(root / "data/budgets.json")
    assert (root.parents[2] / contract["q09"]["source"]).is_file()
    rejected(lambda: compiler_lanes.configure(Path("."), "product-installed-optimized", ""), "missing product artifact")
    with TemporaryDirectory() as directory:
        root = Path(directory)
        binary = root / "sifr"
        binary.write_text("#!/bin/sh\nexit 0\n")
        binary.chmod(0o755)
        (root / "Cargo.lock").write_text("fixed")
        receipt = {
            "schema_version": 1, "lane": "contributor-dev",
            "compiler_build_profile": "dev",
            "cargo_artifact_profile": {"test": False, "opt_level": "1", "debug_assertions": True},
            "artifact": {"path": str(binary), "sha256": compiler_lanes.digest(binary)},
            "source_commit": "a"*40, "cargo_lock_sha256": compiler_lanes.digest(root / "Cargo.lock"),
            "rustc": "rustc fixed", "sysroot": {"path": str(root), "kind": "source-tree"},
        }
        def output(argv, **kwargs):
            return "a"*40 if argv[0] == "git" else "rustc fixed"
        with patch("compiler_lanes.subprocess.check_output", side_effect=output):
            compiler_lanes.validate_receipt(root, "contributor-dev", receipt)
            for key, value in (("lane", "product-installed-optimized"), ("compiler_build_profile", "release")):
                bad = deepcopy(receipt)
                bad[key] = value
                rejected(lambda: compiler_lanes.validate_receipt(root, "contributor-dev", bad), key)
            product = deepcopy(receipt)
            product.update(lane="product-installed-optimized", compiler_build_profile="release")
            rejected(lambda: compiler_lanes.validate_receipt(root, "product-installed-optimized", product), "dev bytes mislabeled product")
            binary.write_text("changed")
            rejected(lambda: compiler_lanes.validate_receipt(root, "contributor-dev", receipt), "stale binary")

    identity = load_profile("linux-i7-4720hq-12gb-dev-v1")["identity"]
    for section, key, value in (
        ("host", "system", "Darwin"), ("execution", "build_profile", "release"),
        ("execution", "compiler_measurement_lane", "product-installed-optimized"),
    ):
        bad = deepcopy(identity)
        bad[section][key] = value
        if not comparison_mismatches(identity, bad):
            raise AssertionError(f"Q08 accepted cross-lane {section}.{key}")
    comparison = {"lane": "contributor-dev", "compiler_build_profile": "dev",
                  "application_profile": "release", "verification_selection": "same"}
    compiler_lanes.assert_comparable(comparison, comparison)
    for key in comparison:
        bad = comparison | {key: "mislabeled"}
        rejected(lambda: compiler_lanes.assert_comparable(comparison, bad), key)
    # Exercise the existing numerical checker and its same-lane negative seeds;
    # do not replace the enforcement with a lane-label-only assertion.
    # The public check_budgets self-test owns the dependency injection contract.
    root = Path(__file__).resolve().parent
    subprocess.run(["python3", str(root / "check_budgets.py"), "--self-test"], check=True)
    print("DX.1 Q07/Q08 lane negative controls passed")


if __name__ == "__main__":
    run_self_test()
