"""Native test processes share the profile budget, without changing selection."""
from dataclasses import asdict, dataclass
import json
import time

from .build_allocation import MIB
from .errors import VerificationError
from .profile_commands import CommandFailed
from .resource_capacity import discover_capacity

ALLOCATION_ENV = "SIFR_VERIFY_NATIVE_TEST_ALLOCATION"


@dataclass(frozen=True)
class NativeAllocation:
    workers: int
    cargo_jobs: int
    rayon_threads: int
    memory_per_worker_mib: int
    reserved_memory_mib: int
    reserved_cpus: int

    def report(self):
        return "[sifr-native-allocation] " + json.dumps(asdict(self), sort_keys=True)


def allocate_native(build):
    policy = build.policy
    for workers in range(min(2, build.slot_limit), 0, -1):
        jobs = min(build.compiler_jobs, build.slot_limit // workers)
        rayon = min(build.sifr_jobs, build.slot_limit // workers)
        memory = max(4096, jobs * policy["memory_per_build_job_mib"])
        if workers * memory + policy["reserved_memory_mib"] <= build.available_memory_mib:
            return NativeAllocation(workers, jobs, rayon, memory,
                                    policy["reserved_memory_mib"], policy["reserved_cpus"])
    # Unrelated profile categories remain usable. Native consumers fail admission.
    return NativeAllocation(0, 1, 1, 4096, policy["reserved_memory_mib"], policy["reserved_cpus"])


def native_environment(env):
    try:
        values = json.loads(env[ALLOCATION_ENV])
        allocation = NativeAllocation(**values)
        if (any(type(v) is not int for v in values.values())
                or not 0 <= allocation.workers <= 2
                or min(allocation.cargo_jobs, allocation.rayon_threads) < 1
                or allocation.memory_per_worker_mib < 4096
                or min(allocation.reserved_memory_mib, allocation.reserved_cpus) < 0):
            raise ValueError("invalid native allocation")
    except (KeyError, TypeError, ValueError) as error:
        raise VerificationError("native tests require the resolved profile allocation") from error
    if allocation.workers == 0:
        raise VerificationError("insufficient native test memory plus profile reserve")
    return allocation, dict(env, CARGO_BUILD_JOBS=str(allocation.cargo_jobs),
                            RAYON_NUM_THREADS=str(allocation.rayon_threads))


def current_capacity():
    try:
        return discover_capacity()
    except VerificationError as error:
        raise CommandFailed(2, "native_capacity") from error


def readmit(allocation):
    capacity = current_capacity()
    cpu_workers = max(1, capacity.cpu_count - allocation.reserved_cpus) // max(
        allocation.cargo_jobs, allocation.rayon_threads)
    memory_workers = (capacity.available_memory_bytes // MIB - allocation.reserved_memory_mib
                      ) // allocation.memory_per_worker_mib
    workers = min(allocation.workers, cpu_workers, memory_workers)
    if workers < 1:
        raise CommandFailed(2, "native_capacity")
    print(allocation.report(), flush=True)
    print(f"[sifr-native-admission] workers={workers} available_memory_mib="
          f"{capacity.available_memory_bytes // MIB} effective_cpus={capacity.cpu_count}", flush=True)
    return workers


def reserve_monitor(allocation):
    last = float("-inf")
    reported = float("-inf")
    minimum = None
    def check():
        nonlocal last, reported, minimum
        now = time.monotonic()
        if now - last < 1:
            return
        last = now
        available = current_capacity().available_memory_bytes // MIB
        minimum = available if minimum is None else min(minimum, available)
        if now - reported >= 30:
            reported = now
            print(f"[sifr-native-capacity] available_memory_mib={available} "
                  f"minimum_observed_mib={minimum} reserve_mib={allocation.reserved_memory_mib}", flush=True)
        if available < allocation.reserved_memory_mib:
            print("[sifr-native-capacity] profile memory reserve exhausted", flush=True)
            raise CommandFailed(2, "native_capacity")
    return check
