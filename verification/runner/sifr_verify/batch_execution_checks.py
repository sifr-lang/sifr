"""Real-process controls for bounded independent verification execution."""
from pathlib import Path
import json
import os
import signal
import subprocess
import sys
import tempfile
import threading
import time
import unittest
from unittest.mock import patch

from . import batch_execution as batch
from .process_execution import SAFETY_DEADLINE_ENV


def command(program):
    return [sys.executable, "-c", program]


def wait_file(path):
    end = time.monotonic() + 5
    while not path.exists() and time.monotonic() < end:
        time.sleep(.01)
    if not path.exists():
        raise AssertionError(f"child did not become ready: {path}")


class BatchExecutionTests(unittest.TestCase):
    def run_batch(self, commands, **kwargs):
        outcomes = {}
        batch.execute_batch(commands, cwd=Path.cwd(), max_parallel=kwargs.pop("max_parallel", 2),
                            completed=lambda index, result: outcomes.__setitem__(index, result), **kwargs)
        return outcomes

    def test_zero_invalid_limits_and_thread_reject_before_spawn(self):
        with patch.object(batch.subprocess, "Popen", side_effect=AssertionError("spawned")):
            for commands, kwargs in [([], {}), ([[]], {}), ([command("pass")], {"max_parallel": True}),
                                     ([command("pass")], {"limit_bytes": 0}),
                                     ([command("pass")], {"deadline_seconds": float("nan")})]:
                with self.assertRaises(ValueError):
                    self.run_batch(commands, **kwargs)
            failures = []
            def worker():
                try:
                    self.run_batch([command("pass")])
                except RuntimeError as error:
                    failures.append(str(error))
            thread = threading.Thread(target=worker)
            thread.start(); thread.join(2)
            self.assertFalse(thread.is_alive())
            self.assertEqual(failures, ["verification subprocesses require the main thread"])

    def test_completion_order_is_indexed_and_independent_failure_continues(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            release = root / "release"
            first = f"import time\nfrom pathlib import Path\np=Path({str(release)!r})\nwhile not p.exists():time.sleep(.01)\nprint('first')"
            rows = []
            def completed(index, result):
                rows.append((index, result))
                if index == 1:
                    release.touch()
            batch.execute_batch([command(first), command("print('second'); raise SystemExit(9)"),
                                 command("print('third')")], cwd=root, max_parallel=2, completed=completed)
            self.assertEqual(rows[0][0], 1)
            by_index = dict(rows)
            self.assertEqual(set(by_index), {0, 1, 2})
            self.assertEqual(by_index[1].returncode, 9)
            self.assertEqual(by_index[0].stdout, b"first\n")
            self.assertEqual(by_index[2].stdout, b"third\n")

    def test_slots_bound_real_overlap(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            counter = root / "counts"
            counter.write_text('[0,0]')
            program = """import fcntl,json,time,sys
from pathlib import Path
root=Path(sys.argv[1]); index=int(sys.argv[2])
def change(delta):
 with (root/'lock').open('w') as lock:
  fcntl.flock(lock,fcntl.LOCK_EX)
  p=root/'counts';current,peak=json.loads(p.read_text());current+=delta
  p.write_text(json.dumps([current,max(current,peak)]))
change(1)
(root/str(index)).touch()
if index < 2:
 limit=time.monotonic()+5
 while not all((root/str(i)).exists() for i in (0,1)):
  assert time.monotonic()<limit
  time.sleep(.01)
time.sleep(.1)
change(-1)
"""
            rows = self.run_batch([[sys.executable, "-c", program, str(root), str(i)] for i in range(4)])
            self.assertTrue(all(row.returncode == 0 for row in rows.values()))
            self.assertEqual(json.loads(counter.read_text()), [0, 2])

    def test_output_bound_and_command_deadline_preserve_other_commands(self):
        rows = self.run_batch([command("import time;time.sleep(3)"),
                               command("print('x'*100)")], deadline_seconds=.2, limit_bytes=16)
        self.assertEqual((rows[0].returncode, rows[0].cause), (124, "safety_deadline"))
        self.assertEqual(rows[1].returncode, 0)
        self.assertTrue(rows[1].truncated)
        self.assertEqual(rows[1].stdout, b"x" * 16)

    def test_inherited_expired_deadline_never_spawns_queued_work(self):
        with patch.object(batch.subprocess, "Popen", side_effect=AssertionError("spawned")):
            rows = self.run_batch([command("pass")] * 3,
                                  env={SAFETY_DEADLINE_ENV: str(time.monotonic() - 1)})
        self.assertEqual(len(rows), 3)
        self.assertTrue(all(row.cause == "safety_deadline" and row.returncode == 124
                            for row in rows.values()))

    def test_normal_exit_cleans_pipe_owning_descendant(self):
        with tempfile.TemporaryDirectory() as directory:
            marker = Path(directory) / "escaped"
            program = ("import os,time\nfrom pathlib import Path\nif os.fork()==0:\n"
                       " time.sleep(.8)\n" + f" Path({str(marker)!r}).touch()\n"
                       "else:print('done',flush=True)\n")
            rows = self.run_batch([command(program)])
            self.assertEqual(rows[0].stdout, b"done\n")
            self.assertEqual(rows[0].returncode, 0)
            time.sleep(.9)
            self.assertFalse(marker.exists())

    def test_partial_spawn_failure_cleans_already_owned_process(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory);ready = root / "ready";marker = root / "escaped"
            program = ("import time\nfrom pathlib import Path\n"
                       f"Path({str(ready)!r}).touch()\ntime.sleep(.8)\nPath({str(marker)!r}).touch()")
            original = subprocess.Popen
            children = []
            def spawn(*args, **kwargs):
                if children:
                    wait_file(ready)
                    raise OSError("injected spawn failure")
                child = original(*args, **kwargs); children.append(child); return child
            with patch.object(batch.subprocess, "Popen", side_effect=spawn):
                with self.assertRaisesRegex(OSError, "injected spawn failure"):
                    self.run_batch([command(program), command("pass")])
            self.assertIsNotNone(children[0].poll())
            self.assertTrue(children[0].stdout.closed and children[0].stderr.closed)
            time.sleep(.9)
            self.assertFalse(marker.exists())

    def test_callback_error_cleans_all_remaining_processes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory);ready = root / "ready";marker = root / "escaped"
            slow = ("import time\nfrom pathlib import Path\n"
                    f"Path({str(ready)!r}).touch()\ntime.sleep(1)\nPath({str(marker)!r}).touch()")
            fast = ("import time\nfrom pathlib import Path\n"
                    f"while not Path({str(ready)!r}).exists():time.sleep(.01)")
            def completed(index, result):
                raise RuntimeError("injected callback failure")
            with self.assertRaisesRegex(RuntimeError, "injected callback failure"):
                batch.execute_batch([command(slow), command(fast)], cwd=root, max_parallel=2,
                                    completed=completed)
            time.sleep(1.1)
            self.assertFalse(marker.exists())

    def test_signal_registration_failure_restores_handlers_before_spawn(self):
        original = signal.signal
        before = signal.getsignal(signal.SIGINT)
        calls = 0
        def register(sig, handler):
            nonlocal calls
            calls += 1
            if calls == 2:
                raise RuntimeError("signal registration failed")
            return original(sig, handler)
        with patch.object(batch.signal, "signal", side_effect=register), \
             patch.object(batch.subprocess, "Popen", side_effect=AssertionError("spawned")):
            with self.assertRaisesRegex(RuntimeError, "signal registration failed"):
                self.run_batch([command("pass")])
        self.assertEqual(signal.getsignal(signal.SIGINT), before)



    def test_late_selector_registration_failure_cleans_every_child(self):
        original_selector = batch.selectors.DefaultSelector
        original_spawn = subprocess.Popen
        children = []
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            programs = [command("import time\nfrom pathlib import Path\n"
                        + f"Path({str(root / str(i))!r}).touch()\ntime.sleep(1)\n"
                        + f"Path({str(root / ('escaped' + str(i)))!r}).touch()") for i in range(2)]
            class BrokenSelector:
                def __init__(self): self.inner = original_selector(); self.calls = 0
                def register(self, *args):
                    self.calls += 1
                    if self.calls == 4:
                        wait_file(root / '0'); wait_file(root / '1')
                        raise RuntimeError('registration failed')
                    return self.inner.register(*args)
                def unregister(self, *args): return self.inner.unregister(*args)
                def close(self): self.inner.close()
            def spawn(*args, **kwargs):
                child = original_spawn(*args, **kwargs); children.append(child); return child
            with patch.object(batch.selectors, 'DefaultSelector', BrokenSelector), \
                 patch.object(batch.subprocess, 'Popen', side_effect=spawn):
                with self.assertRaisesRegex(RuntimeError, 'registration failed'):
                    self.run_batch(programs)
            self.assertEqual(len(children), 2)
            self.assertTrue(all(child.poll() is not None for child in children))
            self.assertTrue(all(child.stdout.closed and child.stderr.closed for child in children))
            time.sleep(1.1)
            self.assertFalse(list(root.glob('escaped*')))

    def test_cancellation_cleans_active_commands_and_does_not_start_queue(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            program = r"""from pathlib import Path
import json,os,sys,time
from sifr_verify.batch_execution import execute_batch
root=Path(sys.argv[1])
commands=[]
for i in range(2):
 source="import os,time\nfrom pathlib import Path\n"+f"Path({str(root/str(i))!r}).write_text(str(os.getpid()))\ntime.sleep(10)\nPath({str(root/('escaped'+str(i)))!r}).touch()"
 commands.append([sys.executable,'-c',source])
commands.append([sys.executable,'-c',f"from pathlib import Path;Path({str(root/'queued')!r}).touch()"])
rows=[]
execute_batch(commands,cwd=root,max_parallel=2,completed=lambda i,r:rows.append([i,r.returncode,r.cause]))
print(json.dumps(rows))
"""
            parent = subprocess.Popen([sys.executable, '-c', program, str(root)],
                                      stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
            try:
                wait_file(root / '0'); wait_file(root / '1')
                parent.send_signal(signal.SIGTERM)
                out, error = parent.communicate(timeout=5)
                self.assertEqual(parent.returncode, 0, error)
                rows = json.loads(out)
                self.assertEqual(sorted(rows), [[0, 130, 'cancelled'], [1, 130, 'cancelled'], [2, 130, 'cancelled']])
                self.assertFalse((root / 'queued').exists())
                self.assertFalse(list(root.glob('escaped*')))
            finally:
                if parent.poll() is None:
                    parent.terminate()
                    try: parent.wait(timeout=2)
                    except subprocess.TimeoutExpired:
                        os.killpg(parent.pid, signal.SIGKILL); parent.wait()
                for i in range(2):
                    ready = root / str(i)
                    if ready.exists():
                        try: os.killpg(int(ready.read_text()), signal.SIGKILL)
                        except ProcessLookupError: pass
                parent.stdout.close(); parent.stderr.close()


def policy_checks():
    result = unittest.TestResult()
    unittest.defaultTestLoader.loadTestsFromTestCase(BatchExecutionTests).run(result)
    if not result.wasSuccessful():
        raise AssertionError("bounded command ownership failures: " + repr(result.failures + result.errors))
