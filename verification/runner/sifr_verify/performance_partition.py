"""Reconcile host-independent and measurement suites into one complete area."""
from __future__ import annotations
import json

from .execution_evidence import write_evidence
from .paths import REPO_ROOT
from .profile_results import AreaResultError, validate_area_result

MEASUREMENT_SUITES = frozenset({'representative', 'full'})


def result_path(profile_name, phase=None, *, root=REPO_ROOT):
    slug = 'performance' if phase is None else 'performance-' + phase
    return root / 'target/verification/areas' / f'{slug}-{profile_name}-results.json'


def combine(profile_name, selected, *, root=REPO_ROOT):
    suites = {}
    levels = json.loads((root / 'verification/areas/performance/data/performance_levels.json').read_text())
    for phase, measured in [('correctness', False), ('measurement', True)]:
        expected = [name for name in selected if (name in MEASUREMENT_SUITES) == measured]
        if not expected:
            continue
        payload = validate_area_result(result_path(profile_name, phase, root=root),
                                       area='performance', expected_suites=expected)
        if payload.get('performance_levels') != levels:
            raise AreaResultError('performance partition claim/selection policy differs')
        for item in payload['suites']:
            if item['name'] in suites:
                raise AreaResultError('duplicate performance partition suite')
            suites[item['name']] = item
    if set(suites) != set(selected) or len(selected) != len(set(selected)):
        raise AreaResultError('incomplete performance partition selection')
    payload = {'schema_version': 1, 'area': 'performance', 'bless': False, 'performance_levels': levels,
        'suites': [suites[name] for name in selected],
        'summary': {'total_variants': sum(item['total_variants'] for item in suites.values()),
                    'total_failures': 0, 'blocking_failures': 0, 'non_blocking_failures': 0}}
    path = result_path(profile_name, root=root)
    write_evidence(path, payload)
    validate_area_result(path, area='performance', expected_suites=selected)
