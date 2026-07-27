#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-create-attribution.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

site_repo="${tmp_dir}/site"
release_index="${tmp_dir}/channels.json"
make_site_repo_fixture "${site_repo}"
make_release_index_fixture "${release_index}"

for forbidden in --real-run --mutation-mode --work-dir; do
  require_failure_contains \
    "local mutation and artifact modes are removed" \
    "${REPO_ROOT}/scripts/distribution/create_new_version.sh" \
      --channel beta \
      --version 0.1.0-beta.8 \
      --dry-run \
      --site-repo "${site_repo}" \
      --release-index "${release_index}" \
      "${forbidden}" local
done
[[ -z "$(git -C "${site_repo}" status --porcelain --untracked-files=all)" ]] || exit 1
