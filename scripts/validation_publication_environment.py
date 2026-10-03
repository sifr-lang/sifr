"""Require server-enforced default-branch isolation for protected App secrets."""
from __future__ import annotations

ENVIRONMENT = "validation-check-publication"


def verify_environment(api, prefix: str) -> None:
    environment = api(prefix + "/environments/" + ENVIRONMENT)
    if environment.get("deployment_branch_policy") != {
            "protected_branches": False, "custom_branch_policies": True}:
        raise ValueError("publication credentials require an explicit main-only branch policy")
    policies = api(prefix + "/environments/" + ENVIRONMENT + "/deployment-branch-policies?per_page=100")
    rows = policies.get("branch_policies", [])
    if (policies.get("total_count") != 1 or len(rows) != 1 or
            rows[0].get("name") != "main" or rows[0].get("type") != "branch"):
        raise ValueError("publication environment must permit only main as a branch, never PR/queue/tag refs")
