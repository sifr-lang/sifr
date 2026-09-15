"""Check graph selection and execution boundaries of crate preparation."""

import copy
import unittest

from .cargo_crate_setup import prepare_crate_test_binaries
from .profile_commands import CommandFailed


class CrateSetupPolicyTests(unittest.TestCase):
    def selected(self, *suites, mode="smoke"):
        return {"toolchain_steps": [f"cargo-test-sifr-{mode}"],
                "crate_test_membership": {"suites": list(suites)}}

    def suite(self, command, modes=("smoke", "full"), status="blocking", executed=True):
        return {"command": command, "modes": list(modes), "status": status,
                "executed_in_merge": executed}

    def test_only_selected_executed_graphs_are_built(self):
        calls = []
        run = lambda args, **kw: calls.append(args)
        prepare_crate_test_binaries({"toolchain_steps": []}, {}, run)
        payload = self.selected(
            self.suite(["test", "-p", "full-only"], modes=("full",)),
            self.suite(["test", "-p", "planned"], status="red-blocker", executed=False),
            self.suite(["test", "-p", "required"], status="red-blocker"),
        )
        prepare_crate_test_binaries(payload, {}, run)
        self.assertEqual(calls, [
            ["cargo", "test", "--locked", "--offline", "--no-run", "-p", "required"]])

    def test_feature_graphs_stay_separate_and_test_filters_are_not_forwarded(self):
        payload = self.selected(
            self.suite(["test", "-p", "runtime", "--", "--skip", "slow"]),
            self.suite(["test", "-p", "runtime", "--no-default-features",
                        "--features", "json,unicode", "--lib"]),
        )
        before = copy.deepcopy(payload)
        env = {"CARGO_BUILD_JOBS": "2", "CARGO_INCREMENTAL": "0"}
        calls = []
        prepare_crate_test_binaries(payload, env,
                                    lambda args, **kw: calls.append((args, kw["env"])))
        prefix = ["cargo", "test", "--locked", "--offline", "--no-run"]
        self.assertEqual(calls, [
            (prefix + ["-p", "runtime"], env),
            (prefix + ["-p", "runtime", "--no-default-features",
                       "--features", "json,unicode", "--lib"], env),
        ])
        self.assertEqual(payload, before)

    def test_full_mode_builds_its_own_selection(self):
        calls = []
        payload = self.selected(
            self.suite(["test", "-p", "smoke-only"], modes=("smoke",)),
            self.suite(["test", "-p", "full-only"], modes=("full",)), mode="full")
        prepare_crate_test_binaries(payload, {}, lambda args, **kw: calls.append(args))
        self.assertEqual(calls[0][-2:], ["-p", "full-only"])
        self.assertEqual(len(calls), 1)

    def test_build_failure_stops_before_later_packages(self):
        calls = []
        def fail(args, **kw):
            calls.append(args)
            raise CommandFailed(101)
        payload = self.selected(self.suite(["test", "-p", "first"]),
                                self.suite(["test", "-p", "second"]))
        with self.assertRaises(CommandFailed):
            prepare_crate_test_binaries(payload, {}, fail)
        self.assertEqual(len(calls), 1)
