"""Behavioral checks for SDK substitution and complete component source inputs."""

from __future__ import annotations

import os
import shlex
import tarfile
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import component_provenance
from component_build_paths import PATH_POLICY, build_path_mappings, remap_environment, validate_build_paths
from component_provenance import guest_environment, source_inputs
from wasi_sdk_inputs import reject_c_overrides, sdk_environment, sdk_version_text, verify_contents


class SdkInputsTests(unittest.TestCase):
    def test_remapping_preserves_space_paths_and_nested_root_precedence(self) -> None:
        with tempfile.TemporaryDirectory(prefix="component paths ") as directory:
            root = Path(directory)
            environment = {"CARGO_HOME": str(root / "cargo"),
                           "RUSTUP_HOME": str(root / "rustup"),
                           "CARGO_TARGET_DIR": "target space",
                           "WASI_SDK_PATH": str(root / "sdk"),
                           "WASI_SYSROOT": str(root / "sdk/share/wasi-sysroot")}
            result = remap_environment(root, environment)
            mappings = build_path_mappings(root, environment)
            rust = result["CARGO_ENCODED_RUSTFLAGS"].split("\x1f")
            c = shlex.split(result["CFLAGS_wasm32_wasip2"])
            self.assertEqual(c[0], f"--sysroot={environment['WASI_SYSROOT']}")
            self.assertNotIn("RUSTFLAGS", result)
            for source, destination in mappings:
                self.assertIn(f"--remap-path-prefix={source}={destination}", rust)
                self.assertIn(f"-ffile-prefix-map={source}={destination}", c)
            self.assertLess(mappings.index((str(root), "/sifr")),
                            mappings.index((str(root / "target space"), "/cargo-target")))
            self.assertNotIn("CARGO_ENCODED_RUSTFLAGS", environment)

    def test_publication_rejects_each_build_root_and_accepts_remapped_paths(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            environment = {"CARGO_HOME": str(root / "cargo"),
                           "RUSTUP_HOME": str(root / "rustup"),
                           "WASI_SDK_PATH": str(root / "sdk")}
            artifact = root / "component.wasm"
            for source, _ in build_path_mappings(root, environment):
                artifact.write_bytes(b"\0asm" + source.encode() + b"/source.rs")
                with self.assertRaisesRegex(ValueError, "producer path"):
                    validate_build_paths(artifact, root, environment)
            artifact.write_bytes(b"\0asm" + "\0".join(PATH_POLICY.values()).encode())
            validate_build_paths(artifact, root, environment)

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
        for name in ("CC", "CC_wasm32-wasip2", "CC_wasm32_wasip2", "TARGET_CC", "AR_wasm32-wasip2", "CFLAGS_wasm32_wasip2", "CC_SHELL_ESCAPED_FLAGS"):
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
        self.assertIn("crates/sifr_sql_postgresql/src/build_selection.rs", source_inputs("postgresql"))
        self.assertIn("crates/sifr_sql_postgresql/src/build_inputs.rs", source_inputs("postgresql"))
        self.assertIn("crates/sifr_compiler_component/fixtures/words_component/Cargo.lock", source_inputs("words"))

    def test_guest_inventory_excludes_only_the_native_engine_edge(self) -> None:
        for family in ("postgresql", "mysql", "sqlite"):
            inputs = source_inputs(family)
            self.assertFalse(any(path.startswith("crates/sifr_compiler_component_engine/")
                                 for path in inputs))
            self.assertIn("crates/sifr_compiler_component/Cargo.toml", inputs)
        original = component_provenance.toml

        def unconditional(path: Path) -> dict:
            value = original(path)
            if path == component_provenance.ROOT / "crates/sifr_compiler_component/Cargo.toml":
                value["dependencies"]["sifr_compiler_component_engine"] = {"workspace": True}
            return value

        with patch("component_provenance.toml", side_effect=unconditional):
            self.assertIn("crates/sifr_compiler_component_engine/src/policy.rs",
                          source_inputs("sqlite"))

    def test_words_inventory_retains_native_encoder_dependencies(self) -> None:
        original = component_provenance.toml

        def native_encoder(path: Path) -> dict:
            value = original(path)
            if path == component_provenance.ROOT / component_provenance.WORDS / "Cargo.toml":
                value.setdefault("target", {})['cfg(not(target_family = "wasm"))'] = {
                    "dependencies": {"sifr_compiler_component_engine": {"workspace": True}},
                }
            return value

        with patch("component_provenance.toml", side_effect=native_encoder):
            inputs = source_inputs("words")
            self.assertIn("crates/sifr_compiler_component_engine/src/policy.rs", inputs)
            self.assertIn(component_provenance.WORDS + "/src/bin/componentize.rs", inputs)

    def test_host_build_dependency_revisits_a_guest_dependency_as_host_code(self) -> None:
        original = component_provenance.toml

        def host_owner(path: Path) -> dict:
            value = original(path)
            if path == component_provenance.ROOT / "crates/sifr_sql_sqlite/Cargo.toml":
                value.setdefault("build-dependencies", {})["sifr_compiler_component"] = {
                    "workspace": True, "features": ["host"],
                }
            return value

        with patch("component_provenance.toml", side_effect=host_owner):
            self.assertIn("crates/sifr_compiler_component_engine/src/policy.rs",
                          source_inputs("sqlite"))


if __name__ == "__main__":
    unittest.main()
