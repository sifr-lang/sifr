"""Verify that generated oracles build, record and execute one Cargo target."""

from __future__ import annotations

import contextlib
import hashlib
import io
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import generated_suite


class GeneratedCompilerSelectionTests(unittest.TestCase):
    def setUp(self) -> None:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        self.root = Path(directory.name) / "repo"
        self.root.mkdir()
        self.outside = Path(directory.name) / "private-target"
        self.manifest = json.loads(generated_suite.MANIFEST.read_text())

    def put_binary(self, path: Path, contents: bytes = b"current compiler") -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(contents)

    def build(self, env: dict[str, str], *, result=None, error=None):
        failures: list[str] = []
        stdout, stderr = io.StringIO(), io.StringIO()
        if result is None:
            result = subprocess.CompletedProcess(["cargo"], 0, "", "")
        with patch.object(generated_suite, "REPO_ROOT", self.root), \
             patch.object(generated_suite, "source_digest", return_value="source"), \
             patch.dict(os.environ, env, clear=True), \
             patch.object(generated_suite.subprocess, "run", return_value=result,
                          side_effect=error) as run, \
             contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            info = generated_suite.build_release_binary(self.manifest, failures)
        return info, failures, run, stdout.getvalue(), stderr.getvalue()

    def test_build_records_default_relative_and_external_targets(self) -> None:
        cases = [
            ({}, self.root / "target/release/sifr"),
            ({"CARGO_TARGET_DIR": "owned/target"}, self.root / "owned/target/release/sifr"),
            ({"CARGO_TARGET_DIR": str(self.outside)}, self.outside / "release/sifr"),
        ]
        for target_env, binary in cases:
            with self.subTest(target=target_env):
                self.put_binary(self.root / "target/release/sifr", b"wrong default")
                self.put_binary(binary)
                env = {**target_env, "CARGO_BUILD_JOBS": "2", "CARGO_NET_OFFLINE": "true"}
                info, failures, run, _, _ = self.build(env)
                self.assertEqual(failures, [])
                self.assertEqual(info["binary"], str(binary.resolve()))
                self.assertEqual(info["binary_sha256"], hashlib.sha256(b"current compiler").hexdigest())
                self.assertEqual(info["source_digest"], "source")
                self.assertEqual(run.call_args.args[0], self.manifest["release_binary"]["build_command"])
                self.assertEqual(run.call_args.kwargs["cwd"], self.root)
                self.assertEqual(run.call_args.kwargs["env"], env)
                self.assertEqual(run.call_args.kwargs["timeout"], 300)

    def test_execution_uses_the_recorded_and_hashed_binary(self) -> None:
        binary = self.outside / "release/sifr"
        self.put_binary(binary)
        self.put_binary(self.root / "target/release/sifr", b"wrong default")
        info, failures, _, _, _ = self.build({"CARGO_TARGET_DIR": str(self.outside)})
        self.assertEqual(failures, [])
        suite = self.manifest["suites"]["generated_broader"]
        case = suite["cases"][0]
        actual = self.root / "actual"
        result = generated_suite.RuntimeResult(0, "[]\n", "", 1.0)
        with patch.object(generated_suite, "REPO_ROOT", self.root), \
             patch.object(generated_suite, "run_command", return_value=result) as run, \
             contextlib.redirect_stdout(io.StringIO()):
            failures = generated_suite.run_case("generated_broader", case, suite, info, actual)
        self.assertEqual(failures, [])
        self.assertEqual(run.call_args_list[1].args[0][0], str(binary.resolve()))
        self.assertEqual(run.call_args_list[1].args[0][1:3], ["--sysroot", str(self.root)])
        seed = json.loads((actual / case["id"] / "seed.json").read_text())
        self.assertEqual(seed["release_binary"], info)

    def test_missing_configured_artifact_does_not_select_default_binary(self) -> None:
        self.put_binary(self.root / "target/release/sifr", b"stale default")
        info, failures, _, _, _ = self.build({"CARGO_TARGET_DIR": str(self.outside)})
        self.assertEqual(info["binary"], str(self.outside / "release/sifr"))
        self.assertEqual(info["binary_sha256"], "missing")
        self.assertEqual(failures, [f"release binary missing after build: {self.outside / 'release/sifr'}"])

    def test_failed_build_cannot_execute_an_existing_stale_artifact(self) -> None:
        self.put_binary(self.outside / "release/sifr", b"stale configured")
        manifest_path = self.root / "manifest.json"
        manifest_path.write_text(json.dumps(self.manifest))
        with patch.object(generated_suite, "REPO_ROOT", self.root), \
             patch.object(generated_suite, "MANIFEST", manifest_path), \
             patch.object(generated_suite, "source_digest", return_value="source"), \
             patch.object(generated_suite, "validate_python_version", return_value=[]), \
             patch.dict(os.environ, {"CARGO_TARGET_DIR": str(self.outside)}, clear=True), \
             patch.object(generated_suite.subprocess, "run",
                          return_value=subprocess.CompletedProcess(["cargo"], 1, "", "build failed")), \
             patch.object(generated_suite, "run_case") as run_case, \
             contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            failures = generated_suite.run_suite("generated_broader")
        self.assertEqual(failures, ["release binary build failed with exit 1"])
        run_case.assert_not_called()

    def test_timeout_keeps_selected_path_and_partial_byte_output(self) -> None:
        error = subprocess.TimeoutExpired(["cargo"], 300, output=b"builder stdout\n", stderr=b"builder stderr\n")
        info, failures, run, stdout, stderr = self.build(
            {"CARGO_TARGET_DIR": str(self.outside)}, error=error)
        self.assertEqual(info["binary"], str(self.outside / "release/sifr"))
        self.assertEqual(info["binary_sha256"], "timeout")
        self.assertEqual(failures, ["release binary build timed out after 300s"])
        self.assertEqual(stdout, "builder stdout\n")
        self.assertEqual(stderr, "builder stderr\n")
        self.assertEqual(run.call_args.kwargs["env"]["CARGO_TARGET_DIR"], str(self.outside))


if __name__ == "__main__":
    unittest.main(verbosity=2)
