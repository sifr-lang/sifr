"""Production wiring and real entrypoint failure tests for architecture policy."""
from __future__ import annotations

import copy
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import Mock

from architecture_policy_schema import validate_policy_schema
from unsafe_policy_segments import SEGMENTS

ROOT = Path(__file__).resolve().parent.parent
GUARDS = {
    'method-dispatch-authority': 'scripts/check_method_dispatch_authority.py',
    'unsafe-abi-contracts': 'scripts/check_unsafe_abi_contracts.py',
}


class PolicyIntegrationTests(unittest.TestCase):
    def test_required_profiles_and_schema_cover_both_guards(self):
        sys.path.insert(0, str(ROOT / 'verification/runner'))
        from sifr_verify.schemas import load_schema, validate_data
        from sifr_verify.errors import SchemaError
        from sifr_verify.profile_runner import ProfileRunner
        schema = load_schema('profile.schema.json')
        registered = {row['name']: row for row in json.loads(
            (ROOT / 'verification/policy/guardrails.json').read_text())['guardrails']}
        assignments = next(row for row in json.loads((ROOT /
            'verification/areas/coverage_matrix/profile_assignment_matrix.json').read_text())['rows']
            if row['surface_id'] == 'architecture_method_unsafe_policy')
        area = json.loads((ROOT / 'verification/areas/developer_tooling/manifest.json').read_text())
        self.assertIn('architecture-policy', {s['name'] for s in area['suites']})
        for profile in ('create-pr', 'merge', 'nightly', 'release'):
            data = json.loads((ROOT / f'verification/profiles/{profile}.json').read_text())
            validate_data(data, schema, source=profile)
            self.assertTrue(set(GUARDS) <= set(data['guardrail_steps']), profile)
            self.assertIn('architecture-policy', next(a['suites'] for a in data['selected_areas']
                                                      if a['area'] == 'developer_tooling'))
            self.assertEqual(assignments['profiles'][profile], ['developer_tooling:architecture-policy'])
            for guard, script in GUARDS.items():
                self.assertEqual(registered[guard]['entrypoint'], script)
                self.assertEqual(registered[guard]['args'], [])
                budget = data['step_budgets']['guardrail_' + guard.replace('-', '_')]
                self.assertEqual(budget['enforcement'], 'blocking')
                self.assertGreaterEqual(budget['budget_ms'], registered[guard]['timeout_seconds'] * 1000)
                runner = ProfileRunner.__new__(ProfileRunner)
                runner.run_script_with_self_test = Mock()
                runner.run_guardrail(guard)
                runner.run_script_with_self_test.assert_called_once_with(script)
        invalid = copy.deepcopy(data)
        invalid['guardrail_steps'].append('unregistered-architecture-guard')
        with self.assertRaises(SchemaError):
            validate_data(invalid, schema, source='unknown guard')
        inventories = [('method_dispatch_sites.json', 'method_dispatch_sites.schema.json')]
        inventories += [('unsafe_abi_sites/' + s + '.json', 'unsafe_abi_segment.schema.json')
                        for s in SEGMENTS]
        for inventory, schema_name in inventories:
            data = json.loads((ROOT / 'verification/policy' / inventory).read_text())
            validate_policy_schema(ROOT, data, schema_name)
            bad = copy.deepcopy(data)
            del bad['sites'][0]['fingerprint']
            with self.assertRaises(ValueError):
                validate_policy_schema(ROOT, bad, schema_name)

    def test_missing_inventory_segment_cannot_pass_live_guard(self):
        with tempfile.TemporaryDirectory(prefix='sifr-h02h5-segment-') as temp:
            checkout = Path(temp)
            shutil.copytree(ROOT / 'scripts', checkout / 'scripts',
                            ignore=shutil.ignore_patterns('__pycache__'))
            shutil.copytree(ROOT / 'verification/policy', checkout / 'verification/policy')
            # Only the live guard's schema engine is needed; source discovery
            # must not happen when even one inventory partition is absent.
            shutil.copyfile(ROOT / 'verification/json_schema_202012.py',
                            checkout / 'verification/json_schema_202012.py')
            for segment in SEGMENTS:
                path = checkout / f'verification/policy/unsafe_abi_sites/{segment}.json'
                original = path.read_bytes()
                path.unlink()
                result = subprocess.run([sys.executable, 'scripts/check_unsafe_abi_contracts.py'],
                    cwd=checkout, env=dict(os.environ, PYTHONDONTWRITEBYTECODE='1'),
                    capture_output=True, text=True, timeout=30)
                self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
                self.assertIn(segment + '.json', result.stderr)
                self.assertIn('unsafe ABI inventory failed:', result.stderr)
                self.assertNotIn('inventory passed', result.stdout + result.stderr)
                path.write_bytes(original)
