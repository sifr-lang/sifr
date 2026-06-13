#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-create-stable.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

site_repo="${tmp_dir}/site"
make_site_repo_fixture "${site_repo}"

require_failure_contains \
  "stable-looking versions are disabled" \
  "${REPO_ROOT}/scripts/distribution/create_new_version.sh" \
    --channel beta \
    --version 1.0.0 \
    --dry-run \
    --site-repo "${site_repo}" \
    --work-dir "${tmp_dir}/work"
