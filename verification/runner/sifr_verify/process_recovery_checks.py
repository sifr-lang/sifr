"""Real failure injection for Linux descendant custody and reaping."""
from __future__ import annotations

import errno
import io
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import threading
import time
import unittest
from unittest.mock import patch

from . import process_execution
from .process_execution import execute


@unittest.skipUnless(sys.platform.startswith("linux"), "Linux subreaper contract")
class RecoveryTests(unittest.TestCase):
    def program(self, pidfile: Path, *, wait: bool) -> str:
        return (
            "import os,signal,time\nfrom pathlib import Path\n"
            "if os.fork() == 0:\n"
            "    os.setsid()\n"
            "    signal.signal(signal.SIGTERM, signal.SIG_IGN)\n"
            f"    Path({str(pidfile)!r}).write_text(str(os.getpid()))\n"
            "    time.sleep(60)\n"
            "else:\n"
            f"    while not Path({str(pidfile)!r}).exists(): time.sleep(.01)\n"
            "    print('ready', flush=True)\n"
            + ("    time.sleep(60)\n" if wait else "    raise SystemExit(7)\n")
        )

    def assert_reaped(self, pidfile: Path) -> None:
        pid = int(pidfile.read_text())
        self.assertFalse(Path(f"/proc/{pid}").exists(), "owned descendant survived or remained a zombie")

    def test_normal_exit_reaps_detached_term_ignoring_child(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "pid"
            result = execute([sys.executable, "-c", self.program(path, wait=False)], cwd=Path.cwd())
            self.assertEqual((result.returncode, result.cause), (7, "exit"))
            self.assertIn(b"ready", result.stdout)
            self.assert_reaped(path)

    def test_deadline_reaps_detached_term_ignoring_child(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "pid"
            result = execute([sys.executable, "-c", self.program(path, wait=True)],
                             cwd=Path.cwd(), deadline_seconds=.8)
            self.assertEqual((result.returncode, result.cause), (124, "safety_deadline"))
            self.assert_reaped(path)

    def test_cancellation_reaps_detached_child_and_retains_bytes(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "pid"
            program = (
                "from pathlib import Path\nimport sys,json\n"
                "from sifr_verify.process_execution import execute\n"
                f"r=execute([sys.executable,'-c',{self.program(path, wait=True)!r}],cwd=Path.cwd(),"
                "emit=lambda stream,data: print('observed',flush=True))\n"
                "print(json.dumps({'cause':r.cause,'stdout':r.stdout.decode()}),flush=True)\n"
            )
            proc = subprocess.Popen([sys.executable, "-c", program], stdout=subprocess.PIPE,
                                    text=True, env={**os.environ, "PYTHONPATH": str(Path(__file__).resolve().parents[1])})
            try:
                self.assertEqual(proc.stdout.readline().strip(), "observed")
                proc.send_signal(signal.SIGTERM)
                output, _ = proc.communicate(timeout=5)
                payload = json.loads(output)
                self.assertEqual(payload["cause"], "cancelled")
                self.assertIn("ready", payload["stdout"])
                self.assert_reaped(path)
            finally:
                if proc.poll() is None:
                    proc.kill()
                    proc.wait()
                proc.stdout.close()

    def test_custody_does_not_reap_unrelated_direct_child(self):
        other = subprocess.Popen([sys.executable, "-c", "import time; time.sleep(60)"])
        try:
            result = execute([sys.executable, "-c", "pass"], cwd=Path.cwd())
            self.assertEqual(result.returncode, 0)
            self.assertIsNone(other.poll())
            expired = execute([sys.executable, "-c", "import time; time.sleep(60)"],
                              cwd=Path.cwd(), deadline_seconds=.2)
            self.assertEqual(expired.cause, "safety_deadline")
            self.assertIsNone(other.poll())
        finally:
            other.terminate()
            other.wait(timeout=3)

    def test_missing_executable_remains_infrastructure_error(self):
        with self.assertRaises(OSError) as failure:
            execute(["/no-such-sifr-verification-command"], cwd=Path.cwd())
        self.assertEqual(failure.exception.errno, errno.ENOENT)

    def test_native_signal_status_is_preserved(self):
        result = execute([sys.executable, "-c", "import os,signal; os.kill(os.getpid(),signal.SIGTERM)"], cwd=Path.cwd())
        self.assertEqual((result.returncode, result.cause), (-signal.SIGTERM, "exit"))

    def test_killed_supervisor_cannot_authorize_success_or_pipe_wait(self):
        # The test has its own subreaper so its injected supervisor death never
        # delegates orphan reaping to the host's PID 1.
        script = (
            "import ctypes,errno,os,signal,sys,time\nfrom pathlib import Path\n"
            "from sifr_verify.process_execution import execute\n"
            "assert ctypes.CDLL(None).prctl(36,1,0,0,0)==0\n"
            "try:\n"
            " execute([sys.executable,'-c','import os,signal,time; os.kill(os.getppid(),signal.SIGKILL); time.sleep(60)'],cwd=Path.cwd())\n"
            "except OSError as error:\n"
            " assert error.errno==errno.EIO\n"
            "else:\n"
            " raise AssertionError('missing custody confirmation passed')\n"
            "os.waitpid(-1,0)\n"
            "print('rejected-and-reaped')\n"
        )
        result = subprocess.run([sys.executable, "-c", script], capture_output=True,
            env={**os.environ, "PYTHONPATH": str(Path(__file__).resolve().parents[1])}, timeout=5)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(b"rejected-and-reaped", result.stdout)

    def slow_start(self, *, cancel: bool) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            copied = root / "supervisor.py"
            text = Path(__file__).with_name("process_supervisor.py").read_text()
            copied.write_text(text.replace("import ctypes", "import time\ntime.sleep(.25)\nimport ctypes"))
            marker = root / "launched"
            original_spawn = subprocess.Popen
            def spawn(command, **kwargs):
                command = list(command)
                command[1] = str(copied)
                return original_spawn(command, **kwargs)
            timer = threading.Timer(.05, lambda: os.kill(os.getpid(), signal.SIGTERM)) if cancel else None
            try:
                if timer is not None:
                    timer.start()
                with patch.object(process_execution.subprocess, "Popen", side_effect=spawn):
                    result = execute([sys.executable, "-c", f"from pathlib import Path; Path({str(marker)!r}).touch()"],
                                     cwd=root, deadline_seconds=5 if cancel else .05)
                self.assertEqual((result.returncode, result.cause),
                                 (130, "cancelled") if cancel else (124, "safety_deadline"))
                self.assertFalse(marker.exists(), "command ran after startup cancellation")
            finally:
                if timer is not None:
                    timer.cancel()
                    timer.join()

    def test_cancellation_before_supervisor_handlers_are_installed(self):
        self.slow_start(cancel=True)

    def test_deadline_before_supervisor_handlers_are_installed(self):
        self.slow_start(cancel=False)

    def test_group_teardown_precedes_leader_reaping(self):
        observed = []
        original = os.killpg
        def checked(pid, sig):
            information = Path(f"/proc/{pid}/stat").read_text()
            observed.append(information[information.rindex(")") + 2:].split()[0])
            return original(pid, sig)
        with patch.object(process_execution.os, "killpg", side_effect=checked):
            result = execute([sys.executable, "-c", "pass"], cwd=Path.cwd())
        self.assertEqual(result.returncode, 0)
        self.assertEqual(observed, ["Z"], "leader PID was not reserved until group teardown")

    def test_missing_kernel_custody_rejects_before_command_spawn(self):
        for missing in ("subreaper", "pidfd"):
            with self.subTest(primitive=missing):
                program = (
                    "import ctypes,errno\nfrom unittest.mock import patch\n"
                    "from sifr_verify import process_supervisor as supervisor\n"
                    f"missing={missing!r}\n"
                    "if missing=='subreaper':\n"
                    " ctypes.set_errno(errno.ENOSYS)\n"
                    " hook=patch.object(supervisor.ctypes,'CDLL')\n"
                    "else:\n"
                    " hook=patch.object(supervisor.os,'pidfd_open',side_effect=OSError(errno.ENOSYS,'unavailable'))\n"
                    "with hook as fake, patch.object(supervisor.subprocess,'Popen',side_effect=AssertionError('spawned')) as spawn:\n"
                    " if missing=='subreaper': fake.return_value.prctl.return_value=-1\n"
                    " try: supervisor.supervise(['never-launch-this'])\n"
                    " except OSError as error: assert error.errno==errno.ENOSYS\n"
                    " else: raise AssertionError('unsupported custody passed')\n"
                    " spawn.assert_not_called()\n"
                )
                result = subprocess.run([sys.executable, "-c", program], capture_output=True,
                    env={**os.environ, "PYTHONPATH": str(Path(__file__).resolve().parents[1])}, timeout=5)
                self.assertEqual(result.returncode, 0, result.stderr)

    def test_unicode_infrastructure_detail_preserves_errno(self):
        with self.assertRaises(OSError) as failure:
            execute(["\U0001f30d" * 500], cwd=Path.cwd())
        self.assertEqual(failure.exception.errno, errno.ENAMETOOLONG)

    def test_killed_supervisor_with_live_escaped_pipe_holder_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "pid"
            command = (
                "import os,signal,time\nfrom pathlib import Path\n"
                "parent=os.getppid()\nos.setsid()\n"
                f"Path({str(path)!r}).write_text(str(os.getpid()))\n"
                "os.kill(parent,signal.SIGKILL)\ntime.sleep(60)\n"
            )
            program = (
                "import ctypes,errno,os,signal,sys\nfrom pathlib import Path\n"
                "from sifr_verify.process_execution import execute\n"
                "assert ctypes.CDLL(None).prctl(36,1,0,0,0)==0\n"
                "try:\n"
                f" execute([sys.executable,'-c',{command!r}],cwd=Path.cwd())\n"
                "except OSError as error:\n"
                " assert error.errno==errno.EIO\n"
                "else:\n"
                " raise AssertionError('escaped pipe holder passed')\n"
                f"pid=int(Path({str(path)!r}).read_text())\n"
                "try: os.kill(pid,signal.SIGKILL)\n"
                "finally: os.waitpid(pid,0)\n"
                "print('rejected-live-pipe-and-owned-test-orphan-reaped')\n"
            )
            result = subprocess.run([sys.executable, "-c", program], capture_output=True,
                env={**os.environ, "PYTHONPATH": str(Path(__file__).resolve().parents[1])}, timeout=5)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn(b"rejected-live-pipe", result.stdout)


def policy_checks():
    result = unittest.TextTestRunner(stream=io.StringIO()).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(RecoveryTests))
    if not result.wasSuccessful():
        raise AssertionError(str(result.errors + result.failures))


if __name__ == "__main__":
    unittest.main()
