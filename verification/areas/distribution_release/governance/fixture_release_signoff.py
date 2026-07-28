"""Local-only stable release sign-off evidence for incident fixtures."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .common import canonical_json_bytes, sha256_bytes
from .incident_planner import IncidentMutation
from .release_plan import validate_release_signoff


def write_fixture_release_signoff(
    *,
    governance_root: Path,
    realized: dict[str, Any],
    mutation: IncidentMutation,
    attempts: list[dict[str, Any]],
    site_facts_sha256: str,
) -> tuple[Path, bytes]:
    """Materialize a valid local sign-off asset for a roll-forward release."""
    version = mutation.successor_version
    release = realized["releases"][version]
    fixture_evidence_sha256 = sha256_bytes(
        canonical_json_bytes(
            {
                "fixture": "non-deploying-incident-roll-forward",
                "version": version,
                "generation": realized["generation"],
                "plan_sha256": mutation.successor_plan_sha256,
            }
        )
    )
    release_signoff = {
        "schema_version": 2,
        "version": version,
        "plan_sha256": mutation.successor_plan_sha256,
        "attempts": [
            {
                "run_id": attempts[-1]["run_id"],
                "mode": attempts[-1]["mode"],
                "approver": attempts[-1]["approver"],
                "status": "completed",
                "mutations": [
                    {
                        "kind": "release-index",
                        "identity": f"generation-{realized['generation']}",
                        "sha256": sha256_bytes(canonical_json_bytes(realized)),
                    }
                ],
            }
        ],
        "published_assets": {
            f"sifr-installer-{version}": release["installer_sha256"],
        },
        "marketplace": {
            "publisher": "sifr-fixture",
            "extension": "sifr",
            "version": version,
            "vsix_sha256": fixture_evidence_sha256,
        },
        "channel_generation": realized["generation"],
        "site_publication": {
            "repository": "sifr-lang/sifr-website",
            "workflow": "release-site.yml",
            "run_id": attempts[-1]["run_id"],
            "deployed_commit": "1" * 40,
        },
        "site_facts_sha256": site_facts_sha256,
        "post_publication_smoke": [
            {
                "id": f"fixture-smoke-{index}",
                "status": "pass",
                "sha256": sha256_bytes(
                    f"fixture-smoke-{version}-{index}\n".encode()
                ),
            }
            for index in range(1, 5)
        ],
    }
    validate_release_signoff(release_signoff)
    encoded = canonical_json_bytes(release_signoff)
    path = governance_root / (
        f"stable-release-signoff-{version}-"
        f"{mutation.successor_plan_sha256[:16]}.json"
    )
    return path, encoded
