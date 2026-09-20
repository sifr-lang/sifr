#!/usr/bin/env python3
"""Named trace-directory contracts against an exact prepared installed CLI."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import statistics
import subprocess
import sys
import tempfile
import time
import unittest

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "verification/areas/performance"))
import compiler_lanes


class TraceDirectoryAcceptance(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.receipt = json.loads(RECEIPT.read_text())
        compiler_lanes.validate_receipt(ROOT, "product-installed-optimized", cls.receipt)
        cls.binary = cls.receipt["artifact"]["path"]
        cls.temporary = tempfile.TemporaryDirectory(prefix="sifr-trace-acceptance-")
        cls.root = Path(cls.temporary.name)
        cls.source = cls.root / "secret-source-sentinel.sifr"
        cls.source.write_text('def main():\n    print("trace-source-sentinel")\n')
        cls.bad = cls.root / "bad.sifr"
        cls.bad.write_text('def main():\n    x: int = "bad"\n')
        cls.env = dict(os.environ, SIFR_TRACE_SECRET="trace-environment-sentinel",
                       SIFR_CACHE_DIR=str(cls.root / "cache"))
        cls.sequence = 0

    @classmethod
    def tearDownClass(cls):
        cls.temporary.cleanup()

    def invoke(self, args, trace=False, destination=None, input=None):
        type(self).sequence += 1
        destination = destination or self.root / f"trace-{self.sequence}"
        command = [self.binary]
        if trace:
            command += ["--trace-dir", str(destination)]
        begin = time.perf_counter_ns()
        result = subprocess.run(command + list(map(str, args)), cwd=self.root,
                                env=self.env, capture_output=True, input=input)
        elapsed = (time.perf_counter_ns() - begin) / 1000
        report = None
        if trace and (destination / "trace-v1.json").exists():
            data = (destination / "trace-v1.json").read_bytes()
            self.assertLessEqual(len(data), 32768)
            report = json.loads(data)
            for secret in [b"trace-source-sentinel", b"trace-environment-sentinel",
                           b"secret-source-sentinel", str(self.root).encode()]:
                self.assertNotIn(secret, data)
            self.assertEqual(list(destination.iterdir()), [destination / "trace-v1.json"])
        return result, report, elapsed

    def test_trace_dir_cli_contract(self):
        result, _, _ = self.invoke(["--help"])
        self.assertIn(b"--trace-dir <DIR>", result.stdout)
        result, _, _ = self.invoke(["check", "--trace-dir"])
        self.assertEqual(result.returncode, 2)
        directory = self.root / "after-command"
        result, _, _ = self.invoke(["check", self.source, "--trace-dir", directory])
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue((directory / "trace-v1.json").is_file())

    def test_trace_dir_versioned_artifacts(self):
        before = set(self.root.glob("trace-*"))
        plain, _, _ = self.invoke(["check", self.source])
        self.assertEqual(set(self.root.glob("trace-*")), before)
        result, report, _ = self.invoke(["check", self.source], trace=True)
        self.assertEqual(result.returncode, plain.returncode)
        self.assertEqual(report["schema_version"], 1)
        self.assertEqual(report["compiler_identity"], self.receipt["embedded_compatibility_identity"]["compiler_build_id"])
        self.assertEqual(report["command"], "check")
        self.assertEqual(report["outcome"], "success")
        self.assertTrue(any(r["owner"] == "ProjectCacheReport" for r in report["reports"]))
        result, report, _ = self.invoke(["trace", self.source], trace=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(any(r.get("phase") == "parse" for r in report["reports"]))

    def test_trace_dir_redaction_and_size_bound(self):
        result, report, _ = self.invoke(["trace", self.source], trace=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report["max_bytes"], 32768)
        self.assertEqual(report["max_reports"], 64)
        self.assertLessEqual(len(report["reports"]), 64)
        self.assertGreaterEqual(report["dropped_reports"], 0)
        # The Rust named test separately drives truncation beyond both bounds.

    def test_trace_dir_output_and_failure_contract(self):
        for args in [["emit", self.source], ["--diagnostic-format", "json", "check", self.source],
                     ["--diagnostic-format", "json", "check", self.bad],
                     ["check", self.root / "missing.sifr"], ["run", "--quiet", self.source]]:
            plain, _, _ = self.invoke(args)
            traced, report, _ = self.invoke(args, trace=True)
            self.assertEqual(traced.returncode, plain.returncode, traced.stderr)
            self.assertEqual(traced.stdout, plain.stdout)
            self.assertIsNotNone(report)
            self.assertEqual(report["exit_code"], traced.returncode)
            self.assertEqual(report["outcome"], "success" if traced.returncode == 0 else "failure")
            if args[0] == "run":
                self.assertEqual(traced.returncode, 0, traced.stderr)
                self.assertEqual(traced.stdout, b"trace-source-sentinel\n")
                self.assertTrue(any(r["owner"] == "BuildReport" for r in report["reports"]))
        for destination in [self.root, self.root / "absent-parent" / "trace", Path("/proc/sifr-trace-unwritable")]:
            result, _, _ = self.invoke(["emit", self.source], trace=True, destination=destination)
            self.assertEqual(result.returncode, 2)
            self.assertEqual(result.stdout, b"")
            self.assertIn(b"--trace-dir", result.stderr)
        # EOF closes the real LSP; trace diagnostics never enter transport.
        plain, _, _ = self.invoke(["lsp", "--stdio"], input=b"")
        traced, report, _ = self.invoke(["lsp", "--stdio"], trace=True, input=b"")
        self.assertEqual(traced.stdout, plain.stdout)
        self.assertEqual(traced.returncode, plain.returncode)
        self.assertEqual(report["command"], "lsp")

    def test_trace_dir_timing_attribution(self):
        # Warm preparation is outside observations. No broad DX p95 claim.
        self.invoke(["check", self.source])
        rows = []
        for _ in range(10):
            plain, _, off = self.invoke(["check", self.source])
            traced, report, on = self.invoke(["check", self.source], trace=True)
            self.assertEqual(plain.returncode, traced.returncode)
            self.assertEqual(plain.stdout, traced.stdout)
            self.assertGreaterEqual(report["invocation_us_before_final_write"], report["trace_overhead_us_before_final_write"])
            self.assertGreater(report["trace_overhead_us_before_final_write"], 0)
            self.assertFalse(report["final_write_included"])
            rows.append({"off_wall_us": off, "on_wall_us": on, "report": report})
        EVIDENCE.write_text(json.dumps({"candidate": self.receipt["source_commit"],
            "artifact_sha256": self.receipt["artifact"]["sha256"], "receipt_sha256": hashlib.sha256(RECEIPT.read_bytes()).hexdigest(),
            "rows": rows, "median_off_us": statistics.median(r["off_wall_us"] for r in rows),
            "median_on_us": statistics.median(r["on_wall_us"] for r in rows)}, indent=2) + "\n")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--receipt", required=True, type=Path)
    parser.add_argument("--evidence", required=True, type=Path)
    args = parser.parse_args()
    RECEIPT, EVIDENCE = args.receipt.resolve(), args.evidence.resolve()
    unittest.main(argv=[sys.argv[0]], failfast=True, verbosity=2)
