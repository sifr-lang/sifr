"""Regression checks for the governed runtime sanitizer command boundary."""

from __future__ import annotations

import contextlib
import copy
import importlib.util
import io
import os
import sys
import time
import tempfile
import unittest
from pathlib import Path
from unittest.mock import Mock, patch

from .paths import REPO_ROOT
from .process_execution import Outcome


def load_runtime_adapter():
    path = REPO_ROOT / "verification" / "areas" / "runtime_platform" / "runner.py"
    spec = importlib.util.spec_from_file_location("runtime_sanitizer_test_adapter", path)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class SanitizerTargetTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.adapter = load_runtime_adapter()
        cls.cases = {case["id"]: case for case in cls.adapter.load_sanitizer_manifest()["cases"]}

    def run_case(self, case, host, *, outcome=None, skip=(), environment=None):
        if outcome is None:
            outcome = Outcome(0, "exit", b"", b"", False, 0.01)
        with patch.object(self.adapter, "sanitizer_skip_reasons", return_value=list(skip)), \
                patch.object(self.adapter, "execute", side_effect=outcome if isinstance(outcome, Exception) else None,
                             return_value=outcome) as execute, \
                patch.dict(os.environ, {"CARGO_HOME": "/owned/cache"} | (environment or {}), clear=True), \
                contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            result = self.adapter.run_sanitizer_case("sanitizer-smoke", case, host)
        return result, execute

    def test_every_instrumented_case_keeps_flags_selection_and_timeout(self):
        instrumented = [case for case in self.cases.values() if case["tool"] in {"asan", "lsan", "tsan"}]
        self.assertEqual(len(instrumented), 5)
        for case in instrumented:
            for host in case["supported_host_triples"]:
                with self.subTest(case=case["id"], host=host):
                    original = copy.deepcopy(case)
                    result, execute = self.run_case(case, host)
                    execute.assert_called_once()
                    args, kwargs = execute.call_args
                    command = args[0]
                    boundary = original["command"].index("--") if "--" in original["command"] else len(original["command"])
                    self.assertEqual(command[boundary:boundary + 2], ["--target", host])
                    self.assertEqual(command[:boundary] + command[boundary + 2:], original["command"])
                    self.assertEqual(kwargs["env"]["RUSTFLAGS"], original["env"]["RUSTFLAGS"])
                    self.assertEqual(kwargs["env"]["CARGO_HOME"], "/owned/cache")
                    self.assertEqual(kwargs["env"]["CARGO_NET_OFFLINE"], "true")
                    self.assertNotIn("CARGO_ENCODED_RUSTFLAGS", kwargs["env"])
                    if original["scope"] == "generated_binary":
                        self.assertEqual(kwargs["env"]["CARGO_BUILD_TARGET"], host)
                    else:
                        self.assertNotIn("CARGO_BUILD_TARGET", kwargs["env"])
                    self.assertNotIn("CARGO_TARGET_DIR", kwargs["env"])
                    self.assertEqual(kwargs["deadline_seconds"], original["timeout_seconds"])
                    self.assertEqual(kwargs["cwd"], REPO_ROOT)
                    self.assertEqual(result["argv"], command)
                    self.assertEqual(result["status"], "pass")
                    self.assertEqual(case, original)

    def test_generated_program_arguments_stay_after_cargo_separator(self):
        case = self.cases["generated-binary-asan-smoke"]
        result, _ = self.run_case(case, "aarch64-apple-darwin")
        command = result["argv"]
        self.assertEqual(command[command.index("--") - 2:],
                         ["--target", "aarch64-apple-darwin", "--", "run", "crates/sifr/tests/e2e/pass/break_continue.sifr"])

    def test_generated_source_is_declared_and_has_runtime_assertions(self):
        case = self.cases["generated-binary-asan-smoke"]
        self.assertEqual(case["required_files"], [case["command"][-1]])
        self.assertIn("assert ", (REPO_ROOT / case["required_files"][0]).read_text())

    def test_missing_source_fails_before_tool_probes_or_build(self):
        case = copy.deepcopy(self.cases["generated-binary-asan-smoke"])
        case["command"][-1] = "demos/absent-sanitizer-fixture.sifr"
        case["required_files"] = [case["command"][-1]]
        with patch.object(self.adapter, "sanitizer_skip_reasons") as probes, \
                patch.object(self.adapter, "execute") as execute:
            with self.assertRaisesRegex(SystemExit, "required input is missing"):
                self.adapter.run_sanitizer_case("sanitizer-smoke", case, "x86_64-unknown-linux-gnu")
            probes.assert_not_called()
            execute.assert_not_called()

    def test_stale_command_cannot_bypass_preflight_with_another_input(self):
        case = copy.deepcopy(self.cases["generated-binary-asan-smoke"])
        case["command"][-1] = "demos/absent-sanitizer-fixture.sifr"
        with self.assertRaisesRegex(SystemExit, "undeclared source input"):
            self.adapter.validate_sanitizer_inputs(case)

    def test_input_list_rejects_invalid_schema(self):
        for value in (None, "fixture.sifr", [1], [""]):
            with self.subTest(value=value):
                case = copy.deepcopy(self.cases["generated-binary-asan-smoke"])
                case["required_files"] = value
                with self.assertRaisesRegex(SystemExit, "required_files must be a string list"):
                    self.adapter.validate_sanitizer_inputs(case)

    def test_required_inputs_stay_inside_repository_including_symlinks(self):
        case = copy.deepcopy(self.cases["runtime-asan-smoke"])
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "repo"
            root.mkdir()
            outside = Path(directory) / "outside"
            outside.write_text("fixture")
            (root / "escape").symlink_to(outside)
            with patch.object(self.adapter, "REPO_ROOT", root):
                for name in ("../outside", str(outside), "escape"):
                    with self.subTest(name=name):
                        case["required_files"] = [name]
                        with self.assertRaisesRegex(SystemExit, "outside the repository"):
                            self.adapter.validate_sanitizer_inputs(case)

    def test_caller_flags_and_storage_survive_generated_target_selection(self):
        case = self.cases["generated-binary-asan-smoke"]
        inherited = {
            'RUSTFLAGS': '  --cfg v013_probe="value"  -C debuginfo=1  ',
            "CARGO_TARGET_DIR": "/owned/build",
            "CARGO_BUILD_TARGET": "other-target",
            "CARGO_BUILD_JOBS": "2",
        }
        _, execute = self.run_case(case, "x86_64-unknown-linux-gnu", environment=inherited)
        env = execute.call_args.kwargs["env"]
        self.assertNotIn("CARGO_ENCODED_RUSTFLAGS", env)
        self.assertEqual(env["RUSTFLAGS"], inherited["RUSTFLAGS"] + " -Zsanitizer=address")
        self.assertEqual(env["CARGO_TARGET_DIR"], "/owned/build")
        self.assertEqual(env["CARGO_BUILD_JOBS"], "2")
        self.assertEqual(env["CARGO_BUILD_TARGET"], "x86_64-unknown-linux-gnu")
        self.assertEqual(inherited["CARGO_BUILD_TARGET"], "other-target")


    def test_plain_quotes_apostrophes_and_embedded_whitespace_match_cargo(self):
        case = self.cases["runtime-asan-smoke"]
        for flags in ("-Clink-arg=it's", '--cfg\tv013_probe="value"',
                      '--cfg\nv013_probe="value"'):
            with self.subTest(flags=flags):
                _, execute = self.run_case(case, "x86_64-unknown-linux-gnu",
                                           environment={"RUSTFLAGS": flags})
                env = execute.call_args.kwargs["env"]
                self.assertNotIn("CARGO_ENCODED_RUSTFLAGS", env)
                self.assertEqual(env["RUSTFLAGS"], flags + " -Zsanitizer=address")


    def test_plain_sanitizer_boundary_whitespace_matches_rust_trim(self):
        case = self.cases["runtime-asan-smoke"]
        for whitespace in self.adapter.RUST_FLAG_WHITESPACE:
            raw = whitespace + "-Zsanitizer=address" + whitespace
            with self.subTest(whitespace=repr(whitespace)):
                _, execute = self.run_case(case, "x86_64-unknown-linux-gnu",
                                           environment={"RUSTFLAGS": raw})
                env = execute.call_args.kwargs["env"]
                self.assertEqual(env["RUSTFLAGS"], raw + " -Zsanitizer=address")
                self.assertNotIn("CARGO_ENCODED_RUSTFLAGS", env)

    def test_plain_control_separators_remain_inside_the_argument(self):
        case = self.cases["runtime-asan-smoke"]
        for separator in ("\x1c", "\x1d", "\x1e", "\x1f"):
            with self.subTest(separator=repr(separator)):
                raw = '--cfg v013_probe="a' + separator + 'b"'
                _, execute = self.run_case(case, "x86_64-unknown-linux-gnu",
                                           environment={"RUSTFLAGS": raw})
                env = execute.call_args.kwargs["env"]
                self.assertEqual(env["RUSTFLAGS"], raw + " -Zsanitizer=address")
                self.assertNotIn("CARGO_ENCODED_RUSTFLAGS", env)
                edge = separator + "-Cdebuginfo=1" + separator
                self.assertEqual(self.adapter.cargo_plain_rustflags(edge), [edge])

    def test_encoded_sanitizer_boundary_whitespace_is_not_trimmed(self):
        case = self.cases["runtime-asan-smoke"]
        with self.assertRaisesRegex(SystemExit, "conflicts with inherited"):
            self.run_case(case, "x86_64-unknown-linux-gnu",
                          environment={"CARGO_ENCODED_RUSTFLAGS": "-Zsanitizer=address\n"})

    def test_declared_encoded_separator_cannot_change_argument_boundaries(self):
        case = copy.deepcopy(self.cases["runtime-asan-smoke"])
        case["env"]["RUSTFLAGS"] += ' --cfg value="a\x1fb"'
        with self.assertRaisesRegex(SystemExit, "unrepresentable declared encoded flag"):
            self.run_case(case, "x86_64-unknown-linux-gnu",
                          environment={"CARGO_ENCODED_RUSTFLAGS": "-Cdebuginfo=1"})

    def test_encoded_arguments_preserve_spaces_without_shell_quoting(self):
        case = self.cases["runtime-asan-smoke"]
        _, execute = self.run_case(case, "x86_64-unknown-linux-gnu", environment={
            "CARGO_ENCODED_RUSTFLAGS": "-C" + chr(31) + "link-arg=/owned/path with spaces",
        })
        self.assertEqual(execute.call_args.kwargs["env"]["CARGO_ENCODED_RUSTFLAGS"].split(chr(31)),
                         ["-C", "link-arg=/owned/path with spaces", "-Zsanitizer=address"])

    def test_encoded_flags_precede_plain_flags_without_losing_instrumentation(self):
        case = self.cases["runtime-asan-smoke"]
        for encoded, expected in (("-C\x1fdebuginfo=1", ["-C", "debuginfo=1"]), ("", [])):
            with self.subTest(encoded=encoded):
                _, execute = self.run_case(case, "x86_64-unknown-linux-gnu", environment={
                    "RUSTFLAGS": "-Copt-level=3", "CARGO_ENCODED_RUSTFLAGS": encoded,
                })
                self.assertEqual(execute.call_args.kwargs["env"]["CARGO_ENCODED_RUSTFLAGS"].split("\x1f"),
                                 [*expected, "-Zsanitizer=address"])

    def test_encoded_empty_arguments_are_not_silently_removed(self):
        case = self.cases["runtime-asan-smoke"]
        _, execute = self.run_case(case, "x86_64-unknown-linux-gnu", environment={
            "CARGO_ENCODED_RUSTFLAGS": "-C" + chr(31) + chr(31) + "debuginfo=1",
        })
        self.assertEqual(execute.call_args.kwargs["env"]["CARGO_ENCODED_RUSTFLAGS"].split(chr(31)),
                         ["-C", "", "debuginfo=1", "-Zsanitizer=address"])

    def test_conflicting_inherited_instrumentation_is_rejected_before_execution(self):
        case = self.cases["generated-binary-asan-smoke"]
        for environment in (
            {"RUSTFLAGS": "-Zsanitizer=thread"},
            {"RUSTFLAGS": "-Z sanitizer=leak"},
            {"CARGO_ENCODED_RUSTFLAGS": "-Z\x1fsanitizer=thread"},
        ):
            with self.subTest(environment=environment), \
                    patch.dict(os.environ, environment, clear=True), \
                    patch.object(self.adapter, "sanitizer_skip_reasons", return_value=[]), \
                    patch.object(self.adapter, "execute") as execute:
                with self.assertRaisesRegex(SystemExit, "conflicts with inherited"):
                    self.adapter.run_sanitizer_case("sanitizer-smoke", case, "x86_64-unknown-linux-gnu")
                execute.assert_not_called()

    def test_declared_instrumentation_must_match_tool(self):
        for flags in ("", "-Cdebuginfo=1", "-Zsanitizer=thread",
                      "-Zsanitizer=address -Zsanitizer=thread"):
            with self.subTest(flags=flags):
                case = copy.deepcopy(self.cases["runtime-asan-smoke"])
                case["env"]["RUSTFLAGS"] = flags
                with self.assertRaisesRegex(SystemExit, "must declare address"):
                    self.adapter.sanitizer_environment(case, "x86_64-unknown-linux-gnu")

    def test_miri_keeps_its_declared_command(self):
        case = self.cases["runtime-miri-full"]
        result, execute = self.run_case(case, "aarch64-apple-darwin")
        self.assertEqual(execute.call_args.args[0], case["command"])
        self.assertEqual(result["argv"], case["command"])

    def test_existing_skip_does_not_launch_or_rewrite_command(self):
        case = self.cases["deterministic-concurrency-model-full"]
        result, execute = self.run_case(case, "aarch64-apple-darwin", skip=[case["skip_reason"]])
        execute.assert_not_called()
        self.assertEqual(result["status"], "skip")
        self.assertEqual(result["argv"], case["command"])
        self.assertEqual(result["skip_reason"], case["skip_reason"])

    def test_nonzero_result_remains_a_blocking_failure(self):
        case = self.cases["runtime-asan-smoke"]
        result, _ = self.run_case(case, "aarch64-apple-darwin",
                                  outcome=Outcome(101, "exit", b"", b"asan finding", False, 0.01))
        self.assertEqual(result["status"], "fail")
        self.assertEqual(result["mismatches"], ["exit=101 expected=0"])

    def test_timeout_remains_a_failure_with_original_limit(self):
        case = self.cases["runtime-asan-smoke"]
        result, execute = self.run_case(case, "aarch64-apple-darwin",
                                         outcome=Outcome(124, "safety_deadline", b"", b"", False, 120))
        self.assertEqual(execute.call_args.kwargs["deadline_seconds"], 120)
        self.assertEqual(result["status"], "fail")
        self.assertEqual(result["mismatches"], ["timeout after 120s"])


    def test_truncated_or_cancelled_output_cannot_pass(self):
        case = self.cases["runtime-asan-smoke"]
        for outcome, reason in [
            (Outcome(0, "exit", b"", b"", True, 0.01), "output-truncated"),
            (Outcome(130, "cancelled", b"", b"", False, 0.01), "process=cancelled"),
        ]:
            with self.subTest(reason=reason):
                result, _ = self.run_case(case, "x86_64-unknown-linux-gnu", outcome=outcome)
                self.assertEqual(result["status"], "fail")
                self.assertIn(reason, result["mismatches"])
                self.assertEqual(result["actual_exit_code"], outcome.returncode)
                self.assertEqual(result["process_cause"], outcome.cause)
                self.assertEqual(result["truncated"], outcome.truncated)

    @unittest.skipUnless(hasattr(os, "fork"), "requires POSIX child process groups")
    def test_actual_sanitizer_timeout_terminates_its_descendant(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            ready, escaped = root / "ready", root / "escaped"
            program = (
                "import os,time\nfrom pathlib import Path\n"
                "if os.fork() == 0:\n"
                f"    Path({str(ready)!r}).write_text('ready')\n"
                "    time.sleep(2)\n"
                f"    Path({str(escaped)!r}).write_text('escaped')\n"
                "else:\n    time.sleep(10)\n"
            )
            case = copy.deepcopy(self.cases["runtime-asan-smoke"])
            case.update(tool="process-control", command=[sys.executable, "-c", program],
                        env={}, timeout_seconds=1)
            with patch.object(self.adapter, "sanitizer_skip_reasons", return_value=[]), \
                    contextlib.redirect_stdout(io.StringIO()), \
                    contextlib.redirect_stderr(io.StringIO()):
                result = self.adapter.run_sanitizer_case("sanitizer-smoke", case,
                                                        "x86_64-unknown-linux-gnu")
            self.assertTrue(ready.exists(), "descendant must actually execute")
            self.assertEqual(result["status"], "fail")
            self.assertEqual(result["process_cause"], "safety_deadline")
            self.assertEqual(result["mismatches"], ["timeout after 1s"])
            time.sleep(1.2)
            self.assertFalse(escaped.exists(), "descendant survived the owned timeout")


class SanitizerPreparationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.adapter = load_runtime_adapter()

    def prepare(self, suites, *, unavailable=(), run=None):
        from .runtime_sanitizer_preparation import prepare
        run = Mock() if run is None else run
        environment = {"CARGO_BUILD_JOBS": "2", "CARGO_TARGET_DIR": "/owned/sanitizers",
                       "CARGO_HOME": "/owned/cargo", "RUSTFLAGS": "-Cdebuginfo=1"}
        with patch.object(self.adapter, "current_rust_host_triple", return_value="x86_64-unknown-linux-gnu"), \
                patch.object(self.adapter, "sanitizer_skip_reasons", return_value=list(unavailable)), \
                patch.dict(os.environ, environment, clear=True), \
                contextlib.redirect_stdout(io.StringIO()):
            prepare(set(suites), run=run, adapter=self.adapter)
        return run

    def test_smoke_prepares_both_exact_graphs_without_assertions(self):
        run = self.prepare(["sanitizer-smoke"])
        self.assertEqual([call.args[0] for call in run.call_args_list], [
            ["cargo", "+nightly", "test", "--no-run", "--locked", "-p", "sifr_runtime",
             "int::tests::parse_enforces_digit_limit_without_panicking", "--target", "x86_64-unknown-linux-gnu"],
            ["cargo", "+nightly", "build", "--locked", "-q", "-p", "sifr", "--target", "x86_64-unknown-linux-gnu"],
        ])
        for call in run.call_args_list:
            env = call.kwargs["env"]
            self.assertEqual(env["CARGO_BUILD_JOBS"], "2")
            self.assertEqual(env["CARGO_TARGET_DIR"], "/owned/sanitizers")
            self.assertEqual(env["CARGO_HOME"], "/owned/cargo")
            self.assertEqual(env["RUSTFLAGS"], "-Cdebuginfo=1 -Zsanitizer=address")
            self.assertEqual(env["CARGO_NET_OFFLINE"], "true")
        self.assertEqual(run.call_args_list[1].kwargs["env"]["CARGO_BUILD_TARGET"],
                         "x86_64-unknown-linux-gnu")

    def test_overlapping_suite_selection_prepares_each_case_once(self):
        one = self.prepare(["sanitizer-full"])
        both = self.prepare(["sanitizer-full", "sanitizer-smoke"])
        self.assertEqual(one.call_count, 5)
        self.assertEqual(one.call_args_list, both.call_args_list)

    def test_unavailable_graphs_do_not_fake_build_or_assertion_execution(self):
        self.prepare(["sanitizer-smoke"], unavailable=["missing nightly"]).assert_not_called()

    def test_preparation_failure_propagates_before_later_graphs(self):
        run = Mock(side_effect=RuntimeError("instrumented build failed"))
        with self.assertRaisesRegex(RuntimeError, "instrumented build failed"):
            self.prepare(["sanitizer-smoke"], run=run)
        self.assertEqual(run.call_count, 1)

    def test_unknown_selection_and_actions_fail_closed(self):
        from .runtime_sanitizer_preparation import preparation_command, prepare
        for suites in (set(), {"golden"}):
            with self.subTest(suites=suites), self.assertRaises(ValueError):
                prepare(suites)
        for command in ([], ["rustc"], ["cargo"], ["cargo", "+nightly"],
                        ["cargo", "+nightly", "check"]):
            with self.subTest(command=command), self.assertRaises(ValueError):
                preparation_command(command)

    def test_feature_profile_target_flags_stay_in_prepared_selection(self):
        from .runtime_sanitizer_preparation import preparation_command
        selection = ["--locked", "--no-default-features", "--features", "http",
                     "--profile", "custom", "--target", "host", "--bin", "program"]
        source = ["cargo", "+pinned", "run", *selection, "--", "argument"]
        self.assertEqual(preparation_command(source), ["cargo", "+pinned", "build", *selection])
        self.assertEqual(source[-2:], ["--", "argument"])
        source = ["cargo", "+pinned", "test", *selection, "filter", "--", "--ignored"]
        self.assertEqual(preparation_command(source), ["cargo", "+pinned", "test", "--no-run",
                                                      *selection, "filter"])

    def test_only_selected_sanitizer_consumer_receives_preparation_edge(self):
        from .cargo_setup import cargo_preparation_plan
        from .profiles import load_all_profiles
        for profile in load_all_profiles().values():
            before = copy.deepcopy(profile)
            nodes, consumers = cargo_preparation_plan(profile, {}, Mock())
            selected = any(a["area"] == "runtime_platform" and
                           {"sanitizer-smoke", "sanitizer-full"}.intersection(a["suites"])
                           for a in profile.get("selected_areas", []))
            edge = "area_graph_runtime_platform"
            self.assertEqual(edge in {n.name for n in nodes}, selected)
            self.assertEqual(edge in consumers.get("area_runtime_platform", []), selected)
            self.assertFalse(any(edge in requires for name, requires in consumers.items()
                                 if name != "area_runtime_platform"))
            self.assertEqual(profile, before)

    def test_failed_preparation_blocks_runtime_but_not_independent_area(self):
        from .cargo_setup import cargo_preparation_plan
        from .profiles import load_all_profiles
        from .preparation import PreparationRun
        from .profile_commands import CommandFailed
        profile = copy.deepcopy(load_all_profiles()["merge"])
        run = Mock(side_effect=lambda cmd, **kw: self.fail_build(cmd))
        nodes, consumers = cargo_preparation_plan(profile, {}, run)
        selected = [n for n in nodes if n.name in {"workspace", "area_graph_runtime_platform"}]
        edges = {name: consumers[name] for name in ["area_runtime_platform", "area_diagnostics"]}
        prepared = PreparationRun(selected, edges, no_fail_fast=True)
        with contextlib.redirect_stdout(io.StringIO()), self.assertRaises(CommandFailed):
            prepared.run()
        self.assertEqual(prepared.blocked_by("area_runtime_platform"), "area_graph_runtime_platform")
        self.assertEqual(prepared.blocked_by("area_diagnostics"), "")

    @staticmethod
    def fail_build(command):
        from .profile_commands import CommandFailed
        if "sifr_verify.runtime_sanitizer_preparation" in command:
            raise CommandFailed(101, "exit")


def policy_checks() -> None:
    suite = unittest.TestSuite(unittest.defaultTestLoader.loadTestsFromTestCase(cls)
                               for cls in (SanitizerTargetTests, SanitizerPreparationTests))
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    if not result.wasSuccessful():
        raise AssertionError("runtime sanitizer target checks failed")


if __name__ == "__main__":
    policy_checks()
