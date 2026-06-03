#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-create-real.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

site_repo="${tmp_dir}/site"
binary="${tmp_dir}/sifr"
work_dir="${tmp_dir}/work"
version="0.1.0-beta.3"
make_site_repo_fixture "${site_repo}"
make_mock_binary "${binary}" "create new version real run"

dry_output="$("${REPO_ROOT}/scripts/distribution/create_new_version.sh" \
  --channel beta \
  --version "${version}" \
  --dry-run \
  --site-repo "${site_repo}" \
  --work-dir "${work_dir}" \
  --mutation-mode local)"
dry_sha="$(printf '%s\n' "${dry_output}" | sed -n 's/^plan_sha256=//p')"

real_output="$("${REPO_ROOT}/scripts/distribution/create_new_version.sh" \
  --channel beta \
  --version "${version}" \
  --real-run \
  --site-repo "${site_repo}" \
  --work-dir "${work_dir}" \
  --binary "${binary}" \
  --mutation-mode local)"
real_sha="$(printf '%s\n' "${real_output}" | sed -n 's/^plan_sha256=//p')"

[[ -n "${dry_sha}" && "${dry_sha}" == "${real_sha}" ]] || {
  echo "dry-run and real-run plan sha mismatch" >&2
  echo "${dry_output}" >&2
  echo "${real_output}" >&2
  exit 1
}

grep -q "BETA_VERSION=\"${version}\"" "${site_repo}/apps/sifr-site/public/install/index"
grep -q "\"beta\": \"${version}\"" "${site_repo}/apps/sifr-site/public/install/metadata/channels.json"
grep -q "APP_VERSION=\"${version}\"" "${site_repo}/apps/sifr-site/public/install/versions/${version}"
test -x "${site_repo}/apps/sifr-site/public/install/versions/${version}"
test -f "${work_dir}/plan.txt"
test -f "${work_dir}/release-checklist.md"
test -f "${work_dir}/recovery-note.md"
"${REPO_ROOT}/scripts/distribution/validate_self_update_metadata.sh" \
  --install-root "${site_repo}/apps/sifr-site/public/install"
