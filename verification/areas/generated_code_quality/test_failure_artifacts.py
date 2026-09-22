import subprocess
import tempfile
import unittest
from pathlib import Path
from failure_artifacts import cargo_target_for_run, preserve_clippy_output

class RetainedArtifactsTests(unittest.TestCase):
    def test_shared_target_is_reused_without_mutating_evidence_or_other_runs(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            first, second, shared = root / "first", root / "second", root / "shared"
            target = cargo_target_for_run(first, shared)
            target.mkdir(parents=True)
            marker = target / "warm"
            marker.write_text("keep")
            self.assertEqual(target, cargo_target_for_run(second, shared))
            self.assertNotEqual(target, cargo_target_for_run(first, None))
            result = subprocess.CompletedProcess([], 1, '{"message":"🦀"}\n', "failure\n")
            preserve_clippy_output(first, "entry", result)
            self.assertEqual((first / "diagnostics/entry.stdout.jsonl").read_text(), result.stdout)
            self.assertEqual((first / "diagnostics/entry.stderr.log").read_text(), result.stderr)
            self.assertFalse(second.exists())
            self.assertEqual(marker.read_text(), "keep")

    def test_unshared_runs_have_separate_targets(self):
        self.assertNotEqual(cargo_target_for_run(Path("one"), None),
                            cargo_target_for_run(Path("two"), None))

if __name__ == "__main__":
    unittest.main()
