"""Disjoint source-owned unsafe partitions, independent of mutable owner labels.

Partial validation explicitly names its source partition. Whole-tree validation
requires every segment; absent segments are errors, never empty defaults.
"""
from __future__ import annotations

import json
from pathlib import Path

from rust_policy_sites import code_tokens, reconcile

SEGMENTS = ('python_core', 'python_resources', 'external')
RESOURCE_MODULES = ('buffer_ops', 'arrow_ops', 'dlpack_ops')


def segment_for(site) -> str:
    prefix = 'crates/sifr_runtime/src/'
    if site.kind == 'generated-rust' or not site.path.startswith(prefix):
        return 'external'
    path = site.path[len(prefix):]
    if any(path == f'python/{name}.rs' or path.startswith(f'python/{name}/')
           for name in RESOURCE_MODULES):
        return 'python_resources'
    # Parent declarations of resource modules belong to the resource segment,
    # including #[allow(unsafe_code)] mod arrow_ops in python.rs.
    values = [t.value for t in code_tokens(site.text)]
    if site.kind == 'unsafe-allowance' and any(
            values[i:i + 2] == ['mod', name]
            for i in range(len(values) - 1) for name in RESOURCE_MODULES):
        return 'python_resources'
    if path == 'python.rs' or path.startswith('python/'):
        return 'python_core'
    return 'external'


def validate_segments(sites, segments: dict, *, selected=SEGMENTS, sources=()) -> list[str]:
    """Validate exactly the explicitly selected source partition (all by default)."""
    import check_unsafe_abi_contracts as unsafe
    if not selected or len(set(selected)) != len(selected) or set(selected) - set(SEGMENTS):
        return ['invalid explicit unsafe segment selection']
    errors, records, proofs = [], [], {}
    for name in selected:
        data = segments.get(name)
        if not isinstance(data, dict):
            errors.append(f'missing required unsafe segment: {name}')
            continue
        if (data.get('schema_version') != 1 or data.get('segment') != name
                or not isinstance(data.get('sites'), list)):
            errors.append(f'invalid unsafe segment schema: {name}')
            continue
        bounded = [s for s in sites if segment_for(s) == name]
        keys = {s.key for s in bounded}
        errors += reconcile(bounded, data['sites'])
        for record in data['sites']:
            if record.get('site') not in keys:
                errors.append(f'stale or wrong source partition in {name}: {record.get("site")}')
        errors += unsafe.validate(bounded, data, sources=sources)
        records += data['sites']
        for key, value in data.get('shared_contracts', {}).items():
            if key in proofs:
                errors.append(f'duplicate shared contract: {key}')
            proofs[key] = value
    bounded = [s for s in sites if segment_for(s) in selected]
    errors += reconcile(bounded, records)
    return errors


def read_segments(root: Path, *, selected=SEGMENTS) -> dict:
    result = {}
    for name in selected:
        if name not in SEGMENTS:
            raise ValueError(f'unknown unsafe segment: {name}')
        path = root / 'verification/policy/unsafe_abi_sites' / f'{name}.json'
        result[name] = json.loads(path.read_text())
    return result
