"""Sysroot source preparation and timeout-diagnostic regression checks."""

from __future__ import annotations

import importlib.util
import io
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from unittest.mock import patch

from .cargo_setup import prepare_sysroot_source_binary
from .paths import REPO_ROOT
from .profile_commands import CommandFailed


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
        error = subprocess.TimeoutExpired(["fixture"], 1, output=b"partial stdout\xff",
                                          stderr=b"partial stderr")
        with patch.object(runner.subprocess, "run", side_effect=error), \
             redirect_stdout(io.StringIO()):
            result = runner.run_command(["fixture"], cwd=REPO_ROOT, env={}, timeout=1)
        self.assertEqual(result.returncode, 124)
        self.assertEqual(result.stdout, "partial stdout\ufffd")
        self.assertIn("partial stderr", result.summary())
        self.assertIn("timeout after 1s", result.summary())

    def test_timeout_without_output_has_a_diagnostic(self):
        runner = sysroot_module()
        with patch.object(runner.subprocess, "run",
                          side_effect=subprocess.TimeoutExpired(["fixture"], 1)), \
             redirect_stdout(io.StringIO()):
            result = runner.run_command(["fixture"], cwd=REPO_ROOT, env={}, timeout=1)
        self.assertEqual(result.stdout, "")
        self.assertEqual(result.summary(), "timeout after 1s")

    def test_checked_timeout_preserves_stdout_and_certification_error(self):
        runner = sysroot_module()
        error = subprocess.TimeoutExpired(["fixture"], 1, output=b"partial stdout",
                                          stderr=b"partial stderr")
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "stdout"
            with patch.object(runner.subprocess, "run", side_effect=error), \
                 redirect_stdout(io.StringIO()), \
                 self.assertRaisesRegex(runner.CertificationError, "(?s)exit=124.*timeout after 1s"):
                runner.run_checked(["fixture"], cwd=REPO_ROOT, env={}, timeout=1,
                                   label="fixture", stdout_path=output)
            self.assertEqual(output.read_text(), "partial stdout")
