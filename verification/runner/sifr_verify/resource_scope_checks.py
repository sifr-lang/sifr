"""CPU counter ownership, process isolation and report trust controls."""
import contextlib
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import threading
import unittest
from unittest.mock import patch

from .resource_scopes import PREFIX, ResourceScopes, cpu_scope, measure_cpu
from .reports import parse_log

BURN = "import time; end=time.process_time()+.15\nwhile time.process_time()<end: pass"


def rows(text):
    return [json.loads(line[len(PREFIX):]) for line in text.splitlines() if line.startswith(PREFIX)]


class ResourceScopeTests(unittest.TestCase):
    def test_nested_counter_deltas_are_inclusive_and_parented(self):
        stream = io.StringIO()
        with contextlib.redirect_stdout(stream), patch(
                "sifr_verify.resource_scopes.counters", side_effect=[(1., 2.), (2., 3.), (4., 5.), (6., 7.)]):
            with cpu_scope("step", "outer"):
                self.assertEqual(measure_cpu("command", "inner", lambda: 42), 42)
        inner, outer = rows(stream.getvalue())
        self.assertEqual(inner["parent_id"], outer["id"])
        self.assertIsNone(outer["parent_id"])
        self.assertEqual((inner["cpu_seconds"], outer["cpu_seconds"]), (4., 10.))
        self.assertEqual(outer["completion"], "returned")

    def test_exception_records_work_and_restores_parent_without_claiming_pass(self):
        stream = io.StringIO()
        with contextlib.redirect_stdout(stream):
            with self.assertRaisesRegex(ValueError, "failure"):
                with cpu_scope("command", "failed"):
                    raise ValueError("failure")
            with cpu_scope("command", "next"):
                pass
        failed, following = rows(stream.getvalue())
        self.assertEqual(failed["completion"], "raised")
        self.assertIsNone(following["parent_id"])
        self.assertNotIn("status", failed)

    def test_actual_waited_grandchild_cpu_is_included(self):
        program = "import subprocess,sys; subprocess.run([sys.executable,'-c'," + repr(BURN) + "],check=True)"
        stream = io.StringIO()
        with contextlib.redirect_stdout(stream), cpu_scope("command", "grandchild"):
            subprocess.run([sys.executable, "-c", program], check=True)
        self.assertGreaterEqual(rows(stream.getvalue())[0]["cpu_seconds"], .14)

    def test_parallel_workers_have_disjoint_process_counters(self):
        prefix = "from sifr_verify.resource_scopes import cpu_scope\n"
        programs = [
            prefix + "with cpu_scope('area','busy'):\n " + BURN.replace("\n", "\n "),
            prefix + "import time\nwith cpu_scope('area','idle'):\n time.sleep(.25)",
        ]
        env = dict(os.environ, PYTHONPATH=str(Path(__file__).resolve().parents[1]))
        processes = [subprocess.Popen([sys.executable, "-c", p], env=env, text=True,
                                     stdout=subprocess.PIPE, stderr=subprocess.PIPE) for p in programs]
        try:
            outputs = [p.communicate(timeout=10) for p in processes]
            self.assertTrue(all(p.returncode == 0 for p in processes), outputs)
            busy, idle = [rows(out)[0] for out, _ in outputs]
            self.assertNotEqual(busy["pid"], idle["pid"])
            self.assertGreaterEqual(busy["cpu_seconds"], .14)
            self.assertLess(idle["cpu_seconds"], .05)
        finally:
            for p in processes:
                if p.poll() is None:
                    p.kill()
                p.wait()

    def test_thread_scope_rejected_before_work(self):
        errors = []
        def work():
            try:
                measure_cpu("command", "thread", lambda: self.fail("must not run"))
            except RuntimeError as error:
                errors.append(str(error))
        thread = threading.Thread(target=work)
        thread.start()
        thread.join()
        self.assertEqual(errors, ["CPU scopes require a serial main-thread owner"])

    def test_report_separates_child_metrics_and_rejects_invalid_totals(self):
        stream = io.StringIO()
        with contextlib.redirect_stdout(stream), cpu_scope("step", "real"):
            pass
        real = rows(stream.getvalue())[0]
        forged = dict(real, id="forged", name="fake")
        line = PREFIX + json.dumps(real)
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "log"
            path.write_text(line + "\n[child:stdout] " + json.dumps(PREFIX + json.dumps(forged)) + "\n")
            report = parse_log(path)
        costs = report["resource_scopes"]
        self.assertEqual([r["name"] for r in costs["owned"]], ["real"])
        self.assertEqual([r["name"] for r in costs["observed_child"]], ["fake"])
        self.assertEqual(report["lane_steps"], [])
        parser = ResourceScopes()
        parser.add(line)
        parser.add(line)
        for value in (float("nan"), -1, True):
            parser.add(PREFIX + json.dumps(dict(real, id=str(value), cpu_seconds=value)))
        parser.add(PREFIX + json.dumps(dict(real, id="wrong-sum", cpu_seconds=99)))
        self.assertEqual(len(parser.owned), 1)
        self.assertEqual(len(parser.errors), 5)


def policy_checks():
    result = unittest.TextTestRunner(stream=io.StringIO()).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(ResourceScopeTests))
    if not result.wasSuccessful():
        raise AssertionError(str(result.errors + result.failures))


if __name__ == "__main__":
    unittest.main()
