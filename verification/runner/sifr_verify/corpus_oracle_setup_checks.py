"""Regression checks for explicit, input-bound corpus oracle preparation."""
from __future__ import annotations

import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from .cargo_sysroot_setup_checks import sysroot_module
from .corpus_ownership import CORPUS_CASE


class CorpusOraclePreparationTests(unittest.TestCase):
    def setUp(self):
        sysroot_module()
        import corpus_preparation
        import package_build
        self.preparation = corpus_preparation
        self.builder = package_build
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.snapshot = self.root / "target/sysroot_release/corpus-producer-source"
        self.snapshot.mkdir(parents=True)
        (self.snapshot / "sysroot.toml").write_text("source version")
        self.binary = corpus_preparation.oracle_path(self.root, "host")
        self.binary.parent.mkdir(parents=True)
        self.binary.write_text("oracle executable")
        self.binary.with_name("sifr").write_text("compiler executable")
        self.development = {name: str(self.root / name) for name in ("driver_test",)}
        for path in self.development.values():
            Path(path).write_text("development executable")
        self.env = {"RUSTFLAGS": "--cfg caller", "CARGO_PROFILE_RELEASE_OPT_LEVEL": "3"}
        self.inventory = {"input_digest": "current inputs"}
        def output(command, **kwargs):
            if command[0] == str(self.binary):
                return f"sifr-corpus-oracle {package_build.RELEASE_VERSION}\n"
            if command == [self.development["driver_test"], "--list", "--ignored"]:
                return CORPUS_CASE + ": test\n\n1 test, 0 benchmarks\n"
            if command == ["rustc", "-vV"]:
                return "release: pinned\nhost: host\n"
            if command == ["cargo", "--version"]:
                return "cargo pinned\n"
            raise AssertionError(command)
        for mock in (patch.object(corpus_preparation, "inventory", return_value=self.inventory),
                     patch.object(corpus_preparation.subprocess, "check_output", side_effect=output)):
            mock.start()
            self.addCleanup(mock.stop)

    def record(self):
        self.preparation.record_prepared(self.root, self.env, self.snapshot, "host", self.development)

    def require(self, env=None):
        return self.preparation.require_prepared(
            self.root, self.env if env is None else env, self.snapshot)

    def test_missing_receipt_and_invalid_receipt_fail_without_building(self):
        with patch.object(self.builder.subprocess, "run") as run:
            for value in (None, "not json", "{}"):
                with self.subTest(value=value):
                    if value is not None:
                        self.preparation.receipt_path(self.root).write_text(value)
                    with self.assertRaises(RuntimeError):
                        self.require()
            run.assert_not_called()

    def test_unchanged_inputs_run_both_profiles_without_cargo(self):
        self.record()
        output = self.root / "published"
        env = self.env | {"SIFR_DX8_CORPUS_OUTPUT": str(output), "SIFR_DX8_CORPUS_CASES": "partial"}
        def run(command, **kwargs):
            destination = Path(kwargs["env"]["SIFR_DX8_CORPUS_OUTPUT"])
            destination.mkdir()
            (destination / "coverage.json").write_text("fresh report")
            return subprocess.CompletedProcess(command, 0,
                (f"test {CORPUS_CASE} ... ok\n"
                 "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 871 filtered out;\n").encode())
        with patch.object(self.builder.subprocess, "run", side_effect=run) as execute, \
             patch.object(self.builder, "compare_profiles") as compare:
            self.builder.run_corpus(self.root, env)
        self.assertEqual(execute.call_count, 2)
        calls = execute.call_args_list
        self.assertEqual(calls[0].args[0][0], self.development["driver_test"])
        self.assertEqual(calls[1].args[0][0], str(self.binary))
        self.assertEqual(calls[0].kwargs["env"]["SIFR_SYSROOT"], str(self.root))
        self.assertEqual(calls[1].kwargs["env"]["SIFR_SYSROOT"], str(self.snapshot))
        for call in calls:
            self.assertNotIn("SIFR_DX8_CORPUS_CASES", call.kwargs["env"])
        compare.assert_called_once()
        self.assertEqual((output / "coverage.json").read_text(), "fresh report")

    def test_profile_comparison_failure_does_not_publish_a_reference(self):
        self.record()
        output = self.root / "published"
        with patch.object(self.builder, "run_development"), \
             patch.object(self.builder.subprocess, "run"), \
             patch.object(self.builder, "compare_profiles", side_effect=AssertionError("different")):
            with self.assertRaises(AssertionError):
                self.builder.run_corpus(self.root, self.env | {"SIFR_DX8_CORPUS_OUTPUT": str(output)})
        self.assertFalse(output.exists())
        self.assertEqual(len(list(self.snapshot.parent.glob("corpus-profiles-*"))), 1)

    def test_content_and_build_environment_changes_invalidate_preparation(self):
        changes = [
            ("oracle", lambda: self.binary.write_text("changed oracle")),
            ("development driver test", lambda: Path(self.development["driver_test"]).write_text("changed")),
            ("compiler", lambda: self.binary.with_name("sifr").write_text("changed compiler")),
            ("snapshot", lambda: (self.snapshot / "sysroot.toml").write_text("changed source")),
            ("snapshot addition", lambda: (self.snapshot / "new").write_text("new source")),
            ("inputs", lambda: self.inventory.update(input_digest="changed inputs")),
        ]
        for label, change in changes:
            with self.subTest(label=label):
                self.record()
                change()
                with self.assertRaisesRegex(RuntimeError, "stale"):
                    self.require()
        self.record()
        for key in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_PROFILE_RELEASE_OPT_LEVEL",
                    "RUSTC_WRAPPER", "CARGO_BUILD_TARGET", "CARGO_TARGET_DIR", "CARGO_INCREMENTAL"):
            with self.subTest(key=key), self.assertRaisesRegex(RuntimeError, "stale"):
                self.require(self.env | {key: "changed"})
        self.assertEqual(self.require(self.env | {"UNRELATED": "safe"})["oracle"], str(self.binary))

    def test_deleted_artifacts_and_wrong_version_reject_preparation(self):
        self.record()
        with patch.object(self.preparation.subprocess, "check_output", return_value="wrong"):
            with self.assertRaises(RuntimeError):
                self.preparation.binding(self.root, self.env, self.snapshot, "host", self.development)
        self.binary.unlink()
        with self.assertRaises(RuntimeError):
            self.require()

    def test_failed_repreparation_removes_receipt_and_does_not_publish_success(self):
        self.record()
        with patch.object(self.builder, "host_triple", return_value="host"), \
             patch.object(self.builder, "prepare_source_snapshot") as snapshot, \
             patch.object(self.builder, "record_prepared") as record:
            def fail(*args, **kwargs):
                raise subprocess.CalledProcessError(101, args[0])
            with self.assertRaises(subprocess.CalledProcessError):
                self.builder.prepare(self.root, self.env, fail, corpus_only=True)
        snapshot.assert_not_called()
        record.assert_not_called()
        self.assertFalse(self.preparation.receipt_path(self.root).exists())

    def test_failed_binding_does_not_replace_previous_evidence(self):
        self.record()
        previous = self.preparation.receipt_path(self.root).read_bytes()
        with patch.object(self.preparation, "binding", side_effect=RuntimeError("not prepared")):
            with self.assertRaises(RuntimeError):
                self.record()
        self.assertEqual(self.preparation.receipt_path(self.root).read_bytes(), previous)
        self.assertFalse(list(self.snapshot.parent.glob("corpus-oracle-prepared.*.tmp")))

    def test_corpus_flag_cannot_be_used_for_artifact_packaging(self):
        # This rejects the mode before invoking Cargo or touching release artifacts.
        script = Path(__file__).resolve().parents[3] / "scripts/distribution/build_release_artifacts.sh"
        result = subprocess.run(["bash", str(script), "--version", "0.1.0-beta.1300",
                                 "--output-dir", str(self.root / "artifacts"),
                                 "--cargo-build", "--prepare-corpus"], text=True, capture_output=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("--prepare-corpus requires --prepare-only", result.stderr)

    def test_prepared_libtest_must_list_the_exact_corpus_case(self):
        original = self.preparation.subprocess.check_output.side_effect
        for listing in ("other_case: test\n", (CORPUS_CASE + ": test\n") * 2):
            def output(command, **kwargs):
                if command == [self.development["driver_test"], "--list", "--ignored"]:
                    return listing
                return original(command, **kwargs)
            with self.subTest(listing=listing), patch.object(
                self.preparation.subprocess, "check_output", side_effect=output
            ), self.assertRaisesRegex(RuntimeError, "lacks the exact corpus"):
                self.record()
        self.assertFalse(self.preparation.receipt_path(self.root).exists())

    def test_failed_development_build_cannot_publish_preparation(self):
        self.record()
        with patch.object(self.builder, "host_triple", return_value="host"), \
             patch.object(self.builder, "prepare_development",
                          side_effect=subprocess.CalledProcessError(101, ["cargo", "build"])), \
             patch.object(self.builder, "prepare_source_snapshot") as snapshot, \
             patch.object(self.builder, "record_prepared") as record:
            with self.assertRaises(subprocess.CalledProcessError):
                self.builder.prepare(self.root, self.env, lambda *a, **k: None, corpus_only=True)
        snapshot.assert_not_called()
        record.assert_not_called()
        self.assertFalse(self.preparation.receipt_path(self.root).exists())

    def test_failed_oracle_execution_keeps_partial_evidence_without_publication(self):
        self.record()
        published = self.root / "published"
        def fail(command, **kwargs):
            output = Path(kwargs["env"]["SIFR_DX8_CORPUS_OUTPUT"])
            output.mkdir()
            (output / "failed-case.source.rs").write_text("partial evidence")
            raise subprocess.CalledProcessError(1, command)
        with patch.object(self.builder.subprocess, "run", side_effect=fail), \
             patch.object(self.builder, "compare_profiles") as compare:
            with self.assertRaises(subprocess.CalledProcessError):
                self.builder.run_corpus(self.root, self.env | {"SIFR_DX8_CORPUS_OUTPUT": str(published)})
        compare.assert_not_called()
        self.assertFalse(published.exists())
        evidence = list(self.snapshot.parent.glob("corpus-profiles-*/development/failed-case.source.rs"))
        self.assertEqual(len(evidence), 1)
        self.assertEqual(evidence[0].read_text(), "partial evidence")
