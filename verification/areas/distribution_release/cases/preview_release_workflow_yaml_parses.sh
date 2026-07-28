#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

preview="${REPO_ROOT}/.github/workflows/preview-release.yml"
publication="${REPO_ROOT}/.github/workflows/release-publication.yml"
prepare="${REPO_ROOT}/.github/workflows/release-publication-prepare.yml"
poller="${REPO_ROOT}/scripts/distribution/poll_site_release_run.sh"
ruby -e 'require "yaml"; YAML.load_file(ARGV.fetch(0))' "${preview}" >/dev/null
ruby -e 'require "yaml"; YAML.load_file(ARGV.fetch(0))' "${publication}" >/dev/null
ruby -e 'require "yaml"; YAML.load_file(ARGV.fetch(0))' "${prepare}" >/dev/null

python3 - "${preview}" "${publication}" "${prepare}" "${poller}" <<'PY'
import pathlib
import sys

preview = pathlib.Path(sys.argv[1]).read_text(encoding="utf-8")
publication = pathlib.Path(sys.argv[2]).read_text(encoding="utf-8")
prepare = pathlib.Path(sys.argv[3]).read_text(encoding="utf-8")
poller = pathlib.Path(sys.argv[4]).read_text(encoding="utf-8")

preview_required = (
    "options:\n          - alpha\n          - beta",
    "version must be a semver prerelease using -alpha.N or -beta.N",
    "SITE_BASE_COMMIT: 07d88cc3c24707e386c5ad73fb0875c06ffd598f",
    "preview release must pin an exact merged website commit",
    "cargo build --release --locked",
    "uses: ./.github/workflows/release-publication.yml",
    "bootstrap-alpha",
    "bootstrap-index",
    "bootstrap_alpha_version",
    "permissions:\n      actions: read\n      contents: write",
    "SIFR_WEBSITE_ACTIONS_TOKEN: ${{ secrets.SIFR_WEBSITE_ACTIONS_TOKEN }}",
)
for fragment in preview_required:
    if fragment not in preview:
        raise SystemExit(f"preview workflow omitted governed caller fragment: {fragment}")
if preview.count("contents: write") != 2:
    raise SystemExit("caller and reusable publish job must grant the required write ceiling exactly")
if "secrets: inherit" in preview:
    raise SystemExit("preview caller must pass only the site Actions credential")
for job in ("validate preview inputs", "build ${{ matrix.target }}"):
    if job not in preview:
        raise SystemExit(f"missing read-only caller job: {job}")
for forbidden in ("-rc.", "alpha|beta|rc", "stable\n          -"):
    if forbidden in preview:
        raise SystemExit(f"preview workflow retained forbidden release input: {forbidden}")

publication_required = (
    "group: sifr-release-index",
    "uses: ./.github/workflows/release-publication-prepare.yml",
    "name: ${{ inputs.governance_mode == 'preview' && 'preview-release' || 'stable-release' }}",
    "actions/runs/${GITHUB_RUN_ID}/approvals",
    "resolve-publication-approvers",
    "--initiator \"${GITHUB_TRIGGERING_ACTOR}\"",
    "generate-schema-bootstrap-index",
    "schema-v2-bootstrap-generation-1.json",
    "run_schema_bootstrap_public_smoke.sh",
    "materialize_schema_bootstrap_evidence.py",
    "release-publication accepts only alpha or beta",
    "version release already exists; preview assets are write-once",
    "version tag already exists; release source identity is write-once",
    '--target "${SOURCE_COMMIT}"',
    "published version tag does not resolve to source_commit",
    "channels-generation-${PROPOSED_GENERATION}.json",
    "gh release upload channels \"${snapshot}\"",
    "Replace only canonical channels.json",
    "--kind site-publication-facts",
    "--clobber",
    "site_base_commit must be an exact commit",
    "repos/${SITE_REPOSITORY}/actions/workflows/${SITE_WORKFLOW}/dispatches",
    "timeout-minutes: 60",
    "poll_site_release_run.sh",
)
for fragment in publication_required:
    if fragment not in publication:
        raise SystemExit(f"publication workflow omitted governed fragment: {fragment}")
if publication.count("--clobber") != 1:
    raise SystemExit("channels.json must be the sole clobbered release asset")
if "gh release upload \"${VERSION}\"" in publication:
    raise SystemExit("version assets must not use a mutable upload path")
if "stable|alpha|beta" in publication or "choices=(\"stable\"" in publication:
    raise SystemExit("stable mutation must remain absent from the preview/bootstrap workflow")
snapshot = publication.index('gh release upload channels "${snapshot}"')
replacement = publication.index("Replace only canonical channels.json")
dispatch = publication.index("Dispatch exact site workflow")
if not snapshot < replacement < dispatch:
    raise SystemExit("snapshot, index replacement, and site dispatch are misordered")

prepare_required = (
    "permissions:\n  actions: read\n  contents: read",
    "prepare governed publication",
    'summary_artifact_name="publication-prepare-${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT}"',
    "name: ${{ steps.summary.outputs.summary_artifact_name }}",
    "one-time bootstrap source is not the exact opaque v1 asset",
    "--kind schema-bootstrap-evidence",
    "retention-days: 30",
    "overwrite: false",
)
for fragment in prepare_required:
    if fragment not in prepare:
        raise SystemExit(f"prepare workflow omitted governed fragment: {fragment}")
if "contents: write" in prepare or "environment:" in prepare:
    raise SystemExit("prepare workflow must remain read-only and unprotected")

poll_required = (
    "poll_deadline=$((SECONDS + deadline_seconds))",
    "query_failures >= 3",
    'timeout --foreground "${remaining_seconds}s" gh api',
    "select(.head_sha == $sha)",
    "select(.created_at >= $since)",
    'actions/runs/${run_id}/cancel',
)
for fragment in poll_required:
    if fragment not in poller:
        raise SystemExit(f"site poller omitted governed fragment: {fragment}")
if not poller.index('poll_error="could not query') < poller.index(
    'actions/runs/${run_id}/cancel'
):
    raise SystemExit("site poller must cancel a matched run after repeated query failure")
PY
