"""Validate promoted stdlib parity audit fixtures."""

from __future__ import annotations

from pathlib import Path

from sifr_verify.audit_fixtures import run_audit_fixture_manifest

MANIFEST = Path(__file__).resolve().parents[1] / "data" / "audit_fixtures.json"


def main() -> int:
    return run_audit_fixture_manifest(MANIFEST, area="stdlib_parity")


if __name__ == "__main__":
    raise SystemExit(main())
