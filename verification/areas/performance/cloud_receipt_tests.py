"""Receipt tampering, missing evidence and stale invocations never qualify."""

import copy
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import cloud_benchmarks as cloud


class CloudReceiptTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        root = Path(self.temporary.name)
        self.output = root / "capture"
        self.manifest = root / "manifest.json"
        self.manifest.write_text(json.dumps({"cases": [{"id": "case", "kind": "command", "warmups": 1,
            "expected_exit_codes": [0]}]}))
        self.budgets = root / "budgets.json"
        self.budgets.write_text(json.dumps({"budgets": [{"benchmark_id": "case", "policy": "command-default",
            "thresholds": {"median_ms": 1200, "p95_ms": 1500}, "cache": {}}]}))
        self.paths = {}
        self.endpoints = {}
        for key, source in [("baseline", "a" * 40), ("candidate", "b" * 40)]:
            repo = root if key == "candidate" else root / "baseline"
            repo.mkdir(exist_ok=True)
            self.paths[key] = root / f"{key}.json"
            value = {"cloud_repo": str(repo), "cloud_source": source,
                "artifact": {"path": str(repo / "sifr")},
                "cloud_identity": {"execution": {"cargo_jobs": "2", "cloud_runtime_environment": {
                    "RAYON_NUM_THREADS": None, "OMP_NUM_THREADS": None}}}}
            self.endpoints[key] = value
            self.paths[key].write_text(json.dumps(value))
        for name, value in [("ROOT", root), ("MANIFEST", self.manifest), ("BUDGETS", self.budgets),
                            ("check_endpoints", lambda *_: self.endpoints),
                            ("validate_manifest", lambda data: [SimpleNamespace(id="case", warmups=1)]),
                            ("worker", self.worker)]:
            patcher = patch.object(cloud, name, value)
            patcher.start()
            self.addCleanup(patcher.stop)
        args = SimpleNamespace(baseline=self.paths["baseline"], candidate=self.paths["candidate"],
                               reference_compiler_commit="a" * 40, output=self.output)
        self.assertEqual(cloud.capture(args), 0)
        self.receipt_path = self.output / "receipt.json"
        self.receipt = cloud.read(self.receipt_path)

    def worker(self, endpoint, *args):
        def value(flag):
            return args[args.index(flag) + 1]
        output = Path(value("--output"))
        output.mkdir(parents=True, exist_ok=False)
        summary = {"case_id": "case", "latencies_ms": [1000.0], "peak_rss_bytes": 100000,
                   "cpu_time_ms": 10.0, "cache": {"hits": 0, "misses": 0},
                   "source": endpoint["cloud_source"],
                   "endpoint_receipt_sha256": cloud.digest(Path(value("--receipt")))}
        cloud.atomic(output / "summary.json", summary)
        raw = output / "samples/case/0.json"
        raw.parent.mkdir(parents=True)
        cloud.atomic(raw, {"case_id": "case", "warmup": "--warmup" in args,
            "command": [endpoint["artifact"]["path"]], "result": {
                "duration_ms": 1000.0, "peak_rss_bytes": 100000, "cpu_time_ms": 10.0,
                "exit_code": 0, "timed_out": False}})
        return ""

    def check(self):
        cloud.atomic(self.receipt_path, self.receipt)
        return cloud.check(SimpleNamespace(receipt=self.receipt_path))

    def test_complete_bound_evidence_passes(self):
        self.assertEqual(self.check(), 0)

    def test_changed_raw_sample_rejected(self):
        raw = self.output / "pairs/case/0/candidate/samples/case/0.json"
        raw.write_text('{}')
        with self.assertRaises(ValueError):
            self.check()

    def test_missing_raw_binding_rejected(self):
        self.receipt["raw_evidence_sha256"] = {}
        with self.assertRaises(ValueError):
            self.check()

    def test_forged_summaries_disagreeing_with_process_rejected(self):
        row = self.receipt["pairs"]["case"][0]
        row["candidate"]["latencies_ms"] = [10.0]
        folder = self.output / "pairs/case/0"
        cloud.atomic(folder / "pair.json", row)
        cloud.atomic(folder / "candidate/summary.json", row["candidate"])
        for path in [folder / "pair.json", folder / "candidate/summary.json"]:
            self.receipt["raw_evidence_sha256"][str(path.relative_to(self.output))] = cloud.digest(path)
        with self.assertRaisesRegex(ValueError, "actual raw measurement"):
            self.check()

    def test_stale_receipt_rejected(self):
        self.receipt["completed_unix"] -= 25 * 3600
        with self.assertRaisesRegex(ValueError, "stale"):
            self.check()

    def test_selectively_combined_invocations_rejected(self):
        self.receipt["pairs"]["case"][0]["invocation"] = "different"
        with self.assertRaisesRegex(ValueError, "combined"):
            self.check()

    def test_changed_verdict_rejected(self):
        self.receipt["evaluation"]["status"] = "regression"
        with self.assertRaisesRegex(ValueError, "recomputed"):
            self.check()

    def test_changed_corpus_or_endpoint_receipt_rejected(self):
        for mutate in [lambda: self.manifest.write_text('{}'),
                       lambda: self.paths["candidate"].write_text('{}')]:
            with self.subTest(mutate=mutate):
                saved_manifest = self.manifest.read_text()
                saved_endpoint = self.paths["candidate"].read_text()
                mutate()
                with self.assertRaises(ValueError):
                    self.check()
                self.manifest.write_text(saved_manifest)
                self.paths["candidate"].write_text(saved_endpoint)

    def test_runtime_thread_configuration_is_bound(self):
        expected = {"execution": {"cloud_runtime_environment": {"RAYON_NUM_THREADS": None, "OMP_NUM_THREADS": None}}}
        changed = copy.deepcopy(expected)
        changed["execution"]["cloud_runtime_environment"]["RAYON_NUM_THREADS"] = "4"
        with patch.object(cloud, "comparison_mismatches", return_value=[]):
            self.assertEqual(cloud.configuration_mismatches(expected, expected), [])
            self.assertIn("execution.cloud_runtime_environment", cloud.configuration_mismatches(expected, changed))

    def test_cli_malformed_shapes_are_invalid_not_regressions(self):
        for value in [[], None, {"schema_version": 1, "specification": []},
                      {"schema_version": 1, "specification": None}]:
            with self.subTest(value=value):
                self.receipt_path.write_text(json.dumps(value))
                result = subprocess.run([sys.executable, cloud.__file__, "check",
                    "--receipt", str(self.receipt_path)], text=True, capture_output=True)
                self.assertEqual(result.returncode, 2, result.stderr)
                self.assertIn("cloud performance error:", result.stderr)


if __name__ == "__main__":
    unittest.main()
