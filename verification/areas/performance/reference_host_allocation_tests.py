"""Admission regressions for explicit finite managed-host allocations."""

import copy
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from reference_host import cpu_power_policy, comparison_mismatches
from reference_host_allocation import managed_allocation


class ManagedAllocationTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.membership = self.root / "membership"
        self.membership.write_text("0::/\n")
        (self.root / "cpu.max").write_text("400000 100000\n")
        (self.root / "memory.max").write_text("17179869184\n")

    def allocation(self):
        return managed_allocation(self.root, self.membership)

    def test_finite_allocation_is_measured(self):
        control = self.allocation()["controls"][0]
        self.assertEqual(control["cpu_quota_us"], 400000)
        self.assertEqual(control["cpu_period_us"], 100000)
        self.assertEqual(control["memory_max_bytes"], 17179869184)

    def test_missing_limits_or_membership_fail(self):
        for filename in ("cpu.max", "memory.max", "membership"):
            with self.subTest(filename=filename):
                path = self.root / filename
                content = path.read_text()
                path.unlink()
                with self.assertRaisesRegex(ValueError, "unavailable"):
                    self.allocation()
                path.write_text(content)

    def test_unlimited_zero_negative_and_malformed_limits_fail(self):
        for filename, invalid in (
            ("cpu.max", "max 100000"), ("cpu.max", "0 100000"),
            ("cpu.max", "-1 100000"), ("cpu.max", "400000 0"),
            ("cpu.max", "400000 nope"), ("cpu.max", "400000"),
            ("memory.max", "max"), ("memory.max", "0"),
            ("memory.max", "-1"), ("memory.max", "16GB"),
        ):
            with self.subTest(filename=filename, invalid=invalid):
                path = self.root / filename
                original = path.read_text()
                path.write_text(invalid)
                with self.assertRaises(ValueError):
                    self.allocation()
                path.write_text(original)

    def test_nested_membership_binds_parent_constraints(self):
        child = self.root / "worker"
        child.mkdir()
        (child / "cpu.max").write_text("800000 100000")
        (child / "memory.max").write_text("34359738368")
        self.membership.write_text("0::/worker\n")
        measured = self.allocation()
        self.assertEqual([row["path"] for row in measured["controls"]], ["worker", "."])
        self.assertEqual(measured["controls"][1]["cpu_quota_us"], 400000)

    def test_invalid_or_escaping_membership_fails(self):
        for value in ("1:cpu:/", "0::relative", "0::/../escape", "0::/\n0::/"):
            with self.subTest(value=value):
                self.membership.write_text(value)
                with self.assertRaises(ValueError):
                    self.allocation()

    def test_physical_default_requires_frequency_exposure(self):
        with patch.dict("os.environ", {}, clear=True), patch(
            "reference_host.platform.system", return_value="Linux"
        ), patch("reference_host.Path.glob", return_value=[]):
            with self.assertRaisesRegex(ValueError, "frequency policy"):
                cpu_power_policy()

    def test_unknown_mode_and_nonlinux_managed_mode_fail(self):
        for kind, system in (("automatic", "Linux"), ("managed-linux", "Darwin")):
            with self.subTest(kind=kind), patch.dict("os.environ", {
                "SIFR_PERFORMANCE_HOST_KIND": kind
            }), patch("reference_host.platform.system", return_value=system):
                with self.assertRaises(ValueError):
                    cpu_power_policy()

    def test_managed_missing_frequency_is_honest_and_binds_limits(self):
        allocation = self.allocation()
        with patch.dict("os.environ", {"SIFR_PERFORMANCE_HOST_KIND": "managed-linux"}), patch(
            "reference_host.platform.system", return_value="Linux"
        ), patch("reference_host.Path.glob", return_value=[]), patch(
            "reference_host.managed_allocation", return_value=allocation
        ):
            measured = cpu_power_policy()
        self.assertEqual(measured["frequency_policy"], {"source": "unavailable"})
        self.assertEqual(measured["allocation"], allocation)

    def test_managed_visible_frequency_is_recorded(self):
        policy = self.root / "policy0"
        policy.mkdir()
        for field in ("affected_cpus", "scaling_driver", "scaling_governor",
                      "scaling_min_freq", "scaling_max_freq"):
            (policy / field).write_text(f"measured-{field}")
        with patch.dict("os.environ", {"SIFR_PERFORMANCE_HOST_KIND": "managed-linux"}), patch(
            "reference_host.platform.system", return_value="Linux"
        ), patch("reference_host.Path.glob", return_value=[policy]), patch(
            "reference_host.managed_allocation", return_value=self.allocation()
        ):
            measured = cpu_power_policy()
        self.assertEqual(measured["frequency_policy"]["policies"][0]["scaling_governor"],
                         "measured-scaling_governor")

    def test_changed_allocation_is_not_comparable(self):
        from reference_profile_tests import identity
        before = identity()
        before["host"]["cpu_power_policy"] = {"source": "managed-linux",
                                               "allocation": self.allocation()}
        before["execution"]["measurement_timer"] = {"path": "/measured/time", "sha256": "a" * 64}
        for field in ("cpu_quota_us", "cpu_period_us", "memory_max_bytes"):
            after = copy.deepcopy(before)
            after["host"]["cpu_power_policy"]["allocation"]["controls"][0][field] += 1
            with self.subTest(field=field):
                self.assertEqual(comparison_mismatches(before, after), ["host.cpu_power_policy"])

    def test_managed_affinity_records_exact_cpus_even_at_equal_cardinality(self):
        with patch.dict("os.environ", {"SIFR_PERFORMANCE_HOST_KIND": "managed-linux"}), patch(
            "reference_host.platform.system", return_value="Linux"
        ), patch("reference_host.Path.glob", return_value=[]), patch(
            "reference_host.managed_allocation", return_value=self.allocation()
        ), patch("reference_host.os.sched_getaffinity", side_effect=[{0, 1}, {1, 2}]):
            before = cpu_power_policy()
            after = cpu_power_policy()
        self.assertEqual(before["affinity_cpus"], [0, 1])
        self.assertEqual(after["affinity_cpus"], [1, 2])
        self.assertNotEqual(before, after)
