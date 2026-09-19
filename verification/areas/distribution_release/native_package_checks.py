"""Negative integrity and transport tests for native package qualification."""
from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "scripts/distribution"))
import qualify_native_package as subject
import metadata_qualification as metadata_subject


class NativePackageContract(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="sifr-native-package-contract-")
        self.addCleanup(self.temporary.cleanup)
        root = Path(self.temporary.name)
        self.subject = subject.NativePackage.__new__(subject.NativePackage)
        self.subject.artifacts = root / "artifacts"
        self.subject.artifacts.mkdir()
        self.subject.output = root / "results"
        self.subject.output.mkdir()
        self.subject.installer = root / "installer"
        self.subject.installer.write_bytes(b"exact qualification installer\n")
        self.subject.version = "0.1.0"
        self.subject.previous_version = "0.0.0"
        self.subject.previous_artifacts = root / "previous-artifacts"
        self.subject.previous_artifacts.mkdir()
        self.subject.previous_installer = root / "previous-installer"
        self.subject.previous_installer.write_bytes(b"exact previous installer\n")
        self.subject.source = "a" * 40
        self.subject.env = dict(os.environ)
        self.subject.report = {}
        for version, artifacts in ((self.subject.version, self.subject.artifacts),
                                   (self.subject.previous_version, self.subject.previous_artifacts)):
            for target in subject.TARGETS:
                archive = artifacts / f"sifr-{version}-{target}.tar.gz"
                archive.write_bytes(("test-only artifact " + version + target).encode())
                Path(str(archive) + ".sha256").write_text(subject.digest(archive) + "\n")
                report = {"source_commit": self.subject.source, "target": target,
                          "candidate_version": version, "smoke_status": "pass",
                          "archive_sha256": subject.digest(archive), "sysroot_sha256": "b" * 64}
                (artifacts / f"qualification-{target}.json").write_text(json.dumps(report))

    def test_transport_serves_only_exact_allowlisted_bytes(self):
        self.subject.prepare_transport()
        curl = self.subject.output / "transport/curl"
        url = f"{subject.PUBLIC}/0.1.0/sifr-installer-0.1.0"
        destination = self.subject.output / "download"
        result = subprocess.run([str(curl), "-fsSL", "--proto", "=https",
                                 "--proto-redir", "=https", url, "-o", str(destination)],
                                capture_output=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(destination.read_bytes(), self.subject.installer.read_bytes())
        for unlisted in (url + "?other", "https://example.invalid/exfiltrate", "file:///etc/passwd"):
            result = subprocess.run([str(curl), "-fsSL", unlisted], capture_output=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(result.stdout, b"")
        result = subprocess.run([str(curl), "-fsSL", url, "https://example.invalid/"],
                                capture_output=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stdout, b"")

    def test_modified_archive_rejects_before_transport_creation(self):
        archive = next(self.subject.artifacts.glob("*.tar.gz"))
        archive.write_bytes(archive.read_bytes() + b"altered")
        with self.assertRaisesRegex(RuntimeError, "archive digest mismatch"):
            self.subject.prepare_transport()
        self.assertFalse((self.subject.output / "transport").exists())

    def test_wrong_source_target_version_or_status_rejects(self):
        path = self.subject.artifacts / f"qualification-{subject.TARGETS[0]}.json"
        original = json.loads(path.read_text())
        for field, value in (("source_commit", "c" * 40), ("target", "other-target"),
                             ("candidate_version", "0.0.1"), ("smoke_status", "fail")):
            with self.subTest(field=field):
                path.write_text(json.dumps(original | {field: value}))
                with self.assertRaisesRegex(RuntimeError, "mismatched target report"):
                    self.subject.prepare_transport()
                self.assertFalse((self.subject.output / "transport").exists())
        path.write_text(json.dumps(original))

    def test_previous_archive_is_also_bound(self):
        archive = next(self.subject.previous_artifacts.glob("*.tar.gz"))
        archive.write_bytes(b"changed previous archive")
        with self.assertRaisesRegex(RuntimeError, "archive digest mismatch"):
            self.subject.prepare_transport()
        self.assertFalse((self.subject.output / "transport").exists())

    def test_version_admission_rejects_known_incompatible_candidates(self):
        root = Path(self.temporary.name) / "source"
        manifest = root / "crates/sifr/Cargo.toml"
        manifest.parent.mkdir(parents=True)
        manifest.write_text('[package]\nversion = "0.0.0"\n')
        editor = root / "editor_integrations/vscode/package.json"
        editor.parent.mkdir(parents=True)
        editor.write_text(json.dumps({"sifrCompilerCompatibility": ">=0.1.0,<0.2.0"}))
        self.assertEqual(subject.transition_fixture_version(root, "0.1.0", "none"), "0.0.0")
        for candidate, rollback in (("0.0.0", "none"), ("0.2.0", "none"), ("0.1.0", "0.0.0")):
            with self.subTest(candidate=candidate, rollback=rollback):
                with self.assertRaises(RuntimeError):
                    subject.transition_fixture_version(root, candidate, rollback)

    def test_instrumented_cargo_keeps_explicit_matching_rustc(self):
        selected = {"cargo": "/selected/toolchain/bin/cargo", "rustc": "/selected/toolchain/bin/rustc"}
        class ReachedNativeCommand(Exception):
            pass
        with patch.object(subject.shutil, "which", side_effect=lambda name, **kwargs: selected[name]), \
             patch.object(self.subject, "run", side_effect=ReachedNativeCommand) as run:
            with self.assertRaises(ReachedNativeCommand):
                self.subject.native_profiles(Path("/installed/bin/sifr"))
        env = run.call_args.kwargs["env"]
        self.assertEqual(env["QUALIFICATION_CARGO"], selected["cargo"])
        self.assertEqual(env["SIFR_RUSTC"], selected["rustc"])
        self.assertEqual(Path(env["SIFR_CARGO"]), self.subject.output / "cargo-events")

    def test_darwin_loader_separates_exact_binary_header_from_dependencies(self):
        binary = subject.ROOT / "native-qualification/relocated/bin/sifr"
        system_library = "/usr/lib/libSystem.B.dylib (compatibility version 1.0.0)"
        subject.validate_loader_output(binary, f"{binary}:\n\t{system_library}\n", "Darwin")
        for output in (
            f"/other/bin/sifr:\n\t{system_library}\n",
            f"{binary}:\n",
            f"{binary}:\n\t{subject.ROOT}/build/libbad.dylib\n",
            f"{binary}:\n\tlibbad.dylib => not found\n",
        ):
            with self.subTest(output=output), self.assertRaises(RuntimeError):
                subject.validate_loader_output(binary, output, "Darwin")

    def test_linux_loader_keeps_all_dependency_lines(self):
        binary = subject.ROOT / "native-qualification/relocated/bin/sifr"
        subject.validate_loader_output(binary, "libc.so.6 => /lib/libc.so.6 (0x1)\n", "Linux")
        for output in ("", "libbad.so => not found\n", f"libbad.so => {subject.ROOT}/libbad.so\n"):
            with self.subTest(output=output), self.assertRaises(RuntimeError):
                subject.validate_loader_output(binary, output, "Linux")

    def test_installed_workload_isolated_from_output_workspace(self):
        root = Path(self.temporary.name) / "source-workspace"
        root.mkdir()
        (root / "sifr.toml").write_text("[workspace]\n")
        qualification = metadata_subject.Qualification.__new__(metadata_subject.Qualification)
        qualification.output = root / "reports"
        qualification.report = {}
        def inspect():
            self.assertFalse(qualification.workspace.is_relative_to(root))
            self.assertTrue(qualification.workspace.is_dir())
        with patch.object(qualification, "save"), \
             patch.object(qualification, "_installed", side_effect=inspect) as installed:
            qualification.installed()
        installed.assert_called_once()
        self.assertFalse(qualification.workspace.exists())

    def test_installed_workload_rejects_temporary_workspace_manifest(self):
        root = Path(self.temporary.name) / "source-workspace"
        root.mkdir()
        (root / "sifr.toml").write_text("[workspace]\n")
        qualification = metadata_subject.Qualification.__new__(metadata_subject.Qualification)
        qualification.report = {}
        real_temporary_directory = tempfile.TemporaryDirectory
        with patch.object(metadata_subject.tempfile, "TemporaryDirectory",
                          side_effect=lambda **kwargs: real_temporary_directory(dir=root, **kwargs)), \
             patch.object(qualification, "_installed") as installed:
            with self.assertRaisesRegex(AssertionError, "manifestless temporary workspace"):
                qualification.installed()
        installed.assert_not_called()

    def test_missing_selected_rustc_rejects_before_native_execution(self):
        with patch.object(subject.shutil, "which", side_effect=lambda name, **kwargs:
                          "/selected/cargo" if name == "cargo" else None), \
             patch.object(self.subject, "run") as run:
            with self.assertRaisesRegex(RuntimeError, "paired Cargo and rustc"):
                self.subject.native_profiles(Path("/installed/bin/sifr"))
        run.assert_not_called()
        self.assertFalse((self.subject.output / "cargo-events").exists())

    def test_checksum_disagreement_rejects(self):
        checksum = next(self.subject.artifacts.glob("*.sha256"))
        checksum.write_text("c" * 64)
        with self.assertRaisesRegex(RuntimeError, "checksum mismatch"):
            self.subject.prepare_transport()
        self.assertFalse((self.subject.output / "transport").exists())


if __name__ == "__main__":
    unittest.main()

