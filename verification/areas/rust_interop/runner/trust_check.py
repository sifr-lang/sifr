"""Trust evidence check skeletons for Rust interop fixtures."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class TrustEvidence:
    fixture_id: str
    pre_execution_checks: tuple[str, ...]
    post_execution_checks: tuple[str, ...]


def empty_trust_evidence(fixture_id: str) -> TrustEvidence:
    """Return an empty evidence record until Cargo metadata integration lands."""
    return TrustEvidence(
        fixture_id=fixture_id,
        pre_execution_checks=(),
        post_execution_checks=(),
    )
