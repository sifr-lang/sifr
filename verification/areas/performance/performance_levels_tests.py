"""Prospective level selection, limited claims and qualification boundaries."""
from __future__ import annotations
import copy
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import runner
import check_budgets
import check_trend_policy as trend
from benchmark_manifest import BenchmarkError
from performance_levels import load_levels, selected_cases, MANIFEST


class LevelTests(unittest.TestCase):
    def test_selections_and_shared_cloud_counts_are_complete(self):
        policy = load_levels()
        self.assertEqual([len(selected_cases(name)) for name in ['smoke', 'representative', 'full']], [5, 10, 65])
        self.assertFalse(policy['levels']['smoke']['requires_reference'])
        self.assertEqual(policy['shared_cloud_full']['fixed_pairs'], 5120)
        root = Path(__file__).resolve().parents[3]
        with patch.object(sys, 'path', [str(root/'verification/runner'), *sys.path]):
            from sifr_verify.performance_partition import MEASUREMENT_SUITES
        self.assertEqual(MEASUREMENT_SUITES,
            {name for name, spec in policy['levels'].items() if spec['requires_reference']})

    def test_drift_omissions_and_false_smoke_qualification_reject(self):
        policy = load_levels()
        variants = []
        missing = copy.deepcopy(policy); missing['levels']['representative']['cases'].pop(); variants.append(missing)
        duplicate = copy.deepcopy(policy); duplicate['levels']['smoke']['cases'][0] = duplicate['levels']['smoke']['cases'][1]; variants.append(duplicate)
        smoke = copy.deepcopy(policy); smoke['levels']['smoke']['requires_reference'] = True; variants.append(smoke)
        full = copy.deepcopy(policy); full['levels']['full']['cases'] = full['levels']['representative']['cases']; variants.append(full)
        pairs = copy.deepcopy(policy); pairs['shared_cloud_full']['fixed_pairs'] = 5119; variants.append(pairs)
        binding = copy.deepcopy(policy); binding['manifest_sha256'] = '0'*64; variants.append(binding)
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)/'policy.json'
            for value in variants:
                path.write_text(json.dumps(value))
                with self.subTest(value=value), self.assertRaises(BenchmarkError):
                    load_levels(path, MANIFEST)

    def test_full_selects_all_cases_and_budget_checker_requires_complete_inventory(self):
        with tempfile.TemporaryDirectory() as directory, \
             patch.dict(os.environ, {'SIFR_VALIDATION_PROFILE':'release'}, clear=True), \
             patch.object(runner, 'REPO_ROOT', Path(directory)), \
             patch.object(runner, 'run_command_variant', side_effect=lambda suite,label,argv:
                {'label':label,'argv':argv,'status':'pass'}) as command:
            result = runner.run_profile_variants('full')
        producer, checker = result
        self.assertEqual(producer['argv'].count('--case'), 65)
        self.assertNotIn('--allow-subset', checker['argv'])
        self.assertEqual(command.call_count, 2)

    def test_smoke_producer_clears_reference_and_never_calls_budget_qualification(self):
        with patch.dict(os.environ, {'SIFR_VALIDATION_PROFILE':'create-pr'}), \
             patch.object(runner, 'run_command_variant', side_effect=lambda suite,label,argv:
                {'label':label,'argv':argv,'status':'pass'}):
            result = runner.run_profile_variants('smoke')
        self.assertEqual(len(result), 1)
        argv = result[0]['argv']
        self.assertEqual(argv.count('--case'), 5)
        self.assertEqual(argv[argv.index('--reference-profile')+1], '')
        self.assertIn('smoke', argv)

    def test_smoke_results_cannot_qualify_numeric_budgets(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)/'smoke.json'
            path.write_text(json.dumps({'metadata':{'sample_scale':'smoke'}}))
            with patch.object(sys,'argv',['check_budgets.py','--results',str(path),'--reference-profile','']):
                self.assertEqual(check_budgets.main(), 1)

    def test_structure_checks_never_make_stale_baseline_eligible(self):
        manifest = trend.load_json(trend.DEFAULT_MANIFEST)
        baseline = trend.load_json(trend.DEFAULT_TREND_BASELINES)
        policy = trend.load_json(trend.DEFAULT_POLICY)
        now = max(row['baseline_captured_at_unix'] for row in baseline['results']) + (policy['baseline_window_days']+1)*86400
        trend.validate_trend_policy(manifest,baseline,policy,manifest_path=MANIFEST,
                                    now_unix=now,qualify_baseline=False)
        with self.assertRaisesRegex(trend.TrendPolicyError,'stale trend baseline'):
            trend.validate_trend_policy(manifest,baseline,policy,manifest_path=MANIFEST,now_unix=now)


if __name__ == '__main__':
    unittest.main()
