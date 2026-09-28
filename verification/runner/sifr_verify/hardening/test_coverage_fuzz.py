"""Named H01a acceptance cases for the parser-guided runner."""

from __future__ import annotations

import tempfile
import tomllib
import unittest
from pathlib import Path
from unittest.mock import patch

from . import coverage_fuzz as fuzz


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

    def fake_run(self, *responses: dict, budget: int = 10) -> dict:
        with patch.object(fuzz, "invoke", side_effect=responses) as invoke:
            receipt = fuzz.run(profile="nightly", corpus=self.corpus, budget_override=budget)
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
        self.assertEqual(set(manifest["dependencies"]), {"libfuzzer-sys", "sifr_syntax"})
        self.assertNotIn("serde_json", manifest["dependencies"])

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
