#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

python3 - "${REPO_ROOT}" <<'PY'
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
workflow = (root / ".github/workflows/release-publication.yml").read_text()
prepare = (root / ".github/workflows/release-publication-prepare.yml").read_text()
runner = (root / "scripts/distribution/run_incident_publication.sh").read_text()
revalidate = (
    root / "scripts/distribution/revalidate_incident_publication.py"
).read_text()
prepare_contract = (
    root
    / "verification/areas/distribution_release/governance/incident_prepare.py"
).read_text()
signoff = (
    root
    / "verification/areas/distribution_release/governance/incident_publish.py"
).read_text()

for fragment in (
    "- rollback",
    "- incident-roll-forward",
    "incident_commit:",
    "incident_path:",
    "expected_request_sha256:",
    "Run exact protected incident publication",
    "scripts/distribution/run_incident_publication.sh",
    "path: incident-evidence",
    "name: mutate governed release",
):
    assert fragment in workflow, fragment
assert workflow.count("\n  publish:\n") == 1
assert "sifr-release-index" in workflow
assert workflow.count("contents: write") == 1

for fragment in (
    "Prepare exact rollback inputs",
    "prepare-incident-publication",
    "--incident-commit \"${INCIDENT_COMMIT}\"",
    "--incident-path \"${INCIDENT_PATH}\"",
    "--expected-request-sha256 \"${EXPECTED_REQUEST_SHA256}\"",
    "incident evidence commit must be merged into protected main",
    "candidate evidence commit must be merged into protected main",
    "prepare/summary.json",
):
    assert fragment in prepare, fragment
for forbidden in (
    "contents: write",
    "${{ secrets.",
    "gh release upload",
    "vsce publish",
):
    assert forbidden not in prepare, forbidden

ordered = (
    'revalidate "${work}/governance-initial"',
    "resolve-publication-approvers",
    'verify_retained_version \\\n  "${affected_version}"',
    "materialize_incident_publication.py stage",
    'upload_or_verify_governance \\\n  "${request_asset}"',
    'revalidate "${work}/governance-before-index"',
    'upload_or_verify_governance "${snapshot}"',
    'gh release upload channels "${work}/incident-staged/channels.json"',
    "dispatch_stable_site_publication.sh",
    "run_stable_public_smoke.sh",
    "run_incident_public_recovery.sh",
    "materialize_incident_publication.py signoff",
)
positions = [runner.index(fragment) for fragment in ordered]
assert positions == sorted(positions)
assert runner.count("--clobber") == 1
assert "--single-maintainer-waiver" not in runner
assert "--expected-waiver-sha256" not in runner
for fragment in (
    '"${workflow_ref}" = "refs/heads/main"',
    "mutation must run from protected main HEAD",
    "candidate evidence:${candidate_commit}",
    "candidate source:${candidate_source_commit}",
    '--mode "${mode}"',
    "--compiler-version \"${successor_version}\"",
    'publication_state}" == "pending"',
    'publication_state}" != "activated"',
    "stable-incident-signoff-${incident_id}-attempt-${publication_attempt}.json",
):
    assert fragment in runner, fragment
assert runner.index("publish_stable_release.py") < runner.index(
    'gh release upload channels "${work}/incident-staged/channels.json"'
)
assert runner.index("publish_marketplace_extension.sh") < runner.index(
    'gh release upload channels "${work}/incident-staged/channels.json"'
)

for fragment in (
    "canonical_json_bytes(recomputed) != summary_bytes",
    "materialize_incident_prepare",
):
    assert fragment in revalidate, fragment
for fragment in (
    "validate_incident_evidence_commit",
    "_recover_realized_mutation",
    "does not contain the realized incident predecessor",
    "release_prepare",
):
    assert fragment in prepare_contract, fragment
for fragment in (
    '"release_signoff_sha256": release_signoff_sha256',
    "**site",
    "incident-recovery.json",
    "withdrawal_evidence_path",
):
    assert fragment in signoff, fragment
PY
