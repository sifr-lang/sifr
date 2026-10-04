"""Synthetic ownership/retention controls; no native compiler qualification."""
import copy
import json
from pathlib import Path
import unittest
from unittest.mock import patch
import uuid

import native_candidate_tests as fixtures
import native_recovery as recovery
from sifr_verify.graph_retirement import GraphLease
from sifr_verify.resource_admission import ResourceError


class RecoveryChecks(unittest.TestCase):
    def setUp(self):
        self.fixture=fixtures.CandidateChecks('test_preparation_never_claims_runtime_assertions')
        self.fixture.setUp();self.addCleanup(self.fixture.doCleanups)
        self.root=self.fixture.root;self.previous=self.fixture.out;self.previous.chmod(0o700)
        self.tools=self.fixture.tools;self.target=self.fixture.target
        self.old=copy.deepcopy(self.fixture.report);self.old['status']='failed'
        (self.previous/'state.json').write_text(json.dumps(self.old))
        self.owner=str(uuid.uuid4())
        lease=GraphLease(self.previous,recovery.RELATIVE,self.owner).acquire({'fixture-prior-attempt'})
        lease.close()
        self.binary=Path(self.old['cargo_artifact']['executable']);self.binary.parent.mkdir(parents=True)
        self.binary.write_bytes(b'explicit synthetic compiler fixture, never executed')
        self.output=self.previous.parent/'new-attempt';self.output.mkdir(mode=0o700)
        self.patch=patch.object(recovery,'active_builds',return_value=[]);self.patch.start();self.addCleanup(self.patch.stop)

    def inspect(self): return recovery.inspect(self.root,self.previous,self.tools,self.target)

    def preserve(self):
        record=self.inspect()
        with patch.object(recovery,'require_native_binary') as native_boundary:
            record=recovery.preserve(record,self.output)
            native_boundary.assert_called_once_with(self.binary,False)
        return record

    def test_owned_failed_attempt_can_continue_without_reclassifying_or_retiring_it(self):
        before=(self.previous/'state.json').read_bytes();record=self.preserve()
        lease=recovery.acquire(record)
        try:
            self.assertTrue(lease.eligible)
            self.binary.write_bytes(b'new synthetic mutable cache bytes')
            self.assertEqual(recovery.check(record,self.output,self.root,self.tools,self.target),Path(record['graph']))
        finally: lease.close()
        self.assertEqual((self.previous/'state.json').read_bytes(),before)
        self.assertTrue((self.previous/'target/.validation-graph-leases/cargo-target.json').exists())
        self.assertTrue((self.output/'original-compiler.gz').is_file())

    def test_tools_dependency_and_status_drift_cannot_authorize_cache_mutation(self):
        for mutation in ('tools','lock','status','producer','target'):
            state=copy.deepcopy(self.old)
            if mutation=='tools': state['tools']={'fixture':'different'}
            if mutation=='lock': state['cargo_lock_sha256']='0'*64
            if mutation=='status': state['status']='incomplete'
            if mutation=='producer': state['producer_sha256']['native_candidate.py']='0'*64
            if mutation=='target': state['target']='foreign-target'
            (self.previous/'state.json').write_text(json.dumps(state))
            with self.subTest(mutation=mutation),self.assertRaises(ValueError): self.inspect()

    def test_unknown_linked_busy_or_foreign_graphs_reject(self):
        self.previous.chmod(0o755)
        with self.assertRaises(ValueError): self.inspect()
        self.previous.chmod(0o700)
        with patch.object(recovery,'active_builds',return_value=[42]),self.assertRaises(ValueError): self.inspect()
        record=self.inspect();lease=recovery.acquire(record)
        try:
            with self.assertRaises((ResourceError,BlockingIOError)): recovery.acquire(record)
        finally: lease.close()
        marker=self.previous/'target/.validation-graph-leases/cargo-target.json'
        data=json.loads(marker.read_text());data['inode']+=1;marker.write_text(json.dumps(data))
        with self.assertRaises(ValueError): self.inspect()
        data['inode']-=1;marker.write_text(json.dumps(data))
        link=self.binary.parent/'linked-input';link.symlink_to(self.root/'Cargo.lock')
        with self.assertRaises(ValueError): self.inspect()

    def test_raw_original_or_retained_artifact_drift_rejects_independent_check(self):
        record=self.preserve()
        retained=self.output/'original-compiler.gz';retained.chmod(0o600);retained.write_bytes(b'drift')
        with self.assertRaises(ValueError): recovery.check(record,self.output,self.root,self.tools,self.target)
        record=self.inspect()
        (self.previous/'build-package.stdout').write_bytes(b'changed raw original output')
        with self.assertRaises(ValueError): self.inspect()

    def test_recorded_provenance_and_unknown_compiler_profile_reject(self):
        record=self.preserve();changed=copy.deepcopy(record);changed['state_sha256']='0'*64
        with self.assertRaises(ValueError): recovery.check(changed,self.output,self.root,self.tools,self.target)
        events=self.previous/'cargo.jsonl';artifact=copy.deepcopy(self.old['cargo_artifact'])
        artifact['profile']['opt_level']='1';events.write_text(json.dumps(artifact)+'\n')
        with self.assertRaises(ValueError): self.inspect()


if __name__=='__main__': unittest.main()
