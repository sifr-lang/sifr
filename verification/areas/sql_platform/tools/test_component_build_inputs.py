"""Behavioral checks for SDK substitution and complete component source inputs."""

from __future__ import annotations

import os
import tarfile
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from component_provenance import guest_environment, source_inputs
from wasi_sdk_inputs import reject_c_overrides, sdk_environment, sdk_version_text, verify_contents


class SdkInputsTests(unittest.TestCase):
    def test_upstream_version_metadata_retains_exact_release_selection(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            version = root / "VERSION"
            upstream = "34.0\nwasi-libc: 2e6fb9d8ee0c\nllvm-version: 23.1.0\n"
            version.write_text(upstream)
            self.assertEqual(sdk_version_text(root), upstream)
            for invalid in ("34.0-rc.3\n", "34.0.1\n", ""):
                version.write_text(invalid)
                with self.assertRaisesRegex(ValueError, "exactly 34.0"):
                    sdk_version_text(root)

    def test_higher_priority_compiler_and_linker_overrides_are_rejected(self) -> None:
        for name in ("CC", "CC_wasm32-wasip2", "CC_wasm32_wasip2", "TARGET_CC", "AR_wasm32-wasip2"):
            with self.assertRaisesRegex(ValueError, "ambient"):
                reject_c_overrides({name: "/substitute"})
        for name in ("RUSTC", "CARGO_TARGET_WASM32_WASIP2_LINKER", "CARGO_ENCODED_RUSTFLAGS"):
            with self.assertRaisesRegex(ValueError, "ambient"):
                guest_environment({name: "/substitute"})

    def test_archive_contents_reject_tool_sysroot_and_link_substitution(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            sdk = root / "sdk"
            (sdk / "bin").mkdir(parents=True)
            (sdk / "share/wasi-sysroot/include").mkdir(parents=True)
            compiler = sdk / "bin/clang"
            header = sdk / "share/wasi-sysroot/include/stdio.h"
            compiler.write_bytes(b"official compiler")
            header.write_bytes(b"official header")
            link = sdk / "bin/clang++"
            link.symlink_to("clang")
            archive = root / "sdk.tar.gz"
            with tarfile.open(archive, "w:gz") as package:
                package.add(sdk, arcname="sdk")
            verify_contents(archive, sdk)
            for path in (compiler, header):
                original = path.read_bytes()
                path.write_bytes(b"substituted")
                with self.assertRaisesRegex(ValueError, "differs"):
                    verify_contents(archive, sdk)
                path.write_bytes(original)
            link.unlink()
            link.symlink_to("elsewhere")
            with self.assertRaisesRegex(ValueError, "symbolic link differs"):
                verify_contents(archive, sdk)
            link.unlink()
            link.symlink_to("clang")
            extra = sdk / "share/wasi-sysroot/include/extra.h"
            extra.write_bytes(b"ambient include")
            with self.assertRaisesRegex(ValueError, "unrecorded"):
                verify_contents(archive, sdk)

    def test_ambient_sysroot_or_compiler_cannot_replace_archive(self) -> None:
        with patch.dict(os.environ, {"WASI_SYSROOT": "/ambient", "CC_wasm32_wasip2": "/ambient/clang"}, clear=True):
            with self.assertRaisesRegex(ValueError, "WASI_SDK_ARCHIVE"):
                sdk_environment()

    def test_wrong_archive_is_rejected_before_tool_execution(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            archive = Path(directory) / "not-sdk.tar.gz"
            archive.write_bytes(b"not the official archive")
            with patch.dict(os.environ, {"WASI_SDK_ARCHIVE": str(archive), "WASI_SDK_PATH": directory}, clear=True):
                with self.assertRaisesRegex(ValueError, "official host release"):
                    sdk_environment()

    def test_source_inventory_covers_shared_code_and_generated_grammar(self) -> None:
        for family in ("postgresql", "mysql", "sqlite"):
            inputs = source_inputs(family)
            self.assertIn("Cargo.lock", inputs)
            self.assertIn(".cargo/config.toml", inputs)
            self.assertIn("crates/sifr_source/src/lib.rs", inputs)
            self.assertIn("crates/sifr_diagnostics/src/lib.rs", inputs)
            self.assertIn("crates/sifr_compiler_component/wit/compiler-component.wit", inputs)
        self.assertIn("crates/sifr_sql_mysql/src/mysql.lalrpop", source_inputs("mysql"))
        self.assertIn("crates/sifr_sql_postgresql/wasi_compat/postgresql_runtime.c", source_inputs("postgresql"))
        self.assertIn("crates/sifr_compiler_component/fixtures/words_component/Cargo.lock", source_inputs("words"))


if __name__ == "__main__":
    unittest.main()
