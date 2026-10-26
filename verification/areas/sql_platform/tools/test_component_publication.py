"""Component publication stays atomic when Cargo targets use another filesystem."""
from __future__ import annotations

from contextlib import ExitStack
import errno
import hashlib
import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import build_sql_components as sql
import build_words_component as words


class ComponentPublicationTests(unittest.TestCase):
    def exercise(self, family: str, fail: bool = False) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "checkout"
            target = Path(directory) / "separate-cargo-filesystem"
            module = words if family == "words" else sql
            crate = root / (words.WORDS if family == "words" else sql.FAMILIES[family])
            crate.mkdir(parents=True)
            target.mkdir()
            virtualizer = root / "third_party/wasi-virt"
            virtualizer.mkdir(parents=True)
            (virtualizer / "Cargo.lock").write_text("locked producer")
            qualification = root / "verification/areas/sql_platform/data/compiler_component_qualification.json"
            qualification.parent.mkdir(parents=True)
            qualification.write_text(json.dumps({"fixture": {}}))
            outputs = ([crate / "words_component.wasm"] if family == "words" else
                       [crate / f"components/mysql-{series}.wasm" for series in sql.SERIES["mysql"]])
            receipt = crate / "component-artifacts.json"
            for path in [*outputs, receipt]:
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(b"previous published bytes")
            previous = {path: path.read_bytes() for path in [*outputs, receipt, qualification]}
            original_replace = Path.replace

            def replace(source, destination):
                # Model the real EXDEV boundary without depending on host mounts.
                if source.is_relative_to(target):
                    raise OSError(errno.EXDEV, "Invalid cross-device link")
                return original_replace(source, destination)

            def run(command, **_kwargs):
                if command[0] == "cargo":
                    for name in ("wasi-virt", "componentize"):
                        path = target / "release" / name
                        path.parent.mkdir(parents=True, exist_ok=True)
                        path.write_bytes(b"selected producer")
                    for relative in ("wasm32-wasip2/release/sifr_sql_mysql.wasm",
                                     "wasm32-unknown-unknown/release/sifr_component_fixture_words.wasm"):
                        core = target / relative
                        core.parent.mkdir(parents=True, exist_ok=True)
                        core.write_bytes(b"compiled core")
                    return subprocess.CompletedProcess(command, 0)
                output = Path(command[-1])
                if fail and (family == "words" or output.name == "mysql-9.7.wasm"):
                    raise subprocess.CalledProcessError(1, command)
                output.write_bytes(b"produced " + output.name.encode())
                return subprocess.CompletedProcess(command, 0)

            with ExitStack() as stack:
                for name, value in {
                    "ROOT": root, "rust_identity": lambda _target: {},
                    "target_directory": lambda _manifest: target,
                    "guest_environment": lambda environment: environment,
                    "provenance": lambda *_args, **_kwargs: {"producer": "tested"},
                }.items():
                    stack.enter_context(patch.object(module, name, value))
                if family != "words":
                    stack.enter_context(patch.object(sql, "validate_wasi_virt", lambda _path: None))
                stack.enter_context(patch.object(module.subprocess, "run", run))
                stack.enter_context(patch.object(Path, "replace", replace))
                if fail:
                    with self.assertRaises(subprocess.CalledProcessError):
                        words.main() if family == "words" else sql.build(family)
                else:
                    self.assertEqual(words.main() if family == "words" else sql.build(family), 0)
            if fail:
                self.assertEqual({path: path.read_bytes() for path in previous}, previous)
            else:
                record = json.loads(receipt.read_text())
                artifacts = [record] if family == "words" else record["artifacts"]
                self.assertEqual(len(artifacts), len(outputs))
                for path, artifact in zip(outputs, artifacts, strict=True):
                    self.assertEqual(path.read_bytes(), b"produced " + path.name.encode())
                    self.assertEqual(artifact["sha256"], hashlib.sha256(path.read_bytes()).hexdigest())
            self.assertFalse(list(crate.glob("*-components-*")))
            self.assertFalse(list(crate.glob("words-component-*")))

    def test_sql_publication_across_target_filesystem(self):
        self.exercise("mysql")

    def test_words_publication_across_target_filesystem(self):
        self.exercise("words")

    def test_failed_production_preserves_previous_publication(self):
        for family in ("mysql", "words"):
            with self.subTest(family=family):
                self.exercise(family, fail=True)


def publication_self_test() -> int:
    result = unittest.TestResult()
    unittest.defaultTestLoader.loadTestsFromTestCase(ComponentPublicationTests).run(result)
    if not result.wasSuccessful():
        raise AssertionError(repr(result.failures + result.errors))
    return result.testsRun


if __name__ == "__main__":
    unittest.main()
