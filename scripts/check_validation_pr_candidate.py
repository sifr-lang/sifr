#!/usr/bin/env python3
"""Real Git controls for the producer and trusted PR publication boundary."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import publish_validation_aggregate as publisher
import select_ci_validation as selector
from test_publish_validation_aggregate import PublisherTests
from validation_aggregate_policy import component_steps, expected_jobs
from validation_pr_candidate import verify_pr_candidate


class PRBindingTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='sifr-pr-binding-')
        self.addCleanup(self.temporary.cleanup)
        self.repo = Path(self.temporary.name)
        self.git('init', '--quiet')
        self.tree = self.git('mktree', data='')
        self.old_base = self.commit('base')
        self.base = self.commit('base advances', self.old_base)
        self.old_head = self.commit('PR head', self.old_base)
        # Identical trees deliberately prove that tree equality or ancestry
        # cannot substitute for the exact current commit identities.
        self.head = self.commit('PR head advances', self.old_head)
        self.valid = self.commit('current test merge', self.base, self.head)
        self.stale_head = self.commit('old head test merge', self.base, self.old_head)
        self.stale_base = self.commit('old base test merge', self.old_base, self.head)

    def git(self, *args, data=None):
        env = os.environ.copy() | {'GIT_AUTHOR_NAME': 'Fixture', 'GIT_AUTHOR_EMAIL': 'fixture@example.invalid',
                                  'GIT_COMMITTER_NAME': 'Fixture', 'GIT_COMMITTER_EMAIL': 'fixture@example.invalid'}
        return subprocess.check_output(['git', '-C', str(self.repo), *args], input=data,
                                       text=True, stderr=subprocess.PIPE, env=env).strip()

    def commit(self, message, *parents):
        args = ['commit-tree', self.tree]
        for parent in parents:
            args.extend(['-p', parent])
        return self.git(*args, data=message+'\n')

    def test_current_diverged_and_fast_forwardable_test_merges_pass(self):
        verify_pr_candidate(self.repo, self.valid, self.base, self.head)
        head = self.commit('fast forwardable PR', self.base)
        candidate = self.commit('GitHub synthetic no-ff merge', self.base, head)
        verify_pr_candidate(self.repo, candidate, self.base, head)

    def test_stale_heads_bases_and_non_merge_candidates_fail(self):
        reversed_merge = self.commit('reversed parents', self.head, self.base)
        for candidate in (self.stale_head, self.stale_base, self.head, reversed_merge):
            with self.subTest(candidate=candidate), self.assertRaisesRegex(ValueError, 'parents differ'):
                verify_pr_candidate(self.repo, candidate, self.base, self.head)
        for candidate in ('f'*40, self.tree, 'HEAD'):
            with self.subTest(candidate=candidate), self.assertRaises(ValueError):
                verify_pr_candidate(self.repo, candidate, self.base, self.head)

    def test_replacement_and_graft_cannot_launder_stale_merge(self):
        self.git('replace', self.stale_head, self.valid)
        (self.repo/'.git/info/grafts').write_text(f'{self.stale_head} {self.base} {self.head}\n')
        with self.assertRaisesRegex(ValueError, 'parents differ'):
            verify_pr_candidate(self.repo, self.stale_head, self.base, self.head)
        verify_pr_candidate(self.repo, self.valid, self.base, self.head)

    def select(self, candidate, *, event='pull_request'):
        self.git('checkout', '--quiet', '--detach', candidate)
        env = {'CANDIDATE_SHA': candidate, 'BASE_SHA': self.base, 'PR_HEAD_SHA': self.head,
               'EVENT_NAME': event, 'GITHUB_RUN_ID': '7', 'GITHUB_RUN_ATTEMPT': '2',
               'GITHUB_OUTPUT': str(self.repo/'selection-output')}
        with patch.dict(os.environ, env), patch.object(selector, 'ROOT', self.repo), \
                patch.object(selector, 'selection', return_value={'profile': 'create-pr'}) as coverage:
            try:
                selector.main()
            except ValueError:
                coverage.assert_not_called()
                raise
            if event == 'pull_request':
                coverage.assert_called_once_with(self.repo, self.base, candidate)
            else:
                coverage.assert_not_called()
        return self.repo/'target/validation-candidate/candidate.json'

    def test_producer_rejects_stale_event_before_artifact_or_selection(self):
        for candidate in (self.stale_head, self.stale_base):
            with self.subTest(candidate=candidate), self.assertRaisesRegex(ValueError, 'parents differ'):
                self.select(candidate)
            self.assertFalse((self.repo/'target/validation-candidate/candidate.json').exists())
            self.assertFalse((self.repo/'selection-output').exists())

    def test_producer_records_exact_valid_merge(self):
        path = self.select(self.valid)
        self.assertEqual(json.loads(path.read_text()),
                         {'candidate_sha': self.valid, 'run_id': 7, 'run_attempt': 2})

    def test_non_pr_producer_does_not_require_synthetic_merge_parents(self):
        self.assertEqual(json.loads(self.select(self.head, event='merge_group').read_text())['candidate_sha'], self.head)

    def publish(self, candidate, *, executed=None):
        fixture = PublisherTests()
        fixture.setUp()
        fixture.candidate = candidate
        fixture.executed = candidate if executed is None else executed
        fixture.run.update(event='pull_request', head_sha=self.head,
                           pull_requests=[{'number': 3, 'base': {'sha': self.base}}])
        fixture.jobs = [dict(fixture.jobs[0], name=name,
                            steps=[{'name': step, 'conclusion': 'success'} for step in component_steps(name)])
                        for name in expected_jobs('pull_request', 'create-pr')]
        original_api = fixture.api
        def api(path, body=None):
            if path.endswith('/pulls/3'):
                return {'merge_commit_sha': candidate, 'head': {'sha': self.head},
                        'base': {'sha': self.base}, 'state': 'open'}
            return original_api(path, body)
        self.last_publications = fixture.published
        with patch.object(fixture, 'api', api), patch.object(publisher, 'ROOT', self.repo), \
                patch.object(publisher, 'fetch'), \
                patch('sifr_verify.change_selection.selection', return_value={'profile': 'create-pr'}):
            return fixture.call()

    def test_publisher_rejects_stale_api_merge_even_when_artifact_matches(self):
        for candidate in (self.stale_head, self.stale_base):
            with self.subTest(candidate=candidate), self.assertRaisesRegex(ValueError, 'parents differ'):
                self.publish(candidate)
            self.assertEqual(self.last_publications, [])

    def test_publisher_accepts_actual_current_merge_and_preserves_artifact_check(self):
        self.assertEqual(self.publish(self.valid), 0)
        self.assertEqual(self.last_publications[0]['head_sha'], self.head)
        self.assertEqual(self.last_publications[0]['conclusion'], 'success')
        self.assertEqual(self.publish(self.valid, executed=self.stale_head), 1)
        self.assertEqual(self.last_publications[0]['conclusion'], 'failure')


if __name__ == '__main__':
    unittest.main()
