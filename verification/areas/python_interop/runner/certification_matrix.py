from __future__ import annotations

from collections import Counter
from typing import Any

from import_matrix import PackageEntry

CORE_TIER1A_PACKAGES = {
    "biip",
    "boto3",
    "botocore",
    "cffi",
    "confluent-kafka",
    "cryptography",
    "google-genai",
    "httpx",
    "numpy",
    "openai",
    "pandas",
    "polars",
    "psycopg",
    "pyarrow",
    "pydantic",
    "pydantic-core",
    "redis",
    "requests",
    "schwifty",
    "sqlalchemy",
    "torch",
}

def validate_certification_policy(entries: list[PackageEntry]) -> None:
    tier1a = {entry.name for entry in entries if entry.tier == "tier1" and entry.gate == "tier1a"}
    missing = sorted(CORE_TIER1A_PACKAGES.difference(tier1a))
    extra = sorted(tier1a.difference(CORE_TIER1A_PACKAGES))
    if missing or extra:
        parts = []
        if missing:
            parts.append(f"missing tier1a package(s): {', '.join(missing)}")
        if extra:
            parts.append(f"unexpected tier1a package(s): {', '.join(extra)}")
        raise SystemExit("; ".join(parts))

    for entry in entries:
        if entry.native and "native" not in entry.groups:
            raise SystemExit(f"native package {entry.name} must include the native group")
        if entry.host_dependent and not entry.skip_reason:
            raise SystemExit(f"host-dependent package {entry.name} must declare skip-reason")
        if entry.tier == "tier4" and not entry.host_dependent:
            raise SystemExit(f"tier4 package {entry.name} must be host-dependent")
        if entry.tier in {"tier2", "tier3"} and entry.host_dependent:
            raise SystemExit(f"{entry.tier} package {entry.name} must be deterministic")
        if not entry.import_roots:
            raise SystemExit(f"package {entry.name} must expose at least one import root")


def build_certification_report(entries: list[PackageEntry]) -> dict[str, Any]:
    certified = [entry for entry in entries if not entry.host_dependent]
    skipped = [entry for entry in entries if entry.host_dependent]
    tier_counts = Counter(entry.tier for entry in entries)
    gate_counts = Counter(entry.gate for entry in entries if entry.gate is not None)
    group_counts = Counter(group for entry in entries for group in entry.groups)

    return {
        "selected_packages": len(entries),
        "certified_packages": len(certified),
        "host_dependent_skips": len(skipped),
        "tier_counts": dict(sorted(tier_counts.items())),
        "gate_counts": dict(sorted(gate_counts.items())),
        "group_counts": dict(sorted(group_counts.items())),
        "packages": [package_payload(entry) for entry in sorted(entries, key=entry_sort_key)],
    }


def package_payload(entry: PackageEntry) -> dict[str, Any]:
    payload: dict[str, Any] = {
        "name": entry.name,
        "tier": entry.tier,
        "groups": list(entry.groups),
        "import_roots": list(entry.import_roots),
        "status": "host-dependent-skip" if entry.host_dependent else "certified",
    }
    if entry.gate is not None:
        payload["gate"] = entry.gate
    if entry.native:
        payload["native"] = True
    if entry.host_dependent:
        payload["skip_reason"] = entry.skip_reason
    return payload


def entry_sort_key(entry: PackageEntry) -> tuple[str, str, str]:
    return (entry.tier, entry.gate or "", entry.name)
