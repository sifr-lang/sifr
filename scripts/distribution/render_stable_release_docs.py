#!/usr/bin/env python3
"""Render stable version and withdrawal facts into the public release document."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[2]
AREA_ROOT = REPO_ROOT / "verification" / "areas" / "distribution_release"
sys.path.insert(0, str(AREA_ROOT))

from governance import GovernanceError, validate_site_release_facts  # noqa: E402
from governance.common import load_json_strict  # noqa: E402

START_MARKER = "<!-- stable-release-facts:start -->"
END_MARKER = "<!-- stable-release-facts:end -->"


class DocumentationRenderError(ValueError):
    """Stable documentation rendering failed."""


def render_facts_block(facts: dict[str, Any]) -> str:
    validate_site_release_facts(facts)
    withdrawals = facts["withdrawals"]
    if withdrawals:
        rendered = ", ".join(
            f"`{item['version']}` ({item['incident_id']})"
            for item in withdrawals
        )
    else:
        rendered = "none."
    return (
        f"{START_MARKER}\n"
        f"Active stable version: `{facts['stable_version']}`\n\n"
        f"Withdrawn stable versions: {rendered}\n"
        f"{END_MARKER}"
    )


def render_document(document: str, facts: dict[str, Any]) -> str:
    if document.count(START_MARKER) != 1 or document.count(END_MARKER) != 1:
        raise DocumentationRenderError(
            "stable release document must contain exactly one facts marker pair"
        )
    start = document.index(START_MARKER)
    end = document.index(END_MARKER) + len(END_MARKER)
    if start >= end:
        raise DocumentationRenderError(
            "stable release document facts markers are not ordered"
        )
    return document[:start] + render_facts_block(facts) + document[end:]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--facts", required=True)
    parser.add_argument("--document", required=True)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true")
    mode.add_argument("--out")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    document_path = Path(args.document)
    try:
        facts = load_json_strict(Path(args.facts), require_canonical=True)
        document = document_path.read_text(encoding="utf-8")
        rendered = render_document(document, facts)
        if args.check:
            if rendered != document:
                raise DocumentationRenderError(
                    "stable release document facts are stale"
                )
        else:
            output = Path(args.out)
            if output.exists():
                raise DocumentationRenderError("output path already exists")
            output.parent.mkdir(parents=True, exist_ok=True)
            output.write_text(rendered, encoding="utf-8")
    except (
        DocumentationRenderError,
        GovernanceError,
        OSError,
        UnicodeError,
    ) as exc:
        print(f"stable-release-docs: {exc}", file=sys.stderr)
        return 2
    print("stable release documentation facts: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
