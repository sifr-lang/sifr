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
make_site_repo_fixture "${site_repo}"
mkdir -p "${artifact_dir}"

require_failure_contains \
  "missing artifact" \
  "${REPO_ROOT}/scripts/distribution/create_new_version.sh" \
    --channel beta \
    --version 0.1.0-beta.5 \
    --real-run \
    --site-repo "${site_repo}" \
    --work-dir "${tmp_dir}/work" \
    --artifact-dir "${artifact_dir}" \
    --mutation-mode local
