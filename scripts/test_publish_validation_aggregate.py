"""Exercise publication against independent API facts without external writes."""
import copy
from datetime import datetime, timezone
import os
import unittest
from unittest.mock import patch

import publish_validation_aggregate as publisher
from validation_aggregate_policy import WORKFLOW, expected_jobs


class PublisherTests(unittest.TestCase):
    def setUp(self):
        self.candidate = "a" * 40
        self.trusted = "b" * 40
        stamp = datetime.now(timezone.utc).isoformat()
        self.run = {"id": 7, "run_attempt": 2, "repository": {"full_name": "owner/repo"},
                    "workflow_id": 9, "path": WORKFLOW, "event": "merge_group", "head_sha": self.candidate,
                    "status": "completed", "conclusion": "success", "html_url": "https://github.com/owner/repo/actions/runs/7"}
        self.jobs = [{"name": name, "status": "completed", "conclusion": "success",
                      "run_id": 7, "run_attempt": 2, "started_at": stamp, "completed_at": stamp}
                     for name in expected_jobs("merge_group", "merge")]
        self.published = []
        self.matches = True
        self.executed = self.candidate

    def api(self, path, body=None):
        if body is not None:
            self.published.append(copy.deepcopy(body))
            return {}
        if path.endswith("/actions/runs/7"):
            return self.run
        if path.endswith("/actions/workflows/local-first-validation.yml"):
            return {"id": 9}
        if "/contents/" in path:
            return {"sha": "file" if self.matches or path.endswith(self.trusted) else "changed"}
        if "/jobs?" in path:
            # Force real pagination instead of authorizing the first partial page.
            return {"total_count": len(self.jobs), "jobs": self.jobs[:3] if path.endswith("page=1") else self.jobs[3:]}
        raise AssertionError(path)

    def call(self):
        with patch.dict(os.environ, {"GITHUB_REPOSITORY": "owner/repo", "VALIDATION_RUN_ID": "7",
                                     "TRUSTED_WORKFLOW_SHA": self.trusted}), patch.object(publisher, "api", self.api), \
                patch.object(publisher, "candidate_identity", return_value={"candidate_sha": self.executed}):
            return publisher.main()

    def test_exact_candidate_and_complete_attempt_published(self):
        self.assertEqual(self.call(), 0)
        self.assertEqual(len(self.published), 1)
        self.assertEqual(self.published[0]["head_sha"], self.candidate)
        self.assertEqual(self.published[0]["external_id"], "validation:7:2:" + self.candidate)
        self.assertEqual(self.published[0]["conclusion"], "success")

    def test_failure_or_untrusted_workflow_cannot_publish_success(self):
        self.jobs[0]["conclusion"] = "skipped"
        self.assertEqual(self.call(), 1)
        self.assertEqual(self.published[-1]["conclusion"], "failure")
        self.jobs[0]["conclusion"] = "success"
        self.matches = False
        self.assertEqual(self.call(), 1)
        self.assertEqual(self.published[-1]["conclusion"], "failure")

    def test_checkout_receipt_drift_cannot_publish_success(self):
        self.executed = "c" * 40
        self.assertEqual(self.call(), 1)
        self.assertEqual(self.published[-1]["conclusion"], "failure")

    def test_api_failure_never_publishes_success(self):
        with patch.dict(os.environ, {"GITHUB_REPOSITORY": "owner/repo", "VALIDATION_RUN_ID": "7"}), \
                patch.object(publisher, "api", side_effect=OSError("API unavailable")):
            with self.assertRaises(OSError):
                publisher.main()
        self.assertEqual(self.published, [])


if __name__ == "__main__":
    unittest.main()
