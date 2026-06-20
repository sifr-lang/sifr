#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-create-checklist.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

site_repo="${tmp_dir}/site"
binary="${tmp_dir}/sifr"
work_dir="${tmp_dir}/work"
version="0.1.0-alpha.3"
make_site_repo_fixture "${site_repo}"
make_mock_binary "${binary}" "checklist fixture"

"${REPO_ROOT}/scripts/distribution/create_new_version.sh" \
  --channel alpha \
  --version "${version}" \
  --real-run \
  --site-repo "${site_repo}" \
  --work-dir "${work_dir}" \
  --binary "${binary}" \
  --mutation-mode local >/dev/null

checklist="${work_dir}/release-checklist.md"
grep -q "Channel: alpha" "${checklist}"
grep -q "Version: ${version}" "${checklist}"
grep -q "GitHub immutable installer asset" "${checklist}"
grep -q "Website bootstrap dispatchers resolve channels from GitHub" "${checklist}"
grep -q "Stable entrypoint unchanged and absent" "${checklist}"
grep -q "uv run --project verification --locked python -m sifr_verify areas run --area distribution_release --suite full" "${checklist}"
