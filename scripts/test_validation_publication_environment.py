"""Broad, incomplete and tag-based secret access cannot protect an issuer."""
import unittest

from validation_publication_environment import verify_environment


class EnvironmentTests(unittest.TestCase):
    def setUp(self):
        self.policy = {"protected_branches": False, "custom_branch_policies": True}
        self.rows = [{"name": "main", "type": "branch"}]
        self.total = 1

    def api(self, path):
        if "/deployment-branch-policies?" in path:
            return {"total_count": self.total, "branch_policies": self.rows}
        return {"deployment_branch_policy": self.policy}

    def test_main_branch_only(self):
        verify_environment(self.api, "/repos/owner/repo")

    def test_every_broad_missing_or_tag_policy_rejected(self):
        for rows, total in (([], 0), ([{"name": "*", "type": "branch"}], 1),
                            ([{"name": "main", "type": "tag"}], 1),
                            ([{"name": "main", "type": "branch"}], 2),
                            ([{"name": "main", "type": "branch"}, {"name": "gh-readonly-queue/*", "type": "branch"}], 2)):
            self.rows, self.total = rows, total
            with self.subTest(rows=rows), self.assertRaises(ValueError):
                verify_environment(self.api, "/repos/owner/repo")
        self.policy = {"protected_branches": True, "custom_branch_policies": False}
        with self.assertRaises(ValueError):
            verify_environment(self.api, "/repos/owner/repo")


if __name__ == "__main__":
    unittest.main()
