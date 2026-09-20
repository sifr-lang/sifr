"""Atomic log, timing, and machine-report publication for profile runs."""

from __future__ import annotations

import argparse
import contextlib
import json
import resource
import shutil
import sys
import tempfile
import time
from pathlib import Path
from typing import Callable, TextIO

from . import reports
from .paths import REPO_ROOT


class Tee:
    """Write text to more than one stream."""

    def __init__(self, *streams: TextIO) -> None:
        self._streams = streams

    def write(self, data: str) -> int:
        for stream in self._streams:
            try:
                stream.write(data)
                stream.flush()
            except BrokenPipeError:
                # A detached terminal is only an observer; the log remains live.
                if stream is self._streams[-1]:
                    raise
        return len(data)

    def flush(self) -> None:
        for stream in self._streams:
            try:
                stream.flush()
            except BrokenPipeError:
                if stream is self._streams[-1]:
                    raise


def write_time_file(path: Path, *, start: float, usage_start: resource.struct_rusage) -> None:
    usage = resource.getrusage(resource.RUSAGE_CHILDREN)
    real_seconds = time.monotonic() - start
    user_seconds = max(0.0, usage.ru_utime - usage_start.ru_utime)
    sys_seconds = max(0.0, usage.ru_stime - usage_start.ru_stime)
    # Persist bytes, as the report parser and memory advisory require. Linux
    # rusage returns KiB; Darwin returns bytes. This is the maximum child RSS,
    # not a sum of concurrent children.
    max_rss = int(usage.ru_maxrss) * (1024 if sys.platform.startswith("linux") else 1)
    swaps = max(0, int(usage.ru_nswap - usage_start.ru_nswap))
    path.write_text(
        f"{real_seconds:.2f} real\n"
        f"{user_seconds:.2f} user\n"
        f"{sys_seconds:.2f} sys\n"
        f"{max_rss} maximum resident set size\n"
        f"{swaps} swaps\n",
        encoding="utf-8",
    )


def temporary_report_path(report_dir: Path, prefix: str) -> Path:
    with tempfile.NamedTemporaryFile(prefix=prefix, dir=report_dir, delete=False) as temp_file:
        return Path(temp_file.name)


def run_profile_with_report(
    profile_name: str,
    run_lane: Callable[[], int],
    *,
    handled_error: type[Exception],
    release_report_out: str | None,
    execution_outcomes: Callable[[], dict[str, int]] | None = None,
) -> int:
    release_output = None
    if release_report_out is not None:
        from .release_evidence import prepare_release_report_output

        try:
            release_output = prepare_release_report_output(
                release_report_out,
                profile_name=profile_name,
            )
        except ValueError as exc:
            print(f"sifr_verify: {exc}", file=sys.stderr)
            return 2
    report_dir = REPO_ROOT / "target" / "validation_lane_reports"
    report_dir.mkdir(parents=True, exist_ok=True)
    temp_log = temporary_report_path(report_dir, f"lane.{profile_name}.log.")
    temp_time = temporary_report_path(report_dir, f"lane.{profile_name}.time.")
    latest_log = report_dir / f"{profile_name}.latest.log"
    latest_time = report_dir / f"{profile_name}.latest.time"
    json_file = report_dir / f"{profile_name}.latest.json"
    start = time.monotonic()
    usage_start = resource.getrusage(resource.RUSAGE_CHILDREN)
    status = 0
    live_status = report_dir / f"{profile_name}.status.json"
    def publish_status(state: str, code: int | None) -> None:
        temporary = temporary_report_path(report_dir, f"{profile_name}.status.")
        temporary.write_text(json.dumps({"state": state, "exit_status": code, "log": str(latest_log if state == "completed" else temp_log)}))
        temporary.replace(live_status)
    publish_status("running", None)

    with temp_log.open("w", encoding="utf-8") as log_file:
        tee = Tee(sys.stdout, log_file)
        with contextlib.redirect_stdout(tee), contextlib.redirect_stderr(tee):
            try:
                status = run_lane()
            except handled_error as exc:
                print(f"sifr_verify: {exc}", file=sys.stderr)
                status = 2

    write_time_file(temp_time, start=start, usage_start=usage_start)
    shutil.copyfile(temp_log, latest_log)
    shutil.copyfile(temp_time, latest_time)
    json_file.unlink(missing_ok=True)
    try:
        reports.summarize(
            argparse.Namespace(
                profile=profile_name,
                log=str(latest_log),
                time_file=str(latest_time),
                json_out=str(json_file),
            )
        )
    except BrokenPipeError:
        pass  # JSON is written before rendering the optional terminal summary.
    except Exception as exc:  # Preserve the original failing execution status.
        status = status or 2
        print(f"warning: lane report summarization failed: {exc}", file=sys.stderr)
    if release_output is not None and status == 0:
        from .release_evidence import write_release_profile_report

        try:
            write_release_profile_report(
                release_output,
                log_path=latest_log,
                status=status,
            )
        except ValueError as exc:
            print(f"sifr_verify: {exc}", file=sys.stderr)
            status = 2
    # Bind functional status to the runner result, never to arbitrary child text.
    if json_file.exists():
        payload = json.loads(json_file.read_text())
        outcomes = execution_outcomes() if execution_outcomes is not None else {
            "functional_exit_status": status, "performance_exit_status": 0,
        }
        payload.update(outcomes)
        payload["exit_status"] = status
        payload["functional_status"] = "pass" if outcomes["functional_exit_status"] == 0 else "fail"
        payload["performance_status"] = "pass" if outcomes["performance_exit_status"] == 0 else "fail"
        json_file.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
    publish_status("completed", status)
    temp_log.unlink(missing_ok=True)
    temp_time.unlink(missing_ok=True)
    return status
