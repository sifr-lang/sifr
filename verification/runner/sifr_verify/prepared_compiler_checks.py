"""Real filesystem/process controls for admitted compiler products."""
from __future__ import annotations

import json
import os
import selectors
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

from .paths import REPO_ROOT

sys.path.insert(0, str(REPO_ROOT / "verification/areas/common"))
import prepared_compiler as products
import sifr_binary


class PreparedCompilerTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.addCleanup(products._release_all)
        self.root = Path(self.temporary.name)
        self.source = self.root / "cargo-sifr"
        self.source.write_text("#!/bin/sh\nprintf 'prepared\\n'\n")
        self.source.chmod(0o755)

    def publish(self, owner):
        return owner.publish(self.source, identity_reader=lambda _: "canonical-product-id",
                             producer={"command": list(sifr_binary.PREPARE_COMMAND)})

    def test_consumers_share_copied_bytes_without_invoking_cargo(self):
        with products.PreparedCompilerSession(self.root) as owner:
            binary = self.publish(owner)
            self.assertNotEqual(binary.stat().st_ino, self.source.stat().st_ino)
            self.source.write_text("later producer replaced the mutable Cargo output")
            env = {products.REFERENCE_ENV: owner.token}
            with patch.object(sifr_binary, "_prepare", side_effect=AssertionError("consumer built")):
                self.assertEqual(sifr_binary.resolve_sifr_binary(self.root, env=env), binary)
                self.assertEqual(sifr_binary.resolve_sifr_binary(self.root, env=env), binary)
            self.assertEqual(subprocess.check_output([str(binary)], text=True), "prepared\n")
            self.assertEqual(json.loads(owner.reference.read_text())["compiler_build_id"],
                             "canonical-product-id")
        self.assertFalse(binary.exists())

    def test_equivalent_override_cannot_escape_owned_product_lifetime(self):
        with products.PreparedCompilerSession(self.root) as owner:
            binary = self.publish(owner)
            env = {products.REFERENCE_ENV: owner.token, "SIFR_GCQ_BIN": str(self.source)}
            self.assertEqual(sifr_binary.resolve_sifr_binary(
                self.root, explicit_env_var="SIFR_GCQ_BIN", env=env), binary)
            self.source.write_text("changed override")
            with self.assertRaisesRegex(RuntimeError, "does not match"):
                sifr_binary.resolve_sifr_binary(self.root, explicit_env_var="SIFR_GCQ_BIN", env=env)

    def test_new_invocation_prepares_again_and_failure_retires_previous_product(self):
        from . import fixture_execution, profile_runner
        from types import SimpleNamespace
        runner = profile_runner.ProfileRunner("create-pr", [], correctness_only=True)
        runner.env = {products.REFERENCE_ENV: "/expired-inherited-reference"}
        binaries = []
        def steps():
            self.assertNotIn(products.REFERENCE_ENV, runner.env)
            runner.prepare_compiler_identity()
            binary = sifr_binary.resolve_sifr_binary(self.root, env=runner.env)
            binaries.append(binary)
            self.assertEqual(runner.env["SIFR_SYSROOT"], str(self.root))
            if len(binaries) == 2:
                raise RuntimeError("consumer failed")
            return 0
        def prepare(*_, **kwargs):
            self.assertNotIn(products.REFERENCE_ENV, kwargs["env"])
            self.source.write_text(self.source.read_text() + "# revised input\n")
            return self.source
        result = SimpleNamespace(returncode=0, cause="exit", truncated=False,
                                 stdout=json.dumps({"identity_kind": "product", "compiler_build_id": "id"}))
        environment = {key: value for key, value in os.environ.items()
                       if key not in ("SIFR_GCQ_BIN", "SIFR_RUNTIME_PLATFORM_BIN")}
        with patch.dict(os.environ, environment, clear=True), \
             patch.object(profile_runner, "REPO_ROOT", self.root), \
             patch.object(profile_runner, "prepare_sifr_binary", side_effect=prepare) as build, \
             patch.object(fixture_execution, "run_process", return_value=result), \
             patch.object(runner, "run_steps", side_effect=steps):
            self.assertEqual(runner.run(), 0)
            with self.assertRaisesRegex(RuntimeError, "consumer failed"):
                runner.run()
        self.assertEqual(build.call_count, 2)
        self.assertNotEqual(binaries[0], binaries[1])
        self.assertTrue(all(not binary.exists() for binary in binaries))
        self.assertNotIn(products.REFERENCE_ENV, runner.env)

    def test_expired_or_foreign_reference_never_falls_back_to_cargo(self):
        with products.PreparedCompilerSession(self.root) as owner:
            self.publish(owner)
            reference = owner.token
            with self.assertRaises(RuntimeError):
                products.consume_prepared_compiler(self.root / "other-checkout", reference)
        with patch.object(sifr_binary, "_prepare", side_effect=AssertionError("fallback")):
            with self.assertRaises(RuntimeError):
                sifr_binary.resolve_sifr_binary(
                    self.root, env={products.REFERENCE_ENV: str(reference)})

    def test_corruption_is_rejected_before_first_borrow_and_after_cached_borrow(self):
        for cached in (False, True):
            with self.subTest(cached=cached), products.PreparedCompilerSession(self.root) as owner:
                binary = self.publish(owner)
                if cached:
                    products.consume_prepared_compiler(self.root, owner.token)
                previous = binary.stat()
                binary.chmod(0o755)
                binary.write_text("#!/bin/sh\nprintf 'corrupt!\\n'\n")
                binary.chmod(0o555)
                os.utime(binary, ns=(previous.st_atime_ns, previous.st_mtime_ns))
                with self.assertRaises(RuntimeError):
                    products.consume_prepared_compiler(self.root, owner.token)

    def test_manifest_corruption_and_executable_symlink_are_rejected(self):
        for corruption in ("manifest", "symlink"):
            with self.subTest(corruption=corruption), products.PreparedCompilerSession(self.root) as owner:
                binary = self.publish(owner)
                if corruption == "manifest":
                    owner.reference.chmod(0o644)
                    owner.reference.write_text("{")
                    owner.reference.chmod(0o444)
                else:
                    binary.unlink()
                    binary.symlink_to(self.source)
                with self.assertRaises(RuntimeError):
                    products.consume_prepared_compiler(self.root, owner.token)

    def test_changed_publication_cannot_update_its_own_admission(self):
        with products.PreparedCompilerSession(self.root) as owner:
            binary = self.publish(owner)
            manifest = json.loads(owner.reference.read_text())
            binary.chmod(0o755)
            binary.write_text("#!/bin/sh\nprintf 'replacement\n'\n")
            binary.chmod(0o555)
            manifest["sha256"] = products._digest(binary)
            manifest["binary_stamp"] = list(products._stamp(binary))
            owner.reference.chmod(0o644)
            owner.reference.write_text(json.dumps(manifest))
            owner.reference.chmod(0o444)
            with self.assertRaisesRegex(RuntimeError, "manifest changed"):
                products.consume_prepared_compiler(self.root, owner.token)

    def test_new_owner_does_not_retire_another_active_publication(self):
        with products.PreparedCompilerSession(self.root) as first:
            binary = self.publish(first)
            with products.PreparedCompilerSession(self.root) as second:
                other = self.publish(second)
                self.assertFalse(products.retire(first.directory, self.root))
                self.assertEqual(products.consume_prepared_compiler(self.root, first.token), binary)
                self.assertEqual(products.consume_prepared_compiler(self.root, second.token), other)
            self.assertTrue(binary.exists())
            self.assertEqual(products.consume_prepared_compiler(self.root, first.token), binary)

    def test_failed_identity_does_not_publish_a_reference(self):
        with products.PreparedCompilerSession(self.root) as owner:
            def fail(_):
                raise RuntimeError("producer did not qualify")
            with self.assertRaisesRegex(RuntimeError, "did not qualify"):
                owner.publish(self.source, identity_reader=fail, producer={})
            self.assertIsNone(owner.reference)
            self.assertFalse((owner.directory / "manifest.json").exists())
        self.assertFalse(owner.directory.exists())

    def test_source_mutation_during_copy_is_rejected(self):
        constructor = products.hashlib.sha256
        source = self.source
        class MutatingDigest:
            def __init__(self):
                self.value = constructor()
            def update(self, data):
                source.write_text("changed producer")
                self.value.update(data)
            def hexdigest(self):
                return self.value.hexdigest()
        with products.PreparedCompilerSession(self.root) as owner:
            with patch.object(products.hashlib, "sha256", MutatingDigest):
                with self.assertRaisesRegex(RuntimeError, "changed during publication"):
                    self.publish(owner)
            self.assertIsNone(owner.reference)

    def test_live_consumer_prevents_cleanup_and_retired_owner_rejects_new_consumers(self):
        owner = products.PreparedCompilerSession(self.root)
        owner.__enter__()
        binary = self.publish(owner)
        code = (
            "import sys;sys.path.insert(0,sys.argv[1]);"
            "from prepared_compiler import consume_prepared_compiler;"
            "p=consume_prepared_compiler(sys.argv[2],sys.argv[3]);"
            "print('leased',flush=True);sys.stdin.readline();"
            "print(p.read_text(),flush=True)"
        )
        child = subprocess.Popen(
            [sys.executable, "-c", code, str(REPO_ROOT / "verification/areas/common"),
             str(self.root), owner.token], text=True,
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            with selectors.DefaultSelector() as ready:
                ready.register(child.stdout, selectors.EVENT_READ)
                self.assertTrue(ready.select(timeout=10), "consumer did not acquire its lease")
            self.assertEqual(child.stdout.readline().strip(), "leased")
            owner.__exit__(None, None, None)
            self.assertTrue(binary.exists())
            with self.assertRaisesRegex(RuntimeError, "no longer active"):
                products.consume_prepared_compiler(self.root, owner.token)
            stdout, stderr = child.communicate("\n", timeout=10)
            self.assertEqual(child.returncode, 0, stderr)
            self.assertIn("prepared", stdout)
            self.assertFalse(owner.directory.exists())
        finally:
            if child.poll() is None:
                child.kill()
                child.wait()
            owner.__exit__(None, None, None)

    def test_interrupted_unpublished_product_is_reclaimed_without_touching_foreign_paths(self):
        code = (
            "import os,sys;sys.path.insert(0,sys.argv[1]);"
            "from prepared_compiler import PreparedCompilerSession;"
            "p=PreparedCompilerSession(sys.argv[2]);p.__enter__();"
            "(p.directory/'sifr.tmp').write_text('incomplete');"
            "print(p.directory,flush=True);os._exit(7)"
        )
        result = subprocess.run(
            [sys.executable, "-c", code, str(REPO_ROOT / "verification/areas/common"), str(self.root)],
            text=True, capture_output=True, timeout=10)
        self.assertEqual(result.returncode, 7)
        abandoned = Path(result.stdout.strip())
        self.assertTrue(abandoned.exists())
        foreign = products._family(self.root) / "foreign"
        foreign.mkdir()
        with products.PreparedCompilerSession(self.root):
            self.assertFalse(abandoned.exists())
            self.assertTrue(foreign.exists())

    def test_build_contract_still_enters_cargo_with_an_inherited_product(self):
        from . import fixture_execution
        from types import SimpleNamespace
        event = {"reason": "compiler-artifact", "target": {"name": "sifr"},
                 "executable": str(self.source)}
        result = SimpleNamespace(returncode=0, cause="exit", truncated=False,
                                 stdout=json.dumps(event), stderr="")
        env = {products.REFERENCE_ENV: "/stale-reference", "CARGO_BUILD_TARGET": "selected"}
        with patch.object(fixture_execution, "run_process", return_value=result) as run:
            for _ in range(2):
                self.assertEqual(sifr_binary.prepare_sifr_binary(self.root, env=env), self.source)
        self.assertEqual(run.call_count, 2)
        self.assertEqual(run.call_args.kwargs["env"], {"CARGO_BUILD_TARGET": "selected"})


def policy_checks():
    result = unittest.TextTestRunner().run(
        unittest.defaultTestLoader.loadTestsFromTestCase(PreparedCompilerTests))
    if not result.wasSuccessful():
        raise AssertionError("prepared compiler ownership checks failed")
