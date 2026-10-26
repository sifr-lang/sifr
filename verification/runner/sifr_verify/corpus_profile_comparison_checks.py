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

    def test_cargo_selects_both_artifacts_in_the_caller_graph(self):
        calls = []
        def run(command, **kwargs):
            calls.append((command, kwargs))
            messages = [{"reason": "compiler-artifact", "target": {"name": name, "kind": ["bin"]},
                         "executable": "/caller/custom-target/" + name}
                        for name in ("sifr", "sifr_corpus_oracle")]
            return subprocess.CompletedProcess(command, 0, "\n".join(map(json.dumps, messages)))
        env = {"CARGO_TARGET_DIR": "/caller/custom-target", "RUSTFLAGS": "--cfg caller"}
        result = self.profiles.prepare_development(Path("/source"), env, run)
        self.assertEqual(set(result), {"sifr", "sifr_corpus_oracle"})
        self.assertEqual(result["sifr_corpus_oracle"], "/caller/custom-target/sifr_corpus_oracle")
        self.assertEqual(calls[0][1]["env"], env)
        self.assertNotIn("--release", calls[0][0])
        def missing(command, **kwargs):
            return subprocess.CompletedProcess(command, 0, "")
        with self.assertRaises(RuntimeError):
            self.profiles.prepare_development(Path("/source"), env, missing)

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
