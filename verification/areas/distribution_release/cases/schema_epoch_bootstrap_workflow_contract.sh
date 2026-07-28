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
    "LEGACY_INDEX_SHA256",
    "LEGACY_INDEX_SIZE_BYTES",
    "BOOTSTRAP_GENERATION = 1",
    "approval by someone other than",
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
for surface in (prepare, publication, bootstrap, evidence_schema):
    assert legacy_digest in surface
for surface, size_fragment in (
    (prepare, '= "105"'),
    (publication, '= "105"'),
    (bootstrap, "LEGACY_INDEX_SIZE_BYTES = 105"),
    (evidence_schema, '"size_bytes": {"const": 105}'),
):
    assert size_fragment in surface
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
    for surface in (prepare, publication, bootstrap):
        assert forbidden not in surface.lower()
PY
