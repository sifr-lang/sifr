"""Native-link evidence probe skeletons for Rust interop fixtures."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class NativeLinkEvidence:
    fixture_id: str
    native_links: tuple[str, ...]
    build_script_outputs: tuple[str, ...]


def empty_native_evidence(fixture_id: str) -> NativeLinkEvidence:
    """Return an empty native evidence record until tier 3 fixtures are implemented."""
    return NativeLinkEvidence(
        fixture_id=fixture_id,
        native_links=(),
        build_script_outputs=(),
    )
