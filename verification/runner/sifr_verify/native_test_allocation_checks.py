"""Native allocation admission and reserve regression controls."""
from dataclasses import asdict, replace
import json
import unittest
from unittest.mock import patch

from .build_allocation import allocate
from .native_test_allocation import (ALLOCATION_ENV, allocate_native, native_environment,
                                     readmit, reserve_monitor)
from .errors import VerificationError
from .profile_commands import CommandFailed
from .profiles import load_profile
from .resource_capacity import Capacity

GIB = 1024**3


class NativeAllocationTests(unittest.TestCase):
    def build(self, memory=12, cpus=8):
        return allocate(load_profile("merge"), {}, {}, Capacity(cpus, memory * GIB))

    def test_outer_and_inner_share_slots_and_memory(self):
        build = self.build()
        result = allocate_native(build)
        self.assertEqual((result.workers, result.cargo_jobs), (2, 2))
        self.assertLessEqual(result.workers * max(result.cargo_jobs, result.rayon_threads), build.slot_limit)
        self.assertGreaterEqual(result.memory_per_worker_mib, 4096)
        self.assertLessEqual(result.workers * result.memory_per_worker_mib + result.reserved_memory_mib,
                             build.available_memory_mib)
        self.assertEqual(result.reserved_memory_mib, build.policy["reserved_memory_mib"])
        self.assertEqual(build.compiler_jobs, 4)

    def test_single_slot_and_low_memory_reduce_width_not_silently_overcommit(self):
        one = allocate_native(replace(self.build(), slot_limit=1))
        self.assertEqual((one.workers, one.cargo_jobs, one.rayon_threads), (1, 1, 1))
        low = allocate_native(replace(self.build(), available_memory_mib=6*1024, compiler_jobs=2))
        self.assertEqual(low.workers, 1)
        zero = allocate_native(replace(self.build(), available_memory_mib=5*1024))
        self.assertEqual(zero.workers, 0)
        with self.assertRaises(VerificationError):
            native_environment({ALLOCATION_ENV: json.dumps(asdict(zero))})

    def test_native_preparation_environment_is_copy_with_only_resolved_settings(self):
        allocation = allocate_native(self.build())
        env = {ALLOCATION_ENV: json.dumps(asdict(allocation)), "CARGO_BUILD_JOBS": "4",
               "RAYON_NUM_THREADS": "4", "CARGO_NET_OFFLINE": "true", "RUSTFLAGS": "bound"}
        actual, derived = native_environment(env)
        self.assertEqual(actual, allocation)
        self.assertEqual(derived, dict(env, CARGO_BUILD_JOBS="2", RAYON_NUM_THREADS=str(allocation.rayon_threads)))
        self.assertEqual(env["CARGO_BUILD_JOBS"], "4")
        for invalid in ({}, {ALLOCATION_ENV: "null"}, {ALLOCATION_ENV: "{}"},
                        {ALLOCATION_ENV: json.dumps(asdict(replace(allocation, workers=3)))}):
            with self.subTest(invalid=invalid), self.assertRaises(VerificationError):
                native_environment(invalid)

    def test_readmission_reduces_workers_preserving_inner_jobs_and_rejects_no_capacity(self):
        allocation = allocate_native(self.build())
        with patch("sifr_verify.native_test_allocation.discover_capacity", return_value=Capacity(8, 6*GIB)):
            self.assertEqual(readmit(allocation), 1)
        with patch("sifr_verify.native_test_allocation.discover_capacity", return_value=Capacity(1, 12*GIB)):
            with self.assertRaises(CommandFailed):
                readmit(allocation)
        with patch("sifr_verify.native_test_allocation.discover_capacity", return_value=Capacity(8, 5*GIB)):
            with self.assertRaises(CommandFailed):
                readmit(allocation)

    def test_live_reserve_loss_is_terminal(self):
        allocation = allocate_native(self.build())
        with patch("sifr_verify.native_test_allocation.discover_capacity", return_value=Capacity(8, GIB)):
            with self.assertRaises(CommandFailed) as caught:
                reserve_monitor(allocation)()
        self.assertEqual(caught.exception.cause, "native_capacity")


def policy_checks():
    result = unittest.TestResult()
    unittest.defaultTestLoader.loadTestsFromTestCase(NativeAllocationTests).run(result)
    if not result.wasSuccessful():
        raise AssertionError(repr(result.failures + result.errors))
