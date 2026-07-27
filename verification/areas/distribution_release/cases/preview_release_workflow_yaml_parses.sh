#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

preview="${REPO_ROOT}/.github/workflows/preview-release.yml"
publication="${REPO_ROOT}/.github/workflows/release-publication.yml"
ruby -e 'require "yaml"; YAML.load_file(ARGV.fetch(0))' "${preview}" >/dev/null
ruby -e 'require "yaml"; YAML.load_file(ARGV.fetch(0))' "${publication}" >/dev/null

python3 - "${preview}" "${publication}" <<'PY'
import pathlib
import sys

preview = pathlib.Path(sys.argv[1]).read_text(encoding="utf-8")
publication = pathlib.Path(sys.argv[2]).read_text(encoding="utf-8")

preview_required = (
    "options:\n          - alpha\n          - beta",
    "version must be a semver prerelease using -alpha.N or -beta.N",
    "SITE_BASE_COMMIT: 721bceca795a79a03af74ccb707d117a6f031f38",
    "preview release must pin an exact merged website commit",
    "cargo build --release --locked",
    "uses: ./.github/workflows/release-publication.yml",
)
for fragment in preview_required:
    if fragment not in preview:
        raise SystemExit(f"preview workflow omitted governed caller fragment: {fragment}")
if "contents: write" in preview:
    raise SystemExit("preview caller must not own release mutation permissions")
for forbidden in ("-rc.", "alpha|beta|rc", "stable\n          -"):
    if forbidden in preview:
        raise SystemExit(f"preview workflow retained forbidden release input: {forbidden}")

publication_required = (
    "group: sifr-release-index",
    "release-publication accepts only alpha or beta",
    "version release already exists; preview assets are write-once",
    "channels-generation-${PROPOSED_GENERATION}.json",
    "gh release upload channels \"${snapshot}\"",
    "Replace only canonical channels.json",
    "--clobber",
    "site_base_commit must be an exact commit",
    "repos/${SITE_REPOSITORY}/actions/workflows/${SITE_WORKFLOW}/dispatches",
    "select(.head_sha == $sha)",
    "select(.created_at >= $since)",
    "seq 1 120",
    "actions/runs/${site_run_id}/cancel",
)
for fragment in publication_required:
    if fragment not in publication:
        raise SystemExit(f"publication workflow omitted governed fragment: {fragment}")
if publication.count("--clobber") != 1:
    raise SystemExit("channels.json must be the sole clobbered release asset")
if "gh release upload \"${VERSION}\"" in publication:
    raise SystemExit("version assets must not use a mutable upload path")
if "stable|alpha|beta" in publication or "choices=(\"stable\"" in publication:
    raise SystemExit("stable mutation must remain absent from milestone 40.2")
snapshot = publication.index('gh release upload channels "${snapshot}"')
replacement = publication.index("Replace only canonical channels.json")
dispatch = publication.index("Dispatch exact site workflow")
if not snapshot < replacement < dispatch:
    raise SystemExit("snapshot, index replacement, and site dispatch are misordered")
PY
