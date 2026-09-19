#!/usr/bin/env python3
"""Focused DX.6 packaging producer contract regressions."""
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import Mock, patch
import metadata_artifact as metadata


class MetadataArtifactTests(unittest.TestCase):
    def fixture(self, directory):
        root = Path(directory)
        (root / "bin").mkdir()
        binary = root / "bin/sifr"
        source = Path(__file__).resolve().parents[2] / "verification/areas/distribution_release/cases/mock_metadata_binary.py"
        binary.write_bytes(source.read_bytes())
        binary.chmod(0o700)
        return root, binary

    def test_complete_fixture_binding_and_corrupt_metadata_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root, binary = self.fixture(directory)
            target = metadata.host_target()
            descriptor = metadata.prepare(binary, root, root, target, True)
            payload = (root / metadata.METADATA_PATH).read_bytes()
            metadata.validate_metadata(payload, descriptor, metadata.file_digest(binary), target)
            for corrupt in [payload[:-1], b"bad" + payload[3:], payload + b"x"]:
                with self.assertRaises(ValueError):
                    metadata.validate_metadata(corrupt, descriptor, metadata.file_digest(binary), target)
            for field in ["compiler_identity", "semantic_target_id", "stdlib_inputs_id", "metadata_id", "compiler_binary_sha256"]:
                altered = copy.deepcopy(descriptor)
                altered[field] = "0" * 64
                with self.assertRaises(ValueError):
                    metadata.validate_metadata(payload, altered, metadata.file_digest(binary), target)

    def test_compatible_source_manifest_is_byte_identical_and_release_is_explicit(self):
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "source.toml"
            staged = Path(directory) / "staged.toml"
            original = ('"sifr-version" = "0.0.0-dev"\n'
                        '"target-triple" = "source-tree"\n'
                        '"built-by-compiler-commit" = "development-checkout"\n')
            source.write_text(original)
            metadata.stage_source_manifest(source, staged, "0.0.0")
            self.assertEqual(staged.read_bytes(), source.read_bytes())
            metadata.stage_source_manifest(source, staged, "0.1.0-beta.1300")
            self.assertEqual(staged.read_text(), original.replace("0.0.0-dev", "0.1.0-beta.1300-dev"))
            self.assertEqual(source.read_text(), original)

    def test_packaging_stamps_release_manifest_before_production(self):
        repo = Path(__file__).resolve().parents[2]
        target = metadata.host_target()
        with tempfile.TemporaryDirectory() as directory:
            root, binary = self.fixture(directory)
            source = root / "source"
            subprocess.run(["bash", "-c",
                'source "$1"; make_mock_sysroot_root "$2"', "fixture",
                str(repo / "verification/areas/distribution_release/cases/common.sh"),
                str(source)], check=True, cwd=repo)
            producer = binary.read_text().replace(
                '    target = arguments["--target"]',
                '    target = arguments["--target"]\n'
                '    import tomllib\n'
                '    manifest = tomllib.loads((Path(arguments["--source-root"]) / "sysroot.toml").read_text())\n'
                '    assert manifest["sifr-version"] == "0.1.0-beta.1300-dev", manifest\n'
                '    assert manifest["target-triple"] == "source-tree", manifest')
            binary.write_text(producer)
            result = subprocess.run([
                str(repo / "scripts/distribution/build_release_artifacts.sh"),
                "--version", "0.1.0-beta.1300", "--output-dir", str(root / "archives"),
                "--sysroot-root", str(source), "--binary", str(binary), "--target", target],
                cwd=repo, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn('0.0.0-fixture', (source / "sysroot.toml").read_text())
            archive = root / "archives" / f"sifr-0.1.0-beta.1300-{target}.tar.gz"
            self.assertTrue(archive.is_file())

    def test_foreign_binary_never_executes_even_in_fixture_packaging(self):
        with tempfile.TemporaryDirectory() as directory:
            root, binary = self.fixture(directory)
            header = bytearray(32)
            header[:6] = b"\x7fELF\x02\x01"
            header[18:20] = (183).to_bytes(2, "little")
            binary.write_bytes(header)
            runner = Mock()
            with patch.object(metadata, "host_target", return_value="x86_64-unknown-linux-gnu"):
                with self.assertRaisesRegex(ValueError, "not native"):
                    metadata.prepare(binary, root, root, "aarch64-unknown-linux-gnu", True, runner)
            runner.assert_not_called()

    def test_original_producer_failure_is_preserved(self):
        with tempfile.TemporaryDirectory() as directory:
            root, binary = self.fixture(directory)
            runner = Mock(side_effect=subprocess.CalledProcessError(17, [str(binary)], stderr="original producer cause"))
            with self.assertRaisesRegex(ValueError, "original producer cause"):
                metadata.prepare(binary, root, root, metadata.host_target(), True, runner)
            self.assertFalse((root / metadata.DESCRIPTOR_PATH).exists())


if __name__ == "__main__":
    unittest.main()
