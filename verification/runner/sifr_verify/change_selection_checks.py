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
        for paths in ([], ["README.md"], ["internal_docs/hir_maintainability_guardrails.md"]):
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

    def test_mixed_paths_and_executable_prose_never_narrow(self):
        unsafe = (
            "internal_docs/sql_schema_polymorphism.md",
            "internal_docs/stdlib_retained_compiler_intrinsics.toml",
            "plans/releases/candidates/1.0.0/stable-candidate.json",
            "plans/issues/active/new-policy.json", "internal_docs/new.md",
            "plans/roadmap.md", "./README.md", "README.md/", "README.md\n",
            "internal_docs/../README.md",
            "verification/areas/diagnostics/fixtures/diagnostics/parser_bad_indent/main.sifr",
            "verification/areas/core_language/fixtures/static_class_adapter/src/api.sifr",
            "verification/areas/performance/query_projects/lsp/main.sifr",
        )
        for path in unsafe:
            with self.subTest(path=path):
                for paths in ([path], ["README.md", path], [path, "README.md", path]):
                    self.assertEqual(classify(paths)["profile"], "merge")
        for path in change_selection.CORE_PROSE:
            self.assertEqual(classify([path])["profile"], "create-pr")
            self.assertEqual(classify([path], unsafe_changes=(path,))["profile"], "merge")

    def test_run_dispatch_forwards_only_explicit_resource_mode(self):
        head, base = "a" * 40, "b" * 40
        for profile in ("create-pr", "merge"):
            for options in ([], ["--compact-resources"]):
                with self.subTest(profile=profile, options=options), \
                        patch.object(change_selection, "selection", return_value={"profile": profile}), \
                        patch.object(change_selection, "git", side_effect=[head.encode(), b""]), \
                        patch("sifr_verify.profile_runner.run_profile", return_value=7) as runner, \
                        contextlib.redirect_stdout(io.StringIO()):
                    self.assertEqual(change_selection.main(
                        ["run", "--base", base, "--head", head, *options]), 7)
                    runner.assert_called_once_with(profile, options)
        for extra in (["--profile", "create-pr"], ["--deadline", "100"], ["--", "--no-fail-fast"]):
            with contextlib.redirect_stderr(io.StringIO()), self.assertRaises(SystemExit), \
                    patch("sifr_verify.profile_runner.run_profile") as runner:
                change_selection.main(["run", "--base", base, "--head", head, *extra])
            runner.assert_not_called()
        with contextlib.redirect_stderr(io.StringIO()), self.assertRaises(SystemExit), \
                patch.object(change_selection, "selection") as select:
            change_selection.main(["plan", "--base", base, "--head", head, "--compact-resources"])
        select.assert_not_called()

    def test_git_regular_content_only_and_mode_or_inventory_changes(self):
        with tempfile.TemporaryDirectory() as raw:
            repo = Path(raw)
            def git(*args):
                return subprocess.check_output(["git", *args], cwd=repo).decode().strip()
            git("init", "-q")
            git("config", "user.email", "test@example.invalid")
            git("config", "user.name", "Selection fixture")
            path = repo / "README.md"
            path.write_text("initial")
            git("add", ".")
            git("commit", "-qm", "base")
            base = git("rev-parse", "HEAD")
            path.write_text("prose edit")
            git("add", ".")
            git("commit", "-qm", "content")
            content = git("rev-parse", "HEAD")
            self.assertEqual(selection(repo, base, content)["profile"], "create-pr")
            path.unlink()
            path.symlink_to("unowned.md")
            git("add", ".")
            git("commit", "-qm", "symlink")
            link = git("rev-parse", "HEAD")
            self.assertEqual(selection(repo, content, link)["profile"], "merge")
            path.unlink()
            git("add", ".")
            git("commit", "-qm", "delete")
            deleted = git("rev-parse", "HEAD")
            self.assertEqual(selection(repo, content, deleted)["profile"], "merge")
            self.assertEqual(selection(repo, deleted, content)["profile"], "merge")

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
