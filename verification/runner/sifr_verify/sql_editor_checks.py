"""Focused editor execution must retain complete crate owners and real assertions."""
import contextlib
import copy
import importlib.util
import io
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from .area_cargo_setup import sql_preparation_commands
from .compiler_configuration_plan import configuration_plan
from .paths import REPO_ROOT
from .profiles import (DELIVERY_PROFILE_NAMES, SQL_EDITOR_PACKAGES, ProfileError,
                       load_profile, validate_editor_ownership)


def area_runner():
    spec = importlib.util.spec_from_file_location(
        "_sql_editor_checks_runner", REPO_ROOT / "verification/areas/sql_platform/runner.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def summary(passed=1, failed=0, ignored=0):
    return (f"test result: ok. {passed} passed; {failed} failed; {ignored} ignored; "
            "0 measured; 100 filtered out; finished in 0.01s\n")


class SqlEditorTests(unittest.TestCase):
    def test_all_delivery_modes_keep_complete_blocking_owners(self):
        for name in DELIVERY_PROFILE_NAMES:
            for mode in ("smoke", "full"):
                profile = load_profile(name)
                profile["toolchain_steps"] = [f"cargo-test-sifr-{mode}"]
                with self.subTest(profile=name, mode=mode):
                    validate_editor_ownership(profile)
                    commands = dict(original for group in configuration_plan(profile, mode)
                                    for original in group.originals)
                    for package in SQL_EDITOR_PACKAGES:
                        self.assertEqual(commands[package], ("test", "-p", package))

    def test_missing_filtered_disabled_and_wrong_mode_owners_are_rejected(self):
        for mode in ("smoke", "full"):
            for package in SQL_EDITOR_PACKAGES:
                for mutation in ("removed", "filter", "lib-only", "disabled", "red", "mode", "no-step"):
                    profile = copy.deepcopy(load_profile("merge"))
                    profile["toolchain_steps"] = [f"cargo-test-sifr-{mode}"]
                    suites = profile["crate_test_membership"]["suites"]
                    owner = next(s for s in suites if s["id"] == package)
                    if mutation == "removed":
                        suites.remove(owner)
                    elif mutation == "filter":
                        owner["command"].append("sql_")
                    elif mutation == "lib-only":
                        owner["command"].append("--lib")
                    elif mutation == "disabled":
                        owner["executed_in_merge"] = False
                    elif mutation == "red":
                        owner["status"] = "red-blocker"
                    elif mutation == "mode":
                        owner["modes"].remove(mode)
                    else:
                        profile["toolchain_steps"] = []
                    with self.subTest(mode=mode, package=package, mutation=mutation), \
                         self.assertRaisesRegex(ProfileError, "SQL incremental-editor requires"):
                        validate_editor_ownership(profile)

    def test_unselected_editor_does_not_require_crate_owners(self):
        for areas in ([], [{"area": "sql_platform", "suites": ["query-substrate"]}],
                      [{"area": "other", "suites": ["incremental-editor"]}]):
            validate_editor_ownership({"selected_areas": areas})

    def test_preparation_uses_exact_focused_execution_graph(self):
        command = area_runner().COMMANDS["sql-incremental-editor-tests"]
        self.assertEqual(command, ["cargo", "test", "--locked", "-p", "sifr_frontend",
                                  "-p", "sifr_analysis", "-p", "sifr_lsp", "--lib",
                                  "--", "sql_", "embedded_", "template_"])
        self.assertEqual(sql_preparation_commands(["incremental-editor"]),
                         [[*command[:2], "--no-run", *command[2:]]])

    def test_only_three_successful_nonempty_harnesses_pass(self):
        runner = area_runner()
        case = {"command": "sql-incremental-editor-tests", "expect_exit_code": 0,
                "id": "editor", "entry": "editor"}
        examples = [(summary(12) + summary(11) + summary(2), 0, "pass"),
                    (summary() * 3, 101, "fail"), ("", 0, "fail"),
                    (summary() * 2, 0, "fail"), (summary() * 4, 0, "fail"),
                    (summary() * 2 + summary(0), 0, "fail"),
                    (summary() * 2 + summary(0, ignored=1), 0, "fail"),
                    (summary() * 2 + summary(1, ignored=1), 0, "fail"),
                    (summary() * 2 + summary(1, failed=1), 0, "fail")]
        for output, returncode, status in examples:
            with self.subTest(output=output, returncode=returncode), \
                 patch.object(runner.subprocess, "run", return_value=SimpleNamespace(
                     stdout=output, returncode=returncode)), contextlib.redirect_stdout(io.StringIO()):
                variant = runner.run_case(case)["variants"][0]
                self.assertEqual(variant["status"], status)
                self.assertEqual(bool(variant["mismatches"]), status == "fail")


def policy_checks():
    result = unittest.TestResult()
    unittest.defaultTestLoader.loadTestsFromTestCase(SqlEditorTests).run(result)
    if not result.wasSuccessful():
        raise AssertionError(repr(result.failures + result.errors))
