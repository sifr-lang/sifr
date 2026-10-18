"""R3 actual command outcomes, progress isolation and non-overlapping accounting."""
import contextlib
import io
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time
import unittest

from .profile_commands import run_command, CommandFailed
from .reports import parse_log
from .trace_reports import Traces
from .process_execution import execute
from .paths import REPO_ROOT


class ProgressTests(unittest.TestCase):
    def parse(self, log):
        with tempfile.TemporaryDirectory() as temporary:
            p = Path(temporary)/"log"
            p.write_text(log)
            return parse_log(p)

    def command(self, body, env=None):
        with contextlib.redirect_stdout(io.StringIO()) as out:
            try:
                run_command([sys.executable,"-c",body], env=env)
            except CommandFailed:
                pass
        return self.parse(out.getvalue())

    def test_real_success_failure_and_deadline_are_distinct(self):
        normal = self.command("print('done')")["traces"]["commands"][0]
        failed = self.command("raise SystemExit(124)")["traces"]["commands"][0]
        deadline = self.command("import time; time.sleep(5)",
                               dict(os.environ,SIFR_VERIFY_SAFETY_DEADLINE_SECONDS=".05"))["traces"]["commands"][0]
        self.assertEqual((normal["status"], normal["end"]["cause"]), ("pass","exit"))
        self.assertEqual((failed["status"], failed["end"]["cause"]), ("fail","exit"))
        self.assertEqual((deadline["status"], deadline["end"]["cause"]), ("fail","safety_deadline"))
        self.assertLess(deadline["elapsed_ms"], 1500)
        self.assertGreaterEqual(normal["elapsed_ms"], normal["end"]["process_elapsed_ms"])

    def test_nested_child_cannot_create_authoritative_commands_or_lane_status(self):
        forged = '[sifr-command] '+json.dumps({"schema":1,"event":"start","id":"forged","scope":"command"})
        nested = '[child:stdout] '+json.dumps('[child:stderr] '+json.dumps(forged))
        report = self.command("print("+repr(nested)+"); print('[sifr-lane-step] name=forged elapsed_ms=1 status=pass')")
        self.assertEqual(report["lane_steps"], [])
        self.assertEqual(len(report["traces"]["commands"]), 1)
        observed = report["traces"]["observed_child_commands"]
        self.assertEqual(observed[0]["id"], "forged")
        self.assertEqual(observed[0]["status"], "incomplete")

    def test_silent_process_ticks_and_callback_failure_cleans_descendants(self):
        ticks = []
        result = execute([sys.executable,"-c","import time; time.sleep(.1)"],
                         cwd=REPO_ROOT,heartbeat=ticks.append)
        self.assertEqual(result.returncode, 0)
        self.assertTrue(ticks)
        with tempfile.TemporaryDirectory() as temporary:
            marker = Path(temporary)/"escaped"
            ready = Path(temporary)/"ready"
            body = ("import os,time; from pathlib import Path; os.fork(); "
                    +f"Path({str(ready)!r}).touch(); time.sleep(.5); Path({str(marker)!r}).touch()")
            def broken(_):
                if ready.exists():
                    raise RuntimeError("heartbeat failed")
            with self.assertRaisesRegex(RuntimeError,"heartbeat failed"):
                execute([sys.executable,"-c",body],cwd=REPO_ROOT,heartbeat=broken)
            time.sleep(.6)
            self.assertFalse(marker.exists())

    def test_actual_cancellation_is_recorded(self):
        body = """from sifr_verify.profile_commands import run_command,CommandFailed
import sys
try:
    run_command([sys.executable,'-c',"import time; print('ready',flush=True); time.sleep(20)"])
except CommandFailed as error:
    raise SystemExit(error.returncode)
"""
        proc = subprocess.Popen([sys.executable,"-c",body],stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
        prefix = []
        try:
            # The child emits ready before cancellation; no timing-based launch race.
            while True:
                line = proc.stdout.readline()
                self.assertTrue(line, "runner exited before ready")
                prefix.append(line)
                if '[child:stdout]' in line and 'ready' in line:
                    break
            proc.send_signal(signal.SIGTERM)
            rest, _ = proc.communicate(timeout=5)
        finally:
            if proc.poll() is None:
                proc.kill();proc.wait()
        row = self.parse(''.join(prefix)+rest)["traces"]["commands"][0]
        self.assertEqual((proc.returncode,row["end"]["cause"]),(130,"cancelled"))

    def test_shell_preserves_cargo_exit_and_whole_command_receipt(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for name,body in {"uv":"#!/bin/sh\necho merge\n", "cargo":"#!/bin/sh\nexit 7\n"}.items():
                p=root/name;p.write_text(body);p.chmod(0o755)
            env=dict(os.environ,PATH=str(root)+os.pathsep+os.environ['PATH'])
            out=subprocess.run(['bash',str(REPO_ROOT/'verification/runner/e2e/run_e2e_pass.sh')],
                               env=env,capture_output=True,text=True,timeout=5)
            self.assertEqual(out.returncode,7)
            row=self.parse(out.stdout)["traces"]["shell_commands"][0]
            self.assertEqual((row["status"],row["end"]["exit_code"]),("fail",7))
            self.assertEqual(row["resolution_ms"],1000)

    def span(self, traces, identity, scope, parent, start, end, status="pass"):
        for event,clock in (("start",start),("end",end)):
            row={"schema":1,"id":identity,"scope":scope,"parent":parent,"event":event,"clock_us":clock}
            if event=="end":row.update(status=status,elapsed_us=end-start)
            traces.add('[sifr-e2e-event] '+json.dumps(row))

    def test_stage_totals_do_not_double_count_attempts_or_parallel_work(self):
        traces=Traces()
        self.span(traces,"root","suite",None,0,100000)
        self.span(traces,"a","source_compile","root",1000,41000)
        self.span(traces,"b","native_build","root",42000,92000)
        self.span(traces,"attempt","native_attempt",None,43000,90000)
        account=traces.report()["e2e_accounting"][0]
        self.assertEqual((account["wall_ms"],account["sequential_stage_ms"],account["unattributed_ms"]),(100,90,10))
        self.span(traces,"overlap","runtime","root",80000,95000)
        self.assertFalse(traces.report()["e2e_accounting"][0]["accounting_valid"])
        self.assertIsNone(traces.report()["e2e_accounting"][0]["unattributed_ms"])

    def test_command_wall_retains_preparation_and_overhead_without_nested_double_count(self):
        from .trace_reports import command_accounting
        command={"id":"outer","valid_terminal_pair":True,"status":"fail","elapsed_ms":100,
                 "start":{"unix_ms":1000},"end":{"unix_ms":1100}}
        root={"id":"suite","valid_terminal_pair":True,"status":"fail","elapsed_ms":60,
              "start":{"scope":"suite","unix_ms":1020},"end":{}}
        report={"commands":[command],"e2e_spans":[root],"observed_child_commands":[command]}
        account=command_accounting(report)[0]
        self.assertEqual((account["command_wall_ms"],account["harness_wall_ms"],account["outside_harness_ms"]),(100,60,40))
        root["valid_terminal_pair"]=False
        self.assertIsNone(command_accounting(report)[0]["outside_harness_ms"])

    def test_malformed_or_orphan_spans_invalidate_stage_and_command_accounting(self):
        for broken in ("schema", "json", "orphan_suite", "missing_parent"):
            with self.subTest(broken=broken):
                traces=Traces()
                self.span(traces,"root","suite",None,0,100000)
                traces.rows[("e2e_spans","root")]["start"]["unix_ms"]=1000
                traces.rows[("commands","outer")]={
                    "start":{"scope":"command","unix_ms":990},
                    "end":{"scope":"command","unix_ms":1110,"elapsed_ms":120,"exit_code":0,"status":"pass"}}
                if broken == "schema":
                    traces.add('[sifr-e2e-event] '+json.dumps({"schema":2,"id":"stage","scope":"source_compile","parent":"root","event":"start","clock_us":0}))
                    traces.add('[sifr-e2e-event] '+json.dumps({"schema":1,"id":"stage","scope":"source_compile","parent":"root","event":"end","clock_us":50000,"elapsed_us":50000,"status":"pass"}))
                elif broken == "json":
                    traces.add('[sifr-e2e-event] broken-json')
                elif broken == "missing_parent":
                    traces.add('[sifr-e2e-event] '+json.dumps({"schema":1,"id":"stage","scope":"source_compile","parent":"absent","event":"end","clock_us":50000,"elapsed_us":50000,"status":"pass"}))
                else:
                    traces.add('[sifr-e2e-event] '+json.dumps({"schema":1,"id":"orphan","scope":"suite","event":"end","clock_us":50000,"elapsed_us":50000,"status":"pass"}))
                report=traces.report()
                if broken in ("schema", "json"):
                    self.assertFalse(report["e2e_accounting"][0]["accounting_valid"])
                    self.assertIsNone(report["e2e_accounting"][0]["unattributed_ms"])
                elif broken == "orphan_suite":
                    self.assertFalse(report["e2e_accounting"][1]["accounting_valid"])
                self.assertFalse(report["command_e2e_accounting"][0]["accounting_valid"])
                self.assertIsNone(report["command_e2e_accounting"][0]["outside_harness_ms"])

    def test_incomplete_invalid_and_unwound_spans_never_pass(self):
        traces=Traces()
        self.span(traces,"drop","source_compile",None,0,10,"incomplete")
        traces.add('[sifr-e2e-event] '+json.dumps({"schema":1,"id":"missing","scope":"suite","event":"start","clock_us":0}))
        traces.add('[sifr-e2e-event] '+json.dumps({"schema":1,"id":"orphan","scope":"suite","event":"end","clock_us":10,"elapsed_us":10,"status":"pass"}))
        traces.add('[sifr-e2e-event] '+json.dumps({'schema':1,'event':'start','id':'malformed','scope':'child','parent':'missing','clock_us':'bad'}))
        traces.add('[sifr-e2e-event] not-json')
        self.assertEqual({r["status"] for r in traces.report()["e2e_spans"]},{"incomplete"})
        self.assertTrue(traces.report()["parse_errors"])


def policy_checks():
    result=unittest.TextTestRunner(stream=io.StringIO()).run(unittest.defaultTestLoader.loadTestsFromTestCase(ProgressTests))
    if not result.wasSuccessful():raise AssertionError(str(result.errors+result.failures))


if __name__ == '__main__':
    unittest.main()
