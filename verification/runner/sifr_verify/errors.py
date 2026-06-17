"""Typed verification runner errors."""

from __future__ import annotations


class VerificationError(RuntimeError):
    """Base error for runner-controlled validation failures."""


class SchemaError(VerificationError):
    """Schema rules or data validation failed."""


class DiscoveryError(VerificationError):
    """Area discovery failed."""
