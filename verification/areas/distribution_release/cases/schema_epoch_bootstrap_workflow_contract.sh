#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

python3 - "${REPO_ROOT}" <<'PY'
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
caller = (root / ".github/workflows/preview-release.yml").read_text(encoding="utf-8")
publication = (root / ".github/workflows/release-publication.yml").read_text(encoding="utf-8")
prepare = (root / ".github/workflows/release-publication-prepare.yml").read_text(
    encoding="utf-8"
)
recovery = (
    root / ".github/workflows/schema-bootstrap-recovery.yml"
).read_text(encoding="utf-8")
recovery_prepare = (
    root / "scripts/distribution/prepare_schema_bootstrap_recovery.sh"
).read_text(encoding="utf-8")
bootstrap = (
    root
    / "verification/areas/distribution_release/governance/schema_bootstrap.py"
).read_text(encoding="utf-8")
evidence_schema = (
    root
    / "verification/areas/distribution_release/schemas/schema_epoch_bootstrap_evidence.schema.json"
).read_text(encoding="utf-8")
smoke = (
    root / "scripts/distribution/run_schema_bootstrap_public_smoke.sh"
).read_text(encoding="utf-8")
alpha = (
    root / "scripts/distribution/fetch_schema_bootstrap_alpha.sh"
).read_text(encoding="utf-8")
beta = (
    root / "scripts/distribution/fetch_schema_bootstrap_beta.sh"
).read_text(encoding="utf-8")
assets = (
    root / "scripts/distribution/verify_release_publication_assets.sh"
).read_text(encoding="utf-8")

for fragment in (
    "bootstrap-alpha",
    "bootstrap-index",
    "bootstrap_alpha_version",
    "test \"${INPUT_CHANNEL}\" = \"alpha\"",
    "test \"${INPUT_CHANNEL}\" = \"beta\"",
):
    assert fragment in caller, fragment
for fragment in (
    "uses: ./.github/workflows/release-publication-prepare.yml",
    "name: ${{ needs.prepare.outputs.summary_artifact_name }}",
    "name: ${{ inputs.governance_mode == 'preview' && 'preview-release' || 'stable-release' }}",
    "actions/runs/${GITHUB_RUN_ID}/approvals",
    "--initiator \"${GITHUB_TRIGGERING_ACTOR}\"",
    "--single-maintainer-waiver \"${SINGLE_MAINTAINER_APPROVAL_WAIVER}\"",
    "SINGLE_MAINTAINER_APPROVAL_WAIVER_SHA256: b9630cc060ca281946da76a9cb9bc67564759c8d5446b6a33157a7d138080008",
    'test "${waiver_sha256}" = "${SINGLE_MAINTAINER_APPROVAL_WAIVER_SHA256}"',
    "--expected-waiver-sha256 \"${SINGLE_MAINTAINER_APPROVAL_WAIVER_SHA256}\"",
    "--include-policy",
    "jq -r '.approval_policy.mode'",
    "--approval-mode \"${APPROVAL_MODE}\"",
    "--approval-waiver-sha256 \"${APPROVAL_WAIVER_SHA256}\"",
    "governed index changed after read-only prepare",
    "staged alpha evidence changed after read-only prepare",
    "generate-schema-bootstrap-index",
    "channels-generation-${PROPOSED_GENERATION}.json",
    "Retain protected alpha bootstrap evidence",
    "Run protected public schema-bootstrap smoke",
    "Retain final protected schema-bootstrap evidence",
    "schema-v2-bootstrap-generation-1.json",
    "--out publication/bootstrap-smoke",
    "--smoke-dir publication/bootstrap-smoke",
):
    assert fragment in publication, fragment
for fragment in (
    "verify_release_publication_assets.sh",
    "protected-prepare/summary.json",
):
    assert fragment in publication, fragment
for fragment in (
    "generate_version_installer.sh",
    "asset set is incomplete or unexpected",
    "bytes differ from read-only prepare",
):
    assert fragment in assets, fragment
assert publication.index("Publish write-once version release and verify assets") < publication.index(
    "Retain protected alpha bootstrap evidence"
)
assert publication.index('gh release upload channels "${snapshot}"') < publication.index(
    "Replace only canonical channels.json"
)
assert publication.index("Replace only canonical channels.json") < publication.index(
    "Dispatch and await exact site workflow"
)
assert publication.index("Dispatch and await exact site workflow") < publication.index(
    "Run protected public schema-bootstrap smoke"
)
assert publication.index("Run protected public schema-bootstrap smoke") < publication.index(
    "Retain final protected schema-bootstrap evidence"
)
assert publication.count("--clobber") == 1
assert "contents: write" not in prepare
assert "environment:" not in prepare
for fragment in (
    "actions: read",
    "contents: read",
    "one-time bootstrap source is not the exact opaque v1 asset",
    'summary_artifact_name="publication-prepare-${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT}"',
    "name: ${{ steps.outputs.outputs.summary_artifact_name }}",
    "retention-days: 30",
    "overwrite: false",
):
    assert fragment in prepare, fragment
for fragment in (
    "concurrency:",
    "group: sifr-release-index",
    "prepare exact bootstrap recovery",
    "recover site and retain bootstrap evidence",
    "environment:",
    "name: stable-release",
    "plans/releases/schema-bootstrap-recovery/prepare-summary-${{ inputs.original_run_id }}-${{ inputs.original_run_attempt }}.json",
    "actions/runs/${ORIGINAL_RUN_ID}/approvals",
    "actions/runs/${GITHUB_RUN_ID}/approvals",
    "--operation bootstrap-index",
    "--expected-waiver-sha256 \"${WAIVER_SHA256}\"",
    "failed site run identity/status drifted",
    'gh run view "${FAILED_SITE_RUN_ID}"',
    "failed site run inputs drifted",
    "DISPATCHER_INDEX_SHA256:",
    "DISPATCHER_BETA_SHA256:",
    "STABLE_SITE_FACTS_SHA256: none",
    "prepare_schema_bootstrap_recovery.sh",
    "cmp protected-prepare/summary.json publication/summary.json",
    "final generation-1 bootstrap evidence already exists",
    "Dispatch exact site recovery without another index mutation",
    "run_schema_bootstrap_public_smoke.sh",
    "--legacy-index-sha256",
    "--legacy-index-size-bytes 105",
    "--recovery-json publication/recovery.json",
    "recovery-approval-decision.json",
    "publication/site-run.json",
    "schema-v2-bootstrap-generation-1.json",
):
    assert fragment in recovery, fragment
assert publication.count("--legacy-index publication/current-channels.json") == 2
assert "--legacy-index-sha256" not in publication
assert "--legacy-index-size-bytes" not in publication
assert recovery.count("--legacy-index-sha256") == 1
assert recovery.count("--legacy-index-size-bytes") == 1
assert recovery.index("prepare exact bootstrap recovery") < recovery.index(
    "recover site and retain bootstrap evidence"
)
assert recovery.index(
    "Dispatch exact site recovery without another index mutation"
) < recovery.index("Run protected public schema-bootstrap smoke")
assert recovery.index("Run protected public schema-bootstrap smoke") < recovery.index(
    "Retain final protected schema-bootstrap evidence"
)
for forbidden in (
    "--clobber",
    "gh release create",
    "channels-generation-${",
    "Replace only canonical channels.json",
    "publication-prepare-${{ inputs.original_run_id }}-${{ inputs.original_run_attempt }}",
):
    assert forbidden not in recovery
for fragment in (
    "original prepare summary digest drifted",
    "generation 1 snapshot and live index drifted",
    "generation 1 digest drifted",
    "alpha evidence digest drifted",
    "published beta assets drifted from prepare",
    "original release plan is not reproducible",
    "site publication facts are not reproducible",
    'operation: "schema-bootstrap-index-recovery"',
):
    assert fragment in recovery_prepare, fragment
for fragment in (
    "tag source mismatch",
    "immutable asset set drifted",
    "checksum drifted",
    "public release disagrees with generation 1",
):
    assert fragment in beta, fragment
for fragment in (
    "LEGACY_INDEX_SHA256",
    "LEGACY_INDEX_SIZE_BYTES",
    "BOOTSTRAP_GENERATION = 1",
    "requires an authorized",
    "single-maintainer waiver requires only the initiating owner",
    '"ga_status": "preview"',
    '"channels": {',
):
    assert fragment in bootstrap, fragment
assert '"stable":' not in bootstrap
for fragment in (
    "https://sifr.sh/install",
    "https://sifr.sh/install/stable",
    "SIFR_TEST_CHANNEL_METADATA_PATH",
    "SIFR_SYSROOT_INSTALL_DIR",
    'test -z "${SIFR_TEST_CHANNEL_METADATA_PATH:-}"',
    "self update --dry-run --format json",
    "stable channel installs require active GA metadata",
    '${out}/governance-index.txt',
    '${out}/dispatcher-default.txt',
    '${out}/dispatcher-stable-rejection.txt',
    '${out}/installed-self-update.txt',
):
    assert fragment in smoke, fragment
assert "unset SIFR_TEST_CHANNEL_METADATA_PATH" not in smoke
assert "SIFR_SYSROOT_DIR" not in smoke
assert smoke.index('test -z "${SIFR_TEST_CHANNEL_METADATA_PATH:-}"') < smoke.index(
    "download_until_matches()"
)
legacy_digest = "71b3243925670f56dc510b8f45b6614a622f58097a0fea9492f61d20dc4bf9ef"
for surface in (
    prepare,
    publication,
    recovery,
    recovery_prepare,
    bootstrap,
    evidence_schema,
):
    assert legacy_digest in surface
for surface, size_fragment in (
    (prepare, '= "105"'),
    (publication, '= "105"'),
    (bootstrap, "LEGACY_INDEX_SIZE_BYTES = 105"),
    (evidence_schema, '"size_bytes": {"const": 105}'),
):
    assert size_fragment in surface
for fragment in (
    '"recovery": {',
    '"failed_site_run_id"',
    '"site_run_id"',
    '{"not": {"required": ["recovery"]}}',
):
    assert fragment in evidence_schema, fragment
for fragment in (
    "immutable asset set drifted",
    "asset digest drifted",
    "release record is not reproducible",
):
    assert fragment in alpha, fragment
for forbidden in (
    "bootstrap_channel_metadata.py",
    "migrate",
    "fallback",
):
    for surface in (prepare, publication, recovery, recovery_prepare, bootstrap):
        assert forbidden not in surface.lower()
PY
