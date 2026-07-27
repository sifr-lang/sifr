#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-create-missing-artifact.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

site_repo="${tmp_dir}/site"
artifact_dir="${tmp_dir}/artifacts"
release_index="${tmp_dir}/channels.json"
make_site_repo_fixture "${site_repo}"
make_release_index_fixture "${release_index}"
mkdir -p "${artifact_dir}"

require_failure_contains \
  "local mutation and artifact modes are removed" \
  "${REPO_ROOT}/scripts/distribution/create_new_version.sh" \
    --channel beta \
    --version 0.1.0-beta.10 \
    --dry-run \
    --site-repo "${site_repo}" \
    --release-index "${release_index}" \
    --artifact-dir "${artifact_dir}"
