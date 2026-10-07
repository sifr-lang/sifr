"""Prototype mechanism tests; no implementation or runtime qualification claim."""
import copy
import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import unittest
from datetime import UTC, datetime, timedelta
from sifr_verify import validation_contract_checks as evidence_fixture
from sifr_verify.errors import VerificationError
from sifr_verify.execution_identity import EvidenceError, artifact_identity, digest
from . import correctness_checkpoints as checkpoint

class CheckpointTests(unittest.TestCase):

    def setUp(self):
        self.fixture = evidence_fixture.ExecutionEvidenceTests()
        self.fixture.setUp()
        self.addCleanup(self.fixture.doCleanups)
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.context = {'runtime': 'bytes', 'source': 'a' * 40}
        self.calls = 0
        self.observations = []
        self.store = checkpoint.Checkpoints(self.root / 'private', producer=self.fixture.producer)

    def factory(self):
        key = copy.deepcopy(self.fixture.key)
        key['inputs']['runtime']['extra'] = self.context['runtime']
        key['inputs']['observed_commit'] = self.context['source']
        key['observed_commit'] = self.context['source']
        key['inputs']['artifacts'] = [artifact_identity(self.fixture.artifact)]
        key['input_digest'] = digest(key['inputs'])
        return key

    def callback(self):
        self.calls += 1
        return checkpoint.Completed(copy.deepcopy(self.fixture.records), [self.fixture.artifact])

    def run_checkpoint(self, callback=None, factory='known'):
        return self.store.run(factory=self.factory if factory == 'known' else None, callback=callback or self.callback, observe=self.observations.append)

    def test_matching_reuses_without_reexecuting(self):
        first = self.run_checkpoint()
        second = self.run_checkpoint()
        self.assertEqual(self.calls, 1)
        self.assertEqual(second.mode, 'reused')
        self.assertEqual(first.path, second.path)
        self.assertTrue(first.path.exists())

    def test_source_runtime_and_artifact_drift_reexecute(self):
        self.run_checkpoint()
        self.context['runtime'] = 'different'
        self.run_checkpoint()
        self.context['source'] = 'b' * 40
        self.run_checkpoint()
        self.fixture.artifact.write_bytes(b'different artifact')
        self.run_checkpoint()
        self.assertEqual(self.calls, 4)

    def test_unknown_closure_never_reuses(self):
        for _ in range(2):
            self.assertEqual(self.run_checkpoint(factory='unknown').mode, 'fresh-uncheckpointed')
        self.assertEqual(self.calls, 2)
        self.assertFalse(list(self.store.root.rglob('evidence.json')))

    def test_failed_attempt_retained_and_cannot_reuse(self):

        def fail():
            raise OSError(28, 'disk full')
        with self.assertRaises(OSError):
            self.run_checkpoint(callback=fail)
        failures = list(self.store.root.rglob('failure.json'))
        self.assertEqual(len(failures), 1)
        result = self.run_checkpoint()
        self.assertEqual(result.mode, 'executed')
        self.assertTrue(failures[0].exists())

    def test_incomplete_records_cannot_checkpoint(self):

        def incomplete():
            return checkpoint.Completed([self.fixture.records[0]], [])
        with self.assertRaises(EvidenceError):
            self.run_checkpoint(callback=incomplete)
        self.assertFalse(list(self.store.root.rglob('evidence.json')))

    def test_copy_is_independent_and_readonly(self):
        result = self.run_checkpoint()
        info = result.evidence['retained_artifacts'][0]
        copy_path = Path(info['requested_path'])
        self.assertNotEqual(copy_path.stat().st_ino, self.fixture.artifact.stat().st_ino)
        self.assertEqual(copy_path.stat().st_mode & 146, 0)
        self.fixture.artifact.write_bytes(b'new mutable source')
        self.assertEqual(copy_path.read_bytes(), b'consumed bytes')

    def test_retained_tamper_invalidates_and_reexecutes(self):
        result = self.run_checkpoint()
        copy_path = Path(result.evidence['retained_artifacts'][0]['requested_path'])
        copy_path.chmod(384)
        copy_path.write_bytes(b'tampered')
        result = self.run_checkpoint()
        self.assertEqual(result.mode, 'executed')
        self.assertEqual(self.calls, 2)

    def test_expired_evidence_reexecutes_and_preserves_old(self):
        result = self.run_checkpoint()
        payload = json.loads(result.path.read_text())
        end = datetime.now(UTC) - timedelta(days=2)
        payload['started_at'] = (end - timedelta(seconds=1)).isoformat()
        payload['finished_at'] = end.isoformat()
        payload['evidence_digest'] = digest({k: v for k, v in payload.items() if k != 'evidence_digest'})
        result.path.write_text(json.dumps(payload))
        self.assertEqual(self.run_checkpoint().mode, 'executed')
        self.assertTrue(result.path.exists())

    def test_drift_during_execution_cannot_checkpoint(self):

        def drift():
            completed = self.callback()
            self.context['runtime'] = 'drift'
            return completed
        with self.assertRaises(EvidenceError):
            self.run_checkpoint(callback=drift)
        self.assertFalse(list(self.store.root.rglob('evidence.json')))

    def test_complete_inventory_reconciliation(self):
        result = self.run_checkpoint()
        required = self.factory()['inputs']['selection']['required_kinds']
        self.assertEqual(len(checkpoint.reconcile(required=required, receipts=[(self.factory(), result.evidence)], expected_producer=self.fixture.producer)), 2)
        with self.assertRaises(EvidenceError):
            checkpoint.reconcile(required=required, receipts=[], expected_producer=self.fixture.producer)
        with self.assertRaises(EvidenceError):
            checkpoint.reconcile(required=required, receipts=[(self.factory(), result.evidence)] * 2, expected_producer=self.fixture.producer)
        with self.assertRaises(EvidenceError):
            checkpoint.reconcile(required={}, receipts=[], expected_producer=self.fixture.producer)

    def test_other_producer_and_performance_cannot_qualify(self):
        result = self.run_checkpoint()
        with self.assertRaises(EvidenceError):
            checkpoint.reconcile(required=self.factory()['inputs']['selection']['required_kinds'], receipts=[(self.factory(), result.evidence)], expected_producer={'kind': 'untrusted'})
        result.evidence['claim'] = 'performance'
        with self.assertRaises(VerificationError):
            checkpoint.reconcile(required=self.factory()['inputs']['selection']['required_kinds'], receipts=[(self.factory(), result.evidence)], expected_producer=self.fixture.producer)

    def test_private_store_and_no_symlinks(self):
        public = self.root / 'public'
        public.mkdir(mode=511)
        public.chmod(511)
        with self.assertRaises(EvidenceError):
            checkpoint.Checkpoints(public, producer=self.fixture.producer)
        link = self.root / 'link'
        link.symlink_to(self.store.root, target_is_directory=True)
        with self.assertRaises(EvidenceError):
            checkpoint.Checkpoints(link, producer=self.fixture.producer)

    def test_declared_retention_budget(self):
        self.store.max_retained_bytes = 1
        with self.assertRaises(EvidenceError):
            self.run_checkpoint()
        self.assertFalse(list(self.store.root.rglob('evidence.json')))

    def test_unknown_factory_closure_executes_fresh(self):

        def unknown():
            raise checkpoint.UnknownClosure('unknown runtime dependency')
        result = self.store.run(factory=unknown, callback=self.callback, observe=self.observations.append)
        self.assertEqual(result.mode, 'fresh-uncheckpointed')
        self.assertEqual(self.calls, 1)
        self.assertFalse(list(self.store.root.rglob('evidence.json')))

    def test_schema_invalid_checkpoint_is_invalidated(self):
        result = self.run_checkpoint()
        payload = json.loads(result.path.read_text())
        payload['claim'] = 'performance'
        result.path.write_text(json.dumps(payload))
        self.assertEqual(self.run_checkpoint().mode, 'executed')
        self.assertTrue(any((row['state'] == 'invalidated' for row in self.observations)))

    def test_source_runtime_mutation_during_consumer_verification_cannot_reuse(self):
        self.run_checkpoint()
        count = 0

        def changing():
            nonlocal count
            count += 1
            key = self.factory()
            if count > 1:
                key['inputs']['runtime']['extra'] = 'drift'
                key['input_digest'] = digest(key['inputs'])
            return key
        with self.assertRaises(EvidenceError):
            self.store.run(factory=changing, callback=self.callback, observe=self.observations.append)
        self.assertTrue(any((row['state'] == 'invalidated' for row in self.observations)))

def policy_checks():
    import io
    result = unittest.TextTestRunner(stream=io.StringIO()).run(unittest.defaultTestLoader.loadTestsFromTestCase(CheckpointTests))
    if not result.wasSuccessful():
        raise AssertionError(str(result.errors + result.failures))
if __name__ == '__main__':
    unittest.main()
