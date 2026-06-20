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

grep -q "\"beta\": \"${version}\"" "${work_dir}/channels.json"
test ! -e "${site_repo}/apps/sifr-site/public/install/metadata/channels.json"
grep -q "APP_VERSION=\"${version}\"" "${work_dir}/sifr-installer-${version}"
grep -q 'CHANNEL_METADATA_URL="https://github.com/sifr-lang/sifr/releases/download/channels/channels.json"' \
  "${site_repo}/apps/sifr-site/public/install/index"
test ! -d "${site_repo}/apps/sifr-site/public/install/versions"
test -f "${work_dir}/plan.txt"
test -f "${work_dir}/channels.json"
test -f "${work_dir}/release-checklist.md"
test -f "${work_dir}/recovery-note.md"
"${REPO_ROOT}/verification/areas/distribution_release/tools/validate_self_update_metadata.sh" \
  --install-root "${site_repo}/apps/sifr-site/public/install" \
  --channels-file "${work_dir}/channels.json"
