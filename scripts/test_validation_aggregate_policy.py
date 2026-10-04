"""Missing, failed, cancelled, stale and untrusted inputs cannot qualify."""
import copy
from datetime import datetime, timezone
import unittest

from validation_aggregate_policy import WORKFLOW, evaluate, expected_jobs, component_steps


class AggregateTests(unittest.TestCase):
    def setUp(self):
        self.now = datetime(2026, 10, 3, 12, tzinfo=timezone.utc)
        self.sha = "a" * 40
        self.run = {"id": 7, "run_attempt": 2, "repository": {"full_name": "owner/repo"},
                    "workflow_id": 9, "path": WORKFLOW, "event": "merge_group", "head_sha": self.sha,
                    "status": "completed", "conclusion": "success"}
        self.jobs = [{"name": name, "steps": [{"name": step, "conclusion": "success"} for step in component_steps(name)], "status": "completed", "conclusion": "success",
                      "run_id": 7, "run_attempt": 2, "started_at": "2026-10-03T10:00:00Z",
                      "completed_at": "2026-10-03T11:00:00Z"}
                     for name in expected_jobs("merge_group", "merge")]

    def check(self, **kwargs):
        return evaluate(self.run, self.jobs, candidate=self.sha, profile="merge", repository="owner/repo",
                        workflow_id=9, workflow_matches=kwargs.get("matches", True), now=self.now)

    def test_complete_current_candidate(self):
        self.assertEqual(self.check(), [])

    def test_every_nonpassing_mandatory_result_rejected(self):
        for conclusion in (None, "failure", "skipped", "cancelled", "timed_out", "neutral", "action_required"):
            with self.subTest(conclusion=conclusion):
                self.jobs[0]["conclusion"] = conclusion
                self.assertTrue(self.check())
        self.jobs = []
        self.assertTrue(self.check())

    def test_stale_drift_duplicate_and_untrusted_inputs_rejected(self):
        for field, value in (("completed_at", "2026-10-01T11:00:00Z"),
                             ("completed_at", "2026-10-04T11:00:00Z"),
                             ("run_attempt", 1), ("run_id", 8), ("status", "in_progress")):
            old = copy.deepcopy(self.jobs)
            self.jobs[0][field] = value
            self.assertTrue(self.check())
            self.jobs = old
        self.jobs.append(copy.deepcopy(self.jobs[0]))
        self.assertTrue(self.check())
        self.jobs.pop()
        self.assertTrue(self.check(matches=False))
        for field, value in (("head_sha", "b" * 40), ("workflow_id", 10), ("path", "untrusted.yml"),
                             ("event", "workflow_dispatch"), ("conclusion", "failure")):
            old = copy.deepcopy(self.run)
            self.run[field] = value
            self.assertTrue(self.check())
            self.run = old


if __name__ == "__main__":
    unittest.main()
