#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-create-drift.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

site_repo="${tmp_dir}/site"
make_site_repo_fixture "${site_repo}"
current_beta="$(
  sed -n 's/^BETA_VERSION="\([^"]*\)"$/\1/p' \
    "${site_repo}/apps/sifr-site/public/install/beta" | head -n 1
)"
sed -i.bak "s/BETA_VERSION=\"${current_beta}\"/BETA_VERSION=\"0.1.0-alpha.1\"/" \
  "${site_repo}/apps/sifr-site/public/install/beta"

require_failure_contains \
  "site dispatcher drift: BETA_VERSION differs across dispatchers" \
  "${REPO_ROOT}/scripts/distribution/create_new_version.sh" \
    --channel beta \
    --version 0.1.0-beta.6 \
    --dry-run \
    --site-repo "${site_repo}" \
    --work-dir "${tmp_dir}/work"
