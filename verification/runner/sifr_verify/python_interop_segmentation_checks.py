"""Bounded Python interop profile execution and complete-result checks."""

from __future__ import annotations

import io
import json
import os
import subprocess
import sys
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from types import SimpleNamespace
from unittest.mock import patch

from verification.areas.python_interop import runner as interop

from . import profile_area_steps
from .profile_commands import CommandFailed, area_command, verification_python_command


class PythonInteropSegmentationTests(unittest.TestCase):
    def setUp(self) -> None:
        manifest = json.loads(interop.MANIFEST_PATH.read_text())
        self.suites = interop.select_suites(manifest, {"self-test", "scaffold"})

    def _part(self, suite: dict) -> dict:
        cases = [
            {
                "id": case["id"],
                "entry": case["entry"],
                "command": case["command"],
                "variants": [{"status": "pass", "expected_exit_code": 0, "actual_exit_code": 0}],
            }
            for case in suite["cases"]
        ]
        result = {
            "name": suite["name"],
            "blocking": True,
            "cases": cases,
            "failed_cases": 0,
            "total_variants": len(cases),
            "total_failures": 0,
        }
        return {
            "schema_version": 1,
            "area": "python_interop",
            "bless": False,
            "manifest": str(interop.MANIFEST_PATH.relative_to(interop.REPO_ROOT)),
            "suites": [result],
            "summary": {
                "total_variants": len(cases),
                "blocking_failures": 0,
                "total_failures": 0,
            },
        }

    def test_combination_requires_every_exact_passing_suite(self) -> None:
        with TemporaryDirectory(prefix="sifr-python-interop-combine-") as directory:
            paths = [Path(directory) / f"{index}.json" for index in range(len(self.suites))]
            for path, suite in zip(paths, self.suites, strict=True):
                path.write_text(json.dumps(self._part(suite)))
            selected = interop.combine_suite_results(self.suites, [str(path) for path in paths])
            self.assertEqual([suite["name"] for suite in selected],
                             [suite["name"] for suite in self.suites])
            with self.assertRaisesRegex(SystemExit, "count"):
                interop.combine_suite_results(self.suites, [str(paths[0])])
            paths[1].unlink()
            with self.assertRaisesRegex(SystemExit, "missing or invalid"):
                interop.combine_suite_results(self.suites, [str(path) for path in paths])
            paths[1].write_text(json.dumps(self._part(self.suites[0])))
            with self.assertRaisesRegex(SystemExit, "identity drift"):
                interop.combine_suite_results(self.suites, [str(path) for path in paths])
            changed = self._part(self.suites[1])
            changed["suites"][0]["cases"][0]["variants"][0]["status"] = "fail"
            paths[1].write_text(json.dumps(changed))
            with self.assertRaisesRegex(SystemExit, "incomplete or failed"):
                interop.combine_suite_results(self.suites, [str(path) for path in paths])

    def test_profile_gives_each_suite_its_own_process_then_combines(self) -> None:
        with TemporaryDirectory(prefix="sifr-python-interop-profile-") as directory:
            root = Path(directory)
            calls = []

            def command_runner(command: list[str]) -> None:
                calls.append(command)
                output = root / command[command.index("--result-json") + 1]
                output.parent.mkdir(parents=True, exist_ok=True)
                if "--combine-suite-result" in command:
                    self.assertNotIn("--defer-certification", command)
                    selected = interop.combine_suite_results(
                        self.suites,
                        [command[index + 1] for index, token in enumerate(command)
                         if token == "--combine-suite-result"],
                    )
                    output.write_text(json.dumps({
                        "schema_version": 1,
                        "area": "python_interop",
                        "bless": False,
                        "suites": [dict(suite, blocking=True) for suite in selected],
                        "compiled_certification": {"status": "not-selected"},
                        "summary": {
                            "total_variants": sum(suite["total_variants"] for suite in selected),
                            "blocking_failures": 0,
                        },
                    }))
                else:
                    self.assertIn("--defer-certification", command)
                    suite = next(suite for suite in self.suites
                                 if suite["name"] == command[command.index("--suite") + 1])
                    output.write_text(json.dumps(self._part(suite)))
                return SimpleNamespace(returncode=0, cause="exit", truncated=False)

            with patch.object(profile_area_steps, "REPO_ROOT", root):
                result = profile_area_steps.run_segmented_python_interop(
                    suites=[suite["name"] for suite in reversed(self.suites)],
                    profile_name="merge",
                    command_runner=command_runner,
                )
            self.assertTrue(result.is_file())
            self.assertEqual(len(calls), len(self.suites) + 1)
            for command in calls:
                self.assertEqual(command[:2], [sys.executable,
                                 "verification/areas/python_interop/runner.py"])
            self.assertEqual(
                [command[command.index("--suite") + 1] for command in calls[:-1]],
                [suite["name"] for suite in self.suites],
            )

    def test_truncated_successful_combiner_cannot_certify_the_selection(self):
        with TemporaryDirectory(prefix="sifr-python-combiner-outcome-") as directory:
            root = Path(directory)
            def run(command):
                if "--combine-suite-result" in command:
                    return SimpleNamespace(returncode=0, cause="exit", truncated=True)
                suite = next(item for item in self.suites
                             if item["name"] == command[command.index("--suite") + 1])
                output = root / command[command.index("--result-json") + 1]
                output.parent.mkdir(parents=True, exist_ok=True)
                output.write_text(json.dumps(self._part(suite)))
                return SimpleNamespace(returncode=0, cause="exit", truncated=False)
            with patch.object(profile_area_steps, "REPO_ROOT", root), \
                 self.assertRaisesRegex(profile_area_steps.AreaResultError, "combiner"):
                profile_area_steps.run_segmented_python_interop(
                    suites=[suite["name"] for suite in self.suites], profile_name="merge",
                    command_runner=run)

    def _collect(self, failures, *, collect=True, invalid_result=False, invalid_outcome=False):
        with TemporaryDirectory(prefix="sifr-python-interop-failures-") as directory:
            root = Path(directory)
            calls = []
            parts = []
            def run(command):
                self.assertNotIn("--combine-suite-result", command)
                suite = next(s for s in self.suites
                             if s["name"] == command[command.index("--suite") + 1])
                calls.append(suite["name"])
                path = root / command[command.index("--result-json") + 1]
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("{" if invalid_result else json.dumps(self._part(suite)))
                parts.append(path)
                failure = failures.get(suite["name"])
                if failure is not None:
                    raise failure
                return SimpleNamespace(returncode=0, cause="exit", truncated=invalid_outcome)
            log = io.StringIO()
            with patch.object(profile_area_steps, "REPO_ROOT", root), \
                 patch("sys.stdout", log):
                try:
                    profile_area_steps.run_segmented_python_interop(
                        suites=[s["name"] for s in self.suites], profile_name="merge",
                        command_runner=run, no_fail_fast=collect)
                except BaseException as error:
                    caught = error
                else:
                    self.fail("failed/incomplete selection was accepted")
            records = [json.loads(line.removeprefix("[sifr-python-suite] "))
                       for line in log.getvalue().splitlines()
                       if line.startswith("[sifr-python-suite] ")]
            self.assertEqual([r["suite"] for r in records], [s["name"] for s in self.suites])
            self.assertTrue(all(p.exists() for p in parts), "failed evidence was removed")
            self.assertFalse((root / "target/verification/areas/python-interop-merge-results.json").exists())
            return calls, records, caught

    def test_collection_executes_later_suites_and_retains_first_failure(self):
        names = [s["name"] for s in self.suites]
        first = CommandFailed(3, "exit")
        for failures in ({names[0]: first}, {names[0]: first, names[1]: CommandFailed(4, "exit")}):
            with self.subTest(failures=list(failures)):
                calls, records, caught = self._collect(failures)
                self.assertEqual(calls, names)
                self.assertIs(caught, first)
                self.assertEqual(records[0]["status"], "command-failed")
                self.assertEqual(records[1]["status"],
                                 "command-failed" if names[1] in failures else "result-passed")

    def test_default_failure_stops_and_labels_remaining_suites(self):
        first = CommandFailed(3, "exit")
        calls, records, caught = self._collect({self.suites[0]["name"]: first}, collect=False)
        self.assertEqual(calls, [self.suites[0]["name"]])
        self.assertIs(caught, first)
        self.assertEqual(records[1]["status"], "not-run")

    def test_terminal_failures_stop_even_when_collecting(self):
        truncated = CommandFailed(1, "exit")
        truncated.outcome = SimpleNamespace(truncated=True)
        for failure in (CommandFailed(124, "safety_deadline"), CommandFailed(2, "area_capacity"),
                        CommandFailed(-15, "exit"), truncated, RuntimeError("unexpected")):
            with self.subTest(failure=failure):
                calls, records, caught = self._collect({self.suites[0]["name"]: failure})
                self.assertEqual(calls, [self.suites[0]["name"]])
                self.assertIs(caught, failure)
                self.assertEqual(records[1]["status"], "not-run")

    def test_later_terminal_failure_takes_precedence(self):
        terminal = CommandFailed(124, "safety_deadline")
        calls, _, caught = self._collect({
            self.suites[0]["name"]: CommandFailed(1, "exit"),
            self.suites[1]["name"]: terminal})
        self.assertEqual(calls, [s["name"] for s in self.suites])
        self.assertIs(caught, terminal)

    def test_success_exit_with_invalid_evidence_stops_collection(self):
        for invalid_result, invalid_outcome in ((True, False), (False, True)):
            with self.subTest(invalid_result=invalid_result, invalid_outcome=invalid_outcome):
                calls, records, caught = self._collect(
                    {}, invalid_result=invalid_result, invalid_outcome=invalid_outcome)
                self.assertEqual(calls, [self.suites[0]["name"]])
                self.assertIsInstance(caught, profile_area_steps.AreaResultError)
                self.assertEqual([r["status"] for r in records], ["incomplete", "not-run"])

    def test_profile_forwards_failure_collection_policy(self):
        from .profile_runner import ProfileRunner
        for collect in (False, True):
            runner = ProfileRunner.__new__(ProfileRunner)
            runner.env = {}
            runner.evidence_name = "merge-correctness"
            runner.no_fail_fast = collect
            with patch("sifr_verify.profile_runner.run_segmented_python_interop") as run:
                runner.run_area("python_interop", ["self-test"])
            self.assertEqual(run.call_args.kwargs["no_fail_fast"], collect)
            self.assertEqual(run.call_args.kwargs["profile_name"], "merge-correctness")

    def test_verification_child_keeps_interpreter_and_exact_inherited_context(self) -> None:
        env = dict(os.environ, VIRTUAL_ENV="/different/active-venv",
                   UV_PROJECT_ENVIRONMENT="/different/uv-project",
                   CARGO_TARGET_DIR="relative/owned-target", CARGO_BUILD_TARGET="declared-target",
                   RUSTFLAGS="--cfg caller", CARGO_ENCODED_RUSTFLAGS="--cfg\x1fencoded",
                   PYO3_PYTHON="/selected/python", SIFR_VERIFY_CONTEXT_CONTROL="present")
        # Repeated PATH entries can be meaningful input. Preserve their exact bytes.
        env["PATH"] = os.pathsep.join(("/caller/bin", env["PATH"], "/caller/bin"))
        keys = ("PATH", "VIRTUAL_ENV", "UV_PROJECT_ENVIRONMENT", "CARGO_TARGET_DIR",
                "CARGO_BUILD_TARGET", "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS",
                "PYO3_PYTHON", "SIFR_VERIFY_CONTEXT_CONTROL")
        code = ("import json,os,sys; print(json.dumps({'executable':sys.executable,"
                "'prefix':sys.prefix,'environment':{k:os.environ[k] for k in "
                + repr(keys) + "}}))")
        completed = subprocess.run(verification_python_command("-c", code), env=env,
                                   check=True, capture_output=True, text=True)
        observed = json.loads(completed.stdout)
        self.assertEqual(observed["executable"], sys.executable)
        self.assertEqual(observed["prefix"], sys.prefix)
        self.assertEqual(observed["environment"], {key: env[key] for key in keys})

    def test_ordinary_area_entrypoint_uses_the_selected_verification_interpreter(self) -> None:
        command = area_command("--help")
        self.assertEqual(command, [sys.executable, "-m", "sifr_verify", "areas", "run", "--help"])
        completed = subprocess.run(command, check=True, capture_output=True, text=True)
        self.assertIn("--area", completed.stdout)

    def test_invalid_manifest_is_a_step_failure_before_child_spawn(self) -> None:
        with TemporaryDirectory(prefix="sifr-python-interop-invalid-") as directory:
            manifest_path = Path(directory) / "manifest.json"
            manifest_path.write_text("{")
            calls = []
            with patch.object(profile_area_steps, "area_by_name",
                              return_value=SimpleNamespace(manifest_path=manifest_path)):
                with self.assertRaisesRegex(profile_area_steps.AreaResultError,
                                            "invalid python_interop area manifest"):
                    profile_area_steps.run_segmented_python_interop(
                        suites=["self-test"], profile_name="merge",
                        command_runner=lambda command: calls.append(command),
                    )
            self.assertEqual(calls, [])


def policy_checks() -> None:
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(PythonInteropSegmentationTests)
    result = unittest.TextTestRunner(stream=io.StringIO()).run(suite)
    if not result.wasSuccessful():
        raise AssertionError(f"Python interop segmentation checks failed: {result.failures} {result.errors}")


if __name__ == "__main__":
    unittest.main()
