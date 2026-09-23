"""Residual emitted-Rust qualification profile contracts."""

from copy import deepcopy
from dataclasses import replace
import json
from pathlib import Path
from tempfile import TemporaryDirectory

from .profiles import (
    ProfileError,
    load_profile,
    required_sql_platform_suites,
    validate_selected_area_suites,
)
from .step_budgets import StepBudgetContext, record_step_success, required_cache_paths, selected_area_id, selected_suites


def policy_checks() -> None:
    for name in ("create-pr", "merge"):
        profile = load_profile(name)
        required = required_sql_platform_suites()
        for missing in sorted(required):
            changed = deepcopy(profile)
            for area in changed["selected_areas"]:
                if area["area"] == "sql_platform":
                    area["suites"] = [suite for suite in area["suites"] if suite != missing]
            try:
                validate_selected_area_suites(changed)
            except ProfileError as error:
                if "required SQL platform" not in str(error) or missing not in str(error):
                    raise
            else:
                raise AssertionError(f"{name} accepted missing SQL suite {missing}")
        for area in ("runtime_platform", "python_interop"):
            direct = selected_suites(profile, area)
            canonical = selected_area_id(f"area_{area}")
            if not direct or canonical != area or selected_suites(profile, None):
                raise AssertionError(f"budget fingerprint lost {area} suite selection")

    runtime_budget = load_profile("create-pr")["step_budgets"]["area_runtime_platform"]
    if runtime_budget["warm_budget_ms"] != 120_000 or runtime_budget["cold_budget_ms"] != 600_000:
        raise AssertionError("runtime warm and cold qualification budgets changed")

    root = Path("/qualification")
    binary = root / "target/debug/sifr"
    expected = (binary, root / "custom-target/debug")
    actual = required_cache_paths(root, "runtime_platform", binary,
                                  {"CARGO_TARGET_DIR": "custom-target"})
    if actual != expected:
        raise AssertionError("runtime receipt does not bind its effective Cargo target")

    with TemporaryDirectory(prefix="sifr-qualification-receipts-") as directory:
        receipt = Path(directory) / "receipt.json"
        context = StepBudgetContext(
            name="area_runtime_platform", budget_ms=600_000, enforcement="blocking",
            cache_state="cold", cache_fingerprint="first", receipt_path=receipt,
            receipt_eligible=True,
        )
        record_step_success(context, 240_000)
        record_step_success(replace(context, cache_state="warm"), 24_000)
        observed = json.loads(receipt.read_text())["observations"]
        if observed != [
            {"cache_state": "cold", "elapsed_ms": 240_000},
            {"cache_state": "warm", "elapsed_ms": 24_000},
        ]:
            raise AssertionError("cold and warm timings were not retained separately")
        for elapsed in range(5):
            record_step_success(replace(context, cache_state="warm"), elapsed)
        if len(json.loads(receipt.read_text())["observations"]) != 4:
            raise AssertionError("timing receipt retention is unbounded")
        record_step_success(replace(context, cache_fingerprint="changed"), 300_000)
        if len(json.loads(receipt.read_text())["observations"]) != 1:
            raise AssertionError("timings leaked across validation input changes")
