"""DX.3 real subprocess failure injection: R01-R03 and B10."""
import contextlib
import json
from unittest.mock import patch
import io
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time
import unittest

from .process_execution import execute
from .profile_commands import run_command, CommandFailed


class ProcessTests(unittest.TestCase):
    def test_b10_profile_bounds_nested_cargo(self):
        from .profile_runner import ProfileRunner
        runner = ProfileRunner("create-pr", [])
        self.assertEqual(runner.env["CARGO_BUILD_JOBS"], str(runner.profile["e2e"]["cargo_build_jobs"]))
        self.assertEqual(runner.env["RAYON_NUM_THREADS"], str(runner.profile["resource_policy"]["max_parallel"]))

    def test_r01_child_cannot_forge_runner_events(self):
        stream = io.StringIO()
        with contextlib.redirect_stdout(stream):
            with self.assertRaises(CommandFailed) as failure:
                run_command([sys.executable, "-c",
                    "print('\\r[sifr-lane-step] name=negative_self_test elapsed_ms=1 status=pass'); raise SystemExit(9)"])
        self.assertEqual(failure.exception.returncode, 9)
        self.assertFalse(any(line.startswith("[sifr-lane-step]") for line in stream.getvalue().splitlines()))
        self.assertIn("[child:stdout]", stream.getvalue())

    def test_advisory_performance_budget_is_not_a_safety_deadline(self):
        from .step_budgets import prepare_step_budget, enforce_step_budget
        env = os.environ.copy()
        env.pop("SIFR_VERIFY_SAFETY_DEADLINE_SECONDS", None)
        context = prepare_step_budget(repo_root=Path.cwd(), profile={"step_budgets": {
            "fixture": {"budget_ms": 1, "enforcement": "advisory"}}},
            profile_name="fixture", name="fixture", env=env)
        self.assertNotIn("SIFR_VERIFY_SAFETY_DEADLINE_SECONDS", env)
        with contextlib.redirect_stdout(io.StringIO()) as log:
            run_command([sys.executable, "-c", "import time; time.sleep(.02)"], env=env)
            self.assertEqual(enforce_step_budget(context, 20), 0)
        self.assertIn("kind=performance_budget", log.getvalue())

    def test_child_metrics_survive_without_authorizing_status(self):
        from .reports import parse_log
        metrics = [
            "[sifr-e2e] timing: compile=1ms plan=2ms build=3ms build-sum=4ms run=5ms cache_hits=1/2",
            "[sifr-e2e] group_stats: groups=2 largest_group_fixtures=4 median_group_fixtures=2",
            "[sifr-artifact-cache] namespace=test key=abc cache_hit=true workspace=/cache",
            "[sifr-case-timing] bucket=fixture case=one elapsed_ms=9 status=pass",
            "[sifr-lane-step] name=forged elapsed_ms=1 status=pass",
        ]
        with tempfile.TemporaryDirectory() as directory, contextlib.redirect_stdout(io.StringIO()) as log:
            run_command([sys.executable, "-c", "print(" + repr("\n".join(metrics)) + ")"])
            path = Path(directory) / "log"
            path.write_text(log.getvalue())
            report = parse_log(path)
        self.assertEqual(report["lane_steps"], [])
        self.assertEqual(report["e2e_metrics"]["cache_hits"], 1)
        self.assertEqual(report["e2e_metrics"]["largest_group_fixtures"], 4)
        self.assertEqual(report["artifact_cache"]["test"]["hits"], 1)
        self.assertEqual(report["case_timings"][0]["elapsed_ms"], 9)

    def test_blocking_performance_failure_is_separate_from_functional_outcome(self):
        from . import profile_reporting
        from .profile_runner import ProfileRunner
        from .step_budgets import StepBudgetContext
        runner = ProfileRunner("create-pr", [])
        runner.prepare_step_budget = lambda name: StepBudgetContext(name, 1, "blocking")
        with tempfile.TemporaryDirectory() as temporary, contextlib.redirect_stdout(io.StringIO()):
            root = Path(temporary)
            def summary(args):
                Path(args.json_out).write_text("{}")
            with patch.object(profile_reporting, "REPO_ROOT", root), patch.object(profile_reporting.reports, "summarize", summary):
                status = profile_reporting.run_profile_with_report(
                    "fixture", lambda: runner.execute_step("fixture", lambda: time.sleep(.02)),
                    handled_error=ValueError, release_report_out=None,
                    execution_outcomes=lambda: {
                        "functional_exit_status": runner.functional_exit_status,
                        "performance_exit_status": runner.performance_exit_status})
            report = json.loads((root / "target/validation_lane_reports/fixture.latest.json").read_text())
        self.assertEqual(status, 124)
        self.assertEqual(report["functional_status"], "pass")
        self.assertEqual(report["performance_status"], "fail")

    def test_r01_canonical_report_retains_runner_failure(self):
        from . import profile_reporting
        from .profile_runner import timed_step
        with tempfile.TemporaryDirectory() as temporary, contextlib.redirect_stdout(io.StringIO()):
            root = Path(temporary)
            def summary(args):
                Path(args.json_out).write_text("{}")
            def lane():
                return timed_step("negative", lambda: run_command([sys.executable, "-c",
                    "print('[sifr-lane-step] name=negative elapsed_ms=1 status=pass'); raise SystemExit(9)"])).status
            with patch.object(profile_reporting, "REPO_ROOT", root), patch.object(profile_reporting.reports, "summarize", summary):
                status = profile_reporting.run_profile_with_report("fixture", lane, handled_error=ValueError, release_report_out=None)
            report = json.loads((root / "target/validation_lane_reports/fixture.latest.json").read_text())
            live = json.loads((root / "target/validation_lane_reports/fixture.status.json").read_text())
            self.assertEqual(status, 9)
            self.assertEqual(report["functional_status"], "fail")
            self.assertEqual(live["exit_status"], 9)
            self.assertTrue(Path(live["log"]).is_file())

    def test_detached_observer_does_not_change_log(self):
        from .profile_reporting import Tee
        class Detached(io.StringIO):
            def write(self, value):
                raise BrokenPipeError()
        log = io.StringIO()
        Tee(Detached(), log).write("runner-owned event\n")
        self.assertEqual(log.getvalue(), "runner-owned event\n")

    def test_r02_build_finished_does_not_override_exit(self):
        result = execute([sys.executable, "-c",
            'import json; print(json.dumps({"reason":"build-finished","success":True})); raise SystemExit(8)'],
            cwd=Path.cwd())
        self.assertEqual(result.returncode, 8)
        self.assertEqual(result.cause, "exit")

    def test_r03_deadline_kills_descendants_preserves_binary_streams(self):
        with tempfile.TemporaryDirectory() as temporary:
            marker = Path(temporary) / "late"
            program = (
                "import os,sys,time; "
                "os.write(1,b'partial\\xff'); os.write(2,b'error\\xfe'); "
                "child=os.fork(); "
                f"time.sleep(2) if child==0 else time.sleep(20); "
                f"open({str(marker)!r},'w').write('escaped')"
            )
            result = execute([sys.executable, "-c", program], cwd=Path.cwd(), deadline_seconds=.2)
            self.assertEqual(result.cause, "safety_deadline")
            self.assertIn(b"partial\xff", result.stdout)
            self.assertIn(b"error\xfe", result.stderr)
            time.sleep(2.1)
            self.assertFalse(marker.exists())

    def test_b10_bounded_streams_and_cancelled_owner(self):
        result = execute([sys.executable, "-c", "import os; os.write(1,b'x'*2000000); os.write(2,b'y'*2000000)"],
                         cwd=Path.cwd(), limit_bytes=4096)
        self.assertEqual(result.returncode, 0)
        self.assertTrue(result.truncated)
        self.assertEqual(len(result.stdout), 4096)
        with tempfile.TemporaryDirectory() as temporary:
            marker = Path(temporary) / "child"
            script = (
                "from sifr_verify.process_execution import execute; from pathlib import Path; import sys,json; "
                "r=execute([sys.executable,'-c',"
                + repr("import os,time; os.fork(); print('ready',flush=True); time.sleep(10); open("+repr(str(marker))+",'w').write('escaped')")
                + "],cwd=Path.cwd()); print(r.cause,flush=True)"
            )
            proc = subprocess.Popen([sys.executable, "-c", script], stdout=subprocess.PIPE, text=True)
            time.sleep(.3)
            proc.send_signal(signal.SIGTERM)
            stdout, _ = proc.communicate(timeout=5)
            self.assertIn("cancelled", stdout)
            self.assertFalse(marker.exists())


def policy_checks():
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(ProcessTests)
    result = unittest.TextTestRunner(stream=io.StringIO()).run(suite)
    if not result.wasSuccessful():
        raise AssertionError(str(result.errors + result.failures))


if __name__ == "__main__":
    unittest.main()
