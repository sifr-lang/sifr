"""Negative controls for one fresh emission with both policy owners."""
from __future__ import annotations

import contextlib
import copy
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch

from .demo_freshness_policy import (
    FRESHNESS_SUITES, STRICT_FLAG, requires_strict_freshness,
    validate_freshness_ownership, validate_strict_freshness_evidence,
)
from .paths import REPO_ROOT
from .profile_results import AreaResultError
from .profiles import build_profile_plan, load_profile


def load_source(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


GCQ_ROOT = REPO_ROOT / "verification/areas/generated_code_quality"
sys.path.insert(0, str(GCQ_ROOT))
inventory = load_source("strict_freshness_inventory", GCQ_ROOT / "inventory_gates.py")
adapter = load_source("strict_freshness_adapter", GCQ_ROOT / "runner.py")
emitter = load_source("strict_freshness_emitter", REPO_ROOT / "scripts/check_demo_emitted_freshness.py")


class FreshnessOwnershipTests(unittest.TestCase):
    def test_each_delivery_profile_has_exactly_one_strict_owner(self):
        for name in ("create-pr", "merge", "nightly", "release"):
            profile = load_profile(name)
            validate_freshness_ownership(profile)
            strict = requires_strict_freshness(profile, "generated_code_quality")
            self.assertEqual(strict, name != "create-pr")
            self.assertEqual("demo-emitted-freshness" in profile["guardrail_steps"], not strict)
            quality = next(a for a in build_profile_plan(name)["selected_areas"]
                           if a["area"] == "generated_code_quality")
            self.assertEqual(quality.get("require_fresh_demo_companions", False), strict)

    def test_profile_contract_matches_actual_freshness_selections(self):
        actual = {name for name, plan in adapter.PROFILE_SUITES.items()
                  if any(gate == "freshness" for gate, _, _ in plan)}
        self.assertEqual(FRESHNESS_SUITES, actual | {"freshness"})

    def test_missing_duplicate_wrong_area_and_empty_owner_reject(self):
        base = load_profile("merge")
        for mutation in ("missing", "duplicate", "wrong-area", "smoke", "false"):
            profile = copy.deepcopy(base)
            quality = next(a for a in profile["selected_areas"]
                           if a["area"] == "generated_code_quality")
            if mutation == "missing":
                quality.pop("require_fresh_demo_companions")
            elif mutation == "duplicate":
                profile["guardrail_steps"].append("demo-emitted-freshness")
            elif mutation == "wrong-area":
                quality["area"] = "core_language"
            elif mutation == "smoke":
                quality["suites"] = ["smoke"]
            else:
                quality["require_fresh_demo_companions"] = False
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                validate_freshness_ownership(profile)

    def test_smoke_cannot_accept_strict_option_without_freshness(self):
        with self.assertRaisesRegex(SystemExit, "requires a selected freshness"):
            adapter.main(["--suite", "smoke", STRICT_FLAG])

    def test_cli_rejects_strict_option_for_another_area(self):
        from .areas import run_command
        from .errors import DiscoveryError
        with self.assertRaises(DiscoveryError):
            run_command(["run", "--area", "core_language", STRICT_FLAG])

    def test_profile_dispatch_passes_explicit_flag_and_requires_result(self):
        from .profile_runner import ProfileRunner
        runner = ProfileRunner("merge", [], correctness_only=True)
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "result.json"
            path.write_text(json.dumps(self.payload()))
            def run_area(**kwargs):
                self.assertIn(STRICT_FLAG, kwargs["command_builder"]("--area", "generated_code_quality"))
                return path
            with patch("sifr_verify.profile_runner.run_selected_area", side_effect=run_area):
                runner.run_area("generated_code_quality", ["representative"])
                path.write_text(json.dumps({"suites": []}))
                with self.assertRaises(AreaResultError):
                    runner.run_area("generated_code_quality", ["representative"])

    @staticmethod
    def payload():
        return {"suites": [{"cases": [{"variants": [{
            "label": "freshness", "status": "pass", "argv": ["quality.py", STRICT_FLAG],
            "case_evidence": {"checks": {"strict": "pass", "debt": "pass"}},
        }]}]}]}

    def test_missing_incomplete_unflagged_or_failed_result_rejects(self):
        validate_strict_freshness_evidence(self.payload())
        for mutation in ("missing", "no-detail", "debt-only", "failed", "unflagged"):
            payload = self.payload()
            variant = payload["suites"][0]["cases"][0]["variants"][0]
            if mutation == "missing":
                payload["suites"] = []
            elif mutation == "no-detail":
                variant.pop("case_evidence")
            elif mutation == "debt-only":
                variant["case_evidence"]["checks"]["strict"] = "not-requested"
            elif mutation == "failed":
                variant["status"] = "fail"
            else:
                variant["argv"] = []
            with self.subTest(mutation=mutation), self.assertRaises(AreaResultError):
                validate_strict_freshness_evidence(payload)

    def test_area_forwards_flag_only_to_freshness_gate(self):
        area_suite = {"name": "representative", "cases": [{
            "id": "representative", "entry": "demos", "command": "generated-code-quality-mode",
        }]}
        variant = {"status": "pass", "duration_ms": 1}
        shared = {gate: variant for gate, _, _ in adapter.PROFILE_SUITES["representative"]
                  if gate in adapter.SHARED_GATES}
        with patch.object(adapter, "run_shared_gates", return_value=shared), \
             patch.object(adapter, "run_gate", return_value=variant) as gate, \
             contextlib.redirect_stdout(io.StringIO()):
            adapter.run_suite(area_suite, True)
        calls = [call.args for call in gate.call_args_list]
        self.assertEqual([args[1] for args in calls if args[4]], ["freshness"])

    def test_successful_child_without_policy_receipt_is_not_a_pass(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with patch.object(adapter, "REPO_ROOT", root), \
                 patch.object(adapter.subprocess, "run", return_value=subprocess.CompletedProcess([], 0)), \
                 contextlib.redirect_stdout(io.StringIO()):
                result = adapter.run_gate("representative", "freshness", None, None, True)
            self.assertEqual(result["status"], "fail")
            self.assertEqual(result["mismatches"], ["missing-policy-evidence"])


class FreshnessPolicyTests(unittest.TestCase):
    def invoke(self, *, stale=False, orphan=False, strict=False, returncode=None, stdout=None,
               debt_error=False):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            pair = root / "demos/example"
            pair.mkdir(parents=True)
            (pair / "emitted.rs").write_text("generated")
            if not orphan:
                (pair / "main.sifr").write_text("print(1)")
            if stdout is None:
                stdout = ("stale: demos/example/emitted.rs\n" if stale
                          else inventory.FRESHNESS_SENTINEL + "\n")
            if returncode is None:
                returncode = 1 if stale else 0
            run = Mock(return_value=subprocess.CompletedProcess([], returncode, stdout, ""))
            compare = Mock(side_effect=RuntimeError("debt mismatch") if debt_error else None)
            records = []
            def record(*args):
                records.extend(args[2])
                return root / "evidence.json"
            with patch.object(inventory, "REPO_ROOT", root), \
                 patch.object(inventory, "load_debt", return_value={}), \
                 patch.object(inventory, "validate_debt_owners"), \
                 patch.object(inventory, "compare_exact_debt", compare), \
                 contextlib.redirect_stdout(io.StringIO()):
                result = inventory.gate_freshness(
                    lambda bucket, case, action: action(), record, lambda mode: "run",
                    run, lambda: "compiler", require_fresh=strict,
                )
            self.assertEqual(run.call_count, 1, "both policies must share one fresh emission")
            self.assertEqual(records, [result])
            return result, compare.call_args.kwargs["actual"]

    def test_fresh_emission_satisfies_both_policies_once(self):
        result, actual = self.invoke(strict=True)
        self.assertIsNone(actual)
        self.assertEqual(result["checks"], {"strict": "pass", "debt": "pass"})
        self.assertEqual(result["companions"], ["demos/example/emitted.rs"])

    def test_allowed_stale_debt_does_not_bypass_strict_policy(self):
        result, actual = self.invoke(stale=True)
        self.assertEqual(result["checks"]["strict"], "not-requested")
        self.assertEqual(actual["stale"], ["demos/example/emitted.rs"])
        with self.assertRaisesRegex(RuntimeError, "strict demo freshness"):
            self.invoke(stale=True, strict=True)

    def test_orphan_and_unexpected_debt_are_still_checked(self):
        _, actual = self.invoke(orphan=True, strict=True)
        self.assertEqual(actual, {"stale": [], "orphans": ["demos/example/emitted.rs"]})
        with self.assertRaisesRegex(RuntimeError, "debt mismatch"):
            self.invoke(orphan=True, strict=True, debt_error=True)
        with self.assertRaisesRegex(RuntimeError, "debt mismatch"):
            self.invoke(stale=True, strict=True, debt_error=True)

    def test_empty_malformed_and_compiler_failure_protocols_reject(self):
        for code, text in [(0, ""), (1, ""), (1, "compiler failed\n"),
                           (0, "stale: demos/example/emitted.rs\n"),
                           (2, inventory.FRESHNESS_SENTINEL + "\n")]:
            with self.subTest(code=code, text=text), self.assertRaisesRegex(RuntimeError, "protocol"):
                self.invoke(strict=True, returncode=code, stdout=text)


class FreshEmissionTests(unittest.TestCase):
    def populate(self, root):
        for relative in emitter.NON_AUTHORITATIVE_TEST_PROJECT_REFERENCES:
            reference = root / relative
            reference.parent.mkdir(parents=True, exist_ok=True)
            reference.write_text("reference")
        compiler = root / "compiler"
        compiler.write_text("test compiler path")
        pair = root / "demos/example"
        pair.mkdir(parents=True)
        (pair / "main.sifr").write_text("print(1)")
        (pair / "emitted.rs").write_bytes(b"fresh bytes")
        return compiler

    def test_reference_boundaries_still_reject_missing_and_authoritative_files(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.populate(root)
            relative = sorted(emitter.NON_AUTHORITATIVE_TEST_PROJECT_REFERENCES)[0]
            with patch.object(emitter, "REPO_ROOT", root):
                emitter.validate_non_authoritative_references()
                (root / relative).unlink()
                with self.assertRaisesRegex(RuntimeError, "missing non-authoritative"):
                    emitter.validate_non_authoritative_references()
                (root / relative).write_text("reference")
                (root / relative).with_name("emitted.rs").write_text("forbidden")
                with self.assertRaisesRegex(RuntimeError, "must not acquire"):
                    emitter.validate_non_authoritative_references()

    def test_fresh_stale_and_compiler_error_run_actual_emission_path(self):
        for code, output, expected in [(0, b"fresh bytes", 0), (0, b"changed", 1), (2, b"", 2)]:
            with tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                compiler = self.populate(root)
                stderr = io.TextIOWrapper(io.BytesIO())
                with patch.object(emitter, "REPO_ROOT", root), \
                     patch.object(sys, "argv", ["freshness", "--sifr", str(compiler)]), \
                     patch.object(emitter, "emit_many", return_value=[subprocess.CompletedProcess(
                         [], code, output, b"compiler diagnostic")]) as run, \
                     contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(stderr):
                    self.assertEqual(emitter.main(), expected)
                self.assertEqual(run.call_count, 1)
                self.assertEqual(run.call_args.args[1], [root / "demos/example/main.sifr"])



batch = sys.modules[emitter.emit_many.__module__]


class BatchEmissionTests(unittest.TestCase):
    def test_empty_relative_and_zero_worker_selections_reject(self):
        for files, jobs in [([], 1), ([Path("relative")], 1), ([Path("/absolute")], 0)]:
            with self.assertRaises(RuntimeError):
                batch.emit_many(Path("/compiler"), files, jobs, Path("/repo"))

    def test_explicit_formatter_is_preserved(self):
        with patch.dict(batch.os.environ, {"RUSTFMT": "/explicit/formatter"}), \
             patch.object(batch.subprocess, "run") as run:
            self.assertEqual(batch.formatter_environment(Path("/unused"))["RUSTFMT"],
                             "/explicit/formatter")
            run.assert_not_called()

    def test_pinned_formatter_is_resolved_once_and_version_mismatch_rejects(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "rust-toolchain.toml").write_text('[toolchain]\nchannel="pinned"\n')
            formatter = root / "rustfmt"
            formatter.write_text("tool")
            for matching in (True, False):
                outputs = [str(formatter).encode() + b"\n", b"rustfmt pinned\n",
                           b"rustfmt pinned\n" if matching else b"wrong\n"]
                with patch.dict(batch.os.environ, {}, clear=True), \
                     patch.object(batch.subprocess, "run", side_effect=[
                         subprocess.CompletedProcess([], 0, value, b"") for value in outputs
                     ]) as run:
                    if matching:
                        self.assertEqual(batch.formatter_environment(root)["RUSTFMT"], str(formatter))
                    else:
                        with self.assertRaisesRegex(RuntimeError, "match"):
                            batch.formatter_environment(root)
                    self.assertEqual(run.call_args_list[0].args[0],
                                     ["rustup", "which", "--toolchain", "pinned", "rustfmt"])

    def test_missing_mismatched_stale_and_incomplete_results_cannot_pass(self):
        for mutation in ("missing", "wrong-source", "reordered", "empty-rust",
                         "missing-rust", "extra", "wrong-exit", "unexpected-stdout", "failed"):
            with tempfile.TemporaryDirectory() as directory:
                output = Path(directory)
                entry = {"index": 0, "source": "/source.sifr", "exit_code": 0, "stderr": ""}
                report = {"schema": 1, "entries": [entry]}
                (output / "0.rs").write_text("generated Rust")
                result = subprocess.CompletedProcess([], 0, b"", b"")
                if mutation == "missing":
                    report["entries"] = []
                elif mutation == "wrong-source":
                    entry["source"] = "/other.sifr"
                elif mutation == "reordered":
                    entry["index"] = 1
                elif mutation == "empty-rust":
                    (output / "0.rs").write_bytes(b"")
                elif mutation == "missing-rust":
                    (output / "0.rs").unlink()
                elif mutation == "extra":
                    (output / "stale.rs").write_text("stale")
                elif mutation == "wrong-exit":
                    result.returncode = 1
                elif mutation == "unexpected-stdout":
                    result.stdout = b"noise"
                elif mutation == "failed":
                    entry.update(exit_code=1, stderr="diagnostic")
                (output / "report.json").write_text(json.dumps(report))
                with self.subTest(mutation=mutation), self.assertRaises((RuntimeError, OSError)):
                    batch.read_results(output, [Path("/source.sifr")], result)

    def test_bounded_batches_restore_lexical_order_and_remove_only_successful_owner(self):
        with tempfile.TemporaryDirectory() as directory:
            owner = Path(directory) / "owner"
            owner.mkdir()
            files = [Path(f"/source-{index}.sifr") for index in range(7)]
            def run(argv, **kwargs):
                manifest = json.loads(Path(argv[3]).read_text())
                output = Path(argv[5])
                output.mkdir()
                entries = []
                for index, source in enumerate(manifest["files"]):
                    (output / f"{index}.rs").write_text(source)
                    entries.append({"index": index, "source": source, "exit_code": 0, "stderr": ""})
                (output / "report.json").write_text(json.dumps({"schema": 1, "entries": entries}))
                return subprocess.CompletedProcess(argv, 0, b"", b"")
            with patch.object(batch, "formatter_environment", return_value={}), \
                 patch.object(batch.tempfile, "mkdtemp", return_value=str(owner)), \
                 patch.object(batch.subprocess, "run", side_effect=run) as launched:
                results = batch.emit_many(Path("/compiler"), files, 3, Path(directory))
                self.assertEqual(launched.call_count, 3)
                self.assertEqual([result.stdout for result in results],
                                 [str(path).encode() for path in files])
                self.assertFalse(owner.exists())


def policy_checks():
    suite = unittest.defaultTestLoader.loadTestsFromModule(sys.modules[__name__])
    result = unittest.TextTestRunner(verbosity=0).run(suite)
    if not result.wasSuccessful():
        raise AssertionError(result.errors + result.failures)


if __name__ == "__main__":
    unittest.main()
