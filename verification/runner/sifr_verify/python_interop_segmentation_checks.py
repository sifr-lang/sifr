"""Bounded Python interop profile execution and complete-result checks."""

from __future__ import annotations

import io
import json
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

from verification.areas.python_interop import runner as interop

from . import profile_area_steps


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
            manifest_path = root / "verification/areas/python_interop/manifest.json"
            manifest_path.parent.mkdir(parents=True)
            manifest_path.write_text(json.dumps({"suites": self.suites}))
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

            with patch.object(profile_area_steps, "REPO_ROOT", root):
                result = profile_area_steps.run_segmented_python_interop(
                    suites=[suite["name"] for suite in reversed(self.suites)],
                    profile_name="merge",
                    command_runner=command_runner,
                )
            self.assertTrue(result.is_file())
            self.assertEqual(len(calls), len(self.suites) + 1)
            self.assertEqual(
                [command[command.index("--suite") + 1] for command in calls[:-1]],
                [suite["name"] for suite in self.suites],
            )


def policy_checks() -> None:
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(PythonInteropSegmentationTests)
    result = unittest.TextTestRunner(stream=io.StringIO()).run(suite)
    if not result.wasSuccessful():
        raise AssertionError(f"Python interop segmentation checks failed: {result.failures} {result.errors}")


if __name__ == "__main__":
    unittest.main()
