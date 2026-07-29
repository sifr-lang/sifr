"""Fixture for single-maintainer publication approval schema checks."""

from __future__ import annotations

from typing import Any


def approval_waiver_fixture() -> dict[str, Any]:
    return {
        "schema_version": 2,
        "repository": "sifr-lang/sifr",
        "environment": "stable-release",
        "owner_login": "yaseralnajjar",
        "allowed_operations": [
            "bootstrap-alpha",
            "bootstrap-index",
            "ga-activation",
        ],
        "expires_at": "2026-08-27T00:00:00Z",
        "reason": "Temporary single-maintainer initial stable approval exception.",
    }
