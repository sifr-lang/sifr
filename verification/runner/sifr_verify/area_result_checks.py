"""Required area execution evidence, including adversarial aggregate reports."""

from __future__ import annotations

import copy
import json
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch

from . import profile_area_steps
from .profile_results import AreaResultError, validate_area_result


def passing_case(name: str) -> dict:
    return {"id": name, "variants": [
        {"label": name, "status": "pass", "mismatches": [],
         "expected_exit_code": 0, "actual_exit_code": 0},
    ]}


class AreaResultTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = TemporaryDirectory(prefix="sifr-area-result-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.path = self.root / "result.json"
        self.payload = {
            "schema_version": 1, "area": "runtime_platform", "bless": False,
            "suites": [{"name": "platform-golden", "blocking": True,
                        "total_variants": 1, "total_failures": 0,
                        "cases": [passing_case("body-text")]}],
            "summary": {"blocking_failures": 0, "total_variants": 1},
        }
        self.suite = self.payload["suites"][0]
        self.variant = self.suite["cases"][0]["variants"][0]

    def validate(self) -> dict:
        self.path.write_text(json.dumps(self.payload))
        return validate_area_result(self.path, area="runtime_platform",
                                    expected_suites=["platform-golden"])

    def test_complete_passing_execution_is_accepted(self) -> None:
        self.assertEqual(self.validate(), self.payload)

    def test_negative_diagnostic_with_expected_nonzero_exit_is_accepted(self) -> None:
        self.variant.update(expected_exit_code=1, actual_exit_code=1)
        self.validate()

    def test_metadata_only_variant_does_not_require_a_child_process(self) -> None:
        self.suite["cases"][0]["variants"] = [
            {"label": "metadata", "status": "pass", "mismatches": []}]
        self.validate()

    def test_fuzz_policy_can_own_exit_semantics_without_a_fixed_expected_code(self) -> None:
        del self.variant["expected_exit_code"]
        self.variant["actual_exit_code"] = 1
        self.validate()

    def test_explicit_nonblocking_failure_policy_is_preserved(self) -> None:
        self.suite.update(blocking=False, total_failures=1)
        self.variant.update(status="fail", actual_exit_code=1, mismatches=["expected failure"])
        self.validate()

    def test_explicit_nonblocking_skip_does_not_become_required_coverage(self) -> None:
        self.suite["blocking"] = False
        self.variant.update(status="skip", actual_exit_code=None)
        self.validate()

    def test_missing_case_rows_cannot_be_hidden_by_positive_aggregate(self) -> None:
        for value in (None, [], {}, True):
            with self.subTest(value=value):
                self.suite["cases"] = value
                with self.assertRaises(AreaResultError):
                    self.validate()

    def test_missing_variant_rows_cannot_be_hidden_by_positive_aggregate(self) -> None:
        for case in (None, {}, {"variants": []}, {"variants": {}}, {"variants": True}):
            with self.subTest(case=case):
                self.suite["cases"] = [case]
                with self.assertRaises(AreaResultError):
                    self.validate()

    def test_invalid_variant_rows_are_rejected(self) -> None:
        for value in (None, [], False, {}, {"status": "pending"}, {"status": ["pass"]}):
            with self.subTest(value=value):
                self.suite["cases"][0]["variants"] = [value]
                with self.assertRaises(AreaResultError):
                    self.validate()

    def test_blocking_skip_is_rejected_even_with_zero_failure_counters(self) -> None:
        self.variant.update(status="skip", actual_exit_code=None)
        with self.assertRaisesRegex(AreaResultError, "blocking variant"):
            self.validate()

    def test_blocking_failure_is_rejected_even_with_zero_failure_counters(self) -> None:
        self.variant["status"] = "fail"
        with self.assertRaises(AreaResultError):
            self.validate()

    def test_nonblocking_failure_counter_must_match_actual_rows(self) -> None:
        self.suite["blocking"] = False
        self.variant["status"] = "fail"
        with self.assertRaises(AreaResultError):
            self.validate()

    def test_suite_count_must_match_actual_rows(self) -> None:
        self.suite["total_variants"] = 2
        with self.assertRaises(AreaResultError):
            self.validate()

    def test_summary_count_must_match_all_actual_rows(self) -> None:
        self.payload["summary"]["total_variants"] = 2
        with self.assertRaises(AreaResultError):
            self.validate()

    def test_unreported_additional_case_cannot_hide_behind_old_count(self) -> None:
        self.suite["cases"].append(passing_case("another"))
        with self.assertRaises(AreaResultError):
            self.validate()

    def test_contradictory_passing_process_evidence_is_rejected(self) -> None:
        for change in (
            {"actual_exit_code": 1}, {"actual_exit_code": None},
            {"actual_exit_code": False}, {"expected_exit_code": True},
            {"truncated": True}, {"process_cause": "cancelled"},
            {"process_cause": "safety_deadline"}, {"mismatches": ["missing assertion"]},
        ):
            with self.subTest(change=change):
                self.suite["cases"][0]["variants"][0] = dict(self.variant, **change)
                with self.assertRaises(AreaResultError):
                    self.validate()

    def test_expected_exit_without_actual_execution_is_rejected(self) -> None:
        del self.variant["actual_exit_code"]
        with self.assertRaises(AreaResultError):
            self.validate()

    def test_multiple_cases_and_suites_are_counted(self) -> None:
        self.suite["cases"].append(passing_case("another"))
        self.suite["total_variants"] = 2
        second = copy.deepcopy(self.suite)
        second["name"] = "sanitizer-smoke"
        self.payload["suites"].append(second)
        self.payload["summary"]["total_variants"] = 4
        self.path.write_text(json.dumps(self.payload))
        validate_area_result(self.path, area="runtime_platform",
                             expected_suites=["platform-golden", "sanitizer-smoke"])

    def test_zero_exit_area_command_does_not_bypass_required_execution(self) -> None:
        self.variant.update(status="skip", actual_exit_code=None)

        def command_runner(command):
            result_path = self.root / command[-1]
            result_path.parent.mkdir(parents=True, exist_ok=True)
            result_path.write_text(json.dumps(self.payload))

        with patch.object(profile_area_steps, "REPO_ROOT", self.root):
            with self.assertRaisesRegex(AreaResultError, "blocking variant"):
                profile_area_steps.run_selected_area(
                    area="runtime_platform", suites=["platform-golden"],
                    profile_name="test", result_slug="runtime-platform",
                    command_builder=lambda *args: list(args),
                    command_runner=command_runner,
                )


def policy_checks() -> None:
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(AreaResultTests)
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    if not result.wasSuccessful():
        raise AssertionError("required area execution checks failed")


if __name__ == "__main__":
    policy_checks()
