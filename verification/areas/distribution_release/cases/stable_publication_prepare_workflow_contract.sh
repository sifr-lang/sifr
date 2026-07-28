#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

workflow="${REPO_ROOT}/.github/workflows/release-publication-prepare.yml"

ruby -e 'require "yaml"; YAML.load_file(ARGV.fetch(0))' "${workflow}" >/dev/null

python3 - "${workflow}" <<'PY'
import pathlib
import sys

workflow = pathlib.Path(sys.argv[1]).read_text(encoding="utf-8")

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
    "submodules: recursive",
    "persist-credentials: false",
    "/actions/runs/${run_id}/attempts/${run_attempt}",
    "/actions/artifacts/${artifact_id}",
    "group_by(.id)",
    "extract_github_artifact.py",
    "--expected-uncompressed-bytes",
    "qualification-artifact-index.json",
    'candidate_version="${CANDIDATE_PATH##*/}"',
    "workflow_artifact_name",
    "prepare-stable-publication",
    '--operation "${GOVERNANCE_MODE}"',
    '--evidence-commit "${EVIDENCE_COMMIT}"',
    '--expected-plan-sha256 "${EXPECTED_PLAN_SHA256}"',
    '--proposed-generation "${PROPOSED_GENERATION}"',
    "prepare/summary.json",
    "release_report_sha256:",
    "qualification_sha256:",
    "live_index_sha256:",
    "proposed_index_sha256:",
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
PY
