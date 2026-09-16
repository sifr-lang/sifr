"""Generated Clippy root isolation must preserve complete diagnostics and dependencies."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
from types import SimpleNamespace
import unittest

from source_quality_checks import run_strict_clippy


class ClippyIsolationTests(unittest.TestCase):
    def test_missing_package_name_cannot_trigger_a_whole_target_clean(self) -> None:
        with tempfile.TemporaryDirectory(prefix="sifr-gcq-invalid-") as directory:
            root = Path(directory)
            (root / "Cargo.toml").write_text("[workspace]\n", encoding="utf-8")
            commands = []
            with self.assertRaises(ValueError):
                run_strict_clippy(root, lambda *args, **kwargs: commands.append(args), [], root / "target")
            self.assertEqual(commands, [])

    def test_only_the_named_root_in_the_private_gate_target_is_invalidated(self) -> None:
        with tempfile.TemporaryDirectory(prefix="sifr-gcq-command-") as directory:
            root = Path(directory)
            (root / "Cargo.toml").write_text('[package]\nname="sifr_output"\n', encoding="utf-8")
            commands = []
            def run(command, **kwargs):
                commands.append((command, kwargs))
                return SimpleNamespace(returncode=0, stdout="", stderr="")
            target = root / "gate-target"
            run_strict_clippy(root, run, ["-D", "warnings"], target)
            self.assertEqual([command[1] for command, _ in commands], ["clean", "clippy"])
            self.assertEqual(commands[0][0][-2:], ["--package", "sifr_output"])
            self.assertTrue(all(kwargs["cargo_target_dir"] == target for _, kwargs in commands))
            self.assertIn("--locked", commands[0][0])
            self.assertIn("--locked", commands[1][0])

    def test_prematerialized_same_named_roots_report_their_own_diagnostics(self) -> None:
        with tempfile.TemporaryDirectory(prefix="sifr-gcq-cache-") as directory:
            root = Path(directory)
            dependency = root / "dependency"
            (dependency / "src").mkdir(parents=True)
            (dependency / "Cargo.toml").write_text(
                '[package]\nname="probe_dependency"\nversion="0.1.0"\nedition="2024"\n[workspace]\n',
                encoding="utf-8",
            )
            (dependency / "src/lib.rs").write_text('pub const fn value() -> i32 { 1 }\n', encoding="utf-8")
            target = root / "gate-target"
            sources = {
                "clean": 'fn main() { let _ = probe_dependency::value(); }\n',
                "warning": '#[must_use] const fn value() -> i32 { probe_dependency::value() } fn main() { value(); }\n',
            }
            for name, source in sources.items():
                project = root / name
                (project / "src").mkdir(parents=True)
                (project / "Cargo.toml").write_text(
                    '[package]\nname="sifr_output"\nversion="0.1.0"\nedition="2024"\n'
                    '[dependencies]\nprobe_dependency={path="../dependency"}\n[workspace]\n',
                    encoding="utf-8",
                )
                (project / "src/main.rs").write_text(source, encoding="utf-8")
                subprocess.run(
                    ["cargo", "generate-lockfile", "--offline", "--manifest-path", str(project / "Cargo.toml")],
                    check=True, capture_output=True,
                )

            def run(command, *, cargo_target_dir, check=True):
                env = os.environ.copy()
                env["CARGO_TARGET_DIR"] = str(cargo_target_dir)
                return subprocess.run(command, env=env, check=check, capture_output=True, text=True)

            first = run_strict_clippy(root / "clean", run, ["-D", "warnings"], target)
            self.assertEqual(first.returncode, 0, first.stderr)
            artifacts = list(target.rglob("libprobe_dependency*"))
            self.assertTrue(artifacts)
            before = {path: (hashlib.sha256(path.read_bytes()).hexdigest(), path.stat().st_mtime_ns) for path in artifacts}
            second = run_strict_clippy(root / "warning", run, ["-D", "warnings"], target)
            self.assertNotEqual(second.returncode, 0, second.stdout)
            self.assertEqual((root / "warning/clippy.stdout.jsonl").read_text(encoding="utf-8"), second.stdout)
            self.assertEqual((root / "warning/clippy.stderr.log").read_text(encoding="utf-8"), second.stderr)
            self.assertEqual((root / "clean/clippy.stdout.jsonl").read_text(encoding="utf-8"), first.stdout)

            diagnostics = [json.loads(line) for line in second.stdout.splitlines() if line.startswith("{")]
            self.assertTrue(any(
                row.get("reason") == "compiler-message"
                and (row["message"].get("code") or {}).get("code") == "unused_must_use"
                for row in diagnostics
            ), second.stdout)
            after = {path: (hashlib.sha256(path.read_bytes()).hexdigest(), path.stat().st_mtime_ns) for path in artifacts}
            self.assertEqual(before, after, "dependency artifacts must remain cached")
            for name, source in sources.items():
                self.assertEqual((root / name / "src/main.rs").read_text(encoding="utf-8"), source)


if __name__ == "__main__":
    unittest.main()
