#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

workflow="${REPO_ROOT}/.github/workflows/release-publication-prepare.yml"
fetcher="${REPO_ROOT}/scripts/distribution/fetch_qualification_artifacts.py"

ruby -e 'require "yaml"; YAML.load_file(ARGV.fetch(0))' "${workflow}" >/dev/null

python3 - "${workflow}" "${fetcher}" <<'PY'
import pathlib
import sys

workflow = pathlib.Path(sys.argv[1]).read_text(encoding="utf-8")
fetcher = pathlib.Path(sys.argv[2]).read_text(encoding="utf-8")

for fragment in (
    "permissions:\n  actions: read\n  contents: read",
    "evidence_commit:",
    "candidate_path:",
    "expected_plan_sha256:",
    "publication_mode:",
    "proposed_generation:",
    "ref: ${{ inputs.evidence_commit }}",
    "path: stable-evidence",
    "path: stable-source",
    "path: governance-source",
    "ref: ${{ github.sha }}",
    "submodules: recursive",
    "persist-credentials: false",
    "fetch_qualification_artifacts.py",
    "allocate_release_generation.py",
    "prepare/history",
    "--paginate",
    "assets?per_page=100",
    "qualification-artifact-index.json",
    'candidate_version="${CANDIDATE_PATH##*/}"',
    'source_commit="$(jq -er \'.source_commit\' "${plan}")"',
    "ref: ${{ steps.stable.outputs.source_commit }}",
    "prepare-stable-publication",
    "governance-source/scripts/distribution/release_governance.py",
    '--operation "${GOVERNANCE_MODE}"',
    '--evidence-commit "${EVIDENCE_COMMIT}"',
    '--expected-plan-sha256 "${EXPECTED_PLAN_SHA256}"',
    "--snapshot-root prepare/history",
    '--proposed-generation "${proposed_generation}"',
    "prepare/summary.json",
    "release_report_sha256:",
    "qualification_sha256:",
    "live_index_sha256:",
    "proposed_index_sha256:",
    "      source_commit:\n        value: ${{ jobs.prepare.outputs.source_commit }}",
    "      proposed_generation:\n        value: ${{ jobs.prepare.outputs.proposed_generation }}",
    "retention-days: 30",
    "overwrite: false",
):
    assert fragment in workflow, fragment

for forbidden in (
    "contents: write",
    "packages: write",
    "id-token: write",
    "${{ secrets.",
    "vsce publish",
    "gh release upload",
    "gh run download",
    "unzip ",
):
    assert forbidden not in workflow, forbidden

stable_step = workflow.split(
    "- name: Prepare exact stable publication inputs",
    1,
)[1].split("- name: Bind prepare outputs", 1)[0]
assert "ga-activation" in stable_step
assert "normal" in stable_step
assert "bootstrap-alpha" not in stable_step
assert "stable-source/scripts/distribution/release_governance.py" not in stable_step
assert '[[ "${CHANNEL}" =~ ^(alpha|beta)$ ]]' in workflow

input_contract = workflow.split("inputs:", 1)[1].split("outputs:", 1)[0]
assert "proposed_generation:" not in input_contract
for fragment in (
    "/actions/runs/{run_id}/attempts/{run_attempt}",
    "/actions/artifacts/{artifact_id}",
    'metadata.get("expired") is not False',
    'metadata.get("expires_at") != expires_at',
    'workflow_run.get("id") != run_id',
    "extract_artifact(",
    "verify_transported_artifacts(qualification, staging)",
):
    assert fragment in fetcher, fragment
PY
