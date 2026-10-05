"""DX.3 real subprocess failure injection: R01-R03 and B10."""
import contextlib
import fcntl
import json
from unittest.mock import patch
import io
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import threading
import time
import unittest

from . import process_execution
from .process_execution import SAFETY_DEADLINE_ENV, execute
from .profile_commands import run_command, CommandFailed


class ProcessTests(unittest.TestCase):
    def _darwin_row(self, pid=991, state='S', parent=None, start='Mon Oct 5 00:00:00 2026'):
        return {'pid': pid, 'parent': os.getpid() if parent is None else parent, 'group': 991,
                'uid': os.geteuid(), 'ruid': os.getuid(), 'state': state, 'start': start}

    def _darwin_proc(self):
        proc = type('OwnedChild', (), {'pid': 991, 'returncode': None})()
        proc.waits = []
        def wait(timeout=None):
            proc.waits.append(timeout)
            proc.returncode = 0
            return 0
        proc.wait = wait
        proc.poll = lambda: self.fail('owned Darwin leader must not be polled/reaped')
        return proc

    def test_darwin_waitid_preserves_leader_and_rejects_foreign_result(self):
        proc = self._darwin_proc()
        result = type('WaitResult', (), {'si_pid': proc.pid})()
        with patch.object(process_execution.os, 'waitid', side_effect=[None, result]) as wait:
            self.assertFalse(process_execution._darwin_child_exited(proc))
            self.assertTrue(process_execution._darwin_child_exited(proc))
        self.assertEqual(wait.call_args.args, (os.P_PID, proc.pid, os.WEXITED | os.WNOHANG | os.WNOWAIT))
        self.assertIsNone(proc.returncode)
        result.si_pid = 992
        with patch.object(process_execution.os, 'waitid', return_value=result):
            with self.assertRaisesRegex(OSError, 'identity differs'):
                process_execution._darwin_child_exited(proc)

    def test_darwin_missing_waitid_refuses_before_spawn(self):
        with patch.object(process_execution.sys, 'platform', 'darwin'), \
             patch.object(process_execution.os, 'waitid', None), \
             patch.object(process_execution.subprocess, 'Popen') as spawn:
            with self.assertRaisesRegex(RuntimeError, 'waitid/WNOWAIT'):
                execute(['unused'], cwd=Path.cwd())
        spawn.assert_not_called()

    def test_darwin_dead_only_group_skips_signals_then_reaps_and_proves_absence(self):
        proc = self._darwin_proc(); group = process_execution._DarwinGroup(proc)
        with patch.object(process_execution, '_darwin_child_exited', return_value=True), \
             patch.object(process_execution, '_darwin_snapshot', side_effect=[{991: self._darwin_row(state='Z<')}, {}]), \
             patch.object(process_execution.os, 'killpg') as kill:
            group.cleanup(); group.cleanup()
        kill.assert_not_called()
        self.assertTrue(group.complete)
        self.assertTrue(group.reaped)
        self.assertEqual(len(proc.waits), 1)

    def test_darwin_live_child_requires_kill_before_reap(self):
        proc = self._darwin_proc(); group = process_execution._DarwinGroup(proc)
        live = {991: self._darwin_row(), 992: self._darwin_row(992, parent=991)}
        resistant = {991: self._darwin_row(state='Z'), 992: self._darwin_row(992, parent=1)}
        dead = {991: self._darwin_row(state='Z')}
        def signal_group(pid, number):
            self.assertIsNone(proc.returncode)
            self.assertEqual(pid, proc.pid)
        with patch.object(process_execution, '_darwin_child_exited', return_value=True), \
             patch.object(process_execution, '_darwin_snapshot', side_effect=[live, resistant, resistant, dead, {}]), \
             patch.object(process_execution.time, 'sleep'), \
             patch.object(process_execution.os, 'killpg', side_effect=signal_group) as kill:
            group.cleanup()
        self.assertEqual([call.args[1] for call in kill.call_args_list], [signal.SIGTERM, signal.SIGKILL])
        self.assertTrue(group.complete)

    def test_darwin_eperm_race_needs_dead_only_reap_and_absence(self):
        proc = self._darwin_proc(); group = process_execution._DarwinGroup(proc)
        permission = PermissionError(1, 'fixture EPERM')
        dead = {991: self._darwin_row(state='Z')}
        with patch.object(process_execution, '_darwin_child_exited', return_value=True), \
             patch.object(process_execution, '_darwin_snapshot', side_effect=[{991: self._darwin_row()}, dead, dead, {}]), \
             patch.object(process_execution.time, 'sleep'), \
             patch.object(process_execution.os, 'killpg', side_effect=permission):
            group.cleanup()
        self.assertTrue(group.complete)
        self.assertEqual(group.signal_errors, [permission])
        self.assertEqual(len(proc.waits), 1)

    def test_darwin_live_or_changed_identity_eperm_is_never_cleanup_success(self):
        for changed in (False, True):
            proc = self._darwin_proc(); group = process_execution._DarwinGroup(proc)
            initial = {991: self._darwin_row()}
            after = {991: self._darwin_row(state='Z' if changed else 'S')}
            if changed: after[991]['uid'] += 1
            permission = PermissionError(1, 'fixture EPERM')
            with self.subTest(changed=changed), \
                 patch.object(process_execution, '_darwin_child_exited', return_value=True), \
                 patch.object(process_execution, '_darwin_snapshot', side_effect=[initial, after]), \
                 patch.object(process_execution.os, 'killpg', side_effect=permission):
                with self.assertRaises(PermissionError) as failure: group.cleanup()
            self.assertIs(failure.exception, permission)
            self.assertFalse(group.complete)
            self.assertFalse(proc.waits)

    def test_darwin_post_reap_reuse_rejects_without_signalling(self):
        for replacement in ({991: self._darwin_row(start='Mon Oct 5 00:00:01 2026')},
                            {992: self._darwin_row(992, parent=1)}):
            proc = self._darwin_proc(); group = process_execution._DarwinGroup(proc)
            with patch.object(process_execution, '_darwin_child_exited', return_value=True), \
                 patch.object(process_execution, '_darwin_snapshot', side_effect=[{991: self._darwin_row(state='Z')}, replacement]), \
                 patch.object(process_execution.os, 'killpg') as kill:
                with self.assertRaisesRegex(RuntimeError, 'identity changed or unknown'): group.cleanup()
                with self.assertRaisesRegex(RuntimeError, 'after leader reaping'):
                    group.send(signal.SIGKILL, list(replacement.values()))
            kill.assert_not_called()
            self.assertTrue(group.reaped)
            self.assertFalse(group.complete)

    def test_darwin_cleanup_deadline_survives_failed_retry(self):
        proc = self._darwin_proc(); group = process_execution._DarwinGroup(proc)
        with patch.object(process_execution, '_darwin_child_exited', return_value=True), \
             patch.object(process_execution, '_darwin_snapshot', side_effect=ValueError('unsupported ps')) as probe:
            with self.assertRaisesRegex(ValueError, 'unsupported ps'): group.cleanup()
            deadline = group.deadline
            with patch.object(process_execution.time, 'monotonic', return_value=deadline+1):
                with self.assertRaisesRegex(TimeoutError, 'cleanup deadline'): group.cleanup()
        self.assertEqual(group.deadline, deadline)
        self.assertEqual(probe.call_count, 1)
        self.assertFalse(group.complete)

    def test_darwin_parser_rejects_malformed_duplicate_and_allows_system_pid_zero(self):
        raw = '991 1 991 501 501 Z< Mon Oct  5 00:00:00 2026\n'
        self.assertEqual(process_execution._darwin_process_rows(raw)[991]['state'], 'Z<')
        self.assertIn(0, process_execution._darwin_process_rows(raw+'0 0 0 0 0 S Mon Oct  5 00:00:00 2026\n'))
        for invalid in ('', 'unsupported ps format', raw+raw):
            with self.assertRaises(ValueError): process_execution._darwin_process_rows(invalid)

    def test_darwin_missing_unreaped_anchor_never_authorizes_signal_or_success(self):
        proc = self._darwin_proc(); group = process_execution._DarwinGroup(proc)
        with patch.object(process_execution, '_darwin_child_exited', return_value=True), \
             patch.object(process_execution, '_darwin_snapshot', return_value={}), \
             patch.object(process_execution.os, 'killpg') as kill:
            with self.assertRaisesRegex(RuntimeError, 'anchor unavailable'): group.cleanup()
        kill.assert_not_called()
        self.assertFalse(group.complete)
        self.assertFalse(proc.waits)

    def test_darwin_probe_original_error_survives_its_cleanup_failure(self):
        primary = RuntimeError('probe registration failed')
        class Probe:
            stdout = io.BytesIO(); stderr = io.BytesIO()
            def poll(self): return None
            def kill(self): raise PermissionError(1, 'probe cleanup failed')
        class BrokenSelector:
            def __enter__(self): return self
            def __exit__(self, *_): pass
            def register(self, *_): raise primary
        probe = Probe()
        with patch.object(process_execution.subprocess, 'Popen', return_value=probe), \
             patch.object(process_execution.selectors, 'SelectSelector', return_value=BrokenSelector()):
            with self.assertRaises(RuntimeError) as failure:
                process_execution._darwin_snapshot(time.monotonic()+5)
        self.assertIs(failure.exception, primary)
        self.assertTrue(any('probe cleanup failed' in note for note in primary.__notes__))
        self.assertTrue(probe.stdout.closed and probe.stderr.closed)

    def test_darwin_probe_registration_failure_reaps_its_own_child(self):
        original_spawn = subprocess.Popen; children = []
        def spawn(*args, **kwargs):
            child = original_spawn(*args, **kwargs); children.append(child); return child
        class BrokenSelector:
            def __enter__(self): return self
            def __exit__(self, *_): pass
            def register(self, *_): raise RuntimeError('probe register failed')
        with patch.object(process_execution.subprocess, 'Popen', side_effect=spawn), \
             patch.object(process_execution.selectors, 'SelectSelector', return_value=BrokenSelector()):
            with self.assertRaisesRegex(RuntimeError, 'probe register failed'):
                process_execution._darwin_snapshot(time.monotonic()+5)
        self.assertEqual(len(children), 1)
        self.assertIsNotNone(children[0].returncode)
        self.assertTrue(children[0].stdout.closed and children[0].stderr.closed)

    def test_darwin_probe_does_not_adopt_callers_handled_exception(self):
        for owned_failure in (False, True):
            caller = LookupError('unrelated caller error')
            primary = ValueError('owned probe error')
            cleanup = PermissionError(1, 'probe final wait failed')
            class Probe:
                def __init__(self):
                    self.stdout = io.BytesIO(); self.stderr = io.BytesIO(); self.waits = 0
                def poll(self): return 0
                def wait(self, **_):
                    self.waits += 1
                    if self.waits == 2: raise cleanup
                    return 0
            class EmptySelector:
                def __enter__(self): return self
                def __exit__(self, *_): pass
                def register(self, *_): pass
                def get_map(self): return {}
            probe = Probe()
            with self.subTest(owned_failure=owned_failure), \
                 patch.object(process_execution.subprocess, 'Popen', return_value=probe), \
                 patch.object(process_execution.selectors, 'SelectSelector', return_value=EmptySelector()), \
                 patch.object(process_execution, '_darwin_process_rows',
                              side_effect=primary if owned_failure else None, return_value={991: {}}):
                try:
                    raise caller
                except LookupError:
                    with self.assertRaises(type(primary if owned_failure else cleanup)) as failure:
                        process_execution._darwin_snapshot(time.monotonic()+5)
                self.assertIs(failure.exception, primary if owned_failure else cleanup)
                self.assertFalse(hasattr(caller, '__notes__'))
                if owned_failure:
                    self.assertTrue(any('probe final wait failed' in note for note in primary.__notes__))
                self.assertTrue(probe.stdout.closed and probe.stderr.closed)

    def test_darwin_execute_does_not_adopt_callers_handled_exception(self):
        for owned_failure in (False, True):
            caller = LookupError('unrelated caller error')
            primary = ValueError('owned signal setup failed')
            cleanup = PermissionError(1, 'signal restoration failed')
            effects = [None, primary, cleanup] if owned_failure else [None, None, cleanup]
            with self.subTest(owned_failure=owned_failure), \
                 patch.object(process_execution.sys, 'platform', 'darwin'), \
                 patch.object(process_execution.signal, 'signal', side_effect=effects), \
                 patch.object(process_execution.subprocess, 'Popen') as spawn:
                try:
                    raise caller
                except LookupError:
                    with self.assertRaises(type(primary if owned_failure else cleanup)) as failure:
                        execute(['unused'], cwd=Path.cwd(), env={SAFETY_DEADLINE_ENV: '1'})
                self.assertIs(failure.exception, primary if owned_failure else cleanup)
                self.assertFalse(hasattr(caller, '__notes__'))
                if owned_failure:
                    self.assertTrue(any('signal restoration failed' in note for note in primary.__notes__))
                spawn.assert_not_called()

    def test_darwin_probe_streaming_cap_is_aggregate_and_reaps_child(self):
        original_spawn = subprocess.Popen; children = []
        def spawn(*args, **kwargs):
            child = original_spawn(*args, **kwargs); children.append(child); return child
        command = [sys.executable, '-I', '-S', '-B', '-c', 'import os;os.write(1,b"x"*200);os.write(2,b"y"*200)']
        with patch.object(process_execution, '_DARWIN_PS', command), \
             patch.object(process_execution, '_DARWIN_PROBE_BYTES', 256), \
             patch.object(process_execution.subprocess, 'Popen', side_effect=spawn):
            with self.assertRaisesRegex(ValueError, 'probe output limit'):
                process_execution._darwin_snapshot(time.monotonic()+5)
        self.assertIsNotNone(children[0].returncode)
        self.assertTrue(children[0].stdout.closed and children[0].stderr.closed)

    def test_darwin_original_error_survives_failed_finally_cleanup(self):
        original_spawn = subprocess.Popen; children = []
        def spawn(*args, **kwargs):
            child = original_spawn(*args, **kwargs); children.append(child); return child
        primary = RuntimeError('original selector failure')
        try:
            with patch.object(process_execution.sys, 'platform', 'darwin'), \
                 patch.object(process_execution.subprocess, 'Popen', side_effect=spawn), \
                 patch.object(process_execution.selectors, 'DefaultSelector', side_effect=primary), \
                 patch.object(process_execution._DarwinGroup, 'cleanup', side_effect=PermissionError(1, 'cleanup failed')):
                with self.assertRaises(RuntimeError) as failure:
                    execute([sys.executable, '-I', '-S', '-B', '-c', 'pass'], cwd=Path.cwd())
            self.assertIs(failure.exception, primary)
            self.assertTrue(any('cleanup failed' in note for note in primary.__notes__))
        finally:
            for child in children:
                if child.poll() is None: child.kill()
                child.wait(timeout=2)

    def test_darwin_failed_first_cleanup_is_retried_without_masking_original(self):
        original_spawn = subprocess.Popen; children = []
        def spawn(*args, **kwargs):
            child = original_spawn(*args, **kwargs); children.append(child); return child
        primary = PermissionError(1, 'original cleanup failure')
        try:
            with patch.object(process_execution.sys, 'platform', 'darwin'), \
                 patch.object(process_execution.subprocess, 'Popen', side_effect=spawn), \
                 patch.object(process_execution._DarwinGroup, 'cleanup', side_effect=[primary, RuntimeError('retry failed')]) as cleanup:
                with self.assertRaises(PermissionError) as failure:
                    execute([sys.executable, '-I', '-S', '-B', '-c', 'pass'], cwd=Path.cwd())
            self.assertIs(failure.exception, primary)
            self.assertEqual(cleanup.call_count, 2)
            self.assertTrue(any('retry failed' in note for note in primary.__notes__))
        finally:
            for child in children:
                if child.poll() is None: child.kill()
                child.wait(timeout=2)

    def test_darwin_algorithm_real_posix_child_preserves_output_and_exit(self):
        # This Linux-hosted mechanism control is not a Darwin kernel claim.
        with patch.object(process_execution.sys, 'platform', 'darwin'):
            result = execute([sys.executable, '-I', '-S', '-B', '-c',
                              'import sys; print("owned output"); sys.exit(3)'], cwd=Path.cwd(), deadline_seconds=2)
        self.assertEqual((result.returncode, result.cause, result.stdout), (3, 'exit', b'owned output\n'))
        self.assertFalse(result.truncated)

    def test_f26_worker_thread_is_rejected_before_spawn(self):
        failures = []
        def worker():
            try:
                execute([sys.executable, "-c", "pass"], cwd=Path.cwd())
            except RuntimeError as error:
                failures.append(str(error))
        with patch.object(process_execution.subprocess, "Popen", side_effect=AssertionError("spawned")) as spawn:
            thread = threading.Thread(target=worker)
            thread.start()
            thread.join(timeout=2)
        self.assertFalse(thread.is_alive())
        spawn.assert_not_called()
        self.assertEqual(failures, ["verification subprocesses require the main thread"])

    def test_f26_signal_setup_failure_is_before_spawn(self):
        original_signal = signal.signal
        original_interrupt = signal.getsignal(signal.SIGINT)
        calls = 0
        def fail_second_registration(sig, handler):
            nonlocal calls
            calls += 1
            if calls == 2:
                raise RuntimeError("signal setup failed")
            return original_signal(sig, handler)
        with patch.object(process_execution.signal, "signal", side_effect=fail_second_registration), \
             patch.object(process_execution.subprocess, "Popen", side_effect=AssertionError("spawned")) as spawn:
            with self.assertRaisesRegex(RuntimeError, "signal setup failed"):
                execute([sys.executable, "-c", "pass"], cwd=Path.cwd())
        spawn.assert_not_called()
        self.assertEqual(signal.getsignal(signal.SIGINT), original_interrupt)

    def _assert_f26_error_kills_descendants(self, failure_point):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            ready = root / "ready"
            marker = root / "escaped"
            program = (
                "import os,time\nfrom pathlib import Path\n"
                "if os.fork() == 0:\n"
                f"    Path({str(ready)!r}).write_text('ready')\n"
                "    time.sleep(0.8)\n"
                f"    Path({str(marker)!r}).write_text('escaped')\n"
                "else:\n"
                "    print('ready', flush=True)\n"
                "    time.sleep(5)\n"
            )
            original_spawn = subprocess.Popen
            spawned = []
            def record_spawn(*args, **kwargs):
                proc = original_spawn(*args, **kwargs)
                spawned.append(proc)
                return proc
            def fail_after_descendant(*_args):
                limit = time.monotonic() + 2
                while not ready.exists() and time.monotonic() < limit:
                    time.sleep(0.01)
                self.assertTrue(ready.exists(), "descendant did not start")
                raise RuntimeError("injected setup or callback failure")
            class BrokenSelector:
                def __init__(self, fail_on):
                    self.registered = 0
                    self.fail_on = fail_on
                    self.closed = False
                def register(self, *_args):
                    self.registered += 1
                    if self.registered == self.fail_on:
                        fail_after_descendant()
                def close(self):
                    self.closed = True
            broken_selector = BrokenSelector(2 if failure_point == "selector_second_register" else 1)
            with patch.object(process_execution.subprocess, "Popen", side_effect=record_spawn):
                if failure_point == "selector_create":
                    selector = patch.object(process_execution.selectors, "DefaultSelector",
                                            side_effect=fail_after_descendant)
                elif failure_point in ("selector_register", "selector_second_register"):
                    selector = patch.object(process_execution.selectors, "DefaultSelector",
                                            return_value=broken_selector)
                else:
                    selector = contextlib.nullcontext()
                with selector:
                    with self.assertRaisesRegex(RuntimeError, "injected setup or callback failure"):
                        execute([sys.executable, "-c", program], cwd=Path.cwd(),
                                emit=fail_after_descendant if failure_point == "callback" else None)
            self.assertEqual(len(spawned), 1)
            self.assertIsNotNone(spawned[0].poll(), "direct child survived")
            self.assertTrue(spawned[0].stdout.closed)
            self.assertTrue(spawned[0].stderr.closed)
            if failure_point in ("selector_register", "selector_second_register"):
                self.assertTrue(broken_selector.closed)
            time.sleep(1)
            self.assertFalse(marker.exists(), "descendant survived")

    def test_f26_selector_creation_failure_cleans_group(self):
        self._assert_f26_error_kills_descendants("selector_create")

    def test_f26_selector_registration_failure_cleans_group(self):
        self._assert_f26_error_kills_descendants("selector_register")

    def test_f26_second_selector_registration_failure_cleans_group(self):
        self._assert_f26_error_kills_descendants("selector_second_register")

    def test_f26_callback_failure_cleans_group(self):
        self._assert_f26_error_kills_descendants("callback")

    def test_f26_normal_exit_retains_output_and_cleans_descendant(self):
        with tempfile.TemporaryDirectory() as temporary:
            marker = Path(temporary) / "escaped"
            program = (
                "import os,time\nfrom pathlib import Path\n"
                "if os.fork() == 0:\n"
                "    time.sleep(0.8)\n"
                f"    Path({str(marker)!r}).write_text('escaped')\n"
                "else:\n"
                "    os.write(1,b'terminal output')\n"
            )
            outcome = execute([sys.executable, "-c", program], cwd=Path.cwd())
            self.assertEqual((outcome.returncode, outcome.cause), (0, "exit"))
            self.assertEqual(outcome.stdout, b"terminal output")
            time.sleep(1)
            self.assertFalse(marker.exists())

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

    def test_dx15_profile_rss_roundtrip_preserves_linux_and_darwin_bytes(self):
        from types import SimpleNamespace
        from . import profile_reporting
        from .reports import parse_time_file
        expected = 7 * 1024**3
        initial = SimpleNamespace(ru_utime=0, ru_stime=0, ru_nswap=0)
        for host, native_rss in (("linux", expected // 1024), ("darwin", expected)):
            with self.subTest(host=host), tempfile.TemporaryDirectory() as directory:
                usage = SimpleNamespace(ru_utime=1, ru_stime=2, ru_nswap=0,
                                        ru_maxrss=native_rss)
                path = Path(directory) / "time"
                with patch.object(profile_reporting.sys, "platform", host), \
                     patch.object(profile_reporting.resource, "getrusage", return_value=usage):
                    profile_reporting.write_time_file(path, start=time.monotonic(),
                                                      usage_start=initial)
                self.assertEqual(parse_time_file(path)["max_rss_bytes"], expected)

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

    def test_f27_native_exit_124_is_distinct_from_deadline_and_cancellation(self):
        native = execute([sys.executable, "-c", "raise SystemExit(124)"], cwd=Path.cwd())
        self.assertEqual((native.returncode, native.cause), (124, "exit"))
        expired = execute([sys.executable, "-c", "import time; time.sleep(10)"],
                          cwd=Path.cwd(), deadline_seconds=.1)
        self.assertEqual((expired.returncode, expired.cause), (124, "safety_deadline"))
        self.assertLess(expired.elapsed_seconds, 1)

    def test_f27_deadline_construction_keeps_the_tighter_inherited_bound(self):
        for inherited, expected in ((None, 100.5), ("100.25", 100.25),
                                    ("101", 100.5), ("99", 99)):
            with self.subTest(inherited=inherited):
                env = {"fixture": "preserved"}
                if inherited is not None:
                    env[SAFETY_DEADLINE_ENV] = inherited
                original = env.copy()
                with patch.object(process_execution.time, "monotonic", return_value=100):
                    child_env, deadline = process_execution.deadline_environment(env, .5)
                self.assertEqual(deadline, expected)
                self.assertEqual(float(child_env[SAFETY_DEADLINE_ENV]), expected)
                self.assertEqual(child_env["fixture"], "preserved")
                self.assertEqual(env, original)

    def test_f27_invalid_deadlines_rejected_before_spawn(self):
        invalid = ("0", "-1", "nan", "inf", "-inf", "bad", True)
        with patch.object(process_execution.subprocess, "Popen", side_effect=AssertionError("spawned")) as spawn:
            for value in invalid:
                with self.subTest(value=value), self.assertRaises(ValueError):
                    execute([sys.executable, "-c", "pass"], cwd=Path.cwd(),
                            deadline_seconds=value)
                with self.subTest(absolute=value), self.assertRaises(ValueError):
                    execute([sys.executable, "-c", "pass"], cwd=Path.cwd(),
                            env={SAFETY_DEADLINE_ENV: value})
        spawn.assert_not_called()

    def test_f27_absolute_deadline_bounds_standalone_helper(self):
        with tempfile.TemporaryDirectory() as temporary:
            env = os.environ.copy()
            env["PYTHONPATH"] = str(Path(__file__).resolve().parents[1])
            env[SAFETY_DEADLINE_ENV] = repr(time.monotonic() + .35)
            script = (
                "from pathlib import Path; import sys; "
                "from sifr_verify.process_execution import execute; "
                "r=execute([sys.executable,'-c','import time; time.sleep(10)'],"
                "cwd=Path.cwd(),deadline_seconds=30); "
                "print(r.cause, flush=True)"
            )
            started = time.monotonic()
            result = subprocess.run([sys.executable, "-c", script], cwd=temporary,
                                    env=env, capture_output=True, timeout=2)
            self.assertLess(time.monotonic() - started, 1.5)
            self.assertEqual(result.returncode, 0)
            self.assertIn(b"safety_deadline", result.stdout)

    def test_f27_standalone_metadata_helper_honors_expired_deadline(self):
        env = os.environ.copy()
        env["PYTHONPATH"] = str(Path(__file__).resolve().parents[1])
        env[SAFETY_DEADLINE_ENV] = repr(time.monotonic() - 1)
        result = subprocess.run(
            [sys.executable, "-m", "sifr_verify.metadata_setup", "--", "cargo", "build"],
            cwd=Path.cwd(), env=env, capture_output=True, timeout=2,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(b"exit code 124 cause=safety_deadline", result.stderr)

    def test_f27_per_process_default_does_not_limit_whole_step(self):
        from .profile_runner import ProfileRunner
        from .step_budgets import StepBudgetContext
        inherited_deadline = os.environ.get(SAFETY_DEADLINE_ENV)
        runner = ProfileRunner("create-pr", [])
        runner.env["SIFR_VERIFY_SAFETY_DEADLINE_SECONDS"] = ".6"
        runner.prepare_step_budget = lambda name: StepBudgetContext(name, 1000, "advisory")
        def step():
            for _ in range(2):
                run_command([sys.executable, "-c", "import time; time.sleep(.35)"],
                            env=runner.env)
        with contextlib.redirect_stdout(io.StringIO()):
            status = runner.execute_step("fixture", step)
        self.assertEqual(status, 0)
        self.assertEqual(runner.env.get(SAFETY_DEADLINE_ENV), inherited_deadline)

    def test_f27_step_deadline_covers_successive_commands(self):
        from .profile_runner import ProfileRunner
        from .step_budgets import StepBudgetContext
        inherited_deadline = os.environ.get(SAFETY_DEADLINE_ENV)
        runner = ProfileRunner("create-pr", [])
        runner.env["SIFR_VERIFY_STEP_SAFETY_DEADLINE_SECONDS"] = ".3"
        runner.prepare_step_budget = lambda name: StepBudgetContext(name, 1000, "advisory")
        def step():
            self.assertEqual(os.environ[SAFETY_DEADLINE_ENV], runner.env[SAFETY_DEADLINE_ENV])
            run_command([sys.executable, "-c", "import time; time.sleep(.12)"],
                        env=runner.env)
            run_command([sys.executable, "-c", "import time; time.sleep(10)"],
                        env=os.environ.copy())
        with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            status = runner.execute_step("fixture", step)
        self.assertEqual(status, 124)
        self.assertEqual(runner.env.get(SAFETY_DEADLINE_ENV), inherited_deadline)
        self.assertEqual(os.environ.get(SAFETY_DEADLINE_ENV), inherited_deadline)

    def test_f27_escaped_pipe_holder_cannot_extend_deadline(self):
        with tempfile.TemporaryDirectory() as temporary:
            program = (
                "import os,time; "
                "child=os.fork(); "
                "os.setsid() if child==0 else None; "
                "time.sleep(1) if child==0 else os.write(1,b'terminal output')"
            )
            started = time.monotonic()
            result = execute([sys.executable, "-c", program], cwd=Path(temporary),
                             deadline_seconds=.2)
            self.assertEqual((result.returncode, result.cause), (124, "safety_deadline"))
            self.assertEqual(result.stdout, b"terminal output")
            self.assertLess(time.monotonic() - started, .8)

    def test_f27_lock_wait_uses_absolute_deadline(self):
        with tempfile.TemporaryDirectory() as temporary:
            lock = Path(temporary) / "held.lock"
            with lock.open("w") as owner:
                fcntl.flock(owner, fcntl.LOCK_EX)
                program = (
                    "import fcntl,sys; "
                    "f=open(sys.argv[1],'w'); "
                    "fcntl.flock(f, fcntl.LOCK_EX); print('acquired',flush=True)"
                )
                result = execute([sys.executable, "-c", program, str(lock)],
                                 cwd=Path.cwd(), deadline_seconds=.2)
            self.assertEqual((result.returncode, result.cause), (124, "safety_deadline"))
            self.assertNotIn(b"acquired", result.stdout)
            self.assertLess(result.elapsed_seconds, 1)

    def _f27_resistant_tree(self, *, foreground_wait: bool, deadline_seconds: float):
        with tempfile.TemporaryDirectory() as temporary:
            marker = Path(temporary) / "escaped"
            program = (
                "import os,signal,time; from pathlib import Path; "
                "signal.signal(signal.SIGTERM,signal.SIG_IGN); "
                "child=os.fork(); "
                "time.sleep(2) if child==0 else os.write(1,b'terminal output'); "
                f"Path({str(marker)!r}).write_text('escaped') if child==0 else None; "
                + ("time.sleep(10) if child!=0 else None" if foreground_wait else "")
            )
            failure = None
            try:
                started = time.monotonic()
                result = execute([sys.executable, "-c", program], cwd=Path.cwd(),
                                 deadline_seconds=deadline_seconds)
                elapsed = time.monotonic() - started
                self.assertEqual((result.returncode, result.cause),
                                 (124, "safety_deadline") if foreground_wait else (0, "exit"))
                self.assertEqual(result.stdout, b"terminal output")
                self.assertLess(elapsed, 1.5)
                if foreground_wait:
                    self.assertGreaterEqual(result.elapsed_seconds, .5)
            except BaseException as error:
                failure = error
                raise
            finally:
                try:
                    time.sleep(2)
                    self.assertFalse(marker.exists())
                except BaseException as marker_error:
                    if failure is None:
                        raise
                    failure.add_note(f"F27 late marker check also failed: {marker_error!r}")
                else:
                    if failure is not None:
                        failure.add_note("F27 late marker check passed: no escaped marker after two seconds")

    def test_f27_natural_exit_preserves_status_and_reaps_resistant_pipe_holder(self):
        # Cleanup is part of supervisor completion; use the existing total
        # completion bound instead of imposing a half-second throughput claim.
        self._f27_resistant_tree(foreground_wait=False, deadline_seconds=1.5)

    def test_f27_deadline_stops_live_resistant_tree_and_preserves_output(self):
        self._f27_resistant_tree(foreground_wait=True, deadline_seconds=.5)

    def test_stdin_large_and_empty_are_delivered_without_pipe_deadlock(self):
        for data in (b"", bytes(range(256)) * 8192):
            result = execute([sys.executable, "-c",
                "import sys; sys.stdout.buffer.write(sys.stdin.buffer.read())"],
                cwd=Path.cwd(), input_bytes=data, limit_bytes=3 * 1024 * 1024)
            self.assertEqual(result.returncode, 0)
            self.assertFalse(result.truncated)
            self.assertEqual(result.stdout, data)

    def test_audit_inventory_allows_only_regular_generated_hint(self):
        from . import audit_fixtures
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture = root / "case.sifr"
            fixture.write_text("def main():\n    pass\n")
            manifest = {"schema_version": 1, "area": "fixture", "fixture_root": ".",
                        "entries": [{"id": "case", "path": "case.sifr", "category": "test",
                                     "command": "check", "smoke": True, "expect_exit_code": 0}]}
            with patch.object(audit_fixtures, "REPO_ROOT", root):
                check = lambda: audit_fixtures.validate_manifest(root / "manifest.json", manifest, area="fixture")
                hint = root / ".sifrbuildinfo"
                hint.write_text("{}")
                self.assertEqual(check(), [])
                extra = root / "untracked.txt"
                extra.write_text("unexpected")
                self.assertTrue(any("non-fixture file" in failure for failure in check()))
                extra.unlink()
                hint.unlink()
                hint.symlink_to("missing")
                self.assertTrue(any("non-fixture file" in failure for failure in check()))
                hint.unlink()
                (root / "missing.sifr").write_text("def main():\n    pass\n")
                self.assertTrue(any("fixture missing from manifest" in failure for failure in check()))

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
