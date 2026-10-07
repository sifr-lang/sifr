"""Managed timer/counter admission rejects absent or changed measurements."""

import copy
import hashlib
import contextlib
import io
import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from benchmark_manifest import BenchmarkError
from measurement_timer import linux_timer, managed_timer_identity, require_managed_counters
from reference_profiles import ReferenceProfileError, assert_comparable


class MeasurementTimerTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.timer = Path(self.temporary.name) / "time"
        self.timer.write_text("test-only timer bytes")
        self.timer.chmod(0o755)

    def test_explicit_timer_identity_binds_actual_bytes_and_version(self):
        with patch.dict(os.environ, {"SIFR_PERFORMANCE_TIME": str(self.timer)}), patch(
            "measurement_timer.subprocess.run", return_value=subprocess.CompletedProcess(
                [], 0, "time (GNU Time) 1.10\n", "")
        ):
            identity = managed_timer_identity()
        self.assertEqual(identity, {"path": str(self.timer), "version": "time (GNU Time) 1.10",
                                    "sha256": hashlib.sha256(self.timer.read_bytes()).hexdigest()})

    def test_missing_relative_and_nonexecutable_timer_rejected(self):
        for value in (str(self.timer.parent / "missing"), "relative/time"):
            with self.subTest(value=value), patch.dict(os.environ, {"SIFR_PERFORMANCE_TIME": value}):
                with self.assertRaises(BenchmarkError):
                    linux_timer()
        self.timer.chmod(0o644)
        with patch.dict(os.environ, {"SIFR_PERFORMANCE_TIME": str(self.timer)}):
            with self.assertRaises(BenchmarkError):
                linux_timer()

    def test_non_gnu_or_failed_version_rejected(self):
        with patch.dict(os.environ, {"SIFR_PERFORMANCE_TIME": str(self.timer)}):
            with patch("measurement_timer.subprocess.run", return_value=subprocess.CompletedProcess(
                [], 0, "different timer", "")
            ), self.assertRaises(BenchmarkError):
                managed_timer_identity()
            with patch("measurement_timer.subprocess.run", side_effect=subprocess.TimeoutExpired([], 30)), \
                    self.assertRaises(BenchmarkError):
                managed_timer_identity()

    def test_managed_samples_require_both_own_counters(self):
        with patch.dict(os.environ, {"SIFR_PERFORMANCE_HOST_KIND": "managed-linux"}):
            require_managed_counters({"peak_rss_bytes": 1024, "cpu_time_ms": 0})
            for metrics in ({}, {"peak_rss_bytes": 1024}, {"cpu_time_ms": 1}):
                with self.subTest(metrics=metrics), self.assertRaises(BenchmarkError):
                    require_managed_counters(metrics)

    def test_managed_producer_cannot_use_cumulative_rss_fallback(self):
        import run_benchmarks
        with patch.dict(os.environ, {"SIFR_PERFORMANCE_HOST_KIND": "managed-linux"}), patch(
            "run_benchmarks.timed_command", return_value=["fixture"]
        ), patch("run_benchmarks.run_owned_process", return_value=(
            subprocess.CompletedProcess([], 0, "", ""), False)
        ), self.assertRaisesRegex(BenchmarkError, "per-process RSS"):
            run_benchmarks.run_subprocess(["fixture"], 1000)

    def test_changed_or_missing_managed_timer_identity_rejected(self):
        from reference_profile_tests import identity
        before = identity()
        before["host"]["cpu_power_policy"] = {"source": "managed-linux"}
        before["execution"]["measurement_timer"] = {"path": str(self.timer), "sha256": "a" * 64,
                                                     "version": "time (GNU Time) fixture"}
        for field in ("path", "sha256", "version"):
            after = copy.deepcopy(before)
            after["execution"]["measurement_timer"][field] = "changed"
            with self.subTest(field=field), self.assertRaisesRegex(ReferenceProfileError, "measurement_timer"):
                assert_comparable({"name": "fixture", "identity": before}, after)
        after = copy.deepcopy(before)
        after["execution"].pop("measurement_timer")
        with self.assertRaisesRegex(ReferenceProfileError, "incomplete"):
            assert_comparable({"name": "fixture", "identity": before}, after)

    def test_managed_command_selects_timer_and_physical_default_unchanged(self):
        from process_metrics import timed_command
        with patch.dict(os.environ, {"SIFR_PERFORMANCE_HOST_KIND": "managed-linux",
                                     "SIFR_PERFORMANCE_TIME": str(self.timer)}), patch(
            "process_metrics.platform.system", return_value="Linux"
        ):
            self.assertEqual(timed_command(["fixture"]), [str(self.timer), "-v", "fixture"])
        with patch.dict(os.environ, {}, clear=True), patch(
            "process_metrics.platform.system", return_value="Linux"
        ), patch("process_metrics.Path.exists", return_value=False):
            self.assertEqual(timed_command(["exploratory"]), ["exploratory"])

    def test_admission_reports_timer_failure_as_unavailable(self):
        import reference_admission
        errors = io.StringIO()
        with patch("sys.argv", ["reference_admission"]), patch(
            "reference_admission.admit_reference", side_effect=BenchmarkError("missing timer")
        ), contextlib.redirect_stderr(errors):
            self.assertEqual(reference_admission.main(), 1)
        self.assertIn("performance qualification unavailable: missing timer", errors.getvalue())
