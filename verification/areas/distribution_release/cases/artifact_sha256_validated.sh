#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-artifact-checksum.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

version="0.1.0-beta.8"
binary="${tmp_dir}/sifr"
artifact_dir="${tmp_dir}/artifacts"
installer="${tmp_dir}/installer.sh"
install_root="${tmp_dir}/install"
install_dir="${install_root}/bin"
target="x86_64-unknown-linux-gnu"
archive="${artifact_dir}/sifr-${version}-${target}.tar.gz"

make_mock_binary "${binary}" "new binary"
build_mock_preview_artifacts "${version}" "${artifact_dir}" "${binary}"
"${REPO_ROOT}/scripts/distribution/generate_version_installer.sh" \
  --version "${version}" \
  --artifact-dir "${artifact_dir}" \
  --out "${installer}" \
  --artifact-base-url "file://${artifact_dir}" >/dev/null

mkdir -p "${install_dir}"
mkdir "${install_dir}/sifr"
chmod 755 "${install_dir}/sifr"
touch "${install_dir}/sifr/old-marker"
printf 'corruption' >>"${archive}"

require_failure_contains \
  "checksum mismatch" \
  env SIFR_TARGET="${target}" SIFR_ARTIFACT_BASE_URL="file://${artifact_dir}" SIFR_INSTALL_DIR="${install_dir}" sh "${installer}"

if [[ ! -d "${install_dir}/sifr" || ! -f "${install_dir}/sifr/old-marker" ]]; then
  echo "installer replaced existing binary despite checksum failure" >&2
  exit 1
fi
