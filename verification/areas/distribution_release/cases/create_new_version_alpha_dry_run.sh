#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-create-alpha-dry.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

site_repo="${tmp_dir}/site"
release_index="${tmp_dir}/channels.json"
make_site_repo_fixture "${site_repo}"
make_release_index_fixture "${release_index}"

output="$("${REPO_ROOT}/scripts/distribution/create_new_version.sh" \
  --channel alpha \
  --version 0.1.0-alpha.2 \
  --dry-run \
  --site-repo "${site_repo}" \
  --release-index "${release_index}")"

[[ "${output}" == *"channel=alpha"* ]] || { echo "${output}" >&2; exit 1; }
[[ "${output}" == *"version=0.1.0-alpha.2"* ]] || { echo "${output}" >&2; exit 1; }
[[ "${output}" == *"proposed_alpha=0.1.0-alpha.2"* ]] || { echo "${output}" >&2; exit 1; }
[[ "${output}" == *"proposed_beta=0.1.0-beta.1"* ]] || { echo "${output}" >&2; exit 1; }
[[ "${output}" == *"mutation_authority=.github/workflows/release-publication.yml"* ]] || { echo "${output}" >&2; exit 1; }
[[ "${output}" == *"local_mutations=disabled"* ]] || { echo "${output}" >&2; exit 1; }
[[ "${output}" == *"site_deployment=paired-after-index"* ]] || { echo "${output}" >&2; exit 1; }
[[ ! -e "${site_repo}/apps/sifr-site/public/install/versions/0.1.0-alpha.2" ]] || exit 1
[[ -z "$(git -C "${site_repo}" status --porcelain --untracked-files=all)" ]] || exit 1
