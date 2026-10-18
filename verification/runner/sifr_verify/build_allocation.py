"""Independent compiler and native batch allocations within one profile budget."""
from __future__ import annotations

from dataclasses import asdict, dataclass
import json

from .errors import VerificationError
from .resource_capacity import Capacity, discover_capacity

MIB = 1024 ** 2
E2E_FLAGS = {'--sifr-jobs': 'sifr_jobs', '--rust-jobs': 'rust_jobs',
             '--run-jobs': 'run_jobs', '--cargo-build-jobs': 'cargo_build_jobs'}
E2E_ENV = {key: 'SIFR_E2E_' + key.upper() for key in E2E_FLAGS.values()}


def positive(value: object, name: str) -> int:
    if isinstance(value, bool) or not str(value).isascii() or not str(value).isdecimal() or int(value) < 1:
        raise VerificationError(f'{name} must be a positive integer, got {value!r}')
    return int(value)


def validate_build_policy(policy: dict) -> None:
    for key in ('max_parallel', 'compiler_build_jobs', 'memory_per_build_job_mib'):
        positive(policy[key], key)
    for key in ('reserved_cpus', 'reserved_memory_mib'):
        if type(policy[key]) is not int or policy[key] < 0:
            raise VerificationError(f'{key} must be a nonnegative integer')


def extract_overrides(arguments: list[str]) -> tuple[dict[str, int], list[str]]:
    overrides, remaining = {}, []
    index = 0
    while index < len(arguments):
        argument = arguments[index]
        flag, separator, inline = argument.partition('=')
        if flag not in E2E_FLAGS:
            remaining.append(argument)
        else:
            if separator:
                value = inline
            else:
                index += 1
                if index == len(arguments):
                    raise VerificationError(f'{flag} requires a value')
                value = arguments[index]
            overrides[E2E_FLAGS[flag]] = positive(value, flag)
        index += 1
    return overrides, remaining


@dataclass(frozen=True)
class BuildAllocation:
    compiler_jobs: int
    sifr_jobs: int
    rust_jobs: int
    run_jobs: int
    cargo_build_jobs: int
    slot_limit: int
    effective_cpus: int
    available_memory_mib: int
    policy: dict[str, int]
    sources: dict[str, str]
    requested: dict[str, int]

    def report(self) -> str:
        return '[sifr-build-allocation] ' + json.dumps(asdict(self), sort_keys=True)


def allocate(profile: dict, env: dict[str, str], overrides: dict[str, int],
             capacity: Capacity | None = None) -> BuildAllocation:
    capacity = capacity if capacity is not None else discover_capacity()
    policy = profile['resource_policy']
    validate_build_policy(policy)
    cpu_slots = max(1, capacity.cpu_count - policy['reserved_cpus'])
    memory_slots = (capacity.available_memory_bytes - policy['reserved_memory_mib'] * MIB) // (
        policy['memory_per_build_job_mib'] * MIB)
    slots = min(positive(policy['max_parallel'], 'max_parallel'), cpu_slots, memory_slots)
    if slots < 1:
        raise VerificationError('insufficient memory for one build job plus the profile reserve')
    sources = {}
    requested = {}
    compiler = env.get('CARGO_BUILD_JOBS', policy['compiler_build_jobs'])
    requested['compiler_jobs'] = positive(compiler, 'CARGO_BUILD_JOBS')
    sources['compiler_jobs'] = 'CARGO_BUILD_JOBS' if 'CARGO_BUILD_JOBS' in env else 'profile'
    for key, variable in E2E_ENV.items():
        value = overrides.get(key, env.get(variable, profile['e2e'][key]))
        requested[key] = positive(value, variable)
        sources[key] = 'CLI' if key in overrides else variable if variable in env else 'profile'
    # Stages execute sequentially. Within native compilation, outer * inner
    # consumes the same slot budget; preserve inner jobs first, then bound groups.
    inner = min(requested['cargo_build_jobs'], slots)
    return BuildAllocation(
        compiler_jobs=min(requested['compiler_jobs'], slots),
        sifr_jobs=min(requested['sifr_jobs'], slots),
        rust_jobs=min(requested['rust_jobs'], slots // inner),
        run_jobs=min(requested['run_jobs'], slots),
        cargo_build_jobs=inner, slot_limit=slots,
        effective_cpus=capacity.cpu_count,
        available_memory_mib=capacity.available_memory_bytes // MIB,
        policy={key: policy[key] for key in ('max_parallel', 'reserved_cpus',
                'reserved_memory_mib', 'memory_per_build_job_mib')},
        sources=sources, requested=requested,
    )
