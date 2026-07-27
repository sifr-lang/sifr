#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-create-beta-dry.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

site_repo="${tmp_dir}/site"
release_index="${tmp_dir}/channels.json"
make_site_repo_fixture "${site_repo}"
make_release_index_fixture "${release_index}"
current_beta="$("${REPO_ROOT}/scripts/distribution/read_channel_versions.py" \
  "${release_index}" | awk '{print $2}')"
[[ -n "${current_beta}" ]] || {
  echo "could not read current beta fixture version" >&2
  exit 1
}
next_beta="0.1.0-beta.$((${current_beta##*.} + 1))"

output="$("${REPO_ROOT}/scripts/distribution/create_new_version.sh" \
  --channel beta \
  --version "${next_beta}" \
  --dry-run \
  --site-repo "${site_repo}" \
  --release-index "${release_index}")"

[[ "${output}" == *"channel=beta"* ]] || { echo "${output}" >&2; exit 1; }
[[ "${output}" == *"version=${next_beta}"* ]] || { echo "${output}" >&2; exit 1; }
[[ "${output}" == *"proposed_beta=${next_beta}"* ]] || { echo "${output}" >&2; exit 1; }
[[ "${output}" == *"version_asset_policy=write-once"* ]] || { echo "${output}" >&2; exit 1; }
[[ "${output}" == *"channel_index_policy=replace-only"* ]] || { echo "${output}" >&2; exit 1; }
[[ ! -e "${site_repo}/apps/sifr-site/public/install/versions/${next_beta}" ]] || exit 1
[[ -z "$(git -C "${site_repo}" status --porcelain --untracked-files=all)" ]] || exit 1
