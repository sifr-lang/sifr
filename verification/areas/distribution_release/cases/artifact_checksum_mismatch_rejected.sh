#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-generator-checksum.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

version="0.1.0-beta.12"
artifact_dir="${tmp_dir}/artifacts"
binary="${tmp_dir}/sifr"
target="x86_64-unknown-linux-gnu"

make_mock_binary "${binary}" "checksum generator fixture"
build_mock_preview_artifacts "${version}" "${artifact_dir}" "${binary}"
printf '0000000000000000000000000000000000000000000000000000000000000000\n' \
  >"${artifact_dir}/sifr-${version}-${target}.tar.gz.sha256"

require_failure_contains \
  "checksum mismatch for sifr-${version}-${target}.tar.gz" \
  "${REPO_ROOT}/scripts/distribution/generate_version_installer.sh" \
    --version "${version}" \
    --artifact-dir "${artifact_dir}" \
    --out "${tmp_dir}/installer.sh"
