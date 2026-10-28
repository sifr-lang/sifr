"""Bound ordinary driver libtest workers and their nested compiler work together."""
from dataclasses import asdict, dataclass
import json
import time

from .build_allocation import MIB, positive
from .errors import VerificationError
from .profile_commands import CommandFailed
from .resource_capacity import discover_capacity


@dataclass(frozen=True)
class DriverTestAllocation:
    workers: int
    cargo_jobs: int
    rayon_threads: int
    slots: int
    reserved_memory_mib: int

    def report(self):
        return "[sifr-driver-test-allocation] " + json.dumps(asdict(self), sort_keys=True)


def current_capacity():
    try:
        return discover_capacity()
    except VerificationError as error:
        raise CommandFailed(2, "driver_test_capacity") from error


def allocate_driver_tests(build, env, command):
    capacity = current_capacity()
    policy = build.policy
    slots = min(build.slot_limit, max(1, capacity.cpu_count - policy["reserved_cpus"]),
                (capacity.available_memory_bytes // MIB - policy["reserved_memory_mib"])
                // policy["memory_per_build_job_mib"])
    if slots < 1:
        raise CommandFailed(2, "driver_test_capacity")
    # libtest's explicit flag overrides its environment. Preserve the command,
    # but reject an explicit width that would bypass the shared resource budget.
    arguments = command[command.index("--") + 1:] if "--" in command else []
    explicit = None
    for index, argument in enumerate(arguments):
        if argument == "--test-threads":
            explicit = positive(arguments[index + 1] if index + 1 < len(arguments) else "",
                                "--test-threads")
        elif argument.startswith("--test-threads="):
            explicit = positive(argument.split("=", 1)[1], "--test-threads")
    requested = explicit if explicit is not None else positive(
        env.get("RUST_TEST_THREADS", slots), "RUST_TEST_THREADS")
    if explicit is not None and explicit > slots:
        raise CommandFailed(2, "driver_test_capacity")
    workers = min(requested, slots)
    allocation = DriverTestAllocation(workers, min(build.compiler_jobs, slots // workers),
                                      min(build.sifr_jobs, slots // workers), slots,
                                      policy["reserved_memory_mib"])
    return allocation, dict(env, RUST_TEST_THREADS=str(workers),
                            CARGO_BUILD_JOBS=str(allocation.cargo_jobs),
                            RAYON_NUM_THREADS=str(allocation.rayon_threads))


def reserve_monitor(allocation):
    last = float("-inf")
    def check():
        nonlocal last
        now = time.monotonic()
        if now - last < 1:
            return
        last = now
        if current_capacity().available_memory_bytes < allocation.reserved_memory_mib * MIB:
            raise CommandFailed(2, "driver_test_capacity")
    return check
