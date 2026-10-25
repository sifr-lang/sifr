#!/usr/bin/env python3
"""Staged sysroot content identity contract regressions."""
import hashlib
import os
from pathlib import Path
import tempfile
import unittest

from sysroot_content_digest import content_file_digests
from verify_release_archive import sysroot_content_sha256


class SysrootContentDigestTests(unittest.TestCase):
    def fixture(self, root):
        files = {
            "Cargo.toml": b"manifest\n",
            "Cargo.lock": b"lock\n",
            ".cargo/config.toml": b"config\n",
            "crates/runtime/src/lib.rs": b"\x00\xffnative",
            "lib/stdlib/space and \u00e9.sifr": b"source\n",
            "vendor/.hidden": b"",
            "vendor/a": b"small",
            "vendor/a-z": b"ordering",
            "vendor/large": bytes(range(256)) * 2049,
        }
        for name, content in reversed(list(files.items())):
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(content)
        return files

    @staticmethod
    def expected(files):
        # Independent byte protocol, including C/UTF-8 lexical path ordering.
        data = b"".join(
            name.encode("utf-8") + b"\n"
            + hashlib.sha256(files[name]).hexdigest().encode("ascii") + b"\n"
            for name in sorted(files, key=lambda name: name.encode("utf-8"))
        )
        return hashlib.sha256(data).hexdigest()

    def test_complete_inventory_and_canonical_byte_protocol(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            files = self.fixture(root)
            observed = content_file_digests(root)
            self.assertEqual(set(observed), set(files))
            self.assertEqual(sysroot_content_sha256(observed), self.expected(files))
            (root / "bin").mkdir()
            (root / "bin/sifr").write_bytes(b"compiler is separately bound")
            (root / "sysroot.toml").write_text("the receipt cannot hash itself")
            self.assertEqual(content_file_digests(root), observed)

    def test_edits_additions_removals_and_renames_change_identity(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            files = self.fixture(root)
            previous = self.expected(files)
            operations = [
                ("edit", "vendor/a", b"changed"),
                ("add", "vendor/new", b"new"),
                ("remove", "vendor/a-z", None),
                ("rename", "vendor/new", "vendor/renamed"),
            ]
            for operation, name, value in operations:
                with self.subTest(operation=operation):
                    path = root / name
                    if operation == "remove":
                        path.unlink()
                        del files[name]
                    elif operation == "rename":
                        path.rename(root / value)
                        files[value] = files.pop(name)
                    else:
                        path.write_bytes(value)
                        files[name] = value
                    observed = sysroot_content_sha256(content_file_digests(root))
                    self.assertEqual(observed, self.expected(files))
                    self.assertNotEqual(observed, previous)
                    previous = observed

    def test_missing_governed_root_is_an_error(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.fixture(root)
            (root / "Cargo.lock").unlink()
            with self.assertRaises(FileNotFoundError):
                content_file_digests(root)

    @unittest.skipUnless(os.name == "posix", "POSIX link and special-file controls")
    def test_links_and_special_files_are_rejected_without_following(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.fixture(root)
            link = root / "vendor/link"
            for target in (root / "Cargo.lock", root / "crates", root / "missing"):
                with self.subTest(target=target):
                    link.symlink_to(target)
                    try:
                        with self.assertRaises(ValueError):
                            content_file_digests(root)
                    finally:
                        link.unlink()
            fifo = root / "vendor/fifo"
            os.mkfifo(fifo)
            with self.assertRaises(ValueError):
                content_file_digests(root)


if __name__ == "__main__":
    unittest.main()
