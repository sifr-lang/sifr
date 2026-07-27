#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-create-drift.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

site_repo="${tmp_dir}/site"
release_index="${tmp_dir}/channels.json"
make_site_repo_fixture "${site_repo}"
make_release_index_fixture "${release_index}"
sed -i.bak 's#https://github.com/sifr-lang/sifr/releases/download/channels/channels.json#https://sifr.sh/install/metadata/channels.json#' \
  "${site_repo}/apps/sifr-site/public/install/index"
rm "${site_repo}/apps/sifr-site/public/install/index.bak"
git -C "${site_repo}" add apps/sifr-site/public/install/index
git -C "${site_repo}" commit -qm "fixture: introduce dispatcher drift"

require_failure_contains \
  "site dispatcher drift: index must resolve channels from GitHub" \
  "${REPO_ROOT}/scripts/distribution/create_new_version.sh" \
    --channel beta \
    --version 0.1.0-beta.6 \
    --dry-run \
    --site-repo "${site_repo}" \
    --release-index "${release_index}"
