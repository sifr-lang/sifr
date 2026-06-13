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
current_beta="$(
  sed -n 's/^BETA_VERSION="0\.1\.0-beta\.\([0-9][0-9]*\)"$/\1/p' \
    "${site_repo}/apps/sifr-site/public/install/beta" | head -n 1
)"
[[ -n "${current_beta}" ]] || {
  echo "could not read current beta fixture version" >&2
  exit 1
}
next_beta="0.1.0-beta.$((current_beta + 1))"

output="$("${REPO_ROOT}/scripts/distribution/create_new_version.sh" \
  --channel beta \
  --version "${next_beta}" \
  --dry-run \
  --site-repo "${site_repo}" \
  --work-dir "${tmp_dir}/work")"

[[ "${output}" == *"channel=beta"* ]] || { echo "${output}" >&2; exit 1; }
[[ "${output}" == *"version=${next_beta}"* ]] || { echo "${output}" >&2; exit 1; }
[[ "${output}" == *"new_beta=${next_beta}"* ]] || { echo "${output}" >&2; exit 1; }
[[ "${output}" == *"github_release=sifr-lang/sifr:${next_beta}"* ]] || { echo "${output}" >&2; exit 1; }
[[ ! -e "${site_repo}/apps/sifr-site/public/install/versions/${next_beta}" ]] || exit 1
