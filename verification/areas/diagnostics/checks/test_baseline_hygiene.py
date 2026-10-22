#!/usr/bin/env python3
"""Exercise diagnostic hygiene against a real isolated tracked-file inventory."""
from __future__ import annotations

import contextlib
import io
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import baseline_hygiene


class BaselineHygieneTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory(prefix="sifr-diagnostic-hygiene-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        subprocess.run(["git", "init", "-q", str(self.root)], check=True)
        root_patch = patch.object(baseline_hygiene, "ROOT", self.root)
        root_patch.start()
        self.addCleanup(root_patch.stop)

    def check_files(self, files: dict[str, str]) -> tuple[int, str]:
        # Each scenario has an independent real index; discovery is not mocked.
        for name, content in files.items():
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")
        subprocess.run(["git", "add", "--all"], cwd=self.root, check=True)
        stderr = io.StringIO()
        with contextlib.redirect_stderr(stderr):
            status = baseline_hygiene.main()
        return status, stderr.getvalue()

    def test_verifier_implementation_can_exercise_real_tool_stderr(self) -> None:
        status, output = self.check_files({
            "verification/runner/sifr_verify/hardening/test_tool.py":
                'stderr = "error[E0308]: type mismatch"',
            "verification/areas/example/checks/check.py":
                'retired = "SIFR-TYPE-0001"',
        })
        self.assertEqual((status, output), (0, ""))

    def test_python_fixtures_and_baselines_still_reject_legacy_text(self) -> None:
        paths = [
            "verification/areas/example/fixtures/tool.py",
            "verification/areas/example/baselines/tool.py",
            "verification/runner/fixtures/tool.py",
            "verification/runner/baselines/tool.py",
        ]
        status, output = self.check_files(dict.fromkeys(paths, "error[E0308]"))
        self.assertEqual(status, 1)
        for path in paths:
            self.assertIn(path + ": contains legacy", output)
        self.assertEqual(len(output.splitlines()), len(paths))

    def test_artifacts_remain_covered_independent_of_extension(self) -> None:
        paths = [
            "crates/sifr/tests/e2e/pass/example.sifr",
            "crates/sifr/tests/verification/example.py",
            "verification/areas/example/baselines/unknown.extension",
            "verification/areas/example/fixtures/extensionless",
            "verification/runner/policy.json",
            "verification/unusual.data",
            "docs/errors/example.py",
            "docs/schemas/example.py",
        ]
        status, output = self.check_files(dict.fromkeys(paths, "[E0001]"))
        self.assertEqual(status, 1)
        for path in paths:
            self.assertIn(path + ": contains legacy", output)
        self.assertEqual(len(output.splitlines()), len(paths))

    def test_catch_all_is_still_rejected_in_artifacts(self) -> None:
        paths = [
            "verification/areas/example/fixtures/example.py",
            "verification/areas/example/baselines/example.txt",
            "docs/errors/example.mdx",
            "docs/schemas/example.json",
            "crates/sifr/tests/e2e/pass/example.sifr",
        ]
        status, output = self.check_files(dict.fromkeys(paths, "SIFR-TYPE-0001"))
        self.assertEqual(status, 1)
        for path in paths:
            self.assertIn(path + ": contains forbidden", output)
        self.assertEqual(len(output.splitlines()), len(paths))

    def test_canonical_expectations_and_codes_pass(self) -> None:
        status, output = self.check_files({
            "crates/sifr/tests/e2e/fail/plain.sifr": "# expect-error: SIFR-TYPE-0002",
            "crates/sifr/tests/e2e/fail/column.sifr":
                "  # expect-error[col=12]: SIFR-PARSER-0002  ",
            "verification/areas/example/baselines/example.txt": "SIFR-TYPE-0002",
        })
        self.assertEqual((status, output), (0, ""))

    def test_malformed_expectations_are_all_rejected(self) -> None:
        expectations = [
            "# expect-error: message fragment",
            "# expect-error: [E0308]",
            "# expect-error[col=x]: SIFR-TYPE-0002",
            "# expect-error: SIFR-TYPE-0002 trailing",
            "# expect-error: SIFR-TYPE-0002 SIFR-TYPE-0003",
        ]
        paths = [f"crates/sifr/tests/e2e/fail/case_{i}.sifr"
                 for i in range(len(expectations))]
        status, output = self.check_files(dict(zip(paths, expectations)))
        self.assertEqual(status, 1)
        for path in paths:
            self.assertIn(path + ":1: expect-error must assert", output)

    def test_implementation_exclusion_is_not_a_directory_prefix_match(self) -> None:
        path = "verification/fixtures_like_name/should_be_implementation.py"
        status, output = self.check_files({path: "[E0308]"})
        self.assertEqual((status, output), (0, ""))

    def test_untracked_scratch_files_are_not_baselines(self) -> None:
        path = self.root / "verification/areas/example/baselines/scratch.txt"
        path.parent.mkdir(parents=True)
        path.write_text("[E0308]")
        stderr = io.StringIO()
        with contextlib.redirect_stderr(stderr):
            status = baseline_hygiene.main()
        self.assertEqual((status, stderr.getvalue()), (0, ""))


if __name__ == "__main__":
    unittest.main()
