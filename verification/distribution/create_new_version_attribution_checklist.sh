#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-create-attribution.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

site_repo="${tmp_dir}/site"
binary="${tmp_dir}/sifr"
work_dir="${tmp_dir}/work"
version="0.1.0-beta.4"
make_site_repo_fixture "${site_repo}"
make_mock_binary "${binary}" "attribution fixture"

"${REPO_ROOT}/scripts/distribution/create_new_version.sh" \
  --channel beta \
  --version "${version}" \
  --real-run \
  --site-repo "${site_repo}" \
  --work-dir "${work_dir}" \
  --binary "${binary}" \
  --mutation-mode local >/dev/null

checklist="${work_dir}/release-checklist.md"
grep -q "uv-derived code used: no" "${checklist}"
grep -q "Copied/adapted uv files: none" "${checklist}"
grep -q "MIT license retention required: not applicable" "${checklist}"
grep -q "Pinned uv source URL/reference required: not applicable" "${checklist}"
