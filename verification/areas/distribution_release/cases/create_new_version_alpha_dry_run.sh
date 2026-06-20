#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-create-alpha-dry.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

site_repo="${tmp_dir}/site"
make_site_repo_fixture "${site_repo}"

output="$("${REPO_ROOT}/scripts/distribution/create_new_version.sh" \
  --channel alpha \
  --version 0.1.0-alpha.2 \
  --dry-run \
  --site-repo "${site_repo}" \
  --work-dir "${tmp_dir}/work")"

[[ "${output}" == *"channel=alpha"* ]] || { echo "${output}" >&2; exit 1; }
[[ "${output}" == *"version=0.1.0-alpha.2"* ]] || { echo "${output}" >&2; exit 1; }
[[ "${output}" == *"new_alpha=0.1.0-alpha.2"* ]] || { echo "${output}" >&2; exit 1; }
[[ "${output}" == *"channel_metadata=${tmp_dir}/work/channels.json"* ]] || { echo "${output}" >&2; exit 1; }
[[ "${output}" == *"github_channel_release=sifr-lang/sifr:channels"* ]] || { echo "${output}" >&2; exit 1; }
[[ "${output}" == *"channel_metadata_update=alpha:0.1.0-alpha.2,beta:"* ]] || { echo "${output}" >&2; exit 1; }
[[ "${output}" == *"dry_run_side_effects=none"* ]] || { echo "${output}" >&2; exit 1; }
[[ ! -e "${site_repo}/apps/sifr-site/public/install/versions/0.1.0-alpha.2" ]] || exit 1
