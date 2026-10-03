"""Prospective workload selections and limits on their performance claims."""
from __future__ import annotations
import hashlib
import json
from pathlib import Path

from benchmark_manifest import BenchmarkError, load_manifest, validate_manifest
from cloud_precision import PAIR_COUNTS, pairs_for_case
from cloud_statistics import POLICY_VERSION

AREA = Path(__file__).parent
POLICY = AREA / 'data/performance_levels.json'
MANIFEST = AREA / 'data/benchmark_manifest.json'


def load_levels(policy_path=POLICY, manifest_path=MANIFEST):
    policy = json.loads(policy_path.read_text())
    if set(policy) != {'schema_version', 'owner', 'manifest_sha256', 'levels', 'shared_cloud_full'}:
        raise BenchmarkError('performance level policy fields drifted')
    if policy['schema_version'] != 1 or policy['owner'] != 'compiler/performance':
        raise BenchmarkError('unknown performance level policy')
    if policy['manifest_sha256'] != hashlib.sha256(manifest_path.read_bytes()).hexdigest():
        raise BenchmarkError('performance level manifest binding changed')
    cases = validate_manifest(load_manifest(manifest_path))
    ids = [case.id for case in cases]
    levels = policy['levels']
    if set(levels) != {'smoke', 'representative', 'full'}:
        raise BenchmarkError('performance levels must declare smoke, representative and full')
    for name, count in [('smoke', 5), ('representative', 10), ('full', 65)]:
        spec = levels[name]
        if set(spec) != {'cases', 'case_count', 'sample_scale', 'requires_reference', 'allowed_claims', 'forbidden_claims'}:
            raise BenchmarkError('performance level fields drifted')
        selected = ids if spec['cases'] == 'all' else spec['cases']
        if not isinstance(selected, list) or any(not isinstance(value, str) for value in selected):
            raise BenchmarkError('performance level case IDs must be explicit')
        if (spec['case_count'] != count or len(selected) != count or len(set(selected)) != count
                or not set(selected).issubset(ids) or selected != sorted(selected)):
            raise BenchmarkError('performance level selection is incomplete or duplicated')
        if name == 'full' and spec['cases'] != 'all':
            raise BenchmarkError('full performance must select the complete manifest')
        if (spec['sample_scale'] != ('smoke' if name == 'smoke' else 'manifest')
                or spec['requires_reference'] is not (name != 'smoke')):
            raise BenchmarkError('performance level qualification semantics changed')
        for field in ['allowed_claims', 'forbidden_claims']:
            if not isinstance(spec[field], list) or not spec[field] or not all(isinstance(value, str) and value for value in spec[field]):
                raise BenchmarkError('performance level claims must be declared')
    cloud = policy['shared_cloud_full']
    if cloud != {'protocol': POLICY_VERSION, 'case_count': 65, 'fixed_pairs': 5120,
                 'qualification': 'independent verified complete receipt only'}:
        raise BenchmarkError('registered shared-cloud full contract changed')
    if set(PAIR_COUNTS) != set(ids) or sum(pairs_for_case(identifier) for identifier in ids) != 5120:
        raise BenchmarkError('shared-cloud fixed case/pair inventory changed')
    return policy


def selected_cases(level):
    policy = load_levels()
    spec = policy['levels'][level]
    return ([case.id for case in validate_manifest(load_manifest(MANIFEST))]
            if spec['cases'] == 'all' else list(spec['cases']))
