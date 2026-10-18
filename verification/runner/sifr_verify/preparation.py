"""Ordered preparation dependencies; failures never become successful prerequisites."""
from dataclasses import dataclass
import json
import time
from typing import Callable

from .profile_commands import CommandFailed


@dataclass(frozen=True)
class Preparation:
    name: str
    requires: tuple[str, ...]
    callback: Callable[[], None]
    unsafe_on_failure: bool = False


class PreparationRun:
    def __init__(self, nodes, consumers, *, no_fail_fast=False):
        self.nodes = tuple(nodes)
        self.consumers = consumers
        self.no_fail_fast = no_fail_fast
        self.results = {}
        self.fatal_reason = None
        seen = set()
        for node in self.nodes:
            if node.name in seen or not set(node.requires) <= seen:
                raise ValueError(f"invalid preparation dependencies: {node.name}")
            seen.add(node.name)
        for name, requires in consumers.items():
            if not set(requires) <= seen:
                raise ValueError(f"unknown preparation prerequisite for {name}")

    def missing(self, requires):
        return tuple(name for name in requires
                     if self.results.get(name, {}).get("status") != "pass")

    def blocked_by(self, consumer):
        if self.fatal_reason:
            return self.fatal_reason
        if consumer not in self.consumers:
            raise ValueError(f"unplanned preparation consumer: {consumer}")
        return ",".join(self.missing(self.consumers[consumer]))

    def record(self, node, status, elapsed_ms, prerequisite=None, cause=None):
        row = {"name": node.name, "status": status, "elapsed_ms": elapsed_ms,
               "requires": list(node.requires), "prerequisite": prerequisite, "cause": cause}
        self.results[node.name] = row
        print("[sifr-preparation] " + json.dumps(row, sort_keys=True), flush=True)

    def run(self):
        failure = None
        stopped = None
        for node in self.nodes:
            blocked = stopped or ",".join(self.missing(node.requires))
            if blocked:
                self.record(node, "blocked", 0, prerequisite=blocked)
                continue
            start = time.monotonic_ns()
            try:
                node.callback()
            except (CommandFailed, ValueError, RuntimeError, OSError) as error:
                cause = getattr(error, "cause", None)
                self.record(node, "fail", (time.monotonic_ns() - start) // 1_000_000,
                            cause=cause or type(error).__name__)
                fatal = (node.unsafe_on_failure or not isinstance(error, CommandFailed)
                         or cause not in (None, "exit") or error.returncode < 0)
                if fatal:
                    self.fatal_reason = node.name
                    failure = error
                elif failure is None:
                    failure = error
                if fatal or not self.no_fail_fast:
                    stopped = node.name
            else:
                self.record(node, "pass", (time.monotonic_ns() - start) // 1_000_000)
        if failure is not None:
            raise failure
