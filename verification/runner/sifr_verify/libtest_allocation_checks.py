"""Ordinary compiler integration resource bounds preserve selection and child ownership."""
import contextlib
import io
import sys
import unittest
from unittest.mock import patch

from .build_allocation import allocate
from .libtest_allocation import allocate_libtest, reserve_monitor
from .errors import VerificationError
from .profile_commands import CommandFailed, run_command
from .profiles import load_profile
from .resource_capacity import Capacity

GIB = 1024 ** 3
ROOM = Capacity(8, 12 * GIB)
COMMAND = ["cargo", "test", "--locked", "--offline", "-p", "sifr_driver", "--lib"]


class LibtestAllocationTests(unittest.TestCase):
    def setUp(self):
        self.build = allocate(load_profile("merge"), {}, {}, ROOM)
        capacity = patch("sifr_verify.libtest_allocation.discover_capacity", return_value=ROOM)
        self.capacity = capacity.start()
        self.addCleanup(capacity.stop)

    def test_default_and_explicit_width_share_nested_budget_without_mutating_inputs(self):
        for request, width in ((None, 4), ("1", 1), ("2", 2), ("99", 4)):
            env = {"CARGO_TARGET_DIR": "/owned", "RUSTFLAGS": "identity", "CARGO_BUILD_JOBS": "4"}
            if request is not None:
                env["RUST_TEST_THREADS"] = request
            original = dict(env)
            command = list(COMMAND)
            result, derived = allocate_libtest(self.build, env, command)
            self.assertEqual(result.workers, width)
            self.assertLessEqual(width * result.cargo_jobs, result.slots)
            self.assertLessEqual(width * result.rayon_threads, result.slots)
            self.assertEqual(derived, dict(env, RUST_TEST_THREADS=str(width),
                CARGO_BUILD_JOBS=str(result.cargo_jobs), RAYON_NUM_THREADS=str(result.rayon_threads),
                SIFR_E2E_CARGO_BUILD_JOBS="1"))
            self.assertEqual(env, original)
            self.assertEqual(command, COMMAND)

    def test_standalone_native_cargo_jobs_share_the_outer_worker_budget(self):
        for width in (1, 2, 4):
            for requested in (1, 2, 4, 99):
                env = {"RUST_TEST_THREADS": str(width), "SIFR_E2E_CARGO_BUILD_JOBS": str(requested)}
                allocation, actual = allocate_libtest(self.build, env, COMMAND)
                self.assertEqual(int(actual["SIFR_E2E_CARGO_BUILD_JOBS"]), min(requested, 4 // width))
                self.assertLessEqual(width * int(actual["SIFR_E2E_CARGO_BUILD_JOBS"]), allocation.slots)
                self.assertEqual(env["SIFR_E2E_CARGO_BUILD_JOBS"], str(requested))
        with self.assertRaises(VerificationError):
            allocate_libtest(self.build, {"SIFR_E2E_CARGO_BUILD_JOBS": "0"}, COMMAND)

    def test_fresh_capacity_reduces_workers_and_refuses_insufficient_memory(self):
        self.capacity.return_value = Capacity(8, 6 * GIB)
        result, _ = allocate_libtest(self.build, {}, COMMAND)
        self.assertEqual((result.workers, result.cargo_jobs), (2, 1))
        self.capacity.return_value = Capacity(2, 12 * GIB)
        result, _ = allocate_libtest(self.build, {}, COMMAND)
        self.assertEqual(result.workers, 1)
        self.capacity.return_value = Capacity(8, 3 * GIB)
        with self.assertRaises(CommandFailed) as caught:
            allocate_libtest(self.build, {}, COMMAND)
        self.assertEqual(caught.exception.cause, "libtest_capacity")

    def test_harness_override_is_honored_only_inside_budget(self):
        for flags in (["--test-threads", "2"], ["--test-threads=2"]):
            result, _ = allocate_libtest(self.build, {"RUST_TEST_THREADS": "1"}, [*COMMAND, "--", *flags])
            self.assertEqual(result.workers, 2)
        with self.assertRaises(CommandFailed):
            allocate_libtest(self.build, {}, [*COMMAND, "--", "--test-threads=9"])
        for flags in (["--test-threads"], ["--test-threads=0"], ["--test-threads=-1"]):
            with self.assertRaises(VerificationError):
                allocate_libtest(self.build, {}, [*COMMAND, "--", *flags])
        for value in ("0", "-1", "", " 2", "auto", "٢"):
            with self.assertRaises(VerificationError):
                allocate_libtest(self.build, {"RUST_TEST_THREADS": value}, COMMAND)

    def test_reserve_loss_is_terminal_and_command_invokes_monitor(self):
        allocation, _ = allocate_libtest(self.build, {}, COMMAND)
        monitor = reserve_monitor(allocation)
        self.capacity.return_value = Capacity(8, GIB)
        with contextlib.redirect_stdout(io.StringIO()), self.assertRaises(CommandFailed) as caught:
            run_command([sys.executable, "-c", "import time; time.sleep(30)"], monitor=monitor)
        self.assertEqual(caught.exception.cause, "libtest_capacity")

    def test_unreadable_capacity_is_terminal(self):
        self.capacity.side_effect = VerificationError("capacity unavailable")
        with self.assertRaises(CommandFailed) as caught:
            allocate_libtest(self.build, {}, COMMAND)
        self.assertEqual(caught.exception.cause, "libtest_capacity")


def policy_checks():
    result = unittest.TestResult()
    unittest.defaultTestLoader.loadTestsFromTestCase(LibtestAllocationTests).run(result)
    if not result.wasSuccessful():
        raise AssertionError(repr(result.failures + result.errors))
