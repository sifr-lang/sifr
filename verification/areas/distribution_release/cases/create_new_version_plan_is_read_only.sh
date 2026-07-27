#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-create-real.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

site_repo="${tmp_dir}/site"
release_index="${tmp_dir}/channels.json"
version="0.1.0-beta.9"
make_site_repo_fixture "${site_repo}"
make_release_index_fixture "${release_index}"
site_before="$(git -C "${site_repo}" rev-parse HEAD)"
index_before="$(sha256_fixture_file "${release_index}")"

output="$("${REPO_ROOT}/scripts/distribution/create_new_version.sh" \
  --channel beta \
  --version "${version}" \
  --dry-run \
  --site-repo "${site_repo}" \
  --release-index "${release_index}")"

[[ "${output}" == *"source_commit="* ]] || { echo "${output}" >&2; exit 1; }
[[ "${output}" == *"site_base_commit=${site_before}"* ]] || { echo "${output}" >&2; exit 1; }
[[ "${output}" == *"current_index_sha256=${index_before}"* ]] || { echo "${output}" >&2; exit 1; }
[[ "$(git -C "${site_repo}" rev-parse HEAD)" == "${site_before}" ]] || exit 1
[[ -z "$(git -C "${site_repo}" status --porcelain --untracked-files=all)" ]] || exit 1
[[ "$(sha256_fixture_file "${release_index}")" == "${index_before}" ]] || exit 1
