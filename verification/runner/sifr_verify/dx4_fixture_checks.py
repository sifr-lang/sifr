"""DX.4 regression seeds for the shared area entrypoint."""
import contextlib
import io
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

from . import area_adapter as adapter
from .fixture_execution import failed_selection, selected_cases
from .paths import REPO_ROOT

class FixtureTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(dir=REPO_ROOT / "target")
        self.root = Path(self.directory.name)
        self.config = adapter.AreaAdapterConfig("seed", "verification", "seed",
            self.root / "manifest.json", self.root / "actual", "seed")
        self.options = adapter.AreaRunOptions(set(), False, self.root / "report.json")

    def tearDown(self):
        self.directory.cleanup()

    def script(self, name, body, expected=0):
        path = self.root / (name + ".py")
        path.write_text(body)
        return {"id": name, "entry": str(path.relative_to(REPO_ROOT)),
                "command": "area-check", "expect_exit_code": expected,
                "diagnostic_formats": []}

    def manifest(self, cases):
        self.config.manifest_path.write_text(json.dumps(
            {"suites": [{"name": "seeds", "cases": cases}]}))

    def run_area(self):
        with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            result = adapter.run_area(self.config, self.options)
        return result, json.loads(self.options.result_json.read_text())

    def test_r04_missing_tool_cannot_satisfy_negative(self):
        case = self.script("negative", "raise SystemExit(1)", expected=1)
        self.manifest([case])
        with patch.object(adapter, "run_process", side_effect=FileNotFoundError("missing compiler")):
            result, report = self.run_area()
        self.assertEqual(result, 1)
        self.assertEqual(report["suites"][0]["cases"][0]["variants"][0]["status"], "blocked")

    def test_r05_independent_failures_collected_after_blocked_prerequisite(self):
        cases = [self.script("missing", ""), self.script("first", "raise SystemExit(3)"),
                 self.script("second", "raise SystemExit(4)")]
        self.manifest(cases)
        original = adapter.run_process
        def run(argv, **kwargs):
            if argv[-1].endswith("missing.py"):
                raise FileNotFoundError("compiler missing")
            return original(argv, **kwargs)
        with patch.object(adapter, "run_process", side_effect=run):
            result, report = self.run_area()
        self.assertEqual(result, 1)
        self.assertEqual(report["summary"]["total_failures"], 3)
        self.assertEqual([c["variants"][0]["status"] for c in report["suites"][0]["cases"]],
                         ["blocked", "fail", "fail"])
        self.assertEqual(failed_selection(self.options.result_json),
                         frozenset({"seeds/missing", "seeds/first", "seeds/second"}))

    def test_r07_warm_artifact_does_not_skip_assertions(self):
        receipt = self.root / "runs"
        self.manifest([self.script("runtime",
            f"from pathlib import Path\np=Path({str(receipt)!r})\np.write_text(p.read_text()+'x' if p.exists() else 'x')")])
        self.assertEqual(self.run_area()[0], 0)
        self.assertEqual(self.run_area()[0], 0)
        self.assertEqual(receipt.read_text(), "xx")

    def test_q01_common_wrong_result_still_fails_independent_expectation(self):
        from .fixture_inventory import compare_paths
        with self.assertRaisesRegex(AssertionError, "independent"):
            compare_paths(lambda: 9, lambda: 9, lambda value: value == 0)
        with self.assertRaises(ValueError):
            compare_paths(lambda: 0, None, lambda value: value == 0)
        self.assertEqual(compare_paths(lambda: 0, lambda: 0, lambda value: value == 0), 0)
        # Identical reference and optimized output is not an expected result.
        self.manifest([self.script("reference", "raise SystemExit(9)"),
                       self.script("optimized", "raise SystemExit(9)")])
        result, report = self.run_area()
        self.assertEqual(result, 1)
        self.assertEqual(report["summary"]["total_failures"], 2)

    def test_r08_unknown_case_fails_before_any_preparation(self):
        self.manifest([self.script("case", "")])
        self.options = adapter.AreaRunOptions(set(), False, self.root / "report.json",
                                             frozenset({"seeds/missing"}))
        with patch.object(adapter, "run_process") as execute:
            with self.assertRaises(ValueError):
                self.run_area()
        execute.assert_not_called()

    def test_q01_native_link_and_runtime_remain_independent(self):
        from .fixture_execution import run_process
        source = self.root / "native.rs"
        output = self.root / "same_name"
        source.write_text('unsafe extern "C" { fn missing_symbol_dx4(); }\n'
                          'fn main() { unsafe { missing_symbol_dx4(); } }')
        check = run_process(["rustc", "--emit=metadata", str(source), "-o", str(self.root / "check.rmeta")],
                            cwd=REPO_ROOT)
        self.assertEqual(check.returncode, 0, check.stderr)
        link = run_process(["rustc", str(source), "-o", str(output)], cwd=REPO_ROOT)
        self.assertNotEqual(link.returncode, 0)
        self.assertIn("missing_symbol_dx4", link.stderr)
        source.write_text("fn main() { assert_eq!(2 + 2, 5); }")
        build = run_process(["rustc", str(source), "-o", str(output)], cwd=REPO_ROOT)
        self.assertEqual(build.returncode, 0, build.stderr)
        runtime = run_process([str(output)], cwd=REPO_ROOT)
        self.assertNotEqual(runtime.returncode, 0)
        self.assertIn("assertion", runtime.stderr)

    def test_r08_inventory_preserves_records_but_detects_added_fixture(self):
        from .fixture_inventory import inventory, compare
        import subprocess
        subprocess.run(["git", "init", "-q", str(self.root)], check=True)
        (self.root / "crates").mkdir()
        (self.root / "crates/one.sifr").write_text("def main(): pass")
        before = inventory(self.root)
        (self.root / "record.md").write_text("unchanged implementation evidence")
        self.assertTrue(compare(before, inventory(self.root))["unchanged"])
        (self.root / "crates/two.sifr").write_text("def main(): pass")
        delta = compare(before, inventory(self.root))
        self.assertFalse(delta["unchanged"])
        self.assertEqual(delta["added"], ["crates/two.sifr"])

    def test_no_fail_fast_reaches_cases_and_blocks_dependent_checks(self):
        from .profile_runner import ProfileRunner
        from .profile_commands import CommandFailed
        runner = ProfileRunner("create-pr", ["--no-fail-fast"])
        runner.profile["guardrail_steps"] = []
        runner.profile["selected_areas"] = []
        runner.profile["toolchain_steps"] = ["e2e-pass", "e2e-report-determinism", "cargo-fmt-check"]
        attempted = []
        def step(name):
            attempted.append(name)
            if name in {"e2e-pass", "cargo-fmt-check"}:
                raise CommandFailed(1)
        with patch.object(runner, "prepare_cargo_cache"), patch.object(runner, "run_toolchain_step", step), contextlib.redirect_stdout(io.StringIO()) as output:
            self.assertNotEqual(runner.run(), 0)
        self.assertEqual(attempted, ["e2e-pass", "cargo-fmt-check"])
        self.assertIn("status=blocked", output.getvalue())
        with patch("sifr_verify.profile_runner.run_selected_area") as selected:
            runner.run_area("diagnostics", ["baselines"])
            builder = selected.call_args.kwargs["command_builder"]
            self.assertIn("--no-fail-fast", builder("--area", "diagnostics"))

    def test_passing_failure_report_does_not_select_every_case(self):
        from .areas import run_command
        self.options.result_json.write_text(json.dumps({"area": "diagnostics", "suites": []}))
        with patch("sifr_verify.area_adapter.run_area") as run, contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(run_command(["run", "--area", "diagnostics",
                "--rerun-failures", str(self.options.result_json)]), 0)
        run.assert_not_called()
        with self.assertRaises(ValueError):
            failed_selection(self.options.result_json, "other-area")

    def test_bless_preserves_declared_package_exit_two(self):
        source = self.root / "main.sifr"
        source.write_text("def main(): pass")
        case = {"id": "package", "entry": str(source.relative_to(REPO_ROOT)),
                "command": "check", "expect_exit_code": 2, "diagnostic_formats": []}
        options = adapter.AreaRunOptions(set(), True, self.root / "report.json")
        with patch.object(adapter, "run_sifr_variant",
                return_value=(2, "", "declared package error\n", 1.0, ["sifr", "check"])):
            result, failed, count = adapter.run_baseline_case(self.config, "seeds", case, options)
        self.assertFalse(failed)
        self.assertEqual(count, 0)
        self.assertEqual(result["variants"][0]["status"], "pass")
        self.assertEqual((self.root / "baselines/check.exit-code.txt").read_text(), "2\n")

    def test_r08_demo_and_vendor_assets_invalidate_evidence(self):
        from .fixture_inventory import inventory, compare
        import subprocess
        subprocess.run(["git", "init", "-q", str(self.root)], check=True)
        (self.root / "demos").mkdir()
        (self.root / "vendor").mkdir()
        demo = self.root / "demos/main.sifr"
        asset = self.root / "vendor/build-input.bin"
        demo.write_text("def main(): pass")
        asset.write_bytes(b"original native input")
        before = inventory(self.root)
        demo.write_text("def main(): assert False")
        self.assertEqual(compare(before, inventory(self.root))["changed"], ["demos/main.sifr"])
        demo.write_text("def main(): pass")
        asset.write_bytes(b"changed native input")
        self.assertEqual(compare(before, inventory(self.root))["changed"], ["vendor/build-input.bin"])

    def test_bless_never_accepts_setup_failure(self):
        self.manifest([self.script("negative", "raise SystemExit(9)", expected=1)])
        self.options = adapter.AreaRunOptions(set(), True, self.root / "report.json")
        self.assertEqual(self.run_area()[0], 1)

    def test_timeout_cannot_satisfy_expected_exit(self):
        from .fixture_execution import ProcessResult
        case = self.script("timeout", "", expected=124)
        self.manifest([case])
        with patch.object(adapter, "run_process",
                          return_value=ProcessResult(124, "", "", "safety_deadline", False)):
            self.assertEqual(self.run_area()[0], 1)

def policy_checks():
    result = unittest.TextTestRunner(stream=io.StringIO()).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(FixtureTests))
    if not result.wasSuccessful():
        raise AssertionError(f"DX.4 fixture seeds failed: {result.failures} {result.errors}")

if __name__ == "__main__":
    unittest.main()

