#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

python3 - "${REPO_ROOT}" <<'PY'
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
workflow = (root / ".github/workflows/release-publication.yml").read_text()
prepare = (
    root / ".github/workflows/release-publication-prepare.yml"
).read_text()
orchestrator = (
    root / "scripts/distribution/run_stable_publication.sh"
).read_text()
release = (
    root / "scripts/distribution/publish_stable_release.py"
).read_text()
marketplace = (
    root / "scripts/distribution/publish_marketplace_extension.sh"
).read_text()
smoke = (
    root / "scripts/distribution/run_stable_public_smoke.sh"
).read_text()
signoff = (
    root
    / "verification/areas/distribution_release/governance/stable_publish.py"
).read_text()

for fragment in (
    "- ga-activation",
    "- normal",
    "evidence_commit:",
    "candidate_path:",
    "expected_plan_sha256:",
    "publication_mode:",
    "contents: read",
    "name: mutate governed release",
    "contents: write",
    "name: ${{ inputs.governance_mode == 'preview' && 'preview-release' || 'stable-release' }}",
    "path: stable-source",
    "path: stable-evidence",
    "submodules: recursive",
    "actions/setup-node@820762786026740c76f36085b0efc47a31fe5020",
    "node-version-file: stable-source/editor_integrations/vscode/.node-version",
    "npm ci --ignore-scripts --include=dev --prefix stable-source/editor_integrations/vscode",
    "node_modules/.bin/vsce",
    "Run exact protected stable publication",
    "scripts/distribution/run_stable_publication.sh",
    "VSCE_PAT: ${{ secrets.VSCE_PAT }}",
):
    assert fragment in workflow, fragment
assert workflow.count("\n  publish:\n") == 1
assert "uses: ./.github/workflows/release-publication-prepare.yml" in workflow
assert "site_base_commit:" in prepare
assert "contents: write" not in prepare

ordered = (
    "revalidate \"${work}/governance-initial\"",
    "verify_site_workflow_identity.sh",
    "publish_stable_release.py",
    "publish_marketplace_extension.sh",
    "fetch_governance \"${work}/governance-before-index\"",
    "revalidate \"${work}/governance-before-index\"",
    'upload_or_verify_governance "${snapshot}"',
    "gh release upload channels \"${work}/staged/channels.json\"",
    "generate_site_publication_facts.py",
    "dispatch_stable_site_publication.sh",
    "run_stable_public_smoke.sh",
    "materialize_stable_publication.py signoff",
    'upload_or_verify_governance \\\n  "${work}/stable-release-signoff-${version}-attempt-${publication_attempt}.json"',
)
positions = [orchestrator.index(fragment) for fragment in ordered]
assert positions == sorted(positions)
assert orchestrator.count("verify_site_workflow_identity.sh") == 1
assert "--clobber" in orchestrator
assert orchestrator.count("--clobber") == 1
assert "channels.json" in orchestrator
assert "resolve-publication-approvers" in orchestrator
assert "--environment stable-release" in orchestrator
assert "--single-maintainer-waiver" in orchestrator
assert 'if [[ "${operation}" == "ga-activation" ]]; then' in orchestrator
assert 'elif [[ -n "${approval_waiver}" ]]; then' in orchestrator
assert "--expected-waiver-sha256" in orchestrator
assert "--include-policy" in orchestrator
assert "'.approval_policy.mode'" in orchestrator
assert "--approval-mode" in orchestrator
assert "--approval-waiver-sha256" in orchestrator
assert '"${workflow_ref}" = "refs/heads/main"' in orchestrator
assert "stable mutation must run from protected main HEAD" in orchestrator
assert "candidate source:${candidate_source_commit}" in orchestrator
assert "candidate evidence:${evidence_commit}" in orchestrator
assert "merge-base --is-ancestor" in orchestrator
assert 'publication_state}" == "pending"' in orchestrator
assert 'publication_state}" != "activated"' in orchestrator

for fragment in (
    "releases/tags/{version}",
    "git/ref/tags/{version}",
    "releases/{release_id}/assets?per_page=100&page={page}",
    "releases/assets/{asset_id}",
    "allow_missing=True",
    "allow_missing=False",
    "initial publication requires an absent release and tag",
    "published bytes drifted",
):
    assert fragment in release, fragment
assert "--clobber" not in release

for fragment in (
    "Microsoft.VisualStudio.Services.VSIXPackage",
    '"${VSCE_BIN}" publish --packagePath',
    "--packagePath",
    "verify_marketplace_vsix.py",
    "Marketplace raw asset did not converge",
):
    assert fragment in marketplace, fragment
assert marketplace.index("verify_marketplace_vsix.py") < marketplace.index(
    '"${VSCE_BIN}" publish --packagePath'
)

for fragment in (
    "https://sifr.sh/install",
    "https://sifr.sh/install/stable",
    "releases/download/${version}/${name}",
    "self update --dry-run --format json",
    "resolved_channel == \"stable\"",
    "marketplace.vsix",
):
    assert fragment in smoke, fragment
for fragment in (
    '"site_publication": site_publication',
    "site_run_path",
    "post_publication_smoke",
):
    assert fragment in signoff, fragment
orchestrator_test = (
    root
    / "verification/areas/distribution_release/governance/stable_orchestrator_selftest.py"
).read_text()
assert "test_orchestrator_rejects_unmerged_candidate" in orchestrator_test
assert "candidate source commit must be merged into protected main" in orchestrator_test
for fragment in (
    'site_token="${SITE_TOKEN}"',
    'marketplace_pat="${VSCE_PAT:-}"',
    "unset SITE_TOKEN VSCE_PAT",
    'GH_TOKEN="" SITE_TOKEN="" VSCE_PAT="${marketplace_pat}"',
    'GH_TOKEN="" SITE_TOKEN="" VSCE_PAT=""',
):
    assert fragment in orchestrator, fragment
PY
