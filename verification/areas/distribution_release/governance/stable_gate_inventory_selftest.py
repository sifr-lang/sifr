"""Self-test for the owned stable-gate inventory."""

from __future__ import annotations

import json
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[4]


def test_stable_gate_inventory() -> None:
    path = REPO_ROOT / "plans" / "releases" / "stable_gate_inventory.json"
    inventory = json.loads(path.read_text(encoding="utf-8"))
    if set(inventory) != {"schema_version", "owner", "gates"}:
        raise AssertionError("stable gate inventory fields drifted")
    if inventory["schema_version"] != 2 or inventory["owner"] != "release/distribution":
        raise AssertionError("stable gate inventory epoch/owner drifted")
    gates = inventory["gates"]
    if not isinstance(gates, list) or not gates:
        raise AssertionError("stable gate inventory is empty")
    ids: set[str] = set()
    required = {
        "id",
        "location",
        "owner",
        "current_behavior",
        "activation_boundary",
        "disposition",
    }
    for gate in gates:
        if not isinstance(gate, dict) or set(gate) != required:
            raise AssertionError(f"stable gate has invalid fields: {gate}")
        if not all(isinstance(gate[field], str) and gate[field] for field in required):
            raise AssertionError(f"stable gate has an empty owned field: {gate}")
        if gate["id"] in ids:
            raise AssertionError(f"duplicate stable gate: {gate['id']}")
        ids.add(gate["id"])
        if not (REPO_ROOT / gate["location"]).exists():
            raise AssertionError(f"stable gate location does not exist: {gate['location']}")
