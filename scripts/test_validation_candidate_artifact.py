"""Immutable candidate bytes and complete producer identity are mandatory."""
import copy
import hashlib
import io
import json
import unittest
from unittest.mock import patch
import zipfile

from validation_candidate_artifact import candidate_identity


class CandidateArtifactTests(unittest.TestCase):
    def setUp(self):
        self.run = {"id": 7, "run_attempt": 2, "head_sha": "a" * 40}
        self.identity = {"candidate_sha": "b" * 40, "run_id": 7, "run_attempt": 2}
        raw = io.BytesIO()
        with zipfile.ZipFile(raw, "w") as archive:
            archive.writestr("candidate.json", json.dumps(self.identity))
        self.raw = raw.getvalue()
        self.artifact = {"name": "validation-candidate-2", "expired": False,
                         "workflow_run": {"id": 7, "head_sha": "a" * 40},
                         "digest": "sha256:" + hashlib.sha256(self.raw).hexdigest(),
                         "archive_download_url": "https://api.github.com/repos/owner/repo/actions/artifacts/1/zip"}

    def check(self, total=1):
        with patch("validation_candidate_artifact.archive_bytes", return_value=self.raw):
            return candidate_identity(lambda *_: {"total_count": total, "artifacts": [self.artifact]},
                                      "/repos/owner/repo", self.run)

    def test_verified_complete_archive_passes(self):
        self.assertEqual(self.check(), self.identity)

    def test_drift_expiry_wrong_producer_and_incomplete_inventory_fail(self):
        for field, value in (("expired", True), ("name", "validation-candidate-1"),
                             ("digest", "sha256:" + "0" * 64),
                             ("workflow_run", {"id": 8, "head_sha": "a" * 40})):
            original = copy.deepcopy(self.artifact)
            self.artifact[field] = value
            with self.subTest(field=field), self.assertRaises(ValueError):
                self.check()
            self.artifact = original
        with self.assertRaises(ValueError):
            self.check(total=2)
        self.raw += b"tampering"
        with self.assertRaises(ValueError):
            self.check()

    def test_unexpected_archive_inventory_fails_without_extracting(self):
        raw = io.BytesIO()
        with zipfile.ZipFile(raw, "w") as archive:
            archive.writestr("../candidate.json", json.dumps(self.identity))
        self.raw = raw.getvalue()
        self.artifact["digest"] = "sha256:" + hashlib.sha256(self.raw).hexdigest()
        with self.assertRaises(ValueError):
            self.check()


if __name__ == "__main__":
    unittest.main()
