"""Focused policy and clean-cache qualification of generated Cargo preparation."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import tempfile
import tomllib
import unittest
from pathlib import Path
from unittest.mock import patch

from .cargo_setup import enable_offline_cargo, prepare_cargo_cache
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
        with patch("sifr_verify.cargo_setup.subprocess.check_output", return_value=REVISION):
            prepare_cargo_cache(load_profile("merge"), env,
                                lambda args, **kw: commands.append((args, kw["env"])))
        self.assertEqual(commands[0][0], ["cargo", "fetch", "--locked"])
        self.assertIn("sifr_verify.generated_cargo_setup", commands[1][0])
        self.assertEqual(commands[1][0][-1], REVISION)
        for _, setup_env in commands:
            self.assertNotIn("CARGO_NET_OFFLINE", setup_env)
            self.assertEqual(setup_env["CARGO_HOME"], "/owned/cache")
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

    def test_setup_failure_prevents_offline_switch_and_execution(self):
        with patch.dict(os.environ, {}, clear=True):
            runner = ProfileRunner("merge", [])
            with patch.object(runner, "prepare_step_budget", return_value=None), \
                 patch.object(runner, "prepare_cargo_cache", side_effect=CommandFailed(101)), \
                 patch.object(runner, "run_guardrail") as guard, \
                 patch("sifr_verify.profile_runner.enable_profile_offline_cargo") as offline:
                self.assertEqual(runner.run(), 101)
                offline.assert_not_called()
                guard.assert_not_called()

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
    result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(SetupPolicyTests))
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
        # No manual fetch/cache population: the production prelude owns both graphs.
        runner = ProfileRunner("merge", [])
        runner.prepare_cargo_cache()
        enable_offline_cargo(runner.env)
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
                "cargo_home": str(cargo_home), "setup_report": str(report_path),
                "setup_report_sha256": hashlib.sha256(report_path.read_bytes()).hexdigest()}
    destination = REPO_ROOT / "target/verification/areas/item12k-b11-clean-cache.json"
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
