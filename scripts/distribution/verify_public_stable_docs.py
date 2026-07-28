#!/usr/bin/env python3
"""Verify public stable documentation renders the exact governed site facts."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT))

from verification.areas.distribution_release.governance.common import (  # noqa: E402
    GovernanceError,
    load_json_strict,
)
from verification.areas.distribution_release.governance.release_plan import (  # noqa: E402
    validate_site_release_facts,
)

RENDERED_LABELS = (
    "Active stable version",
    "Withdrawn stable versions",
)


def verify_public_stable_docs(*, facts_path: Path, document_path: Path) -> None:
    """Require the stable and every withdrawal identity in rendered docs."""
    facts = validate_site_release_facts(
        load_json_strict(facts_path, require_canonical=True)
    )
    document = document_path.read_text(encoding="utf-8")
    required = [
        RENDERED_LABELS[0],
        facts["stable_version"],
        RENDERED_LABELS[1],
    ]
    for withdrawal in facts["withdrawals"]:
        required.extend((withdrawal["version"], withdrawal["incident_id"]))
    missing = [value for value in required if value not in document]
    if missing:
        raise GovernanceError(
            "public stable documentation omitted governed fact(s): "
            + ", ".join(missing)
        )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--facts", type=Path, required=True)
    parser.add_argument("--document", type=Path, required=True)
    args = parser.parse_args()
    try:
        verify_public_stable_docs(
            facts_path=args.facts,
            document_path=args.document,
        )
    except (GovernanceError, OSError, UnicodeError) as exc:
        print(f"public-stable-docs: {exc}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
