"""Coverage preservation and evidence separation for local correctness execution."""
from __future__ import annotations

import contextlib
import io
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from . import profile_reporting, reports
from .compiler_configuration_plan import configuration_plan
from .errors import VerificationError
from .frozen_crate_scope_checks import FrozenCrateCoverageTests, assert_frozen_crate_coverage
from .execution_scope import select_execution_scope
from .paths import REPO_ROOT
from .profile_runner import ProfileRunner, run_profile
from .profiles import build_profile_plan, compare_plans, load_profile, run_command


class ExecutionScopeTests(unittest.TestCase):
    def test_only_explicit_benchmark_selection_is_removed(self):
        for name, excluded in (("create-pr", "smoke"), ("merge", "representative")):
            full = load_profile(name)
            selected = select_execution_scope(full, True)
            for key in full.keys() - {"selected_areas"}:
                self.assertEqual(selected[key], full[key], key)
            before = [(a["area"], s) for a in full["selected_areas"] for s in a["suites"]]
            after = [(a["area"], s) for a in selected["selected_areas"] for s in a["suites"]]
            self.assertEqual(after, [x for x in before if x != ("performance", excluded)])
            self.assertEqual(configuration_plan(full, "full"), configuration_plan(selected, "full"))
            selected["e2e"]["fixture_manifest"] = "mutated"
            self.assertEqual(full, load_profile(name), "selection must not mutate canonical profiles")

    def test_frozen_correctness_scope_is_retained(self):
        frozen = json.loads((REPO_ROOT / "plans/issues/active/local-correctness-gate-evidence/scope.json").read_text())
        expected = {(x["area"], x["suite"]) for x in frozen["area_suites"]}
        plan = build_profile_plan("merge", correctness_only=True)
        actual = {(a["area"], s) for a in plan["selected_areas"] for s in a["suites"]}
        self.assertEqual(actual, expected)
        self.assertEqual(len(actual), 105)
        expected_guardrails = list(frozen["guardrails"])
        expected_guardrails.insert(
            expected_guardrails.index("stdlib-manifest-schema") + 1,
            "dx-metadata-inventory",
        )
        expected_guardrails.remove("demo-emitted-freshness")
        self.assertEqual(plan["guardrail_steps"], expected_guardrails)
        quality = next(a for a in plan["selected_areas"] if a["area"] == "generated_code_quality")
        self.assertIs(quality["require_fresh_demo_companions"], True)
        assert_frozen_crate_coverage(
            frozen["crate_configurations"],
            configuration_plan(select_execution_scope(load_profile("merge"), True), "full"),
        )
        self.assertEqual(plan["e2e"]["fixture_selection"], "full-corpus")
        self.assertEqual(plan["excluded_area_suites"], [
            {"area": "performance", "suite": "representative", "reason": "benchmark capture"}])

    def test_future_correctness_suites_and_policy_checks_remain_selected(self):
        full = load_profile("merge")
        area = next(a for a in full["selected_areas"] if a["area"] == "performance")
        area["suites"].extend(["rules", "future-correctness-check"])
        selected = select_execution_scope(full, True)
        suites = next(a["suites"] for a in selected["selected_areas"] if a["area"] == "performance")
        self.assertIn("rules", suites)
        self.assertIn("future-correctness-check", suites)

    def test_nonlocal_profiles_and_release_evidence_reject_scope(self):
        for name in ("nightly", "release", "python-interop-live"):
            with self.assertRaisesRegex(VerificationError, "supports create-pr and merge only"):
                build_profile_plan(name, correctness_only=True)
        with self.assertRaisesRegex(VerificationError, "cannot produce release evidence"):
            run_profile("merge", [], correctness_only=True, release_report_out="must-not-exist.json")

    def test_scoped_runner_executes_every_retained_step_without_benchmark_admission(self):
        runner = ProfileRunner("merge", ["--no-fail-fast"], correctness_only=True)
        selected = [name for name, _ in runner.selected_steps()]
        def paired(owner, names):
            # Keep this selection-only control in-process. The area ownership
            # suite exercises real bounded children and their result validation.
            self.assertIs(owner, runner)
            for name, callback in owner.selected_steps():
                if name in {"area_" + area for area in names}:
                    self.assertEqual(owner.execute_step(name, callback), 0)
            return 0
        with patch("sifr_verify.profile_runner.run_area_group", side_effect=paired) as groups, \
             patch.object(runner, "prepare_step_budget", return_value=None), \
             patch.object(runner, "admit_performance_reference") as admission, \
             patch.object(runner, "prepare_cargo_cache") as setup, \
             patch.object(runner, "run_guardrail"), patch.object(runner, "run_area") as areas, \
             patch.object(runner, "run_toolchain_step"), contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(runner.run(), 0)
        groups.assert_called_once_with(runner, ["python_interop", "generated_code_quality"])
        self.assertEqual(len(areas.call_args_list), len(runner.profile["selected_areas"]))
        admission.assert_not_called()
        setup.assert_called_once_with()
        self.assertTrue(set(selected).issubset(runner.completed_steps))
        self.assertIn(("performance", ["frontend-syntax-guardrails", "lsp-workspace-cache"]),
                      [call.args for call in areas.call_args_list])
        self.assertEqual(runner.profile_name, "merge", "native/E2E semantics retain base profile")
        self.assertEqual(runner.evidence_name, "merge-correctness")

    def test_scope_is_part_of_plan_equivalence(self):
        with tempfile.TemporaryDirectory() as tmp, contextlib.redirect_stdout(io.StringIO()), \
             contextlib.redirect_stderr(io.StringIO()):
            a, b = Path(tmp) / "a.json", Path(tmp) / "b.json"
            full = build_profile_plan("merge")
            a.write_text(json.dumps(full))
            b.write_text(json.dumps(full))
            self.assertEqual(compare_plans(str(a), str(b)), 0)
            full["execution_scope"] = "correctness"
            b.write_text(json.dumps(full))
            self.assertEqual(compare_plans(str(a), str(b)), 1)

    def test_scoped_report_cannot_overwrite_full_evidence(self):
        with tempfile.TemporaryDirectory() as tmp, \
             patch.object(profile_reporting, "REPO_ROOT", Path(tmp)), \
             patch.object(reports, "directory_stats", return_value=None), \
             contextlib.redirect_stdout(io.StringIO()):
            root = Path(tmp) / "target/validation_lane_reports"
            root.mkdir(parents=True)
            sentinels = [root / f"merge.{suffix}" for suffix in
                         ("latest.log", "latest.time", "latest.json", "status.json")]
            for sentinel in sentinels:
                sentinel.write_text("full-profile evidence")
            status = profile_reporting.run_profile_with_report(
                "merge", lambda: 7, handled_error=VerificationError,
                release_report_out=None, correctness_only=True)
            self.assertEqual(status, 7)
            for sentinel in sentinels:
                self.assertEqual(sentinel.read_text(), "full-profile evidence")
            payload = json.loads((root / "merge-correctness.latest.json").read_text())
            self.assertEqual(payload["execution_scope"], "correctness")
            self.assertEqual(payload["lane"], "merge-correctness")
            self.assertEqual(payload["functional_status"], "fail")
            self.assertEqual(payload["exit_status"], 7)
            self.assertEqual(payload["excluded_area_suites"][0]["suite"], "representative")

    def test_cli_scope_is_not_forwarded_to_e2e(self):
        with patch("sifr_verify.profile_runner.run_profile", return_value=0) as run:
            self.assertEqual(run_command(["run", "--profile", "merge", "--correctness-only",
                                         "--", "--no-fail-fast", "--sifr-jobs", "2"]), 0)
        self.assertEqual(run.call_args.args, ("merge", ["--no-fail-fast", "--sifr-jobs", "2"]))
        self.assertTrue(run.call_args.kwargs["correctness_only"])
        with self.assertRaisesRegex(VerificationError, "requires profiles plan or run"):
            run_command(["check", "--correctness-only"])

    def test_shell_facade_forwards_scope_for_plan_and_execution(self):
        with tempfile.TemporaryDirectory() as tmp:
            fake = Path(tmp) / "uv"
            version = __import__("tomllib").loads((REPO_ROOT / "verification/pyproject.toml").read_text())["tool"]["uv"]["required-version"][2:]
            fake.write_text(f'#!/bin/sh\nif [ "$1" = "--version" ]; then echo "uv {version}"; else printf "%s\\n" "$@"; fi\n')
            fake.chmod(0o700)
            for flags, command in ((["--emit-plan"], "plan"), (["--no-fail-fast", "--sifr-jobs", "2"], "run")):
                result = subprocess.run(["bash", "scripts/run_all_tests.sh", "--profile", "merge",
                                         "--correctness-only", *flags], cwd=REPO_ROOT,
                                        env={**os.environ, "PATH": tmp + os.pathsep + os.environ["PATH"]},
                                        text=True, capture_output=True, check=True)
                args = result.stdout.splitlines()
                self.assertEqual(args.count("--correctness-only"), 1)
                self.assertIn(command, args)
                if command == "run":
                    self.assertLess(args.index("--correctness-only"), args.index("--"))
                    self.assertEqual(args[args.index("--") + 1:], flags)


def policy_checks() -> None:
    result = unittest.TestResult()
    for case in (ExecutionScopeTests, FrozenCrateCoverageTests):
        unittest.defaultTestLoader.loadTestsFromTestCase(case).run(result)
    if not result.wasSuccessful():
        raise AssertionError(result.errors + result.failures)


if __name__ == "__main__":
    unittest.main()
