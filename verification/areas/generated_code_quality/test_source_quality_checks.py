"""Generated Clippy root isolation must preserve complete diagnostics and dependencies."""
from __future__ import annotations

from dataclasses import replace
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from failure_artifacts import preserve_clippy_output
from fresh_corpus import snapshot
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

            inputs = {name: snapshot(root / name) for name in sources}
            evidence = root / "gate-evidence"
            first = run_strict_clippy(root / "clean", run, ["-D", "warnings"], target)
            preserve_clippy_output(evidence, "clean", first)
            self.assertEqual(first.returncode, 0, first.stderr)
            artifacts = list(target.rglob("libprobe_dependency*"))
            self.assertTrue(artifacts)
            before = {path: (hashlib.sha256(path.read_bytes()).hexdigest(), path.stat().st_mtime_ns) for path in artifacts}
            second = run_strict_clippy(root / "warning", run, ["-D", "warnings"], target)
            preserve_clippy_output(evidence, "warning", second)
            self.assertNotEqual(second.returncode, 0, second.stdout)
            self.assertEqual((evidence / "diagnostics/warning.stdout.jsonl").read_text(encoding="utf-8"), second.stdout)
            self.assertEqual((evidence / "diagnostics/warning.stderr.log").read_text(encoding="utf-8"), second.stderr)
            self.assertEqual((evidence / "diagnostics/clean.stdout.jsonl").read_text(encoding="utf-8"), first.stdout)

            diagnostics = [json.loads(line) for line in second.stdout.splitlines() if line.startswith("{")]
            self.assertTrue(any(
                row.get("reason") == "compiler-message"
                and (row["message"].get("code") or {}).get("code") == "unused_must_use"
                for row in diagnostics
            ), second.stdout)
            after = {path: (hashlib.sha256(path.read_bytes()).hexdigest(), path.stat().st_mtime_ns) for path in artifacts}
            self.assertEqual(before, after, "dependency artifacts must remain cached")
            for name, source in sources.items():
                self.assertEqual(snapshot(root / name), inputs[name], "Clippy must not write into generated inputs")
                self.assertEqual((root / name / "src/main.rs").read_text(encoding="utf-8"), source)


class MaterializationTests(unittest.TestCase):
    def setUp(self):
        import generated_code_quality as gcq
        self.gcq = gcq
        self.temp = tempfile.TemporaryDirectory(prefix="sifr-gcq-materialization-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.source = self.root / "main.sifr"
        self.source.write_text("entry source\n", encoding="utf-8")
        self.shared = self.root / "shared"
        self.run = self.root / "run"
        self.entry = gcq.Entry("probe", "test", "main.sifr", "build", "test")
        self.addCleanup(patch.stopall)
        patch.object(gcq, "REPO_ROOT", self.root).start()
        patch.object(gcq, "shared_artifact_root", return_value=self.shared).start()
        self.compiler = patch.object(gcq, "sifr_binary", return_value="selected-sifr").start()
        self.commands = []
        self.dependency = self.root / "provider.sifr"
        self.dependency.write_bytes(b"provider=41\n")
        self.runner = patch.object(gcq, "run_command", side_effect=self.generate).start()

    def generate(self, command):
        self.commands.append(command)
        self.assertEqual(command[:3], ["selected-sifr", "build", "main.sifr"])
        self.assertEqual(command[-1], "--materialize-only")
        output = Path(command[command.index("-o") + 1]) / "sifr_output"
        (output / "src").mkdir(parents=True)
        (output / "Cargo.toml").write_text('[package]\nname="probe"\n', encoding="utf-8")
        (output / "src/main.rs").write_bytes(self.dependency.read_bytes())

    def materialize(self):
        return self.gcq.materialize_entry(self.entry, self.run)

    def test_every_request_uses_current_compiler_and_private_output(self):
        first, second = self.materialize(), self.materialize()
        self.assertNotEqual(first, second)
        self.assertEqual(self.compiler.call_count, 2)
        self.assertEqual(len(self.commands), 2)
        self.assertEqual((first / "src/main.rs").read_bytes(), (second / "src/main.rs").read_bytes())
        self.assertTrue(first.is_relative_to(self.run))
        self.assertTrue(second.is_relative_to(self.run))
        self.assertFalse(self.shared.exists())

    def test_entry_labels_cannot_select_a_path_outside_the_run_root(self):
        for label in ("../escaped", str(self.root / "absolute"), "x" * 512):
            with self.subTest(label=label):
                self.entry = replace(self.entry, id=label)
                output = self.materialize()
                self.assertTrue(output.resolve().is_relative_to(self.run.resolve()))
                self.assertTrue((output / "src/main.rs").is_file())

    def test_same_mtime_inputs_cannot_reuse_previous_output(self):
        # The former inventory missed imports, new compiler owners and arbitrary
        # data extensions. Moved stdlib owners retain their original regression
        # coverage here without requiring a second semantic dependency graph.
        paths = ["provider.sifr", "crates/sifr_ir/src/lib.rs",
                 "crates/sifr_type_system/src/lib.rs", "crates/sifr_compiler_services/src/lib.rs",
                 "crates/sifr_semantic_services/src/lib.rs", "schema.bin",
                 "crates/sifr_stdlib_artifact/Cargo.toml", "crates/sifr_stdlib_artifact/build.rs",
                 "crates/sifr_stdlib_artifact/src/metadata.rs", "crates/sifr_metadata/Cargo.toml",
                 "crates/sifr_metadata/build.rs", "crates/sifr_metadata/src/metadata/store.rs"]
        for relative in paths:
            with self.subTest(input=relative):
                self.dependency = self.root / relative
                self.dependency.parent.mkdir(parents=True, exist_ok=True)
                self.dependency.write_bytes(b"original\n")
                first = self.materialize()
                times = self.dependency.stat()
                self.dependency.write_bytes(b"modified\n")
                os.utime(self.dependency, ns=(times.st_atime_ns, times.st_mtime_ns))
                second = self.materialize()
                self.assertEqual((first / "src/main.rs").read_bytes(), b"original\n")
                self.assertEqual((second / "src/main.rs").read_bytes(), b"modified\n")
                self.assertEqual(self.dependency.stat().st_mtime_ns, times.st_mtime_ns)

    def test_manifest_only_previous_output_cannot_skip_materialization(self):
        first = self.materialize()
        (first / "src/main.rs").unlink()
        (first / "src").rmdir()
        self.runner.reset_mock()
        second = self.materialize()
        self.runner.assert_called_once()
        self.assertTrue((second / "src/main.rs").is_file())
        self.assertFalse((first / "src").exists())

    def test_compiler_admission_failure_propagates_after_previous_success(self):
        first = self.materialize()
        self.compiler.side_effect = RuntimeError("current compiler preparation failed")
        with self.assertRaisesRegex(RuntimeError, "current compiler preparation failed"):
            self.materialize()
        self.assertEqual(len(self.commands), 1)
        self.assertTrue((first / "src/main.rs").is_file())

    def test_partial_failure_is_preserved_and_retry_uses_new_output(self):
        def fail(command):
            self.generate(command)
            raise RuntimeError("materialization failed")
        self.runner.side_effect = fail
        with self.assertRaisesRegex(RuntimeError, "materialization failed"):
            self.materialize()
        failed = next(self.run.iterdir())
        self.runner.side_effect = self.generate
        result = self.materialize()
        self.assertNotEqual(failed, result.parent)
        self.assertTrue((failed / "sifr_output/src/main.rs").is_file())
        self.assertTrue((result / "src/main.rs").is_file())

    def test_success_without_manifest_is_rejected(self):
        self.runner.side_effect = lambda command: None
        with self.assertRaisesRegex(RuntimeError, "generated crate missing Cargo.toml"):
            self.materialize()

    def test_concurrent_requests_have_independent_consumer_outputs(self):
        from concurrent.futures import ThreadPoolExecutor
        import threading
        barrier = threading.Barrier(2)
        def generate(command):
            self.generate(command)
            barrier.wait(timeout=10)
        self.runner.side_effect = generate
        with ThreadPoolExecutor(max_workers=2) as pool:
            futures = [pool.submit(self.materialize) for _ in range(2)]
            outputs = [future.result() for future in futures]
        self.assertNotEqual(outputs[0], outputs[1])
        (outputs[1] / "src/main.rs").write_bytes(b"consumer mutation\n")
        self.assertEqual((outputs[0] / "src/main.rs").read_bytes(), self.dependency.read_bytes())

    def test_dependency_target_routing_is_unchanged(self):
        self.assertEqual(self.gcq.cargo_target_for_run(self.run, self.shared),
                         self.shared / "cargo-target")
        self.assertEqual(self.gcq.cargo_target_for_run(self.run, None), self.run / "cargo-target")


class CommandContextTests(unittest.TestCase):
    def test_generic_processes_inherit_the_complete_caller_environment(self):
        import generated_code_quality as gcq
        from sifr_verify.fixture_execution import ProcessResult
        for shared in (None, Path('/owned/gcq')):
            for target in (None, '/owned/compiler'):
                with self.subTest(shared=shared, target=target):
                    environment = {'PATH': '/selected/tools', 'RUSTFLAGS': '--cfg selected',
                                   'RUSTC_WRAPPER': '/selected/wrapper'}
                    if target is not None:
                        environment['CARGO_TARGET_DIR'] = target
                    if shared is not None:
                        environment['SIFR_GCQ_SHARED_ROOT'] = str(shared)
                    with patch.dict(os.environ, environment, clear=True), patch(
                        'sifr_verify.fixture_execution.run_process',
                        return_value=ProcessResult(0, 'output', '', 'exit', False),
                    ) as execute:
                        for command in (['selected-sifr', 'build'], ['selected-sifr', 'emit'],
                                        ['python', 'freshness.py'], ['rustfmt', '--check'],
                                        ['native-output']):
                            gcq.run_command(command, cwd=Path('/owned/source'))
                            execute.assert_called_with(command, cwd=Path('/owned/source'), env=environment)
                        self.assertEqual(dict(os.environ), environment)

    def test_explicit_generated_target_changes_only_the_child_target(self):
        import generated_code_quality as gcq
        from sifr_verify.fixture_execution import ProcessResult
        environment = {'CARGO_TARGET_DIR': '/owned/compiler', 'SIFR_GCQ_SHARED_ROOT': '/owned/gcq',
                       'PATH': '/selected/tools', 'RUSTFLAGS': '--cfg selected',
                       'RUSTC_WRAPPER': '/selected/wrapper'}
        with patch.dict(os.environ, environment, clear=True), patch(
            'sifr_verify.fixture_execution.run_process',
            return_value=ProcessResult(0, '', '', 'exit', False),
        ) as execute:
            gcq.run_command(['cargo', 'check', '--locked'], cargo_target_dir=Path('/owned/generated'))
            self.assertEqual(execute.call_args.kwargs['env'],
                             environment | {'CARGO_TARGET_DIR': '/owned/generated'})
            self.assertEqual(dict(os.environ), environment)

    def test_process_failures_and_incomplete_output_remain_failures(self):
        import generated_code_quality as gcq
        from sifr_verify.fixture_execution import ProcessResult
        for result in (ProcessResult(1, '', 'failed', 'exit', False),
                       ProcessResult(0, '', '', 'timeout', False),
                       ProcessResult(0, '', '', 'exit', True)):
            with self.subTest(result=result), patch(
                'sifr_verify.fixture_execution.run_process', return_value=result,
            ):
                with self.assertRaises(RuntimeError):
                    gcq.run_command(['selected-sifr', 'build'])
                if result.cause == 'exit' and not result.truncated:
                    self.assertEqual(gcq.run_command(['cargo', 'clippy'], check=False), result)
                else:
                    with self.assertRaises(RuntimeError):
                        gcq.run_command(['cargo', 'clippy'], check=False)

    def test_generated_gates_explicitly_select_their_owned_target(self):
        from contextlib import ExitStack
        import generated_code_quality as gcq
        from native_assertions import run_native_gate
        for shared in (None, Path('/owned/shared')):
            for gate in ('corpus', 'rustfmt', 'native'):
                with self.subTest(shared=shared, gate=gate), tempfile.TemporaryDirectory() as directory:
                    root = Path(directory)
                    crate = root / 'generated'
                    (crate / 'src').mkdir(parents=True)
                    (crate / 'src/main.rs').write_text('fn main() {}\n', encoding='utf-8')
                    entry = gcq.Entry('probe', 'demos-required', 'main.sifr', 'build', 'test', 'ok\n')
                    calls = []
                    def run(command, **kwargs):
                        calls.append((command, kwargs))
                        stdout = (json.dumps({'reason': 'compiler-artifact', 'executable': '/owned/program'})
                                  if command[:2] == ['cargo', 'build'] else
                                  'ok\n' if command == ['/owned/program'] else '')
                        return SimpleNamespace(returncode=0, stdout=stdout, stderr='')
                    with ExitStack() as stack:
                        for name, value in [('REPO_ROOT', root), ('TARGET_ROOT', root / 'target')]:
                            stack.enter_context(patch.object(gcq, name, value))
                        for name, value in [('run_id', 'case'), ('shared_artifact_root', shared),
                                            ('materialize_entry', crate), ('load_debt', {}),
                                            ('record_evidence', root / 'evidence.json')]:
                            stack.enter_context(patch.object(gcq, name, return_value=value))
                        for name in ('validate_debt_owners', 'compare_exact_debt', 'assert_negative_rustfmt'):
                            stack.enter_context(patch.object(gcq, name))
                        stack.enter_context(patch.dict(os.environ, {}, clear=True))
                        stack.enter_context(patch.object(gcq, 'run_command', side_effect=run))
                        args = SimpleNamespace(group=[], keep_success=True, materialize=gcq.materialize_entry)
                        if gate == 'native':
                            run_native_gate(gcq, [entry], args)
                        else:
                            getattr(gcq, 'gate_' + gate)([entry], args)
                    cargo = [(command, kwargs) for command, kwargs in calls if command[0] == 'cargo']
                    self.assertEqual(len(cargo), 1)
                    self.assertEqual(cargo[0][1].get('cargo_target_dir'),
                                     (shared or root / 'target/case') / 'cargo-target')
                    if gate != 'rustfmt':
                        self.assertIn('--locked', cargo[0][0])
                    if gate == 'native':
                        self.assertIn('--release', cargo[0][0])
                        self.assertEqual(calls[-1], (['/owned/program'], {'cwd': root}))


if __name__ == "__main__":
    unittest.main()
