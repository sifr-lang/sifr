"""Atomic log, timing, and machine-report publication for profile runs."""

from __future__ import annotations

import argparse
import contextlib
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
            stream.write(data)
            stream.flush()
        return len(data)

    def flush(self) -> None:
        for stream in self._streams:
            stream.flush()


def write_time_file(path: Path, *, start: float, usage_start: resource.struct_rusage) -> None:
    usage = resource.getrusage(resource.RUSAGE_CHILDREN)
    real_seconds = time.monotonic() - start
    user_seconds = max(0.0, usage.ru_utime - usage_start.ru_utime)
    sys_seconds = max(0.0, usage.ru_stime - usage_start.ru_stime)
    max_rss = int(usage.ru_maxrss)
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
    try:
        reports.summarize(
            argparse.Namespace(
                profile=profile_name,
                log=str(latest_log),
                time_file=str(latest_time),
                json_out=str(json_file),
            )
        )
    except Exception as exc:  # Preserve validation status while surfacing report regressions.
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
    temp_log.unlink(missing_ok=True)
    temp_time.unlink(missing_ok=True)
    return status
