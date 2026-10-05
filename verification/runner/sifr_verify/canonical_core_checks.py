"""Registered feedback inventory and preparation keep their distinct claims."""
import contextlib
import io
import json
import unittest
from unittest.mock import Mock

from .area_cargo_setup import sql_preparation_commands
from .cargo_cli_command import ordinary_cli_build_command
from .cargo_setup import prepare_performance_binaries
from .early_sql import run_early_sql
from .paths import REPO_ROOT
from .profiles import build_profile_plan, load_profile


class CanonicalCoreChecks(unittest.TestCase):
    def test_registered_core_retains_actual_smoke_inventory(self):
        profile = load_profile("create-pr")
        total = 0
        sql = None
        for row in profile["selected_areas"]:
            manifest = json.loads((REPO_ROOT / "verification/areas" / row["area"] / "manifest.json").read_text())
            selected = [suite for suite in manifest["suites"] if suite["name"] in row["suites"]]
            total += sum(len(suite["cases"]) for suite in selected)
            if row["area"] == "sql_platform":
                sql = selected
        self.assertEqual(total, 96)
        self.assertEqual((len(sql), sum(len(suite["cases"]) for suite in sql)), (7, 22))
        self.assertEqual(len(profile["guardrail_steps"]), 15)
        self.assertEqual(profile["toolchain_steps"], ["cargo-test-sifr-smoke", "e2e-pass"])
        self.assertEqual(build_profile_plan("create-pr")["e2e"]["fixture_count"], 143)

    def test_core_does_not_schedule_clean_build_or_lose_ordinary_cli(self):
        profile = load_profile("create-pr")
        suites = next(row["suites"] for row in profile["selected_areas"] if row["area"] == "sql_platform")
        commands = sql_preparation_commands(suites)
        self.assertEqual(commands, [
            ["cargo", "test", "--no-run", "--locked", "-p", "sifr_sql_contract", "--test", "common_sql_contracts"],
            ["cargo", "test", "--no-run", "--locked", "-p", "sifr_sql_runtime"],
            ["cargo", "test", "--no-run", "--locked", "-p", "sifr_package", "host_tool"],
            ["cargo", "test", "--no-run", "--locked", "-p", "sifr", "--test", "host_tool_cli"],
        ])
        runner, schedule = Mock(profile=profile), Mock()
        self.assertFalse(run_early_sql(runner, schedule).selected)
        self.assertEqual(runner.mock_calls, [])
        self.assertEqual(schedule.mock_calls, [])
        prepared = []
        with contextlib.redirect_stdout(io.StringIO()):
            prepare_performance_binaries(profile, {}, lambda command, **kw: prepared.append(command))
        # LSP/generated consumers still need the ordinary CLI after SQL's
        # incremental-editor setup leaves the feedback profile.
        self.assertIn(ordinary_cli_build_command(), prepared)

    def test_full_profiles_still_include_specialist_qualification(self):
        for name in ("merge", "cloud", "nightly", "release"):
            profile = load_profile(name)
            suites = next(row["suites"] for row in profile["selected_areas"] if row["area"] == "sql_platform")
            self.assertEqual(len(suites), 19)
            self.assertIn("build-qualification", suites)
            self.assertIn("schema-polymorphism", suites)
            self.assertEqual(len(sql_preparation_commands(suites)), 31)


def policy_checks():
    result = unittest.TestResult()
    unittest.defaultTestLoader.loadTestsFromTestCase(CanonicalCoreChecks).run(result)
    if not result.wasSuccessful():
        raise AssertionError(result.errors + result.failures)
