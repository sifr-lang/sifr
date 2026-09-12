"""Named reference boundary tests; no host measurements or compiler builds."""

from __future__ import annotations

import copy
import json
import tempfile
import unittest
from unittest.mock import patch
from subprocess import CompletedProcess
from pathlib import Path

from trend_reports import build_trend_report

from reference_profiles import (
    ReferenceProfileError,
    assert_comparable,
    capture_profile,
    derive_budgets,
    load_profile,
    profile_digest,
    profile_path,
    validate_result_profile,
    validate_compiler_reference,
    validate_manifest_binding,
)


def identity():
    return {
        "host": {
            "system": "Linux", "os_version": "Ubuntu 24.04", "kernel": "6.8",
            "architecture": "x86_64", "cpu_models": ["Intel Core i7-4720HQ"],
            "physical_cores": 4, "logical_cpus": 8, "available_cpus": 8,
            "memory_capacity_gib": 12,
            "memory": {"total_bytes": 12 * 1024**3, "available_bytes": 8 * 1024**3},
        },
        "execution": {
            "rustc": "rustc 1.98.1", "cargo": "cargo 1.98.1", "python": "3.14.7",
            "build_profile": "dev", "control_mode": "latency", "cargo_jobs": "2",
            "rust_test_threads": "default", "build_environment": {},
            "benchmark_inputs_sha256": "a" * 64,
            "target_storage": {"filesystem": "ext4", "source": "disk"},
            "temporary_storage": {"filesystem": "ext4", "source": "disk"},
            "cargo_manifest_sha256": "b" * 64, "cargo_config_sha256": None,
            "user_cargo_config_sha256": None,
        },
    }


def baseline():
    return {
        "runner_version": 1,
        "metadata": {"reference_identity": identity()},
        "reference_capture": {"approval_owner": "compiler/performance", "controlled_host": True},
        "results": [{
            "id": "command",
            "metrics": {"median_ms": 100, "p95_ms": 120, "peak_rss_bytes": 100 * 1024**2,
                        "coefficient_variation": 0.01},
            "cache": {"hits": 0, "misses": 0},
        }],
    }


def template():
    return {"version": 1, "budgets": [{
        "benchmark_id": "command", "budget_id": "perf.command", "policy": "command-default",
        "thresholds": {"median_ms": 10, "p95_ms": 20, "peak_rss_bytes": 80 * 1024**2,
                       "timeout_ms": 60000},
        "cache": {},
    }]}


class NamedReferenceTests(unittest.TestCase):
    def profile(self):
        return {"name": "linux-reference", "identity": identity(), "baseline": baseline()}

    def test_matching_host(self):
        assert_comparable(self.profile(), identity())

    def test_rejects_each_host_identity_mismatch(self):
        for key in identity()["host"]:
            if key == "memory":
                continue
            with self.subTest(key=key):
                observed = identity()
                observed["host"][key] = "different"
                with self.assertRaisesRegex(ReferenceProfileError, f"host.{key}"):
                    assert_comparable(self.profile(), observed)

    def test_rejects_each_execution_mismatch(self):
        for key in identity()["execution"]:
            if key == "cargo_manifest_sha256":
                continue
            with self.subTest(key=key):
                observed = identity()
                observed["execution"][key] = "different"
                with self.assertRaisesRegex(ReferenceProfileError, f"execution.{key}"):
                    assert_comparable(self.profile(), observed)

    def test_candidate_source_change_is_recorded_not_rebaselined(self):
        observed = identity()
        observed["execution"]["cargo_manifest_sha256"] = "c" * 64
        assert_comparable(self.profile(), observed)

    def test_available_memory_is_telemetry_not_a_different_machine(self):
        observed = identity()
        observed["host"]["memory"]["available_bytes"] //= 2
        assert_comparable(self.profile(), observed)

    def test_missing_fields_cannot_match_each_other(self):
        profile = self.profile()
        profile["identity"]["execution"].pop("cargo_jobs")
        observed = copy.deepcopy(profile["identity"])
        with self.assertRaisesRegex(ReferenceProfileError, "incomplete"):
            assert_comparable(profile, observed)

    def test_name_cannot_escape_reference_directory(self):
        for name in ("../escape", "/tmp/escape", "", "Linux Reference"):
            with self.subTest(name=name), self.assertRaises(ReferenceProfileError):
                profile_path(name)

    def test_unknown_reference_does_not_use_historical_baseline(self):
        with tempfile.TemporaryDirectory() as raw:
            with self.assertRaisesRegex(ReferenceProfileError, "unavailable"):
                load_profile("unknown-host", Path(raw))

    def test_capture_is_atomic_and_cannot_replace_reference(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            path = capture_profile("linux-reference", baseline(), template(), root=root)
            initial = path.read_bytes()
            loaded = load_profile("linux-reference", root)
            self.assertEqual(loaded["identity"], identity())
            with self.assertRaisesRegex(ReferenceProfileError, "already exists"):
                capture_profile("linux-reference", baseline(), template(), root=root)
            self.assertEqual(path.read_bytes(), initial)
            self.assertEqual(list(root.iterdir()), [path])

    def test_unapproved_or_uncontrolled_capture_rejected(self):
        for key, value in (("approval_owner", "other"), ("controlled_host", False)):
            candidate = baseline()
            candidate["reference_capture"][key] = value
            with tempfile.TemporaryDirectory() as raw, self.assertRaises(ReferenceProfileError):
                capture_profile("linux-reference", candidate, template(), root=Path(raw))

    def test_missing_explicit_jobs_rejected(self):
        candidate = baseline()
        candidate["metadata"]["reference_identity"]["execution"]["cargo_jobs"] = "cargo-default"
        with tempfile.TemporaryDirectory() as raw, self.assertRaisesRegex(ReferenceProfileError, "CARGO_BUILD_JOBS"):
            capture_profile("linux-reference", candidate, template(), root=Path(raw))

    def test_partial_corpus_rejected(self):
        candidate = baseline()
        candidate["results"] = []
        with self.assertRaisesRegex(ReferenceProfileError, "complete"):
            derive_budgets(template(), candidate)

    def test_missing_rss_rejected(self):
        candidate = baseline()
        candidate["results"][0]["metrics"]["peak_rss_bytes"] = None
        with self.assertRaisesRegex(ReferenceProfileError, "RSS"):
            derive_budgets(template(), candidate)

    def test_reference_rules_preserve_timeout_and_template(self):
        original = template()
        captured = derive_budgets(original, baseline())["budgets"][0]["thresholds"]
        self.assertEqual(captured["median_ms"], 125)
        self.assertEqual(captured["p95_ms"], 170)
        self.assertEqual(captured["peak_rss_bytes"], 132 * 1024**2)
        self.assertEqual(captured["timeout_ms"], 60000)
        self.assertEqual(original, template())

    def test_editor_latency_ceiling_cannot_be_relaxed_by_capture(self):
        original = template()
        original["budgets"][0]["policy"] = "lsp-query"
        thresholds = derive_budgets(original, baseline())["budgets"][0]["thresholds"]
        self.assertEqual(thresholds["median_ms"], 10)
        self.assertEqual(thresholds["p95_ms"], 20)

    def test_frontend_cache_requirement_comes_from_reference(self):
        original, candidate = template(), baseline()
        original["budgets"][0]["policy"] = "frontend-query-edit-loop"
        candidate["results"][0]["cache"]["hits"] = 9
        entry = derive_budgets(original, candidate)["budgets"][0]
        self.assertEqual(entry["cache"], {"min_hits": 9})
        self.assertEqual(entry["thresholds"]["median_ms"], 105)

    def report(self, profile):
        return {"runner_version": 1, "metadata": {
            "reference_profile": profile["name"], "reference_identity": identity(),
            "source_dirty": False, "sample_scale": "manifest",
            "source_commit_at_start": "a" * 40, "compiler_fingerprint": "a" * 40,
            "reference_profile_sha256": profile_digest(profile),
            "host_control": {"status": "controlled"},
        }}

    def test_selected_reference_bound_to_producer(self):
        profile = self.profile()
        validate_result_profile(profile, self.report(profile))
        for key, value in (
            ("reference_profile", "other"),
            ("reference_profile_sha256", "stale"),
            ("source_dirty", True),
            ("compiler_fingerprint", "b" * 40),
            ("source_commit_at_start", ""),
            ("sample_scale", "smoke"),
            ("host_control", {"status": "controlled", "policy": {"changed": True}}),
            ("host_control", {"status": "record-only"}),
        ):
            report = self.report(profile)
            report["metadata"][key] = value
            with self.subTest(key=key), self.assertRaises(ReferenceProfileError):
                validate_result_profile(profile, report)


    def test_reference_compiler_must_be_merged(self):
        with patch("reference_profiles.subprocess.run", return_value=CompletedProcess([], 1)):
            with self.assertRaisesRegex(ReferenceProfileError, "ancestor"):
                validate_compiler_reference(Path("."), "a" * 40)

    def test_reference_tooling_changes_are_allowed(self):
        responses = [CompletedProcess([], 0), CompletedProcess([], 0, "verification/areas/performance/runner.py\n")]
        with patch("reference_profiles.subprocess.run", side_effect=responses):
            self.assertEqual(validate_compiler_reference(Path("."), "a" * 40), "a" * 40)

    def test_reference_compiler_or_lock_changes_are_rejected(self):
        for changed in ("Cargo.toml", "Cargo.lock", "crates/sifr/src/main.rs", "third_party/ruff"):
            responses = [CompletedProcess([], 0), CompletedProcess([], 0, changed)]
            with self.subTest(changed=changed), patch("reference_profiles.subprocess.run", side_effect=responses):
                with self.assertRaisesRegex(ReferenceProfileError, "compiler inputs"):
                    validate_compiler_reference(Path("."), "a" * 40)


    def test_historical_trend_does_not_compare_unknown_hardware(self):
        run = {"run_id": "test", "metadata": {"reference_profile": "linux", "reference_identity": identity()}}
        old = {"schema_version": 1, "runner_version": 1, "results": []}
        report = build_trend_report(run, old, 1)
        self.assertEqual(report["comparison_status"], "incomparable")
        self.assertEqual(report["results"], [])

    def test_mismatched_named_trend_has_no_numeric_deltas(self):
        run = {"run_id": "test", "metadata": {"reference_profile": "linux", "reference_identity": identity()}}
        old = {"schema_version": 1, "runner_version": 1, "results": [],
               "metadata": {"reference_identity": identity()}}
        old["metadata"]["reference_identity"]["host"]["cpu_models"] = ["Other CPU"]
        report = build_trend_report(run, old, 1)
        self.assertEqual(report["comparison_status"], "incomparable")
        self.assertEqual(report["results"], [])


    def test_checker_manifest_cannot_weaken_reference(self):
        with tempfile.TemporaryDirectory() as raw:
            path = Path(raw) / "manifest.json"
            path.write_text("{}")
            profile = self.profile()
            profile["baseline"]["manifest_sha256"] = "different"
            with self.assertRaisesRegex(ReferenceProfileError, "checker manifest"):
                validate_manifest_binding(profile, path)


    def test_checked_in_corpus_has_derivation_for_every_policy(self):
        data = Path(__file__).resolve().parent / "data"
        original = json.loads((data / "budgets.json").read_text())
        measured = json.loads((data / "baselines.json").read_text())
        derived = derive_budgets(original, measured)
        self.assertEqual(len(derived["budgets"]), len(original["budgets"]))
        for before, after in zip(original["budgets"], derived["budgets"], strict=True):
            self.assertEqual(before["thresholds"]["timeout_ms"], after["thresholds"]["timeout_ms"])
            if before["policy"] in {"lsp-query", "formatter-command-default"}:
                self.assertLessEqual(after["thresholds"]["median_ms"], before["thresholds"]["median_ms"])
                self.assertLessEqual(after["thresholds"]["p95_ms"], before["thresholds"]["p95_ms"])


def run_self_test():
    result = unittest.TestResult()
    unittest.defaultTestLoader.loadTestsFromTestCase(NamedReferenceTests).run(result)
    if not result.wasSuccessful():
        raise ReferenceProfileError(str(result.errors + result.failures))


if __name__ == "__main__":
    unittest.main()
