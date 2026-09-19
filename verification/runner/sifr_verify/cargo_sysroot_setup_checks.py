"""Sysroot source preparation and timeout-diagnostic regression checks."""

from __future__ import annotations

import importlib.util
import io
import os
import time
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from unittest.mock import patch

from .cargo_setup import prepare_sysroot_source_binary, prepare_sysroot_package_binary
from .paths import REPO_ROOT
from .profile_commands import CommandFailed
from .process_execution import Outcome


def sysroot_module():
    path = REPO_ROOT / "verification/areas/sysroot_release/runner.py"
    spec = importlib.util.spec_from_file_location("_sysroot_setup_checks_runner", path)
    if spec is None or spec.loader is None:
        raise AssertionError(f"cannot load sysroot runner: {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class SysrootSetupPolicyTests(unittest.TestCase):
    def test_only_boundary_equivalence_prepares_the_source_graph_once(self):
        calls = []
        run = lambda args, **kw: calls.append((args, kw["env"]))
        env = {"CARGO_BUILD_JOBS": "2", "CARGO_TARGET_DIR": "/outer/target"}
        for areas in ([], [{"area": "sysroot_release", "suites": ["host-installed-smoke"]}],
                      [{"area": "sysroot_release", "suites": ["host-installed-stdlib-heavy"]}]):
            prepare_sysroot_source_binary({"selected_areas": areas}, env, run)
        self.assertEqual(calls, [])
        profile = {"selected_areas": [
            {"area": "sysroot_release", "suites": ["boundary-equivalence", "host-installed-smoke"]},
            {"area": "sysroot_release", "suites": ["boundary-equivalence"]},
        ]}
        prepare_sysroot_source_binary(profile, env, run)
        self.assertEqual(calls, [([sys.executable, str(REPO_ROOT /
            "verification/areas/sysroot_release/source_build.py")], env)])
        self.assertEqual(env["CARGO_TARGET_DIR"], "/outer/target")

    def test_installed_suites_prepare_one_matching_release_graph(self):
        calls = []
        run = lambda args, **kw: calls.append((args, kw["env"]))
        env = {"CARGO_BUILD_JOBS": "2"}
        prepare_sysroot_package_binary({"selected_areas": [
            {"area": "sysroot_release", "suites": ["metadata-structural"]}]}, env, run)
        self.assertEqual(calls, [])
        for suite in ("boundary-equivalence", "host-installed-smoke",
                      "host-installed-stdlib-heavy", "metadata-corpus"):
            with self.subTest(suite=suite):
                calls.clear()
                prepare_sysroot_package_binary({"selected_areas": [
                    {"area": "sysroot_release", "suites": [suite, "metadata-structural"]}]}, env, run)
                self.assertEqual(calls, [([sys.executable, str(REPO_ROOT /
                    "verification/areas/sysroot_release/package_build.py")], env)])
        runner = sysroot_module()
        root = Path("/owned/worktree")
        artifact = root / "artifacts"
        production, prod_env = runner.package_build_configuration(root, env, "host", artifact)
        prepared, prep_env = runner.package_build_configuration(root, env, "host", artifact,
                                                               prepare_only=True)
        self.assertEqual(prepared, [*production, "--prepare-only"])
        self.assertEqual(prep_env, prod_env)
        self.assertEqual(prod_env["SIFR_RELEASE_VERSION"], runner.RELEASE_VERSION)
        self.assertEqual(prod_env["CARGO_TARGET_DIR"], str(root / "target/sysroot_release/cargo-target"))
        self.assertNotIn("SIFR_RELEASE_VERSION", env)

    def test_selected_area_preparation_preserves_execution_arguments(self):
        from .area_cargo_setup import sql_preparation_commands, prepare_area_graphs
        commands = sql_preparation_commands(["schema-profiles"])
        self.assertTrue(commands)
        for command in commands:
            self.assertEqual(command[:3], ["cargo", "test", "--no-run"])
            self.assertIn("--locked", command)
        calls = []
        env = {"CARGO_NET_OFFLINE": "true", "VIRTUAL_ENV": "/unrelated"}
        prepare_area_graphs({"selected_areas": [
            {"area": "python_interop", "suites": ["dataframe-examples", "arrow-examples", "tier1"]},
            {"area": "fuzz_property", "suites": ["fuzz-smoke"]}]}, env,
            lambda command, **kw: calls.append((command, kw["env"])))
        self.assertEqual(calls[0][0][-2:], ["--suite", "arrow-examples"])
        self.assertEqual(calls[1][0][-2:], ["--suite", "dataframe-examples"])
        self.assertNotIn("VIRTUAL_ENV", calls[0][1])
        self.assertEqual(calls[0][1]["CARGO_NET_OFFLINE"], "true")
        self.assertEqual(calls[2][0], ["cargo", "build", "--locked", "--offline",
                                     "-p", "sifr_driver", "--bin", "diagnostic_rendering_harness"])
        self.assertIn("VIRTUAL_ENV", env)

    def test_release_corpus_preparation_pins_matching_producer_version(self):
        runner = sysroot_module()
        import package_build
        calls = []
        root = Path("/owned/worktree")
        env = {"CARGO_NET_OFFLINE": "true", "CARGO_BUILD_JOBS": "2"}
        with patch.object(package_build.subprocess, "check_output", return_value="host: x86_64-unknown-linux-gnu\n"), \
             patch.object(package_build, "prepare_source_snapshot") as snapshot:
            package_build.prepare(root, env, lambda command, **kw: calls.append((command, kw)))
        corpus, execution_env, producer = runner.corpus_configuration(root, env)
        snapshot.assert_called_once_with(root, producer, runner.RELEASE_VERSION)
        self.assertEqual(calls[1][0], [*corpus[:7], "--no-run"])
        self.assertEqual(calls[1][1]["env"], execution_env)
        self.assertEqual(execution_env["SIFR_RELEASE_VERSION"], runner.RELEASE_VERSION)
        self.assertEqual(execution_env["SIFR_SYSROOT"], str(producer))
        self.assertEqual(corpus[-4:], ["full_corpus_exact_emission", "--", "--ignored", "--nocapture"])
        self.assertNotIn("SIFR_RELEASE_VERSION", env)

    def test_source_preparation_failure_propagates(self):
        def fail(*args, **kwargs):
            raise CommandFailed(101)
        with self.assertRaises(CommandFailed):
            prepare_sysroot_source_binary({"selected_areas": [
                {"area": "sysroot_release", "suites": ["boundary-equivalence"]}
            ]}, {}, fail)

    def test_preparation_and_execution_share_locked_offline_private_graph(self):
        runner = sysroot_module()
        root = Path("/owned/worktree")
        original = {"CARGO_TARGET_DIR": "/outer/target", "CARGO_NET_OFFLINE": "false",
                    "CARGO_BUILD_JOBS": "2"}
        command, env, binary = runner.source_build_configuration(root, original)
        self.assertEqual(command, ["cargo", "build", "--locked", "--offline", "-p", "sifr"])
        self.assertEqual(binary, root / "target/sysroot_release/source-cargo-target/debug/sifr")
        self.assertEqual(env, {**original, "CARGO_TARGET_DIR": str(binary.parents[1]),
                               "CARGO_NET_OFFLINE": "true"})
        self.assertEqual(original["CARGO_NET_OFFLINE"], "false")
        with patch.object(runner, "REPO_ROOT", root), \
             patch.object(runner, "base_env", return_value=original), \
             patch.object(runner, "run_checked") as run, \
             patch.object(Path, "is_file", return_value=True):
            self.assertEqual(runner.build_source_sifr(), binary)
        run.assert_called_once_with(command, cwd=root, env=env,
            label="build source-tree compiler", timeout=900)

    def test_successful_preparation_requires_the_actual_binary(self):
        runner = sysroot_module()
        import source_build
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with self.assertRaisesRegex(RuntimeError, "compiler was not produced"):
                source_build.prepare_source_sifr(root, {}, lambda *args, **kw: None)
            def build(command, **kwargs):
                self.assertTrue(kwargs["check"])
                binary = runner.source_build_configuration(root, {})[2]
                binary.parent.mkdir(parents=True)
                binary.write_text("prepared fixture")
            self.assertEqual(source_build.prepare_source_sifr(root, {}, build),
                             runner.source_build_configuration(root, {})[2])

    def test_timeout_bytes_remain_text_with_an_explicit_timeout_reason(self):
        runner = sysroot_module()
        outcome = Outcome(124, "safety_deadline", b"partial stdout\xff",
                          b"partial stderr", False, 1.0)
        with patch.object(runner, "execute", return_value=outcome), \
             redirect_stdout(io.StringIO()):
            result = runner.run_command(["fixture"], cwd=REPO_ROOT, env={}, timeout=1)
        self.assertEqual(result.returncode, 124)
        self.assertEqual(result.stdout, "partial stdout\ufffd")
        self.assertIn("partial stderr", result.summary())
        self.assertIn("safety_deadline after 1s", result.summary())

    def test_checked_timeout_preserves_stdout_and_certification_error(self):
        runner = sysroot_module()
        outcome = Outcome(124, "safety_deadline", b"partial stdout", b"", False, 1.0)
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "stdout"
            with patch.object(runner, "execute", return_value=outcome), \
                 redirect_stdout(io.StringIO()), \
                 self.assertRaisesRegex(runner.CertificationError, "(?s)exit=124.*safety_deadline after 1s"):
                runner.run_checked(["fixture"], cwd=REPO_ROOT, env={}, timeout=1,
                                   label="fixture", stdout_path=output)
            self.assertEqual(output.read_text(), "partial stdout")

    def test_installed_runner_reaps_timed_out_descendants(self):
        runner = sysroot_module()
        with tempfile.TemporaryDirectory() as directory, redirect_stdout(io.StringIO()):
            marker = Path(directory) / "escaped"
            script = ("import os,time; child=os.fork(); "
                      "time.sleep(1) if child==0 else time.sleep(20); "
                      f"open({str(marker)!r},'w').write('escaped')")
            result = runner.run_command([sys.executable, "-c", script],
                                        cwd=REPO_ROOT, env=os.environ.copy(), timeout=.1)
            self.assertEqual(result.returncode, 124)
            time.sleep(1.1)
            self.assertFalse(marker.exists())

    def test_installed_runner_preserves_stdin_and_rejects_truncated_success(self):
        runner = sysroot_module()
        text = "λ" * 100000
        with redirect_stdout(io.StringIO()):
            result = runner.run_command([sys.executable, "-c",
                 "import sys; sys.stdout.write(sys.stdin.read())"],
                 cwd=REPO_ROOT, env=os.environ.copy(), input_text=text)
        self.assertEqual((result.returncode, result.stdout), (0, text))
        outcome = Outcome(0, "exit", b"partial", b"", True, .1)
        with patch.object(runner, "execute", return_value=outcome), redirect_stdout(io.StringIO()):
            result = runner.run_command(["fixture"], cwd=REPO_ROOT, env={})
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("bounded capture limit", result.summary())
