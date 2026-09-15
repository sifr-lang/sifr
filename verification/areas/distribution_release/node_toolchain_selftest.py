"""Focused checkout, selector and runtime diagnostics for the Node contract."""

from __future__ import annotations

import copy
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[3] / "scripts"))
from check_node_toolchain import ToolchainError, read_selectors, validate


class NodeToolchainTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory(prefix="sifr-node-contract-")
        self.addCleanup(self.directory.cleanup)
        self.extension = Path(self.directory.name)
        self.package = {
            "packageManager": "npm@12.0.2",
            "engines": {"node": "26.8.2", "npm": "12.0.2", "vscode": "^1.91.0"},
            "devEngines": {
                "runtime": {"name": "node", "version": "26.8.2", "onFail": "error"},
                "packageManager": {"name": "npm", "version": "12.0.2", "onFail": "error"},
            },
        }
        (self.extension / ".node-version").write_text("26.8.2\n", encoding="utf-8")
        self.write_package(self.package)
        (self.extension / "package-lock.json").write_text(
            json.dumps({"lockfileVersion": 3, "packages": {"": {"engines": self.package["engines"]}}}),
            encoding="utf-8",
        )

    def write_package(self, package: dict) -> None:
        (self.extension / "package.json").write_text(json.dumps(package), encoding="utf-8")

    def test_missing_checkout(self) -> None:
        (self.extension / "package.json").unlink()
        with self.assertRaisesRegex(ToolchainError, "missing package.json; run git submodule update"):
            validate(self.extension)

    def test_selector_drift(self) -> None:
        for field in ("engines", "devEngines", "packageManager"):
            with self.subTest(field=field):
                package = copy.deepcopy(self.package)
                if field == "engines":
                    package[field]["npm"] = "11.19.0"
                elif field == "devEngines":
                    package[field]["runtime"]["onFail"] = "warn"
                else:
                    package[field] = "npm@latest"
                self.write_package(package)
                with self.assertRaises(ToolchainError):
                    read_selectors(self.extension)
        self.write_package(self.package)
        (self.extension / "package-lock.json").write_text('{"lockfileVersion": 3}', encoding="utf-8")
        with self.assertRaisesRegex(ToolchainError, "lock root engines drifted"):
            read_selectors(self.extension)

    def test_missing_runtime(self) -> None:
        with patch("check_node_toolchain.subprocess.run", side_effect=FileNotFoundError("node")):
            with self.assertRaisesRegex(ToolchainError, "required node executable unavailable.*setup-npm.sh"):
                validate(self.extension)

    def test_mismatched_runtime(self) -> None:
        for versions in (("v26.8.1", "12.0.2"), ("v26.8.2", "11.19.0")):
            with self.subTest(versions=versions):
                responses = [subprocess.CompletedProcess([], 0, value + "\n", "") for value in versions]
                with patch("check_node_toolchain.subprocess.run", side_effect=responses):
                    with self.assertRaisesRegex(ToolchainError, "Node toolchain mismatch:.*expected.*found"):
                        validate(self.extension)

    def test_matching_runtime(self) -> None:
        responses = [subprocess.CompletedProcess([], 0, value + "\n", "") for value in ("v26.8.2", "12.0.2")]
        with patch("check_node_toolchain.subprocess.run", side_effect=responses):
            self.assertEqual(validate(self.extension), ("26.8.2", "12.0.2"))


if __name__ == "__main__":
    unittest.main()
