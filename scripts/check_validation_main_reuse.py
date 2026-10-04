#!/usr/bin/env python3
"""Synthetic provenance controls; never claims actual hosted reuse execution."""
import copy
from datetime import datetime, timedelta, timezone
import hashlib
import io
import json
import unittest
from unittest.mock import patch
import zipfile

from validation_aggregate_policy import evaluate, expected_jobs, WORKFLOW, component_steps
import validation_main_reuse as reuse


class MainReuse(unittest.TestCase):
    def setUp(self):
        self.now = datetime.now(timezone.utc)
        self.sha = 'a' * 40; self.repo = 'unit/repo'; self.prefix = '/repos/' + self.repo
        self.run = {'id': 1, 'run_attempt': 1, 'event': 'merge_group', 'head_sha': self.sha,
                    'repository': {'full_name': self.repo}, 'workflow_id': 3, 'path': WORKFLOW,
                    'status': 'completed', 'conclusion': 'success'}
        self.jobs = [{'name': name, 'steps': [{'name': step, 'conclusion': 'success'} for step in component_steps(name)], 'run_id': 1, 'run_attempt': 1, 'status': 'completed', 'conclusion': 'success',
                      'started_at': (self.now-timedelta(seconds=3)).isoformat(),
                      'completed_at': (self.now-timedelta(seconds=1)).isoformat()}
                     for name in sorted(expected_jobs('merge_group', 'merge'))]
        self.workflow_blob = 'trusted-blob'; self.protected_blob = 'trusted-blob'
        self.identity = {'candidate_sha': self.sha, 'run_id': 1, 'run_attempt': 1}
        self.pr = {'merged_at': self.now.isoformat(), 'merge_commit_sha': self.sha, 'head': {'sha': self.sha}}
        self.artifact = None

    def api(self, path):
        if '/artifacts?' in path: return {'total_count': 1, 'artifacts': [self.artifact]}
        if path.endswith('/actions/runs/1'): return self.run
        if '/jobs?' in path: return {'total_count': len(self.jobs), 'jobs': self.jobs}
        if path.endswith('/actions/workflows/local-first-validation.yml'): return {'id': 3}
        if '/contents/' in path: return {'sha': self.protected_blob if path.endswith('ref=main') else self.workflow_blob}
        if '/pulls/' in path: return self.pr
        if '/runs?' in path: return {'workflow_runs': [self.run]}
        raise AssertionError(path)

    def identity_reader(self, *args): return self.identity

    def verify(self):
        return reuse.verify(self.api, self.repo, self.sha, 1, now=self.now, identity_reader=self.identity_reader)

    def test_exact_complete_premerge_producer_qualifies_only_correctness_jobs(self):
        proof = self.verify()
        self.assertEqual(set(proof['reused_jobs']), reuse.REUSED_JOBS)
        self.assertNotIn('local-first-merge', proof['reused_jobs'])
        self.assertEqual(reuse.select(self.api, self.repo, self.sha, 2, now=self.now,
                                     identity_reader=self.identity_reader)['state'], 'reused')

    def test_incomplete_stale_foreign_or_recursive_producers_run_fresh(self):
        for mutation in ('commit', 'artifact', 'event', 'missing', 'duplicate', 'skip', 'failure', 'attempt', 'stale', 'future', 'workflow', 'repository', 'running', 'component-step'):
            self.setUp()
            if mutation == 'commit': self.run['head_sha'] = 'b'*40
            if mutation == 'artifact': self.identity['candidate_sha'] = 'b'*40
            if mutation == 'event': self.run['event'] = 'push'
            if mutation == 'missing': self.jobs.pop()
            if mutation == 'duplicate': self.jobs.append(copy.deepcopy(self.jobs[0]))
            if mutation == 'skip': self.jobs[0]['conclusion'] = 'skipped'
            if mutation == 'failure': self.run['conclusion'] = 'failure'
            if mutation == 'attempt': self.jobs[0]['run_attempt'] = 2
            if mutation == 'stale': self.jobs[0]['completed_at'] = (self.now-timedelta(days=2)).isoformat()
            if mutation == 'future': self.jobs[0]['completed_at'] = (self.now+timedelta(seconds=1)).isoformat()
            if mutation == 'workflow': self.workflow_blob = 'untrusted'
            if mutation == 'repository': self.run['repository']['full_name'] = 'other/repo'
            if mutation == 'running': self.run['status'] = 'in_progress'
            if mutation == 'component-step': next(job for job in self.jobs if component_steps(job['name']))['steps'][0]['conclusion']='skipped'
            with self.subTest(mutation=mutation):
                with self.assertRaises(ValueError): self.verify()
                self.assertEqual(reuse.select(self.api, self.repo, self.sha, 2, now=self.now,
                    identity_reader=self.identity_reader)['state'], 'fresh')

    def test_pr_must_be_merged_as_actual_executed_candidate_with_full_merge_jobs(self):
        self.run.update(event='pull_request', pull_requests=[{'number': 8}])
        self.jobs.append(dict(self.jobs[0], name='deterministic-report-signature'))
        self.verify()
        for field,value in [('merged_at', None), ('merge_commit_sha', 'b'*40), ('head', {'sha': 'b'*40})]:
            original=self.pr[field];self.pr[field]=value
            with self.subTest(field=field), self.assertRaises(ValueError): self.verify()
            self.pr[field]=original
        self.jobs=[job for job in self.jobs if job['name']!='local-first-merge']
        self.jobs.append(dict(self.jobs[0], name='local-first-create-pr'))
        with self.assertRaises(ValueError): self.verify()

    def test_current_attempt_can_skip_only_independently_verified_correctness_jobs(self):
        current=dict(self.run, event='push', id=2)
        jobs=[dict(job, run_id=2) for job in self.jobs]
        for job in jobs:
            if job['name'] in reuse.REUSED_JOBS:
                job.update(conclusion='success' if component_steps(job['name']) else 'skipped', started_at=None, completed_at=None,
                           steps=[{'name':'Record exact-commit correctness reuse','conclusion':'success'}] if component_steps(job['name']) else [])
        args=dict(candidate=self.sha, profile='merge', repository=self.repo, workflow_id=3,
                  workflow_matches=True, now=self.now)
        self.assertTrue(evaluate(current, jobs, **args))
        self.assertEqual(evaluate(current, jobs, **args, reused_jobs=set(reuse.REUSED_JOBS)), [])
        self.assertTrue(evaluate(current, jobs, **args, reused_jobs={'local-first-merge'}))
        self.assertTrue(evaluate(dict(current,event='merge_group'), jobs, **args, reused_jobs=set(reuse.REUSED_JOBS)))
        jobs[0]['run_attempt']=2
        self.assertTrue(evaluate(current, jobs, **args, reused_jobs=set(reuse.REUSED_JOBS)))

    def decision_archive(self, decision):
        stream=io.BytesIO()
        with zipfile.ZipFile(stream,'w') as archive: archive.writestr('main-reuse.json',json.dumps(decision))
        raw=stream.getvalue()
        self.artifact={'name':'validation-main-reuse-1','expired':False,
                       'workflow_run':{'id':2,'head_sha':self.sha},
                       'archive_download_url':'unit-url','digest':'sha256:'+hashlib.sha256(raw).hexdigest()}
        return raw

    def test_trusted_reader_recomputes_source_facts_and_rejects_substitution(self):
        decision=dict(self.verify(), state='reused',candidate_sha=self.sha,run_id=2,run_attempt=1)
        current=dict(self.run,id=2,event='push',head_branch='main')
        for mutation in ('none','source-attempt','candidate','job','current-attempt','source-id','source-drift','digest','producer'):
            modified=copy.deepcopy(decision)
            if mutation=='source-attempt': modified['source_run_attempt']=2
            if mutation=='candidate': modified['candidate_sha']='b'*40
            if mutation=='job': modified['reused_jobs'].append('local-first-merge')
            if mutation=='current-attempt': modified['run_attempt']=2
            if mutation=='source-id': modified['source_run_id']=2
            raw=self.decision_archive(modified)
            if mutation=='source-drift': self.jobs[0]['conclusion']='failure'
            if mutation=='digest': self.artifact['digest']='sha256:'+'0'*64
            if mutation=='producer': self.artifact['workflow_run']['id']=3
            with patch.object(reuse,'archive_bytes',return_value=raw),patch.object(reuse,'candidate_identity',self.identity_reader):
                # Default argument is bound at definition; verify's reader is
                # replaced explicitly while preserving the real verifier.
                original=reuse.verify
                with patch.object(reuse,'verify',side_effect=lambda *a,**kw: original(*a,**kw,identity_reader=self.identity_reader)):
                    if mutation=='none': self.assertEqual(reuse.read_decision(self.api,self.prefix,current),set(reuse.REUSED_JOBS))
                    else:
                        with self.subTest(mutation=mutation), self.assertRaises(ValueError): reuse.read_decision(self.api,self.prefix,current)
            self.jobs[0]['conclusion']='success'


if __name__=='__main__': unittest.main()
