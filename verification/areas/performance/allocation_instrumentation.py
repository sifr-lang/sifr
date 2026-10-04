"""Instrument a separately identified Rust application; never alter timed bytes."""
from pathlib import Path
import re

MODULE = Path(__file__).parent/'instrumentation/rust_allocator.rs'
NAME = '__sifr_allocation_observer_v1'
FIELDS = ('allocation_calls','allocation_requested_bytes','reallocation_calls',
          'reallocation_requested_bytes','deallocation_calls')
SCOPE = ('successful Rust GlobalAlloc request deltas while generated main executes; '
         'single-threaded registered workloads only; libc/loader and post-main cleanup excluded; '
         'separate instrumented artifact, never timed process bytes')


def instrument(source: bytes) -> bytes:
    text = source.decode('utf-8')
    matches = list(re.finditer(r'(?m)^fn main\(\)\s*\{', text))
    if len(matches) != 1 or NAME in text or 'global_allocator' in text:
        raise ValueError('allocation instrumentation requires one uninstrumented generated main')
    offset = matches[0].end()
    guard = f'\n    let _sifr_allocation_guard = {NAME}::MainObservation::begin();'
    return (f'mod {NAME} {{\n'+MODULE.read_text()+'\n}\n'+text[:offset]+guard+text[offset:]).encode()


def validate_counts(payload):
    if not isinstance(payload, dict) or set(payload) != {'schema_version',*FIELDS} or type(payload['schema_version']) is not int or payload['schema_version'] != 1:
        raise ValueError('allocation observation fields are incomplete')
    for field in FIELDS:
        value = payload[field]
        if type(value) is not int or not 0 <= value < 2**64:
            raise ValueError('allocation observations require unsigned integer counters')
    if (payload['allocation_calls']==0) != (payload['allocation_requested_bytes']==0):
        raise ValueError('registered nonzero-layout allocation counters disagree')
    if (payload['reallocation_calls']==0) != (payload['reallocation_requested_bytes']==0):
        raise ValueError('registered nonzero-layout reallocation counters disagree')
    return {field:payload[field] for field in FIELDS}
