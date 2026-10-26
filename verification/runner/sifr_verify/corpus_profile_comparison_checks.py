"""Cross-profile equivalence keeps bytes, maps, modules and complete selection."""
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

from .cargo_sysroot_setup_checks import sysroot_module


class CorpusProfileComparisonTests(unittest.TestCase):
    def setUp(self):
        sysroot_module()
        import corpus_profiles
        self.profiles = corpus_profiles

    def test_cargo_selects_exact_libtest_in_the_caller_graph(self):
        calls = []
        message = {"reason": "compiler-artifact",
                   "target": {"name": "sifr_driver", "kind": ["lib"]},
                   "profile": {"test": True}, "executable": "/caller/custom-target/driver-test"}
        def run(command, **kwargs):
            calls.append((command, kwargs))
            return subprocess.CompletedProcess(command, 0, json.dumps(message))
        env = {"CARGO_TARGET_DIR": "/caller/custom-target", "RUSTFLAGS": "--cfg caller"}
        result = self.profiles.prepare_development(Path("/source"), env, run)
        self.assertEqual(result, {"driver_test": "/caller/custom-target/driver-test"})
        self.assertEqual(calls[0][1]["env"], env)
        self.assertIn("--no-run", calls[0][0])
        self.assertNotIn("--release", calls[0][0])
        for records in ([], [message | {"profile": {"test": False}}],
                        [message, message | {"executable": "/other/driver-test"}]):
            with self.subTest(records=records), self.assertRaises(RuntimeError):
                self.profiles.prepare_development(Path("/source"), env,
                    lambda command, **kw: subprocess.CompletedProcess(
                        command, 0, "\n".join(map(json.dumps, records))))

    def test_development_execution_requires_the_exact_case_and_one_pass(self):
        import contextlib
        import io
        from unittest.mock import patch
        from .corpus_ownership import CORPUS_CASE
        from .errors import VerificationError
        passed = (f"test {CORPUS_CASE} ... ok\n"
                  "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 871 filtered out;\n").encode()
        with patch.object(self.profiles.subprocess, "run",
                          return_value=subprocess.CompletedProcess([], 0, passed)) as run, \
             contextlib.redirect_stdout(io.StringIO()):
            self.profiles.run_development("/prepared/driver-test", Path("/source"), {})
        self.assertEqual(run.call_args.args[0],
                         ["/prepared/driver-test", "--exact", CORPUS_CASE, "--ignored", "--nocapture"])
        for output in (b"", passed.replace(b"1 passed", b"0 passed"),
                       passed.replace(b"0 ignored", b"1 ignored"),
                       passed.replace(CORPUS_CASE.encode(), b"other_test")):
            with self.subTest(output=output), patch.object(
                self.profiles.subprocess, "run",
                return_value=subprocess.CompletedProcess([], 0, output)
            ), contextlib.redirect_stdout(io.StringIO()), self.assertRaises((VerificationError, RuntimeError)):
                self.profiles.run_development("/prepared/driver-test", Path("/source"), {})

    def test_failed_development_assertions_keep_their_diagnostic_output(self):
        import contextlib
        import io
        from unittest.mock import patch
        stream = io.StringIO()
        with patch.object(self.profiles.subprocess, "run",
                          return_value=subprocess.CompletedProcess([], 101, b"assertion details\n")), \
             contextlib.redirect_stdout(stream), self.assertRaises(subprocess.CalledProcessError):
            self.profiles.run_development("/prepared/driver-test", Path("/source"), {})
        self.assertEqual(stream.getvalue(), "assertion details\n")

    def test_exact_outputs_accept_set_order_only_and_reject_missing_or_changed_coverage(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixtures = root / "crates/sifr/tests/e2e/pass"
            development, release = root / "development", root / "release"
            for path in (fixtures, development, release):
                path.mkdir(parents=True)
            rows = []
            for i in range(701):
                name = f"case_{i:04d}"
                (fixtures / (name + ".sifr")).write_bytes(b"source")
                for output in (development, release):
                    (output / (name + ".sifr")).write_bytes(b"source")
                    (output / (name + ".source.rs")).write_bytes(b"exact Rust\n")
                rows.append({"fixture": name, "stdlib_modules": ["a", "b"],
                             "generated_source_maps": [{"path": "main", "source_bytes": 11}]})
            left = {"selection": None, "failures": [], "rows": rows, "portable_payload_sha256": "dev"}
            right = json.loads(json.dumps(left))
            right["portable_payload_sha256"] = "release"
            for row in right["rows"]:
                row["stdlib_modules"].reverse()
            def check(report):
                (development / "coverage.json").write_text(json.dumps(left))
                (release / "coverage.json").write_text(json.dumps(report))
                self.profiles.compare_profiles(root, development, release)
            check(right)
            def changed(mutator):
                report = json.loads(json.dumps(right))
                mutator(report)
                with self.assertRaises(AssertionError):
                    check(report)
            for mutation in (
                lambda r: r["rows"].pop(),
                lambda r: r["rows"].reverse(),
                lambda r: r["rows"][0]["stdlib_modules"].append("b"),
                lambda r: r["rows"][0].update(stdlib_modules=["a", "c"]),
                lambda r: r["rows"][0]["generated_source_maps"][0].update(source_bytes=12),
                lambda r: r.update(selection=["case_0000"]),
                lambda r: r.update(failures=["error"]),
            ):
                changed(mutation)
            source = release / "case_0000.source.rs"
            source.write_bytes(b"different")
            with self.assertRaises(AssertionError):
                check(right)
            source.write_bytes(b"exact Rust\n")
            (release / "case_0000.sifr").write_bytes(b"different")
            with self.assertRaises(AssertionError):
                check(right)
