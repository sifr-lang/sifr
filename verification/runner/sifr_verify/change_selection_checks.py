"""Selection broadens on uncertain inputs and never removes mandatory core."""
import contextlib
import io
import subprocess
import tempfile
import unittest
from pathlib import Path

from unittest.mock import patch

from . import change_selection
from .change_selection import SelectionError, classify, selection
from .profiles import load_profile


class SelectionTests(unittest.TestCase):
    def test_complete_core_and_explicit_reasons(self):
        for paths in ([], ["README.md"], ["plans/roadmap.md"]):
            result = classify(paths)
            self.assertEqual(result["profile"], "create-pr")
            self.assertEqual(result["execution_state"], "not-executed")
            core = load_profile("create-pr")["selected_areas"]
            self.assertEqual([(j["area"], j["suites"]) for j in result["selected_jobs"]],
                             [(j["area"], j["suites"]) for j in core])
            self.assertTrue(all(j["reason"] for j in result["selected_jobs"]))

    def test_every_uncertain_and_shared_path_broadens(self):
        for path in ("verification/areas/performance/runner.py", "Cargo.lock", "crates/sifr_lsp/src/lib.rs", ".github/workflows/a.yml",
                     "verification/areas/common/runner.py", "verification/areas/new/run.py",
                     "verification/areas/coverage_matrix/manifest.json", "vendor/x/a", "../README.md",
                     "/README.md", "unknown.txt", "verification/contracts/new.json"):
            with self.subTest(path=path):
                self.assertEqual(classify([path])["profile"], "merge")
        self.assertEqual(classify(None)["profile"], "merge")
        self.assertEqual(classify([], error="diff failed")["profile"], "merge")

    def test_real_git_rename_includes_deleted_and_added_paths(self):
        with tempfile.TemporaryDirectory() as raw:
            repo = Path(raw)
            def git(*args):
                return subprocess.check_output(["git", *args], cwd=repo).decode().strip()
            git("init", "-q")
            git("config", "user.email", "test@example.invalid")
            git("config", "user.name", "Selection fixture")
            (repo / "README.md").write_text("unchanged payload")
            git("add", ".")
            git("commit", "-qm", "base")
            base = git("rev-parse", "HEAD")
            (repo / "README.md").rename(repo / "unowned.md")
            git("add", ".")
            git("commit", "-qm", "rename")
            head = git("rev-parse", "HEAD")
            result = selection(repo, base, head)
            self.assertEqual(result["changed_paths"], ["README.md", "unowned.md"])
            self.assertEqual(result["profile"], "merge")
            self.assertEqual(result["candidate_commit"], head)
            self.assertEqual(selection(repo, "missing", head)["profile"], "merge")
            self.assertFalse(selection(repo, "missing", head)["diff_available"])
            (repo / "unowned.md").write_text("dirty candidate")
            with patch.object(change_selection, "REPO_ROOT", repo), contextlib.redirect_stdout(io.StringIO()), \
                    patch("sifr_verify.profile_runner.run_profile") as runner:
                with self.assertRaises(SelectionError):
                    change_selection.main(["run", "--base", base, "--head", head])
                runner.assert_not_called()


def policy_checks():
    result = unittest.TestResult()
    unittest.defaultTestLoader.loadTestsFromTestCase(SelectionTests).run(result)
    if not result.wasSuccessful():
        raise AssertionError(result.errors + result.failures)
