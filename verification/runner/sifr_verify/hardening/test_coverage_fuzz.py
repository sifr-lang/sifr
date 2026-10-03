"""Named acceptance cases for the frontend guided runner."""

from __future__ import annotations

import os
import sys
import tempfile
import tomllib
import unittest
from pathlib import Path
from unittest.mock import patch

from . import coverage_fuzz as fuzz
from ..process_execution import Outcome


def result(exit_code: int = 0, output: str = "", timed_out: bool = False) -> dict:
    return {
        "exit_code": exit_code, "output_tail": output,
        "timed_out": timed_out, "duration_ms": 1.0,
    }


class CoverageFuzzTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.corpus = Path(self.temp.name) / "corpus"
        self.corpus.mkdir()
        (self.corpus / "seed").write_text("def main():\n    pass\n", encoding="utf-8")

    def tearDown(self) -> None:
        self.temp.cleanup()

    def fake_run(self, *responses: dict, budget: int = 10, target: str | None = None) -> dict:
        with patch.object(fuzz, "invoke", side_effect=responses) as invoke:
            receipt = fuzz.run(
                profile="nightly", target=target, corpus=self.corpus, budget_override=budget,
            )
        self.assertEqual(invoke.call_count, len(responses))
        return receipt

    def test_missing_tool(self) -> None:
        receipt = self.fake_run(result(127, "no such command: fuzz"), result(0, "rustc nightly"))
        self.assertEqual(receipt["status"], "missing-tool")
        self.assertEqual(len(receipt["variants"]), 1)

    def test_offline_dependency_failure(self) -> None:
        receipt = self.fake_run(
            result(0, "cargo-fuzz 0.13.2"), result(0, "rustc nightly"),
            result(101, "no matching package named libfuzzer-sys found in offline mode"),
        )
        self.assertEqual(receipt["status"], "offline-dependency-failure")
        self.assertLessEqual(len(receipt["variants"][0]["output_tail"]), fuzz.MAX_OUTPUT_TAIL)

    def test_instrumented_build_failure(self) -> None:
        receipt = self.fake_run(
            result(0, "cargo-fuzz 0.13.2"), result(0, "rustc nightly"),
            result(101, "error[E0308]: type mismatch"),
        )
        self.assertEqual(receipt["status"], "instrumented-build-failure")

    def test_target_timeout(self) -> None:
        receipt = self.fake_run(
            result(0, "cargo-fuzz 0.13.2"), result(0, "rustc nightly"),
            result(), result(124, "#42 cov: 9", timed_out=True),
        )
        self.assertEqual(receipt["status"], "target-timeout")
        self.assertEqual(receipt["variants"][-1]["executions"], 42)

    def test_non_crash_exit_ignores_old_artifact(self) -> None:
        old = fuzz.ROOT / "target/verification/fuzz/artifacts/parser/old-artifact-h01a-unit"
        old.parent.mkdir(parents=True, exist_ok=True)
        old.write_bytes(b"old crash")
        try:
            receipt = self.fake_run(
                result(0, "cargo-fuzz 0.13.2"), result(0, "rustc nightly"),
                result(), result(137, "Killed by host memory pressure"),
            )
        finally:
            old.unlink(missing_ok=True)
        self.assertEqual(receipt["status"], "target-run-failure")
        self.assertNotIn("finding", receipt)

    def test_compiler_finding_minimized_seed(self) -> None:
        artifact_dir = Path(self.temp.name) / "artifacts"
        artifact_dir.mkdir()
        crash = artifact_dir / "crash-raw"
        crash.write_bytes(b"long crashing input")
        minimized_crash = artifact_dir / "minimized-by-fuzzer"
        calls = iter([
            result(0, f"Minimized artifact:\n\t{minimized_crash}"),
            result(1, "ERROR: libFuzzer: deadly signal"),
            result(1, "ERROR: libFuzzer: deadly signal"),
        ])

        def minimize(argv: list[str], **_kwargs: object) -> dict:
            if "tmin" in argv:
                minimized_crash.write_bytes(b"minimal crash")
            return next(calls)

        with patch.object(fuzz, "invoke", side_effect=minimize):
            finding = fuzz.preserve_finding(
                target="parser", output=f"Test unit written to {crash}",
                artifact_dir=artifact_dir, env={}, label="sustained-fuzz:parser:nightly",
            )
        self.assertEqual(finding["status"], "compiler-finding")
        self.assertEqual(Path(finding["minimized_seed"]).read_bytes(), b"minimal crash")
        self.assertEqual(len(finding["replays"]), 2)

    def test_project_graph_finding_exports_minimized_tree(self) -> None:
        artifact_dir = Path(self.temp.name) / "project-artifacts"
        artifact_dir.mkdir()
        crash = artifact_dir / "crash-raw"
        crash.write_bytes(b"\x07\x07\x03\x01")
        minimized = artifact_dir / "minimized-by-fuzzer"
        calls = iter([
            result(0, f"Minimized artifact: {minimized}"),
            result(1, "ERROR: libFuzzer: deadly signal"),
            result(1, "ERROR: libFuzzer: deadly signal"),
            result(1, "ERROR: libFuzzer: deadly signal"),
        ])

        def invoke(argv: list[str], *, env: dict, **_kwargs: object) -> dict:
            if "tmin" in argv:
                minimized.write_bytes(crash.read_bytes())
            if "SIFR_FUZZ_PROJECT_TREE_EXPORT_DIR" in env:
                root = Path(env["SIFR_FUZZ_PROJECT_TREE_EXPORT_DIR"])
                root.mkdir()
                for name in ("app", "alpha", "beta", "gamma"):
                    package = root / name
                    (package / "src").mkdir(parents=True)
                    for relative in ("Cargo.toml", "sifr.toml", "src/lib.rs"):
                        (package / relative).write_text(relative, encoding="utf-8")
                    source = "src/main.sifr" if name == "app" else "src/__init__.sifr"
                    (package / source).write_text(source, encoding="utf-8")
            return next(calls)

        with patch.object(fuzz, "invoke", side_effect=invoke):
            finding = fuzz.preserve_finding(
                target="project_graph", output=f"Test unit written to {crash}",
                artifact_dir=artifact_dir, env={}, label="sustained-fuzz:project_graph:nightly",
            )
        self.assertEqual(finding["status"], "compiler-finding")
        self.assertEqual(len(finding["minimized_project_tree_sha256"]), 16)
        self.assertTrue(Path(finding["minimized_project_tree"]).joinpath("app/sifr.toml").is_file())

    def test_diagnostic_target_minimized_json_artifact(self) -> None:
        artifact_dir = Path(self.temp.name) / "diagnostic-artifacts"
        artifact_dir.mkdir()
        crash = artifact_dir / "crash-raw"
        crash.write_text('{"version":1,"diagnostics":[{"code":"SIFR-TYPE-0002"}]}')
        minimized_crash = artifact_dir / "minimized-by-fuzzer"
        calls = iter([
            result(0, f"Minimized artifact: {minimized_crash}"),
            result(1, "ERROR: libFuzzer: deadly signal"),
            result(1, "ERROR: libFuzzer: deadly signal"),
        ])

        def minimize(argv: list[str], **_kwargs: object) -> dict:
            if "tmin" in argv:
                minimized_crash.write_bytes(crash.read_bytes())
            return next(calls)

        with patch.object(fuzz, "invoke", side_effect=minimize):
            finding = fuzz.preserve_finding(
                target="diagnostics", output=f"Test unit written to {crash}",
                artifact_dir=artifact_dir, env={}, label="sustained-fuzz:diagnostics:nightly",
            )
        self.assertEqual(finding["status"], "compiler-finding")
        self.assertTrue(finding["minimized_seed"].endswith(".json"))
        self.assertEqual(fuzz.file_hash(Path(finding["minimized_seed"])), finding["minimized_sha256"])
        self.assertEqual(len(finding["replays"]), 2)

    def test_diagnostic_target_selection_and_identity(self) -> None:
        receipt = self.fake_run(
            result(0, "cargo-fuzz 0.13.2"), result(0, "rustc nightly"),
            result(), result(0, "#12 cov: 8"), target="diagnostics",
        )
        self.assertEqual(receipt["status"], "pass")
        self.assertEqual(receipt["input_identity"]["configuration"]["corpus"],
                         "verification/fuzz/corpus/diagnostics")
        self.assertEqual(receipt["variants"][-1]["executions"], 12)

    def test_counters_survive_bounded_dictionary_tail(self) -> None:
        output = "#12 cov: 8\n" + "dictionary" * fuzz.MAX_OUTPUT_TAIL
        completed = Outcome(0, "exit", output.encode(), b"", False, .01)
        with patch.object(fuzz, "execute", return_value=completed):
            actual = fuzz.invoke(["cargo"], timeout=10, env={})
        self.assertEqual(actual["executions"], 12)
        self.assertEqual(actual["coverage_edges"], 8)
        self.assertLessEqual(len(actual["output_tail"]), fuzz.MAX_OUTPUT_TAIL)

    def test_capture_loss_and_missing_custody_fail_closed(self):
        for outcome in (Outcome(0, "exit", b"#12 cov: 8", b"", True, .01),
                        Outcome(130, "cancelled", b"#12 cov: 8", b"", False, .01)):
            with patch.object(fuzz, "execute", return_value=outcome):
                actual = fuzz.invoke(["unused"], timeout=10, env={})
            self.assertTrue(actual["infrastructure_error"])
            self.assertEqual(fuzz.classify_preflight(actual), "infrastructure-failure")
        with patch.object(fuzz, "execute", side_effect=OSError("missing cleanup confirmation")):
            actual = fuzz.invoke(["unused"], timeout=10, env={})
        self.assertTrue(actual["infrastructure_error"])
        receipt = self.fake_run(result(0, "cargo-fuzz 0.13.2"), result(0, "rustc nightly"),
                                result(), actual)
        self.assertEqual(receipt["status"], "infrastructure-failure")
        self.assertNotIn("finding", receipt)

    def test_real_detached_resistant_timeout_is_reaped_before_next_command(self):
        if not sys.platform.startswith("linux"):
            self.skipTest("Linux canonical subreaper custody")
        pid_file = Path(self.temp.name) / "child.pid"
        child = "import os,signal; os.setsid(); signal.signal(signal.SIGTERM,signal.SIG_IGN); " \
                "open(__import__('sys').argv[1], 'w').write(str(os.getpid())); signal.pause()"
        parent = "import subprocess,sys,signal; signal.signal(signal.SIGTERM,signal.SIG_IGN); " \
                 "subprocess.Popen([sys.executable,'-c',sys.argv[1],sys.argv[2]]); signal.pause()"
        actual = fuzz.invoke([sys.executable, "-c", parent, child, str(pid_file)], timeout=2,
                             env=os.environ.copy())
        self.assertTrue(actual["timed_out"])
        self.assertFalse(actual["infrastructure_error"])
        pid = int(pid_file.read_text())
        with self.assertRaises(ProcessLookupError):
            os.kill(pid, 0)
        next_run = fuzz.invoke([sys.executable, "-c", "import os,sys; "
                               "os.kill(int(sys.argv[1]),0)", str(pid)], timeout=10, env=os.environ.copy())
        self.assertNotEqual(next_run["exit_code"], 0)
        self.assertIn("ProcessLookupError", next_run["output_tail"])

    def test_nonzero_guided_executions_and_coverage(self) -> None:
        receipt = self.fake_run(
            result(0, "cargo-fuzz 0.13.2"), result(0, "rustc nightly"),
            result(), result(0, "#12 NEW cov: 8\nstat::number_of_executed_units: 30"),
        )
        self.assertEqual(receipt["status"], "pass")
        self.assertEqual(receipt["variants"][-1]["executions"], 30)
        self.assertEqual(receipt["variants"][-1]["coverage_edges"], 8)
        self.assertIn("seed", receipt["input_identity"]["corpus"])

    def test_budget_and_unique_labels(self) -> None:
        receipt = self.fake_run(
            result(0, "cargo-fuzz 0.13.2"), result(0, "rustc nightly"),
            result(), result(0, "#12 cov: 8"),
            budget=10,
        )
        labels = [item["label"] for item in receipt["variants"]]
        self.assertEqual(len(labels), len(set(labels)))
        self.assertTrue(all(label.startswith("sustained-fuzz:parser:nightly:") for label in labels))
        self.assertIn("-max_total_time=10", receipt["variants"][-1]["argv"])
        self.assertEqual(receipt["budget_seconds"], 10)

    def test_no_unused_fuzz_dependencies(self) -> None:
        manifest = tomllib.loads((fuzz.ROOT / "verification/fuzz/Cargo.toml").read_text())
        self.assertEqual(set(manifest["dependencies"]), {
            "libfuzzer-sys", "sifr_syntax", "sifr_frontend", "sifr_diagnostics",
            "sifr_driver", "serde_json",
        })
        self.assertIn("serde_json::from_slice", (
            fuzz.ROOT / "verification/fuzz/fuzz_targets/diagnostics.rs"
        ).read_text())

    def test_frontend_target_selection_and_identity(self) -> None:
        for target in ("lowering", "ownership"):
            with self.subTest(target=target):
                receipt = self.fake_run(
                    result(0, "cargo-fuzz 0.13.2"), result(0, "rustc nightly"),
                    result(), result(0, "#12 cov: 8"),
                    target=target,
                )
                self.assertEqual(receipt["target"], target)
                self.assertEqual(receipt["status"], "pass")
                self.assertEqual(
                    receipt["input_identity"]["target_source_sha256"],
                    fuzz.file_hash(fuzz.ROOT / f"verification/fuzz/fuzz_targets/{target}.rs"),
                )
                self.assertEqual(
                    receipt["input_identity"]["configuration"]["corpus"],
                    f"verification/fuzz/corpus/{target}",
                )
                self.assertEqual(receipt["variants"][0]["argv"][-1], target)
                self.assertEqual(receipt["variants"][-1]["argv"][6], target)
                self.assertEqual(
                    (Path(receipt["working_corpus"]) / "seed").read_text(encoding="utf-8"),
                    "def main():\n    pass\n",
                )

    def test_frontend_target_classification(self) -> None:
        for target in ("lowering", "ownership"):
            with self.subTest(target=target):
                receipt = self.fake_run(
                    result(0, "cargo-fuzz 0.13.2"), result(0, "rustc nightly"),
                    result(101, "no matching package named libfuzzer-sys found in offline mode"),
                    target=target,
                )
                self.assertEqual(receipt["status"], "offline-dependency-failure")
                self.assertTrue(receipt["variants"][0]["label"].startswith(
                    f"sustained-fuzz:{target}:nightly:"
                ))

    def test_sustained_suite_is_explicit_only(self) -> None:
        from verification.areas.fuzz_property.runner import select_suites

        manifest = {
            "suites": [{"name": "fuzz-smoke"}, {"name": "sustained-fuzz"}],
        }
        self.assertEqual([item["name"] for item in select_suites(manifest, set())], ["fuzz-smoke"])
        self.assertEqual(
            [item["name"] for item in select_suites(manifest, {"sustained-fuzz"})],
            ["sustained-fuzz"],
        )


if __name__ == "__main__":
    unittest.main()
