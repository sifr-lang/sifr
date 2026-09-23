"""Focused policy and clean-cache qualification of generated Cargo preparation."""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import os
import shutil
import subprocess
import sys
import tempfile
import tomllib
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from unittest.mock import patch

from .cargo_setup import enable_offline_cargo, prepare_cargo_cache, prepare_authoring_test_binaries, prepare_maintained_demo_cache, prepare_tooling_test_binaries, prepare_performance_binaries, prepare_generated_oracle_binary
from .cargo_crate_setup import prepare_crate_test_binaries
from .cargo_fixture_setup import fixture_graph_hashes, locked_fixture_manifests
from .cargo_fixture_setup_checks import FixtureSetupPolicyTests
from .cargo_crate_setup_checks import CrateSetupPolicyTests
from .cargo_sysroot_setup_checks import SysrootSetupPolicyTests
from .generated_cargo_setup import (
    GIT_SOURCE, fetch_generated_graph, portable_graph, preparation_entries, quality_module,
)
from .paths import REPO_ROOT
from .profile_commands import CommandFailed, run_command
from .profile_runner import ProfileRunner
from .profiles import load_profile

REVISION = "a" * 40


class SetupPolicyTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.manifest = self.root / "Cargo.toml"
        self.lock = self.root / "Cargo.lock"
        self.manifest.write_text(
            '[package]\nname = "probe"\nversion = "0.1.0"\n[dependencies]\n'
            f'sifr_runtime = {{ git = "{GIT_SOURCE}", rev = "{REVISION}" }}\n'
        )
        self.lock.write_text(
            'version = 4\n[[package]]\nname = "sifr_runtime"\nversion = "0.0.0"\n'
            f'source = "git+{GIT_SOURCE}?rev={REVISION}#{REVISION}"\n'
        )

    def test_profile_order_environment_and_exact_source_namespace(self):
        env = {"CARGO_NET_OFFLINE": "true", "CARGO_HOME": "/owned/cache"}
        commands = []
        with patch("sifr_verify.cargo_setup.subprocess.check_output", return_value=REVISION), \
             patch("sifr_verify.cargo_setup.prepare_crate_test_binaries",
                   wraps=prepare_crate_test_binaries) as compiler_setup, \
             patch("sifr_verify.cargo_setup.prepare_area_graphs") as area_setup:
            prepare_cargo_cache(load_profile("merge"), env,
                                lambda args, **kw: commands.append((args, kw["env"])))
        self.assertEqual(commands[0][0], ["cargo", "fetch", "--locked"])
        self.assertEqual(commands[1][0], ["cargo", "fetch", "--locked", "--manifest-path",
                                         str(locked_fixture_manifests(load_profile("merge"))[0])])
        self.assertIn("sifr_verify.generated_cargo_setup", commands[2][0])
        self.assertEqual(commands[2][0][-1], REVISION)
        compiler_env = compiler_setup.call_args.args[1]
        self.assertIsNot(compiler_env, env)
        self.assertIs(area_setup.call_args.args[1], compiler_env)
        compiler_commands = 0
        for args, setup_env in commands:
            if setup_env is compiler_env:
                compiler_commands += 1
                self.assertEqual(setup_env["CARGO_NET_OFFLINE"], "true")
                self.assertIn("--offline", args)
            else:
                self.assertNotIn("CARGO_NET_OFFLINE", setup_env)
            self.assertEqual(setup_env["CARGO_HOME"], "/owned/cache")
        self.assertGreater(compiler_commands, 0)
        self.assertIn(REVISION, env["SIFR_GCQ_SHARED_ROOT"])
        self.assertEqual(env["CARGO_NET_OFFLINE"], "true")

    def test_workspace_failure_does_not_prepare_generated_graphs(self):
        calls = []
        def fail(args, **kw):
            calls.append(args)
            raise CommandFailed(101)
        with self.assertRaises(CommandFailed):
            prepare_cargo_cache(load_profile("merge"), {}, fail)
        self.assertEqual(len(calls), 1)

    def test_fixture_failure_prevents_generated_setup(self):
        calls = []
        def fail_fixture(args, **kwargs):
            calls.append(args)
            if "--manifest-path" in args:
                raise CommandFailed(101)
        with self.assertRaises(CommandFailed):
            prepare_cargo_cache(load_profile("merge"), {}, fail_fixture)
        self.assertEqual(len(calls), 2)
        self.assertNotIn("sifr_verify.generated_cargo_setup", calls[-1])

    def test_setup_failure_prevents_offline_switch_and_execution(self):
        profile = load_profile("merge")
        with patch.dict(os.environ, {}, clear=True), \
             patch("sifr_verify.profile_runner.load_profile", return_value=profile):
            runner = ProfileRunner("merge", [])
            with patch.object(runner, "prepare_step_budget", return_value=None), \
                 patch.object(runner, "admit_performance_reference"), \
                 patch.object(runner, "prepare_cargo_cache", side_effect=CommandFailed(101)), \
                 patch.object(runner, "run_guardrail") as guard, \
                 patch("sifr_verify.profile_runner.enable_profile_offline_cargo") as offline:
                with redirect_stdout(io.StringIO()):
                    self.assertEqual(runner.run(), 101)
                offline.assert_not_called()
                self.assertEqual([call.args[0] for call in guard.call_args_list], [
                "hir-maintainability", "file-size", "source-crate-dependency-direction",
                "submodule-ownership", "stdlib-manifest-schema"])

    def test_simulated_runner_output_preserves_release_step_evidence(self):
        from .release_evidence import build_steps

        output = io.StringIO()
        case = SetupPolicyTests("test_setup_failure_prevents_offline_switch_and_execution")
        with redirect_stdout(output):
            result = case.run(unittest.TestResult())
        self.assertTrue(result.wasSuccessful(), result.errors + result.failures)
        with tempfile.TemporaryDirectory() as directory:
            log = Path(directory) / "profile.log"
            log.write_text(
                "[sifr-lane-step] name=cargo_cache_setup elapsed_ms=1 status=pass\n"
                + output.getvalue(),
                encoding="utf-8",
            )
            self.assertEqual(build_steps(log), [{
                "name": "cargo_cache_setup", "status": "pass",
                "elapsed_ms": 1, "suite_results": [],
            }])

    def test_constructor_does_not_build_before_preparation(self):
        with patch("sifr_verify.profile_runner.resolve_sifr_binary") as resolve:
            ProfileRunner("merge", [])
            resolve.assert_not_called()

    def test_complete_positive_selection_ignores_execution_filters(self):
        quality = quality_module()
        with patch.dict(os.environ, {"SIFR_GCQ_MAX_ENTRIES": "1", "SIFR_GCQ_ENTRY_IDS": "missing"}):
            selected = preparation_entries(quality, ["representative"])
        expected = [entry for entry in quality.load_manifest(quality.MANIFEST)
                    if entry.group in quality.POSITIVE_GROUPS]
        self.assertEqual(selected, expected)
        self.assertGreater(len(selected), 12)
        self.assertIn("demos-required", {entry.group for entry in selected})

    def test_full_selection_includes_companions(self):
        quality = quality_module()
        extra = quality.Entry("extra", "companions", "extra.sifr", "build", "test")
        with patch.object(quality, "authoritative_companion_entries", return_value=[(None, extra)]):
            self.assertIn(extra, preparation_entries(quality, ["full"]))
            self.assertIn(extra, preparation_entries(quality, ["companions"]))

    def test_locked_fetch_preserves_graph(self):
        commands = []
        before = portable_graph(self.root, REVISION)
        actual = fetch_generated_graph(self.root, REVISION,
                                       lambda args, **kw: commands.append(args))
        self.assertEqual(actual, before)
        self.assertEqual(commands, [["cargo", "fetch", "--locked", "--manifest-path", str(self.manifest)]])

    def test_local_dependency_rejected_before_fetch(self):
        self.manifest.write_text(self.manifest.read_text() + 'other = { path = "/tmp/local" }\n')
        with self.assertRaisesRegex(ValueError, "local dependency"):
            fetch_generated_graph(self.root, REVISION, lambda *a, **k: self.fail("fetch ran"))

    def test_stale_manifest_revision_rejected(self):
        with self.assertRaisesRegex(ValueError, "stale Sifr revision"):
            portable_graph(self.root, "b" * 40)

    def test_stale_lock_revision_rejected(self):
        self.lock.write_text(self.lock.read_text().replace(REVISION, "b" * 40))
        with self.assertRaisesRegex(ValueError, "nonportable or stale"):
            portable_graph(self.root, REVISION)

    def test_sqlite_patch_and_lock_are_exact(self):
        manifest = self.manifest.read_text()
        lock = self.lock.read_text()
        patch_text = ('\n[patch.crates-io]\n'
                      f'libsqlite3-sys = {{ git = "{GIT_SOURCE}", rev = "{REVISION}" }}\n')
        native_lock = ('\n[[package]]\nname = "libsqlite3-sys"\nversion = "0.38.2"\n'
                       f'source = "git+{GIT_SOURCE}?rev={REVISION}#{REVISION}"\n')
        self.manifest.write_text(manifest + patch_text)
        self.lock.write_text(lock + native_lock)
        portable_graph(self.root, REVISION)
        for invalid in ("", patch_text.replace(REVISION, "b" * 40),
                        patch_text.replace('git =', 'path ='),
                        patch_text + 'extra = "1"\n',
                        patch_text + '[replace]\n"foo:1.0.0" = { path = "/local" }\n'):
            with self.subTest(manifest=invalid):
                self.manifest.write_text(manifest + invalid)
                with self.assertRaises(ValueError):
                    portable_graph(self.root, REVISION)
        self.manifest.write_text(manifest + patch_text)
        for source in ("registry+https://github.com/rust-lang/crates.io-index",
                       f"git+{GIT_SOURCE}?rev={'b' * 40}#{'b' * 40}", ""):
            with self.subTest(source=source):
                self.lock.write_text(lock + native_lock.replace(
                    f"git+{GIT_SOURCE}?rev={REVISION}#{REVISION}", source))
                with self.assertRaisesRegex(ValueError, "nonportable or stale"):
                    portable_graph(self.root, REVISION)

    def test_authoring_test_prebuild_selection_and_failure(self):
        calls = []
        runner = lambda args, **kw: calls.append(args)
        prepare_authoring_test_binaries({"selected_areas": []}, {}, runner)
        self.assertEqual(calls, [])
        profile = {"selected_areas": [{"area": "python_interop",
                                      "suites": ["lsp-declaration-authoring"]}]}
        prepare_authoring_test_binaries(profile, {}, runner)
        self.assertEqual(calls, [
            ["cargo", "test", "--locked", "--offline", "--no-run", "-p", package]
            for package in ("sifr_lsp", "sifr_driver", "sifr_analysis")
        ])
        def fail(*args, **kw):
            raise CommandFailed(101)
        with self.assertRaises(CommandFailed):
            prepare_authoring_test_binaries(profile, {}, fail)

    def test_tooling_preparation_matches_selected_execution_environments(self):
        calls = []
        def run(args, **kw):
            calls.append((args[-1], kw["env"].copy()))
        def profile(suites):
            return {"selected_areas": [{"area": "developer_tooling", "suites": suites}]}
        prepare_tooling_test_binaries(profile(["lsp-smoke"]), {}, run)
        self.assertEqual(calls, [])
        original = {"CARGO_BUILD_JOBS": "2"}
        prepare_tooling_test_binaries(profile(["static"]), original, run)
        self.assertEqual(calls, [
            ("sifr_lint", original),
            ("sifr_analysis", {**original, "CARGO_INCREMENTAL": "0"})])
        self.assertNotIn("CARGO_INCREMENTAL", original)
        calls.clear()
        explicit = {"CARGO_INCREMENTAL": "1"}
        prepare_tooling_test_binaries(profile(["full", "static"]), explicit, run)
        self.assertEqual(calls, [(name, explicit) for name in
                                ("sifr_lint", "sifr_analysis", "sifr_format", "sifr_analysis")])
        calls.clear()
        prepare_tooling_test_binaries(profile(["formatter", "analysis"]), {}, run)
        self.assertEqual(calls, [("sifr_format", {}), ("sifr_analysis", {})])
        def fail(*args, **kw):
            raise CommandFailed(101)
        with self.assertRaises(CommandFailed):
            prepare_tooling_test_binaries(profile(["static"]), {}, fail)

    def test_performance_preparation_selects_benchmarks_and_propagates_failure(self):
        calls = []
        def profile(suites):
            return {"selected_areas": [{"area": "performance", "suites": suites}]}
        runner = lambda args, **kw: calls.append(args)
        prepare_performance_binaries({"selected_areas": []}, {}, runner)
        self.assertEqual(calls, [])
        expected = [
            ["cargo", "build", "--locked", "--offline", "-p", "sifr"],
            ["cargo", "build", "--locked", "--offline", "-p", "sifr_frontend",
             "--bin", "frontend_query_bench"],
        ]
        for suites in (["smoke"], ["representative"], ["full"], ["smoke", "full"]):
            calls.clear()
            prepare_performance_binaries(profile(suites), {}, runner)
            self.assertEqual(calls, expected)
        guards = [
            ["cargo", "test", "--locked", "--offline", "--no-run", "-p", package, "--lib"]
            for package in ("sifr_syntax", "sifr_frontend")
        ]
        for suites, wanted in [
            (["frontend-syntax-guardrails"], guards),
            (["smoke", "frontend-syntax-guardrails"], expected + guards),
        ]:
            calls.clear()
            prepare_performance_binaries(profile(suites), {}, runner)
            self.assertEqual(calls, wanted)
        def fail(*args, **kw):
            raise CommandFailed(101)
        for suites in (["smoke"], ["frontend-syntax-guardrails"]):
            with self.assertRaises(CommandFailed):
                prepare_performance_binaries(profile(suites), {}, fail)

    def test_generated_oracle_preparation_selection_environment_and_failure(self):
        calls = []
        env = {"CARGO_TARGET_DIR": "/owned/target", "CARGO_BUILD_JOBS": "2"}
        def profile(suites):
            return {"selected_areas": [{"area": "cpython_differential", "suites": suites}]}
        def run(args, **kw):
            calls.append((args, kw["env"]))
        for suites in ([], ["policy"], ["hand_seeded_merge"]):
            prepare_generated_oracle_binary(profile(suites), env, run)
        self.assertEqual(calls, [])
        command = ["cargo", "build", "--release", "-p", "sifr", "--locked", "--offline"]
        for suites in (["generated_broader"], ["generated_minimized_seeds"],
                       ["generated_broader", "generated_minimized_seeds"]):
            calls.clear()
            prepare_generated_oracle_binary(profile(suites), env, run)
            prepare = [sys.executable, str(REPO_ROOT / "verification/areas/cpython_differential/"
                                           "checks/prepare_generated.py")]
            for suite in sorted(suites):
                prepare.extend(["--suite", suite])
            self.assertEqual(calls, [(command, env), (prepare, env)])
            self.assertIs(calls[0][1], env)
            self.assertIs(calls[1][1], env)
        def fail(*args, **kw):
            raise CommandFailed(101)
        with self.assertRaises(CommandFailed):
            prepare_generated_oracle_binary(profile(["generated_broader"]), env, fail)

    def test_generated_program_preparation_failure_propagates(self):
        def fail_programs(args, **kw):
            if args[0] == sys.executable:
                raise CommandFailed(101)
        profile = {"selected_areas": [{"area": "cpython_differential",
                                       "suites": ["generated_broader"]}]}
        with self.assertRaises(CommandFailed):
            prepare_generated_oracle_binary(profile, {}, fail_programs)

    def test_generated_oracle_preparation_is_enrolled_in_profile_setup(self):
        for name, expected in (("create-pr", False), ("merge", False),
                               ("nightly", True), ("release", True)):
            with self.subTest(profile=name):
                calls = []
                with patch("sifr_verify.cargo_setup.subprocess.check_output", return_value=REVISION):
                    prepare_cargo_cache(load_profile(name), {"CARGO_TARGET_DIR": "/owned/target"},
                                        lambda args, **kw: calls.append((args, kw["env"])))
                release_builds = [(args, env) for args, env in calls
                                  if args[:3] == ["cargo", "build", "--release"]]
                self.assertEqual(len(release_builds), int(expected))
                if expected:
                    self.assertEqual(release_builds[0][1]["CARGO_TARGET_DIR"], "/owned/target")

    def test_demo_preparation_selection_and_failure(self):
        calls = []
        runner = lambda args, **kw: calls.append(args)
        prepare_maintained_demo_cache({"selected_areas": []}, {}, runner)
        prepare_maintained_demo_cache({"selected_areas": [
            {"area": "rust_interop", "suites": ["tiers"]}]}, {}, runner)
        self.assertEqual(calls, [])
        profile = {"name": "create-pr", "selected_areas": [
            {"area": "rust_interop", "suites": ["matrix"]}]}
        prepare_maintained_demo_cache(profile, {}, runner)
        self.assertEqual(len(calls), 1)
        self.assertTrue(calls[0][1].endswith("/check_maintained_rust_demos.py"))
        self.assertEqual(calls[0][2], "--output")
        self.assertTrue(Path(calls[0][3]).is_relative_to(REPO_ROOT / "target"))
        def fail(*args, **kw):
            raise CommandFailed(101)
        with self.assertRaises(CommandFailed):
            prepare_maintained_demo_cache(profile, {}, fail)

    def test_missing_lock_rejected_before_fetch(self):
        self.lock.unlink()
        with self.assertRaises(FileNotFoundError):
            fetch_generated_graph(self.root, REVISION, lambda *a, **k: self.fail("fetch ran"))

    def test_changed_lock_rejected(self):
        def change(*args, **kw):
            self.lock.write_text(self.lock.read_text() + "\n# unexpected rewrite\n")
        with self.assertRaisesRegex(ValueError, "preparation changed"):
            fetch_generated_graph(self.root, REVISION, change)


def policy_checks() -> None:
    from .ci_smoke_setup_checks import SmokePreparationChecks
    suite = unittest.TestSuite(unittest.defaultTestLoader.loadTestsFromTestCase(case)
                               for case in (SetupPolicyTests, FixtureSetupPolicyTests, CrateSetupPolicyTests,
                                            SysrootSetupPolicyTests, SmokePreparationChecks))
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    if not result.wasSuccessful():
        raise AssertionError("generated Cargo setup policy checks failed")


def clean_cache_checks() -> None:
    """Exercise the actual profile prelude, then resolve every prepared graph offline."""
    (REPO_ROOT / "target").mkdir(exist_ok=True)
    root = Path(tempfile.mkdtemp(prefix="b11-clean-cache-", dir=REPO_ROOT / "target"))
    cargo_home = root / "cargo-home"
    cargo_home.mkdir()
    revision = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO_ROOT, text=True).strip()
    with patch.dict(os.environ, {"CARGO_HOME": str(cargo_home), "CARGO_NET_OFFLINE": "true"}):
        # No manual population: the production prelude owns every selected graph.
        runner = ProfileRunner("merge", [])
        runner.prepare_cargo_cache()
        enable_offline_cargo(runner.env)
        fixture_graphs = []
        for manifest in locked_fixture_manifests(runner.profile):
            before = fixture_graph_hashes(manifest)
            subprocess.run(
                ["cargo", "metadata", "--format-version", "1", "--locked", "--offline",
                 "--manifest-path", str(manifest)],
                cwd=REPO_ROOT, env=runner.env, text=True, capture_output=True, check=True,
            )
            if fixture_graph_hashes(manifest) != before:
                raise AssertionError("offline fixture resolution mutated its graph")
            fixture_graphs.append({"manifest": str(manifest.relative_to(REPO_ROOT)), **before})
        report_path = REPO_ROOT / "target/verification/areas/generated-cargo-setup-merge.json"
        report = json.loads(report_path.read_text())
        if report["revision"] != revision:
            raise AssertionError("preparation did not cover the candidate SHA")
        quality = quality_module()
        expected = preparation_entries(quality, ["representative"])
        if {entry.id for entry in expected} != {entry["id"] for entry in report["entries"]}:
            raise AssertionError("incomplete generated preparation coverage")
        demand = set()
        for record in report["entries"]:
            crate_root = REPO_ROOT / record["crate_root"]
            before = portable_graph(crate_root, revision)
            metadata = subprocess.run(
                ["cargo", "metadata", "--format-version", "1", "--locked", "--offline",
                 "--manifest-path", str(crate_root / "Cargo.toml")],
                cwd=REPO_ROOT, env=runner.env, text=True, capture_output=True, check=True,
            )
            demand.update(package["name"] for package in json.loads(metadata.stdout)["packages"])
            if portable_graph(crate_root, revision) != before:
                raise AssertionError("offline resolution mutated the graph")
            print(f"[b11-offline-graph] {record['id']} status=pass", flush=True)
        if not {"sifr_runtime", "sifr_stdlib"}.issubset(demand):
            raise AssertionError("runtime and stdlib graph demand was not covered")

        # Re-enter materialization through the same path used by corpus, positive
        # Clippy, and demos, after preparation has made networking unavailable.
        with patch.dict(os.environ, runner.env):
            entries = {entry.id: entry for entry in expected}
            for mode, group in (("corpus", "concurrency-runtime-readiness"),
                                ("clippy", "stdlib-flows"), ("demos", "demos-required")):
                entry = next(entry for entry in expected if entry.group == group)
                crate_root = quality.materialize_entry(entries[entry.id], root / mode)
                run_command(["cargo", "metadata", "--format-version", "1", "--no-deps", "--locked",
                             "--offline", "--manifest-path", str(crate_root / "Cargo.toml")], env=runner.env)

        # A genuinely empty second cache must fail on that same exact Git graph.
        empty_home = root / "negative-empty-home"
        empty_home.mkdir()
        graph = REPO_ROOT / report["entries"][0]["crate_root"]
        command = ["cargo", "metadata", "--format-version", "1", "--locked", "--offline",
                   "--manifest-path", str(graph / "Cargo.toml")]
        negative = subprocess.run(command, cwd=REPO_ROOT,
                                  env={**runner.env, "CARGO_HOME": str(empty_home)},
                                  text=True, capture_output=True)
        if negative.returncode == 0 or "offline" not in negative.stderr:
            raise AssertionError("unprepared exact Git graph did not fail offline")
        (root / "negative-empty-cache.log").write_text(negative.stderr)

        # A real generated manifest with changed dependency requirements cannot
        # escape --locked in production fetch, even with its cache prepared.
        invalid = root / "negative-lock-drift"
        shutil.copytree(graph, invalid)
        manifest = invalid / "Cargo.toml"
        packages = tomllib.loads((invalid / "Cargo.lock").read_text())["package"]
        dependency = next(package for package in packages if package["name"] == "num-traits")
        manifest.write_text(manifest.read_text() + '\n[dependencies.b11_lock_drift]\n'
                            f'package = "num-traits"\nversion = "={dependency["version"]}"\n')
        def locked_failure(args, **kw):
            proc = subprocess.run(args, cwd=REPO_ROOT, text=True, capture_output=True, **kw)
            (root / "negative-lock-drift.log").write_text(proc.stderr)
            if proc.returncode and "--locked" in proc.stderr:
                raise CommandFailed(proc.returncode)
            raise AssertionError(f"lock drift did not fail specifically under --locked: {proc.stderr}")
        try:
            fetch_generated_graph(invalid, revision, locked_failure)
        except CommandFailed:
            pass
        else:
            raise AssertionError("changed generated requirements passed locked fetch")
    evidence = {"revision": revision, "status": "pass", "prepared_graphs": len(expected),
                "offline_graphs": len(expected), "entry_modes": ["corpus", "clippy", "demos"],
                "runtime_and_stdlib": True, "negative_checks": ["empty-cache", "lock-drift"],
                "fixture_graphs": fixture_graphs,
                "cargo_home": str(cargo_home), "setup_report": str(report_path),
                "setup_report_sha256": hashlib.sha256(report_path.read_bytes()).hexdigest()}
    destination = REPO_ROOT / "target/verification/areas/generated-cargo-clean-cache.json"
    destination.write_text(json.dumps(evidence, indent=2) + "\n")
    print(json.dumps(evidence, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=["policy", "clean-cache"])
    args = parser.parse_args()
    if args.mode == "policy":
        policy_checks()
    else:
        clean_cache_checks()
