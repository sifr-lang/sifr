#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-create-checklist.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

site_repo="${tmp_dir}/site"
release_index="${tmp_dir}/channels.json"
make_site_repo_fixture "${site_repo}"
make_release_index_fixture "${release_index}"
printf '%s\n' "uncommitted" >>"${site_repo}/apps/sifr-site/public/install/index"

require_failure_contains \
  "site checkout must be clean" \
  "${REPO_ROOT}/scripts/distribution/create_new_version.sh" \
    --channel alpha \
    --version 0.1.0-alpha.5 \
    --dry-run \
    --site-repo "${site_repo}" \
    --release-index "${release_index}"
