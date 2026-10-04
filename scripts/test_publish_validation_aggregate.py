"""Exercise publication against independent API facts without external writes."""
import copy
from datetime import datetime, timezone
import os
import unittest
from unittest.mock import patch

import publish_validation_aggregate as publisher
from validation_aggregate_policy import WORKFLOW, expected_jobs, component_steps


class PublisherTests(unittest.TestCase):
    def setUp(self):
        self.candidate = "a" * 40
        self.trusted = "b" * 40
        stamp = datetime.now(timezone.utc).isoformat()
        self.run = {"id": 7, "run_attempt": 2, "repository": {"full_name": "owner/repo"},
                    "workflow_id": 9, "path": WORKFLOW, "event": "merge_group", "head_sha": self.candidate,
                    "status": "completed", "conclusion": "success", "html_url": "https://github.com/owner/repo/actions/runs/7"}
        self.jobs = [{"name": name, "steps": [{"name": step, "conclusion": "success"} for step in component_steps(name)], "status": "completed", "conclusion": "success",
                      "run_id": 7, "run_attempt": 2, "started_at": stamp, "completed_at": stamp}
                     for name in expected_jobs("merge_group", "merge")]
        self.published = []
        self.matches = True
        self.executed = self.candidate

    def api(self, path, body=None):
        if path.endswith("/environments/validation-check-publication"):
            return {"deployment_branch_policy": {"protected_branches": False, "custom_branch_policies": True}}
        if "/deployment-branch-policies?" in path:
            return {"total_count": 1, "branch_policies": [{"name": "main", "type": "branch"}]}
        if body is not None:
            self.published.append(copy.deepcopy(body))
            return {"app": {"id": 42, "slug": "validation-fixture"}}
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
                                     "TRUSTED_WORKFLOW_SHA": self.trusted, "CHECK_APP_ID": "42"}), patch.object(publisher, "api", self.api), \
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

    def test_pr_publishes_head_while_binding_tested_merge_candidate(self):
        branch = "c" * 40
        base = "d" * 40
        self.run.update(event="pull_request", head_sha=branch,
                        pull_requests=[{"number": 3, "base": {"sha": base}}])
        self.jobs = [dict(self.jobs[0], name=name, steps=[{"name": step, "conclusion": "success"} for step in component_steps(name)]) for name in expected_jobs("pull_request", "create-pr")]
        old_api = self.api
        def pr_api(path, body=None):
            if path.endswith("/pulls/3"):
                return {"merge_commit_sha": self.candidate, "head": {"sha": branch},
                        "base": {"sha": base}, "state": "open"}
            return old_api(path, body)
        with patch.object(self, "api", pr_api), patch.object(publisher, "fetch"), \
                patch("sifr_verify.change_selection.selection", return_value={"profile": "create-pr"}):
            self.assertEqual(self.call(), 0)
        self.assertEqual(self.published[-1]["head_sha"], branch)
        self.assertTrue(self.published[-1]["external_id"].endswith(self.candidate))
        self.assertIn(self.candidate, self.published[-1]["output"]["summary"])

    def test_actions_app_identity_cannot_publish_protected_check(self):
        with patch.dict(os.environ, {"CHECK_APP_ID": "15368"}), patch.object(publisher, "api") as api:
            with self.assertRaises(ValueError):
                publisher.main()
            api.assert_not_called()

    def test_api_failure_never_publishes_success(self):
        with patch.dict(os.environ, {"GITHUB_REPOSITORY": "owner/repo", "VALIDATION_RUN_ID": "7", "CHECK_APP_ID": "42"}), \
                patch.object(publisher, "api", side_effect=OSError("API unavailable")):
            with self.assertRaises(OSError):
                publisher.main()
        self.assertEqual(self.published, [])

    def test_main_requires_independent_reuse_verification_before_accepting_skips(self):
        from validation_main_reuse import REUSED_JOBS
        self.run.update(event='push', head_branch='main')
        for job in self.jobs:
            if job['name'] in REUSED_JOBS:
                job.update(conclusion='success' if component_steps(job['name']) else 'skipped', started_at=None, completed_at=None,
                           steps=[{'name':'Record exact-commit correctness reuse','conclusion':'success'}] if component_steps(job['name']) else [])
        with patch('validation_main_reuse.read_decision', return_value=set()):
            self.assertEqual(self.call(), 1)
        with patch('validation_main_reuse.read_decision', return_value=set(REUSED_JOBS)) as reader:
            self.assertEqual(self.call(), 0)
            reader.assert_called_once()
        before=len(self.published)
        with patch('validation_main_reuse.read_decision', side_effect=ValueError('source evidence differs')):
            with self.assertRaises(ValueError): self.call()
        self.assertEqual(len(self.published), before)


if __name__ == "__main__":
    unittest.main()
