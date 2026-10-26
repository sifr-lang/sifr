"""Actual dependency-graph controls for the shared pinned Python environment."""
import contextlib
import copy
import importlib.util
import io
import sys
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from .cargo_setup import cargo_preparation_plan
from .paths import REPO_ROOT
from .preparation import Preparation, PreparationRun
from .profile_commands import CommandFailed
from .profiles import load_profile
from .python_preparation import prepare_python_environment, python_consumers


class PythonPreparationTests(unittest.TestCase):
    def selection(self, *, areas=(), crate=False):
        profile = copy.deepcopy(load_profile("merge"))
        profile["selected_areas"] = list(areas)
        profile["toolchain_steps"] = ["cargo-test-sifr-full"] if crate else []
        profile["guardrail_steps"] = []
        return profile

    def test_sql_editor_alone_requires_environment(self):
        profile = self.selection(areas=[{"area": "sql_platform", "suites": ["incremental-editor"]}])
        nodes, consumers = cargo_preparation_plan(profile, {}, lambda *a, **kw: None)
        self.assertEqual(python_consumers(profile), {"area_sql_platform"})
        self.assertIn("python_environment", consumers["area_sql_platform"])
        sql = next(n for n in nodes if n.name == "area_graph_sql_platform")
        self.assertIn("python_environment", sql.requires)
        PreparationRun(nodes, consumers)

    def test_crate_only_selection_requires_environment_without_python_area(self):
        profile = self.selection(crate=True)
        nodes, consumers = cargo_preparation_plan(profile, {}, lambda *a, **kw: None)
        self.assertEqual(python_consumers(profile), {"toolchain_cargo_test_sifr_full"})
        self.assertIn("python_environment", consumers["toolchain_cargo_test_sifr_full"])
        crate = next(n for n in nodes if n.name == "crate_tests")
        self.assertIn("python_environment", crate.requires)

    def test_unrelated_sql_and_crate_selection_does_not_prepare_python(self):
        profile = self.selection(areas=[{"area": "sql_platform", "suites": ["contracts"]}], crate=True)
        profile["crate_test_membership"]["suites"] = [
            s for s in profile["crate_test_membership"]["suites"]
            if s["package"] == "sifr_type_system"]
        self.assertEqual(python_consumers(profile), set())
        nodes, _ = cargo_preparation_plan(profile, {}, lambda *a, **kw: None)
        self.assertNotIn("python_environment", [n.name for n in nodes])

    def test_nonexecuted_crate_blocker_is_not_a_consumer(self):
        profile = self.selection(crate=True)
        for suite in profile["crate_test_membership"]["suites"]:
            if suite["package"] in {"sifr_driver", "sifr_lsp"}:
                suite.update(status="red-blocker", executed_in_merge=False)
        self.assertEqual(python_consumers(profile), set())

    def test_driver_library_and_native_selection_require_the_pinned_environment(self):
        for suite_id, required in (("sifr_driver_lib", True),
                                   ("sifr_driver_generated_builds", True)):
            with self.subTest(suite=suite_id):
                profile = self.selection(crate=True)
                profile["crate_test_membership"]["suites"] = [
                    s for s in profile["crate_test_membership"]["suites"] if s["id"] == suite_id]
                self.assertEqual(bool(python_consumers(profile)), required)
                nodes, consumers = cargo_preparation_plan(profile, {}, lambda *a, **kw: None)
                self.assertIn("python_environment", consumers["toolchain_cargo_test_sifr_full"])
                self.assertIn("python_environment", next(n for n in nodes if n.name == "crate_tests").requires)

    def test_environment_failure_blocks_all_consumers_but_not_independent_work(self):
        profile = load_profile("merge")
        nodes, consumers = cargo_preparation_plan(profile, {}, lambda *a, **kw: None)
        calls = []
        def callback(name):
            def execute():
                calls.append(name)
                if name == "python_environment":
                    raise CommandFailed(7, "exit")
            return execute
        simulated = [Preparation(n.name, n.requires, callback(n.name), n.unsafe_on_failure)
                     for n in nodes]
        run = PreparationRun(simulated, consumers, no_fail_fast=True)
        with contextlib.redirect_stdout(io.StringIO()), self.assertRaises(CommandFailed):
            run.run()
        self.assertEqual(calls.count("python_environment"), 1)
        for owner in ("area_python_interop", "area_sql_platform", "toolchain_cargo_test_sifr_full"):
            self.assertIn("python_environment", run.blocked_by(owner))
        for name in ("crate_tests", "authoring", "area_graph_sql_platform"):
            self.assertEqual(run.results[name]["status"], "blocked", name)
            self.assertNotIn(name, calls)
        self.assertIn("workspace", calls)
        self.assertFalse(run.blocked_by("area_rust_interop"))
        self.assertFalse(run.blocked_by("area_generated_code_quality"))
        self.assertIsNone(run.fatal_reason)

    def test_examples_keep_environment_dependency_without_precompilation(self):
        for name in ("create-pr", "merge"):
            with self.subTest(profile=name):
                profile = load_profile(name)
                original = copy.deepcopy(profile)
                nodes, consumers = cargo_preparation_plan(profile, {}, lambda *a, **kw: None)
                names = {node.name for node in nodes}
                self.assertNotIn("area_graph_python_interop", names)
                self.assertIn("python_environment", consumers["area_python_interop"])
                self.assertIn("workspace", consumers["area_python_interop"])
                self.assertIn("authoring", consumers["area_python_interop"])
                self.assertEqual(profile, original, "preparation changed assertion selection")
                PreparationRun(nodes, consumers)

    def test_real_preparation_callback_keeps_storage_and_uses_locked_canonical_project(self):
        original = {"CARGO_TARGET_DIR": "/owned/target", "CARGO_BUILD_JOBS": "2",
                    "RUSTFLAGS": "--cfg=caller", "CARGO_ENCODED_RUSTFLAGS": "--cfg=encoded",
                    "VIRTUAL_ENV": "/other/active", "UV_PROJECT_ENVIRONMENT": "/other/uv"}
        saved = original.copy()
        calls = []
        profile = self.selection(areas=[{"area": "python_interop", "suites": ["env"]}])
        nodes, _ = cargo_preparation_plan(profile, original, lambda argv, **kw: calls.append((argv, kw["env"])))
        next(n for n in nodes if n.name == "python_environment").callback()
        self.assertEqual(len(calls), 1)
        command, env = calls[0]
        area = REPO_ROOT / "verification/areas/python_interop"
        self.assertEqual(command, ["uv", "run", "--project", str(area), "--locked", "python",
                                   str(area / "runner/prepare_environment.py")])
        self.assertEqual(env["UV_PROJECT_ENVIRONMENT"], str(area / ".venv"))
        self.assertNotIn("VIRTUAL_ENV", env)
        for key in ("CARGO_TARGET_DIR", "CARGO_BUILD_JOBS", "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"):
            self.assertEqual(env[key], original[key])
        self.assertEqual(original, saved)

    def test_environment_execution_failure_is_not_a_successful_preparation(self):
        def fail(*args, **kwargs):
            raise CommandFailed(3, "exit")
        with self.assertRaises(CommandFailed):
            prepare_python_environment({}, fail)

    def test_probe_rejects_wrong_environment_and_runs_canonical_version_check(self):
        area = REPO_ROOT / "verification/areas/python_interop"
        module_spec = importlib.util.spec_from_file_location(
            "_python_preparation_probe", area / "runner/prepare_environment.py")
        module = importlib.util.module_from_spec(module_spec)
        checked = []
        fake = SimpleNamespace(discover_paths=lambda: SimpleNamespace(area_root=area),
                               require_canonical_python=lambda root: checked.append(root) or "pinned")
        with patch.dict(sys.modules, {"env": fake}):
            module_spec.loader.exec_module(module)
        with patch.object(module.sys, "prefix", "/foreign/environment"), \
             self.assertRaisesRegex(SystemExit, "Python fixture environment must be"):
            module.main()
        self.assertEqual(checked, [area])
        with patch.object(module.sys, "prefix", str(area / ".venv")), \
             patch.object(module.sys, "base_prefix", str(area / ".venv")), \
             self.assertRaises(SystemExit):
            module.main()
        with patch.object(module, "require_canonical_python", side_effect=SystemExit("wrong version")), \
             self.assertRaisesRegex(SystemExit, "wrong version"):
            module.main()


def policy_checks():
    result = unittest.TestResult()
    unittest.defaultTestLoader.loadTestsFromTestCase(PythonPreparationTests).run(result)
    if not result.wasSuccessful():
        raise AssertionError(result.errors + result.failures)


if __name__ == "__main__":
    unittest.main()
