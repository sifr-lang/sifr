"""Repository path helpers for verification tooling."""

from __future__ import annotations

from pathlib import Path


PACKAGE_ROOT = Path(__file__).resolve().parent
VERIFICATION_ROOT = PACKAGE_ROOT.parents[1]
REPO_ROOT = VERIFICATION_ROOT.parent
SCHEMAS_DIR = VERIFICATION_ROOT / "schemas"
AREAS_DIR = VERIFICATION_ROOT / "areas"
PROFILES_DIR = VERIFICATION_ROOT / "profiles"
