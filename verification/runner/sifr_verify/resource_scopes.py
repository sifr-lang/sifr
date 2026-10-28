"""Inclusive CPU accounting at serial, process-owned verification boundaries.

A process must not measure overlapping siblings with these cumulative counters.
Parallel areas measure inside their own workers; native batches are measured as
one configuration. Values include self and descendants reaped by their parents,
not abandoned/unwaited processes. Nested scopes overlap and must not be summed.
"""
from contextlib import contextmanager
from contextvars import ContextVar
import json
import math
import os
import resource
import threading
import time
import uuid

PREFIX = "[sifr-resource-scope] "
_parent = ContextVar("verification_resource_scope", default=None)


def counters():
    own = resource.getrusage(resource.RUSAGE_SELF)
    children = resource.getrusage(resource.RUSAGE_CHILDREN)
    return own.ru_utime + children.ru_utime, own.ru_stime + children.ru_stime


@contextmanager
def cpu_scope(kind, name):
    if threading.current_thread() is not threading.main_thread():
        raise RuntimeError("CPU scopes require a serial main-thread owner")
    identity = uuid.uuid4().hex
    parent = _parent.get()
    before = counters()
    started = time.monotonic()
    token = _parent.set(identity)
    completion = "raised"
    try:
        yield
        completion = "returned"
    finally:
        _parent.reset(token)
        after = counters()
        user, system = (after[i] - before[i] for i in (0, 1))
        row = {
            "schema": 1, "id": identity, "parent_id": parent, "pid": os.getpid(),
            "kind": kind, "name": name, "completion": completion,
            "elapsed_seconds": time.monotonic() - started,
            "user_cpu_seconds": user, "system_cpu_seconds": system,
            "cpu_seconds": user + system,
            "accounting": "inclusive-self-and-reaped-descendants",
        }
        print(PREFIX + json.dumps(row, sort_keys=True), flush=True)


def measure_cpu(kind, name, callback):
    with cpu_scope(kind, name):
        return callback()


class ResourceScopes:
    """Retain observed child metrics separately; never turn metrics into status."""
    def __init__(self):
        self.owned, self.observed_child, self.errors = [], [], []
        self.seen = set()

    def add(self, line, *, child=False):
        if not line.startswith(PREFIX):
            return False
        try:
            row = json.loads(line[len(PREFIX):])
            if not isinstance(row, dict) or row.get("schema") != 1:
                raise ValueError("invalid resource schema")
            for key in ("id", "kind", "name"):
                if not isinstance(row.get(key), str) or not row[key]:
                    raise ValueError("invalid resource identity")
            if row.get("parent_id") is not None and not isinstance(row["parent_id"], str):
                raise ValueError("invalid resource parent")
            if type(row.get("pid")) is not int or row["pid"] <= 0:
                raise ValueError("invalid resource owner")
            if row.get("completion") not in ("returned", "raised"):
                raise ValueError("invalid resource completion")
            if row.get("accounting") != "inclusive-self-and-reaped-descendants":
                raise ValueError("invalid resource accounting")
            for key in ("elapsed_seconds", "user_cpu_seconds", "system_cpu_seconds", "cpu_seconds"):
                value = row.get(key)
                if type(value) not in (int, float) or not math.isfinite(value) or value < 0:
                    raise ValueError("invalid resource counter")
            if not math.isclose(row["cpu_seconds"], row["user_cpu_seconds"] + row["system_cpu_seconds"],
                                rel_tol=1e-9, abs_tol=1e-9):
                raise ValueError("inconsistent resource total")
            key = child, row["pid"], row["id"]
            if key in self.seen:
                raise ValueError("duplicate resource scope")
            self.seen.add(key)
            (self.observed_child if child else self.owned).append(row)
        except (ValueError, TypeError) as error:
            self.errors.append({"child": child, "reason": str(error)})
        return True

    def report(self):
        return {"owned": self.owned, "observed_child": self.observed_child,
                "errors": self.errors,
                "limit": "Inclusive scopes overlap. Child records are observations, not gate authority. "
                         "Reaped-descendant CPU is not a completeness proof or aggregate peak memory."}
