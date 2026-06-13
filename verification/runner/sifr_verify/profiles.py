"""Profile policy helpers for the verification runner."""

from __future__ import annotations

from typing import Any


def selected_resource_classes(profile: dict[str, Any]) -> set[str]:
    classes: set[str] = set()
    resource_policy = profile.get("resource_policy", {})
    if isinstance(resource_policy, dict):
        raw_classes = resource_policy.get("classes", [])
        if isinstance(raw_classes, list):
            classes.update(item for item in raw_classes if isinstance(item, str))
    for selection in profile.get("selected_areas", []):
        if not isinstance(selection, dict):
            continue
        raw_classes = selection.get("resource_classes", [])
        if isinstance(raw_classes, list):
            classes.update(item for item in raw_classes if isinstance(item, str))
    return classes


def failure_reproduction_command(profile_name: str, case_id: str) -> str:
    return f"uv run --project verification python -m sifr_verify --profile {profile_name} --case {case_id}"
