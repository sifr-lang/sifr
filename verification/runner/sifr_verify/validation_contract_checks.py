"""Negative checks for stage selection and input-bound execution evidence."""
from __future__ import annotations

import copy
import json
import tempfile
import unittest
from datetime import UTC, datetime, timedelta
from pathlib import Path
from unittest.mock import patch

from .execution_evidence import build_evidence, validate_evidence, write_evidence
from .execution_identity import EvidenceError, artifact_identity, digest
from .validation_contracts import ContractError, POLICY, load_policy, selected_inventory, stage_plan


class ValidationContractTests(unittest.TestCase):
    def test_cloud_uses_the_live_merge_inventory(self):
        cloud, merge = stage_plan("cloud"), stage_plan("merge")
        for key in ("profile_plan", "selected_inventory", "e2e_inventory"):
            self.assertEqual(cloud[key], merge[key])
        self.assertEqual(cloud["execution_state"], "not-executed")
        self.assertEqual(cloud["selection_state"], "selected")

    def test_selected_ids_are_real_and_complete(self):
        selected = stage_plan("create-pr")
        e2e = selected["e2e_inventory"]
        self.assertEqual(e2e["selected_count"], len(e2e["selected_ids"]))
        self.assertTrue(set(e2e["selected_ids"]).issubset(e2e["total_ids"]))
        for suite in selected["selected_inventory"]:
            self.assertEqual(suite["inventory_case_count"], len(suite["cases"]))
            self.assertTrue(suite["owner"])
            self.assertGreater(suite["timeout_seconds"], 0)

    def test_artifact_qualification_cannot_claim_source_profile_execution(self):
        result = stage_plan("artifact-qualification")
        self.assertNotIn("profile_plan", result)
        self.assertIn("artifact_hashes", result["required_bindings"])
        self.assertTrue(result["artifact_consumers"])

    def test_stage_omissions_and_duplicates_fail(self):
        data = json.loads(POLICY.read_text())
        for rows in [data["stages"][:-1], [*data["stages"], data["stages"][0]]]:
            changed = dict(data, stages=rows)
            with tempfile.TemporaryDirectory() as directory:
                path = Path(directory) / "policy.json"
                path.write_text(json.dumps(changed))
                with self.assertRaises(Exception):
                    load_policy(path)

    def test_cloud_selection_cannot_be_demoted_to_smoke(self):
        data = json.loads(POLICY.read_text())
        next(row for row in data["stages"] if row["stage"] == "cloud")["profile"] = "create-pr"
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "policy.json"
            path.write_text(json.dumps(data))
            with self.assertRaises(ContractError):
                load_policy(path)

    def test_duplicate_adapter_cases_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "verification/areas/example/manifest.json"
            path.parent.mkdir(parents=True)
            path.write_text(json.dumps({"name": "example", "owner": "owner", "timeout_seconds": 1,
                                        "resource_classes": [], "suites": [{"name": "suite", "cases": [{"id": "same"}, {"id": "same"}]}]}))
            with self.assertRaises(ContractError):
                selected_inventory({"selected_areas": [{"area": "example", "suites": ["suite"]}]}, root)


class ExecutionEvidenceTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.artifact = Path(self.directory.name) / "artifact"
        self.artifact.write_bytes(b"consumed bytes")
        self.producer = {"kind": "local-owned", "owner": "test-session"}
        self.selected = ["runtime/a", "static/b"]
        inputs = {"observed_commit": "a" * 40, "source": {"schema_version": 1, "input_digest": digest([]), "inputs": []},
                  "selection": {"required_kinds": {"runtime/a": "runtime", "static/b": "validation"}},
                  "commands": [["run", "actual"]], "runtime": {"interpreter": "bytes"},
                  "artifacts": [artifact_identity(self.artifact)], "services": {},
                  "producer": self.producer, "resource_identity": {"quota": 4}}
        self.key = {"schema_version": 1, "observed_commit": "a" * 40,
                    "inputs": inputs, "input_digest": digest(inputs)}
        self.records = [{"id": identifier, "state": "passed",
                         "phases": ["selected", "executed", "passed"], "executed_count": 1,
                         "elapsed_seconds": .1, "infrastructure": "none",
                         "execution_kind": inputs["selection"]["required_kinds"][identifier]}
                        for identifier in self.selected]
        self.now = datetime.now(UTC)

    def payload(self, records=None, complete=True):
        return build_evidence(key=self.key, selected=self.selected,
                              records=self.records if records is None else records,
                              complete=complete, started_at=(self.now - timedelta(seconds=10)).isoformat(),
                              finished_at=self.now.isoformat(), retained_artifacts=[self.artifact])

    def validate(self, payload, key=None, **kwargs):
        return validate_evidence(payload, expected_key=key or self.key, require_complete=True,
                                 expected_producer=self.producer, now=self.now, **kwargs)

    def test_complete_matching_runtime_evidence_qualifies(self):
        self.validate(self.payload())

    def test_missing_duplicate_or_unknown_required_work_cannot_pass(self):
        for records in [self.records[:-1], [*self.records, self.records[0]],
                        [dict(self.records[0], id="unknown"), self.records[1]]]:
            with self.assertRaises(EvidenceError):
                self.payload(records)

    def test_compilation_and_selected_only_do_not_satisfy_runtime(self):
        for change in [{"execution_kind": "compile"}, {"executed_count": 0},
                       {"phases": ["selected", "compiled", "passed"]},
                       {"phases": ["selected", "failed", "executed", "passed"]},
                       {"phases": ["selected", "executed", "compiled", "passed"]}]:
            with self.subTest(change=change), self.assertRaises(EvidenceError):
                self.payload([dict(self.records[0], **change), self.records[1]])

    def test_skip_block_failure_and_infrastructure_cannot_pass(self):
        for state in ["skipped", "blocked", "failed", "infrastructure-failure"]:
            with self.subTest(state=state), self.assertRaises(EvidenceError):
                self.payload([dict(self.records[0], state=state, phases=["selected", state]), self.records[1]])
        with self.assertRaises(EvidenceError):
            self.payload([dict(self.records[0], infrastructure="oom"), self.records[1]])

    def test_checkpoint_is_usable_only_as_partial_correctness(self):
        payload = self.payload(self.records[:-1], complete=False)
        validate_evidence(payload, expected_key=self.key, require_complete=False,
                          expected_producer=self.producer, now=self.now)
        with self.assertRaises(EvidenceError):
            self.validate(payload)

    def test_source_runtime_artifact_service_command_and_host_drift_reject_reuse(self):
        payload = self.payload()
        for name in ["source", "runtime", "artifacts", "services", "commands", "resource_identity", "selection"]:
            changed = copy.deepcopy(self.key)
            changed["inputs"][name] = {"changed": True}
            changed["input_digest"] = digest(changed["inputs"])
            with self.subTest(name=name), self.assertRaises(EvidenceError):
                self.validate(payload, changed)

    def test_untrusted_producer_rejected(self):
        with self.assertRaises(EvidenceError):
            validate_evidence(self.payload(), expected_key=self.key, require_complete=True,
                              expected_producer={"kind": "untrusted-pr"}, now=self.now)

    def test_digest_tampering_rejected(self):
        payload = self.payload()
        payload["records"][0]["elapsed_seconds"] = .2
        with self.assertRaises(EvidenceError):
            self.validate(payload)

    def test_retained_artifact_drift_rejected(self):
        payload = self.payload()
        self.artifact.write_bytes(b"different")
        with self.assertRaises(EvidenceError):
            self.validate(payload)

    def test_consumed_artifact_drift_rejected_without_retention_copy(self):
        payload = build_evidence(key=self.key, records=self.records, selected=self.selected,
                                 complete=True, started_at=(self.now - timedelta(seconds=1)).isoformat(),
                                 finished_at=self.now.isoformat(), retained_artifacts=[])
        self.artifact.write_bytes(b"different consumed bytes")
        with self.assertRaises(EvidenceError):
            self.validate(payload)

    def test_freshness_begins_at_completion_not_capture_start(self):
        payload = self.payload()
        payload["started_at"] = (self.now - timedelta(hours=13)).isoformat()
        payload["evidence_digest"] = digest({k: v for k, v in payload.items() if k != "evidence_digest"})
        self.validate(payload, max_age_seconds=86400)
        with self.assertRaises(EvidenceError):
            validate_evidence(payload, expected_key=self.key, require_complete=True,
                              expected_producer=self.producer, now=self.now + timedelta(hours=25), max_age_seconds=86400)

    def test_failure_records_are_not_overwritten(self):
        path = Path(self.directory.name) / "result.json"
        payload = self.payload()
        write_evidence(path, payload)
        with self.assertRaises(FileExistsError):
            write_evidence(path, payload)
        self.assertEqual(json.loads(path.read_text()), payload)


def policy_checks():
    suite = unittest.TestSuite(unittest.defaultTestLoader.loadTestsFromTestCase(cls)
                               for cls in (ValidationContractTests, ExecutionEvidenceTests))
    result = unittest.TestResult()
    suite.run(result)
    if not result.wasSuccessful():
        raise AssertionError(result.errors + result.failures)


if __name__ == "__main__":
    unittest.main()
