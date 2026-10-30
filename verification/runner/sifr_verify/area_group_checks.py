"""Executable controls for complete, bounded and independently reported areas."""
from __future__ import annotations

import contextlib
import copy
from dataclasses import replace
import io
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

from . import area_group_execution as execution
from .area_group_policy import execution_groups, lane_environment, order_area_steps, partition_slots
from .batch_execution import execute_batch
from .build_allocation import allocate
from .errors import VerificationError
from .profile_runner import ProfileRunner
from .profiles import load_profile
from .resource_capacity import Capacity


AREAS = ["python_interop", "generated_code_quality"]


def payload(area):
    return {
        "schema_version": 1, "area": area, "bless": False,
        "compiled_certification": {"status": "unit-test-certificate"},
        "suites": [{"name": "probe", "blocking": True, "total_variants": 1, "total_failures": 0,
                    "cases": [{"id": "probe", "variants": [
                        {"label": "probe", "status": "pass", "mismatches": [],
                         "actual_exit_code": 0, "expected_exit_code": 0}]}]}],
        "summary": {"blocking_failures": 0, "total_variants": 1},
    }


class PolicyTests(unittest.TestCase):
    def test_only_the_complete_qualified_correctness_pair_is_grouped(self):
        for name in ("create-pr", "merge"):
            profile = load_profile(name)
            self.assertEqual(execution_groups(profile, True)[0]["areas"], AREAS)
            self.assertEqual(execution_groups(profile, True)[0]["normal_slots"],
                             partition_slots(AREAS, profile["resource_policy"]["max_parallel"])[0])
            self.assertEqual(execution_groups(profile, False), [])
            profile["selected_areas"] = [a for a in profile["selected_areas"]
                                         if a["area"] != "python_interop"]
            self.assertNotIn("python-quality", [g["id"] for g in execution_groups(profile, True)])
        self.assertEqual(execution_groups(load_profile("release"), False), [])

    def test_capacity_and_inner_requests_are_bounded_without_mutating_parent(self):
        profile = load_profile("merge")
        build = allocate(profile, {}, {}, Capacity(8, 16 * 1024**3))
        env = {"RUST_TEST_THREADS": "2", "OPENBLAS_NUM_THREADS": "1"}
        for total in (1, 2, 3, 4):
            parts, parallel = partition_slots(AREAS, total)
            self.assertLessEqual(sum(parts.values()) if parallel == 2 else max(parts.values()), total)
            for area in AREAS:
                lane, child = lane_environment(replace(build, slot_limit=total), env, parts[area])
                self.assertLessEqual(lane.rust_jobs * lane.cargo_build_jobs, parts[area])
                self.assertLessEqual(lane.compiler_jobs, parts[area])
                self.assertEqual(child["OPENBLAS_NUM_THREADS"], "1")
                self.assertEqual(child["RUST_TEST_THREADS"], str(min(2, parts[area])))
                native = json.loads(child["SIFR_VERIFY_NATIVE_TEST_ALLOCATION"])
                self.assertLessEqual(native["workers"] * native["cargo_jobs"], parts[area])
                if native["workers"]:
                    self.assertLessEqual(native["workers"] * native["memory_per_worker_mib"]
                                         + native["reserved_memory_mib"], lane.available_memory_mib)
        self.assertEqual(env, {"RUST_TEST_THREADS": "2", "OPENBLAS_NUM_THREADS": "1"})
        self.assertEqual(partition_slots(AREAS, 4), (dict(zip(AREAS, (1, 3))), 2))
        self.assertEqual(partition_slots([AREAS[0]], 4), ({AREAS[0]: 4}, 1))
        small = replace(build, compiler_jobs=1, sifr_jobs=1, run_jobs=1)
        lane, _ = lane_environment(small, {}, 3)
        self.assertEqual((lane.compiler_jobs, lane.sifr_jobs, lane.run_jobs), (1, 1, 1))
        for invalid in ("+2", "1_0", "٢", "0"):
            with self.assertRaises(VerificationError):
                lane_environment(build, {"RUST_TEST_THREADS": invalid}, 3)
        with self.assertRaises(ValueError):
            lane_environment(build, {}, 5)

    def test_report_order_preserves_observed_duration_and_nonarea_steps(self):
        profile = {"selected_areas": [{"area": "a"}, {"area": "b"}, {"area": "c"}]}
        steps = [{"name": "guardrail", "elapsed_ms": 1}, {"name": "area_a", "elapsed_ms": 90},
                 {"name": "area_c", "elapsed_ms": 20}, {"name": "area_b", "elapsed_ms": 60}]
        ordered = order_area_steps(profile, steps)
        self.assertEqual([s["name"] for s in ordered], ["guardrail", "area_a", "area_b", "area_c"])
        self.assertEqual([s["elapsed_ms"] for s in ordered], [1, 90, 60, 20])
        self.assertEqual(steps[2]["name"], "area_c")


class ExecutionTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.compiler = self.root / "compiler"
        self.compiler.write_bytes(b"prepared compiler")
        self.runner = ProfileRunner("merge", ["--no-fail-fast"], correctness_only=True)
        self.runner.profile = copy.deepcopy(self.runner.profile)
        self.runner.profile.update(
            guardrail_steps=[], toolchain_steps=[], step_budgets={},
            selected_areas=[{"area": a, "suites": ["probe"]} for a in AREAS])
        self.runner.build_allocation = allocate(self.runner.profile, {}, {}, Capacity(8, 16 * 1024**3))
        self.runner.env.update(SIFR_GCQ_BIN=str(self.compiler), RUST_TEST_THREADS="4")
        self.runner.env.pop("SIFR_VERIFY_SAFETY_DEADLINE_MONOTONIC", None)
        self.blocked = {}
        self.runner.preparation = type("Prepared", (), {
            "blocked_by": lambda _, name: self.blocked.get(name, "")})()
        self.commands = []
        self.programs = {}

    def program(self, area, *, document=True, code=0, extra="", barrier=False):
        path = self.root / "target/verification/areas" / f"{area.replace('_', '-')}-merge-correctness-results.json"
        data = payload(area) if document is True else document
        text = "import os,json,time\nfrom pathlib import Path\n"
        text += f"root=Path({str(self.root)!r})\n(root/{area!r}).write_text(json.dumps(dict(os.environ)))\n"
        if barrier:
            sibling = next(a for a in AREAS if a != area)
            text += (f"end=time.monotonic()+3\nwhile not (root/{sibling!r}).exists():\n"
                     " if time.monotonic()>end:raise RuntimeError('no sibling overlap')\n time.sleep(.01)\n")
        text += extra + "\n"
        if data is not False:
            text += f"p=Path({str(path)!r});p.parent.mkdir(parents=True,exist_ok=True);p.write_text({json.dumps(data)!r})\n"
        text += f"raise SystemExit({code})\n"
        return [sys.executable, "-c", text]

    def invoke(self, areas=AREAS):
        self.log = io.StringIO()
        def run(commands, **kwargs):
            self.commands.extend(commands)
            actual = [self.programs.get(command[command.index("--area") + 1],
                                       self.program(command[command.index("--area") + 1]))
                      for command in commands]
            kwargs["limit_bytes"] = getattr(self, "output_limit", kwargs["limit_bytes"])
            execute_batch(actual, **kwargs)
        with patch.object(execution, "REPO_ROOT", self.root), \
             patch.object(execution, "execute_batch", side_effect=run), \
             patch.object(execution, "discover_capacity", return_value=Capacity(8, 16 * 1024**3)), \
             contextlib.redirect_stdout(self.log):
            return execution.run_area_group(self.runner, areas)

    def test_real_children_overlap_receive_partition_and_emit_complete_results(self):
        self.programs = {area: self.program(area, barrier=True) for area in AREAS}
        self.assertEqual(self.invoke(), 0)
        self.assertEqual(len(self.commands), 2)
        self.assertEqual(self.runner.completed_steps, {"area_" + a for a in AREAS})
        for area, slots in zip(AREAS, (1, 3), strict=True):
            env = json.loads((self.root / area).read_text())
            self.assertEqual(env["SIFR_VERIFY_PROCESS_SLOTS"], str(slots))
            self.assertEqual(env["CARGO_BUILD_JOBS"], str(slots))
            self.assertEqual(env["RUST_TEST_THREADS"], str(slots))
        self.assertFalse(self.runner.stop_requested)

    def test_admitted_sibling_finishes_in_both_failure_modes(self):
        for no_fail_fast in (False, True):
            with self.subTest(no_fail_fast=no_fail_fast):
                self.runner.no_fail_fast = no_fail_fast
                self.runner.completed_steps.clear()
                self.programs[AREAS[0]] = self.program(AREAS[0], code=9)
                self.assertNotEqual(self.invoke(), 0)
                self.assertTrue((self.root / AREAS[1]).exists())
                self.assertEqual(len(self.runner.completed_steps), 2)

    def test_single_slot_stops_queued_consumer_only_in_fail_fast_mode(self):
        self.runner.build_allocation = replace(self.runner.build_allocation, slot_limit=1)
        self.runner.no_fail_fast = False
        self.programs[AREAS[0]] = self.program(AREAS[0], code=9)
        self.assertNotEqual(self.invoke(), 0)
        self.assertFalse((self.root / AREAS[1]).exists())
        self.runner.completed_steps.clear()
        self.runner.no_fail_fast = True
        self.assertNotEqual(self.invoke(), 0)
        self.assertTrue((self.root / AREAS[1]).exists())

    def test_blocked_prerequisite_never_launches_its_consumer(self):
        self.blocked["area_python_interop"] = "python_environment"
        self.assertNotEqual(self.invoke(), 0)
        self.assertFalse((self.root / AREAS[0]).exists())
        env = json.loads((self.root / AREAS[1]).read_text())
        self.assertEqual(env["SIFR_VERIFY_PROCESS_SLOTS"], "4")
        self.runner.completed_steps.clear()
        self.runner.no_fail_fast = False
        self.commands.clear()
        self.assertNotEqual(self.invoke(), 0)
        self.assertEqual(self.commands, [])

    def test_missing_stale_forged_and_incomplete_certification_cannot_pass(self):
        variants = [False, payload("wrong-area"), payload(AREAS[0])]
        variants[-1].pop("compiled_certification")
        for document in variants:
            with self.subTest(document=document):
                self.runner.completed_steps.clear()
                old = self.root / "target/verification/areas/python-interop-merge-correctness-results.json"
                old.parent.mkdir(parents=True, exist_ok=True)
                old.write_text(json.dumps(payload(AREAS[0])))
                self.programs[AREAS[0]] = self.program(AREAS[0], document=document)
                self.assertNotEqual(self.invoke(), 0)
                self.assertEqual(len(self.runner.completed_steps), 2)

    def test_shared_compiler_mutation_invalidates_both_outcomes(self):
        self.programs[AREAS[0]] = self.program(
            AREAS[0], extra=f"Path({str(self.compiler)!r}).write_bytes(b'mutated')")
        self.assertNotEqual(self.invoke(), 0)
        self.assertTrue(self.runner.stop_requested)
        reports = list((self.root / "target/verification/area-groups").glob("*/outcomes.json"))
        data = json.loads(reports[-1].read_text())
        self.assertTrue(all(row["status"] != 0 for row in data["outcomes"].values()))


    def test_truncated_output_and_deadlines_fail_closed(self):
        self.output_limit = 64
        self.programs[AREAS[0]] = self.program(AREAS[0], extra="print('x'*4096)")
        self.assertNotEqual(self.invoke(), 0)
        self.assertTrue(self.runner.stop_requested)
        self.output_limit = 32 * 1024 * 1024
        self.runner.completed_steps.clear()
        self.runner.stop_requested = False
        self.runner.env["SIFR_VERIFY_STEP_SAFETY_DEADLINE_SECONDS"] = "1"
        self.programs = {area: self.program(area, extra="time.sleep(20)") for area in AREAS}
        self.assertNotEqual(self.invoke(), 0)
        self.assertTrue(self.runner.stop_requested)
        self.assertEqual(len(self.runner.completed_steps), 2)

    def test_nested_process_owner_finishes_cleanup_on_cancellation(self):
        for executor in ("single", "batch"):
            with self.subTest(executor=executor):
                self._nested_cleanup(executor)

    def _nested_cleanup(self, executor):
        import fcntl
        import signal
        import subprocess
        from .batch_execution_checks import wait_file
        lock = self.root / f"nested-{executor}.lock"
        ready = self.root / f"nested-{executor}.pid"
        nested = ("import os,signal,time,fcntl\nfrom pathlib import Path\n"
                  "signal.signal(signal.SIGTERM,signal.SIG_IGN)\n"
                  f"f=open({str(lock)!r},'w');fcntl.flock(f,fcntl.LOCK_EX)\n"
                  f"Path({str(ready)!r}).write_text(str(os.getpid()))\ntime.sleep(30)\n")
        parent_program = (
            "import sys,json\nfrom pathlib import Path\n"
            "from sifr_verify.area_group_checks import ExecutionTests,AREAS\n"
            "t=ExecutionTests();t.setUp()\n"
            f"t.root=Path({str(self.root)!r});t.compiler=t.root/'compiler'\n"
            "t.runner.env['SIFR_GCQ_BIN']=str(t.compiler)\n"
            "t.programs[AREAS[0]]=t.program(AREAS[0],extra="
            + repr(("from sifr_verify.process_execution import execute\n"
                    f"execute([{sys.executable!r},'-c',{nested!r}],cwd=root,env=os.environ.copy())")
                   if executor == "single" else
                   ("from sifr_verify.batch_execution import execute_batch\n"
                    f"execute_batch([[{sys.executable!r},'-c',{nested!r}]],cwd=root,env=os.environ.copy(),"
                    "max_parallel=1,completed=lambda *_:None)"))
            + ")\nt.programs[AREAS[1]]=t.program(AREAS[1],extra='time.sleep(30)')\n"
            "status=t.invoke();print(json.dumps({'status':status,'stopped':t.runner.stop_requested}))\n")
        parent = subprocess.Popen([sys.executable, "-c", parent_program], stdout=subprocess.PIPE,
                                  stderr=subprocess.PIPE, start_new_session=True)
        try:
            wait_file(ready)
            wait_file(self.root / AREAS[1])
            parent.send_signal(signal.SIGTERM)
            out, error = parent.communicate(timeout=8)
            self.assertEqual(parent.returncode, 0, error)
            data = json.loads(out)
            self.assertNotEqual(data["status"], 0)
            self.assertTrue(data["stopped"])
            with lock.open("a") as handle:
                # The nested child deliberately holds this lock until termination.
                fcntl.flock(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
        finally:
            if parent.poll() is None:
                os.killpg(parent.pid, signal.SIGKILL)
                parent.wait()
            if ready.exists():
                try:
                    os.killpg(int(ready.read_text()), signal.SIGKILL)
                except ProcessLookupError:
                    pass
            parent.stdout.close()
            parent.stderr.close()


    def test_successful_worker_with_live_descendant_fails_and_cleans_only_its_session(self):
        import fcntl
        import signal
        import subprocess
        from .batch_execution_checks import wait_file
        lock = self.root / "abandoned.lock"
        ready = self.root / "abandoned.pid"
        child = ("import os,time,signal,fcntl\nfrom pathlib import Path\n"
                 "signal.signal(signal.SIGTERM,signal.SIG_IGN)\n"
                 f"f=open({str(lock)!r},'w');fcntl.flock(f,fcntl.LOCK_EX)\n"
                 f"Path({str(ready)!r}).write_text(str(os.getpid()))\ntime.sleep(30)")
        extra = ("import subprocess\n"
                 f"subprocess.Popen([{sys.executable!r},'-c',{child!r}],"
                 "stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,process_group=0)\n"
                 f"while not Path({str(ready)!r}).exists():time.sleep(.01)")
        outsider = subprocess.Popen([sys.executable, "-c", "import time;time.sleep(30)"],
                                    start_new_session=True)
        try:
            self.programs[AREAS[0]] = self.program(AREAS[0], extra=extra)
            self.assertNotEqual(self.invoke(), 0)
            wait_file(ready)
            with lock.open("a") as handle:
                fcntl.flock(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
            self.assertIsNone(outsider.poll(), "cleanup crossed the owned session boundary")
            reports = list((self.root / "target/verification/area-groups").glob("*/outcomes.json"))
            data = json.loads(reports[-1].read_text())
            self.assertEqual(data["outcomes"][AREAS[0]]["cause"], "leftover_process")
        finally:
            os.killpg(outsider.pid, signal.SIGKILL)
            outsider.wait()
            if ready.exists():
                try:
                    os.killpg(int(ready.read_text()), signal.SIGKILL)
                except ProcessLookupError:
                    pass

    def test_foreign_or_malformed_session_rejected_before_spawn(self):
        from .process_execution import execute
        from .process_ownership import SESSION_ENV
        for marker in ("-1", "+2", "٢", "0", str(os.getsid(0) + 1)):
            with self.subTest(marker=marker), patch("subprocess.Popen") as spawn:
                with self.assertRaises(ValueError):
                    execute([sys.executable, "-c", "pass"], cwd=self.root,
                            env={**os.environ, SESSION_ENV: marker})
                with self.assertRaises(ValueError):
                    execute_batch([[sys.executable, "-c", "pass"]], cwd=self.root,
                                  env={**os.environ, SESSION_ENV: marker}, max_parallel=1,
                                  completed=lambda *_: None)
                spawn.assert_not_called()


    def test_parent_requires_strict_freshness_even_after_a_successful_worker(self):
        selection = next(a for a in self.runner.profile["selected_areas"]
                         if a["area"] == "generated_code_quality")
        selection["require_fresh_demo_companions"] = True
        self.assertNotEqual(self.invoke(), 0)
        self.assertNotEqual(self.runner.functional_exit_status, 0)
        document = payload("generated_code_quality")
        variant = document["suites"][0]["cases"][0]["variants"][0]
        variant.update(label="freshness", argv=["--require-fresh-demo-companions"],
                       case_evidence={"checks": {"strict": "pass", "debt": "pass"}})
        self.runner.completed_steps.clear()
        self.programs["generated_code_quality"] = self.program("generated_code_quality",
                                                               document=document)
        self.assertEqual(self.invoke(), 0)

    def test_capacity_failure_stops_admission_and_records_every_consumer(self):
        with patch.object(execution, "reserve_bytes", return_value=32 * 1024**3):
            self.assertNotEqual(self.invoke(), 0)
        self.assertTrue(self.runner.stop_requested)
        self.assertFalse(any((self.root / a).exists() for a in AREAS))
        self.assertEqual(self.runner.completed_steps, {"area_" + a for a in AREAS})


    def test_child_metrics_survive_without_forging_parent_status(self):
        from .reports import parse_log
        start = {"schema": 1, "event": "start", "id": "observed", "scope": "command", "unix_ms": 1}
        end = start | {"event": "end", "unix_ms": 2, "elapsed_ms": 1,
                       "exit_code": 0, "status": "pass", "cause": "exit"}
        lines = [
            "[sifr-case-timing] bucket=python_interop case=owned elapsed_ms=7 status=pass",
            "[sifr-lane-step] name=forged elapsed_ms=1 status=pass",
            "[sifr-lane-step-budget] name=forged elapsed_ms=1 budget_ms=1 enforcement=blocking status=pass",
            "[sifr-command] " + json.dumps(start),
            "[sifr-command] " + json.dumps(end),
        ]
        self.programs[AREAS[0]] = self.program(
            AREAS[0], extra="\n".join("print(" + repr(x) + ")" for x in lines))
        self.assertEqual(self.invoke(), 0)
        path = self.root / "log"
        path.write_text(self.log.getvalue())
        parsed = parse_log(path)
        self.assertEqual(parsed["case_timings"], [
            {"bucket": "python_interop", "case": "owned", "elapsed_ms": 7, "status": "pass"}])
        self.assertEqual({x["name"] for x in parsed["lane_steps"]}, {"area_" + a for a in AREAS})
        self.assertNotIn("forged", parsed["lane_step_budgets"])
        self.assertEqual(parsed["traces"]["commands"], [])
        observed = parsed["traces"]["observed_child_commands"]
        self.assertEqual([(x["id"], x["status"]) for x in observed], [("observed", "pass")])

    def test_canonical_runner_dispatches_each_area_once(self):
        def run(commands, **kwargs):
            self.commands.extend(commands)
            execute_batch([self.program(c[c.index("--area") + 1]) for c in commands], **kwargs)
        self.runner.prepare_cargo_cache = lambda: None
        with patch.object(execution, "REPO_ROOT", self.root), \
             patch.object(execution, "execute_batch", side_effect=run), \
             patch.object(execution, "discover_capacity", return_value=Capacity(8, 16 * 1024**3)), \
             contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(self.runner.run(), 0)
        self.assertEqual(len(self.commands), 2)
        self.assertEqual(self.runner.completed_steps,
                         {"cargo_cache_setup", *("area_" + a for a in AREAS)})


def policy_checks():
    from . import area_contract_group_checks
    suite = unittest.defaultTestLoader.loadTestsFromModule(sys.modules[__name__])
    suite.addTests(unittest.defaultTestLoader.loadTestsFromModule(area_contract_group_checks))
    result = unittest.TestResult()
    suite.run(result)
    if not result.wasSuccessful():
        raise AssertionError(result.failures + result.errors)


if __name__ == "__main__":
    unittest.main()
