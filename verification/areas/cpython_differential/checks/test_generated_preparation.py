"""Check explicit generated-program preparation without changing oracle limits."""

import contextlib
import io
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import generated_suite as generated
import prepare_generated


class GeneratedPreparationTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.info = {"binary": str(self.root / "owned/release/sifr"),
                     "binary_sha256": "compiler-hash", "source_digest": "source-hash"}
        self.manifest = json.loads(generated.MANIFEST.read_text())

    def prepare(self, suites, result, *, build_failure=False):
        def build(manifest, failures):
            if build_failure:
                failures.append("compiler failed")
            return self.info
        with patch.object(generated, "REPO_ROOT", self.root), \
             patch.object(generated, "ACTUAL_ROOT", self.root / "generated"), \
             patch.object(generated, "validate_python_version", return_value=[]), \
             patch.object(generated, "build_release_binary", side_effect=build), \
             patch.object(generated, "run_command", return_value=result) as run, \
             contextlib.redirect_stdout(io.StringIO()):
            failures = prepare_generated.prepare_suites(suites)
        return failures, run

    def test_selected_cases_share_materialization_and_record_actual_preparation(self):
        suite = "generated_minimized_seeds"
        result = generated.RuntimeResult(0, "preparation output\n", "", 45000.0)
        failures, run = self.prepare([suite, suite], result)
        self.assertEqual(failures, [])
        cases = self.manifest["suites"][suite]["cases"]
        self.assertEqual(run.call_count, len(cases))
        for case, call in zip(cases, run.call_args_list, strict=True):
            source = self.root / "generated" / suite / case["id"] / "main.sifr"
            self.assertEqual(call.args, ([self.info["binary"], "--sysroot", str(self.root),
                                         "run", str(source)], 300))
            self.assertEqual(source.read_text(), generated.generate_program(case).sifr_source)
            seed = json.loads(source.with_name("seed.json").read_text())
            record = json.loads(source.with_name("preparation.json").read_text())
            self.assertEqual(seed["release_binary"], self.info)
            self.assertEqual(record["release_binary"], self.info)
            self.assertEqual(record["result"]["stdout"], result.stdout)
            self.assertEqual(record["result"]["duration_ms"], 45000.0)
        self.assertEqual(self.manifest["suites"][suite]["per_program_timeout_seconds"], 20)
        self.assertFalse((self.root / "generated/generated_broader").exists())

    def test_build_failure_prevents_program_preparation(self):
        failures, run = self.prepare(["generated_broader"],
                                     generated.RuntimeResult(0, "", "", 0), build_failure=True)
        self.assertEqual(failures, ["compiler failed"])
        run.assert_not_called()

    def test_timeout_and_wrong_exit_fail_preparation_for_every_selected_case(self):
        for result in (generated.RuntimeResult(124, "", "partial", 300000.0, True),
                       generated.RuntimeResult(1, "", "error: compile failed", 10.0)):
            with self.subTest(result=result):
                failures, run = self.prepare(["generated_broader"], result)
                self.assertTrue(failures)
                self.assertEqual(run.call_count, len(self.manifest["suites"]["generated_broader"]["cases"]))
                for case in self.manifest["suites"]["generated_broader"]["cases"]:
                    self.assertTrue(any(case["id"] in failure for failure in failures))


if __name__ == "__main__":
    unittest.main(verbosity=2)
