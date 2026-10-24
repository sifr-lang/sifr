"""Progress for native commands, using the existing bounded process owner."""
import json
import time

from .batch_execution import execute_batch
from .command_progress import CommandProgress
from .paths import REPO_ROOT


def run_native_batch(commands, *, env, workers, completed, monitor):
    progress = {}
    deadline = env.get("SIFR_VERIFY_SAFETY_DEADLINE_SECONDS", "2400")
    def started(index):
        progress[index] = CommandProgress(commands[index], REPO_ROOT, env, deadline)
    def heartbeat():
        monitor()
        for receipt in progress.values():
            receipt.heartbeat(time.monotonic() - receipt.start)
    def finish(index, outcome):
        receipt = progress.pop(index, None)
        for stream in ("stdout", "stderr"):
            for line in getattr(outcome, stream).decode("utf-8", errors="replace").splitlines():
                print(f"[child:{stream}] " + json.dumps(line, ensure_ascii=True), flush=True)
        if receipt is not None:
            receipt.finish(outcome)
        completed(index, outcome)
    try:
        execute_batch(commands, cwd=REPO_ROOT, env=env, max_parallel=workers,
                      completed=finish, started=started, heartbeat=heartbeat,
                      deadline_seconds=deadline)
    except BaseException as error:
        for receipt in progress.values():
            receipt.error(error)
        raise
