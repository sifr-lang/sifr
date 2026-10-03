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
import time
import unittest

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


def policy_checks():
    result = unittest.TextTestRunner(stream=io.StringIO()).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(RecoveryTests))
    if not result.wasSuccessful():
        raise AssertionError(str(result.errors + result.failures))


if __name__ == "__main__":
    unittest.main()
