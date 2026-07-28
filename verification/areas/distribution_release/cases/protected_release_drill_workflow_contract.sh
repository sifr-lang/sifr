#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

publication="${REPO_ROOT}/.github/workflows/release-publication.yml"
drill="${REPO_ROOT}/.github/workflows/release-publication-drill.yml"

ruby -e 'require "yaml"; YAML.load_file(ARGV.fetch(0))' "${publication}" >/dev/null
ruby -e 'require "yaml"; YAML.load_file(ARGV.fetch(0))' "${drill}" >/dev/null

python3 - "${publication}" "${drill}" "${REPO_ROOT}" <<'PY'
import pathlib
import sys

publication = pathlib.Path(sys.argv[1]).read_text(encoding="utf-8")
drill = pathlib.Path(sys.argv[2]).read_text(encoding="utf-8")
root = pathlib.Path(sys.argv[3])
sys.path.insert(0, str(root))
from verification.areas.distribution_release.governance.common import (
    PRODUCTION_CREDENTIAL_NAMES,
)
planner = (
    root
    / "verification/areas/distribution_release/governance/stable_planner.py"
).read_text(encoding="utf-8")
drill_selftest = (
    root
    / "verification/areas/distribution_release/governance/protected_drill_selftest.py"
).read_text(encoding="utf-8")

for fragment in (
    "workflow_dispatch:",
    "- drill-publication",
    "if: ${{ !startsWith(inputs.governance_mode, 'drill-') }}",
    "if: ${{ startsWith(inputs.governance_mode, 'drill-') }}",
    "uses: ./.github/workflows/release-publication-drill.yml",
    "drill-first-ga",
    "drill-rollback",
    "permissions:\n      contents: read",
    "'sifr-release-drill' || 'sifr-release-index'",
    "mode: ${{ inputs.governance_mode }}",
):
    assert fragment in publication, fragment
assert "secrets: inherit" not in publication
site_secret = publication.split("SIFR_WEBSITE_ACTIONS_TOKEN:", 1)[1].split(
    "\n\npermissions:", 1
)[0]
assert "required: true" in site_secret

for fragment in (
    "name: stable-release-drill",
    "permissions:\n  contents: read",
    "persist-credentials: false",
    "unshare --net --mount-proc",
    "governance.protected_drill_selftest",
    "--kind protected-drill-evidence",
    '--expected-drill-scenario "${DRILL_SCENARIO}"',
    "stable-release-drill-${{ github.run_id }}-${{ github.run_attempt }}",
    "retention-days: 30",
    "overwrite: false",
    "drill-publication) scenario=publication",
    "drill-rollback) scenario=rollback",
    "drill-first-ga) scenario=first-ga",
):
    assert fragment in drill, fragment
for forbidden in (
    "${{ secrets.",
    "gh release",
    "vsce publish",
    "/dispatches",
    "contents: write",
):
    assert forbidden not in drill, forbidden
credential_boundary = drill.split("for credential in", 1)[1].split("; do", 1)[0]
credential_scrub = drill.split("sudo env", 1)[1].split("unshare --net", 1)[0]
for credential in PRODUCTION_CREDENTIAL_NAMES:
    assert credential in credential_boundary, credential
    assert f"-u {credential}" in credential_scrub, credential

for fragment in (
    "materialize_stable_mutation",
    "validate_release_plan",
    "propose_stable_release",
    "expected_generation",
    "expected_sha256",
):
    assert fragment in planner, fragment
for fragment in (
    "test_ga_activation",
    "test_normal_successor",
    "test_rollback_burns_generation_and_resumes",
    "test_site_timeout_resumes_without_second_index_mutation",
    "test_first_ga_incident_roll_forward",
    "PRODUCTION_CREDENTIAL_NAMES",
    '"external_network": "blocked"',
    "validate_drill_evidence",
):
    assert fragment in drill_selftest, fragment
PY
