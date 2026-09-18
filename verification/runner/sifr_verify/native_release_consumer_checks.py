"""Retained release consumer contracts, separate from CLI default-dev checks."""
from __future__ import annotations

import ast
import importlib.util
import json
from pathlib import Path
import sys
from tempfile import TemporaryDirectory
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from .paths import REPO_ROOT

# Native application producers; compiler Cargo profiles are deliberately excluded.
LITERAL_PRODUCERS = {
    "verification/areas/performance/dx_capture.py": 1,
    "verification/areas/performance/tools/run_integer_model_readiness_perf.py": 1,
    "verification/areas/sysroot_release/runner.py": 2,
    "verification/areas/sysroot_release/attached_api_certification.py": 1,
    "verification/areas/sysroot_release/metadata_qualification.py": 1,
    "verification/areas/python_interop/runner/live_packages.py": 1,
    "verification/areas/cpython_differential/checks/generated_suite.py": 1,
    "verification/areas/cpython_differential/checks/prepare_generated.py": 1,
    "verification/areas/cpython_differential/checks/hand_seeded_merge.py": 1,
}
FINAL_CONSUMERS = {
    "verification/areas/performance/run_benchmarks.py": 1,
    "verification/areas/sysroot_release/runner.py": 2,
    "verification/areas/sysroot_release/attached_api_certification.py": 1,
    "verification/areas/python_interop/runner/live_packages.py": 1,
}


def literal_native_commands(source):
    """Inventory literal application argv after Cargo's --, excluding compiler work."""
    rows = []
    for node in ast.walk(ast.parse(source)):
        if not isinstance(node, (ast.List, ast.Tuple)):
            continue
        tokens = [x.value if isinstance(x, ast.Constant) else None for x in node.elts]
        if not tokens:
            continue
        if tokens[0] == "cargo":
            if "--" not in tokens or "sifr" not in tokens:
                continue
            tokens = tokens[tokens.index("--") + 1:]
        if any(token in ("build", "run") for token in tokens):
            rows.append((node.lineno, tokens))
    return rows


def load_area(name, relative):
    path = REPO_ROOT / relative
    sys.path.insert(0, str(path.parent))
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


class NativeReleaseConsumerTests(unittest.TestCase):
    def test_declared_native_producers_keep_release_after_cargo_separator(self):
        for relative, count in LITERAL_PRODUCERS.items():
            rows = literal_native_commands((REPO_ROOT / relative).read_text())
            self.assertEqual(len(rows), count, relative)
            for line, tokens in rows:
                self.assertEqual(tokens.count("--release"), 1, (relative, line, tokens))
        # A compiler release flag cannot satisfy an application's release contract.
        mutated = '["cargo", "run", "--release", "-p", "sifr", "--", "build", source]'
        self.assertNotIn("--release", literal_native_commands(mutated)[0][1])

    def test_final_consumers_do_not_consume_cargo_intermediate_paths(self):
        for relative, count in FINAL_CONSUMERS.items():
            source = (REPO_ROOT / relative).read_text()
            self.assertEqual(source.count('/ "target" / "final" /'), count, relative)
            self.assertNotIn('/ "target" / "release" /', source, relative)

    def test_benchmarks_select_release_and_measure_only_finalized_binary(self):
        bench = load_area("dx10_benchmarks", "verification/areas/performance/run_benchmarks.py")
        manifest = json.loads(bench.DEFAULT_MANIFEST.read_text())
        builds = [case for case in manifest["cases"] if case.get("mode") == "build"]
        self.assertTrue(builds)
        with TemporaryDirectory() as directory, patch.object(bench, "sifr_binary", return_value=Path("/compiler")):
            root = Path(directory)
            for case in builds:
                command = bench.command_for_case(bench.BenchmarkCase(case), root)
                self.assertEqual(command.count("--release"), 1)
            for mode in ("check", "fmt-check"):
                command = bench.command_for_case(bench.BenchmarkCase({"mode": mode, "source_path": "app.sifr"}), root)
                self.assertNotIn("--release", command)
            project = root / "sifr_output"
            (project / "src").mkdir(parents=True)
            (project / "src/main.rs").write_text("fn main() {}\n")
            stale = project / "target/release" / bench.executable_name("sifr_output")
            stale.parent.mkdir(parents=True)
            stale.write_bytes(b"stale intermediate")
            with self.assertRaises(bench.BenchmarkError):
                bench.collect_build_size_metrics(root)
            final = project / "target/final" / stale.name
            final.parent.mkdir()
            final.write_bytes(b"final")
            self.assertEqual(bench.collect_build_size_metrics(root)["generated_binary_bytes"], 5)

    def test_capture_rejects_mislabeled_default_application(self):
        capture = load_area("dx10_capture", "verification/areas/performance/dx_capture.py")
        with patch.object(capture.runner, "run_subprocess") as run:
            with self.assertRaisesRegex(ValueError, "explicit --release"):
                capture.observed(["/compiler", "build", "app.sifr"], Path("/unused"), "dx-native-first", 0)
            run.assert_not_called()

    def test_hardening_retains_native_release_without_changing_test_or_check(self):
        from .hardening import core
        result = SimpleNamespace(returncode=0, stdout="", stderr="")
        with patch.object(core.subprocess, "run", return_value=result):
            for command in ("run", "build", "test", "check"):
                *_, argv = core.run_variant(repo_root=REPO_ROOT, command_name=command,
                    entry=REPO_ROOT / "app.sifr", diagnostic_format="json")
                self.assertEqual(argv.count("--release"), int(command in ("run", "build")))
                self.assertIn("--diagnostic-format", argv)

    def test_namespace_selection_retains_run_release_only(self):
        module = load_area("dx10_namespace", "verification/areas/stdlib_parity/tools/run_stdlib_namespace_corpus_validation.py")
        with patch.object(module.subprocess, "run") as run:
            for command in ("run", "check"):
                module.run_fixture(Path("/compiler"), command, REPO_ROOT / "app.sifr")
                self.assertEqual(run.call_args.args[0].count("--release"), int(command == "run"))

    def test_golden_runtime_inventory_keeps_explicit_release(self):
        manifest = json.loads((REPO_ROOT / "verification/areas/runtime_platform/golden/manifest.json").read_text())
        def commands(value):
            if isinstance(value, dict):
                if isinstance(value.get("command"), str):
                    yield value["command"]
                for child in value.values():
                    yield from commands(child)
            elif isinstance(value, list):
                for child in value:
                    yield from commands(child)
        native = [cmd for cmd in commands(manifest) if cmd.startswith("{sifr} run ")]
        self.assertEqual(len(native), 4)
        self.assertTrue(all("--release" in cmd.split() for cmd in native))

    def test_python_runtime_call_sites_retain_release(self):
        for relative, call_name in (
            ("verification/areas/python_interop/runner/example_packages.py", "_run_sifr_process"),
            ("verification/areas/python_interop/runner/binding_authoring.py", "run"),
        ):
            nodes = ast.walk(ast.parse((REPO_ROOT / relative).read_text()))
            native = [n for n in nodes if isinstance(n, ast.Call)
                and isinstance(n.func, ast.Name) and n.func.id == call_name
                and any(isinstance(a, ast.Constant) and a.value == "run" for a in n.args)]
            self.assertEqual(len(native), 1, relative)
            self.assertTrue(any(isinstance(a, ast.Constant) and a.value == "--release" for a in native[0].args))


if __name__ == "__main__":
    unittest.main()
