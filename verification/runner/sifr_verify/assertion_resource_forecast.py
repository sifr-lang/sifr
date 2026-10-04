"""Select prospective assertion allocations without changing suite execution."""
from __future__ import annotations

from .paths import REPO_ROOT
from .schemas import load_json

SQL_BUILD_ALLOCATION = "sql-build-qualification"
# Prospective request for the admitted clean-build callback only; CPU-clamped.
SQL_BUILD_CARGO_WORKERS = 2


def assertion_allocation(name: str, profile: dict) -> str:
    """Only the exact SQL area with its declared clean-build suite gets this budget."""
    generic = "remaining-assertions"
    if name != "area_sql_platform":
        return generic
    rows = [row for row in profile.get("selected_areas", [])
            if isinstance(row, dict) and row.get("area") == "sql_platform"]
    if len(rows) != 1:
        return generic
    suites = rows[0].get("suites")
    if (not isinstance(suites, list) or not all(isinstance(suite, str) for suite in suites)
            or len(suites) != len(set(suites)) or "build-qualification" not in suites):
        return generic
    manifest = load_json(REPO_ROOT / "verification/areas/sql_platform/manifest.json")
    declared = {suite["name"] for suite in manifest["suites"]}
    return SQL_BUILD_ALLOCATION if set(suites) <= declared else generic
