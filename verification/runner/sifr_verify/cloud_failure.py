"""Classify observed cloud failures without converting assertions into passes."""
from __future__ import annotations

import errno


def oom_kills(diagnostics: dict) -> int:
    # Ancestor counters can describe the same event: compare the maximum rather
    # than adding overlapping hierarchies.
    counts = []
    for name, text in diagnostics.items():
        if name.endswith("/memory.events"):
            rows = dict(line.split() for line in text.splitlines())
            counts.append(int(rows.get("oom_kill", "0")))
    return max(counts, default=0)


def classify_failure(error: BaseException, before: dict, after: dict) -> str:
    if isinstance(error, KeyboardInterrupt):
        return "cancelled"
    explicit = getattr(error, "classification", None)
    if explicit:
        return explicit
    cause = getattr(error, "cause", None)
    if cause in {"safety_deadline", "cancelled"}:
        return {"safety_deadline": "timeout", "cancelled": "cancelled"}[cause]
    if isinstance(error, OSError):
        return "enospc" if error.errno == errno.ENOSPC else "unavailable"
    outcome = getattr(error, "outcome", None)
    stderr = getattr(outcome, "stderr", b"").lower()
    if b"no space left on device" in stderr:
        return "enospc"
    killed = getattr(error, "returncode", 0) in {-9, 137} or b"signal: 9" in stderr or b"sigkill" in stderr
    if killed and oom_kills(after) > oom_kills(before):
        return "oom"
    return "assertion"
