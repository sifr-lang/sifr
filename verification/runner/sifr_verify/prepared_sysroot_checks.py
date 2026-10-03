"""Preparation receipts cannot skip runtime assertions or trust drifted bytes."""
from datetime import UTC, datetime, timedelta
import json
import os
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest.mock import patch
import uuid
from types import SimpleNamespace

from . import prepared_sysroot as receipts
from .execution_identity import EvidenceError, artifact_identity, digest
from .graph_retirement import GRAPH_PATHS
from .resource_admission import ResourceError
from .compressed_artifacts import compress, decoded_identity, restore


class PreparationTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.env = {receipts.OWNER_VARIABLE: str(uuid.uuid4())}
        self.graph = self.root / GRAPH_PATHS[0]
        self.output = self.graph / "debug/sifr"
        self.expected = {"source": "exact-input", "kind": "source"}
        self.addCleanup(patch.stopall)
        patch.object(receipts, "key", return_value=self.expected).start()
        patch.object(receipts, "admit", return_value={}).start()
        patch.object(receipts, "recipe", return_value=(
            ["build"], {}, self.output, GRAPH_PATHS[0])).start()
        self.commands = []
        def run(argv, **kwargs):
            self.commands.append(argv)
            if argv == ["build"]:
                self.output.parent.mkdir(parents=True, exist_ok=True)
                self.output.write_bytes(b"actual-compiler-output")
                self.output.chmod(0o755)
            elif argv[:2] == ["cargo", "clean"]:
                shutil.rmtree(Path(argv[-1]))
            else:
                raise AssertionError(argv)
        patch.object(receipts, "checked", side_effect=run).start()
        patch("sifr_verify.graph_retirement.active_builds", return_value=[]).start()

    def prepare(self):
        return receipts.prepare("source", root=self.root, env=self.env)

    def rewrite(self, callback):
        directory, _ = receipts.store(self.root, self.env)
        path = directory / "source.json"
        value = json.loads(path.read_text())
        callback(value)
        value.pop("receipt_digest")
        value["receipt_digest"] = digest(value)
        path.write_text(json.dumps(value))

    def consume(self):
        return receipts.consume("source", root=self.root, env=self.env)

    def test_graph_retires_after_actual_build_and_exact_readonly_output_survives(self):
        payload = self.prepare()
        self.assertFalse(self.graph.exists())
        binary = self.consume()
        self.assertEqual(binary.read_bytes(), b"actual-compiler-output")
        self.assertEqual(binary.stat().st_mode & 0o222, 0)
        self.assertEqual(payload["runtime_assertions_executed"], 0)
        self.assertEqual([argv[0] for argv in self.commands], ["build", "cargo"])
        with self.assertRaises(EvidenceError):
            self.prepare()

    def test_failed_build_never_retires_graph_or_publishes_receipt(self):
        receipts.checked.side_effect = RuntimeError("build failed")
        with self.assertRaises(RuntimeError):
            self.prepare()
        self.assertTrue(self.graph.exists())
        with self.assertRaises(FileNotFoundError):
            self.consume()

    def test_input_drift_prevents_retirement(self):
        receipts.key.side_effect = [self.expected, {"source": "changed"}]
        with self.assertRaises(EvidenceError):
            self.prepare()
        self.assertTrue(self.output.exists())
        self.assertEqual(len(self.commands), 1)

    def test_input_drift_at_consumption_rejects_old_binary(self):
        self.prepare()
        receipts.key.return_value = {"source": "changed"}
        with self.assertRaises(EvidenceError):
            self.consume()

    def test_expired_future_tampered_and_runtime_claim_receipts_are_rejected(self):
        self.prepare()
        for callback in (
            lambda row: row.update(finished_at=(datetime.now(UTC)-timedelta(days=2)).isoformat()),
            lambda row: row.update(finished_at=(datetime.now(UTC)+timedelta(days=2)).isoformat()),
            lambda row: row.update(finished_at=datetime.now(UTC).isoformat(), runtime_assertions_executed=1),
        ):
            self.rewrite(callback)
            with self.assertRaises(EvidenceError):
                self.consume()

    def test_artifact_drift_or_writable_output_rejects_consumption(self):
        self.prepare()
        binary = self.consume()
        binary.chmod(0o755)
        with self.assertRaises(EvidenceError):
            self.consume()
        binary.write_bytes(b"changed")
        binary.chmod(0o555)
        with self.assertRaises(EvidenceError):
            self.consume()

    def test_link_ancestor_cannot_redirect_output_to_another_owner(self):
        self.prepare()
        binary = self.consume()
        elsewhere = self.root / "elsewhere"
        binary.parent.rename(elsewhere)
        binary.parent.symlink_to(elsewhere, target_is_directory=True)
        with self.assertRaises(ResourceError):
            self.consume()

    def test_unknown_legacy_graph_is_preserved_with_independent_output_copy(self):
        self.graph.mkdir(parents=True)
        payload = self.prepare()
        self.assertFalse(payload["retirement"]["retired"])
        binary = self.consume()
        self.assertEqual(binary.read_bytes(), self.output.read_bytes())
        self.assertNotEqual(binary.stat().st_ino, self.output.stat().st_ino)
        self.assertEqual(len(self.commands), 1)

    def test_other_session_and_unowned_store_cannot_consume(self):
        self.prepare()
        with self.assertRaises(FileNotFoundError):
            receipts.consume("source", root=self.root, env={receipts.OWNER_VARIABLE: str(uuid.uuid4())})
        directory, _ = receipts.store(self.root, self.env)
        directory.chmod(0o777)
        with self.assertRaises(EvidenceError):
            self.consume()

    def test_external_build_tree_additions_byte_drift_and_directory_links(self):
        dependency = self.root / "dependencies"
        before = receipts.tree_identity(dependency)
        dependency.mkdir()
        source = dependency / "input.h"
        source.write_text("original")
        first = receipts.tree_identity(dependency)
        self.assertNotEqual(before, first)
        source.write_text("changed")
        self.assertNotEqual(first, receipts.tree_identity(dependency))
        (dependency / "unknown").symlink_to(self.root, target_is_directory=True)
        with self.assertRaises(EvidenceError):
            receipts.tree_identity(dependency)

    def test_packaged_archive_survives_its_graph_without_claiming_runtime_pass(self):
        self.graph = self.root / GRAPH_PATHS[1]
        binary = self.graph / "host/release/sifr"
        directory, _ = receipts.store(self.root, self.env)
        output = directory / "archive/package.tar.gz"
        receipts.recipe.return_value = (["package"], {}, output, GRAPH_PATHS[1])
        def run(argv, **kwargs):
            if argv == ["package"]:
                binary.parent.mkdir(parents=True)
                binary.write_bytes(b"native-packaged-compiler")
                output.parent.mkdir()
                output.write_bytes(b"verified-archive-fixture")
            else:
                shutil.rmtree(Path(argv[-1]))
        receipts.checked.side_effect = run
        patch.object(receipts, "producer", return_value=SimpleNamespace(
            host_target=lambda env: "host", package_compiler_path=lambda root, env, host: binary)).start()
        payload = receipts.prepare("package", root=self.root, env=self.env)
        self.assertFalse(self.graph.exists())
        consumed = receipts.consume("package", root=self.root, env=self.env)
        self.assertEqual(consumed.read_bytes(), b"verified-archive-fixture")
        self.assertEqual(payload["runtime_assertions_executed"], 0)

    def test_compressed_custody_restores_exact_executable_bytes(self):
        source = self.root / "compiler"
        source.write_bytes(b"compiler"*1000)
        source.chmod(0o755)
        encoded = self.root / "compiler.gz"
        copy = compress(source, encoded, max_encoded_bytes=4096)
        source.unlink()
        output = restore(copy, self.root / "restored")
        self.assertEqual(output.read_bytes(), b"compiler"*1000)
        self.assertEqual(output.stat().st_mode & 0o777, 0o555)
        self.assertEqual(artifact_identity(encoded), copy["retained"])
        encoded.chmod(0o600)
        encoded.write_bytes(b"tampered")
        with self.assertRaises(ResourceError):
            restore(copy, self.root / "invalid")
        self.assertFalse((self.root / "invalid").exists())

    def test_encoded_and_decoded_bounds_preserve_the_original(self):
        source = self.root / "compiler"
        source.write_bytes(os.urandom(4096))
        expected = artifact_identity(source)
        with self.assertRaises(ResourceError):
            compress(source, self.root / "too-small.gz", max_encoded_bytes=16)
        self.assertEqual(artifact_identity(source), expected)
        copy = compress(source, self.root / "valid.gz", max_encoded_bytes=8192)
        with self.assertRaises(ResourceError):
            decoded_identity(Path(copy["retained"]["path"]), limit=100)


def policy_checks():
    result = unittest.TestResult()
    unittest.defaultTestLoader.loadTestsFromTestCase(PreparationTests).run(result)
    if not result.wasSuccessful():
        raise AssertionError(result.errors + result.failures)


if __name__ == "__main__":
    unittest.main()
