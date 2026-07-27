#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-create-active-site.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

site_repo="${tmp_dir}/site"
release_index="${tmp_dir}/channels.json"
make_site_repo_fixture "${site_repo}" stable
generate_channel_metadata_fixture \
  "${release_index}" \
  "0.1.0-alpha.1" \
  "0.1.0-beta.1" \
  "0.1.0"

output="$("${REPO_ROOT}/scripts/distribution/create_new_version.sh" \
  --channel beta \
  --version 0.1.0-beta.2 \
  --dry-run \
  --site-repo "${site_repo}" \
  --release-index "${release_index}")"

[[ "${output}" == *"ga_status=active"* ]] || { echo "${output}" >&2; exit 1; }
[[ "${output}" == *"site_default_channel=stable"* ]] || { echo "${output}" >&2; exit 1; }
[[ -z "$(git -C "${site_repo}" status --porcelain --untracked-files=all)" ]] || exit 1
