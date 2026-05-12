#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-create-beta-dry.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

site_repo="${tmp_dir}/site"
make_site_repo_fixture "${site_repo}"

output="$("${REPO_ROOT}/scripts/distribution/create_new_version.sh" \
  --channel beta \
  --version 0.1.0-beta.2 \
  --dry-run \
  --site-repo "${site_repo}" \
  --work-dir "${tmp_dir}/work")"

[[ "${output}" == *"channel=beta"* ]] || { echo "${output}" >&2; exit 1; }
[[ "${output}" == *"version=0.1.0-beta.2"* ]] || { echo "${output}" >&2; exit 1; }
[[ "${output}" == *"new_beta=0.1.0-beta.2"* ]] || { echo "${output}" >&2; exit 1; }
[[ "${output}" == *"github_release=sifr-lang/sifr:0.1.0-beta.2"* ]] || { echo "${output}" >&2; exit 1; }
[[ ! -e "${site_repo}/apps/sifr-site/public/install/versions/0.1.0-beta.2" ]] || exit 1
