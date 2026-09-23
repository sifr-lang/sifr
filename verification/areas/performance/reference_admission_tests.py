"""Early selected-reference admission, independent of compiler builds."""

from __future__ import annotations

import copy
import unittest
from datetime import UTC, datetime
from unittest.mock import patch

from check_trend_policy import TrendPolicyError
from reference_admission import admit_reference
from reference_profiles import ReferenceProfileError, load_profile

NAME = "linux-i7-4720hq-12gb-dev-v1"


class ReferenceAdmissionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.profile = load_profile(NAME)
        self.captured = self.profile["baseline"]["results"][0]["baseline_captured_at_unix"]
        self.identity = copy.deepcopy(self.profile["identity"])

    def admit(self, *, at: int, identity=None):
        return admit_reference(
            NAME,
            today=datetime.fromtimestamp(at, UTC).date(),
            now_unix=at,
            identity_fn=lambda *_: copy.deepcopy(self.identity if identity is None else identity),
        )

    def test_selected_reference_passes_at_freshness_boundary(self):
        # All rows in the immutable capture share this timestamp.
        self.assertEqual({row["baseline_captured_at_unix"] for row in self.profile["baseline"]["results"]}, {self.captured})
        boundary = self.captured + 45 * 86_400
        self.assertEqual(self.admit(at=boundary)["name"], NAME)
        with self.assertRaisesRegex(TrendPolicyError, "stale trend baseline"):
            self.admit(at=boundary + 1)

    def test_future_clock_skew_boundary(self):
        before = self.captured - 2 * 86_400
        self.assertEqual(self.admit(at=before)["name"], NAME)
        with self.assertRaisesRegex(TrendPolicyError, "too far in the future"):
            self.admit(at=before - 1)

    def test_unselected_or_unknown_reference_is_unavailable(self):
        with self.assertRaisesRegex(ReferenceProfileError, "select SIFR_PERFORMANCE_REFERENCE"):
            admit_reference("")
        with self.assertRaisesRegex(ReferenceProfileError, "unavailable"):
            admit_reference("unknown-reference")

    def test_host_and_toolchain_mismatches_reject_before_work(self):
        for group, key in (("host", "cpu_power_policy"), ("execution", "rustc"), ("execution", "python")):
            changed = copy.deepcopy(self.identity)
            changed[group][key] = "other"
            with self.subTest(group=group, key=key), self.assertRaisesRegex(
                ReferenceProfileError, f"{group}.{key}"
            ):
                self.admit(at=self.captured + 86_400, identity=changed)

    def test_cargo_jobs_use_selected_reference_and_restore_environment(self):
        def observe(*_):
            from os import environ
            self.assertEqual(environ["CARGO_BUILD_JOBS"], "2")
            return copy.deepcopy(self.identity)

        with patch.dict("os.environ", {"CARGO_BUILD_JOBS": "7"}):
            admit_reference(NAME, now_unix=self.captured + 86_400,
                            today=datetime.fromtimestamp(self.captured + 86_400, UTC).date(),
                            identity_fn=observe)
            from os import environ
            self.assertEqual(environ["CARGO_BUILD_JOBS"], "7")


if __name__ == "__main__":
    unittest.main()
