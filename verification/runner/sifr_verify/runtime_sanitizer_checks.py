"""Regression checks for the governed runtime sanitizer command boundary."""

from __future__ import annotations

import contextlib
import copy
import importlib.util
import io
import os
import subprocess
import unittest
from unittest.mock import patch

from .paths import REPO_ROOT


def load_runtime_adapter():
    path = REPO_ROOT / "verification" / "areas" / "runtime_platform" / "runner.py"
    spec = importlib.util.spec_from_file_location("runtime_sanitizer_test_adapter", path)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class SanitizerTargetTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.adapter = load_runtime_adapter()
        cls.cases = {case["id"]: case for case in cls.adapter.load_sanitizer_manifest()["cases"]}

    def run_case(self, case, host, *, outcome=None, skip=()):
        if outcome is None:
            outcome = subprocess.CompletedProcess([], 0, "", "")
        with patch.object(self.adapter, "sanitizer_skip_reasons", return_value=list(skip)), \
                patch.object(self.adapter.subprocess, "run", side_effect=outcome if isinstance(outcome, Exception) else None,
                             return_value=outcome) as execute, \
                patch.dict(os.environ, {"CARGO_HOME": "/owned/cache"}, clear=True), \
                contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            result = self.adapter.run_sanitizer_case("sanitizer-smoke", case, host)
        return result, execute

    def test_every_instrumented_case_keeps_flags_selection_and_timeout(self):
        instrumented = [case for case in self.cases.values() if case["tool"] in {"asan", "lsan", "tsan"}]
        self.assertEqual(len(instrumented), 5)
        for case in instrumented:
            for host in case["supported_host_triples"]:
                with self.subTest(case=case["id"], host=host):
                    original = copy.deepcopy(case)
                    result, execute = self.run_case(case, host)
                    execute.assert_called_once()
                    args, kwargs = execute.call_args
                    command = args[0]
                    boundary = original["command"].index("--") if "--" in original["command"] else len(original["command"])
                    self.assertEqual(command[boundary:boundary + 2], ["--target", host])
                    self.assertEqual(command[:boundary] + command[boundary + 2:], original["command"])
                    self.assertEqual(kwargs["env"]["RUSTFLAGS"], original["env"]["RUSTFLAGS"])
                    self.assertEqual(kwargs["env"]["CARGO_HOME"], "/owned/cache")
                    self.assertEqual(kwargs["env"]["CARGO_NET_OFFLINE"], "true")
                    self.assertNotIn("CARGO_BUILD_TARGET", kwargs["env"])
                    self.assertNotIn("CARGO_TARGET_DIR", kwargs["env"])
                    self.assertEqual(kwargs["timeout"], original["timeout_seconds"])
                    self.assertEqual(kwargs["cwd"], REPO_ROOT)
                    self.assertEqual(result["argv"], command)
                    self.assertEqual(result["status"], "pass")
                    self.assertEqual(case, original)

    def test_generated_program_arguments_stay_after_cargo_separator(self):
        case = self.cases["generated-binary-asan-smoke"]
        result, _ = self.run_case(case, "aarch64-apple-darwin")
        command = result["argv"]
        self.assertEqual(command[command.index("--") - 2:],
                         ["--target", "aarch64-apple-darwin", "--", "run", "demos/hello.sifr"])

    def test_miri_keeps_its_declared_command(self):
        case = self.cases["runtime-miri-full"]
        result, execute = self.run_case(case, "aarch64-apple-darwin")
        self.assertEqual(execute.call_args.args[0], case["command"])
        self.assertEqual(result["argv"], case["command"])

    def test_existing_skip_does_not_launch_or_rewrite_command(self):
        case = self.cases["deterministic-concurrency-model-full"]
        result, execute = self.run_case(case, "aarch64-apple-darwin", skip=[case["skip_reason"]])
        execute.assert_not_called()
        self.assertEqual(result["status"], "skip")
        self.assertEqual(result["argv"], case["command"])
        self.assertEqual(result["skip_reason"], case["skip_reason"])

    def test_nonzero_result_remains_a_blocking_failure(self):
        case = self.cases["runtime-asan-smoke"]
        result, _ = self.run_case(case, "aarch64-apple-darwin",
                                  outcome=subprocess.CompletedProcess([], 101, "", "asan finding"))
        self.assertEqual(result["status"], "fail")
        self.assertEqual(result["mismatches"], ["exit=101 expected=0"])

    def test_timeout_remains_a_failure_with_original_limit(self):
        case = self.cases["runtime-asan-smoke"]
        result, execute = self.run_case(case, "aarch64-apple-darwin",
                                         outcome=subprocess.TimeoutExpired(case["command"], 120))
        self.assertEqual(execute.call_args.kwargs["timeout"], 120)
        self.assertEqual(result["status"], "fail")
        self.assertEqual(result["mismatches"], ["timeout after 120s"])


def policy_checks() -> None:
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(SanitizerTargetTests)
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    if not result.wasSuccessful():
        raise AssertionError("runtime sanitizer target checks failed")


if __name__ == "__main__":
    policy_checks()
