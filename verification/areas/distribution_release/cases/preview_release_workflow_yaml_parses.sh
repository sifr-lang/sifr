#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

ruby -e 'require "yaml"; YAML.load_file(ARGV.fetch(0))' \
  "${REPO_ROOT}/.github/workflows/preview-release.yml" >/dev/null

ruby - "${REPO_ROOT}/.github/workflows/preview-release.yml" <<'RUBY'
require "yaml"

workflow = YAML.load_file(ARGV.fetch(0))
jobs = workflow.fetch("jobs")
validate_run = jobs.fetch("validate").fetch("steps").find {
  |step| step["name"] == "Validate channel and version"
}.fetch("run")
unless validate_run.include?("alpha|beta") &&
       validate_run.include?("stable-looking versions are disabled")
  abort "preview workflow no longer rejects stable publication input"
end
publish_run = jobs.fetch("publish-release").fetch("steps").find {
  |step| step["name"] == "Publish channel metadata"
}.fetch("run")
recheck = publish_run.index("live_metadata=")
replacement = publish_run.index("gh release upload channels")
unless recheck && replacement && recheck < replacement
  abort "live index identity must be checked before channels.json replacement"
end
RUBY

python3 - "${REPO_ROOT}/.github/workflows/preview-release.yml" <<'PY'
import pathlib
import sys

text = pathlib.Path(sys.argv[1]).read_text(encoding="utf-8")
required = (
    "source_sha: ${{ steps.validate.outputs.source_sha }}",
    "echo \"source_sha=$(git rev-parse HEAD)\"",
    "ref: ${{ needs.validate.outputs.source_sha }}",
    "ref: ${{ env.SOURCE_SHA }}",
    "EXPECTED_INDEX_GENERATION=",
    "EXPECTED_INDEX_SHA256=",
    "release index changed after proposal generation; refusing stale mutation",
)
for fragment in required:
    if fragment not in text:
        raise SystemExit(f"preview workflow omitted governed publication fragment: {fragment}")
if text.count("ref: ${{ needs.validate.outputs.source_sha }}") != 1:
    raise SystemExit("build jobs must check out the resolved source SHA")
if text.count("ref: ${{ env.SOURCE_SHA }}") != 1:
    raise SystemExit("publish job must check out the resolved source SHA")
metadata_url = "releases/download/channels/channels.json"
if text.count(metadata_url) < 2:
    raise SystemExit("preview workflow must re-fetch the live release index before replacement")
PY
