"""Inner native timing probe; the outer canonical supervisor owns this tree.

The launcher interval wraps GNU Time and the native program. It excludes Python
probe startup and supervisor startup, without subtracting measured overhead.
"""
import json
import os
from pathlib import Path
import resource
import subprocess
import sys
import time


def main():
    timer, binary, raw_output = sys.argv[1:]
    output = Path(raw_output)
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    os.umask(0o077)
    def bounds():
        resource.setrlimit(resource.RLIMIT_FSIZE, (4*1024**2, 4*1024**2))
    metrics = output / 'timer.txt'
    command = [timer, '-o', str(metrics), '-f',
               'wall_seconds=%e\nuser_seconds=%U\nsystem_seconds=%S\npeak_rss_kib=%M\nexit=%x', binary]
    with (output/'stdout').open('wb') as stdout, (output/'stderr').open('wb') as stderr:
        started = time.perf_counter_ns()
        proc = subprocess.Popen(command, stdout=stdout, stderr=stderr, cwd=output, preexec_fn=bounds)
        code = proc.wait()
        elapsed = time.perf_counter_ns()-started
    print(json.dumps({'timed_command_exit':code,'launcher_elapsed_ns':elapsed}),flush=True)


if __name__ == '__main__':
    main()
