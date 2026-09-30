"""Schema admission for live architecture inventories; cold fixtures stay pure."""
from __future__ import annotations

from pathlib import Path
import sys


def validate_policy_schema(root: Path, data: dict, schema: str) -> None:
    # The live checkout owns this dependency-free Draft 2020-12 validator.
    # Self-tests copied into cold directories never invoke live admission.
    sys.path.insert(0, str(root))
    from verification.json_schema_202012 import validate_instance
    validate_instance(data, root / 'verification/policy/schemas' / schema)
