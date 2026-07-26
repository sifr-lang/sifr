#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-create-bad-semver.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

site_repo="${tmp_dir}/site"
make_site_repo_fixture "${site_repo}"

require_failure_contains \
  "version must be a semver prerelease using -alpha.N, -beta.N, or -rc.N" \
  "${REPO_ROOT}/scripts/distribution/create_new_version.sh" \
    --channel beta \
    --version 0.1.0-gamma.1 \
    --dry-run \
    --site-repo "${site_repo}" \
    --work-dir "${tmp_dir}/work"

require_failure_contains \
  "belongs to alpha, not beta" \
  "${REPO_ROOT}/scripts/distribution/create_new_version.sh" \
    --channel beta \
    --version 0.1.0-alpha.4 \
    --dry-run \
    --site-repo "${site_repo}" \
    --work-dir "${tmp_dir}/work"
