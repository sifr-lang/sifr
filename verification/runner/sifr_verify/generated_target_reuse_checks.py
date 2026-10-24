"""Real Cargo checks for fresh generated sources and retained dependency targets."""
from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import tempfile
import time
import tomllib
import unittest
from unittest.mock import patch

from sifr_verify import cargo_setup
from sifr_verify.generated_cargo_setup import quality_module
from sifr_verify.paths import REPO_ROOT
from sifr_verify.profiles import load_profile


class GeneratedTargetReuseTests(unittest.TestCase):
    def test_fresh_roots_reuse_dependencies_but_observe_revision_and_code_changes(self):
        toolchain = tomllib.loads((REPO_ROOT / "rust-toolchain.toml").read_text())["toolchain"]["channel"]
        with tempfile.TemporaryDirectory(prefix="generated-target-reuse-") as temporary:
            root = Path(temporary)
            environment = {
                **os.environ, "CARGO_HOME": str(root / "cargo-home"),
                "RUSTUP_TOOLCHAIN": toolchain, "CARGO_NET_OFFLINE": "false",
                "CARGO_BUILD_JOBS": "1",
            }
            environment.pop("CARGO_TARGET_DIR", None)
            foundation, product = root / "foundation", root / "product"
            for directory, name in ((foundation, "reuse_foundation"), (product, "reuse_product")):
                (directory / "src").mkdir(parents=True)
                (directory / "Cargo.toml").write_text(
                    f'[package]\nname = "{name}"\nversion = "0.1.0"\nedition = "2024"\n'
                )
                self.command(["git", "init", "--initial-branch=main"], directory, environment)
            (foundation / "src/lib.rs").write_text("pub fn value() -> u32 { 10 }\n")
            foundation_revision = self.commit(foundation, environment, "foundation")
            self.product_manifest(product, foundation, foundation_revision)
            (product / "src/lib.rs").write_text("pub fn value() -> u32 { reuse_foundation::value() + 1 }\n")
            first_revision = self.commit(product, environment, "first product")
            first = self.build_and_run(root, product, first_revision, environment,
                                       "first", "reuse_product::value()", "11\n")
            self.assertFalse(first["fresh"]["reuse_foundation"], first)
            same_bytes_revision = self.commit(product, environment, "same bytes, new revision")
            self.assertNotEqual(first_revision, same_bytes_revision)
            second = self.build_and_run(root, product, same_bytes_revision, environment,
                                        "second", "reuse_product::value() + 1", "12\n")
            self.assertTrue(second["fresh"]["reuse_foundation"], second)
            self.assertEqual(first["target"], second["target"], (first, second))
            self.assertFalse(second["fresh"]["reuse_product"], second)
            self.assertFalse(second["fresh"]["reuse_program"], second)

            (product / "src/lib.rs").write_text("pub fn value() -> u32 { reuse_foundation::value() + 3 }\n")
            changed_revision = self.commit(product, environment, "changed product")
            third = self.build_and_run(root, product, changed_revision, environment,
                                       "third", "reuse_product::value()", "13\n")
            self.assertTrue(third["fresh"]["reuse_foundation"], third)
            self.assertFalse(third["fresh"]["reuse_product"], third)

            (foundation / "src/lib.rs").write_text("pub fn value() -> u32 { 20 }\n")
            new_foundation_revision = self.commit(foundation, environment, "changed dependency")
            self.product_manifest(product, foundation, new_foundation_revision)
            final_revision = self.commit(product, environment, "changed dependency selection")
            fourth = self.build_and_run(root, product, final_revision, environment,
                                        "fourth", "reuse_product::value()", "23\n")
            self.assertFalse(fourth["fresh"]["reuse_foundation"], fourth)
            self.assertFalse(fourth["fresh"]["reuse_product"], fourth)

    def command(self, args, cwd, environment):
        result = subprocess.run(args, cwd=cwd, env=environment, text=True,
                                capture_output=True, timeout=120)
        self.assertEqual(result.returncode, 0, f"{args}\n{result.stdout}\n{result.stderr}")
        return result

    def commit(self, directory, environment, message):
        hooks = directory / "empty-hooks"
        hooks.mkdir(exist_ok=True)
        self.command(["git", "add", "."], directory, environment)
        self.command(["git", "-c", "user.name=Generated target test",
                      "-c", "user.email=generated-target@example.invalid",
                      "-c", f"core.hooksPath={hooks}", "-c", "commit.gpgsign=false",
                      "commit", "--allow-empty", "-m", message], directory, environment)
        return self.command(["git", "rev-parse", "HEAD"], directory, environment).stdout.strip()

    def product_manifest(self, product, foundation, revision):
        (product / "Cargo.toml").write_text(
            '[package]\nname = "reuse_product"\nversion = "0.1.0"\nedition = "2024"\n'
            f'[dependencies]\nreuse_foundation = {{ git = "{foundation.as_uri()}", rev = "{revision}" }}\n'
        )

    def target_for_revision(self, root, revision):
        profile = load_profile("merge")
        profile.update(selected_areas=[{"area": "generated_code_quality", "suites": ["full"]}],
                       toolchain_steps=[], guardrail_steps=[])
        environment, commands = {}, []
        with patch.object(cargo_setup, "REPO_ROOT", root), \
             patch.object(cargo_setup.subprocess, "check_output", return_value=revision):
            nodes, _ = cargo_setup.cargo_preparation_plan(
                profile, environment, lambda command, **kwargs: commands.append(command)
            )
            next(node for node in nodes if node.name == "generated").callback()
        self.assertEqual(len(commands), 1)
        self.assertEqual(commands[0][-2:], ["--revision", revision])
        shared = Path(environment["SIFR_GCQ_SHARED_ROOT"])
        self.assertTrue(shared.is_relative_to(root))
        return quality_module().cargo_target_for_run(root / "invocation", shared)

    def build_and_run(self, root, product, revision, environment, label, expression, expected):
        source = root / label
        (source / "src").mkdir(parents=True)
        (source / "Cargo.toml").write_text(
            '[package]\nname = "reuse_program"\nversion = "0.1.0"\nedition = "2024"\n'
            f'[dependencies]\nreuse_product = {{ git = "{product.as_uri()}", rev = "{revision}" }}\n'
        )
        (source / "src/main.rs").write_text(f'fn main() {{ println!("{{}}", {expression}); }}\n')
        target = self.target_for_revision(root, revision)
        selected = {**environment, "CARGO_TARGET_DIR": str(target)}
        self.command(["cargo", "generate-lockfile"], source, selected)
        lock = (source / "Cargo.lock").read_bytes()
        package = next(p for p in tomllib.loads(lock.decode())["package"]
                       if p["name"] == "reuse_product")
        self.assertEqual(package["source"], f"git+{product.as_uri()}?rev={revision}#{revision}")
        started = time.monotonic()
        result = self.command(["cargo", "build", "--locked", "--offline", "--message-format=json"],
                              source, selected)
        seconds = time.monotonic() - started
        self.assertEqual((source / "Cargo.lock").read_bytes(), lock)
        artifacts = [json.loads(line) for line in result.stdout.splitlines()
                     if line.startswith("{")]
        artifacts = [m for m in artifacts if m.get("reason") == "compiler-artifact"]
        fresh = {m["target"]["name"]: m["fresh"] for m in artifacts}
        self.assertEqual(set(fresh), {"reuse_foundation", "reuse_product", "reuse_program"})
        executable = next(m["executable"] for m in artifacts if m.get("executable"))
        actual = self.command([executable], source, selected)
        self.assertEqual(actual.stdout, expected)
        self.assertEqual(actual.stderr, "")
        record = {"label": label, "revision": revision, "target": str(target),
                  "source": str(source), "fresh": fresh, "seconds": seconds,
                  "stdout": actual.stdout}
        print("[generated-target-reuse-probe] " + json.dumps(record), flush=True)
        return record


if __name__ == "__main__":
    unittest.main()
