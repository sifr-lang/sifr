"""Shared release preparation selection, freshness and failure controls."""
from contextlib import redirect_stdout
from dataclasses import replace
import io
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from verification.areas.common import release_compiler as release
from .cargo_setup import cargo_preparation_plan
from .preparation import PreparationRun
from .profile_commands import CommandFailed


class ReleaseCompilerPreparationTests(unittest.TestCase):
    def profile(self, areas):
        return {"name": "test", "cargo_policy": {"locked": True, "offline": True,
                 "setup_command": "cargo fetch --locked"}, "selected_areas": areas,
                "toolchain_steps": [], "guardrail_steps": []}

    def test_only_actual_release_compiler_consumers_select_preparation(self):
        for area, suites in (("distribution_release", ["qualification", "stable-prepare"]),
                             ("sysroot_release", ["metadata-structural"]),
                             ("coverage_matrix", ["representative"])):
            profile = self.profile([{"area": area, "suites": suites}])
            self.assertEqual(release.release_compiler_owners(profile), ())
            nodes, _ = cargo_preparation_plan(profile, {}, lambda *a, **k: None)
            self.assertNotIn("release_compiler", [node.name for node in nodes])
        for suite in ("representative", "full"):
            self.assertEqual(release.release_compiler_owners(self.profile([
                {"area": "distribution_release", "suites": [suite]}])),
                ("area_distribution_release",))
        for suite in release.INSTALLED_PACKAGE_SUITES:
            self.assertEqual(release.release_compiler_owners(self.profile([
                {"area": "sysroot_release", "suites": [suite]}])),
                ("area_sysroot_release",))

    def shared_plan(self, run):
        return cargo_preparation_plan(self.profile([
            {"area": "distribution_release", "suites": ["representative", "full"]},
            {"area": "sysroot_release", "suites": ["host-installed-smoke", "metadata-corpus"]},
            {"area": "coverage_matrix", "suites": ["representative"]},
        ]), {"CARGO_BUILD_JOBS": "2"}, run)

    def test_both_consumers_share_one_node_and_corpus_depends_on_it(self):
        calls = []
        nodes, consumers = self.shared_plan(lambda *a, **k: calls.append((a, k)))
        shared = [node for node in nodes if node.name == "release_compiler"]
        self.assertEqual(len(shared), 1)
        shared[0].callback()
        self.assertEqual(len(calls), 1)
        self.assertTrue(calls[0][0][0][-1].endswith("/common/release_compiler.py"))
        self.assertEqual(calls[0][1]["env"]["CARGO_NET_OFFLINE"], "true")
        self.assertEqual(calls[0][1]["env"]["CARGO_BUILD_JOBS"], "2")
        for name in ("area_distribution_release", "area_sysroot_release"):
            self.assertEqual(consumers[name].count("release_compiler"), 1)
        package = next(node for node in nodes if node.name == "sysroot_package")
        self.assertIn("release_compiler", package.requires)
        self.assertNotIn("release_compiler", consumers["area_coverage_matrix"])

    def test_shared_failure_blocks_both_consumers_and_keeps_independent_work(self):
        def fail(*args, **kwargs):
            raise CommandFailed(101, "exit")
        nodes, consumers = self.shared_plan(fail)
        nodes = [node if node.name == "release_compiler" else
                 replace(node, callback=lambda: None) for node in nodes]
        preparation = PreparationRun(nodes, consumers, no_fail_fast=True)
        with redirect_stdout(io.StringIO()), self.assertRaises(CommandFailed):
            preparation.run()
        self.assertEqual(preparation.results["release_compiler"]["status"], "fail")
        self.assertEqual(preparation.results["sysroot_package"]["status"], "blocked")
        for name in ("area_distribution_release", "area_sysroot_release"):
            self.assertIn("release_compiler", preparation.blocked_by(name))
        self.assertEqual(preparation.blocked_by("area_coverage_matrix"), "")

    def fixture(self, root):
        path = root / "target/sysroot_release/cargo-target/test-host/release/sifr"
        path.parent.mkdir(parents=True)
        path.write_bytes(b"fixture executable; never run")
        path.chmod(0o755)
        return path

    def test_every_request_uses_canonical_release_freshness_and_exact_version(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            expected = self.fixture(root)
            env = {"CARGO_TARGET_DIR": "/unrelated", "SIFR_RELEASE_VERSION": "wrong",
                   "CARGO_BUILD_JOBS": "2", "RUSTFLAGS": "caller-flags"}
            before = dict(env)
            calls = []
            with patch.object(release.subprocess, "check_output", side_effect=[
                    "host: test-host\n", f"sifr {release.RELEASE_VERSION}\n"] * 2):
                for _ in range(2):
                    self.assertEqual(release.prepare_release_compiler(
                        root, env, lambda *a, **k: calls.append((a, k))), expected)
            self.assertEqual(len(calls), 2, "no existing-path shortcut may skip Cargo freshness")
            command, prepared = release.package_build_configuration(
                root, env, "test-host", root / "target/sysroot_release/preparation",
                prepare_only=True)
            self.assertEqual(calls[0][0][0], command)
            self.assertEqual(calls[1][1]["env"], prepared)
            self.assertEqual(command[-2:], ["--cargo-build", "--prepare-only"])
            self.assertEqual(prepared["SIFR_RELEASE_VERSION"], release.RELEASE_VERSION)
            self.assertEqual(prepared["CARGO_NET_OFFLINE"], "true")
            self.assertEqual(prepared["RUSTFLAGS"], "caller-flags")
            self.assertEqual(env, before)

    def test_missing_and_wrong_version_artifacts_reject_successful_build_exit(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with patch.object(release.subprocess, "check_output", return_value="host: test-host\n"), \
                 self.assertRaisesRegex(RuntimeError, "not executable"):
                release.prepare_release_compiler(root, {}, lambda *a, **k: None)
            self.fixture(root)
            with patch.object(release.subprocess, "check_output", side_effect=[
                    "host: test-host\n", "sifr 0.1.0-beta.12\n"]), \
                 self.assertRaisesRegex(RuntimeError, "version mismatch"):
                release.prepare_release_compiler(root, {}, lambda *a, **k: None)

    def test_build_failure_cannot_reuse_an_existing_matching_version(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.fixture(root)
            def fail(*args, **kwargs):
                raise subprocess.CalledProcessError(101, args[0])
            with patch.object(release.subprocess, "check_output", return_value="host: test-host\n") as read, \
                 self.assertRaises(subprocess.CalledProcessError):
                release.prepare_release_compiler(root, {}, fail)
            self.assertEqual(read.call_count, 1)

    def test_invalid_host_cannot_select_an_arbitrary_artifact_path(self):
        for host in ("", "host: \n", "host: first\nhost: second\n"):
            with patch.object(release.subprocess, "check_output", return_value=host), \
                 self.assertRaisesRegex(RuntimeError, "one host target"):
                release.prepare_release_compiler(Path("/unused"), {},
                    lambda *a, **k: self.fail("build must not start"))
