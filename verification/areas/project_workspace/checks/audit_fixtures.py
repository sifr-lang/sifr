"""Validate promoted project-workspace audit fixtures."""

from __future__ import annotations

from pathlib import Path

from sifr_verify.audit_fixtures import run_audit_fixture_manifest

MANIFEST = Path(__file__).resolve().parents[1] / "data" / "audit_fixtures.json"


def main() -> int:
    return run_audit_fixture_manifest(MANIFEST, area="project_workspace")


if __name__ == "__main__":
    raise SystemExit(main())
